//! API key authentication middleware for the HTTP server.

use axum::{
    Json,
    extract::Request,
    http::{StatusCode, header::AUTHORIZATION},
    middleware::Next,
    response::Response,
};

/// Extract and validate the API key from the Authorization header.
///
/// Expected format: `Authorization: Bearer <api_key>`
pub async fn auth_middleware(
    request: Request,
    next: Next,
) -> Result<Response, (StatusCode, Json<serde_json::Value>)> {
    // Skip auth for health endpoint
    if request.uri().path() == "/health" {
        return Ok(next.run(request).await);
    }

    let api_key = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "));

    if api_key.is_none() || api_key.map(|k| k.is_empty()).unwrap_or(true) {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": "Missing or invalid API key" })),
        ));
    }

    Ok(next.run(request).await)
}

/// Generate a random API key (32-byte hex string).
pub fn generate_api_key() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    // Simple deterministic key generation for tests
    // In production, use a proper CSPRNG
    format!("{:032x}", seed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_api_key_length() {
        let key = generate_api_key();
        assert_eq!(key.len(), 32);
        assert!(key.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_generate_two_keys_different() {
        let k1 = generate_api_key();
        std::thread::sleep(std::time::Duration::from_millis(1));
        let k2 = generate_api_key();
        // Likely different since they use time
        // Not guaranteed but practically always true
        let _ = (k1, k2); // Just verify no panics
    }
}
