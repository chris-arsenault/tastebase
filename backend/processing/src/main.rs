mod extraction;
mod llm;
mod voice;

use lambda_runtime::{Error, LambdaEvent, service_fn};
use serde::Deserialize;
use shared::types::ProcessingStatus;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
struct ProcessEvent {
    // Common
    process_type: Option<String>, // "tasting" (default) or "recipe_review"

    // Tasting fields
    record_id: Option<Uuid>,
    image_key: Option<String>,
    ingredients_image_key: Option<String>,
    nutrition_image_key: Option<String>,
    image_mime_type: Option<String>,
    ingredients_image_mime_type: Option<String>,
    nutrition_image_mime_type: Option<String>,
    force_voice: Option<bool>,

    // Shared
    voice_key: Option<String>,
    voice_mime_type: Option<String>,

    // Recipe review fields
    #[allow(dead_code)]
    recipe_id: Option<Uuid>,
    review_id: Option<Uuid>,
}

pub struct Ctx {
    db: PgPool,
    s3: aws_sdk_s3::Client,
    bedrock: aws_sdk_bedrockruntime::Client,
    transcribe: aws_sdk_transcribe::Client,
    media_bucket: String,
    bedrock_model_id: String,
}

async fn handler(event: LambdaEvent<ProcessEvent>, ctx: &Ctx) -> Result<(), Error> {
    let payload = event.payload;
    let process_type = payload.process_type.as_deref().unwrap_or("tasting");

    match process_type {
        "recipe_review" => {
            let review_id = payload
                .review_id
                .ok_or("recipe_review requires review_id")?;
            tracing::info!(review_id = %review_id, "recipe review processing started");

            if let Err(e) = process_recipe_review(&payload, ctx, review_id).await {
                tracing::error!(review_id = %review_id, error = %e, "recipe review processing failed");
                update_review_status(
                    &ctx.db,
                    review_id,
                    ProcessingStatus::Error,
                    Some(&e.to_string()),
                )
                .await;
            }
        }
        _ => {
            let record_id = payload.record_id.ok_or("tasting requires record_id")?;
            tracing::info!(record_id = %record_id, "tasting processing started");

            if let Err(e) = process_tasting_pipeline(&payload, ctx, record_id).await {
                tracing::error!(record_id = %record_id, error = %e, "tasting processing failed");
                update_tasting_status(
                    &ctx.db,
                    record_id,
                    ProcessingStatus::Error,
                    Some(&e.to_string()),
                )
                .await;
            }
        }
    }

    Ok(())
}

// -- Tasting pipeline (existing) --

async fn process_tasting_pipeline(
    payload: &ProcessEvent,
    ctx: &Ctx,
    record_id: Uuid,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if let Some(ref key) = payload.image_key {
        process_image(ctx, record_id, key, payload.image_mime_type.as_deref()).await?;
        update_tasting_status(&ctx.db, record_id, ProcessingStatus::ImageExtracted, None).await;
    }

    if let Some(ref key) = payload.ingredients_image_key {
        process_ingredients(
            ctx,
            record_id,
            key,
            payload.ingredients_image_mime_type.as_deref(),
        )
        .await?;
        update_tasting_status(
            &ctx.db,
            record_id,
            ProcessingStatus::IngredientsExtracted,
            None,
        )
        .await;
    }

    if let Some(ref key) = payload.nutrition_image_key {
        process_nutrition(
            ctx,
            record_id,
            key,
            payload.nutrition_image_mime_type.as_deref(),
        )
        .await?;
        update_tasting_status(
            &ctx.db,
            record_id,
            ProcessingStatus::NutritionExtracted,
            None,
        )
        .await;
    }

    if let Some(ref key) = payload.voice_key {
        let force = payload.force_voice.unwrap_or(false);

        let transcript = voice::transcribe_voice(
            ctx,
            key,
            payload.voice_mime_type.as_deref().unwrap_or("audio/webm"),
        )
        .await?;
        voice::apply_transcript(&ctx.db, record_id, &transcript, force).await?;
        update_tasting_status(&ctx.db, record_id, ProcessingStatus::VoiceTranscribed, None).await;

        let metrics = extraction::extract_voice_metrics(ctx, &transcript).await?;
        extraction::apply_voice_metrics(&ctx.db, record_id, &metrics, force).await?;
        update_tasting_status(&ctx.db, record_id, ProcessingStatus::VoiceExtracted, None).await;

        let notes_result = voice::format_tasting_notes(ctx, &transcript).await?;
        voice::apply_voice_notes(&ctx.db, record_id, &notes_result, force).await?;
        update_tasting_status(&ctx.db, record_id, ProcessingStatus::NotesFormatted, None).await;
    }

    update_tasting_status(&ctx.db, record_id, ProcessingStatus::Complete, None).await;
    tracing::info!(record_id = %record_id, "tasting processing complete");
    Ok(())
}

// -- Recipe review pipeline (new, reuses voice transcription + LLM) --

