//! Session management and refresh token rotation.

use serde::{Deserialize, Serialize};

use super::AuthError;

/// Metadata about a session (user agent, IP address).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SessionMetadata {
    /// Client user agent string.
    pub user_agent: Option<String>,
    /// Client IP address.
    pub ip_address: Option<String>,
}

impl SessionMetadata {
    /// Create new session metadata.
    pub fn new(user_agent: Option<String>, ip_address: Option<String>) -> Self {
        Self {
            user_agent,
            ip_address,
        }
    }
}

/// Session information returned to clients (without sensitive data).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    /// Session ID.
    pub id: String,
    /// When the session was created.
    pub created_at: String,
    /// Client user agent.
    pub user_agent: Option<String>,
    /// Client IP address (may be masked).
    pub ip_address: Option<String>,
    /// Whether this is the current session.
    pub is_current: bool,
}

/// Result of a token refresh operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshResult {
    /// New access token.
    pub access_token: String,
    /// New refresh token.
    pub refresh_token: String,
    /// Access token expiration in seconds.
    pub expires_in: i64,
    /// Session ID.
    pub session_id: String,
}

/// Hash a refresh token for storage using SHA-256.
///
/// We use SHA-256 instead of Argon2 for refresh tokens because:
/// 1. Refresh tokens are high-entropy random values (not user passwords)
/// 2. SHA-256 is fast enough for token validation on every request
/// 3. The security model relies on token secrecy, not hash strength
pub fn hash_refresh_token(token: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    let result = hasher.finalize();
    hex::encode(result)
}

/// Verify a refresh token against a stored hash.
pub fn verify_refresh_token(token: &str, hash: &str) -> bool {
    hash_refresh_token(token) == hash
}

/// Generate a secure random refresh token.
pub fn generate_refresh_token_value() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let bytes: [u8; 32] = rng.r#gen();
    hex::encode(bytes)
}

/// Errors specific to session operations.
#[derive(Debug, Clone)]
pub enum SessionError {
    /// Session not found.
    NotFound,
    /// Session has been revoked.
    Revoked,
    /// Session has expired.
    Expired,
    /// Refresh token reuse detected (security breach).
    TokenReuse,
    /// Maximum sessions reached.
    MaxSessionsReached,
}

impl std::fmt::Display for SessionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SessionError::NotFound => write!(f, "Session not found"),
            SessionError::Revoked => write!(f, "Session has been revoked"),
            SessionError::Expired => write!(f, "Session has expired"),
            SessionError::TokenReuse => write!(f, "Refresh token reuse detected"),
            SessionError::MaxSessionsReached => write!(f, "Maximum sessions reached"),
        }
    }
}

impl std::error::Error for SessionError {}

impl From<SessionError> for AuthError {
    fn from(err: SessionError) -> Self {
        match err {
            SessionError::NotFound => {
                AuthError::TokenValidationFailed("Session not found".to_string())
            }
            SessionError::Revoked => {
                AuthError::TokenValidationFailed("Session revoked".to_string())
            }
            SessionError::Expired => AuthError::TokenExpired,
            SessionError::TokenReuse => AuthError::TokenValidationFailed(
                "Token reuse detected - all sessions revoked".to_string(),
            ),
            SessionError::MaxSessionsReached => {
                AuthError::TokenValidationFailed("Maximum sessions reached".to_string())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_refresh_token() {
        let token = "test-refresh-token-123";
        let hash = hash_refresh_token(token);

        assert_eq!(hash.len(), 64);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_verify_refresh_token() {
        let token = "my-secret-token";
        let hash = hash_refresh_token(token);

        assert!(verify_refresh_token(token, &hash));
        assert!(!verify_refresh_token("wrong-token", &hash));
    }

    #[test]
    fn test_generate_refresh_token_value() {
        let token1 = generate_refresh_token_value();
        let token2 = generate_refresh_token_value();

        assert_eq!(token1.len(), 64);
        assert_ne!(token1, token2);
    }

    #[test]
    fn test_session_metadata() {
        let meta = SessionMetadata::new(
            Some("Mozilla/5.0".to_string()),
            Some("192.168.1.1".to_string()),
        );

        assert_eq!(meta.user_agent, Some("Mozilla/5.0".to_string()));
        assert_eq!(meta.ip_address, Some("192.168.1.1".to_string()));
    }

    #[test]
    fn test_session_error_display() {
        assert_eq!(format!("{}", SessionError::NotFound), "Session not found");
        assert_eq!(
            format!("{}", SessionError::TokenReuse),
            "Refresh token reuse detected"
        );
    }
}
