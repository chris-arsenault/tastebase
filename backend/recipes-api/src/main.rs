use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;
use shared::auth::RequireAuth;
use shared::error::AppError;
use shared::types::{Recipe, RecipeFull, RecipeIngredient, RecipeStep, RecipeSource, UnitType};
use shared::AppState;
use uuid::Uuid;

async fn list_recipes(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    let recipes: Vec<Recipe> = sqlx::query_as(
        "SELECT * FROM recipes ORDER BY created_at DESC"
    )
    .fetch_all(&state.db)
    .await?;

    tracing::info!(count = recipes.len(), "recipes listed");
    Ok(Json(serde_json::json!({ "data": recipes })))
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

    let ingredients: Vec<RecipeIngredient> = sqlx::query_as(
        "SELECT * FROM recipe_ingredients WHERE recipe_id = $1 ORDER BY sort_order"
    )
    .bind(id)
    .fetch_all(&state.db)
    .await?;

    let steps: Vec<RecipeStep> = sqlx::query_as(
        "SELECT * FROM recipe_steps WHERE recipe_id = $1 ORDER BY sort_order"
    )
    .bind(id)
    .fetch_all(&state.db)
    .await?;

    tracing::info!(recipe_id = %id, ingredients = ingredients.len(), steps = steps.len(), "recipe fetched");
    let full = RecipeFull { recipe, ingredients, steps };
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
    amount: f64,
    unit: Option<UnitType>,
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
    RequireAuth(user): RequireAuth,
    Json(input): Json<CreateRecipeInput>,
) -> Result<Json<serde_json::Value>, AppError> {
    tracing::info!(user_sub = %user.sub, title = %input.title, "creating recipe");

    // Validate
    shared::validate::validate_recipe_input(
        &input.title,
        input.description.as_deref(),
        input.base_servings,
        input.notes.as_deref(),
    )?;

    // Sanitize
    let title = shared::sanitize::clean(&input.title);
    let description = shared::sanitize::clean_option(input.description.as_deref());
    let notes = shared::sanitize::clean_option(input.notes.as_deref());

    let user_id = shared::db::resolve_user(&state.db, &user.sub, user.email.as_deref()).await?;
    let recipe_id = Uuid::new_v4();
    let now = time::OffsetDateTime::now_utc();
    let source = input.source.unwrap_or(RecipeSource::Manual);

    let mut tx = state.db.begin().await?;

    sqlx::query(
        "INSERT INTO recipes (id, user_id, title, description, base_servings, notes, source, source_meta, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $9)"
    )
    .bind(recipe_id)
    .bind(user_id)
    .bind(&title)
    .bind(&description)
    .bind(input.base_servings)
    .bind(&notes)
    .bind(source)
    .bind(input.source_meta.as_ref().map(|v| sqlx::types::Json(v)))
    .bind(now)
    .execute(&mut *tx)
    .await?;

    for (i, ing) in input.ingredients.iter().enumerate() {
        let unit = ing.unit.unwrap_or(UnitType::None);
        let name = shared::sanitize::clean(&ing.name);
        sqlx::query(
            "INSERT INTO recipe_ingredients (recipe_id, widget_id, name, amount, unit, sort_order)
             VALUES ($1, $2, $3, $4, $5, $6)"
        )
        .bind(recipe_id)
        .bind(&ing.id)
        .bind(&name)
        .bind(rust_decimal::Decimal::try_from(ing.amount).unwrap_or_default())
        .bind(unit)
        .bind(i as i32)
        .execute(&mut *tx)
        .await?;
    }

    for (i, step) in input.steps.iter().enumerate() {
        let step_title = shared::sanitize::clean(&step.title);
        // Store raw content with {ingredient_id} tokens intact (round-trip fidelity)
        let content = shared::sanitize::clean(&step.content);
        sqlx::query(
            "INSERT INTO recipe_steps (recipe_id, widget_id, title, content, timer_seconds, sort_order)
             VALUES ($1, $2, $3, $4, $5, $6)"
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

    tracing::info!(
        recipe_id = %recipe_id,
        user_id = %user_id,
        source = ?source,
        ingredients = input.ingredients.len(),
        steps = input.steps.len(),
        "recipe created"
    );

    let url = format!("https://tastebase.ahara.io/recipes/{recipe_id}");
    Ok(Json(serde_json::json!({
        "recipe_id": recipe_id,
        "url": url,
        "message": "Saved. View at the link above."
    })))
}

async fn delete_recipe(
    State(state): State<AppState>,
    RequireAuth(user): RequireAuth,
    Path(id): Path<Uuid>,
) -> Result<axum::http::StatusCode, AppError> {
    let user_id = shared::db::resolve_user(&state.db, &user.sub, user.email.as_deref()).await?;
    let result = sqlx::query("DELETE FROM recipes WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(user_id)
        .execute(&state.db)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    tracing::info!(recipe_id = %id, user_id = %user_id, "recipe deleted");
    Ok(axum::http::StatusCode::NO_CONTENT)
}

fn router(state: AppState) -> Router {
    Router::new()
        .route("/recipes", get(list_recipes).post(create_recipe))
        .route("/recipes/{id}", get(get_recipe).delete(delete_recipe))
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
