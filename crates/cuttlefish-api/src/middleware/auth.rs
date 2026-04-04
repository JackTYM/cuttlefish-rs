//! Authentication middleware for the API server.
//!
//! Supports two authentication methods:
//! - JWT tokens via `Authorization: Bearer <token>` header
//! - API keys via `X-API-Key: <key>` header (both legacy and user-created)

use std::sync::Arc;

use axum::{
    Json,
    body::Body,
    extract::{Request, State},
    http::{HeaderMap, StatusCode, header::AUTHORIZATION},
    middleware::Next,
    response::Response,
};
use cuttlefish_core::auth::{TokenClaims, TokenType, hash_api_key, validate_token};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

const X_API_KEY_HEADER: &str = "x-api-key";

/// Authenticated user information extracted from request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticatedUser {
    /// User ID (from JWT sub claim or API key owner).
    pub user_id: String,
    /// Authentication method used.
    pub auth_method: AuthMethod,
}

/// How the user was authenticated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuthMethod {
    /// Authenticated via JWT access token.
    Jwt,
    /// Authenticated via API key.
    ApiKey,
}

/// Configuration for the auth middleware.
#[derive(Clone)]
pub struct AuthConfig {
    /// JWT secret for token validation.
    pub jwt_secret: Arc<Vec<u8>>,
    /// Legacy API key (for backwards compatibility).
    pub legacy_api_key: Option<String>,
    /// Database pool for validating user-created API keys.
    pub db: Option<Arc<SqlitePool>>,
}

impl AuthConfig {
    /// Create a new auth configuration.
    pub fn new(jwt_secret: Vec<u8>) -> Self {
        Self {
            jwt_secret: Arc::new(jwt_secret),
            legacy_api_key: None,
            db: None,
        }
    }

    /// Set the legacy API key for backwards compatibility.
    pub fn with_legacy_api_key(mut self, key: String) -> Self {
        self.legacy_api_key = Some(key);
        self
    }

    /// Set the database pool for user-created API key validation.
    pub fn with_db(mut self, db: SqlitePool) -> Self {
        self.db = Some(Arc::new(db));
        self
    }
}

/// Extract authentication from request headers.
fn extract_auth(headers: &HeaderMap) -> Option<ExtractedAuth> {
    if let Some(auth_header) = headers.get(AUTHORIZATION)
        && let Ok(auth_str) = auth_header.to_str()
        && let Some(token) = auth_str.strip_prefix("Bearer ")
        && !token.is_empty()
    {
        return Some(ExtractedAuth::Bearer(token.to_string()));
    }

    if let Some(api_key_header) = headers.get(X_API_KEY_HEADER)
        && let Ok(key) = api_key_header.to_str()
        && !key.is_empty()
    {
        return Some(ExtractedAuth::ApiKey(key.to_string()));
    }

    None
}

enum ExtractedAuth {
    Bearer(String),
    ApiKey(String),
}

/// Authentication middleware that validates JWT tokens or API keys.
///
/// On success, adds `AuthenticatedUser` to request extensions.
/// On failure, returns 401 Unauthorized.
pub async fn require_auth(
    State(config): State<AuthConfig>,
    mut request: Request<Body>,
    next: Next,
) -> Result<Response, (StatusCode, Json<serde_json::Value>)> {
    let path = request.uri().path();

    if path == "/health"
        || path.starts_with("/api/auth/login")
        || path.starts_with("/api/auth/register")
        || path.starts_with("/api/auth/refresh")
        || path.starts_with("/api/auth/reset")
    {
        return Ok(next.run(request).await);
    }

    let auth = extract_auth(request.headers());

    let authenticated_user = match auth {
        Some(ExtractedAuth::Bearer(token)) => validate_jwt_token(&token, &config.jwt_secret)?,
        Some(ExtractedAuth::ApiKey(key)) => validate_api_key_async(&key, &config).await?,
        None => {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({ "error": "Missing authentication" })),
            ));
        }
    };

    request.extensions_mut().insert(authenticated_user);
    Ok(next.run(request).await)
}

/// Optional authentication middleware.
///
/// Attempts to authenticate but allows unauthenticated requests through.
/// If authentication is present and valid, adds `AuthenticatedUser` to extensions.
pub async fn optional_auth(
    State(config): State<AuthConfig>,
    mut request: Request<Body>,
    next: Next,
) -> Response {
    let auth = extract_auth(request.headers());

    if let Some(extracted) = auth {
        let authenticated_user = match extracted {
            ExtractedAuth::Bearer(token) => validate_jwt_token(&token, &config.jwt_secret).ok(),
            ExtractedAuth::ApiKey(key) => validate_api_key_async(&key, &config).await.ok(),
        };

        if let Some(user) = authenticated_user {
            request.extensions_mut().insert(user);
        }
    }

    next.run(request).await
}

