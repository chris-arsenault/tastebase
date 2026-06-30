mod extraction;
mod llm;
mod voice;

use std::sync::Arc;

use ahara_lambda_telemetry::{Operation, OperationKind, TelemetryConfig};
use lambda_runtime::{Error, LambdaEvent, service_fn};
use serde::Deserialize;
use shared::types::ProcessingStatus;
use sqlx::PgPool;
use uuid::Uuid;

const SERVICE_NAME: &str = "tastebase-processing";

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
    telemetry: TelemetryConfig,
    db: PgPool,
    s3: aws_sdk_s3::Client,
    bedrock: aws_sdk_bedrockruntime::Client,
    transcribe: aws_sdk_transcribe::Client,
    media_bucket: String,
    bedrock_model_id: String,
}

fn processing_operation(ctx: &Ctx, name: &'static str) -> Operation {
    Operation::new(ctx.telemetry.clone(), name)
        .with_domain("tastebase.processing")
        .with_kind(OperationKind::Background)
}

async fn handle_recipe_review(payload: &ProcessEvent, ctx: &Ctx) -> Result<(), Error> {
    let review_id = payload
        .review_id
        .ok_or("recipe_review requires review_id")?;
    tracing::info!(review_id = %review_id, "recipe review processing started");

    let result = processing_operation(ctx, "tastebase.processing.recipe_review")
        .with_detail("review.id", review_id.to_string())
        .with_detail("media.voice", payload.voice_key.is_some())
        .observe(async { process_recipe_review(payload, ctx, review_id).await })
        .await;

    if let Err(e) = result {
        tracing::error!(review_id = %review_id, error = %e, "recipe review processing failed");
        update_review_status(
            &ctx.db,
            review_id,
            ProcessingStatus::Error,
            Some(&e.to_string()),
        )
        .await;
    }
    Ok(())
}

async fn handle_tasting(payload: &ProcessEvent, ctx: &Ctx) -> Result<(), Error> {
    let record_id = payload.record_id.ok_or("tasting requires record_id")?;
    tracing::info!(record_id = %record_id, "tasting processing started");

    let media_count = [
        payload.image_key.is_some(),
        payload.ingredients_image_key.is_some(),
        payload.nutrition_image_key.is_some(),
        payload.voice_key.is_some(),
    ]
    .into_iter()
    .filter(|present| *present)
    .count() as u64;

    let result = processing_operation(ctx, "tastebase.processing.tasting")
        .with_detail("tasting.id", record_id.to_string())
        .with_detail("media.count", media_count)
        .with_detail("media.image", payload.image_key.is_some())
        .with_detail(
            "media.ingredients_image",
            payload.ingredients_image_key.is_some(),
        )
        .with_detail(
            "media.nutrition_image",
            payload.nutrition_image_key.is_some(),
        )
        .with_detail("media.voice", payload.voice_key.is_some())
        .with_detail("force_voice", payload.force_voice.unwrap_or(false))
        .observe(async { process_tasting_pipeline(payload, ctx, record_id).await })
        .await;

    if let Err(e) = result {
        tracing::error!(record_id = %record_id, error = %e, "tasting processing failed");
        update_tasting_status(
            &ctx.db,
            record_id,
            ProcessingStatus::Error,
            Some(&e.to_string()),
        )
        .await;
    }
    Ok(())
}

async fn handler(event: LambdaEvent<ProcessEvent>, ctx: &Ctx) -> Result<(), Error> {
    let payload = event.payload;
    match payload.process_type.as_deref().unwrap_or("tasting") {
        "recipe_review" => handle_recipe_review(&payload, ctx).await,
        _ => handle_tasting(&payload, ctx).await,
    }
}

// -- Tasting pipeline (existing) --

async fn run_tasting_image_stages(
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
    Ok(())
}

async fn run_tasting_voice_stage(
    payload: &ProcessEvent,
    ctx: &Ctx,
    record_id: Uuid,
    voice_key: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let force = payload.force_voice.unwrap_or(false);
    let mime = payload.voice_mime_type.as_deref().unwrap_or("audio/webm");

    processing_operation(ctx, "tastebase.processing.tasting.voice_stage")
        .with_detail("tasting.id", record_id.to_string())
        .with_detail("media.mime", mime.to_string())
        .with_detail("force_voice", force)
        .observe(async {
            let transcript = voice::transcribe_voice(ctx, voice_key, mime).await?;
            voice::apply_transcript(&ctx.db, record_id, &transcript, force).await?;
            update_tasting_status(&ctx.db, record_id, ProcessingStatus::VoiceTranscribed, None)
                .await;

            let metrics = extraction::extract_voice_metrics(ctx, &transcript).await?;
            extraction::apply_voice_metrics(&ctx.db, record_id, &metrics, force).await?;
            update_tasting_status(&ctx.db, record_id, ProcessingStatus::VoiceExtracted, None).await;

            let notes_result = voice::format_tasting_notes(ctx, &transcript).await?;
            voice::apply_voice_notes(&ctx.db, record_id, &notes_result, force).await?;
            update_tasting_status(&ctx.db, record_id, ProcessingStatus::NotesFormatted, None).await;
            Ok(())
        })
        .await
}

