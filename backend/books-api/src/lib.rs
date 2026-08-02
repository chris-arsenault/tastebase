use ahara_lambda_telemetry::Operation;
use axum::extract::{Path, State};
use axum::routing::{get, put};
use axum::{Json, Router};
use serde::Deserialize;
use shared::AppState;
use shared::auth::RequireAuth;
use shared::error::AppError;
use shared::types::{BookRecommendation, BookStatus};
use uuid::Uuid;

pub const SERVICE_NAME: &str = "tastebase-books-api";

fn book_operation(state: &AppState, name: &'static str) -> Operation {
    Operation::new(state.telemetry.clone(), name).with_domain("tastebase.books")
}

async fn list_books(
    State(state): State<AppState>,
    RequireAuth(_user): RequireAuth,
) -> Result<Json<serde_json::Value>, AppError> {
    book_operation(&state, "tastebase.books.list")
        .observe(async move {
            let books = shared::books::list_recommendations(&state.db, None, false).await?;

            tracing::info!(count = books.len(), "books listed");
            Ok(Json(serde_json::json!({ "data": books })))
        })
        .await
}

async fn list_public_books(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    book_operation(&state, "tastebase.books.list_public")
        .observe(async move {
            let books = shared::books::list_recommendations(&state.db, None, true).await?;

            tracing::info!(count = books.len(), "public books listed");
            Ok(Json(serde_json::json!({ "data": books })))
        })
        .await
}

#[derive(Debug, Deserialize)]
struct UpdateStatusInput {
    status: BookStatus,
}

async fn update_status(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    RequireAuth(_user): RequireAuth,
    Json(input): Json<UpdateStatusInput>,
) -> Result<Json<serde_json::Value>, AppError> {
    book_operation(&state, "tastebase.books.update_status")
        .with_detail("book.id", id.to_string())
        .with_detail("book.status", format!("{:?}", input.status))
        .observe(async move {
            let updated_id: Uuid = sqlx::query_scalar(
                "UPDATE book_recommendations
                 SET status = $1,
                     read_at = CASE
                       WHEN $1 = 'read'::book_status THEN COALESCE(read_at, now())
                       ELSE read_at
                     END,
                     updated_at = now()
                 WHERE id = $2
                 RETURNING id",
            )
            .bind(input.status)
            .bind(id)
            .fetch_optional(&state.db)
            .await?
            .ok_or(AppError::NotFound)?;
            let book = shared::books::get_recommendation(&state.db, updated_id)
                .await?
                .ok_or(AppError::NotFound)?;

            tracing::info!(book_id = %id, "book status updated");
            Ok(Json(serde_json::json!({ "data": book })))
        })
        .await
}

#[derive(Debug, Deserialize)]
struct SaveReviewInput {
    rating: i16,
    writeup: String,
}

async fn save_review(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    RequireAuth(_user): RequireAuth,
    Json(input): Json<SaveReviewInput>,
) -> Result<Json<serde_json::Value>, AppError> {
    book_operation(&state, "tastebase.books.save_review")
        .with_detail("book.id", id.to_string())
        .with_detail("book.rating", input.rating as i64)
        .observe(async move {
            let writeup = shared::sanitize::clean(&input.writeup).trim().to_owned();
            shared::validate::validate_book_review(input.rating, &writeup)?;
            let updated_id: Uuid = sqlx::query_scalar(
                "UPDATE book_recommendations
                 SET rating = $1,
                     writeup = $2,
                     status = CASE
                       WHEN status = 'did_not_finish'::book_status THEN status
                       ELSE 'read'::book_status
                     END,
                     read_at = CASE
                       WHEN status = 'did_not_finish'::book_status THEN read_at
                       ELSE COALESCE(read_at, now())
                     END,
                     updated_at = now()
                 WHERE id = $3
                 RETURNING id",
            )
            .bind(input.rating)
            .bind(writeup)
            .bind(id)
            .fetch_optional(&state.db)
            .await?
            .ok_or(AppError::NotFound)?;
            let book = shared::books::get_recommendation(&state.db, updated_id)
                .await?
                .ok_or(AppError::NotFound)?;

            tracing::info!(book_id = %id, "book review saved");
            Ok(Json(serde_json::json!({ "data": book })))
        })
        .await
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpdateVisibilityInput {
    is_public: bool,
}

async fn update_visibility(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    RequireAuth(_user): RequireAuth,
    Json(input): Json<UpdateVisibilityInput>,
) -> Result<Json<serde_json::Value>, AppError> {
    book_operation(&state, "tastebase.books.update_visibility")
        .with_detail("book.id", id.to_string())
        .with_detail("book.is_public", input.is_public)
        .observe(async move {
            let current: BookRecommendation = shared::books::get_recommendation(&state.db, id)
                .await?
                .ok_or(AppError::NotFound)?;

            if input.is_public && (current.rating.is_none() || current.writeup.trim().is_empty()) {
                return Err(AppError::BadRequest(
                    "Add a rating and review before sharing this book".into(),
                ));
            }

            let updated_id: Uuid = sqlx::query_scalar(
                "UPDATE book_recommendations
                 SET is_public = $1, updated_at = now()
                 WHERE id = $2
                 RETURNING id",
            )
            .bind(input.is_public)
            .bind(id)
            .fetch_optional(&state.db)
            .await?
            .ok_or(AppError::NotFound)?;
            let book = shared::books::get_recommendation(&state.db, updated_id)
                .await?
                .ok_or(AppError::NotFound)?;

            tracing::info!(book_id = %id, is_public = input.is_public, "book visibility updated");
            Ok(Json(serde_json::json!({ "data": book })))
        })
        .await
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/books", get(list_books))
        .route("/books/public", get(list_public_books))
        .route("/books/{id}/status", put(update_status))
        .route("/books/{id}/review", put(save_review))
        .route("/books/{id}/visibility", put(update_visibility))
        .layer(shared::cors::layer())
        .with_state(state)
}
