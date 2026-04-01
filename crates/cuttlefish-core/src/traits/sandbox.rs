//! Sandbox trait and supporting types.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::Path;
use uuid::Uuid;

use crate::error::SandboxError;

/// Result type for sandbox operations.
pub type SandboxResult<T> = Result<T, SandboxError>;

/// Unique identifier for a sandbox instance.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SandboxId(pub String);

impl SandboxId {
    /// Create a new sandbox ID.
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }
}

impl Default for SandboxId {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration for creating a sandbox.
#[derive(Debug, Clone)]
pub struct SandboxConfig {
    /// Docker image to use.
    pub image: String,
    /// Memory limit in MB.
    pub memory_limit_mb: u64,
    /// CPU limit (fractional cores).
    pub cpu_limit: f64,
    /// Disk limit in GB.
    pub disk_limit_gb: u64,
    /// Environment variables.
    pub env_vars: Vec<(String, String)>,
    /// Network access enabled?
    pub network_enabled: bool,
    /// Execution timeout in seconds.
    pub timeout_secs: u64,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            image: "ubuntu:22.04".to_string(),
            memory_limit_mb: 2048,
            cpu_limit: 2.0,
            disk_limit_gb: 10,
            env_vars: Vec::new(),
            network_enabled: true,
            timeout_secs: 300,
        }
    }
}

/// Output from executing a command in a sandbox.
#[derive(Debug, Clone)]
pub struct ExecOutput {
    /// Standard output.
    pub stdout: String,
    /// Standard error.
    pub stderr: String,
    /// Exit code (0 = success).
    pub exit_code: i64,
    /// Whether the command timed out.
    pub timed_out: bool,
}

impl ExecOutput {
    /// Returns true if the command exited successfully.
    pub fn success(&self) -> bool {
        self.exit_code == 0 && !self.timed_out
    }
}

/// An isolated execution environment (Docker container, etc.).
#[async_trait]
pub trait Sandbox: Send + Sync {
    /// Create a new sandbox with the given configuration.
    async fn create(&self, config: &SandboxConfig) -> SandboxResult<SandboxId>;

    /// Execute a command inside the sandbox.
    async fn exec(&self, id: &SandboxId, command: &str) -> SandboxResult<ExecOutput>;

    /// Write a file into the sandbox.
    async fn write_file(
        &self,
        id: &SandboxId,
        path: &Path,
        content: &[u8],
    ) -> SandboxResult<()>;

    /// Read a file from the sandbox.
    async fn read_file(&self, id: &SandboxId, path: &Path) -> SandboxResult<Vec<u8>>;

    /// List files in a directory inside the sandbox.
    async fn list_files(&self, id: &SandboxId, path: &Path) -> SandboxResult<Vec<String>>;

    /// Destroy the sandbox and clean up resources.
    async fn destroy(&self, id: &SandboxId) -> SandboxResult<()>;
}
