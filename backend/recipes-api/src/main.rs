use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use shared::AppState;
use shared::error::AppError;
use shared::types::{
    Recipe, RecipeFull, RecipeImage, RecipeIngredient, RecipeReview, RecipeSource, RecipeStep,
    RecipeWithThumb, UnitType,
};
use uuid::Uuid;

async fn list_recipes(State(state): State<AppState>) -> Result<Json<serde_json::Value>, AppError> {
    let rows: Vec<RecipeWithThumb> = sqlx::query_as(
        "SELECT r.*, ri.image_url AS thumbnail_url, rv.score AS latest_score
         FROM recipes r
         LEFT JOIN LATERAL (
           SELECT image_url FROM recipe_images WHERE recipe_id = r.id ORDER BY created_at DESC LIMIT 1
         ) ri ON true
         LEFT JOIN LATERAL (
           SELECT score FROM recipe_reviews WHERE recipe_id = r.id AND status = 'complete' ORDER BY created_at DESC LIMIT 1
         ) rv ON true
         ORDER BY r.created_at DESC",
    )
    .fetch_all(&state.db)
    .await?;

    tracing::info!(count = rows.len(), "recipes listed");
    Ok(Json(serde_json::json!({ "data": rows })))
}

async fn get_recipe(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let recipe: Recipe = sqlx::query_as("SELECT * FROM recipes WHERE id = $1")
        .bind(id)
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::NotFound)?;

    let ingredients: Vec<RecipeIngredient> =
        sqlx::query_as("SELECT * FROM recipe_ingredients WHERE recipe_id = $1 ORDER BY sort_order")
            .bind(id)
            .fetch_all(&state.db)
            .await?;

    let steps: Vec<RecipeStep> =
        sqlx::query_as("SELECT * FROM recipe_steps WHERE recipe_id = $1 ORDER BY sort_order")
            .bind(id)
            .fetch_all(&state.db)
            .await?;

    let reviews: Vec<RecipeReview> = sqlx::query_as(
        "SELECT * FROM recipe_reviews WHERE recipe_id = $1 ORDER BY created_at DESC",
    )
    .bind(id)
    .fetch_all(&state.db)
    .await?;

    let images: Vec<RecipeImage> =
        sqlx::query_as("SELECT * FROM recipe_images WHERE recipe_id = $1 ORDER BY created_at")
            .bind(id)
            .fetch_all(&state.db)
            .await?;

    tracing::info!(recipe_id = %id, ingredients = ingredients.len(), steps = steps.len(), reviews = reviews.len(), images = images.len(), "recipe fetched");
    let full = RecipeFull {
        recipe,
        ingredients,
        steps,
        reviews,
        images,
    };
    Ok(Json(serde_json::json!({ "data": full })))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateRecipeInput {
    title: String,
    description: Option<String>,
    base_servings: i32,
    notes: Option<String>,
    source: Option<RecipeSource>,
    source_meta: Option<serde_json::Value>,
    ingredients: Vec<IngredientInput>,
    steps: Vec<StepInput>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct IngredientInput {
    id: String,
    name: String,
    short_name: Option<String>,
    amount: f64,
    unit: Option<UnitType>,
    linked_recipe_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct StepInput {
    id: String,
    title: String,
    content: String,
    timer_seconds: Option<i32>,
}

async fn create_recipe(
    State(state): State<AppState>,
    Json(input): Json<CreateRecipeInput>,
) -> Result<Json<serde_json::Value>, AppError> {
    tracing::info!(title = %input.title, "creating recipe");

    shared::validate::validate_recipe_input(
        &input.title,
        input.description.as_deref(),
        input.base_servings,
        input.notes.as_deref(),
    )?;

    let title = shared::sanitize::clean(&input.title);
    let description = shared::sanitize::clean_option(input.description.as_deref());
    let notes = shared::sanitize::clean_option(input.notes.as_deref());

    let recipe_id = Uuid::new_v4();
    let now = time::OffsetDateTime::now_utc();
    let source = input.source.unwrap_or(RecipeSource::Manual);

    let mut tx = state.db.begin().await?;

    sqlx::query(
        "INSERT INTO recipes (id, title, description, base_servings, notes, source, source_meta, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $8)",
    )
    .bind(recipe_id)
    .bind(&title)
    .bind(&description)
    .bind(input.base_servings)
    .bind(&notes)
    .bind(source)
    .bind(input.source_meta.as_ref().map(sqlx::types::Json))
    .bind(now)
    .execute(&mut *tx)
    .await?;

    for (i, ing) in input.ingredients.iter().enumerate() {
        let unit = ing.unit.unwrap_or(UnitType::None);
        let name = shared::sanitize::clean(&ing.name);
        let short_name = shared::sanitize::clean_or_empty(ing.short_name.as_deref());
        sqlx::query(
            "INSERT INTO recipe_ingredients (recipe_id, widget_id, name, short_name, amount, unit, sort_order, linked_recipe_id)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
        )
        .bind(recipe_id)
        .bind(&ing.id)
        .bind(&name)
        .bind(&short_name)
        .bind(rust_decimal::Decimal::try_from(ing.amount).unwrap_or_default())
        .bind(unit)
        .bind(i as i32)
        .bind(ing.linked_recipe_id)
        .execute(&mut *tx)
        .await?;
    }

    for (i, step) in input.steps.iter().enumerate() {
        let step_title = shared::sanitize::clean(&step.title);
        let content = shared::sanitize::clean(&step.content);
        sqlx::query(
            "INSERT INTO recipe_steps (recipe_id, widget_id, title, content, timer_seconds, sort_order)
             VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(recipe_id)
        .bind(&step.id)
        .bind(&step_title)
        .bind(&content)
        .bind(step.timer_seconds)
        .bind(i as i32)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    tracing::info!(recipe_id = %recipe_id, source = ?source, "recipe created");

    let url = format!("https://tastebase.ahara.io/recipes/{recipe_id}");
    Ok(Json(serde_json::json!({
        "recipe_id": recipe_id,
        "url": url,
        "message": "Saved. View at the link above."
    })))
}

async fn delete_recipe(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<axum::http::StatusCode, AppError> {
    let result = sqlx::query("DELETE FROM recipes WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    tracing::info!(recipe_id = %id, "recipe deleted");
    Ok(axum::http::StatusCode::NO_CONTENT)
}

// -- Upload URL endpoint: returns presigned S3 PUT URLs --

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UploadUrlInput {
    content_type: String,
    upload_type: String, // "image" or "voice"
}

async fn get_upload_url(
    State(state): State<AppState>,
    Path(recipe_id): Path<Uuid>,
    Json(input): Json<UploadUrlInput>,
) -> Result<Json<serde_json::Value>, AppError> {
    let exists: Option<(Uuid,)> = sqlx::query_as("SELECT id FROM recipes WHERE id = $1")
        .bind(recipe_id)
        .fetch_optional(&state.db)
        .await?;
    if exists.is_none() {
        return Err(AppError::NotFound);
    }

    let ext = input.content_type.split('/').nth(1).unwrap_or("bin");
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let prefix = match input.upload_type.as_str() {
        "voice" => "recipe-voice",
        _ => "recipe-images",
    };
    let key = format!("{prefix}/{recipe_id}-{ts}.{ext}");

    let (presigned_url, public_url) =
        shared::media::presign_upload(&state.s3, &state.media_bucket, &key, &input.content_type)
            .await?;

    tracing::info!(recipe_id = %recipe_id, upload_type = %input.upload_type, key = %key, "presigned URL generated");

    Ok(Json(serde_json::json!({
        "uploadUrl": presigned_url,
        "key": key,
        "publicUrl": public_url
    })))
}

// -- Confirm upload: register the uploaded file in the DB --

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ConfirmImageInput {
    key: String,
    public_url: String,
}

async fn confirm_image(
    State(state): State<AppState>,
    Path(recipe_id): Path<Uuid>,
    Json(input): Json<ConfirmImageInput>,
) -> Result<axum::http::StatusCode, AppError> {
    sqlx::query("INSERT INTO recipe_images (recipe_id, image_url, image_key) VALUES ($1, $2, $3)")
        .bind(recipe_id)
        .bind(&input.public_url)
        .bind(&input.key)
        .execute(&state.db)
        .await?;

    tracing::info!(recipe_id = %recipe_id, key = %input.key, "recipe image confirmed");

    invalidate_recipe_og(recipe_id, &state.db).await;

    Ok(axum::http::StatusCode::NO_CONTENT)
}

fn slugify(title: &str) -> String {
    title
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

async fn invalidate_recipe_og(recipe_id: Uuid, db: &sqlx::PgPool) {
    let distribution_id = match std::env::var("CLOUDFRONT_DISTRIBUTION_ID") {
        Ok(id) if !id.is_empty() => id,
        _ => {
            tracing::warn!("CLOUDFRONT_DISTRIBUTION_ID not set, skipping invalidation");
            return;
        }
    };

    let title: Option<(String,)> = sqlx::query_as("SELECT title FROM recipes WHERE id = $1")
        .bind(recipe_id)
        .fetch_optional(db)
        .await
        .ok()
        .flatten();

    let slug = match title {
        Some((t,)) => slugify(&t),
        None => {
            tracing::warn!(recipe_id = %recipe_id, "recipe not found for OG invalidation");
            return;
        }
    };

    let path = format!("/recipes/{slug}");
    let config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
    let client = aws_sdk_cloudfront::Client::new(&config);

    let paths = aws_sdk_cloudfront::types::Paths::builder()
        .quantity(1)
        .items(&path)
        .build()
        .unwrap();

    let batch = aws_sdk_cloudfront::types::InvalidationBatch::builder()
        .paths(paths)
        .caller_reference(format!(
            "{recipe_id}-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis()
        ))
        .build()
        .unwrap();

    match client
        .create_invalidation()
        .distribution_id(&distribution_id)
        .invalidation_batch(batch)
        .send()
        .await
    {
        Ok(_) => tracing::info!(path = %path, "CloudFront OG invalidation created"),
        Err(e) => tracing::error!(path = %path, error = %e, "failed to invalidate CloudFront"),
    }
}

// -- Submit voice review: register and trigger processing --

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SubmitVoiceReviewInput {
    key: String,
    mime_type: String,
}

async fn submit_voice_review(
    State(state): State<AppState>,
    Path(recipe_id): Path<Uuid>,
    Json(input): Json<SubmitVoiceReviewInput>,
) -> Result<axum::http::StatusCode, AppError> {
    let review_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO recipe_reviews (id, recipe_id, voice_key, status) VALUES ($1, $2, $3, 'pending')",
    )
    .bind(review_id)
    .bind(recipe_id)
    .bind(&input.key)
    .execute(&state.db)
    .await?;

    tracing::info!(recipe_id = %recipe_id, review_id = %review_id, "voice review created");

    let mime = input
        .mime_type
        .split(';')
        .next()
        .unwrap_or(&input.mime_type)
        .trim();
    invoke_processing(recipe_id, review_id, &input.key, mime).await;

    Ok(axum::http::StatusCode::NO_CONTENT)
}

async fn invoke_processing(recipe_id: Uuid, review_id: Uuid, voice_key: &str, voice_mime: &str) {
    let function_name = match std::env::var("PROCESSING_FUNCTION_NAME") {
        Ok(name) if !name.is_empty() => name,
        _ => {
            tracing::warn!("PROCESSING_FUNCTION_NAME not set, skipping");
            return;
        }
    };

    let payload = serde_json::json!({
        "process_type": "recipe_review",
        "recipe_id": recipe_id.to_string(),
        "review_id": review_id.to_string(),
        "voice_key": voice_key,
        "voice_mime_type": voice_mime,
    });

    let config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
    let client = aws_sdk_lambda::Client::new(&config);

    match client
        .invoke()
        .function_name(&function_name)
        .invocation_type(aws_sdk_lambda::types::InvocationType::Event)
        .payload(aws_sdk_lambda::primitives::Blob::new(
            serde_json::to_vec(&payload).unwrap_or_default(),
        ))
        .send()
        .await
    {
        Ok(_) => tracing::info!(review_id = %review_id, "processing invoked"),
        Err(e) => {
            tracing::error!(review_id = %review_id, error = %e, "failed to invoke processing")
        }
    }
}

async fn delete_review(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<axum::http::StatusCode, AppError> {
    let result = sqlx::query("DELETE FROM recipe_reviews WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    tracing::info!(review_id = %id, "review deleted");
    Ok(axum::http::StatusCode::NO_CONTENT)
}

async fn rerun_review(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<axum::http::StatusCode, AppError> {
    let row: Option<(Uuid, Uuid, Option<String>, Option<String>)> = sqlx::query_as(
        "SELECT id, recipe_id, voice_key, voice_key FROM recipe_reviews WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?;

    let (review_id, recipe_id, voice_key, _) = row.ok_or(AppError::NotFound)?;
    let voice_key =
        voice_key.ok_or_else(|| AppError::BadRequest("no voice data to reprocess".into()))?;

    sqlx::query("UPDATE recipe_reviews SET status = 'pending', processing_error = NULL, updated_at = now() WHERE id = $1")
        .bind(review_id)
        .execute(&state.db)
        .await?;

    let mime = if voice_key.ends_with(".webm") {
        "audio/webm"
    } else {
        "audio/mpeg"
    };
    invoke_processing(recipe_id, review_id, &voice_key, mime).await;

    tracing::info!(review_id = %review_id, "review rerun triggered");
    Ok(axum::http::StatusCode::NO_CONTENT)
}

async fn delete_image(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<axum::http::StatusCode, AppError> {
    let result = sqlx::query("DELETE FROM recipe_images WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    tracing::info!(image_id = %id, "image deleted");
    Ok(axum::http::StatusCode::NO_CONTENT)
}

fn router(state: AppState) -> Router {
    Router::new()
        .route("/recipes", get(list_recipes).post(create_recipe))
        .route("/recipes/{id}", get(get_recipe).delete(delete_recipe))
        .route("/recipes/{id}/upload-url", post(get_upload_url))
        .route("/recipes/{id}/image", post(confirm_image))
        .route("/recipes/{id}/voice-review", post(submit_voice_review))
        .route(
            "/recipes/reviews/{id}",
            axum::routing::delete(delete_review),
        )
        .route("/recipes/reviews/{id}/rerun", post(rerun_review))
        .route("/recipes/images/{id}", axum::routing::delete(delete_image))
        .layer(shared::cors::layer())
        .with_state(state)
}

#[tokio::main]
async fn main() -> Result<(), lambda_http::Error> {
    shared::init_tracing();
    tracing::info!("recipes-api starting");
    let state = AppState::from_env().await;
    let app = router(state);
    lambda_http::run(app).await
}