async fn process_tasting_pipeline(
    payload: &ProcessEvent,
    ctx: &Ctx,
    record_id: Uuid,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    run_tasting_image_stages(payload, ctx, record_id).await?;
    if let Some(ref key) = payload.voice_key {
        run_tasting_voice_stage(payload, ctx, record_id, key).await?;
    }
    update_tasting_status(&ctx.db, record_id, ProcessingStatus::Complete, None).await;
    tracing::info!(record_id = %record_id, "tasting processing complete");
    Ok(())
}

// -- Recipe review pipeline (new, reuses voice transcription + LLM) --

async fn extract_review_score_opt(ctx: &Ctx, review_id: Uuid, transcript: &str) -> Option<i16> {
    match extraction::extract_review_score(ctx, transcript).await {
        Ok(s) => {
            tracing::info!(review_id = %review_id, score = s, "score extracted");
            Some(s)
        }
        Err(e) => {
            tracing::warn!(review_id = %review_id, error = %e, "score extraction failed");
            None
        }
    }
}

async fn write_recipe_review_results(
    db: &PgPool,
    review_id: Uuid,
    transcript: &str,
    notes: &str,
    score: Option<i16>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE recipe_reviews SET voice_transcript = $2, notes = $3, score = $4, status = 'complete', updated_at = now() WHERE id = $1",
    )
    .bind(review_id)
    .bind(transcript)
    .bind(notes)
    .bind(score)
    .execute(db)
    .await?;
    Ok(())
}

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

    let transcript = voice::transcribe_voice(ctx, voice_key, voice_mime).await?;
    update_review_status(&ctx.db, review_id, ProcessingStatus::VoiceTranscribed, None).await;

    let formatted = voice::format_recipe_review(ctx, &transcript).await?;
    update_review_status(&ctx.db, review_id, ProcessingStatus::NotesFormatted, None).await;

    let score = extract_review_score_opt(ctx, review_id, &transcript).await;
    write_recipe_review_results(&ctx.db, review_id, &transcript, &formatted, score).await?;

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
    processing_operation(ctx, "tastebase.processing.tasting.image")
        .with_detail("tasting.id", id.to_string())
        .with_optional_detail("media.mime_hint", mime.map(String::from))
        .observe(async {
            let (base64, content_type) =
                shared::media::download_base64(&ctx.s3, &ctx.media_bucket, key).await?;
            let mime_type = content_type.as_deref().or(mime).unwrap_or("image/jpeg");
            let result = extraction::run_image_extraction(ctx, &base64, mime_type).await?;
            extraction::apply_image_enrichment(&ctx.db, id, &result).await?;
            tracing::info!(record_id = %id, "image extraction complete");
            Ok(())
        })
        .await
}

async fn process_ingredients(
    ctx: &Ctx,
    id: Uuid,
    key: &str,
    mime: Option<&str>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    processing_operation(ctx, "tastebase.processing.tasting.ingredients")
        .with_detail("tasting.id", id.to_string())
        .with_optional_detail("media.mime_hint", mime.map(String::from))
        .observe(async {
            let (base64, content_type) =
                shared::media::download_base64(&ctx.s3, &ctx.media_bucket, key).await?;
            let mime_type = content_type.as_deref().or(mime).unwrap_or("image/jpeg");
            let result = extraction::run_ingredients_extraction(ctx, &base64, mime_type).await?;
            extraction::apply_ingredients_enrichment(&ctx.db, id, &result).await?;
            tracing::info!(record_id = %id, "ingredients extraction complete");
            Ok(())
        })
        .await
}

async fn process_nutrition(
    ctx: &Ctx,
    id: Uuid,
    key: &str,
    mime: Option<&str>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    processing_operation(ctx, "tastebase.processing.tasting.nutrition")
        .with_detail("tasting.id", id.to_string())
        .with_optional_detail("media.mime_hint", mime.map(String::from))
        .observe(async {
            let (base64, content_type) =
                shared::media::download_base64(&ctx.s3, &ctx.media_bucket, key).await?;
            let mime_type = content_type.as_deref().or(mime).unwrap_or("image/jpeg");
            let result = extraction::run_nutrition_extraction(ctx, &base64, mime_type).await?;
            extraction::apply_nutrition_enrichment(&ctx.db, id, &result).await?;
            tracing::info!(record_id = %id, "nutrition extraction complete");
            Ok(())
        })
        .await
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
    let telemetry = shared::telemetry_config(SERVICE_NAME);
    ahara_lambda_telemetry::init_lambda_logging(&telemetry);

    let config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
    let db = shared::db::connect().await;
    let media_bucket = std::env::var("MEDIA_BUCKET").expect("MEDIA_BUCKET required");
    let bedrock_model_id = std::env::var("BEDROCK_MODEL_ID")
        .unwrap_or_else(|_| "us.anthropic.claude-haiku-4-5-20251001-v1:0".into());

    let ctx = Ctx {
        telemetry: telemetry.clone(),
        db,
        s3: aws_sdk_s3::Client::new(&config),
        bedrock: aws_sdk_bedrockruntime::Client::new(&config),
        transcribe: aws_sdk_transcribe::Client::new(&config),
        media_bucket,
        bedrock_model_id,
    };
    let ctx = Arc::new(ctx);

    ahara_lambda_telemetry::run_event_lambda(
        telemetry,
        service_fn(move |event| {
            let ctx = Arc::clone(&ctx);
            async move { handler(event, &ctx).await }
        }),
    )
    .await
}
