use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::routing::{get, head};
use axum::{Json, Router};
use base64::Engine;
use serde::{Deserialize, Serialize};
use shared::error::AppError;
use shared::types::{RecipeSource, UnitType};
use shared::AppState;
use uuid::Uuid;

const MCP_PROTOCOL_VERSION: &str = "2025-03-26";

// -- Well-known metadata endpoints (unauthenticated) --

async fn oauth_authorization_server_metadata() -> Json<serde_json::Value> {
    let raw_domain = std::env::var("COGNITO_DOMAIN").unwrap_or_default();
    let cognito_domain = if raw_domain.starts_with("https://") || raw_domain.starts_with("http://") {
        raw_domain
    } else {
        format!("https://{raw_domain}")
    };
    let issuer = std::env::var("COGNITO_ISSUER").unwrap_or_default();

    Json(serde_json::json!({
        "issuer": issuer,
        "authorization_endpoint": format!("{cognito_domain}/oauth2/authorize"),
        "token_endpoint": format!("{cognito_domain}/oauth2/token"),
        "revocation_endpoint": format!("{cognito_domain}/oauth2/revoke"),
        "jwks_uri": format!("{issuer}/.well-known/jwks.json"),
        "response_types_supported": ["code"],
        "grant_types_supported": ["authorization_code", "refresh_token"],
        "code_challenge_methods_supported": ["S256"],
        "scopes_supported": ["openid", "profile", "email", "recipe.write", "recipe.read"]
    }))
}

async fn oauth_protected_resource() -> Json<serde_json::Value> {
    let api_url = std::env::var("API_BASE_URL").unwrap_or_default();
    let app_url = std::env::var("APP_BASE_URL").unwrap_or_default();

    Json(serde_json::json!({
        "resource": api_url,
        "authorization_servers": [app_url],
        "bearer_methods_supported": ["header"],
        "scopes_supported": ["recipe.write", "recipe.read"]
    }))
}

// -- MCP protocol endpoints --

async fn mcp_head() -> (StatusCode, HeaderMap) {
    let mut headers = HeaderMap::new();
    headers.insert("mcp-protocol-version", MCP_PROTOCOL_VERSION.parse().unwrap());
    (StatusCode::OK, headers)
}

#[derive(Debug, Deserialize)]
struct McpMessage {
    jsonrpc: String,
    id: Option<serde_json::Value>,
    method: String,
    params: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
struct McpResponse {
    jsonrpc: String,
    id: Option<serde_json::Value>,
    result: serde_json::Value,
}

// Auth is handled by ALB jwt-validation. The ALB validates the JWT and
// forwards the request with the token still in the Authorization header.
// We extract the user identity from the token claims.
async fn mcp_post(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(msg): Json<McpMessage>,
) -> Result<Json<McpResponse>, AppError> {
    tracing::info!(method = %msg.method, "MCP request");
    match msg.method.as_str() {
        "initialize" => handle_initialize(msg),
        "tools/list" => handle_tools_list(msg),
        "tools/call" => {
            let user = extract_user_from_token(&headers)?;
            tracing::info!(user_sub = %user.sub, "MCP tools/call");
            handle_tools_call(msg, &state, &user).await
        }
        _ => Err(AppError::BadRequest(format!("unknown method: {}", msg.method))),
    }
}

/// Decode user identity from the ALB-validated JWT.
/// ALB already validated the token — just extract claims, no crypto.
fn extract_user_from_token(headers: &HeaderMap) -> Result<shared::types::UserContext, AppError> {
    let auth_header = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| AppError::Unauthorized("missing authorization header".into()))?;
    let token = auth_header
        .strip_prefix("Bearer ")
        .or_else(|| auth_header.strip_prefix("bearer "))
        .ok_or_else(|| AppError::Unauthorized("missing bearer token".into()))?;

