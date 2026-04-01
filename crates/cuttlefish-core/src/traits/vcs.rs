//! Version control trait and supporting types.

use async_trait::async_trait;
use std::path::{Path, PathBuf};

use crate::error::VcsError;

/// Result type for VCS operations.
pub type VcsResult<T> = Result<T, VcsError>;

/// A version control system (Git, etc.).
#[async_trait]
pub trait VersionControl: Send + Sync {
    /// Clone a repository to a local path.
    async fn clone_repo(&self, url: &str, path: &Path) -> VcsResult<()>;

    /// Checkout an existing or create a new branch.
    async fn checkout_branch(&self, path: &Path, branch: &str, create: bool) -> VcsResult<()>;

    /// Stage specific files and create a commit.
    async fn commit(&self, path: &Path, message: &str, files: &[PathBuf]) -> VcsResult<String>;

    /// Push a branch to a remote.
    async fn push(&self, path: &Path, remote: &str, branch: &str) -> VcsResult<()>;

    /// Get the unified diff of working directory changes.
    async fn diff(&self, path: &Path) -> VcsResult<String>;

    /// Get recent commit log entries.
    async fn log(&self, path: &Path, limit: usize) -> VcsResult<Vec<CommitInfo>>;

    /// Get the current branch name.
    async fn current_branch(&self, path: &Path) -> VcsResult<String>;
}

/// Information about a git commit.
#[derive(Debug, Clone)]
pub struct CommitInfo {
    /// Commit hash (short).
    pub hash: String,
    /// Commit message.
    pub message: String,
    /// Author name.
    pub author: String,
    /// Timestamp (Unix seconds).
    pub timestamp: i64,
}