fn validate_jwt_token(
    token: &str,
    secret: &[u8],
) -> Result<AuthenticatedUser, (StatusCode, Json<serde_json::Value>)> {
    let claims: TokenClaims = validate_token(token, secret).map_err(|e| {
        (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": format!("Invalid token: {}", e) })),
        )
    })?;

    if claims.token_type != TokenType::Access {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": "Invalid token type, expected access token" })),
        ));
    }

    Ok(AuthenticatedUser {
        user_id: claims.sub,
        auth_method: AuthMethod::Jwt,
    })
}

#[cfg(test)]
fn validate_api_key(
    key: &str,
    config: &AuthConfig,
) -> Result<AuthenticatedUser, (StatusCode, Json<serde_json::Value>)> {
    if let Some(ref legacy_key) = config.legacy_api_key
        && key == legacy_key
    {
        return Ok(AuthenticatedUser {
            user_id: "system".to_string(),
            auth_method: AuthMethod::ApiKey,
        });
    }

    Err((
        StatusCode::UNAUTHORIZED,
        Json(serde_json::json!({ "error": "Invalid API key" })),
    ))
}

async fn validate_api_key_async(
    key: &str,
    config: &AuthConfig,
) -> Result<AuthenticatedUser, (StatusCode, Json<serde_json::Value>)> {
    if let Some(ref legacy_key) = config.legacy_api_key
        && key == legacy_key
    {
        return Ok(AuthenticatedUser {
            user_id: "system".to_string(),
            auth_method: AuthMethod::ApiKey,
        });
    }

    if let Some(ref db) = config.db {
        let key_hash = hash_api_key(key);
        if let Ok(Some(api_key)) = cuttlefish_db::api_keys::get_api_key_by_hash(db, &key_hash).await
        {
            if let Err(e) = cuttlefish_db::api_keys::update_api_key_last_used(db, &api_key.id).await
            {
                tracing::warn!("Failed to update API key last_used_at: {}", e);
            }
            return Ok(AuthenticatedUser {
                user_id: api_key.user_id,
                auth_method: AuthMethod::ApiKey,
            });
        }
    }

    Err((
        StatusCode::UNAUTHORIZED,
        Json(serde_json::json!({ "error": "Invalid API key" })),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;
    use cuttlefish_core::auth::generate_access_token;

    const TEST_SECRET: &[u8] = b"test-secret-key-for-jwt-testing-32bytes!";

    #[test]
    fn test_extract_auth_bearer() {
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_static("Bearer my-token-123"),
        );

        let auth = extract_auth(&headers);
        assert!(matches!(auth, Some(ExtractedAuth::Bearer(t)) if t == "my-token-123"));
    }

    #[test]
    fn test_extract_auth_api_key() {
        let mut headers = HeaderMap::new();
        headers.insert(X_API_KEY_HEADER, HeaderValue::from_static("my-api-key"));

        let auth = extract_auth(&headers);
        assert!(matches!(auth, Some(ExtractedAuth::ApiKey(k)) if k == "my-api-key"));
    }

    #[test]
    fn test_extract_auth_bearer_takes_precedence() {
        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, HeaderValue::from_static("Bearer my-token"));
        headers.insert(X_API_KEY_HEADER, HeaderValue::from_static("my-api-key"));

        let auth = extract_auth(&headers);
        assert!(matches!(auth, Some(ExtractedAuth::Bearer(_))));
    }

    #[test]
    fn test_extract_auth_none() {
        let headers = HeaderMap::new();
        let auth = extract_auth(&headers);
        assert!(auth.is_none());
    }

    #[test]
    fn test_extract_auth_empty_bearer() {
        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, HeaderValue::from_static("Bearer "));

        let auth = extract_auth(&headers);
        assert!(auth.is_none());
    }

    #[test]
    fn test_validate_jwt_token_success() {
        let token = generate_access_token("user-123", TEST_SECRET).expect("generate");
        let result = validate_jwt_token(&token, TEST_SECRET);

        assert!(result.is_ok());
        let user = result.expect("user");
        assert_eq!(user.user_id, "user-123");
        assert_eq!(user.auth_method, AuthMethod::Jwt);
    }

    #[test]
    fn test_validate_jwt_token_invalid() {
        let result = validate_jwt_token("invalid-token", TEST_SECRET);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_api_key_legacy() {
        let config =
            AuthConfig::new(TEST_SECRET.to_vec()).with_legacy_api_key("legacy-key-123".to_string());

        let result = validate_api_key("legacy-key-123", &config);
        assert!(result.is_ok());
        let user = result.expect("user");
        assert_eq!(user.user_id, "system");
        assert_eq!(user.auth_method, AuthMethod::ApiKey);
    }

    #[test]
    fn test_validate_api_key_invalid() {
        let config = AuthConfig::new(TEST_SECRET.to_vec());
        let result = validate_api_key("invalid-key", &config);
        assert!(result.is_err());
    }

    #[test]
    fn test_auth_config_builder() {
        let config = AuthConfig::new(vec![1, 2, 3]).with_legacy_api_key("test-key".to_string());

        assert_eq!(*config.jwt_secret, vec![1, 2, 3]);
        assert_eq!(config.legacy_api_key, Some("test-key".to_string()));
    }
}
