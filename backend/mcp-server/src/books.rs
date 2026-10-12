use serde::{Deserialize, Deserializer};
use shared::AppState;
use shared::books::PreparedBookRecommendation;
use shared::error::AppError;
use shared::types::{BookRecommendation, BookStatus, BookTag, UserContext};
use std::collections::BTreeMap;
use uuid::Uuid;

use crate::{JsonRpcResponse, mcp_operation, tool_json_response, tool_text_response};

pub(crate) fn list_book_recommendations_tool_def() -> serde_json::Value {
    serde_json::json!({
        "name": "list_book_recommendations",
        "description": "List the Tastebase owner's complete book recommendation history, including IDs, page counts, key/value tags, purchase links, reading status, 1-5 ratings, writeups. Call this before making another round of recommendations so prior suggestions and feedback inform the next choices. All recommendations and feedback are publicly visible on the bookshelf.",
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
        "description": "Save initial book recommendations to the public Tastebase shelf. Read list_book_recommendations and get_book_tag_corpus first; reuse existing tag keys and values wherever accurate. Existing case-insensitive title/author identities are rejected unless update=true. Updating recommendations is normally incorrect: use save_book_review to record the owner's experience from your conversation instead. Only set update=true for an explicitly requested metadata correction; it preserves reading status and feedback. A conflict rejects the entire batch.",
        "inputSchema": {
            "type": "object",
            "required": ["recommendations"],
            "properties": {
                "update": { "type": "boolean", "default": false, "description": "Explicit permission to change existing recommendation metadata. Normally leave false and save a review instead." },
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
                                "description": "A personalized explanation of why this book suits the owner, informed by prior ratings and writeups when available."
                            },
                            "page_count": {
                                "type": "integer",
                                "minimum": 1,
                                "description": "Page count for the edition associated with the purchase link. Omit when uncertain."
                            },
                            "tags": {
                                "type": "array",
                                "maxItems": 32,
                                "description": "Reusable key/value tags, for example {\"key\":\"category\",\"value\":\"psychology\"} or {\"key\":\"style\",\"value\":\"academic\"}. Multiple values may use the same key. Call get_book_tag_corpus first and reuse its vocabulary whenever accurate.",
                                "items": {
                                    "type": "object",
                                    "required": ["key", "value"],
                                    "properties": {
                                        "key": { "type": "string" },
                                        "value": { "type": "string" }
                                    }
                                }
                            },
                            "purchase_link": {
                                "type": "string",
                                "format": "uri",
                                "description": "A verified direct purchase URL for this edition. Prefer an Amazon product page when Amazon has the book; otherwise use a reputable publisher or bookseller. Never invent a URL."
                            }
                        }
                    }
                }
            }
        }
    })
}

pub(crate) fn patch_book_recommendation_tool_def() -> serde_json::Value {
    serde_json::json!({
        "name": "patch_book_recommendation",
        "description": "Correct existing book recommendation metadata only when explicitly requested, with update=true (default false). Updating recommendations is normally incorrect: use save_book_review for the owner's feedback instead. Read list_book_recommendations first and use its ID. Reading status and feedback survive. Before changing tags, read get_book_tag_corpus and reuse existing keys and values.",
        "inputSchema": {
            "type": "object",
            "required": ["id"],
            "minProperties": 2,
            "properties": {
                "update": { "type": "boolean", "default": false },
                "id": {
                    "type": "string",
                    "format": "uuid",
                    "description": "The existing recommendation ID."
                },
                "title": { "type": "string" },
                "author": { "type": "string" },
                "summary": { "type": "string" },
                "why_recommended": { "type": "string" },
                "page_count": {
                    "anyOf": [
                        { "type": "integer", "minimum": 1 },
                        { "type": "null" }
                    ],
                    "description": "Page count for the linked edition. Set null to clear it."
                },
                "tags": {
                    "type": "array",
                    "maxItems": 32,
                    "description": "The complete replacement set of key/value tags. Multiple values may share a key. An empty array clears all tags.",
                    "items": {
                        "type": "object",
                        "required": ["key", "value"],
                        "properties": {
                            "key": { "type": "string" },
                            "value": { "type": "string" }
                        }
                    }
                },
                "purchase_link": {
                    "anyOf": [
                        { "type": "string", "format": "uri" },
                        { "type": "null" }
                    ],
                    "description": "A verified direct purchase URL. Prefer Amazon when available, otherwise a reputable publisher or bookseller. Set null to clear it; never invent a URL."
                }
            }
        }
    })
}

