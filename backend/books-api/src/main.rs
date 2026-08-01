use books_api::{SERVICE_NAME, router};
use shared::AppState;

#[tokio::main]
async fn main() -> Result<(), lambda_http::Error> {
    let telemetry = shared::telemetry_config(SERVICE_NAME);
    ahara_lambda_telemetry::init_lambda_logging(&telemetry);
    tracing::info!("books-api starting");
    let state = AppState::from_env(telemetry.clone()).await;
    let app = router(state);
    ahara_lambda_telemetry::run_http_lambda(telemetry, app).await
}
