//! API key generation and validation.

use sha2::{Digest, Sha256};

use super::AuthError;

const API_KEY_PREFIX: &str = "cfish_";
const API_KEY_RANDOM_LENGTH: usize = 32;

/// Scopes that can be assigned to an API key.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApiKeyScope {
    /// Read-only access.
    Read,
    /// Read and write access.
    Write,
    /// Full administrative access.
    Admin,
}

impl ApiKeyScope {
    /// Convert scope to string representation.
    pub fn as_str(&self) -> &'static str {
        match self {
            ApiKeyScope::Read => "read",
            ApiKeyScope::Write => "write",
            ApiKeyScope::Admin => "admin",
        }
    }

    /// Parse scope from string.
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "read" => Some(ApiKeyScope::Read),
            "write" => Some(ApiKeyScope::Write),
            "admin" => Some(ApiKeyScope::Admin),
            _ => None,
        }
    }

    /// Check if this scope includes another scope.
    pub fn includes(&self, other: ApiKeyScope) -> bool {
        match self {
            ApiKeyScope::Admin => true,
            ApiKeyScope::Write => matches!(other, ApiKeyScope::Read | ApiKeyScope::Write),
            ApiKeyScope::Read => matches!(other, ApiKeyScope::Read),
        }
    }
}

impl std::fmt::Display for ApiKeyScope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// A generated API key with its plaintext value (shown once) and hash.
#[derive(Debug, Clone)]
pub struct GeneratedApiKey {
    /// The full plaintext key (show to user ONCE, then discard).
    pub plaintext: String,
    /// SHA-256 hash of the key (store in database).
    pub hash: String,
    /// First 8 characters for identification.
    pub prefix: String,
}

/// Generate a new API key.
pub fn generate_api_key() -> GeneratedApiKey {
    use rand::Rng;

    let random_part: String = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(API_KEY_RANDOM_LENGTH)
        .map(char::from)
        .collect();

    let plaintext = format!("{}{}", API_KEY_PREFIX, random_part);
    let hash = hash_api_key(&plaintext);
    let prefix = plaintext.chars().take(API_KEY_PREFIX.len() + 3).collect();

    GeneratedApiKey {
        plaintext,
        hash,
        prefix,
    }
}

/// Hash an API key using SHA-256.
pub fn hash_api_key(key: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(key.as_bytes());
    hex::encode(hasher.finalize())
}

/// Validate an API key format.
pub fn validate_api_key_format(key: &str) -> Result<(), AuthError> {
    if !key.starts_with(API_KEY_PREFIX) {
        return Err(AuthError::InvalidApiKey(
            "Key must start with 'cfish_'".to_string(),
        ));
    }

    let random_part = &key[API_KEY_PREFIX.len()..];
    if random_part.len() != API_KEY_RANDOM_LENGTH {
        return Err(AuthError::InvalidApiKey(format!(
            "Key must have {} random characters after prefix",
            API_KEY_RANDOM_LENGTH
        )));
    }

    if !random_part.chars().all(|c| c.is_ascii_alphanumeric()) {
        return Err(AuthError::InvalidApiKey(
            "Key must contain only alphanumeric characters".to_string(),
        ));
    }

    Ok(())
}

/// Check if a list of scopes includes a required scope.
pub fn has_required_scope(scopes: &[String], required: ApiKeyScope) -> bool {
    scopes.iter().any(|s| {
        ApiKeyScope::parse(s)
            .map(|scope| scope.includes(required))
            .unwrap_or(false)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_api_key() {
        let key = generate_api_key();

        assert!(key.plaintext.starts_with(API_KEY_PREFIX));
        assert_eq!(
            key.plaintext.len(),
            API_KEY_PREFIX.len() + API_KEY_RANDOM_LENGTH
        );
        assert!(!key.hash.is_empty());
        assert!(key.prefix.starts_with(API_KEY_PREFIX));
    }

    #[test]
    fn test_hash_api_key_deterministic() {
        let key = "cfish_abc123";
        let hash1 = hash_api_key(key);
        let hash2 = hash_api_key(key);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_hash_api_key_different_keys() {
        let hash1 = hash_api_key("cfish_abc123");
        let hash2 = hash_api_key("cfish_xyz789");
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_validate_api_key_format_valid() {
        let key = generate_api_key();
        assert!(validate_api_key_format(&key.plaintext).is_ok());
    }

    #[test]
    fn test_validate_api_key_format_wrong_prefix() {
        let result = validate_api_key_format("wrong_abc123abc123abc123abc123abc123ab");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_api_key_format_too_short() {
        let result = validate_api_key_format("cfish_short");
        assert!(result.is_err());
    }

    #[test]
    fn test_api_key_scope_includes() {
        assert!(ApiKeyScope::Admin.includes(ApiKeyScope::Read));
        assert!(ApiKeyScope::Admin.includes(ApiKeyScope::Write));
        assert!(ApiKeyScope::Admin.includes(ApiKeyScope::Admin));

        assert!(ApiKeyScope::Write.includes(ApiKeyScope::Read));
        assert!(ApiKeyScope::Write.includes(ApiKeyScope::Write));
        assert!(!ApiKeyScope::Write.includes(ApiKeyScope::Admin));

        assert!(ApiKeyScope::Read.includes(ApiKeyScope::Read));
        assert!(!ApiKeyScope::Read.includes(ApiKeyScope::Write));
        assert!(!ApiKeyScope::Read.includes(ApiKeyScope::Admin));
    }

    #[test]
    fn test_api_key_scope_from_str() {
        assert_eq!(ApiKeyScope::parse("read"), Some(ApiKeyScope::Read));
        assert_eq!(ApiKeyScope::parse("write"), Some(ApiKeyScope::Write));
        assert_eq!(ApiKeyScope::parse("admin"), Some(ApiKeyScope::Admin));
        assert_eq!(ApiKeyScope::parse("invalid"), None);
    }

    #[test]
    fn test_has_required_scope() {
        let scopes = vec!["read".to_string(), "write".to_string()];

        assert!(has_required_scope(&scopes, ApiKeyScope::Read));
        assert!(has_required_scope(&scopes, ApiKeyScope::Write));
        assert!(!has_required_scope(&scopes, ApiKeyScope::Admin));
    }

    #[test]
    fn test_has_required_scope_admin_includes_all() {
        let scopes = vec!["admin".to_string()];

        assert!(has_required_scope(&scopes, ApiKeyScope::Read));
        assert!(has_required_scope(&scopes, ApiKeyScope::Write));
        assert!(has_required_scope(&scopes, ApiKeyScope::Admin));
    }

    #[test]
    fn test_api_key_scope_display() {
        assert_eq!(format!("{}", ApiKeyScope::Read), "read");
        assert_eq!(format!("{}", ApiKeyScope::Write), "write");
        assert_eq!(format!("{}", ApiKeyScope::Admin), "admin");
    }
}
