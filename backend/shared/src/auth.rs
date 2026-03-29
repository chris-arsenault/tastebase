use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use base64::Engine;
use serde::Deserialize;

use crate::error::AppError;
use crate::types::UserContext;

#[derive(Debug, Deserialize)]
struct Claims {
    sub: String,
    email: Option<String>,
}

/// Decode user identity from a JWT. No cryptographic validation —
/// ALB jwt-validation handles that before requests reach the Lambda.
pub fn decode_token(token: &str) -> Result<UserContext, AppError> {
    let payload_b64 = token
        .split('.')
        .nth(1)
        .ok_or_else(|| AppError::Unauthorized("malformed token".into()))?;
    let payload_bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(payload_b64)
        .map_err(|_| AppError::Unauthorized("invalid token encoding".into()))?;
    let claims: Claims = serde_json::from_slice(&payload_bytes)
        .map_err(|_| AppError::Unauthorized("invalid token claims".into()))?;

    Ok(UserContext {
        sub: claims.sub,
        email: claims.email,
        user_id: None,
    })
}

/// Extract a bearer token from the Authorization header.
pub fn extract_bearer(auth_header: Option<&str>) -> Result<&str, AppError> {
    let header =
        auth_header.ok_or_else(|| AppError::Unauthorized("missing Authorization header".into()))?;
    header
        .strip_prefix("Bearer ")
        .or_else(|| header.strip_prefix("bearer "))
        .ok_or_else(|| AppError::Unauthorized("missing Bearer token".into()))
}

/// Axum extractor: required auth. Decodes the JWT to get user identity.
pub struct RequireAuth(pub UserContext);

impl<S: Send + Sync> FromRequestParts<S> for RequireAuth {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get("authorization")
            .and_then(|v| v.to_str().ok());
        let token = extract_bearer(auth_header)?;
        let ctx = decode_token(token)?;
        Ok(RequireAuth(ctx))
    }
}
