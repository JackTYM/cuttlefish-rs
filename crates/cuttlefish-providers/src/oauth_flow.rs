//! Claude Code OAuth PKCE flow utilities.
//!
//! Implements the OAuth 2.0 Authorization Code flow with PKCE as used by
//! the Claude Code CLI client.

use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use cuttlefish_core::error::ProviderError;
use sha2::{Digest, Sha256};

/// The client ID used by Claude Code CLI.
pub const CLAUDE_CODE_CLIENT_ID: &str = "9d1c250a-e61b-44d9-88ed-5944d1962f5e";

/// OAuth authorization endpoint.
pub const AUTH_ENDPOINT: &str = "https://platform.claude.com/oauth/authorize";

/// Token exchange endpoint.
pub const TOKEN_ENDPOINT: &str = "https://platform.claude.com/v1/oauth/token";

/// Required OAuth scopes.
pub const OAUTH_SCOPES: &str = "user:inference user:profile user:sessions:claude_code user:mcp_servers user:file_upload org:create_api_key";

/// Required Anthropic beta header value.
pub const ANTHROPIC_BETA: &str =
    "claude-code-20250219,oauth-2025-04-20,interleaved-thinking-2025-05-14";

/// Claude Code CLI user agent.
pub const USER_AGENT: &str = "claude-cli/2.1.87 (external, cli)";

/// An OAuth token pair with refresh capability.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OAuthTokens {
    /// The access token for API calls.
    pub access_token: String,
    /// The refresh token for obtaining new access tokens.
    pub refresh_token: Option<String>,
    /// Unix timestamp when the access token expires.
    pub expires_at: Option<u64>,
}

/// PKCE code verifier (random string).
#[derive(Debug, Clone)]
pub struct PkceVerifier(
    /// The raw verifier string.
    pub String,
);

/// PKCE code challenge (SHA256 of verifier, base64url encoded).
#[derive(Debug, Clone)]
pub struct PkceChallenge(
    /// The base64url-encoded challenge.
    pub String,
);

/// Generate a PKCE code verifier (32 random bytes, base64url encoded).
pub fn generate_pkce_verifier() -> PkceVerifier {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    use std::time::SystemTime;

    let mut bytes = [0u8; 32];
    let seed = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);

    for (i, byte) in bytes.iter_mut().enumerate() {
        let mut hasher = DefaultHasher::new();
        (seed + i as u128).hash(&mut hasher);
        std::process::id().hash(&mut hasher);
        *byte = (hasher.finish() & 0xFF) as u8;
    }

    PkceVerifier(URL_SAFE_NO_PAD.encode(bytes))
}

/// Derive the PKCE code challenge from a verifier.
///
/// `challenge = base64url(SHA256(verifier))`
pub fn derive_pkce_challenge(verifier: &PkceVerifier) -> PkceChallenge {
    let mut hasher = Sha256::new();
    hasher.update(verifier.0.as_bytes());
    let hash = hasher.finalize();
    PkceChallenge(URL_SAFE_NO_PAD.encode(hash))
}

/// Build the authorization URL for the OAuth flow.
pub fn build_auth_url(
    verifier: &PkceVerifier,
    state: &str,
    redirect_port: u16,
) -> Result<String, ProviderError> {
    let challenge = derive_pkce_challenge(verifier);
    let redirect_uri = format!("http://localhost:{}/callback", redirect_port);

    Ok(format!(
        "{}?code=true&client_id={}&response_type=code&redirect_uri={}&scope={}&code_challenge={}&code_challenge_method=S256&state={}",
        AUTH_ENDPOINT,
        CLAUDE_CODE_CLIENT_ID,
        urlencoded(&redirect_uri),
        urlencoded(OAUTH_SCOPES),
        challenge.0,
        state
    ))
}

fn urlencoded(s: &str) -> String {
    let mut encoded = String::new();
    for byte in s.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(byte as char);
            }
            b' ' => encoded.push('+'),
            other => {
                encoded.push('%');
                encoded.push_str(&format!("{:02X}", other));
            }
        }
    }
    encoded
}

/// Compute the Claude Code Hash (CCH) for request body signing.
///
/// Uses xxHash64 with seed `0x6e52736ac806831e`.
/// Takes the lower 20 bits and formats as 5-character lowercase hex.
pub fn compute_cch(body: &str) -> String {
    use xxhash_rust::xxh64::xxh64;
    const CCH_SEED: u64 = 0x6e52736ac806831e;
    let hash = xxh64(body.as_bytes(), CCH_SEED);
    let lower_20_bits = hash & 0xFFFFF;
    format!("{:05x}", lower_20_bits)
}

/// Replace the `cch=00000` placeholder in a request body with the computed hash.
pub fn sign_request_body(body: &str) -> String {
    let cch = compute_cch(body);
    body.replace("cch=00000", &format!("cch={}", cch))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pkce_challenge_derives_from_verifier() {
        let verifier = PkceVerifier("test_verifier_value".to_string());
        let challenge = derive_pkce_challenge(&verifier);
        assert!(!challenge.0.is_empty());
        assert!(
            challenge
                .0
                .chars()
                .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        );
    }

    #[test]
    fn test_pkce_challenge_is_deterministic() {
        let verifier = PkceVerifier("same_verifier".to_string());
        let c1 = derive_pkce_challenge(&verifier);
        let c2 = derive_pkce_challenge(&verifier);
        assert_eq!(c1.0, c2.0);
    }

    #[test]
    fn test_pkce_verifier_generates_non_empty() {
        let v = generate_pkce_verifier();
        assert!(!v.0.is_empty());
        assert!(v.0.len() >= 40);
    }

    #[test]
    fn test_cch_compute_returns_5_char_hex() {
        let hash = compute_cch("test body");
        assert_eq!(hash.len(), 5);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_cch_is_deterministic() {
        let h1 = compute_cch("same body");
        let h2 = compute_cch("same body");
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_cch_differs_for_different_bodies() {
        let h1 = compute_cch("body one");
        let h2 = compute_cch("body two different");
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_sign_request_body_replaces_placeholder() {
        let body = r#"{"content":"test","cch=00000"}"#;
        let signed = sign_request_body(body);
        assert!(!signed.contains("cch=00000"));
        assert!(signed.contains("cch="));
    }

    #[test]
    fn test_build_auth_url_contains_required_params() {
        let verifier = PkceVerifier("test_verifier".to_string());
        let url = build_auth_url(&verifier, "random_state", 3456).expect("build_url");
        assert!(url.contains("client_id=9d1c250a-e61b-44d9-88ed-5944d1962f5e"));
        assert!(url.contains("code_challenge_method=S256"));
        assert!(url.contains("response_type=code"));
        assert!(url.contains("state=random_state"));
        assert!(url.contains("localhost%3A3456"));
    }

    #[test]
    fn test_constants_have_expected_values() {
        assert_eq!(
            CLAUDE_CODE_CLIENT_ID,
            "9d1c250a-e61b-44d9-88ed-5944d1962f5e"
        );
        assert!(ANTHROPIC_BETA.contains("claude-code-20250219"));
        assert!(ANTHROPIC_BETA.contains("oauth-2025-04-20"));
    }
}
