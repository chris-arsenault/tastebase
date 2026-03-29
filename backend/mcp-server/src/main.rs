use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::routing::{get, head};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use shared::AppState;
use shared::auth;
use shared::error::AppError;
use shared::types::{RecipeSource, UnitType};
use uuid::Uuid;

const MCP_PROTOCOL_VERSION: &str = "2025-03-26";

// -- Well-known metadata endpoints (unauthenticated) --

async fn oauth_authorization_server_metadata() -> Json<serde_json::Value> {
    let raw_domain = std::env::var("COGNITO_DOMAIN").unwrap_or_default();
    let cognito_domain = if raw_domain.starts_with("https://") || raw_domain.starts_with("http://")
    {
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
        "scopes_supported": ["openid", "profile", "email"]
    }))
}

async fn oauth_protected_resource() -> Json<serde_json::Value> {
    let api_url = std::env::var("API_BASE_URL").unwrap_or_default();

    Json(serde_json::json!({
        "resource": api_url,
        "authorization_servers": [api_url],
        "bearer_methods_supported": ["header"],
        "scopes_supported": ["openid", "profile", "email"]
    }))
}

// -- MCP transport endpoints --

// HEAD /mcp — protocol version discovery (no auth)
async fn mcp_head() -> (StatusCode, HeaderMap) {
    let mut headers = HeaderMap::new();
    headers.insert(
        "mcp-protocol-version",
        MCP_PROTOCOL_VERSION.parse().unwrap(),
    );
    (StatusCode::OK, headers)
}

// GET /mcp — server-initiated SSE stream. We don't support this.
async fn mcp_get() -> StatusCode {
    StatusCode::METHOD_NOT_ALLOWED
}

// POST /mcp — all JSON-RPC messages from client
async fn mcp_post(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(msg): Json<McpMessage>,
) -> axum::response::Response {
    let method = msg.method.clone();
    let is_notification = msg.id.is_none();
    tracing::info!(method = %method, is_notification, "MCP request");

    // Authenticate — return 401 with WWW-Authenticate for OAuth discovery on failure
    let user = match authenticate(&headers) {
        Ok(u) => {
            tracing::info!(user_sub = %u.sub, method = %method, "MCP auth OK");
            u
        }
        Err(e) => {
            tracing::warn!(method = %method, error = %e, "MCP auth failed");
            let api_url = std::env::var("API_BASE_URL").unwrap_or_default();
            return (
                StatusCode::UNAUTHORIZED,
                [(
                    "WWW-Authenticate",
                    format!("Bearer resource_metadata=\"{api_url}/.well-known/oauth-protected-resource\""),
                )],
                Json(serde_json::json!({"message": "unauthorized"})),
            )
                .into_response();
        }
    };

    // Notifications have no id and expect 202 Accepted with no body
    if is_notification {
        tracing::info!(method = %method, "notification acknowledged");
        return StatusCode::ACCEPTED.into_response();
    }

    // JSON-RPC requests — dispatch by method
    let result = match method.as_str() {
        "initialize" => handle_initialize(msg),
        "ping" => handle_ping(msg),
        "tools/list" => handle_tools_list(msg),
        "tools/call" => {
            tracing::info!(user_sub = %user.sub, "tools/call");
            handle_tools_call(msg, &state, &user).await
        }
        other => {
            tracing::warn!(method = %other, "unknown method");
            Err(jsonrpc_error(
                msg.id,
                -32601,
                &format!("Method not found: {other}"),
            ))
        }
    };

    match result {
        Ok(resp) => {
            tracing::info!(method = %method, "response OK");
            Json(resp).into_response()
        }
        Err(err) => {
            tracing::error!(method = %method, error = ?err.error, "response error");
            Json(err).into_response()
        }
    }
}

// -- Auth --

fn authenticate(headers: &HeaderMap) -> Result<shared::types::UserContext, AppError> {
    let auth_header = headers.get("authorization").and_then(|v| v.to_str().ok());
    let token = auth::extract_bearer(auth_header)?;
    auth::decode_token(token)
}

// -- JSON-RPC types --

#[derive(Debug, Deserialize)]
struct McpMessage {
    #[allow(dead_code)]
    jsonrpc: String,
    id: Option<serde_json::Value>,
    method: String,
    params: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<serde_json::Value>,
}

fn jsonrpc_result(id: Option<serde_json::Value>, result: serde_json::Value) -> JsonRpcResponse {
    JsonRpcResponse {
        jsonrpc: "2.0".into(),
        id,
        result: Some(result),
        error: None,
    }
}

