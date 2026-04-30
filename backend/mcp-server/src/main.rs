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
#[allow(clippy::cognitive_complexity)]
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
    let tools = vec![
        list_recipes_tool_def(),
        save_recipe_tool_def(),
        update_recipe_tool_def(),
    ];
    tracing::info!(tool_count = tools.len(), "tools/list");
    Ok(jsonrpc_result(
        msg.id,
        serde_json::json!({ "tools": tools }),
    ))
}

fn list_recipes_tool_def() -> serde_json::Value {
    serde_json::json!({
        "name": "list_recipes",
        "description": "List saved recipes. Returns id, title, and description for each recipe. Use this to find existing recipes that can be linked as ingredients in other recipes (via linked_recipe_id in save_recipe).",
        "inputSchema": {
            "type": "object",
            "properties": {
                "search": {
                    "type": "string",
                    "description": "Optional text filter — matches against recipe title (case-insensitive substring match). Omit to list all recipes."
                }
            }
        }
    })
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
                            },
                            "linked_recipe_id": {
                                "type": "string",
                                "description": "Optional UUID of another recipe that this ingredient refers to. Use list_recipes to find the ID. Example: if this ingredient is '1 cup olive tapenade' and there's already a tapenade recipe saved, pass its ID here to link them."
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

fn update_recipe_tool_def() -> serde_json::Value {
    serde_json::json!({
        "name": "update_recipe",
        "description": "Update an existing recipe. Only provide the fields you want to change — omitted fields are left unchanged. When providing ingredients or steps, the full list replaces the existing one. Set new_version to true to create a new version (preserving the previous one) instead of overwriting.",
        "inputSchema": {
            "type": "object",
            "required": ["recipe_id"],
            "properties": {
                "recipe_id": {
                    "type": "string",
                    "description": "UUID of the recipe to update. If this is an older version, the latest version in the same group will be updated."
                },
                "new_version": {
                    "type": "boolean",
                    "description": "If true, creates a new version of the recipe instead of overwriting the current one. Defaults to false."
                },
                "title": {
                    "type": "string",
                    "description": "New recipe title"
                },
                "description": {
                    "type": "string",
                    "description": "New recipe description"
                },
                "base_servings": {
                    "type": "integer",
                    "minimum": 1,
                    "description": "New serving count"
                },
                "notes": {
                    "type": "string",
                    "description": "New markdown-formatted notes"
                },
                "ingredients": {
                    "type": "array",
                    "description": "Full replacement ingredient list. If provided, replaces ALL existing ingredients.",
                    "items": {
                        "type": "object",
                        "required": ["id", "name", "short_name", "amount", "unit"],
                        "properties": {
                            "id": {
                                "type": "string",
                                "description": "Unique ingredient identifier (e.g. '0001')"
                            },
                            "name": {
                                "type": "string",
                                "description": "Full ingredient description"
                            },
                            "short_name": {
                                "type": "string",
                                "description": "Short name for step references"
                            },
                            "amount": {
                                "type": "number",
                                "description": "Numeric quantity"
                            },
                            "unit": {
                                "type": "string",
                                "description": "Unit: g, kg, ml, l, tsp, tbsp, cup, fl_oz, oz, lb, pinch, piece, or empty string"
                            },
                            "linked_recipe_id": {
                                "type": "string",
                                "description": "Optional UUID of a linked recipe"
                            }
                        }
                    }
                },
                "steps": {
                    "type": "array",
                    "description": "Full replacement step list. If provided, replaces ALL existing steps.",
                    "items": {
                        "type": "object",
                        "required": ["id", "title", "content"],
                        "properties": {
                            "id": {
                                "type": "string",
                                "description": "Unique step identifier (e.g. 's1')"
                            },
                            "title": {
                                "type": "string",
                                "description": "Short step title"
                            },
                            "content": {
                                "type": "string",
                                "description": "Full step instructions with {ingredient_id} references"
                            },
                            "timer_seconds": {
                                "type": "integer",
                                "description": "Optional timer duration in seconds"
                            }
                        }
                    }
                }
            }
        }
    })
}

#[allow(clippy::cognitive_complexity)]
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
        "list_recipes" => {
            let arguments = params
                .get("arguments")
                .cloned()
                .unwrap_or(serde_json::json!({}));
            let search = arguments
                .get("search")
                .and_then(|v| v.as_str())
                .map(String::from);
            match list_recipes(state, search).await {
                Ok(result) => Ok(jsonrpc_result(
                    msg.id,
                    serde_json::json!({
                        "content": [{ "type": "text", "text": serde_json::to_string(&result).unwrap() }],
                        "isError": false
                    }),
                )),
                Err(e) => {
                    tracing::error!(error = %e, "list_recipes failed");
                    Ok(jsonrpc_result(
                        msg.id,
                        serde_json::json!({
                            "content": [{ "type": "text", "text": format!("list_recipes failed: {e}") }],
                            "isError": true
                        }),
                    ))
                }
            }
        }
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
        "update_recipe" => {
            let arguments = params
                .get("arguments")
                .ok_or_else(|| jsonrpc_error(msg.id.clone(), -32602, "missing arguments"))?;
            let recipe_params: UpdateRecipeParams = match serde_json::from_value(arguments.clone())
            {
                Ok(p) => p,
                Err(e) => {
                    tracing::warn!(error = %e, "update_recipe validation failed");
                    return Ok(jsonrpc_result(
                        msg.id,
                        serde_json::json!({
                            "content": [{ "type": "text", "text": format!("{e}") }],
                            "isError": true
                        }),
                    ));
                }
            };

            match update_recipe(state, user, recipe_params).await {
                Ok(result) => Ok(jsonrpc_result(
                    msg.id,
                    serde_json::json!({
                        "content": [{ "type": "text", "text": serde_json::to_string(&result).unwrap() }],
                        "isError": false
                    }),
                )),
                Err(e) => {
                    tracing::error!(error = %e, "update_recipe execution failed");
                    Ok(jsonrpc_result(
                        msg.id,
                        serde_json::json!({
                            "content": [{ "type": "text", "text": format!(
                                "update_recipe failed: {e}. The recipe was not modified."
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
    linked_recipe_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct StepParam {
    id: String,
    title: String,
    content: String,
    timer_seconds: Option<i32>,
}

async fn list_recipes(
    state: &AppState,
    search: Option<String>,
) -> Result<serde_json::Value, AppError> {
    // Only return the latest version of each recipe
    let rows: Vec<(Uuid, String, Option<String>)> = match &search {
        Some(term) => {
            let pattern = format!("%{term}%");
            sqlx::query_as(
                "SELECT r.id, r.title, r.description FROM recipes r
                 INNER JOIN (
                   SELECT version_group_id, MAX(version) AS max_v FROM recipes GROUP BY version_group_id
                 ) latest ON r.version_group_id = latest.version_group_id AND r.version = latest.max_v
                 WHERE r.title ILIKE $1
                 ORDER BY r.created_at DESC",
            )
            .bind(&pattern)
            .fetch_all(&state.db)
            .await?
        }
        None => {
            sqlx::query_as(
                "SELECT r.id, r.title, r.description FROM recipes r
                 INNER JOIN (
                   SELECT version_group_id, MAX(version) AS max_v FROM recipes GROUP BY version_group_id
                 ) latest ON r.version_group_id = latest.version_group_id AND r.version = latest.max_v
                 ORDER BY r.created_at DESC",
            )
            .fetch_all(&state.db)
            .await?
        }
    };

    let recipes: Vec<serde_json::Value> = rows
        .into_iter()
        .map(|(id, title, description)| {
            serde_json::json!({
                "id": id,
                "title": title,
                "description": description,
            })
        })
        .collect();

    tracing::info!(count = recipes.len(), search = ?search, "recipes listed via MCP");
    Ok(serde_json::json!({ "recipes": recipes }))
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
        "INSERT INTO recipes (id, title, description, base_servings, notes, source, version, version_group_id, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, $6, 1, $1, $7, $7)",
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
        let linked_id: Option<Uuid> = ing.linked_recipe_id.as_deref().and_then(|s| s.parse().ok());
        sqlx::query(
            "INSERT INTO recipe_ingredients (recipe_id, widget_id, name, short_name, amount, unit, sort_order, linked_recipe_id)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
        )
        .bind(recipe_id)
        .bind(&ing.id)
        .bind(&ing.name)
        .bind(&ing.short_name)
        .bind(rust_decimal::Decimal::try_from(ing.amount).unwrap_or_default())
        .bind(unit)
        .bind(i as i32)
        .bind(linked_id)
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
    let url = format!("https://tastebase.ahara.io/recipes/{slug}");
    Ok(serde_json::json!({
        "recipe_id": recipe_id,
        "url": url,
        "message": "Saved. View at the link above."
    }))
}

#[derive(Debug, Deserialize)]
struct UpdateRecipeParams {
    recipe_id: String,
    #[serde(default)]
    new_version: bool,
    title: Option<String>,
    description: Option<String>,
    base_servings: Option<i32>,
    notes: Option<String>,
    ingredients: Option<Vec<IngredientParam>>,
    steps: Option<Vec<StepParam>>,
}

#[allow(clippy::cognitive_complexity)]
async fn update_recipe(
    state: &AppState,
    _user: &shared::types::UserContext,
    params: UpdateRecipeParams,
) -> Result<serde_json::Value, AppError> {
    let recipe_id: Uuid = params
        .recipe_id
        .parse()
        .map_err(|_| AppError::BadRequest("invalid recipe_id UUID".into()))?;

    // Find the recipe's version group, then get the latest version in that group
    let group_id: Uuid = sqlx::query_scalar("SELECT version_group_id FROM recipes WHERE id = $1")
        .bind(recipe_id)
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::NotFound)?;

    let (
        latest_id,
        latest_version,
        current_title,
        current_description,
        current_servings,
        current_notes,
    ): (Uuid, i32, String, Option<String>, i32, Option<String>) = sqlx::query_as(
        "SELECT id, version, title, description, base_servings, notes
         FROM recipes WHERE version_group_id = $1
         ORDER BY version DESC LIMIT 1",
    )
    .bind(group_id)
    .fetch_one(&state.db)
    .await?;

    let final_title = params.title.as_deref().unwrap_or(&current_title);
    let final_description = params
        .description
        .as_deref()
        .or(current_description.as_deref());
    let final_servings = params.base_servings.unwrap_or(current_servings);
    let final_notes = params.notes.as_deref().or(current_notes.as_deref());

    let now = time::OffsetDateTime::now_utc();
    let mut tx = state.db.begin().await?;

    let target_id = if params.new_version {
        // Create a new version row
        let new_id = Uuid::new_v4();
        let new_version = latest_version + 1;

        sqlx::query(
            "INSERT INTO recipes (id, title, description, base_servings, notes, source, version, version_group_id, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $9)",
        )
        .bind(new_id)
        .bind(final_title)
        .bind(final_description)
        .bind(final_servings)
        .bind(final_notes)
        .bind(RecipeSource::Claude)
        .bind(new_version)
        .bind(group_id)
        .bind(now)
        .execute(&mut *tx)
        .await?;

        // Copy ingredients from latest version if not replaced
        if params.ingredients.is_none() {
            sqlx::query(
                "INSERT INTO recipe_ingredients (recipe_id, widget_id, name, short_name, amount, unit, sort_order, linked_recipe_id)
                 SELECT $1, widget_id, name, short_name, amount, unit, sort_order, linked_recipe_id
                 FROM recipe_ingredients WHERE recipe_id = $2 ORDER BY sort_order",
            )
            .bind(new_id)
            .bind(latest_id)
            .execute(&mut *tx)
            .await?;
        }

        // Copy steps from latest version if not replaced
        if params.steps.is_none() {
            sqlx::query(
                "INSERT INTO recipe_steps (recipe_id, widget_id, title, content, timer_seconds, sort_order)
                 SELECT $1, widget_id, title, content, timer_seconds, sort_order
                 FROM recipe_steps WHERE recipe_id = $2 ORDER BY sort_order",
            )
            .bind(new_id)
            .bind(latest_id)
            .execute(&mut *tx)
            .await?;
        }

        tracing::info!(recipe_id = %new_id, version = new_version, group = %group_id, "new recipe version created via MCP");
        new_id
    } else {
        // Update existing latest version in place
        sqlx::query(
            "UPDATE recipes SET title = $1, description = $2, base_servings = $3, notes = $4, updated_at = $5 WHERE id = $6",
        )
        .bind(final_title)
        .bind(final_description)
        .bind(final_servings)
        .bind(final_notes)
        .bind(now)
        .bind(latest_id)
        .execute(&mut *tx)
        .await?;

        // Delete existing ingredients/steps if replacing
        if params.ingredients.is_some() {
            sqlx::query("DELETE FROM recipe_ingredients WHERE recipe_id = $1")
                .bind(latest_id)
                .execute(&mut *tx)
                .await?;
        }

        if params.steps.is_some() {
            sqlx::query("DELETE FROM recipe_steps WHERE recipe_id = $1")
                .bind(latest_id)
                .execute(&mut *tx)
                .await?;
        }

        tracing::info!(recipe_id = %latest_id, "recipe updated in place via MCP");
        latest_id
    };

    // Insert new ingredients if provided
    if let Some(ref ingredients) = params.ingredients {
        for (i, ing) in ingredients.iter().enumerate() {
            let unit = parse_unit(&ing.unit).unwrap_or(UnitType::None);
            let linked_id: Option<Uuid> =
                ing.linked_recipe_id.as_deref().and_then(|s| s.parse().ok());
            sqlx::query(
                "INSERT INTO recipe_ingredients (recipe_id, widget_id, name, short_name, amount, unit, sort_order, linked_recipe_id)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
            )
            .bind(target_id)
            .bind(&ing.id)
            .bind(&ing.name)
            .bind(&ing.short_name)
            .bind(rust_decimal::Decimal::try_from(ing.amount).unwrap_or_default())
            .bind(unit)
            .bind(i as i32)
            .bind(linked_id)
            .execute(&mut *tx)
            .await?;
        }
    }

    // Insert new steps if provided
    if let Some(ref steps) = params.steps {
        for (i, step) in steps.iter().enumerate() {
            sqlx::query(
                "INSERT INTO recipe_steps (recipe_id, widget_id, title, content, timer_seconds, sort_order)
                 VALUES ($1, $2, $3, $4, $5, $6)",
            )
            .bind(target_id)
            .bind(&step.id)
            .bind(&step.title)
            .bind(&step.content)
            .bind(step.timer_seconds)
            .bind(i as i32)
            .execute(&mut *tx)
            .await?;
        }
    }

    tx.commit().await?;

    let slug = slugify(final_title);
    let url = format!("https://tastebase.ahara.io/recipes/{slug}");
    Ok(serde_json::json!({
        "recipe_id": target_id,
        "url": url,
        "message": if params.new_version { "New version created." } else { "Recipe updated." }
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
