use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{error::AppError, types::BookRecommendation};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReviewInput {
    pub rating: i16,
    pub writeup: String,
}

impl ReviewInput {
    pub fn prepare(mut self) -> Result<Self, AppError> {
        self.writeup = crate::sanitize::clean(&self.writeup).trim().to_owned();
        crate::validate::validate_book_review(self.rating, &self.writeup)?;
        Ok(self)
    }
}

pub fn require_update(update: bool) -> Result<(), AppError> {
    if !update {
        return Err(update_required());
    }
    Ok(())
}

pub fn update_required() -> AppError {
    AppError::BadRequest(
        "Recommendation already exists. Updating recommendations is normally incorrect; save the owner's review instead. Set update=true only for an explicitly requested metadata correction.".into(),
    )
}

pub async fn save_book_review(
    pool: &PgPool,
    id: Uuid,
    input: ReviewInput,
) -> Result<BookRecommendation, AppError> {
    let input = input.prepare()?;
    let changed = sqlx::query(
        "UPDATE book_recommendations SET rating = $2, writeup = $3,
         status = CASE WHEN status = 'did_not_finish'::book_status THEN status
                       ELSE 'read'::book_status END,
         read_at = CASE WHEN status = 'did_not_finish'::book_status THEN read_at
                        ELSE COALESCE(read_at, now()) END,
         updated_at = now() WHERE id = $1",
    )
    .bind(id)
    .bind(input.rating)
    .bind(input.writeup)
    .execute(pool)
    .await?;
    if changed.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    crate::books::get_recommendation(pool, id)
        .await?
        .ok_or(AppError::NotFound)
}
