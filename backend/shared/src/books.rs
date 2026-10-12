use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::error::AppError;
use crate::types::{BookRecommendation, BookStatus, BookTag};

pub struct PreparedBookRecommendation {
    pub title: String,
    pub author: String,
    pub summary: String,
    pub why_recommended: String,
    pub page_count: Option<i32>,
    pub tags: Option<Vec<BookTag>>,
    pub purchase_link: Option<String>,
}

pub async fn save_recommendations(
    pool: &PgPool,
    recommendations: Vec<PreparedBookRecommendation>,
    update: bool,
) -> Result<Vec<BookRecommendation>, AppError> {
    let mut transaction = pool.begin().await?;
    let mut ids = Vec::with_capacity(recommendations.len());
    for recommendation in recommendations {
        let id: Option<Uuid> = sqlx::query_scalar(
            "INSERT INTO book_recommendations
               (id, title, author, summary, why_recommended, page_count, purchase_link)
             VALUES ($1, $2, $3, $4, $5, $6, $7)
             ON CONFLICT (lower(title), lower(author)) DO UPDATE SET
               summary = EXCLUDED.summary,
               why_recommended = EXCLUDED.why_recommended,
               page_count = COALESCE(EXCLUDED.page_count, book_recommendations.page_count),
               purchase_link = COALESCE(EXCLUDED.purchase_link, book_recommendations.purchase_link),
               updated_at = now()
             WHERE $8 RETURNING id",
        )
        .bind(Uuid::new_v4())
        .bind(recommendation.title)
        .bind(recommendation.author)
        .bind(recommendation.summary)
        .bind(recommendation.why_recommended)
        .bind(recommendation.page_count)
        .bind(recommendation.purchase_link)
        .bind(update)
        .fetch_optional(&mut *transaction)
        .await?;
        let id = id.ok_or_else(crate::shelf_reviews::update_required)?;
        if let Some(tags) = recommendation.tags {
            replace_tags(&mut transaction, id, &tags).await?;
        }
        ids.push(id);
    }
    transaction.commit().await?;
    let mut saved = Vec::with_capacity(ids.len());
    for id in ids {
        saved.push(
            get_recommendation(pool, id)
                .await?
                .ok_or(AppError::NotFound)?,
        );
    }
    Ok(saved)
}

pub async fn list_recommendations(
    pool: &PgPool,
    status: Option<BookStatus>,
) -> Result<Vec<BookRecommendation>, sqlx::Error> {
    sqlx::query_as(
        "SELECT b.*,
                COALESCE(
                    jsonb_agg(
                        jsonb_build_object('key', t.tag_key, 'value', t.tag_value)
                        ORDER BY t.tag_key, t.tag_value
                    ) FILTER (WHERE t.book_id IS NOT NULL),
                    '[]'::jsonb
                ) AS tags
         FROM book_recommendations b
         LEFT JOIN book_tags t ON t.book_id = b.id
         WHERE ($1::book_status IS NULL OR b.status = $1)
         GROUP BY b.id
         ORDER BY b.recommended_at DESC, b.id",
    )
    .bind(status)
    .fetch_all(pool)
    .await
}

pub async fn get_recommendation(
    pool: &PgPool,
    id: Uuid,
) -> Result<Option<BookRecommendation>, sqlx::Error> {
    sqlx::query_as(
        "SELECT b.*,
                COALESCE(
                    jsonb_agg(
                        jsonb_build_object('key', t.tag_key, 'value', t.tag_value)
                        ORDER BY t.tag_key, t.tag_value
                    ) FILTER (WHERE t.book_id IS NOT NULL),
                    '[]'::jsonb
                ) AS tags
         FROM book_recommendations b
         LEFT JOIN book_tags t ON t.book_id = b.id
         WHERE b.id = $1
         GROUP BY b.id",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn replace_tags(
    transaction: &mut Transaction<'_, Postgres>,
    book_id: Uuid,
    tags: &[BookTag],
) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM book_tags WHERE book_id = $1")
        .bind(book_id)
        .execute(&mut **transaction)
        .await?;

    for tag in tags {
        sqlx::query(
            "INSERT INTO book_tags (book_id, tag_key, tag_value)
             VALUES ($1, $2, $3)",
        )
        .bind(book_id)
        .bind(&tag.key)
        .bind(&tag.value)
        .execute(&mut **transaction)
        .await?;
    }

    Ok(())
}

#[derive(Debug, sqlx::FromRow)]
pub struct BookTagCorpusRow {
    pub key: String,
    pub value: String,
    pub book_count: i64,
    pub publication_count: i64,
    pub item_count: i64,
}

pub async fn tag_corpus(pool: &PgPool) -> Result<Vec<BookTagCorpusRow>, sqlx::Error> {
    sqlx::query_as(
        "SELECT tag_key AS key, tag_value AS value, count(*) AS item_count,
                count(*) FILTER (WHERE kind = 'book') AS book_count,
                count(*) FILTER (WHERE kind = 'publication') AS publication_count
         FROM (SELECT tag_key, tag_value, 'book' AS kind FROM book_tags
               UNION ALL SELECT tag_key, tag_value, 'publication' AS kind FROM publication_tags) shelf_tags
         GROUP BY tag_key, tag_value
         ORDER BY tag_key, count(*) DESC, tag_value",
    )
    .fetch_all(pool)
    .await
}
