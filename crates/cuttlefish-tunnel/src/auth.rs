//! Authentication for tunnel connections — link codes and JWTs.

use crate::error::TunnelError;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tracing::debug;

/// Link code configuration — 6 characters for easy typing
pub const LINK_CODE_LENGTH: usize = 6;
/// Link code expiry (10 minutes)
pub const LINK_CODE_EXPIRY: Duration = Duration::from_secs(10 * 60);
/// JWT expiry (7 days for reconnection support)
pub const JWT_EXPIRY: Duration = Duration::from_secs(7 * 24 * 60 * 60);

/// Characters allowed in link codes (uppercase alphanumeric, excluding confusing chars)
/// Excluded: 0/O (zero/oh), 1/I/L (one/eye/ell)
const LINK_CODE_CHARS: &[u8] = b"ABCDEFGHJKMNPQRSTUVWXYZ23456789";

/// JWT claims for tunnel authentication
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TunnelClaims {
    /// Subject (user ID)
    pub sub: String,
    /// Assigned subdomain
    pub subdomain: String,
    /// Expiration time (Unix timestamp)
    pub exp: i64,
    /// Issued at (Unix timestamp)
    pub iat: i64,
}

/// Alias for backwards compatibility
pub type JwtClaims = TunnelClaims;

impl TunnelClaims {
    /// Get the user ID (alias for sub field)
    #[must_use]
    pub fn user_id(&self) -> &str {
        &self.sub
    }
}

/// Result of validating a link code
#[derive(Debug, Clone)]
pub struct LinkCodeValidationResult {
    /// Generated JWT for future authentication
    pub jwt: String,
    /// Assigned subdomain
    pub subdomain: String,
    /// User ID
    pub user_id: String,
}

/// Validate a link code and generate auth credentials.
///
/// **Note**: This is a placeholder that always fails. In production,
/// this needs integration with a link code store (database or in-memory).
/// The actual validation flow:
/// 1. Look up the hashed link code in the store
/// 2. Verify the code hasn't expired
/// 3. Retrieve the associated user_id and subdomain
/// 4. Generate a JWT for the user
/// 5. Mark the link code as used (single-use)
///
/// # Errors
///
/// Returns `TunnelError::InvalidLinkCode` if the code is not found or expired
#[allow(unused_variables)]
pub fn validate_link_code(
    _code: &str,
    _jwt_secret: &[u8],
) -> Result<LinkCodeValidationResult, TunnelError> {
    Err(TunnelError::InvalidLinkCode(
        "Link code validation requires integration with link code store".to_string(),
    ))
}

/// Generate a random link code (6 uppercase alphanumeric characters).
///
/// Uses a character set that excludes easily confused characters:
/// - No 0/O (zero/oh)
/// - No 1/I/L (one/eye/ell)
///
/// # Example
///
/// ```
/// use cuttlefish_tunnel::auth::generate_link_code;
///
/// let code = generate_link_code();
/// assert_eq!(code.len(), 6);
/// assert!(code.chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit()));
/// ```
pub fn generate_link_code() -> String {
    let mut rng = rand::thread_rng();
    (0..LINK_CODE_LENGTH)
        .map(|_| {
            let idx = rng.gen_range(0..LINK_CODE_CHARS.len());
            LINK_CODE_CHARS[idx] as char
        })
        .collect()
}

/// Hash a link code for secure storage using SHA-256.
///
/// Link codes should never be stored in plaintext. This function
/// produces a deterministic hex-encoded hash suitable for database storage.
///
/// # Arguments
///
/// * `code` - The plaintext link code to hash
///
/// # Returns
///
/// A lowercase hex-encoded SHA-256 hash
pub fn hash_link_code(code: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(code.as_bytes());
    let result = hasher.finalize();
    hex::encode(result)
}

/// Verify a link code against its hash.
///
/// Uses constant-time comparison to prevent timing attacks.
///
/// # Arguments
///
/// * `code` - The plaintext link code to verify
/// * `hash` - The expected hash to compare against
///
/// # Returns
///
/// `true` if the code hashes to the expected value
pub fn verify_link_code(code: &str, hash: &str) -> bool {
    let computed_hash = hash_link_code(code);
    // Use constant-time comparison to prevent timing attacks
    constant_time_eq(computed_hash.as_bytes(), hash.as_bytes())
}

