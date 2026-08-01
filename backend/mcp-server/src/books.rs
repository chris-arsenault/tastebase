use serde::Deserialize;
use shared::AppState;
use shared::error::AppError;
use shared::types::{BookRecommendation, BookStatus, UserContext};
use uuid::Uuid;

use crate::{JsonRpcResponse, mcp_operation, tool_json_response, tool_text_response};

pub(crate) fn list_book_recommendations_tool_def() -> serde_json::Value {
    serde_json::json!({
        "name": "list_book_recommendations",
        "description": "List this user's book recommendation history, including reading status, 1-5 ratings, writeups, and visibility. Call this before making another round of recommendations so prior suggestions and feedback inform the next choices. Results are private to the authenticated user.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "status": {
                    "type": "string",
                    "enum": ["recommended", "reading", "read", "did_not_finish"],
                    "description": "Optional reading-status filter. Omit to retrieve the full recommendation and feedback history."
                }
            }
        }
    })
}

pub(crate) fn save_book_recommendations_tool_def() -> serde_json::Value {
    serde_json::json!({
        "name": "save_book_recommendations",
        "description": "Save one or more book recommendations to the authenticated user's private Tastebase shelf. Use this when giving the user a round of recommendations. Recommending the same title and author again refreshes its summary and reason without erasing reading status or feedback.",
        "inputSchema": {
            "type": "object",
            "required": ["recommendations"],
            "properties": {
                "recommendations": {
                    "type": "array",
                    "minItems": 1,
                    "maxItems": 20,
                    "items": {
                        "type": "object",
                        "required": ["title", "author", "summary", "why_recommended"],
                        "properties": {
                            "title": {
                                "type": "string",
                                "description": "The book's title."
                            },
                            "author": {
                                "type": "string",
                                "description": "The book's author or authors."
                            },
                            "summary": {
                                "type": "string",
                                "description": "A concise, spoiler-light summary of the book."
                            },
                            "why_recommended": {
                                "type": "string",
                                "description": "A personalized explanation of why this book suits the user, informed by their prior ratings and writeups when available."
                            }
                        }
                    }
                }
            }
        }
    })
}

#[derive(Debug, Deserialize)]
struct ListBookRecommendationsParams {
    status: Option<BookStatus>,
}

#[derive(Debug, Deserialize)]
struct SaveBookRecommendationsParams {
    recommendations: Vec<BookRecommendationParam>,
}

#[derive(Debug, Deserialize)]
struct BookRecommendationParam {
    title: String,
    author: String,
    summary: String,
    why_recommended: String,
}

pub(crate) async fn dispatch_list_book_recommendations(
    msg_id: Option<serde_json::Value>,
    state: &AppState,
    user: &UserContext,
    arguments: serde_json::Value,
) -> JsonRpcResponse {
    let params: ListBookRecommendationsParams = match serde_json::from_value(arguments) {
        Ok(params) => params,
        Err(error) => {
            tracing::warn!(error = %error, "list_book_recommendations validation failed");
            return tool_text_response(msg_id, error.to_string(), true);
        }
    };
    let operation = mcp_operation(state, "tastebase.mcp.tools.list_book_recommendations")
        .with_detail("book.status_filter", params.status.is_some());

    match operation
        .observe(async { list_book_recommendations(state, user, params.status).await })
        .await
    {
        Ok(result) => tool_json_response(msg_id, &result),
        Err(error) => {
            tracing::error!(error = %error, "list_book_recommendations failed");
            tool_text_response(
                msg_id,
                format!("list_book_recommendations failed: {error}"),
                true,
            )
        }
    }
}

