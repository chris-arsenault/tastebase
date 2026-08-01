use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use base64::Engine;
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode, decode_header};
use serde::Deserialize;
use std::sync::OnceLock;
use tokio::sync::RwLock;

use crate::error::AppError;
use crate::types::UserContext;

#[derive(Debug, Deserialize)]
struct Claims {
    sub: String,
    email: Option<String>,
    client_id: Option<String>,
    aud: Option<String>,
}

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

static JWKS_CACHE: OnceLock<RwLock<Vec<Jwk>>> = OnceLock::new();

fn jwks_cache() -> &'static RwLock<Vec<Jwk>> {
    JWKS_CACHE.get_or_init(|| RwLock::new(Vec::new()))
}

fn cognito_issuer() -> String {
    std::env::var("COGNITO_ISSUER").expect("COGNITO_ISSUER required")
}

fn cognito_jwks_url() -> String {
    format!("{}/.well-known/jwks.json", cognito_issuer())
}

async fn fetch_jwks() -> Result<Vec<Jwk>, AppError> {
    reqwest::get(cognito_jwks_url())
        .await
        .map_err(|error| AppError::Internal(format!("JWKS fetch failed: {error}")))?
        .error_for_status()
        .map_err(|error| AppError::Internal(format!("JWKS fetch failed: {error}")))?
        .json::<JwkSet>()
        .await
        .map(|set| set.keys)
        .map_err(|error| AppError::Internal(format!("JWKS parse failed: {error}")))
}

async fn get_decoding_key(kid: &str) -> Result<DecodingKey, AppError> {
    {
        let cache = jwks_cache().read().await;
        if let Some(key) = cache.iter().find(|key| key.kid == kid) {
            return DecodingKey::from_rsa_components(&key.n, &key.e)
                .map_err(|error| AppError::Internal(format!("invalid RSA key: {error}")));
        }
    }

    let keys = fetch_jwks().await?;
    let decoding_key = keys
        .iter()
        .find(|key| key.kid == kid)
        .ok_or_else(|| AppError::Unauthorized("unknown signing key".into()))
        .and_then(|key| {
            DecodingKey::from_rsa_components(&key.n, &key.e)
                .map_err(|error| AppError::Internal(format!("invalid RSA key: {error}")))
        })?;
    *jwks_cache().write().await = keys;
    Ok(decoding_key)
}

fn token_matches_client(claims: &Claims, expected_client_id: &str) -> bool {
    claims.client_id.as_deref() == Some(expected_client_id)
        || claims.aud.as_deref() == Some(expected_client_id)
}

/// Verify a Cognito JWT for a route that is not protected by ALB jwt-validation.
pub async fn verify_token(token: &str) -> Result<UserContext, AppError> {
    let header = decode_header(token)
        .map_err(|error| AppError::Unauthorized(format!("bad token header: {error}")))?;
    let kid = header
        .kid
        .ok_or_else(|| AppError::Unauthorized("token missing kid".into()))?;
    let key = get_decoding_key(&kid).await?;

    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_issuer(&[cognito_issuer()]);
    validation.validate_aud = false;
    let claims = decode::<Claims>(token, &key, &validation)
        .map_err(|error| AppError::Unauthorized(format!("token validation failed: {error}")))?
        .claims;

    let expected_client_id =
        std::env::var("COGNITO_CLIENT_ID").expect("COGNITO_CLIENT_ID required");
    if !token_matches_client(&claims, &expected_client_id) {
        return Err(AppError::Unauthorized("token client mismatch".into()));
    }

    Ok(UserContext {
        sub: claims.sub,
        email: claims.email,
        user_id: None,
    })
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

#[cfg(test)]
mod tests {
    use super::{Claims, token_matches_client};

    fn claims(client_id: Option<&str>, audience: Option<&str>) -> Claims {
        Claims {
            sub: "reader".into(),
            email: None,
            client_id: client_id.map(String::from),
            aud: audience.map(String::from),
        }
    }

    #[test]
    fn access_tokens_must_match_the_mcp_client() {
        assert!(token_matches_client(
            &claims(Some("mcp-client"), None),
            "mcp-client"
        ));
        assert!(!token_matches_client(
            &claims(Some("frontend-client"), None),
            "mcp-client"
        ));
    }

    #[test]
    fn id_tokens_may_match_by_audience() {
        assert!(token_matches_client(
            &claims(None, Some("mcp-client")),
            "mcp-client"
        ));
    }
}
