use axum::extract::{Path, Query, State};
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use serde::Deserialize;
use shared::auth::RequireAuth;
use shared::error::AppError;
use shared::types::{Tasting, TastingPublic};
use shared::AppState;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
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
    // Build dynamic query with filters
    let mut sql = String::from(
        "SELECT * FROM tastings WHERE 1=1"
    );
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
    let public: Vec<TastingPublic> = tastings.into_iter().map(Into::into).collect();
    Ok(Json(serde_json::json!({ "data": public })))
}

#[derive(Debug, Deserialize)]
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
    image_base64: Option<String>,
    image_mime_type: Option<String>,
    ingredients_image_base64: Option<String>,
    ingredients_image_mime_type: Option<String>,
    nutrition_image_base64: Option<String>,
    nutrition_image_mime_type: Option<String>,
    voice_base64: Option<String>,
    voice_mime_type: Option<String>,
}

async fn create_tasting(
    State(state): State<AppState>,
    RequireAuth(user): RequireAuth,
    Json(input): Json<CreateTastingInput>,
) -> Result<axum::http::StatusCode, AppError> {
    let user_id = shared::db::resolve_user(&state.db, &user.sub, user.email.as_deref()).await?;
    let id = Uuid::new_v4();
    let now = time::OffsetDateTime::now_utc();
    let date_str = input.date.as_deref().unwrap_or("");
    let date = time::Date::parse(
        date_str,
        &time::format_description::well_known::Iso8601::DEFAULT,
    )
    .unwrap_or_else(|_| now.date());

    // Upload media to S3
    let image_key = upload_if_present(&state, &input.image_base64, &input.image_mime_type, &format!("images/{id}"), "jpg").await?;
    let ingredients_key = upload_if_present(&state, &input.ingredients_image_base64, &input.ingredients_image_mime_type, &format!("images/{id}-ingredients"), "jpg").await?;
    let nutrition_key = upload_if_present(&state, &input.nutrition_image_base64, &input.nutrition_image_mime_type, &format!("images/{id}-nutrition"), "jpg").await?;
    let voice_key = upload_if_present(&state, &input.voice_base64, &input.voice_mime_type, &format!("voice/{id}"), "webm").await?;

    sqlx::query(
        "INSERT INTO tastings (id, user_id, name, maker, date, score, style,
         heat_user, heat_vendor, refreshing, sweet,
         tasting_notes_user, tasting_notes_vendor, product_url,
         image_url, image_key, ingredients_image_url, ingredients_image_key,
         nutrition_image_url, nutrition_image_key, voice_key,
         status, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14,
                 $15, $16, $17, $18, $19, $20, $21, 'pending', $22, $22)"
    )
    .bind(id)
    .bind(user_id)
    .bind(input.name.as_deref().unwrap_or(""))
    .bind(input.maker.as_deref().unwrap_or(""))
    .bind(date)
    .bind(input.score)
    .bind(input.style.as_deref().unwrap_or(""))
    .bind(input.heat_user)
    .bind(input.heat_vendor)
    .bind(input.refreshing)
    .bind(input.sweet)
    .bind(input.tasting_notes_user.as_deref().unwrap_or(""))
    .bind(input.tasting_notes_vendor.as_deref().unwrap_or(""))
    .bind(input.product_url.as_deref().unwrap_or(""))
    .bind(image_key.as_ref().map(|(url, _)| url.as_str()))
    .bind(image_key.as_ref().map(|(_, key)| key.as_str()))
    .bind(ingredients_key.as_ref().map(|(url, _)| url.as_str()))
    .bind(ingredients_key.as_ref().map(|(_, key)| key.as_str()))
    .bind(nutrition_key.as_ref().map(|(url, _)| url.as_str()))
    .bind(nutrition_key.as_ref().map(|(_, key)| key.as_str()))
    .bind(voice_key.as_ref().map(|(_, key)| key.as_str()))
    .bind(now)
    .execute(&state.db)
    .await?;

    // Queue async processing
    invoke_processing(id, &image_key, &ingredients_key, &nutrition_key, &voice_key).await;

    Ok(axum::http::StatusCode::NO_CONTENT)
}

