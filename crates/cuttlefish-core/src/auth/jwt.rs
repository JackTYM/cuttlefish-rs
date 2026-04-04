//! JWT token generation and validation.

use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};

use super::AuthError;

/// Token type (access or refresh).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TokenType {
    /// Short-lived access token (24 hours).
    Access,
    /// Long-lived refresh token (30 days).
    Refresh,
}

impl std::fmt::Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenType::Access => write!(f, "access"),
            TokenType::Refresh => write!(f, "refresh"),
        }
    }
}

/// JWT token claims.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenClaims {
    /// Subject (user ID).
    pub sub: String,
    /// Expiration timestamp (Unix epoch seconds).
    pub exp: i64,
    /// Issued at timestamp (Unix epoch seconds).
    pub iat: i64,
    /// Token type.
    pub token_type: TokenType,
}

/// A pair of access and refresh tokens.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenPair {
    /// The access token (short-lived).
    pub access_token: String,
    /// The refresh token (long-lived).
    pub refresh_token: String,
    /// Access token expiration in seconds.
    pub expires_in: i64,
}

const ACCESS_TOKEN_DURATION_HOURS: i64 = 24;
const REFRESH_TOKEN_DURATION_DAYS: i64 = 30;

/// Generate an access token (24 hour expiry).
pub fn generate_access_token(user_id: &str, secret: &[u8]) -> Result<String, AuthError> {
    generate_token(user_id, secret, TokenType::Access)
}

/// Generate a refresh token (30 day expiry).
pub fn generate_refresh_token(user_id: &str, secret: &[u8]) -> Result<String, AuthError> {
    generate_token(user_id, secret, TokenType::Refresh)
}

fn generate_token(
    user_id: &str,
    secret: &[u8],
    token_type: TokenType,
) -> Result<String, AuthError> {
    let now = Utc::now();
    let exp = match token_type {
        TokenType::Access => now + Duration::hours(ACCESS_TOKEN_DURATION_HOURS),
        TokenType::Refresh => now + Duration::days(REFRESH_TOKEN_DURATION_DAYS),
    };

    let claims = TokenClaims {
        sub: user_id.to_string(),
        exp: exp.timestamp(),
        iat: now.timestamp(),
        token_type,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret),
    )
    .map_err(|e| AuthError::TokenGenerationFailed(e.to_string()))
}

/// Validate a token and return its claims.
pub fn validate_token(token: &str, secret: &[u8]) -> Result<TokenClaims, AuthError> {
    let token_data = decode::<TokenClaims>(
        token,
        &DecodingKey::from_secret(secret),
        &Validation::default(),
    )
    .map_err(|e| match e.kind() {
        jsonwebtoken::errors::ErrorKind::ExpiredSignature => AuthError::TokenExpired,
        _ => AuthError::TokenValidationFailed(e.to_string()),
    })?;

    Ok(token_data.claims)
}

/// Generate a token pair (access + refresh) for a user.
pub fn generate_token_pair(user_id: &str, secret: &[u8]) -> Result<TokenPair, AuthError> {
    let access_token = generate_access_token(user_id, secret)?;
    let refresh_token = generate_refresh_token(user_id, secret)?;

    Ok(TokenPair {
        access_token,
        refresh_token,
        expires_in: ACCESS_TOKEN_DURATION_HOURS * 3600,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_SECRET: &[u8] = b"test-secret-key-for-jwt-testing-32bytes!";

    #[test]
    fn test_generate_access_token() {
        let token = generate_access_token("user-123", TEST_SECRET).expect("token generation");
        assert!(!token.is_empty());

        let claims = validate_token(&token, TEST_SECRET).expect("validation");
        assert_eq!(claims.sub, "user-123");
        assert_eq!(claims.token_type, TokenType::Access);
    }

    #[test]
    fn test_generate_refresh_token() {
        let token = generate_refresh_token("user-456", TEST_SECRET).expect("token generation");
        assert!(!token.is_empty());

        let claims = validate_token(&token, TEST_SECRET).expect("validation");
        assert_eq!(claims.sub, "user-456");
        assert_eq!(claims.token_type, TokenType::Refresh);
    }

    #[test]
    fn test_token_pair_generation() {
        let pair = generate_token_pair("user-789", TEST_SECRET).expect("pair generation");

        assert!(!pair.access_token.is_empty());
        assert!(!pair.refresh_token.is_empty());
        assert_ne!(pair.access_token, pair.refresh_token);
        assert_eq!(pair.expires_in, 24 * 3600);
    }

    #[test]
    fn test_invalid_token() {
        let result = validate_token("invalid.token.here", TEST_SECRET);
        assert!(result.is_err());
    }

    #[test]
    fn test_wrong_secret() {
        let token = generate_access_token("user-123", TEST_SECRET).expect("token generation");
        let result = validate_token(&token, b"wrong-secret");
        assert!(result.is_err());
    }

    #[test]
    fn test_token_type_display() {
        assert_eq!(format!("{}", TokenType::Access), "access");
        assert_eq!(format!("{}", TokenType::Refresh), "refresh");
    }

    #[test]
    fn test_token_claims_expiration() {
        let token = generate_access_token("user-123", TEST_SECRET).expect("token generation");
        let claims = validate_token(&token, TEST_SECRET).expect("validation");

        let now = Utc::now().timestamp();
        let expected_exp = now + (ACCESS_TOKEN_DURATION_HOURS * 3600);

        // Allow 5 second tolerance for test execution time
        assert!((claims.exp - expected_exp).abs() < 5);
    }
}