/// Constant-time comparison to prevent timing attacks.
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter()
        .zip(b.iter())
        .fold(0u8, |acc, (x, y)| acc | (x ^ y))
        == 0
}

/// Generate a JWT for tunnel authentication.
///
/// Creates a token with the user ID and subdomain that expires in 7 days.
/// The long expiry supports automatic reconnection without re-authentication.
///
/// # Arguments
///
/// * `user_id` - Unique identifier for the user/tunnel
/// * `subdomain` - Assigned subdomain for this tunnel
/// * `secret` - Secret key for signing (should be at least 32 bytes)
///
/// # Errors
///
/// Returns `TunnelError::Jwt` if token encoding fails
pub fn generate_jwt(user_id: &str, subdomain: &str, secret: &[u8]) -> Result<String, TunnelError> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time before Unix epoch")
        .as_secs() as i64;

    let claims = TunnelClaims {
        sub: user_id.to_string(),
        subdomain: subdomain.to_string(),
        exp: now + JWT_EXPIRY.as_secs() as i64,
        iat: now,
    };

    debug!(
        user_id = user_id,
        subdomain = subdomain,
        exp = claims.exp,
        "Generating JWT for tunnel authentication"
    );

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret),
    )?;

    Ok(token)
}

/// Validate a JWT and return the claims.
///
/// Verifies the token signature and checks that it has not expired.
///
/// # Arguments
///
/// * `token` - The JWT string to validate
/// * `secret` - Secret key used to sign the original token
///
/// # Errors
///
/// Returns `TunnelError::Jwt` if:
/// - Token signature is invalid
/// - Token has expired
/// - Token is malformed
pub fn validate_jwt(token: &str, secret: &[u8]) -> Result<TunnelClaims, TunnelError> {
    let validation = Validation::default();

    let token_data = decode::<TunnelClaims>(token, &DecodingKey::from_secret(secret), &validation)?;

    debug!(
        sub = %token_data.claims.sub,
        subdomain = %token_data.claims.subdomain,
        "JWT validated successfully"
    );

    Ok(token_data.claims)
}

