use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::types::{BookRecommendation, BookStatus, BookTag};

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
