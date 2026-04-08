//! LSP client error types.

use thiserror::Error;

/// LSP client errors.
#[derive(Debug, Error)]
pub enum LspError {
    /// Failed to spawn the language server process.
    #[error("Failed to spawn language server: {0}")]
    SpawnFailed(String),

    /// I/O error communicating with the server.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON serialization/deserialization error.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Request timeout.
    #[error("Request timeout")]
    Timeout,

    /// Server returned an error response.
    #[error("Server error ({code}): {message}")]
    ServerError {
        /// Error code.
        code: i32,
        /// Error message.
        message: String,
    },

    /// Server not initialized.
    #[error("Server not initialized")]
    NotInitialized,

    /// Invalid response from server.
    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    /// Server shutdown.
    #[error("Server has shut down")]
    ServerShutdown,

    /// File not found.
    #[error("File not found: {0}")]
    FileNotFound(String),

    /// Invalid URI.
    #[error("Invalid URI: {0}")]
    InvalidUri(String),
}
