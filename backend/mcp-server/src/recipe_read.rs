use serde::Deserialize;
use shared::{
    AppState,
    error::AppError,
    types::{Recipe, RecipeIngredient, RecipeStep},
};
use uuid::Uuid;

pub fn tool_def() -> serde_json::Value {
    serde_json::json!({
        "name": "get_recipe",
        "description": "Read an exact recipe version before editing it, including variation groups, all conditional ingredients and steps, and notes. Find its recipe_id with list_recipes. Responses use camelCase fields; save_recipe and update_recipe use snake_case (widgetId becomes id).",
        "inputSchema": {
            "type": "object", "required": ["recipe_id"],
            "properties": {"recipe_id": {"type": "string", "format": "uuid"}}
        }
    })
}

#[derive(Deserialize)]
struct Input {
    recipe_id: Uuid,
}

pub(super) async fn get(state: &AppState, id: Uuid) -> Result<serde_json::Value, AppError> {
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
    let mut result = serde_json::json!(recipe);
    result["ingredients"] = serde_json::json!(ingredients);
    result["steps"] = serde_json::json!(steps);
    Ok(result)
}

pub async fn dispatch(
    msg_id: Option<serde_json::Value>,
    state: &AppState,
    arguments: serde_json::Value,
) -> super::JsonRpcResponse {
    let input: Input = match serde_json::from_value(arguments) {
        Ok(input) => input,
        Err(error) => return super::tool_text_response(msg_id, error.to_string(), true),
    };
    match super::mcp_operation(state, "tastebase.mcp.tools.get_recipe")
        .observe(get(state, input.recipe_id))
        .await
    {
        Ok(recipe) => super::tool_json_response(msg_id, &recipe),
        Err(error) => super::tool_text_response(msg_id, error.to_string(), true),
    }
}
