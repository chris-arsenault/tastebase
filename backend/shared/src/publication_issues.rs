use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use sqlx::{PgPool, types::Json};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::{error::AppError, shelf_reviews::ReviewInput};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct IssueMetadata {
    pub title: Option<String>,
    pub volume: Option<String>,
    #[serde(alias = "issue_number")]
    pub issue_number: Option<String>,
    #[serde(alias = "publication_date")]
    pub publication_date: Option<String>,
    #[serde(alias = "page_count")]
    pub page_count: Option<i32>,
    pub url: Option<String>,
    pub summary: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct PublicationIssue {
    pub id: Uuid,
    pub publication_id: Uuid,
    pub label: String,
    #[serde(flatten)]
    pub metadata: Json<IssueMetadata>,
    pub rating: Option<i16>,
    pub writeup: String,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IssueInput {
    pub label: String,
    #[serde(default)]
    pub metadata: Map<String, Value>,
}

fn prepare(input: IssueInput) -> Result<IssueInput, AppError> {
    let label = crate::sanitize::clean(&input.label).trim().to_owned();
    if label.is_empty() || label.len() > 200 {
        return Err(AppError::BadRequest(
            "issue label must contain 1 to 200 bytes".into(),
        ));
    }
    let mut metadata = Map::new();
    for (key, value) in input.metadata {
        let canonical = match key.as_str() {
            "issue_number" => "issueNumber",
            "publication_date" => "publicationDate",
            "page_count" => "pageCount",
            other => other,
        };
        if metadata.insert(canonical.to_owned(), value).is_some() {
            return Err(AppError::BadRequest(format!(
                "duplicate issue field {canonical}"
            )));
        }
    }
    let mut typed: IssueMetadata = serde_json::from_value(Value::Object(metadata.clone()))
        .map_err(|e| AppError::BadRequest(e.to_string()))?;
    for (field, value, max) in [
        ("title", &mut typed.title, 1000),
        ("volume", &mut typed.volume, 200),
        ("issueNumber", &mut typed.issue_number, 200),
        ("url", &mut typed.url, 2000),
        ("summary", &mut typed.summary, 6000),
    ] {
        if let Some(value) = value {
            *value = crate::sanitize::clean(value).trim().to_owned();
            if value.is_empty() || value.len() > max {
                return Err(AppError::BadRequest(format!(
                    "{field} must contain 1 to {max} bytes"
                )));
            }
        }
    }
    crate::validate::validate_book_metadata(typed.page_count, typed.url.as_deref())?;
    if let Some(date) = &typed.publication_date {
        let format = time::format_description::parse("[year]-[month]-[day]").expect("date format");
        if date.len() != 10 || time::Date::parse(date, &format).is_err() {
            return Err(AppError::BadRequest(
                "publicationDate must be a valid YYYY-MM-DD date".into(),
            ));
        }
    }
    let normalized = serde_json::to_value(typed).map_err(|e| AppError::Internal(e.to_string()))?;
    for (key, value) in &mut metadata {
        *value = normalized[key].clone();
    }
    Ok(IssueInput { label, metadata })
}

pub async fn list(pool: &PgPool, publication_id: Uuid) -> Result<Vec<PublicationIssue>, AppError> {
    crate::publications::get(pool, publication_id).await?;
    Ok(sqlx::query_as(
        "SELECT * FROM publication_issues WHERE publication_id = $1 ORDER BY created_at DESC, id",
    )
    .bind(publication_id)
    .fetch_all(pool)
    .await?)
}

pub async fn save(
    pool: &PgPool,
    publication_id: Uuid,
    input: IssueInput,
) -> Result<PublicationIssue, AppError> {
    let input = prepare(input)?;
    let mut transaction = pool.begin().await?;
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM publication_recommendations WHERE id = $1 FOR KEY SHARE)",
    )
    .bind(publication_id)
    .fetch_one(&mut *transaction)
    .await?;
    if !exists {
        return Err(AppError::NotFound);
    }
    let issue = sqlx::query_as(
        "INSERT INTO publication_issues (publication_id, label, metadata) VALUES ($1, $2, $3)
         ON CONFLICT (publication_id, lower(label)) DO UPDATE
         SET metadata = publication_issues.metadata || EXCLUDED.metadata, updated_at = now()
         RETURNING *",
    )
    .bind(publication_id)
    .bind(input.label)
    .bind(Json(input.metadata))
    .fetch_one(&mut *transaction)
    .await?;
    transaction.commit().await?;
    Ok(issue)
}

pub async fn save_review(
    pool: &PgPool,
    issue_id: Uuid,
    input: ReviewInput,
) -> Result<PublicationIssue, AppError> {
    let input = input.prepare()?;
    sqlx::query_as(
        "UPDATE publication_issues SET rating = $2, writeup = $3, updated_at = now()
         WHERE id = $1 RETURNING *",
    )
    .bind(issue_id)
    .bind(input.rating)
    .bind(input.writeup)
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn metadata_preserves_omissions_and_explicit_null_and_cleans_text() {
        let input = prepare(serde_json::from_value(json!({"label":" October 2026 ", "metadata":{
            "title":"<b>Special issue</b>", "issue_number":"10", "publication_date":"2026-10-01", "url":null
        }})).unwrap()).unwrap();
        assert_eq!(input.label, "October 2026");
        assert_eq!(input.metadata["title"], "Special issue");
        assert_eq!(input.metadata["issueNumber"], "10");
        assert!(input.metadata["url"].is_null());
        assert!(!input.metadata.contains_key("volume"));
    }

    #[test]
    fn rejects_invalid_issue_metadata() {
        for metadata in [
            json!({"publication_date":"2026-02-30"}),
            json!({"page_count":0}),
            json!({"url":"javascript:alert(1)"}),
            json!({"rating":5}),
            json!({"title":" "}),
            json!({"page_count":1,"pageCount":2}),
        ] {
            assert!(
                prepare(IssueInput {
                    label: "Issue".into(),
                    metadata: metadata.as_object().unwrap().clone()
                })
                .is_err()
            );
        }
        assert!(
            prepare(IssueInput {
                label: " ".into(),
                metadata: Map::new()
            })
            .is_err()
        );
    }
}