async fn process_recipe_review(
    payload: &ProcessEvent,
    ctx: &Ctx,
    review_id: Uuid,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let voice_key = payload
        .voice_key
        .as_deref()
        .ok_or("recipe_review requires voice_key")?;
    let voice_mime = payload.voice_mime_type.as_deref().unwrap_or("audio/webm");

    // Transcribe — reuses the same function as tastings
    let transcript = voice::transcribe_voice(ctx, voice_key, voice_mime).await?;
    update_review_status(&ctx.db, review_id, ProcessingStatus::VoiceTranscribed, None).await;

    // Format review — uses a review-specific prompt
    let formatted = voice::format_recipe_review(ctx, &transcript).await?;
    update_review_status(&ctx.db, review_id, ProcessingStatus::NotesFormatted, None).await;

    // Extract score
    let score = match extraction::extract_review_score(ctx, &transcript).await {
        Ok(s) => {
            tracing::info!(review_id = %review_id, score = s, "score extracted");
            Some(s)
        }
        Err(e) => {
            tracing::warn!(review_id = %review_id, error = %e, "score extraction failed");
            None
        }
    };

    // Write results
    sqlx::query(
        "UPDATE recipe_reviews SET voice_transcript = $2, notes = $3, score = $4, status = 'complete', updated_at = now() WHERE id = $1",
    )
    .bind(review_id)
    .bind(&transcript)
    .bind(&formatted)
    .bind(score)
    .execute(&ctx.db)
    .await?;

    tracing::info!(review_id = %review_id, "recipe review processing complete");
    Ok(())
}

// -- Tasting pipeline stages (unchanged) --

async fn process_image(
    ctx: &Ctx,
    id: Uuid,
    key: &str,
    mime: Option<&str>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (base64, content_type) =
        shared::media::download_base64(&ctx.s3, &ctx.media_bucket, key).await?;
    let mime_type = content_type.as_deref().or(mime).unwrap_or("image/jpeg");
    let result = extraction::run_image_extraction(ctx, &base64, mime_type).await?;
    extraction::apply_image_enrichment(&ctx.db, id, &result).await?;
    tracing::info!(record_id = %id, "image extraction complete");
    Ok(())
}

async fn process_ingredients(
    ctx: &Ctx,
    id: Uuid,
    key: &str,
    mime: Option<&str>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (base64, content_type) =
        shared::media::download_base64(&ctx.s3, &ctx.media_bucket, key).await?;
    let mime_type = content_type.as_deref().or(mime).unwrap_or("image/jpeg");
    let result = extraction::run_ingredients_extraction(ctx, &base64, mime_type).await?;
    extraction::apply_ingredients_enrichment(&ctx.db, id, &result).await?;
    tracing::info!(record_id = %id, "ingredients extraction complete");
    Ok(())
}

async fn process_nutrition(
    ctx: &Ctx,
    id: Uuid,
    key: &str,
    mime: Option<&str>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (base64, content_type) =
        shared::media::download_base64(&ctx.s3, &ctx.media_bucket, key).await?;
    let mime_type = content_type.as_deref().or(mime).unwrap_or("image/jpeg");
    let result = extraction::run_nutrition_extraction(ctx, &base64, mime_type).await?;
    extraction::apply_nutrition_enrichment(&ctx.db, id, &result).await?;
    tracing::info!(record_id = %id, "nutrition extraction complete");
    Ok(())
}

// -- Database helpers --

async fn update_tasting_status(
    db: &PgPool,
    id: Uuid,
    status: ProcessingStatus,
    error: Option<&str>,
) {
    let result = sqlx::query(
        "UPDATE tastings SET status = $2, processing_error = $3, updated_at = now() WHERE id = $1",
    )
    .bind(id)
    .bind(status)
    .bind(error)
    .execute(db)
    .await;

    if let Err(e) = result {
        tracing::error!(record_id = %id, error = %e, "failed to update tasting status");
    }
}

async fn update_review_status(
    db: &PgPool,
    id: Uuid,
    status: ProcessingStatus,
    error: Option<&str>,
) {
    let result = sqlx::query(
        "UPDATE recipe_reviews SET status = $2, processing_error = $3, updated_at = now() WHERE id = $1",
    )
    .bind(id)
    .bind(status)
    .bind(error)
    .execute(db)
    .await;

    if let Err(e) = result {
        tracing::error!(review_id = %id, error = %e, "failed to update review status");
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    shared::init_tracing();

    let config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
    let db = shared::db::connect().await;
    let media_bucket = std::env::var("MEDIA_BUCKET").expect("MEDIA_BUCKET required");
    let bedrock_model_id = std::env::var("BEDROCK_MODEL_ID")
        .unwrap_or_else(|_| "us.anthropic.claude-haiku-4-5-20251001-v1:0".into());

    let ctx = Ctx {
        db,
        s3: aws_sdk_s3::Client::new(&config),
        bedrock: aws_sdk_bedrockruntime::Client::new(&config),
        transcribe: aws_sdk_transcribe::Client::new(&config),
        media_bucket,
        bedrock_model_id,
    };

    lambda_runtime::run(service_fn(|event| handler(event, &ctx))).await
}
