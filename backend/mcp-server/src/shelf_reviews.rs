use serde::Deserialize;
use serde_json::{Value, json};
use shared::{
    AppState, error::AppError, publication_issues::IssueInput, shelf_reviews::ReviewInput,
};
use uuid::Uuid;

use crate::{JsonRpcResponse, mcp_operation, tool_json_response, tool_text_response};

fn object(properties: Value, required: &[&str]) -> Value {
    json!({"type":"object", "properties":properties, "required":required, "additionalProperties":false})
}

fn review_schema(id: &str) -> Value {
    object(
        json!({
            id:{"type":"string", "format":"uuid"},
            "rating":{"type":"integer", "minimum":1, "maximum":5},
            "writeup":{"type":"string", "minLength":1, "description":"The owner's feedback, grounded in their conversation. Never invent their opinions or rating."}
        }),
        &[id, "rating", "writeup"],
    )
}

pub(crate) fn tool_defs() -> Vec<Value> {
    vec![
        json!({"name":"save_book_review", "description":"Save a public 1–5 rating and writeup on the owner's behalf, based on what they tell you in conversation. Read list_book_recommendations for the ID and existing feedback. This saves or replaces the book's review and marks it read, except did_not_finish stays unchanged. Recommendation metadata is preserved. Use this for feedback rather than updating the original recommendation. Never infer a rating the owner has not supplied or agreed to.", "inputSchema":review_schema("id")}),
        json!({"name":"save_publication_review", "description":"Save or replace the owner's public review of a non-magazine publication from their conversation. Read list_publication_recommendations first. Recommendation metadata and subscription status survive. Magazine reviews must use save_publication_issue_review for a specific issue. Never invent the owner's opinions or rating.", "inputSchema":review_schema("id")}),
        json!({"name":"list_publication_issues", "description":"Read the issues, issue IDs, metadata and owner reviews for an existing publication. Call before creating an issue or saving a magazine review to reuse the correct issue.",
            "inputSchema":object(json!({"publication_id":{"type":"string", "format":"uuid"}}), &["publication_id"])}),
        json!({"name":"save_publication_issue", "description":"Create an issue or set its metadata without changing the original publication recommendation or any reviews. Read list_publication_issues first. Identity is publication_id plus case-insensitive label; reuse the existing label, such as October 2026 or Vol. 12 No. 3. Metadata fields omitted from this call survive; null clears a field. Dates must be exact YYYY-MM-DD; use the label for a month or season when the exact date is unknown. Do not invent metadata or URLs. Use the returned issue ID with save_publication_issue_review.",
            "inputSchema":object(json!({
                "publication_id":{"type":"string", "format":"uuid"},
                "label":{"type":"string", "minLength":1, "maxLength":200},
                "metadata":object(json!({
                    "title":{"type":["string","null"]},
                    "volume":{"type":["string","null"]},
                    "issue_number":{"type":["string","null"], "description":"A string supports combined issue numbers, for example 3/4."},
                    "publication_date":{"type":["string","null"], "format":"date"},
                    "page_count":{"type":["integer","null"], "minimum":1},
                    "url":{"type":["string","null"], "format":"uri"},
                    "summary":{"type":["string","null"]}
                }), &[])
            }), &["publication_id","label"])}),
        json!({"name":"save_publication_issue_review", "description":"Save or replace the owner's public 1–5 rating and writeup for one specific magazine/publication issue, grounded in their conversation. Read list_publication_issues for its ID, or create the issue with save_publication_issue first. Preserves issue metadata, all other issues, the original recommendation and subscription status. This is the normal tool for magazine feedback. Never invent the owner's opinions or rating.", "inputSchema":review_schema("issue_id")}),
    ]
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ReviewParams {
    #[serde(alias = "issue_id")]
    id: Uuid,
    rating: i16,
    writeup: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ListParams {
    publication_id: Uuid,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct IssueParams {
    publication_id: Uuid,
    label: String,
    #[serde(default)]
    metadata: serde_json::Map<String, Value>,
}

fn bad_input(error: serde_json::Error) -> AppError {
    AppError::BadRequest(error.to_string())
}

async fn execute(state: &AppState, name: &str, arguments: Value) -> Result<Value, AppError> {
    match name {
        "list_publication_issues" => {
            let params: ListParams = serde_json::from_value(arguments).map_err(bad_input)?;
            Ok(
                json!({"issues":shared::publication_issues::list(&state.db, params.publication_id).await?}),
            )
        }
        "save_publication_issue" => {
            let params: IssueParams = serde_json::from_value(arguments).map_err(bad_input)?;
            Ok(
                json!({"issue":shared::publication_issues::save(&state.db, params.publication_id, IssueInput { label:params.label, metadata:params.metadata }).await?}),
            )
        }
        _ => {
            let params: ReviewParams = serde_json::from_value(arguments).map_err(bad_input)?;
            let input = ReviewInput {
                rating: params.rating,
                writeup: params.writeup,
            };
            match name {
                "save_book_review" => Ok(
                    json!({"recommendation":shared::shelf_reviews::save_book_review(&state.db, params.id, input).await?}),
                ),
                "save_publication_review" => Ok(
                    json!({"recommendation":shared::publications::save_review(&state.db, params.id, input).await?}),
                ),
                "save_publication_issue_review" => Ok(
                    json!({"issue":shared::publication_issues::save_review(&state.db, params.id, input).await?}),
                ),
                _ => Err(AppError::BadRequest("unknown shelf review tool".into())),
            }
        }
    }
}

pub(crate) async fn dispatch(
    msg_id: Option<Value>,
    state: &AppState,
    name: &str,
    arguments: Value,
) -> JsonRpcResponse {
    match mcp_operation(state, "tastebase.mcp.shelf_reviews")
        .with_detail("tool.name", name.to_owned())
        .observe(execute(state, name, arguments))
        .await
    {
        Ok(value) => tool_json_response(msg_id, &value),
        Err(error) => tool_text_response(msg_id, error.to_string(), true),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn issue_metadata_and_review_inputs_match_the_tools() {
        let id = Uuid::new_v4();
        let params: IssueParams = serde_json::from_value(
            json!({"publication_id":id,"label":"October 2026","metadata":{"issue_number":"10"}}),
        )
        .unwrap();
        assert_eq!(params.label, "October 2026");
        assert_eq!(params.metadata["issue_number"], "10");
        assert!(
            serde_json::from_value::<IssueParams>(
                json!({"publication_id":id,"label":"October 2026","rating":5})
            )
            .is_err()
        );
        let review: ReviewParams =
            serde_json::from_value(json!({"issue_id":id,"rating":4,"writeup":"Good"})).unwrap();
        assert_eq!(review.id, id);
        let defs = tool_defs();
        let schema = &defs
            .iter()
            .find(|def| def["name"] == "save_publication_issue_review")
            .unwrap()["inputSchema"];
        assert!(schema["properties"].get("issue_id").is_some());
        assert_eq!(schema["required"], json!(["issue_id", "rating", "writeup"]));
    }
}
