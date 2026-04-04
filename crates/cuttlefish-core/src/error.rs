//! Error types for the Cuttlefish platform.

use thiserror::Error;

/// Top-level error type for Cuttlefish.
#[derive(Error, Debug)]
pub enum CuttlefishError {
    /// Configuration error.
    #[error("Config error: {0}")]
    Config(#[from] ConfigError),
    /// Provider/model error.
    #[error("Provider error: {0}")]
    Provider(#[from] ProviderError),
    /// Sandbox error.
    #[error("Sandbox error: {0}")]
    Sandbox(#[from] SandboxError),
    /// VCS/Git error.
    #[error("VCS error: {0}")]
    Vcs(#[from] VcsError),
    /// Agent error.
    #[error("Agent error: {0}")]
    Agent(#[from] AgentError),
    /// Database error.
    #[error("Database error: {0}")]
    Database(#[from] DatabaseError),
    /// Discord error.
    #[error("Discord error: {0}")]
    Discord(#[from] DiscordError),
    /// API error.
    #[error("API error: {0}")]
    Api(#[from] ApiError),
    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// Configuration error.
#[derive(Error, Debug)]
#[error("{0}")]
pub struct ConfigError(pub String);

/// Provider/model error.
#[derive(Error, Debug)]
#[error("{0}")]
pub struct ProviderError(pub String);

/// Sandbox error.
#[derive(Error, Debug)]
pub enum SandboxError {
    /// Docker image was not found locally or in remote registry.
    #[error("Image not found: {name}:{tag}")]
    ImageNotFound {
        /// Image name.
        name: String,
        /// Image tag.
        tag: String,
    },

    /// Docker image build failed.
    #[error("Image build failed: {reason}")]
    ImageBuildFailed {
        /// Failure reason.
        reason: String,
    },

    /// Container with the given ID was not found.
    #[error("Container not found: {id}")]
    ContainerNotFound {
        /// Container ID.
        id: String,
    },

    /// Command execution timed out.
    #[error("Execution timeout after {seconds}s")]
    Timeout {
        /// Timeout duration in seconds.
        seconds: u64,
    },

    /// Container exceeded resource limits (CPU, memory, disk).
    #[error("Resource limit exceeded: {resource}")]
    ResourceLimitExceeded {
        /// Which resource was exceeded.
        resource: String,
    },

    /// Failed to mount a volume into the container.
    #[error("Volume mount error: {reason}")]
    VolumeMountError {
        /// Failure reason.
        reason: String,
    },

    /// Snapshot operation failed.
    #[error("Snapshot error: {reason}")]
    SnapshotError {
        /// Failure reason.
        reason: String,
    },

    /// Invalid or unsupported programming language.
    #[error("Invalid language: {0}")]
    InvalidLanguage(String),

    /// I/O error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Other sandbox error.
    #[error("{0}")]
    Other(String),
}

/// VCS/Git error.
#[derive(Error, Debug)]
#[error("{0}")]
pub struct VcsError(pub String);

/// Agent error.
#[derive(Error, Debug)]
#[error("{0}")]
pub struct AgentError(pub String);

/// Database error.
#[derive(Error, Debug)]
#[error("{0}")]
pub struct DatabaseError(pub String);

/// Discord error.
#[derive(Error, Debug)]
#[error("{0}")]
pub struct DiscordError(pub String);

/// API error.
#[derive(Error, Debug)]
#[error("{0}")]
pub struct ApiError(pub String);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_error_display() {
        let err = ConfigError("test config error".to_string());
        assert_eq!(err.to_string(), "test config error");
    }

    #[test]
    fn test_cuttlefish_error_from_config() {
        let config_err = ConfigError("invalid config".to_string());
        let err: CuttlefishError = config_err.into();
        assert_eq!(err.to_string(), "Config error: invalid config");
    }

    #[test]
    fn test_cuttlefish_error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err: CuttlefishError = io_err.into();
        assert!(err.to_string().contains("I/O error"));
    }

    #[test]
    fn test_provider_error_display() {
        let err = ProviderError("provider unavailable".to_string());
        assert_eq!(err.to_string(), "provider unavailable");
    }

    #[test]
    fn test_sandbox_error_display() {
        let err = SandboxError::Other("sandbox failed".to_string());
        assert_eq!(err.to_string(), "sandbox failed");
    }

    #[test]
    fn test_vcs_error_display() {
        let err = VcsError("git error".to_string());
        assert_eq!(err.to_string(), "git error");
    }

    #[test]
    fn test_agent_error_display() {
        let err = AgentError("agent crashed".to_string());
        assert_eq!(err.to_string(), "agent crashed");
    }

    #[test]
    fn test_database_error_display() {
        let err = DatabaseError("connection failed".to_string());
        assert_eq!(err.to_string(), "connection failed");
    }

    #[test]
    fn test_discord_error_display() {
        let err = DiscordError("discord api error".to_string());
        assert_eq!(err.to_string(), "discord api error");
    }

    #[test]
    fn test_api_error_display() {
        let err = ApiError("request failed".to_string());
        assert_eq!(err.to_string(), "request failed");
    }
}
