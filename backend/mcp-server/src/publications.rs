use serde::Deserialize;
use serde_json::{Value, json};
use shared::{
    AppState,
    error::AppError,
    publication_types::{PublicationInput, PublicationStatus},
};
use uuid::Uuid;

use crate::{JsonRpcResponse, mcp_operation, tool_json_response, tool_text_response};

fn object(properties: Value, required: &[&str]) -> Value {
    json!({"type":"object", "properties":properties, "required":required, "additionalProperties":false})
}

fn optional(schema: Value) -> Value {
    json!({"anyOf":[schema, {"type":"null"}]})
}

fn price_schema() -> Value {
    object(
        json!({
            "amount":{"type":"string", "pattern":"^[0-9]+(\\.[0-9]+)?$", "description":"Exact nonnegative decimal price, for example 49.95. No currency conversion."},
            "currency":{"type":"string", "pattern":"^[A-Z]{3}$"},
            "period":{"enum":["month","year","issue"]},
            "price_type":optional(json!({"enum":["standard","introductory"]}))
        }),
        &["amount", "currency", "period"],
    )
}

fn subscription_schema() -> Value {
    object(
        json!({
            "label":{"type":"string"},
            "formats":{"type":"array", "minItems":1, "maxItems":2, "uniqueItems":true, "items":{"enum":["print","digital"]}},
            "region":optional(json!({"type":"string"})),
            "access_model":{"enum":["subscription","membership","free","institutional"]},
            "price":optional(price_schema()),
            "url":optional(json!({"type":"string", "format":"uri"})),
            "verified_at":optional(json!({"type":"string", "format":"date", "description":"Date this option's price and availability were checked. Omit when not verified; never invent prices or URLs."})),
            "notes":optional(json!({"type":"string"}))
        }),
        &["label", "formats", "access_model"],
    )
}

fn recommendation_properties() -> Value {
    json!({
        "title":{"type":"string"},
        "publisher":optional(json!({"type":"string"})),
        "homepage":optional(json!({"type":"string", "format":"uri"})),
        "summary":{"type":"string", "description":"What the publication is."},
        "why_recommended":{"type":"string", "description":"Why it belongs on the owner's shelf, informed by prior feedback."},
        "publication_type":{"enum":["magazine","journal","review","newsletter","other"]},
        "cadence":object(json!({"label":{"type":"string"}, "issues_per_year":optional(json!({"type":"integer", "minimum":1}))}), &["label"]),
        "audience_level":{"enum":["general","informed-generalist","professional","academic-adjacent","academic"]},
        "editorial_home":optional(object(json!({"country":optional(json!({"type":"string"})), "city":optional(json!({"type":"string"}))}), &[])),
        "subscriptions":{"type":"array", "maxItems":20, "items":subscription_schema(), "description":"Complete set of subscription options; use [] when unknown. Preserve region, billing period and verification date separately for each option."},
        "outlook":optional(object(json!({"summary":{"type":"string"}, "relationship":{"enum":["aligned","compatible","neutral","contrast","mixed"]}}), &["summary","relationship"])),
        "tags":{"type":"array", "maxItems":32, "items":object(json!({"key":{"type":"string"},"value":{"type":"string"}}), &["key","value"]),
            "description":"Shared book/publication vocabulary: category, topic, style, fit, outlook. Multiple values per key are allowed. Fit measures usefulness; outlook.relationship describes intellectual alignment, so high fit and contrast can coexist."}
    })
}