    // JWT is header.payload.signature — decode the payload (index 1)
    let payload_b64 = token
        .split('.')
        .nth(1)
        .ok_or_else(|| AppError::Unauthorized("malformed token".into()))?;
    let payload_bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(payload_b64)
        .map_err(|_| AppError::Unauthorized("invalid token encoding".into()))?;

    #[derive(Deserialize)]
    struct Claims {
        sub: String,
        email: Option<String>,
    }

    let claims: Claims = serde_json::from_slice(&payload_bytes)
        .map_err(|_| AppError::Unauthorized("invalid token claims".into()))?;

    Ok(shared::types::UserContext {
        sub: claims.sub,
        email: claims.email,
        user_id: None,
    })
}

fn handle_initialize(msg: McpMessage) -> Result<Json<McpResponse>, AppError> {
    Ok(Json(McpResponse {
        jsonrpc: msg.jsonrpc,
        id: msg.id,
        result: serde_json::json!({
            "protocolVersion": MCP_PROTOCOL_VERSION,
            "serverInfo": {
                "name": "tastebase",
                "version": "1.0.0"
            },
            "capabilities": {
                "tools": {}
            }
        }),
    }))
}

fn handle_tools_list(msg: McpMessage) -> Result<Json<McpResponse>, AppError> {
    Ok(Json(McpResponse {
        jsonrpc: msg.jsonrpc,
        id: msg.id,
        result: serde_json::json!({
            "tools": [save_recipe_tool_def()]
        }),
    }))
}

fn save_recipe_tool_def() -> serde_json::Value {
    serde_json::json!({
        "name": "save_recipe",
        "description": "Save a recipe to the user's Tastebase account. Only call this after presenting the recipe and receiving explicit confirmation from the user that they want to save it.",
        "inputSchema": {
            "type": "object",
            "required": ["title", "base_servings", "ingredients", "steps"],
            "properties": {
                "title": { "type": "string" },
                "description": { "type": "string" },
                "base_servings": { "type": "integer", "minimum": 1 },
                "notes": { "type": "string" },
                "ingredients": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "required": ["id", "name", "amount"],
                        "properties": {
                            "id": { "type": "string" },
                            "name": { "type": "string" },
                            "amount": { "type": "number" },
                            "unit": { "type": "string" }
                        }
                    }
                },
                "steps": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "required": ["id", "title", "content"],
                        "properties": {
                            "id": { "type": "string" },
                            "title": { "type": "string" },
                            "content": { "type": "string" },
                            "timer_seconds": { "type": "integer" }
                        }
                    }
                }
            }
        }
    })
}

#[derive(Debug, Deserialize)]
struct SaveRecipeParams {
    title: String,
    description: Option<String>,
    base_servings: i32,
    notes: Option<String>,
    ingredients: Vec<IngredientParam>,
    steps: Vec<StepParam>,
}

#[derive(Debug, Deserialize)]
struct IngredientParam {
    id: String,
    name: String,
    amount: f64,
    unit: Option<String>,
}

#[derive(Debug, Deserialize)]
struct StepParam {
    id: String,
    title: String,
    content: String,
    timer_seconds: Option<i32>,
}

async fn handle_tools_call(
    msg: McpMessage,
    state: &AppState,
    user: &shared::types::UserContext,
) -> Result<Json<McpResponse>, AppError> {
    let params = msg.params.ok_or_else(|| AppError::BadRequest("missing params".into()))?;
    let tool_name = params.get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest("missing tool name".into()))?;

    match tool_name {
        "save_recipe" => {
            let arguments = params.get("arguments")
                .ok_or_else(|| AppError::BadRequest("missing arguments".into()))?;
            let recipe_params: SaveRecipeParams = serde_json::from_value(arguments.clone())
                .map_err(|e| AppError::BadRequest(format!("invalid arguments: {e}")))?;

            let result = save_recipe(state, user, recipe_params).await?;
            Ok(Json(McpResponse {
                jsonrpc: msg.jsonrpc,
                id: msg.id,
                result: serde_json::json!({
                    "content": [{ "type": "text", "text": serde_json::to_string(&result).unwrap() }]
                }),
            }))
        }
        _ => Err(AppError::BadRequest(format!("unknown tool: {tool_name}"))),
    }
}

