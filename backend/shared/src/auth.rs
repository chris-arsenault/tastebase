use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use serde::Deserialize;
use std::sync::OnceLock;
use tokio::sync::RwLock;

use crate::error::AppError;
use crate::types::UserContext;

#[derive(Debug, Deserialize)]
struct JwkSet {
    keys: Vec<Jwk>,
}

#[derive(Debug, Deserialize)]
struct Jwk {
    kid: String,
    n: String,
    e: String,
}

#[derive(Debug, Deserialize)]
struct Claims {
    sub: String,
    email: Option<String>,
}

struct JwksCache {
    keys: Vec<Jwk>,
}

static JWKS_CACHE: OnceLock<RwLock<Option<JwksCache>>> = OnceLock::new();

fn jwks_lock() -> &'static RwLock<Option<JwksCache>> {
    JWKS_CACHE.get_or_init(|| RwLock::new(None))
}

fn cognito_jwks_url() -> String {
    let pool_id = std::env::var("COGNITO_USER_POOL_ID").expect("COGNITO_USER_POOL_ID required");
    let region = std::env::var("AWS_REGION").unwrap_or_else(|_| "us-east-1".into());
    format!("https://cognito-idp.{region}.amazonaws.com/{pool_id}/.well-known/jwks.json")
}

fn cognito_issuer() -> String {
    let pool_id = std::env::var("COGNITO_USER_POOL_ID").expect("COGNITO_USER_POOL_ID required");
    let region = std::env::var("AWS_REGION").unwrap_or_else(|_| "us-east-1".into());
    format!("https://cognito-idp.{region}.amazonaws.com/{pool_id}")
}

async fn fetch_jwks() -> Result<Vec<Jwk>, AppError> {
    let url = cognito_jwks_url();
    let resp = reqwest::get(&url)
        .await
        .map_err(|e| AppError::Internal(format!("JWKS fetch failed: {e}")))?;
    let jwks: JwkSet = resp
        .json()
        .await
        .map_err(|e| AppError::Internal(format!("JWKS parse failed: {e}")))?;
    Ok(jwks.keys)
}

async fn get_decoding_key(kid: &str) -> Result<DecodingKey, AppError> {
    // Try cache first
    {
        let cache = jwks_lock().read().await;
        if let Some(ref c) = *cache
            && let Some(key) = c.keys.iter().find(|k| k.kid == kid) {
                return DecodingKey::from_rsa_components(&key.n, &key.e)
                    .map_err(|e| AppError::Internal(format!("bad RSA key: {e}")));
            }
    }
    // Refresh cache
    let keys = fetch_jwks().await?;
    let result = keys
        .iter()
        .find(|k| k.kid == kid)
        .ok_or_else(|| AppError::Unauthorized("unknown signing key".into()))
        .and_then(|k| {
            DecodingKey::from_rsa_components(&k.n, &k.e)
                .map_err(|e| AppError::Internal(format!("bad RSA key: {e}")))
        });
    *jwks_lock().write().await = Some(JwksCache { keys });
    result
}

/// Verify a Cognito JWT and return the user context.
pub async fn verify_token(token: &str) -> Result<UserContext, AppError> {
    let header =
        decode_header(token).map_err(|e| AppError::Unauthorized(format!("bad token header: {e}")))?;
    let kid = header
        .kid
        .ok_or_else(|| AppError::Unauthorized("token missing kid".into()))?;

    let key = get_decoding_key(&kid).await?;

    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_issuer(&[cognito_issuer()]);
    validation.validate_exp = true;

    let data = decode::<Claims>(token, &key, &validation)
        .map_err(|e| AppError::Unauthorized(format!("token validation failed: {e}")))?;

    Ok(UserContext {
        sub: data.claims.sub,
        email: data.claims.email,
        user_id: None,
    })
}

/// Extract a bearer token from the Authorization header.
pub fn extract_bearer(auth_header: Option<&str>) -> Result<&str, AppError> {
    let header = auth_header.ok_or_else(|| AppError::Unauthorized("missing Authorization header".into()))?;
    header
        .strip_prefix("Bearer ")
        .or_else(|| header.strip_prefix("bearer "))
        .ok_or_else(|| AppError::Unauthorized("missing Bearer token".into()))
}

/// Axum extractor: optional authentication. Returns None for unauthenticated requests.
pub struct OptionalAuth(pub Option<UserContext>);

impl<S: Send + Sync> FromRequestParts<S> for OptionalAuth {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let auth_header = parts.headers.get("authorization").and_then(|v| v.to_str().ok());
        match auth_header {
            None => Ok(OptionalAuth(None)),
            Some(header) => {
                let token = extract_bearer(Some(header))?;
                let ctx = verify_token(token).await?;
                Ok(OptionalAuth(Some(ctx)))
            }
        }
    }
}

/// Axum extractor: required authentication. Rejects unauthenticated requests.
pub struct RequireAuth(pub UserContext);

impl<S: Send + Sync> FromRequestParts<S> for RequireAuth {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get("authorization")
            .and_then(|v| v.to_str().ok());
        let token = extract_bearer(auth_header)?;
        let ctx = verify_token(token).await?;
        Ok(RequireAuth(ctx))
    }
}