fn jsonrpc_error(id: Option<serde_json::Value>, code: i32, message: &str) -> JsonRpcResponse {
    JsonRpcResponse {
        jsonrpc: "2.0".into(),
        id,
        result: None,
        error: Some(serde_json::json!({
            "code": code,
            "message": message,
        })),
    }
}

// -- Method handlers --

fn handle_initialize(msg: McpMessage) -> Result<JsonRpcResponse, JsonRpcResponse> {
    Ok(jsonrpc_result(
        msg.id,
        serde_json::json!({
            "protocolVersion": MCP_PROTOCOL_VERSION,
            "serverInfo": {
                "name": "tastebase",
                "version": "1.0.0"
            },
            "capabilities": {
                "tools": {}
            }
        }),
    ))
}

fn handle_ping(msg: McpMessage) -> Result<JsonRpcResponse, JsonRpcResponse> {
    Ok(jsonrpc_result(msg.id, serde_json::json!({})))
}

fn handle_tools_list(msg: McpMessage) -> Result<JsonRpcResponse, JsonRpcResponse> {
    let tools = vec![save_recipe_tool_def()];
    tracing::info!(tool_count = tools.len(), "tools/list");
    Ok(jsonrpc_result(
        msg.id,
        serde_json::json!({ "tools": tools }),
    ))
}

fn save_recipe_tool_def() -> serde_json::Value {
    serde_json::json!({
        "name": "save_recipe",
        "description": "Save a recipe to the user's Tastebase account. Only call this after presenting the full recipe and receiving explicit user confirmation to save it. All required fields must be provided — the call will fail with a descriptive error if any are missing.",
        "inputSchema": {
            "type": "object",
            "required": ["title", "description", "base_servings", "notes", "ingredients", "steps"],
            "properties": {
                "title": {
                    "type": "string",
                    "description": "Recipe title (e.g. 'Chimichurri for Costco Beef Cubes')"
                },
                "description": {
                    "type": "string",
                    "description": "One or two sentence summary of the recipe"
                },
                "base_servings": {
                    "type": "integer",
                    "minimum": 1,
                    "description": "Number of servings this recipe makes"
                },
                "notes": {
                    "type": "string",
                    "description": "Markdown-formatted tips, variations, and notes. Use **bold** for emphasis and \\n for line breaks."
                },
                "ingredients": {
                    "type": "array",
                    "description": "All ingredients used in the recipe. Each must have a unique id (e.g. '0001', '0002') that is referenced in step content as {0001}.",
                    "items": {
                        "type": "object",
                        "required": ["id", "name", "short_name", "amount", "unit"],
                        "properties": {
                            "id": {
                                "type": "string",
                                "description": "Unique identifier for this ingredient, referenced in step content as {id}. Use zero-padded numbers like '0001', '0002'."
                            },
                            "name": {
                                "type": "string",
                                "description": "Full ingredient description shown in the ingredient list. Include prep notes and alternatives. Example: 'fresh oregano (or 1 tsp dried)', 'lemon, zested and juiced'."
                            },
                            "short_name": {
                                "type": "string",
                                "description": "Short canonical name used when this ingredient appears in step text via {id} token. Should be 1-2 words. Example: 'oregano', 'lemon', 'olive oil', 'garlic'."
                            },
                            "amount": {
                                "type": "number",
                                "description": "Numeric quantity. Use 0.5 for '1/2', 0.25 for '1/4', etc."
                            },
                            "unit": {
                                "type": "string",
                                "description": "Unit of measurement. Must be one of: g, kg, ml, l, tsp, tbsp, cup, fl_oz, oz, lb, pinch, piece, or empty string for unitless items (e.g. '2 lemons')."
                            }
                        }
                    }
                },
                "steps": {
                    "type": "array",
                    "description": "Ordered preparation steps. Reference ingredients by their id wrapped in curly braces (e.g. {0001}). The app resolves these to the ingredient's short_name.",
                    "items": {
                        "type": "object",
                        "required": ["id", "title", "content"],
                        "properties": {
                            "id": {
                                "type": "string",
                                "description": "Unique step identifier. Use 's1', 's2', etc."
                            },
                            "title": {
                                "type": "string",
                                "description": "Short step title (e.g. 'Macerate the garlic', 'Build the chimichurri')"
                            },
                            "content": {
                                "type": "string",
                                "description": "Full step instructions. Reference ingredients using {ingredient_id} tokens (e.g. 'Mince {0004} finely and combine with {0006}'). These tokens are resolved to ingredient short names in the app."
                            },
                            "timer_seconds": {
                                "type": "integer",
                                "description": "Optional timer duration in seconds for this step. Omit if no specific timing is needed."
                            }
                        }
                    }
                }
            }
        }
    })
}

