use sqlx::{PgPool, Postgres, Transaction, types::Json};
use uuid::Uuid;

use crate::error::AppError;
use crate::publication_types::{PublicationInput, PublicationRecommendation, PublicationStatus};

const SELECT: &str = "SELECT p.*, COALESCE(
    (SELECT jsonb_agg(jsonb_build_object('key', t.tag_key, 'value', t.tag_value)
        ORDER BY t.tag_key, t.tag_value)
     FROM publication_tags t WHERE t.publication_id = p.id), '[]'::jsonb) AS tags
    FROM publication_recommendations p";

pub async fn list(
    pool: &PgPool,
    status: Option<PublicationStatus>,
) -> Result<Vec<PublicationRecommendation>, sqlx::Error> {
    sqlx::query_as(&format!(
        "{SELECT} WHERE ($1::text IS NULL OR p.status = $1) ORDER BY p.recommended_at DESC, p.id"
    ))
    .bind(status)
    .fetch_all(pool)
    .await
}

pub async fn get(pool: &PgPool, id: Uuid) -> Result<PublicationRecommendation, AppError> {
    sqlx::query_as(&format!("{SELECT} WHERE p.id = $1"))
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::NotFound)
}

async fn replace_tags(
    transaction: &mut Transaction<'_, Postgres>,
    id: Uuid,
    tags: Option<Vec<crate::types::BookTag>>,
) -> Result<(), AppError> {
    let Some(tags) = tags else { return Ok(()) };
    sqlx::query("DELETE FROM publication_tags WHERE publication_id = $1")
        .bind(id)
        .execute(&mut **transaction)
        .await?;
    for tag in tags {
        sqlx::query(
            "INSERT INTO publication_tags (publication_id, tag_key, tag_value) VALUES ($1, $2, $3)",
        )
        .bind(id)
        .bind(tag.key)
        .bind(tag.value)
        .execute(&mut **transaction)
        .await?;
    }
    Ok(())
}

pub async fn save(
    pool: &PgPool,
    inputs: Vec<PublicationInput>,
) -> Result<Vec<PublicationRecommendation>, AppError> {
    if inputs.is_empty() || inputs.len() > 20 {
        return Err(AppError::BadRequest(
            "recommendations must contain between 1 and 20 publications".into(),
        ));
    }
    let inputs = inputs
        .into_iter()
        .map(crate::publication_validation::prepare)
        .collect::<Result<Vec<_>, _>>()?;
    let mut transaction = pool.begin().await?;
    let mut ids = Vec::new();
    for input in inputs {
        let id: Uuid = sqlx::query_scalar(
            "INSERT INTO publication_recommendations (title, publisher, summary, why_recommended, details)
             VALUES ($1, $2, $3, $4, $5)
             ON CONFLICT (lower(title), lower(COALESCE(publisher, '')))
             DO UPDATE SET summary = EXCLUDED.summary, why_recommended = EXCLUDED.why_recommended,
                details = EXCLUDED.details, updated_at = now()
             RETURNING id"
        ).bind(input.title).bind(input.publisher).bind(input.summary).bind(input.why_recommended)
            .bind(Json(input.details)).fetch_one(&mut *transaction).await?;
        replace_tags(&mut transaction, id, input.tags).await?;
        ids.push(id);
    }
    transaction.commit().await?;
    let mut saved = Vec::new();
    for id in ids {
        saved.push(get(pool, id).await?);
    }
    Ok(saved)
}

// Patch recommendation metadata only. Nested objects and arrays are complete replacements;
// omitted fields survive and explicit null clears optional fields.
pub fn merge_patch(
    mut current: serde_json::Value,
    patch: serde_json::Map<String, serde_json::Value>,
) -> Result<PublicationInput, AppError> {
    if patch.is_empty() {
        return Err(AppError::BadRequest(
            "provide at least one recommendation field to patch".into(),
        ));
    }
    for (key, value) in patch {
        let canonical = match key.as_str() {
            "why_recommended" => "whyRecommended",
            "publication_type" => "publicationType",
            "audience_level" => "audienceLevel",
            "editorial_home" => "editorialHome",
            other => other,
        };
        if !matches!(
            canonical,
            "title"
                | "publisher"
                | "summary"
                | "whyRecommended"
                | "homepage"
                | "publicationType"
                | "cadence"
                | "audienceLevel"
                | "editorialHome"
                | "subscriptions"
                | "outlook"
                | "tags"
        ) {
            return Err(AppError::BadRequest(format!("cannot patch {key}")));
        }
        current[canonical] = value;
    }
    let input = serde_json::from_value(current).map_err(|e| AppError::BadRequest(e.to_string()))?;
    crate::publication_validation::prepare(input)
}

pub async fn patch(
    pool: &PgPool,
    id: Uuid,
    fields: serde_json::Map<String, serde_json::Value>,
) -> Result<PublicationRecommendation, AppError> {
    let mut transaction = pool.begin().await?;
    // Serialize concurrent patches before merging so omitted fields cannot be lost.
    let current: PublicationRecommendation =
        sqlx::query_as(&format!("{SELECT} WHERE p.id = $1 FOR UPDATE OF p"))
            .bind(id)
            .fetch_optional(&mut *transaction)
            .await?
            .ok_or(AppError::NotFound)?;
    let input = PublicationInput {
        title: current.title,
        publisher: current.publisher,
        summary: current.summary,
        why_recommended: current.why_recommended,
        details: current.details.0,
        tags: None,
    };
    let input = merge_patch(
        serde_json::to_value(input).map_err(|e| AppError::Internal(e.to_string()))?,
        fields,
    )?;
    sqlx::query(
        "UPDATE publication_recommendations SET title = $2, publisher = $3,
        summary = $4, why_recommended = $5, details = $6, updated_at = now() WHERE id = $1",
    )
    .bind(id)
    .bind(input.title)
    .bind(input.publisher)
    .bind(input.summary)
    .bind(input.why_recommended)
    .bind(Json(input.details))
    .execute(&mut *transaction)
    .await?;
    replace_tags(&mut transaction, id, input.tags).await?;
    transaction.commit().await?;
    get(pool, id).await
}