pub(crate) async fn dispatch_save_book_recommendations(
    msg_id: Option<serde_json::Value>,
    state: &AppState,
    user: &UserContext,
    arguments: serde_json::Value,
) -> JsonRpcResponse {
    let params: SaveBookRecommendationsParams = match serde_json::from_value(arguments) {
        Ok(params) => params,
        Err(error) => {
            tracing::warn!(error = %error, "save_book_recommendations validation failed");
            return tool_text_response(msg_id, error.to_string(), true);
        }
    };
    let operation = mcp_operation(state, "tastebase.mcp.tools.save_book_recommendations")
        .with_detail(
            "book.recommendation_count",
            params.recommendations.len() as u64,
        );

    match operation
        .observe(async { save_book_recommendations(state, user, params).await })
        .await
    {
        Ok(result) => tool_json_response(msg_id, &result),
        Err(error) => {
            tracing::error!(error = %error, "save_book_recommendations failed");
            tool_text_response(
                msg_id,
                format!(
                    "save_book_recommendations failed: {error}. No recommendations were saved."
                ),
                true,
            )
        }
    }
}

async fn resolve_user_id(state: &AppState, user: &UserContext) -> Result<Uuid, AppError> {
    shared::db::resolve_user(&state.db, &user.sub, user.email.as_deref())
        .await
        .map_err(AppError::from)
}

async fn list_book_recommendations(
    state: &AppState,
    user: &UserContext,
    status: Option<BookStatus>,
) -> Result<serde_json::Value, AppError> {
    let user_id = resolve_user_id(state, user).await?;
    let books: Vec<BookRecommendation> = sqlx::query_as(
        "SELECT * FROM book_recommendations
         WHERE user_id = $1
           AND ($2::book_status IS NULL OR status = $2)
         ORDER BY recommended_at DESC",
    )
    .bind(user_id)
    .bind(status)
    .fetch_all(&state.db)
    .await?;

    tracing::info!(user_id = %user_id, count = books.len(), "book history listed via MCP");
    Ok(serde_json::json!({ "recommendations": books }))
}

async fn save_book_recommendations(
    state: &AppState,
    user: &UserContext,
    params: SaveBookRecommendationsParams,
) -> Result<serde_json::Value, AppError> {
    if params.recommendations.is_empty() || params.recommendations.len() > 20 {
        return Err(AppError::BadRequest(
            "recommendations must contain between 1 and 20 books".into(),
        ));
    }
    for recommendation in &params.recommendations {
        shared::validate::validate_book_recommendation(
            &recommendation.title,
            &recommendation.author,
            &recommendation.summary,
            &recommendation.why_recommended,
        )?;
    }

    let user_id = resolve_user_id(state, user).await?;
    let mut transaction = state.db.begin().await?;
    let mut saved = Vec::with_capacity(params.recommendations.len());

    for recommendation in params.recommendations {
        let title = shared::sanitize::clean(&recommendation.title)
            .trim()
            .to_owned();
        let author = shared::sanitize::clean(&recommendation.author)
            .trim()
            .to_owned();
        let summary = shared::sanitize::clean(&recommendation.summary)
            .trim()
            .to_owned();
        let why_recommended = shared::sanitize::clean(&recommendation.why_recommended)
            .trim()
            .to_owned();
        shared::validate::validate_book_recommendation(
            &title,
            &author,
            &summary,
            &why_recommended,
        )?;
        let row: (Uuid, String, String) = sqlx::query_as(
            "INSERT INTO book_recommendations
               (id, user_id, title, author, summary, why_recommended)
             VALUES ($1, $2, $3, $4, $5, $6)
             ON CONFLICT (user_id, lower(title), lower(author))
             DO UPDATE SET
               summary = EXCLUDED.summary,
               why_recommended = EXCLUDED.why_recommended,
               updated_at = now()
             RETURNING id, title, author",
        )
        .bind(Uuid::new_v4())
        .bind(user_id)
        .bind(title)
        .bind(author)
        .bind(summary)
        .bind(why_recommended)
        .fetch_one(&mut *transaction)
        .await?;
        saved.push(serde_json::json!({
            "id": row.0,
            "title": row.1,
            "author": row.2,
        }));
    }

    transaction.commit().await?;
    tracing::info!(user_id = %user_id, count = saved.len(), "book recommendations saved via MCP");
    Ok(serde_json::json!({
        "recommendations": saved,
        "url": "https://tastebase.ahara.io/books",
        "message": "Saved privately to the user's Books shelf."
    }))
}