pub(crate) fn get_book_tag_corpus_tool_def() -> serde_json::Value {
    serde_json::json!({
        "name": "get_book_tag_corpus",
        "description": "Fetch the shared book and publication tag vocabulary with an item count for every key/value pair. Call this before saving or patching tags. Reuse existing keys almost always. Reuse existing values when they accurately describe the book; create new values more readily than new keys, while avoiding synonyms and spelling variants so sorting and filtering remain useful. If the corpus is empty, introduce only a small set of broad, durable keys.",
        "inputSchema": {
            "type": "object",
            "properties": {}
        }
    })
}

#[derive(Debug, Deserialize)]
struct ListBookRecommendationsParams {
    status: Option<BookStatus>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SaveBookRecommendationsParams {
    recommendations: Vec<BookRecommendationParam>,
    #[serde(default)]
    update: bool,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct BookRecommendationParam {
    title: String,
    author: String,
    summary: String,
    why_recommended: String,
    page_count: Option<i32>,
    tags: Option<Vec<BookTag>>,
    purchase_link: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct PatchBookRecommendationParams {
    id: Uuid,
    #[serde(default)]
    update: bool,
    title: Option<String>,
    author: Option<String>,
    summary: Option<String>,
    why_recommended: Option<String>,
    #[serde(default, deserialize_with = "deserialize_present_option")]
    page_count: Option<Option<i32>>,
    tags: Option<Vec<BookTag>>,
    #[serde(default, deserialize_with = "deserialize_present_option")]
    purchase_link: Option<Option<String>>,
}

fn deserialize_present_option<'de, D, T>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(deserializer).map(Some)
}

pub(crate) async fn dispatch_list_book_recommendations(
    msg_id: Option<serde_json::Value>,
    state: &AppState,
    _user: &UserContext,
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
        .observe(async { list_book_recommendations(state, params.status).await })
        .await
    {
        Ok(result) => tool_json_response(msg_id, &result),
        Err(error) => tool_error_response(msg_id, "list_book_recommendations", error),
    }
}

pub(crate) async fn dispatch_save_book_recommendations(
    msg_id: Option<serde_json::Value>,
    state: &AppState,
    _user: &UserContext,
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
        .observe(async { save_book_recommendations(state, params).await })
        .await
    {
        Ok(result) => tool_json_response(msg_id, &result),
        Err(error) => tool_error_response(msg_id, "save_book_recommendations", error),
    }
}

pub(crate) async fn dispatch_patch_book_recommendation(
    msg_id: Option<serde_json::Value>,
    state: &AppState,
    _user: &UserContext,
    arguments: serde_json::Value,
) -> JsonRpcResponse {
    let params: PatchBookRecommendationParams = match serde_json::from_value(arguments) {
        Ok(params) => params,
        Err(error) => {
            tracing::warn!(error = %error, "patch_book_recommendation validation failed");
            return tool_text_response(msg_id, error.to_string(), true);
        }
    };
    let operation = mcp_operation(state, "tastebase.mcp.tools.patch_book_recommendation")
        .with_detail("book.id", params.id.to_string());

    match operation
        .observe(async { patch_book_recommendation(state, params).await })
        .await
    {
        Ok(result) => tool_json_response(msg_id, &result),
        Err(error) => tool_error_response(msg_id, "patch_book_recommendation", error),
    }
}

pub(crate) async fn dispatch_get_book_tag_corpus(
    msg_id: Option<serde_json::Value>,
    state: &AppState,
    _user: &UserContext,
) -> JsonRpcResponse {
    let operation = mcp_operation(state, "tastebase.mcp.tools.get_book_tag_corpus");
    match operation
        .observe(async { get_book_tag_corpus(state).await })
        .await
    {
        Ok(result) => tool_json_response(msg_id, &result),
        Err(error) => tool_error_response(msg_id, "get_book_tag_corpus", error),
    }
}

fn tool_error_response(
    msg_id: Option<serde_json::Value>,
    tool_name: &str,
    error: AppError,
) -> JsonRpcResponse {
    tracing::error!(error = %error, tool_name, "book MCP tool failed");
    tool_text_response(msg_id, format!("{tool_name} failed: {error}"), true)
}

async fn list_book_recommendations(
    state: &AppState,
    status: Option<BookStatus>,
) -> Result<serde_json::Value, AppError> {
    let books = shared::books::list_recommendations(&state.db, status).await?;
    tracing::info!(count = books.len(), "book history listed via MCP");
    Ok(serde_json::json!({ "recommendations": books }))
}

fn clean(value: String) -> String {
    shared::sanitize::clean(&value).trim().to_owned()
}

fn clean_purchase_link(value: String) -> Option<String> {
    let value = clean(value);
    (!value.is_empty()).then_some(value)
}

fn prepare_recommendation(
    recommendation: BookRecommendationParam,
) -> Result<PreparedBookRecommendation, AppError> {
    let prepared = PreparedBookRecommendation {
        title: clean(recommendation.title),
        author: clean(recommendation.author),
        summary: clean(recommendation.summary),
        why_recommended: clean(recommendation.why_recommended),
        page_count: recommendation.page_count,
        tags: recommendation
            .tags
            .map(shared::validate::normalize_book_tags)
            .transpose()?,
        purchase_link: recommendation.purchase_link.and_then(clean_purchase_link),
    };
    shared::validate::validate_book_recommendation(
        &prepared.title,
        &prepared.author,
        &prepared.summary,
        &prepared.why_recommended,
    )?;
    shared::validate::validate_book_metadata(
        prepared.page_count,
        prepared.purchase_link.as_deref(),
    )?;
    Ok(prepared)
}

async fn save_book_recommendations(
    state: &AppState,
    params: SaveBookRecommendationsParams,
) -> Result<serde_json::Value, AppError> {
    if params.recommendations.is_empty() || params.recommendations.len() > 20 {
        return Err(AppError::BadRequest(
            "recommendations must contain between 1 and 20 books".into(),
        ));
    }
    let recommendations = params
        .recommendations
        .into_iter()
        .map(prepare_recommendation)
        .collect::<Result<Vec<_>, _>>()?;

    let saved =
        shared::books::save_recommendations(&state.db, recommendations, params.update).await?;

    tracing::info!(count = saved.len(), "book recommendations saved via MCP");
    Ok(serde_json::json!({
        "recommendations": saved,
        "url": "https://tastebase.ahara.io/books",
        "message": "Saved to the public bookshelf."
    }))
}

async fn patch_book_recommendation(
    state: &AppState,
    params: PatchBookRecommendationParams,
) -> Result<serde_json::Value, AppError> {
    shared::shelf_reviews::require_update(params.update)?;
    let title = params.title.map(clean);
    let author = params.author.map(clean);
    let summary = params.summary.map(clean);
    let why_recommended = params.why_recommended.map(clean);
    let page_count = params.page_count;
    let tags = params
        .tags
        .map(shared::validate::normalize_book_tags)
        .transpose()?;
    let purchase_link = match params.purchase_link {
        None => None,
        Some(None) => Some(None),
        Some(Some(value)) => Some(clean_purchase_link(value)),
    };

    if title.is_none()
        && author.is_none()
        && summary.is_none()
        && why_recommended.is_none()
        && page_count.is_none()
        && tags.is_none()
        && purchase_link.is_none()
    {
        return Err(AppError::BadRequest(
            "provide at least one recommendation field to patch".into(),
        ));
    }

    shared::validate::validate_book_recommendation_patch(
        title.as_deref(),
        author.as_deref(),
        summary.as_deref(),
        why_recommended.as_deref(),
        page_count,
        purchase_link.as_ref().map(|value| value.as_deref()),
    )?;

    let mut transaction = state.db.begin().await?;
    let updated_id: Uuid = sqlx::query_scalar(
        "UPDATE book_recommendations
         SET title = COALESCE($2, title),
             author = COALESCE($3, author),
             summary = COALESCE($4, summary),
             why_recommended = COALESCE($5, why_recommended),
             page_count = CASE WHEN $6 THEN $7 ELSE page_count END,
             purchase_link = CASE WHEN $8 THEN $9 ELSE purchase_link END,
             updated_at = now()
         WHERE id = $1
         RETURNING id",
    )
    .bind(params.id)
    .bind(title)
    .bind(author)
    .bind(summary)
    .bind(why_recommended)
    .bind(page_count.is_some())
    .bind(page_count.flatten())
    .bind(purchase_link.is_some())
    .bind(purchase_link.flatten())
    .fetch_optional(&mut *transaction)
    .await?
    .ok_or(AppError::NotFound)?;

    if let Some(tags) = tags {
        shared::books::replace_tags(&mut transaction, updated_id, &tags).await?;
    }
    transaction.commit().await?;

    let book: BookRecommendation = shared::books::get_recommendation(&state.db, updated_id)
        .await?
        .ok_or(AppError::NotFound)?;
    tracing::info!(book_id = %updated_id, "book recommendation patched via MCP");
    Ok(serde_json::json!({
        "recommendation": book,
        "message": "Recommendation metadata updated. Reading feedback was unchanged."
    }))
}

async fn get_book_tag_corpus(state: &AppState) -> Result<serde_json::Value, AppError> {
    let rows = shared::books::tag_corpus(&state.db).await?;
    let mut values_by_key: BTreeMap<String, Vec<serde_json::Value>> = BTreeMap::new();
    for row in rows {
        values_by_key
            .entry(row.key)
            .or_default()
            .push(serde_json::json!({
                "value": row.value,
                "bookCount": row.book_count,
                "publicationCount": row.publication_count,
                "itemCount": row.item_count,
            }));
    }
    let corpus = values_by_key
        .into_iter()
        .map(|(key, values)| serde_json::json!({ "key": key, "values": values }))
        .collect::<Vec<_>>();

    tracing::info!(key_count = corpus.len(), "book tag corpus listed via MCP");
    Ok(serde_json::json!({
        "tagCorpus": corpus,
        "guidance": "Reuse existing keys almost always. Reuse an existing value when it fits; add a new value more readily than a new key, but avoid synonyms and spelling variants."
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn recommendation_updates_require_explicit_opt_in() {
        let params: SaveBookRecommendationsParams =
            serde_json::from_value(json!({"recommendations":[]})).unwrap();
        assert!(!params.update);
        let id = Uuid::new_v4();
        let params: PatchBookRecommendationParams =
            serde_json::from_value(json!({"id":id,"summary":"Correction"})).unwrap();
        assert!(!params.update);
        assert!(shared::shelf_reviews::require_update(params.update).is_err());
        let params: PatchBookRecommendationParams =
            serde_json::from_value(json!({"id":id,"summary":"Correction","update":true})).unwrap();
        assert!(shared::shelf_reviews::require_update(params.update).is_ok());
        assert!(
            serde_json::from_value::<PatchBookRecommendationParams>(
                json!({"id":id,"rating":4,"update":true})
            )
            .is_err()
        );
        for definition in [
            save_book_recommendations_tool_def(),
            patch_book_recommendation_tool_def(),
        ] {
            assert_eq!(
                definition["inputSchema"]["properties"]["update"]["default"],
                false
            );
        }
    }
}
