//! Password reset token generation and validation.

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use rand::RngCore;
use sha2::{Digest, Sha256};

const RESET_TOKEN_BYTES: usize = 32;

/// A generated password reset token.
#[derive(Debug, Clone)]
pub struct GeneratedResetToken {
    /// The plaintext token (URL-safe base64, send to user via email).
    pub plaintext: String,
    /// SHA-256 hash of the token (store in database).
    pub hash: String,
}

/// Generate a new password reset token.
pub fn generate_reset_token() -> GeneratedResetToken {
    let mut bytes = [0u8; RESET_TOKEN_BYTES];
    rand::thread_rng().fill_bytes(&mut bytes);

    let plaintext = URL_SAFE_NO_PAD.encode(bytes);
    let hash = hash_reset_token(&plaintext);

    GeneratedResetToken { plaintext, hash }
}

/// Hash a reset token using SHA-256.
pub fn hash_reset_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    hex::encode(hasher.finalize())
}

/// Verify a reset token against its hash.
pub fn verify_reset_token(token: &str, hash: &str) -> bool {
    let computed_hash = hash_reset_token(token);
    constant_time_compare(&computed_hash, hash)
}

fn constant_time_compare(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }

    let mut result = 0u8;
    for (x, y) in a.bytes().zip(b.bytes()) {
        result |= x ^ y;
    }
    result == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_reset_token() {
        let token = generate_reset_token();

        assert!(!token.plaintext.is_empty());
        assert!(!token.hash.is_empty());
        assert!(
            token
                .plaintext
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        );
    }

    #[test]
    fn test_generate_reset_token_unique() {
        let token1 = generate_reset_token();
        let token2 = generate_reset_token();

        assert_ne!(token1.plaintext, token2.plaintext);
        assert_ne!(token1.hash, token2.hash);
    }

    #[test]
    fn test_hash_reset_token_deterministic() {
        let token = "test-token-abc123";
        let hash1 = hash_reset_token(token);
        let hash2 = hash_reset_token(token);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_hash_reset_token_different_tokens() {
        let hash1 = hash_reset_token("token1");
        let hash2 = hash_reset_token("token2");
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_verify_reset_token_valid() {
        let token = generate_reset_token();
        assert!(verify_reset_token(&token.plaintext, &token.hash));
    }

    #[test]
    fn test_verify_reset_token_invalid() {
        let token = generate_reset_token();
        assert!(!verify_reset_token("wrong-token", &token.hash));
    }

    #[test]
    fn test_verify_reset_token_wrong_hash() {
        let token = generate_reset_token();
        let wrong_hash = hash_reset_token("different-token");
        assert!(!verify_reset_token(&token.plaintext, &wrong_hash));
    }

    #[test]
    fn test_constant_time_compare_equal() {
        assert!(constant_time_compare("abc123", "abc123"));
    }

    #[test]
    fn test_constant_time_compare_different() {
        assert!(!constant_time_compare("abc123", "xyz789"));
    }

    #[test]
    fn test_constant_time_compare_different_length() {
        assert!(!constant_time_compare("short", "longer"));
    }

    #[test]
    fn test_token_is_url_safe() {
        for _ in 0..10 {
            let token = generate_reset_token();
            assert!(
                token
                    .plaintext
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'),
                "Token contains non-URL-safe characters: {}",
                token.plaintext
            );
        }
    }
}