async fn handle_tools_call(
    msg: McpMessage,
    state: &AppState,
    user: &shared::types::UserContext,
) -> Result<JsonRpcResponse, JsonRpcResponse> {
    let params = msg
        .params
        .ok_or_else(|| jsonrpc_error(msg.id.clone(), -32602, "missing params"))?;
    let tool_name = params
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| jsonrpc_error(msg.id.clone(), -32602, "missing tool name"))?;

    match tool_name {
        "save_recipe" => {
            let arguments = params
                .get("arguments")
                .ok_or_else(|| jsonrpc_error(msg.id.clone(), -32602, "missing arguments"))?;
            let recipe_params: SaveRecipeParams = match serde_json::from_value(arguments.clone()) {
                Ok(p) => p,
                Err(e) => {
                    tracing::warn!(error = %e, "save_recipe validation failed");
                    return Ok(jsonrpc_result(
                        msg.id,
                        serde_json::json!({
                            "content": [{ "type": "text", "text": format!("{e}") }],
                            "isError": true
                        }),
                    ));
                }
            };

            match save_recipe(state, user, recipe_params).await {
                Ok(result) => Ok(jsonrpc_result(
                    msg.id,
                    serde_json::json!({
                        "content": [{ "type": "text", "text": serde_json::to_string(&result).unwrap() }],
                        "isError": false
                    }),
                )),
                Err(e) => {
                    tracing::error!(error = %e, "save_recipe execution failed");
                    Ok(jsonrpc_result(
                        msg.id,
                        serde_json::json!({
                            "content": [{ "type": "text", "text": format!(
                                "save_recipe failed: {e}. This is a server-side error. The recipe was not saved. You may retry the call with the same arguments."
                            ) }],
                            "isError": true
                        }),
                    ))
                }
            }
        }
        _ => Err(jsonrpc_error(
            msg.id,
            -32602,
            &format!("unknown tool: {tool_name}"),
        )),
    }
}

// -- Recipe persistence --

#[derive(Debug, Deserialize)]
struct SaveRecipeParams {
    title: String,
    description: String,
    base_servings: i32,
    notes: String,
    ingredients: Vec<IngredientParam>,
    steps: Vec<StepParam>,
}

#[derive(Debug, Deserialize)]
struct IngredientParam {
    id: String,
    name: String,
    short_name: String,
    amount: f64,
    unit: String,
}

#[derive(Debug, Deserialize)]
struct StepParam {
    id: String,
    title: String,
    content: String,
    timer_seconds: Option<i32>,
}

async fn save_recipe(
    state: &AppState,
    _user: &shared::types::UserContext,
    params: SaveRecipeParams,
) -> Result<serde_json::Value, AppError> {
    let recipe_id = Uuid::new_v4();
    let now = time::OffsetDateTime::now_utc();

    let mut tx = state.db.begin().await?;

    sqlx::query(
        "INSERT INTO recipes (id, title, description, base_servings, notes, source, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $7)",
    )
    .bind(recipe_id)
    .bind(&params.title)
    .bind(&params.description)
    .bind(params.base_servings)
    .bind(&params.notes)
    .bind(RecipeSource::Claude)
    .bind(now)
    .execute(&mut *tx)
    .await?;

    for (i, ing) in params.ingredients.iter().enumerate() {
        let unit = parse_unit(&ing.unit).unwrap_or(UnitType::None);
        sqlx::query(
            "INSERT INTO recipe_ingredients (recipe_id, widget_id, name, short_name, amount, unit, sort_order)
             VALUES ($1, $2, $3, $4, $5, $6, $7)",
        )
        .bind(recipe_id)
        .bind(&ing.id)
        .bind(&ing.name)
        .bind(&ing.short_name)
        .bind(rust_decimal::Decimal::try_from(ing.amount).unwrap_or_default())
        .bind(unit)
        .bind(i as i32)
        .execute(&mut *tx)
        .await?;
    }

    for (i, step) in params.steps.iter().enumerate() {
        sqlx::query(
            "INSERT INTO recipe_steps (recipe_id, widget_id, title, content, timer_seconds, sort_order)
             VALUES ($1, $2, $3, $4, $5, $6)",
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

    tracing::info!(recipe_id = %recipe_id, "recipe saved via MCP");
    let slug = slugify(&params.title);
    let url = format!("https://tastebase.ahara.io/#/recipes/{slug}");
    Ok(serde_json::json!({
        "recipe_id": recipe_id,
        "url": url,
        "message": "Saved. View at the link above."
    }))
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
        .route(
            "/.well-known/oauth-authorization-server",
            get(oauth_authorization_server_metadata),
        )
        .route(
            "/.well-known/oauth-protected-resource",
            get(oauth_protected_resource),
        )
        .route("/mcp", head(mcp_head).get(mcp_get).post(mcp_post))
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
