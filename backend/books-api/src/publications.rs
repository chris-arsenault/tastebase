use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{get, put},
};
use serde::Deserialize;
use shared::{AppState, auth::RequireAuth, error::AppError, publication_types::PublicationStatus};
use uuid::Uuid;

use crate::{SaveReviewInput, book_operation};

async fn list(State(state): State<AppState>) -> Result<Json<serde_json::Value>, AppError> {
    book_operation(&state, "tastebase.publications.list")
        .observe(async {
            Ok(Json(
                serde_json::json!({"data": shared::publications::list(&state.db, None).await?}),
            ))
        })
        .await
}

#[derive(Deserialize)]
struct StatusInput {
    status: PublicationStatus,
}

async fn status(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    RequireAuth(_user): RequireAuth,
    Json(input): Json<StatusInput>,
) -> Result<Json<serde_json::Value>, AppError> {
    book_operation(&state, "tastebase.publications.update_status").with_detail("publication.id", id.to_string())
        .observe(async {
            let changed = sqlx::query("UPDATE publication_recommendations SET status = $2, updated_at = now() WHERE id = $1")
                .bind(id).bind(input.status).execute(&state.db).await?;
            if changed.rows_affected() == 0 { return Err(AppError::NotFound); }
            Ok(Json(serde_json::json!({"data": shared::publications::get(&state.db, id).await?})))
        }).await
}

async fn review(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    RequireAuth(_user): RequireAuth,
    Json(input): Json<SaveReviewInput>,
) -> Result<Json<serde_json::Value>, AppError> {
    book_operation(&state, "tastebase.publications.save_review").with_detail("publication.id", id.to_string())
        .observe(async {
            let writeup = shared::sanitize::clean(&input.writeup).trim().to_owned();
            shared::validate::validate_book_review(input.rating, &writeup)?;
            let changed = sqlx::query("UPDATE publication_recommendations SET rating = $2, writeup = $3, updated_at = now() WHERE id = $1")
                .bind(id).bind(input.rating).bind(writeup).execute(&state.db).await?;
            if changed.rows_affected() == 0 { return Err(AppError::NotFound); }
            Ok(Json(serde_json::json!({"data": shared::publications::get(&state.db, id).await?})))
        }).await
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/books/publications", get(list))
        .route("/books/publications/{id}/status", put(status))
        .route("/books/publications/{id}/review", put(review))
}