pub(crate) fn tool_defs() -> Vec<Value> {
    let mut patch_properties = recommendation_properties();
    patch_properties["id"] = json!({"type":"string", "format":"uuid"});
    patch_properties["update"] = json!({"type":"boolean", "default":false});
    let mut patch_schema = object(patch_properties, &["id"]);
    patch_schema["minProperties"] = json!(2);
    vec![
        json!({"name":"list_publication_recommendations", "description":"Read the complete public publication history, subscription options, issues and owner feedback before recommending more publications or saving reviews.",
            "inputSchema":object(json!({"status":{"enum":["recommended","subscribed","cancelled","not_interested"]}}), &[])}),
        json!({"name":"save_publication_recommendations", "description":"Save initial publication recommendations to the public bookshelf. First read recommendation history and get_publication_tag_corpus. Reuse existing tags. Existing case-insensitive title/publisher identities are rejected unless update=true (default false); a conflict rejects the entire batch. Updating recommendations is normally incorrect: add owner reviews instead, using save_publication_issue_review for magazines. Only set update=true for explicitly requested metadata corrections. It replaces recommendation metadata including subscriptions; omitted tags, subscription status, feedback and issues survive. Use patch by ID to correct title or publisher.",
            "inputSchema":object(json!({"update":{"type":"boolean", "default":false}, "recommendations":{"type":"array","minItems":1,"maxItems":20,
                "items":object(recommendation_properties(), &["title","summary","why_recommended","publication_type","cadence","audience_level","subscriptions"])}}), &["recommendations"])}),
        json!({"name":"patch_publication_recommendation", "description":"Correct publication recommendation metadata by ID only when explicitly requested, with update=true (default false). Updating recommendations is normally incorrect: add reviews instead, using save_publication_issue_review for magazines. Read the record and shared tag corpus first. Omitted fields survive; null clears optional fields; supplied nested objects and arrays replace complete values. Owner status, feedback and issues cannot be patched.", "inputSchema":patch_schema}),
        json!({"name":"get_publication_tag_corpus", "description":"Get the shared book and publication tag vocabulary. Reuse existing keys and values; category, topic, style, fit and outlook are reusable keys. Multiple outlook values are allowed. Fit and outlook alignment are separate dimensions.", "inputSchema":object(json!({}), &[])}),
    ]
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ListInput {
    status: Option<PublicationStatus>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SaveInput {
    recommendations: Vec<PublicationInput>,
    #[serde(default)]
    update: bool,
}

async fn execute(state: &AppState, name: &str, arguments: Value) -> Result<Value, AppError> {
    match name {
        "list_publication_recommendations" => {
            let input: ListInput = serde_json::from_value(arguments).map_err(bad_input)?;
            Ok(
                json!({"recommendations":shared::publications::list(&state.db, input.status).await?}),
            )
        }
        "save_publication_recommendations" => {
            let input: SaveInput = serde_json::from_value(arguments).map_err(bad_input)?;
            Ok(
                json!({"recommendations":shared::publications::save(&state.db, input.recommendations, input.update).await?, "url":"https://tastebase.ahara.io/books"}),
            )
        }
        "patch_publication_recommendation" => {
            let mut fields = arguments
                .as_object()
                .cloned()
                .ok_or_else(|| AppError::BadRequest("arguments must be an object".into()))?;
            let id: Uuid = serde_json::from_value(fields.remove("id").unwrap_or(Value::Null))
                .map_err(bad_input)?;
            let update: bool =
                serde_json::from_value(fields.remove("update").unwrap_or(json!(false)))
                    .map_err(bad_input)?;
            Ok(
                json!({"recommendation":shared::publications::patch(&state.db, id, fields, update).await?}),
            )
        }
        _ => Err(AppError::BadRequest("unknown publication tool".into())),
    }
}

fn bad_input(error: serde_json::Error) -> AppError {
    AppError::BadRequest(error.to_string())
}

pub(crate) async fn dispatch(
    msg_id: Option<Value>,
    state: &AppState,
    name: &str,
    arguments: Value,
) -> JsonRpcResponse {
    match mcp_operation(state, "tastebase.mcp.publications")
        .with_detail("tool.name", name.to_owned())
        .observe(execute(state, name, arguments))
        .await
    {
        Ok(value) => tool_json_response(msg_id, &value),
        Err(error) => tool_text_response(msg_id, error.to_string(), true),
    }
}
