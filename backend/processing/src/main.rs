mod extraction;
mod llm;
mod voice;

use lambda_runtime::{service_fn, Error, LambdaEvent};
use serde::Deserialize;
use shared::types::ProcessingStatus;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
struct ProcessEvent {
    record_id: Uuid,
    image_key: Option<String>,
    ingredients_image_key: Option<String>,
    nutrition_image_key: Option<String>,
    voice_key: Option<String>,
    image_mime_type: Option<String>,
    ingredients_image_mime_type: Option<String>,
    nutrition_image_mime_type: Option<String>,
    voice_mime_type: Option<String>,
    force_voice: Option<bool>,
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
    tracing::info!(record_id = %payload.record_id, "processing started");

    if let Err(e) = process_pipeline(&payload, ctx).await {
        tracing::error!(record_id = %payload.record_id, error = %e, "processing failed");
        update_status(&ctx.db, payload.record_id, ProcessingStatus::Error, Some(&e.to_string()))
            .await;
    }

    Ok(())
}

async fn process_pipeline(
    payload: &ProcessEvent,
    ctx: &Ctx,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // -- Image extraction: product type, name, maker, style --
    if let Some(ref key) = payload.image_key {
        process_image(ctx, payload.record_id, key, payload.image_mime_type.as_deref()).await?;
        update_status(
            &ctx.db,
            payload.record_id,
            ProcessingStatus::ImageExtracted,
            None,
        )
        .await;
    }

    // -- Ingredients extraction from label image --
    if let Some(ref key) = payload.ingredients_image_key {
        process_ingredients(
            ctx,
            payload.record_id,
            key,
            payload.ingredients_image_mime_type.as_deref(),
        )
        .await?;
        update_status(
            &ctx.db,
            payload.record_id,
            ProcessingStatus::IngredientsExtracted,
            None,
        )
        .await;
    }

    // -- Nutrition facts extraction from label image --
    if let Some(ref key) = payload.nutrition_image_key {
        process_nutrition(
            ctx,
            payload.record_id,
            key,
            payload.nutrition_image_mime_type.as_deref(),
        )
        .await?;
        update_status(
            &ctx.db,
            payload.record_id,
            ProcessingStatus::NutritionExtracted,
            None,
        )
        .await;
    }

    // -- Voice pipeline: transcribe -> extract metrics -> format notes --
    if let Some(ref key) = payload.voice_key {
        let force = payload.force_voice.unwrap_or(false);

        // Transcribe
        let transcript = voice::transcribe_voice(ctx, key, payload.voice_mime_type.as_deref().unwrap_or("audio/webm")).await?;
        voice::apply_transcript(&ctx.db, payload.record_id, &transcript, force).await?;
        update_status(
            &ctx.db,
            payload.record_id,
            ProcessingStatus::VoiceTranscribed,
            None,
        )
        .await;

        // Extract metrics (score, heat_user)
        let metrics = extraction::extract_voice_metrics(ctx, &transcript).await?;
        extraction::apply_voice_metrics(&ctx.db, payload.record_id, &metrics, force).await?;
        update_status(
            &ctx.db,
            payload.record_id,
            ProcessingStatus::VoiceExtracted,
            None,
        )
        .await;

        // Format tasting notes
        let notes_result = voice::format_tasting_notes(ctx, &transcript).await?;
        voice::apply_voice_notes(&ctx.db, payload.record_id, &notes_result, force).await?;
        update_status(
            &ctx.db,
            payload.record_id,
            ProcessingStatus::NotesFormatted,
            None,
        )
        .await;
    }

    update_status(
        &ctx.db,
        payload.record_id,
        ProcessingStatus::Complete,
        None,
    )
    .await;
    tracing::info!(record_id = %payload.record_id, "processing complete");
    Ok(())
}

// -- Pipeline stage implementations --

async fn process_image(
    ctx: &Ctx,
    id: Uuid,
    key: &str,
    mime: Option<&str>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (base64, content_type) = shared::media::download_base64(&ctx.s3, &ctx.media_bucket, key).await?;
    let mime_type = content_type.as_deref().or(mime).unwrap_or("image/jpeg");

    let result = extraction::run_image_extraction(ctx, &base64, mime_type).await?;
    extraction::apply_image_enrichment(&ctx.db, id, &result).await?;

    tracing::info!(
        record_id = %id,
        product_type = ?result.product_type,
        name = ?result.name,
        maker = ?result.maker,
        style = ?result.style,
        "image extraction complete"
    );
    Ok(())
}

async fn process_ingredients(
    ctx: &Ctx,
    id: Uuid,
    key: &str,
    mime: Option<&str>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (base64, content_type) = shared::media::download_base64(&ctx.s3, &ctx.media_bucket, key).await?;
    let mime_type = content_type.as_deref().or(mime).unwrap_or("image/jpeg");

    let result = extraction::run_ingredients_extraction(ctx, &base64, mime_type).await?;
    extraction::apply_ingredients_enrichment(&ctx.db, id, &result).await?;

    tracing::info!(
        record_id = %id,
        ingredient_count = result.ingredients.as_ref().map_or(0, |v| v.len()),
        "ingredients extraction complete"
    );
    Ok(())
}

async fn process_nutrition(
    ctx: &Ctx,
    id: Uuid,
    key: &str,
    mime: Option<&str>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (base64, content_type) = shared::media::download_base64(&ctx.s3, &ctx.media_bucket, key).await?;
    let mime_type = content_type.as_deref().or(mime).unwrap_or("image/jpeg");

    let result = extraction::run_nutrition_extraction(ctx, &base64, mime_type).await?;
    extraction::apply_nutrition_enrichment(&ctx.db, id, &result).await?;

    tracing::info!(
        record_id = %id,
        has_nutrition = result.nutrition_facts.is_some(),
        "nutrition extraction complete"
    );
    Ok(())
}

// -- Database helpers --

async fn update_status(db: &PgPool, id: Uuid, status: ProcessingStatus, error: Option<&str>) {
    let result = sqlx::query(
        "UPDATE tastings SET status = $2, processing_error = $3, updated_at = now() WHERE id = $1",
    )
    .bind(id)
    .bind(status)
    .bind(error)
    .execute(db)
    .await;

    if let Err(e) = result {
        tracing::error!(record_id = %id, error = %e, "failed to update status");
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    shared::init_tracing();

    let config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
    let db = shared::db::connect().await;
    let media_bucket = std::env::var("MEDIA_BUCKET").expect("MEDIA_BUCKET required");
    let bedrock_model_id = std::env::var("BEDROCK_MODEL_ID")
        .unwrap_or_else(|_| "anthropic.claude-3-haiku-20240307-v1:0".into());

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
