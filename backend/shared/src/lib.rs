pub mod auth;
pub mod books;
pub mod cors;
pub mod db;
pub mod error;
pub mod media;
pub mod sanitize;
pub mod types;
pub mod validate;

use sqlx::PgPool;

/// Shared application state passed to all handlers via axum State.
#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub s3: aws_sdk_s3::Client,
    pub media_bucket: String,
    pub telemetry: ahara_lambda_telemetry::TelemetryConfig,
}

impl AppState {
    pub async fn from_env(telemetry: ahara_lambda_telemetry::TelemetryConfig) -> Self {
        let config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
        let db = db::connect().await;
        let s3 = aws_sdk_s3::Client::new(&config);
        let media_bucket = std::env::var("MEDIA_BUCKET").expect("MEDIA_BUCKET env var required");
        Self {
            db,
            s3,
            media_bucket,
            telemetry,
        }
    }
}

pub fn telemetry_config(service_name: &'static str) -> ahara_lambda_telemetry::TelemetryConfig {
    ahara_lambda_telemetry::TelemetryConfig::new(service_name)
}
