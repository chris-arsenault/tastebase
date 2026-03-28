pub mod auth;
pub mod db;
pub mod error;
pub mod media;
pub mod types;

use sqlx::PgPool;

/// Shared application state passed to all handlers via axum State.
#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub s3: aws_sdk_s3::Client,
    pub media_bucket: String,
}

impl AppState {
    pub async fn from_env() -> Self {
        let config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
        let db = db::connect().await;
        let s3 = aws_sdk_s3::Client::new(&config);
        let media_bucket =
            std::env::var("MEDIA_BUCKET").expect("MEDIA_BUCKET env var required");
        Self {
            db,
            s3,
            media_bucket,
        }
    }
}

/// Initialize tracing for Lambda (JSON structured logs).
pub fn init_tracing() {
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .without_time() // Lambda adds timestamps
        .init();
}
