//! Error types for the tunnel system.

use thiserror::Error;

/// Errors that can occur in tunnel operations.
#[derive(Debug, Error)]
pub enum TunnelError {
    /// WebSocket connection error
    #[error("WebSocket error: {0}")]
    WebSocket(Box<tokio_tungstenite::tungstenite::Error>),

    /// JSON serialization/deserialization error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Authentication failed
    #[error("Authentication failed: {0}")]
    AuthFailed(String),

    /// Invalid link code
    #[error("Invalid link code: {0}")]
    InvalidLinkCode(String),

    /// JWT error
    #[error("JWT error: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),

    /// Connection closed unexpectedly
    #[error("Connection closed: {0}")]
    ConnectionClosed(String),

    /// Request timeout
    #[error("Request timeout after {0}ms")]
    Timeout(u64),

    /// HTTP forwarding error
    #[error("HTTP forward error: {0}")]
    HttpForward(String),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Tunnel not found
    #[error("Tunnel not found for subdomain: {0}")]
    TunnelNotFound(String),
}

impl From<tokio_tungstenite::tungstenite::Error> for TunnelError {
    fn from(err: tokio_tungstenite::tungstenite::Error) -> Self {
        Self::WebSocket(Box::new(err))
    }
}