async fn delete_tasting(
    State(state): State<AppState>,
    RequireAuth(user): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<axum::http::StatusCode, AppError> {
    let user_id = shared::db::resolve_user(&state.db, &user.sub, user.email.as_deref()).await?;
    let result = sqlx::query("DELETE FROM tastings WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(user_id)
        .execute(&state.db)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    Ok(axum::http::StatusCode::NO_CONTENT)
}

async fn update_media(
    State(state): State<AppState>,
    RequireAuth(user): RequireAuth,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateMediaInput>,
) -> Result<Json<serde_json::Value>, AppError> {
    let user_id = shared::db::resolve_user(&state.db, &user.sub, user.email.as_deref()).await?;
    let tasting: Option<Tasting> = sqlx::query_as("SELECT * FROM tastings WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(user_id)
        .fetch_optional(&state.db)
        .await?;

    let tasting = tasting.ok_or(AppError::NotFound)?;

    let image_key = upload_if_present(&state, &input.image_base64, &input.image_mime_type, &format!("images/{}", tasting.id), "jpg").await?;
    let ingredients_key = upload_if_present(&state, &input.ingredients_image_base64, &input.ingredients_image_mime_type, &format!("images/{}-ingredients", tasting.id), "jpg").await?;
    let nutrition_key = upload_if_present(&state, &input.nutrition_image_base64, &input.nutrition_image_mime_type, &format!("images/{}-nutrition", tasting.id), "jpg").await?;

    if image_key.is_none() && ingredients_key.is_none() && nutrition_key.is_none() {
        return Err(AppError::BadRequest("no media provided".into()));
    }

    // Update whichever media fields were provided
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
         WHERE id = $1"
    )
    .bind(id)
    .bind(image_key.as_ref().map(|(url, _)| url.as_str()))
    .bind(image_key.as_ref().map(|(_, key)| key.as_str()))
    .bind(ingredients_key.as_ref().map(|(url, _)| url.as_str()))
    .bind(ingredients_key.as_ref().map(|(_, key)| key.as_str()))
    .bind(nutrition_key.as_ref().map(|(url, _)| url.as_str()))
    .bind(nutrition_key.as_ref().map(|(_, key)| key.as_str()))
    .bind(now)
    .execute(&state.db)
    .await?;

    let updated: Tasting = sqlx::query_as("SELECT * FROM tastings WHERE id = $1")
        .bind(id)
        .fetch_one(&state.db)
        .await?;

    let public: TastingPublic = updated.into();
    Ok(Json(serde_json::json!({ "data": public })))
}

async fn rerun_processing(
    State(state): State<AppState>,
    RequireAuth(user): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<axum::http::StatusCode, AppError> {
    let user_id = shared::db::resolve_user(&state.db, &user.sub, user.email.as_deref()).await?;
    let tasting: Option<Tasting> = sqlx::query_as("SELECT * FROM tastings WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(user_id)
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

    let image_key = tasting.image_key.map(|k| (String::new(), k));
    let ingredients_key = tasting.ingredients_image_key.map(|k| (String::new(), k));
    let nutrition_key = tasting.nutrition_image_key.map(|k| (String::new(), k));
    let voice_key = tasting.voice_key.map(|k| (String::new(), k));
    invoke_processing(id, &image_key, &ingredients_key, &nutrition_key, &voice_key).await;

    Ok(axum::http::StatusCode::NO_CONTENT)
}

// -- Helpers --

#[derive(Debug, Deserialize)]
struct UpdateMediaInput {
    image_base64: Option<String>,
    image_mime_type: Option<String>,
    ingredients_image_base64: Option<String>,
    ingredients_image_mime_type: Option<String>,
    nutrition_image_base64: Option<String>,
    nutrition_image_mime_type: Option<String>,
}

async fn upload_if_present(
    state: &AppState,
    data: &Option<String>,
    mime: &Option<String>,
    prefix: &str,
    default_ext: &str,
) -> Result<Option<(String, String)>, AppError> {
    let data = match data {
        Some(d) if !d.is_empty() => d,
        _ => return Ok(None),
    };
    let fallback_mime = mime.as_deref();
    let (bytes, content_type) = shared::media::parse_base64_payload(data, fallback_mime)
        .ok_or_else(|| AppError::BadRequest("invalid base64 media".into()))?;

    let ext = content_type.split('/').nth(1).unwrap_or(default_ext);
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let key = format!("{prefix}-{ts}.{ext}");
    let url = shared::media::upload(&state.s3, &state.media_bucket, &key, bytes, &content_type).await?;
    Ok(Some((url, key)))
}

async fn invoke_processing(
    record_id: Uuid,
    image_key: &Option<(String, String)>,
    ingredients_key: &Option<(String, String)>,
    nutrition_key: &Option<(String, String)>,
    voice_key: &Option<(String, String)>,
) {
    let function_name = match std::env::var("PROCESSING_FUNCTION_NAME") {
        Ok(name) if !name.is_empty() => name,
        _ => {
            tracing::warn!(record_id = %record_id, "PROCESSING_FUNCTION_NAME not set, skipping invocation");
            return;
        }
    };

    let payload = serde_json::json!({
        "record_id": record_id.to_string(),
        "image_key": image_key.as_ref().map(|(_, key)| key),
        "ingredients_image_key": ingredients_key.as_ref().map(|(_, key)| key),
        "nutrition_image_key": nutrition_key.as_ref().map(|(_, key)| key),
        "voice_key": voice_key.as_ref().map(|(_, key)| key),
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
                "processing Lambda invoked asynchronously"
            );
        }
        Err(e) => {
            tracing::error!(
                record_id = %record_id,
                error = %e,
                "failed to invoke processing Lambda"
            );
        }
    }
}

fn router(state: AppState) -> Router {
    Router::new()
        .route("/tastings", get(list_tastings).post(create_tasting))
        .route("/tastings/{id}", delete(delete_tasting))
        .route("/tastings/{id}/media", post(update_media))
        .route("/tastings/{id}/rerun", post(rerun_processing))
        .with_state(state)
}

#[tokio::main]
async fn main() -> Result<(), lambda_http::Error> {
    shared::init_tracing();
    let state = AppState::from_env().await;
    let app = router(state);
    lambda_http::run(app).await
}