/// Check if a JWT is expired.
///
/// # Arguments
///
/// * `claims` - The claims extracted from a JWT
///
/// # Returns
///
/// `true` if the current time is past the expiration time
pub fn is_jwt_expired(claims: &TunnelClaims) -> bool {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time before Unix epoch")
        .as_secs() as i64;
    claims.exp < now
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_link_code_format() {
        let code = generate_link_code();

        // Check length
        assert_eq!(code.len(), LINK_CODE_LENGTH);

        // Check all characters are uppercase alphanumeric
        assert!(code
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit()));

        // Check no confusing characters are present
        assert!(!code.contains('0'));
        assert!(!code.contains('O'));
        assert!(!code.contains('1'));
        assert!(!code.contains('I'));
        assert!(!code.contains('L'));
    }

    #[test]
    fn test_generate_link_code_unique() {
        // Generate multiple codes and ensure they're (probabilistically) unique
        let codes: Vec<String> = (0..100).map(|_| generate_link_code()).collect();

        // With 30^6 possible combinations, 100 codes should all be unique
        let unique_count = codes.iter().collect::<std::collections::HashSet<_>>().len();
        assert_eq!(unique_count, 100, "Expected all 100 codes to be unique");
    }

    #[test]
    fn test_hash_link_code_deterministic() {
        let code = "ABC123";
        let hash1 = hash_link_code(code);
        let hash2 = hash_link_code(code);

        assert_eq!(hash1, hash2, "Same input should produce same hash");

        // Verify it's a valid hex string of expected length (64 chars for SHA-256)
        assert_eq!(hash1.len(), 64);
        assert!(hash1.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_hash_link_code_different_inputs() {
        let hash1 = hash_link_code("ABC123");
        let hash2 = hash_link_code("ABC124");

        assert_ne!(
            hash1, hash2,
            "Different inputs should produce different hashes"
        );
    }

    #[test]
    fn test_verify_link_code_success() {
        let code = "XYZDEF";
        let hash = hash_link_code(code);

        assert!(verify_link_code(code, &hash));
    }

    #[test]
    fn test_verify_link_code_failure() {
        let code = "XYZDEF";
        let hash = hash_link_code(code);

        assert!(!verify_link_code("WRONGC", &hash));
    }

    #[test]
    fn test_verify_link_code_case_sensitive() {
        let code = "ABCDEF";
        let hash = hash_link_code(code);

        // Should be case-sensitive
        assert!(!verify_link_code("abcdef", &hash));
    }

    #[test]
    fn test_generate_jwt_success() {
        let secret = b"test-secret-key-at-least-32-bytes!";
        let user_id = "user123";
        let subdomain = "my-tunnel";

        let token =
            generate_jwt(user_id, subdomain, secret).expect("JWT generation should succeed");

        // JWT should have 3 parts separated by dots
        let parts: Vec<&str> = token.split('.').collect();
        assert_eq!(
            parts.len(),
            3,
            "JWT should have header.payload.signature format"
        );

        // Validate the token and check claims
        let claims = validate_jwt(&token, secret).expect("JWT validation should succeed");
        assert_eq!(claims.sub, user_id);
        assert_eq!(claims.subdomain, subdomain);
    }

    #[test]
    fn test_validate_jwt_success() {
        let secret = b"another-secret-key-32-bytes-long";
        let user_id = "tunnel-user";
        let subdomain = "dev-tunnel";

        let token =
            generate_jwt(user_id, subdomain, secret).expect("JWT generation should succeed");

        let claims = validate_jwt(&token, secret).expect("Validation should succeed");

        assert_eq!(claims.sub, user_id);
        assert_eq!(claims.subdomain, subdomain);
        assert!(!is_jwt_expired(&claims), "Fresh JWT should not be expired");
    }

    #[test]
    fn test_validate_jwt_wrong_secret() {
        let secret1 = b"first-secret-key-32-bytes-long!!";
        let secret2 = b"wrong-secret-key-32-bytes-long!!";

        let token =
            generate_jwt("user", "subdomain", secret1).expect("JWT generation should succeed");

        let result = validate_jwt(&token, secret2);
        assert!(result.is_err(), "Validation with wrong secret should fail");
    }

    #[test]
    fn test_validate_jwt_expired() {
        // Create claims that are already expired
        let expired_claims = TunnelClaims {
            sub: "user".to_string(),
            subdomain: "sub".to_string(),
            exp: 1, // Expired in 1970
            iat: 0,
        };

        let secret = b"test-secret-for-expired-jwt-test";

        // Manually encode an expired token
        let token = encode(
            &Header::default(),
            &expired_claims,
            &EncodingKey::from_secret(secret),
        )
        .expect("Encoding should succeed");

        let result = validate_jwt(&token, secret);
        assert!(result.is_err(), "Expired JWT should fail validation");
    }

    #[test]
    fn test_is_jwt_expired() {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should work")
            .as_secs() as i64;

        // Not expired (expires in 1 hour)
        let future_claims = TunnelClaims {
            sub: "user".to_string(),
            subdomain: "sub".to_string(),
            exp: now + 3600,
            iat: now,
        };
        assert!(!is_jwt_expired(&future_claims));

        // Already expired (expired 1 hour ago)
        let past_claims = TunnelClaims {
            sub: "user".to_string(),
            subdomain: "sub".to_string(),
            exp: now - 3600,
            iat: now - 7200,
        };
        assert!(is_jwt_expired(&past_claims));
    }

    #[test]
    fn test_jwt_claims_expiry_duration() {
        let secret = b"test-secret-for-expiry-duration!";
        let token =
            generate_jwt("user", "subdomain", secret).expect("JWT generation should succeed");

        let claims = validate_jwt(&token, secret).expect("Validation should succeed");

        // Check that expiry is approximately 7 days from now
        let expected_expiry = JWT_EXPIRY.as_secs() as i64;
        let actual_expiry = claims.exp - claims.iat;

        assert_eq!(
            actual_expiry, expected_expiry,
            "JWT should expire in 7 days"
        );
    }

    #[test]
    fn test_constant_time_eq() {
        assert!(constant_time_eq(b"hello", b"hello"));
        assert!(!constant_time_eq(b"hello", b"world"));
        assert!(!constant_time_eq(b"hello", b"hell"));
        assert!(!constant_time_eq(b"", b"a"));
        assert!(constant_time_eq(b"", b""));
    }
}