async fn save_recipe(
    state: &AppState,
    user: &shared::types::UserContext,
    params: SaveRecipeParams,
) -> Result<serde_json::Value, AppError> {
    let user_id = shared::db::resolve_user(&state.db, &user.sub, user.email.as_deref()).await?;
    let recipe_id = Uuid::new_v4();
    let now = time::OffsetDateTime::now_utc();

    let mut tx = state.db.begin().await?;

    sqlx::query(
        "INSERT INTO recipes (id, user_id, title, description, base_servings, notes, source, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $8)"
    )
    .bind(recipe_id)
    .bind(user_id)
    .bind(&params.title)
    .bind(&params.description)
    .bind(params.base_servings)
    .bind(&params.notes)
    .bind(RecipeSource::Claude)
    .bind(now)
    .execute(&mut *tx)
    .await?;

    for (i, ing) in params.ingredients.iter().enumerate() {
        let unit = ing.unit.as_deref()
            .and_then(parse_unit)
            .unwrap_or(UnitType::None);

        sqlx::query(
            "INSERT INTO recipe_ingredients (recipe_id, widget_id, name, amount, unit, sort_order)
             VALUES ($1, $2, $3, $4, $5, $6)"
        )
        .bind(recipe_id)
        .bind(&ing.id)
        .bind(&ing.name)
        .bind(rust_decimal::Decimal::try_from(ing.amount).unwrap_or_default())
        .bind(unit)
        .bind(i as i32)
        .execute(&mut *tx)
        .await?;
    }

    for (i, step) in params.steps.iter().enumerate() {
        sqlx::query(
            "INSERT INTO recipe_steps (recipe_id, widget_id, title, content, timer_seconds, sort_order)
             VALUES ($1, $2, $3, $4, $5, $6)"
        )
        .bind(recipe_id)
        .bind(&step.id)
        .bind(&step.title)
        .bind(&step.content)
        .bind(step.timer_seconds)
        .bind(i as i32)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    tracing::info!(recipe_id = %recipe_id, user_sub = %user.sub, "recipe saved via MCP");
    let url = format!("https://tastebase.ahara.io/recipes/{recipe_id}");
    Ok(serde_json::json!({
        "recipe_id": recipe_id,
        "url": url,
        "message": "Saved. View at the link above."
    }))
}

fn parse_unit(s: &str) -> Option<UnitType> {
    match s {
        "g" => Some(UnitType::G),
        "kg" => Some(UnitType::Kg),
        "ml" => Some(UnitType::Ml),
        "l" => Some(UnitType::L),
        "tsp" => Some(UnitType::Tsp),
        "tbsp" => Some(UnitType::Tbsp),
        "cup" => Some(UnitType::Cup),
        "fl_oz" => Some(UnitType::FlOz),
        "oz" => Some(UnitType::Oz),
        "lb" => Some(UnitType::Lb),
        "pinch" => Some(UnitType::Pinch),
        "piece" => Some(UnitType::Piece),
        "" => Some(UnitType::None),
        _ => None,
    }
}

fn router(state: AppState) -> Router {
    Router::new()
        .route("/.well-known/oauth-authorization-server", get(oauth_authorization_server_metadata))
        .route("/.well-known/oauth-protected-resource", get(oauth_protected_resource))
        .route("/mcp", head(mcp_head).post(mcp_post))
        .layer(shared::cors::layer())
        .with_state(state)
}

#[tokio::main]
async fn main() -> Result<(), lambda_http::Error> {
    shared::init_tracing();
    tracing::info!("mcp-server starting");
    let state = AppState::from_env().await;
    let app = router(state);
    lambda_http::run(app).await
}
