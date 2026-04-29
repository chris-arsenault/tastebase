use axum::extract::{Path, Query, State};
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use serde::Deserialize;
use shared::AppState;
use shared::error::AppError;
use shared::types::{Tasting, TastingPublic};
use uuid::Uuid;

fn public_url_for_key(bucket: &str, key: &str) -> String {
    format!("https://{bucket}.s3.amazonaws.com/{key}")
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ListParams {
    name: Option<String>,
    style: Option<String>,
    min_score: Option<i16>,
    max_score: Option<i16>,
    min_heat: Option<i16>,
    max_heat: Option<i16>,
    date: Option<String>,
}

async fn list_tastings(
    State(state): State<AppState>,
    Query(params): Query<ListParams>,
) -> Result<Json<serde_json::Value>, AppError> {
    tracing::info!("listing tastings");

    let mut sql = String::from("SELECT * FROM tastings WHERE 1=1");
    let mut binds: Vec<String> = Vec::new();
    let mut bind_idx = 1u32;

    if let Some(ref name) = params.name {
        sql.push_str(&format!(" AND name ILIKE ${bind_idx}"));
        binds.push(format!("%{name}%"));
        bind_idx += 1;
    }
    if let Some(ref style) = params.style {
        sql.push_str(&format!(" AND style ILIKE ${bind_idx}"));
        binds.push(format!("%{style}%"));
        bind_idx += 1;
    }
    if let Some(min) = params.min_score {
        sql.push_str(&format!(" AND score >= ${bind_idx}"));
        binds.push(min.to_string());
        bind_idx += 1;
    }
    if let Some(max) = params.max_score {
        sql.push_str(&format!(" AND score <= ${bind_idx}"));
        binds.push(max.to_string());
        bind_idx += 1;
    }
    if let Some(min) = params.min_heat {
        sql.push_str(&format!(" AND heat_user >= ${bind_idx}"));
        binds.push(min.to_string());
        bind_idx += 1;
    }
    if let Some(max) = params.max_heat {
        sql.push_str(&format!(" AND heat_user <= ${bind_idx}"));
        binds.push(max.to_string());
        bind_idx += 1;
    }
    if let Some(ref date) = params.date {
        sql.push_str(&format!(" AND date = ${bind_idx}"));
        binds.push(date.clone());
        bind_idx += 1;
    }
    let _ = bind_idx;

    sql.push_str(" ORDER BY date DESC, created_at DESC");

    let mut query = sqlx::query_as::<_, Tasting>(&sql);
    for b in &binds {
        query = query.bind(b);
    }

    let tastings = query.fetch_all(&state.db).await?;
    let count = tastings.len();
    let public: Vec<TastingPublic> = tastings.into_iter().map(Into::into).collect();
    tracing::info!(count, "tastings listed");
    Ok(Json(serde_json::json!({ "data": public })))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateTastingInput {
    name: Option<String>,
    maker: Option<String>,
    date: Option<String>,
    score: Option<i16>,
    style: Option<String>,
    heat_user: Option<i16>,
    heat_vendor: Option<i16>,
    refreshing: Option<i16>,
    sweet: Option<i16>,
    tasting_notes_user: Option<String>,
    tasting_notes_vendor: Option<String>,
    product_url: Option<String>,
    image_key: Option<String>,
    image_mime_type: Option<String>,
    ingredients_image_key: Option<String>,
    ingredients_image_mime_type: Option<String>,
    nutrition_image_key: Option<String>,
    nutrition_image_mime_type: Option<String>,
    voice_key: Option<String>,
    voice_mime_type: Option<String>,
}

async fn create_tasting(
    State(state): State<AppState>,
    Json(input): Json<CreateTastingInput>,
) -> Result<axum::http::StatusCode, AppError> {
    tracing::info!("creating tasting");

    shared::validate::validate_tasting_input(
        input.name.as_deref(),
        input.maker.as_deref(),
        input.style.as_deref(),
        input.score,
        input.heat_user,
        input.heat_vendor,
        input.refreshing,
        input.sweet,
        input.tasting_notes_user.as_deref(),
        input.tasting_notes_vendor.as_deref(),
        input.product_url.as_deref(),
    )?;

    let name = shared::sanitize::clean_or_empty(input.name.as_deref());
    let maker = shared::sanitize::clean_or_empty(input.maker.as_deref());
    let style = shared::sanitize::clean_or_empty(input.style.as_deref());
    let tasting_notes_user = shared::sanitize::clean_or_empty(input.tasting_notes_user.as_deref());
    let tasting_notes_vendor =
        shared::sanitize::clean_or_empty(input.tasting_notes_vendor.as_deref());
    let product_url = shared::sanitize::clean_or_empty(input.product_url.as_deref());

    let id = Uuid::new_v4();
    let now = time::OffsetDateTime::now_utc();
    let date_str = input.date.as_deref().unwrap_or("");
    let date = time::Date::parse(
        date_str,
        &time::format_description::well_known::Iso8601::DEFAULT,
    )
    .unwrap_or_else(|_| now.date());

    let image_url = input
        .image_key
        .as_deref()
        .map(|k| public_url_for_key(&state.media_bucket, k));
    let ingredients_url = input
        .ingredients_image_key
        .as_deref()
        .map(|k| public_url_for_key(&state.media_bucket, k));
    let nutrition_url = input
        .nutrition_image_key
        .as_deref()
        .map(|k| public_url_for_key(&state.media_bucket, k));

    sqlx::query(
        "INSERT INTO tastings (id, name, maker, date, score, style,
         heat_user, heat_vendor, refreshing, sweet,
         tasting_notes_user, tasting_notes_vendor, product_url,
         image_url, image_key, ingredients_image_url, ingredients_image_key,
         nutrition_image_url, nutrition_image_key, voice_key,
         status, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13,
                 $14, $15, $16, $17, $18, $19, $20, 'pending', $21, $21)",
    )
    .bind(id)
    .bind(&name)
    .bind(&maker)
    .bind(date)
    .bind(input.score)
    .bind(&style)
    .bind(input.heat_user)
    .bind(input.heat_vendor)
    .bind(input.refreshing)
    .bind(input.sweet)
    .bind(&tasting_notes_user)
    .bind(&tasting_notes_vendor)
    .bind(&product_url)
    .bind(image_url.as_deref())
    .bind(input.image_key.as_deref())
    .bind(ingredients_url.as_deref())
    .bind(input.ingredients_image_key.as_deref())
    .bind(nutrition_url.as_deref())
    .bind(input.nutrition_image_key.as_deref())
    .bind(input.voice_key.as_deref())
    .bind(now)
    .execute(&state.db)
    .await?;

    tracing::info!(tasting_id = %id, "tasting created");

    invoke_processing(
        &state.db,
        id,
        false,
        input.image_key.as_deref(),
        input.ingredients_image_key.as_deref(),
        input.nutrition_image_key.as_deref(),
        input.voice_key.as_deref(),
        normalize_mime(input.image_mime_type.as_deref())
            .or_else(|| infer_mime(input.image_key.as_deref())),
        normalize_mime(input.ingredients_image_mime_type.as_deref())
            .or_else(|| infer_mime(input.ingredients_image_key.as_deref())),
        normalize_mime(input.nutrition_image_mime_type.as_deref())
            .or_else(|| infer_mime(input.nutrition_image_key.as_deref())),
        normalize_mime(input.voice_mime_type.as_deref())
            .or_else(|| infer_mime(input.voice_key.as_deref())),
    )
    .await;

    Ok(axum::http::StatusCode::NO_CONTENT)
}

async fn delete_tasting(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<axum::http::StatusCode, AppError> {
    let result = sqlx::query("DELETE FROM tastings WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    tracing::info!(tasting_id = %id, "tasting deleted");
    Ok(axum::http::StatusCode::NO_CONTENT)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpdateMediaInput {
    image_key: Option<String>,
    ingredients_image_key: Option<String>,
    nutrition_image_key: Option<String>,
}

async fn update_media(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateMediaInput>,
) -> Result<Json<serde_json::Value>, AppError> {
    let exists: Option<(Uuid,)> = sqlx::query_as("SELECT id FROM tastings WHERE id = $1")
        .bind(id)
        .fetch_optional(&state.db)
        .await?;
    if exists.is_none() {
        return Err(AppError::NotFound);
    }

    if input.image_key.is_none()
        && input.ingredients_image_key.is_none()
        && input.nutrition_image_key.is_none()
    {
        return Err(AppError::BadRequest("no media provided".into()));
    }

    let image_url = input
        .image_key
        .as_deref()
        .map(|k| public_url_for_key(&state.media_bucket, k));
    let ingredients_url = input
        .ingredients_image_key
        .as_deref()
        .map(|k| public_url_for_key(&state.media_bucket, k));
    let nutrition_url = input
        .nutrition_image_key
        .as_deref()
        .map(|k| public_url_for_key(&state.media_bucket, k));

    let now = time::OffsetDateTime::now_utc();
    sqlx::query(
        "UPDATE tastings SET
            image_url = COALESCE($2, image_url),
            image_key = COALESCE($3, image_key),
            ingredients_image_url = COALESCE($4, ingredients_image_url),
            ingredients_image_key = COALESCE($5, ingredients_image_key),
            nutrition_image_url = COALESCE($6, nutrition_image_url),
            nutrition_image_key = COALESCE($7, nutrition_image_key),
            updated_at = $8
         WHERE id = $1",
    )
    .bind(id)
    .bind(image_url.as_deref())
    .bind(input.image_key.as_deref())
    .bind(ingredients_url.as_deref())
    .bind(input.ingredients_image_key.as_deref())
    .bind(nutrition_url.as_deref())
    .bind(input.nutrition_image_key.as_deref())
    .bind(now)
    .execute(&state.db)
    .await?;

    let updated: Tasting = sqlx::query_as("SELECT * FROM tastings WHERE id = $1")
        .bind(id)
        .fetch_one(&state.db)
        .await?;

    tracing::info!(tasting_id = %id, "media updated");
    let public: TastingPublic = updated.into();
    Ok(Json(serde_json::json!({ "data": public })))
}

// -- Upload URL endpoint: returns presigned S3 PUT URL --

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UploadUrlInput {
    content_type: String,
    upload_type: String, // "image" or "voice"
}

async fn get_upload_url(
    State(state): State<AppState>,
    Json(input): Json<UploadUrlInput>,
) -> Result<Json<serde_json::Value>, AppError> {
    let ext = input
        .content_type
        .split(';')
        .next()
        .unwrap_or(&input.content_type)
        .split('/')
        .nth(1)
        .unwrap_or("bin");
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let prefix = match input.upload_type.as_str() {
        "voice" => "tasting-voice",
        _ => "tasting-images",
    };
    let id = Uuid::new_v4();
    let key = format!("{prefix}/{ts}-{id}.{ext}");

    let (presigned_url, public_url) =
        shared::media::presign_upload(&state.s3, &state.media_bucket, &key, &input.content_type)
            .await?;

    tracing::info!(upload_type = %input.upload_type, key = %key, "presigned URL generated");

    Ok(Json(serde_json::json!({
        "uploadUrl": presigned_url,
        "key": key,
        "publicUrl": public_url
    })))
}

async fn rerun_processing(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<axum::http::StatusCode, AppError> {
    let tasting: Option<Tasting> = sqlx::query_as("SELECT * FROM tastings WHERE id = $1")
        .bind(id)
        .fetch_optional(&state.db)
        .await?;

    let tasting = tasting.ok_or(AppError::NotFound)?;
    let has_media = tasting.image_key.is_some()
        || tasting.ingredients_image_key.is_some()
        || tasting.nutrition_image_key.is_some()
        || tasting.voice_key.is_some();

    if !has_media {
        return Err(AppError::BadRequest("no media available to process".into()));
    }

    sqlx::query("UPDATE tastings SET status = 'pending', updated_at = now() WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await?;

    tracing::info!(tasting_id = %id, "rerun processing requested");

    invoke_processing(
        &state.db,
        id,
        true,
        tasting.image_key.as_deref(),
        tasting.ingredients_image_key.as_deref(),
        tasting.nutrition_image_key.as_deref(),
        tasting.voice_key.as_deref(),
        infer_mime(tasting.image_key.as_deref()),
        infer_mime(tasting.ingredients_image_key.as_deref()),
        infer_mime(tasting.nutrition_image_key.as_deref()),
        infer_mime(tasting.voice_key.as_deref()),
    )
    .await;

    Ok(axum::http::StatusCode::NO_CONTENT)
}

// -- Helpers --

fn normalize_mime(mime: Option<&str>) -> Option<String> {
    mime.map(|s| s.split(';').next().unwrap_or(s).trim().to_string())
        .filter(|s| !s.is_empty())
}

fn infer_mime(key: Option<&str>) -> Option<String> {
    let ext = key?.rsplit('.').next()?;
    match ext {
        "jpg" | "jpeg" => Some("image/jpeg".into()),
        "png" => Some("image/png".into()),
        "webp" => Some("image/webp".into()),
        "gif" => Some("image/gif".into()),
        "webm" => Some("audio/webm".into()),
        "mp3" => Some("audio/mpeg".into()),
        "wav" => Some("audio/wav".into()),
        "ogg" => Some("audio/ogg".into()),
        "m4a" | "mp4" => Some("audio/mp4".into()),
        "flac" => Some("audio/flac".into()),
        _ => None,
    }
}

#[allow(clippy::too_many_arguments)]
async fn invoke_processing(
    db: &sqlx::PgPool,
    record_id: Uuid,
    force_voice: bool,
    image_key: Option<&str>,
    ingredients_image_key: Option<&str>,
    nutrition_image_key: Option<&str>,
    voice_key: Option<&str>,
    image_mime_type: Option<String>,
    ingredients_image_mime_type: Option<String>,
    nutrition_image_mime_type: Option<String>,
    voice_mime_type: Option<String>,
) {
    let function_name = match std::env::var("PROCESSING_FUNCTION_NAME") {
        Ok(name) if !name.is_empty() => name,
        _ => {
            tracing::warn!(record_id = %record_id, "PROCESSING_FUNCTION_NAME not set, skipping");
            return;
        }
    };

    let payload = serde_json::json!({
        "record_id": record_id.to_string(),
        "image_key": image_key,
        "ingredients_image_key": ingredients_image_key,
        "nutrition_image_key": nutrition_image_key,
        "voice_key": voice_key,
        "image_mime_type": image_mime_type,
        "ingredients_image_mime_type": ingredients_image_mime_type,
        "nutrition_image_mime_type": nutrition_image_mime_type,
        "voice_mime_type": voice_mime_type,
        "force_voice": force_voice,
    });

    let config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
    let lambda_client = aws_sdk_lambda::Client::new(&config);

    let result = lambda_client
        .invoke()
        .function_name(&function_name)
        .invocation_type(aws_sdk_lambda::types::InvocationType::Event)
        .payload(aws_sdk_lambda::primitives::Blob::new(
            serde_json::to_vec(&payload).unwrap_or_default(),
        ))
        .send()
        .await;

    match result {
        Ok(resp) => {
            tracing::info!(
                record_id = %record_id,
                status_code = resp.status_code(),
                "processing Lambda invoked"
            );
        }
        Err(e) => {
            tracing::error!(record_id = %record_id, error = %e, "failed to invoke processing Lambda");
            let _ = sqlx::query(
                "UPDATE tastings SET status = 'error', processing_error = $2, updated_at = now() WHERE id = $1"
            )
            .bind(record_id)
            .bind(format!("Failed to invoke processing: {e}"))
            .execute(db)
            .await;
        }
    }
}

fn router(state: AppState) -> Router {
    Router::new()
        .route("/tastings", get(list_tastings).post(create_tasting))
        .route("/tastings/upload-url", post(get_upload_url))
        .route("/tastings/{id}", delete(delete_tasting))
        .route("/tastings/{id}/media", post(update_media))
        .route("/tastings/{id}/rerun", post(rerun_processing))
        .layer(shared::cors::layer())
        .with_state(state)
}

#[tokio::main]
async fn main() -> Result<(), lambda_http::Error> {
    shared::init_tracing();
    tracing::info!("tastings-api starting");
    let state = AppState::from_env().await;
    let app = router(state);
    lambda_http::run(app).await
}
