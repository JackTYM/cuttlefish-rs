//! MCP error types.

use thiserror::Error;

/// MCP errors.
#[derive(Debug, Error)]
pub enum McpError {
    /// Transport error (connection, I/O).
    #[error("Transport error: {0}")]
    Transport(String),

    /// Protocol error (invalid message format).
    #[error("Protocol error: {0}")]
    Protocol(String),

    /// Request timeout.
    #[error("Request timeout")]
    Timeout,

    /// Server returned an error.
    #[error("Server error ({code}): {message}")]
    Server {
        /// Error code.
        code: i32,
        /// Error message.
        message: String,
    },

    /// Tool not found.
    #[error("Tool not found: {0}")]
    ToolNotFound(String),

    /// Resource not found.
    #[error("Resource not found: {0}")]
    ResourceNotFound(String),

    /// Prompt not found.
    #[error("Prompt not found: {0}")]
    PromptNotFound(String),

    /// Server not initialized.
    #[error("Server not initialized")]
    NotInitialized,

    /// Invalid server response.
    #[error("Invalid server response: {0}")]
    InvalidResponse(String),
}
