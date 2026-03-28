use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use uuid::Uuid;

use crate::types::{CognitoUser, User};

/// Create a connection pool from environment variables.
/// Reads DB_HOST, DB_PORT, DB_NAME, DB_USERNAME, DB_PASSWORD.
pub async fn connect() -> PgPool {
    let host = std::env::var("DB_HOST").expect("DB_HOST required");
    let port = std::env::var("DB_PORT").unwrap_or_else(|_| "5432".into());
    let name = std::env::var("DB_NAME").expect("DB_NAME required");
    let user = std::env::var("DB_USERNAME").expect("DB_USERNAME required");
    let pass = std::env::var("DB_PASSWORD").expect("DB_PASSWORD required");

    let url = format!("postgres://{user}:{pass}@{host}:{port}/{name}?sslmode=require");

    PgPoolOptions::new()
        .max_connections(5)
        .connect(&url)
        .await
        .expect("failed to connect to database")
}

/// Resolve a Cognito sub to an internal user ID via JIT provisioning.
/// Creates the user and cognito_users mapping if they don't exist.
pub async fn resolve_user(pool: &PgPool, cognito_sub: &str, email: Option<&str>) -> Result<Uuid, sqlx::Error> {
    // Check existing mapping
    let existing: Option<CognitoUser> = sqlx::query_as(
        "SELECT cognito_sub, user_id, email, linked_at FROM cognito_users WHERE cognito_sub = $1"
    )
    .bind(cognito_sub)
    .fetch_optional(pool)
    .await?;

    if let Some(cu) = existing {
        return Ok(cu.user_id);
    }

    // JIT provision: create user + mapping in a transaction
    let email = email.unwrap_or("");
    let mut tx = pool.begin().await?;

    let user: User = sqlx::query_as(
        "INSERT INTO users (email, display_name) VALUES ($1, $1) RETURNING id, email, display_name, created_at"
    )
    .bind(email)
    .fetch_one(&mut *tx)
    .await?;

    sqlx::query(
        "INSERT INTO cognito_users (cognito_sub, user_id, email) VALUES ($1, $2, $3)"
    )
    .bind(cognito_sub)
    .bind(user.id)
    .bind(email)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(user.id)
}
