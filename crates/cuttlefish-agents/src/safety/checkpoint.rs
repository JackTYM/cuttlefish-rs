//! Checkpoint system for capturing and restoring project state.
//!
//! This module provides:
//! - [`Checkpoint`] struct capturing container, git, and memory state
//! - [`CheckpointStore`] trait for storage abstraction
//! - [`CheckpointManager`] for creating, listing, and restoring checkpoints
//!
//! Checkpoints enable users to rollback to safe states after agent mistakes.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::path::PathBuf;
use std::time::Duration;
use thiserror::Error;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Maximum number of checkpoints per project.
pub const MAX_CHECKPOINTS_PER_PROJECT: usize = 20;

/// Default timeout for checkpoint operations in seconds.
pub const DEFAULT_CHECKPOINT_TIMEOUT_SECS: u64 = 30;

/// Environment variable keys that should never be stored in checkpoints.
const SECRET_ENV_KEYS: &[&str] = &[
    "API_KEY",
    "SECRET",
    "PASSWORD",
    "TOKEN",
    "CREDENTIAL",
    "PRIVATE_KEY",
    "AWS_SECRET",
    "ANTHROPIC_API_KEY",
    "OPENAI_API_KEY",
    "DISCORD_BOT_TOKEN",
    "CUTTLEFISH_API_KEY",
];

/// Unique identifier for a checkpoint.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CheckpointId(pub String);

impl CheckpointId {
    /// Create a new random checkpoint ID.
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }

    /// Create a checkpoint ID from an existing string.
    pub fn from_string(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    /// Get the inner string value.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for CheckpointId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for CheckpointId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// What triggered the checkpoint creation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CheckpointTrigger {
    /// User manually requested a checkpoint.
    Manual {
        /// ID of the user who requested the checkpoint.
        user_id: String,
    },
    /// Automatic checkpoint before a risky operation.
    AutoPreRiskyOp {
        /// Description of the risky operation.
        operation: String,
    },
    /// Scheduled checkpoint (e.g., periodic backup).
    Scheduled,
    /// Checkpoint created before a rollback operation.
    PreRollback {
        /// ID of the checkpoint being rolled back to.
        target_checkpoint_id: CheckpointId,
    },
}

impl fmt::Display for CheckpointTrigger {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Manual { user_id } => write!(f, "Manual (user: {user_id})"),
            Self::AutoPreRiskyOp { operation } => write!(f, "Auto (before: {operation})"),
            Self::Scheduled => write!(f, "Scheduled"),
            Self::PreRollback {
                target_checkpoint_id,
            } => {
                write!(f, "Pre-rollback (target: {target_checkpoint_id})")
            }
        }
    }
}

/// Components captured in a checkpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointComponents {
    /// Git reference (branch name or commit SHA).
    pub git_ref: String,
    /// ID of the container snapshot (Docker image ID).
    pub container_snapshot_id: String,
    /// Path to the memory backup file.
    pub memory_backup_path: PathBuf,
    /// Non-secret environment variables at checkpoint time.
    pub env_snapshot: HashMap<String, String>,
}

impl CheckpointComponents {
    /// Create new checkpoint components.
    pub fn new(
        git_ref: impl Into<String>,
        container_snapshot_id: impl Into<String>,
        memory_backup_path: impl Into<PathBuf>,
    ) -> Self {
        Self {
            git_ref: git_ref.into(),
            container_snapshot_id: container_snapshot_id.into(),
            memory_backup_path: memory_backup_path.into(),
            env_snapshot: HashMap::new(),
        }
    }

    /// Add environment variables, filtering out secrets.
    pub fn with_env(mut self, env: HashMap<String, String>) -> Self {
        self.env_snapshot = filter_secret_env_vars(env);
        self
    }
}

/// A checkpoint capturing project state at a point in time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    /// Unique checkpoint identifier.
    pub id: CheckpointId,
    /// Project this checkpoint belongs to.
    pub project_id: String,
    /// When the checkpoint was created.
    pub created_at: DateTime<Utc>,
    /// Human-readable description.
    pub description: String,
    /// What triggered this checkpoint.
    pub trigger: CheckpointTrigger,
    /// Captured state components.
    pub components: CheckpointComponents,
}

impl Checkpoint {
    /// Create a new checkpoint.
    pub fn new(
        project_id: impl Into<String>,
        description: impl Into<String>,
        trigger: CheckpointTrigger,
        components: CheckpointComponents,
    ) -> Self {
        Self {
            id: CheckpointId::new(),
            project_id: project_id.into(),
            created_at: Utc::now(),
            description: description.into(),
            trigger,
            components,
        }
    }

    /// Create a checkpoint with a specific ID (for testing or restoration).
    pub fn with_id(
        id: CheckpointId,
        project_id: impl Into<String>,
        description: impl Into<String>,
        trigger: CheckpointTrigger,
        components: CheckpointComponents,
    ) -> Self {
        Self {
            id,
            project_id: project_id.into(),
            created_at: Utc::now(),
            description: description.into(),
            trigger,
            components,
        }
    }

    /// Get the age of this checkpoint.
    pub fn age(&self) -> chrono::Duration {
        Utc::now().signed_duration_since(self.created_at)
    }
}

impl fmt::Display for Checkpoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}] {} - {} ({})",
            self.id,
            self.description,
            self.created_at.format("%Y-%m-%d %H:%M:%S UTC"),
            self.trigger
        )
    }
}

/// Errors that can occur during checkpoint operations.
#[derive(Error, Debug)]
pub enum CheckpointError {
    /// Checkpoint was not found.
    #[error("Checkpoint not found: {id}")]
    NotFound {
        /// The checkpoint ID that was not found.
        id: CheckpointId,
    },

    /// Failed to create a checkpoint.
    #[error("Failed to create checkpoint: {reason}")]
    CreationFailed {
        /// Reason for the failure.
        reason: String,
    },

    /// Failed to restore from a checkpoint.
    #[error("Failed to restore checkpoint: {reason}")]
    RestoreFailed {
        /// Reason for the failure.
        reason: String,
    },

    /// Checkpoint operation timed out.
    #[error("Checkpoint operation timed out after {seconds}s")]
    Timeout {
        /// Timeout duration in seconds.
        seconds: u64,
    },

    /// Storage operation failed.
    #[error("Storage error: {reason}")]
    StorageError {
        /// Reason for the failure.
        reason: String,
    },

    /// Git operation failed during checkpoint.
    #[error("Git error: {reason}")]
    GitError {
        /// Reason for the failure.
        reason: String,
    },

    /// Container snapshot operation failed.
    #[error("Container snapshot error: {reason}")]
    ContainerError {
        /// Reason for the failure.
        reason: String,
    },

    /// Memory backup operation failed.
    #[error("Memory backup error: {reason}")]
    MemoryError {
        /// Reason for the failure.
        reason: String,
    },

    /// Project not found.
    #[error("Project not found: {project_id}")]
    ProjectNotFound {
        /// The project ID that was not found.
        project_id: String,
    },

    /// Checkpoint limit exceeded.
    #[error("Checkpoint limit exceeded for project {project_id}: max {max} checkpoints")]
    LimitExceeded {
        /// The project ID.
        project_id: String,
        /// Maximum allowed checkpoints.
        max: usize,
    },
}

/// Result type for checkpoint operations.
pub type CheckpointResult<T> = Result<T, CheckpointError>;

/// Trait for checkpoint storage backends.
///
/// Implementations can store checkpoints in a database, filesystem, or other storage.
#[async_trait]
pub trait CheckpointStore: Send + Sync {
    /// Save a checkpoint to storage.
    async fn save(&self, checkpoint: &Checkpoint) -> CheckpointResult<()>;

    /// Load a checkpoint by ID.
    async fn load(&self, id: &CheckpointId) -> CheckpointResult<Checkpoint>;

    /// Delete a checkpoint by ID.
    async fn delete(&self, id: &CheckpointId) -> CheckpointResult<()>;

    /// List all checkpoints for a project, ordered by creation time (newest first).
    async fn list_for_project(&self, project_id: &str) -> CheckpointResult<Vec<Checkpoint>>;

    /// Count checkpoints for a project.
    async fn count_for_project(&self, project_id: &str) -> CheckpointResult<usize>;

    /// Get the oldest checkpoint for a project.
    async fn get_oldest_for_project(
        &self,
        project_id: &str,
    ) -> CheckpointResult<Option<Checkpoint>>;
}

/// Configuration for the checkpoint manager.
#[derive(Debug, Clone)]
pub struct CheckpointConfig {
    /// Maximum checkpoints per project.
    pub max_checkpoints_per_project: usize,
    /// Timeout for checkpoint operations.
    pub operation_timeout: Duration,
    /// Base directory for checkpoint data.
    pub checkpoint_dir: PathBuf,
    /// Operations that trigger automatic checkpoints.
    pub auto_checkpoint_triggers: Vec<String>,
}

impl Default for CheckpointConfig {
    fn default() -> Self {
        Self {
            max_checkpoints_per_project: MAX_CHECKPOINTS_PER_PROJECT,
            operation_timeout: Duration::from_secs(DEFAULT_CHECKPOINT_TIMEOUT_SECS),
            checkpoint_dir: PathBuf::from(".cuttlefish/checkpoints"),
            auto_checkpoint_triggers: vec![
                "git reset".to_string(),
                "git rebase".to_string(),
                "rm -rf".to_string(),
                "rm -r".to_string(),
                "file delete".to_string(),
            ],
        }
    }
}

impl CheckpointConfig {
    /// Create a new config with custom max checkpoints.
    pub fn with_max_checkpoints(mut self, max: usize) -> Self {
        self.max_checkpoints_per_project = max;
        self
    }

    /// Create a new config with custom timeout.
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.operation_timeout = timeout;
        self
    }

    /// Create a new config with custom checkpoint directory.
    pub fn with_checkpoint_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.checkpoint_dir = dir.into();
        self
    }

    /// Add an auto-checkpoint trigger.
    pub fn with_auto_trigger(mut self, trigger: impl Into<String>) -> Self {
        self.auto_checkpoint_triggers.push(trigger.into());
        self
    }

    /// Check if an operation should trigger an automatic checkpoint.
    pub fn should_auto_checkpoint(&self, operation: &str) -> bool {
        let op_lower = operation.to_lowercase();
        self.auto_checkpoint_triggers
            .iter()
            .any(|t| op_lower.contains(&t.to_lowercase()))
    }
}

/// Result of a rollback operation.
#[derive(Debug, Clone)]
pub struct RollbackResult {
    /// The checkpoint that was restored.
    pub restored_checkpoint: Checkpoint,
    /// Optional checkpoint created before rollback (safety checkpoint).
    pub safety_checkpoint: Option<Checkpoint>,
    /// Whether git state was restored.
    pub git_restored: bool,
    /// Whether container state was restored.
    pub container_restored: bool,
    /// Whether memory state was restored.
    pub memory_restored: bool,
}

impl RollbackResult {
    /// Check if all components were successfully restored.
    pub fn fully_restored(&self) -> bool {
        self.git_restored && self.container_restored && self.memory_restored
    }
}

/// Manager for checkpoint operations.
///
/// Coordinates checkpoint creation, storage, and restoration across
/// git, container, and memory subsystems.
pub struct CheckpointManager<S: CheckpointStore> {
    store: S,
    config: CheckpointConfig,
}

impl<S: CheckpointStore> CheckpointManager<S> {
    /// Create a new checkpoint manager with the given store and config.
    pub fn new(store: S, config: CheckpointConfig) -> Self {
        Self { store, config }
    }

    /// Create a new checkpoint manager with default config.
    pub fn with_defaults(store: S) -> Self {
        Self::new(store, CheckpointConfig::default())
    }

    /// Get the configuration.
    pub fn config(&self) -> &CheckpointConfig {
        &self.config
    }

    /// Get a reference to the store.
    pub fn store(&self) -> &S {
        &self.store
    }

    /// Create a new checkpoint for a project.
    ///
    /// This captures the current git state, container snapshot, and memory backup.
    /// If the project has reached the checkpoint limit, the oldest checkpoint is deleted.
    pub async fn create_checkpoint(
        &self,
        project_id: &str,
        description: &str,
        trigger: CheckpointTrigger,
        components: CheckpointComponents,
    ) -> CheckpointResult<Checkpoint> {
        info!(
            project_id = project_id,
            description = description,
            "Creating checkpoint"
        );

        // Enforce checkpoint limit by rotating oldest
        self.enforce_checkpoint_limit(project_id).await?;

        // Create the checkpoint
        let checkpoint = Checkpoint::new(project_id, description, trigger, components);

        // Save to store
        self.store.save(&checkpoint).await?;

        info!(
            checkpoint_id = %checkpoint.id,
            project_id = project_id,
            "Checkpoint created successfully"
        );

        Ok(checkpoint)
    }

    /// Create an automatic checkpoint before a risky operation.
    ///
    /// Returns `None` if the operation doesn't match any auto-checkpoint triggers.
    pub async fn maybe_auto_checkpoint(
        &self,
        project_id: &str,
        operation: &str,
        components: CheckpointComponents,
    ) -> CheckpointResult<Option<Checkpoint>> {
        if !self.config.should_auto_checkpoint(operation) {
            debug!(
                operation = operation,
                "Operation does not trigger auto-checkpoint"
            );
            return Ok(None);
        }

        let description = format!("Auto-checkpoint before: {operation}");
        let trigger = CheckpointTrigger::AutoPreRiskyOp {
            operation: operation.to_string(),
        };

        let checkpoint = self
            .create_checkpoint(project_id, &description, trigger, components)
            .await?;

        Ok(Some(checkpoint))
    }

    /// List all checkpoints for a project.
    pub async fn list_checkpoints(&self, project_id: &str) -> CheckpointResult<Vec<Checkpoint>> {
        self.store.list_for_project(project_id).await
    }

    /// Get a specific checkpoint by ID.
    pub async fn get_checkpoint(&self, id: &CheckpointId) -> CheckpointResult<Checkpoint> {
        self.store.load(id).await
    }

    /// Delete a checkpoint.
    pub async fn delete_checkpoint(&self, id: &CheckpointId) -> CheckpointResult<()> {
        info!(checkpoint_id = %id, "Deleting checkpoint");
        self.store.delete(id).await
    }

    /// Restore project state from a checkpoint.
    ///
    /// If `create_safety_checkpoint` is true, creates a checkpoint of the current
    /// state before restoring (allows undoing the rollback).
    ///
    /// Note: This method prepares the rollback result but actual restoration
    /// of git, container, and memory state must be performed by the caller
    /// using the appropriate subsystem APIs.
    pub async fn prepare_rollback(
        &self,
        project_id: &str,
        checkpoint_id: &CheckpointId,
        create_safety_checkpoint: bool,
        current_components: Option<CheckpointComponents>,
    ) -> CheckpointResult<(Checkpoint, Option<Checkpoint>)> {
        info!(
            project_id = project_id,
            checkpoint_id = %checkpoint_id,
            create_safety = create_safety_checkpoint,
            "Preparing rollback"
        );

        // Load the target checkpoint
        let target = self.store.load(checkpoint_id).await?;

        // Verify it belongs to the correct project
        if target.project_id != project_id {
            return Err(CheckpointError::NotFound {
                id: checkpoint_id.clone(),
            });
        }

        // Optionally create a safety checkpoint first
        let safety_checkpoint = if create_safety_checkpoint {
            if let Some(components) = current_components {
                let trigger = CheckpointTrigger::PreRollback {
                    target_checkpoint_id: checkpoint_id.clone(),
                };
                let description = format!("Safety checkpoint before rollback to {checkpoint_id}");
                let safety = self
                    .create_checkpoint(project_id, &description, trigger, components)
                    .await?;
                Some(safety)
            } else {
                warn!("Cannot create safety checkpoint: no current components provided");
                None
            }
        } else {
            None
        };

        Ok((target, safety_checkpoint))
    }

    /// Enforce the checkpoint limit for a project.
    ///
    /// If the project has reached the maximum number of checkpoints,
    /// the oldest checkpoint is deleted.
    async fn enforce_checkpoint_limit(&self, project_id: &str) -> CheckpointResult<()> {
        let count = self.store.count_for_project(project_id).await?;

        if count >= self.config.max_checkpoints_per_project {
            debug!(
                project_id = project_id,
                count = count,
                max = self.config.max_checkpoints_per_project,
                "Checkpoint limit reached, rotating oldest"
            );

            if let Some(oldest) = self.store.get_oldest_for_project(project_id).await? {
                info!(
                    checkpoint_id = %oldest.id,
                    project_id = project_id,
                    "Deleting oldest checkpoint to make room"
                );
                self.store.delete(&oldest.id).await?;
            }
        }

        Ok(())
    }
}

/// Filter out environment variables that might contain secrets.
fn filter_secret_env_vars(env: HashMap<String, String>) -> HashMap<String, String> {
    env.into_iter()
        .filter(|(key, _)| {
            let key_upper = key.to_uppercase();
            !SECRET_ENV_KEYS
                .iter()
                .any(|secret| key_upper.contains(secret))
        })
        .collect()
}

/// In-memory checkpoint store for testing.
#[derive(Debug, Default)]
pub struct InMemoryCheckpointStore {
    checkpoints: std::sync::RwLock<HashMap<CheckpointId, Checkpoint>>,
}

impl InMemoryCheckpointStore {
    /// Create a new empty in-memory store.
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl CheckpointStore for InMemoryCheckpointStore {
    async fn save(&self, checkpoint: &Checkpoint) -> CheckpointResult<()> {
        let mut checkpoints =
            self.checkpoints
                .write()
                .map_err(|e| CheckpointError::StorageError {
                    reason: format!("Lock poisoned: {e}"),
                })?;
        checkpoints.insert(checkpoint.id.clone(), checkpoint.clone());
        Ok(())
    }

    async fn load(&self, id: &CheckpointId) -> CheckpointResult<Checkpoint> {
        let checkpoints = self
            .checkpoints
            .read()
            .map_err(|e| CheckpointError::StorageError {
                reason: format!("Lock poisoned: {e}"),
            })?;
        checkpoints
            .get(id)
            .cloned()
            .ok_or_else(|| CheckpointError::NotFound { id: id.clone() })
    }

    async fn delete(&self, id: &CheckpointId) -> CheckpointResult<()> {
        let mut checkpoints =
            self.checkpoints
                .write()
                .map_err(|e| CheckpointError::StorageError {
                    reason: format!("Lock poisoned: {e}"),
                })?;
        checkpoints.remove(id);
        Ok(())
    }

    async fn list_for_project(&self, project_id: &str) -> CheckpointResult<Vec<Checkpoint>> {
        let checkpoints = self
            .checkpoints
            .read()
            .map_err(|e| CheckpointError::StorageError {
                reason: format!("Lock poisoned: {e}"),
            })?;
        let mut result: Vec<_> = checkpoints
            .values()
            .filter(|c| c.project_id == project_id)
            .cloned()
            .collect();
        // Sort by created_at descending (newest first)
        result.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(result)
    }

    async fn count_for_project(&self, project_id: &str) -> CheckpointResult<usize> {
        let checkpoints = self
            .checkpoints
            .read()
            .map_err(|e| CheckpointError::StorageError {
                reason: format!("Lock poisoned: {e}"),
            })?;
        Ok(checkpoints
            .values()
            .filter(|c| c.project_id == project_id)
            .count())
    }

    async fn get_oldest_for_project(
        &self,
        project_id: &str,
    ) -> CheckpointResult<Option<Checkpoint>> {
        let checkpoints = self
            .checkpoints
            .read()
            .map_err(|e| CheckpointError::StorageError {
                reason: format!("Lock poisoned: {e}"),
            })?;
        Ok(checkpoints
            .values()
            .filter(|c| c.project_id == project_id)
            .min_by_key(|c| c.created_at)
            .cloned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_components() -> CheckpointComponents {
        CheckpointComponents::new("main", "snapshot-123", "/tmp/memory.json")
    }

    fn make_components_with_env() -> CheckpointComponents {
        let mut env = HashMap::new();
        env.insert("PATH".to_string(), "/usr/bin".to_string());
        env.insert("HOME".to_string(), "/home/user".to_string());
        env.insert("API_KEY".to_string(), "secret123".to_string());
        env.insert("MY_SECRET_TOKEN".to_string(), "token456".to_string());
        env.insert("NORMAL_VAR".to_string(), "value".to_string());

        CheckpointComponents::new("main", "snapshot-123", "/tmp/memory.json").with_env(env)
    }

    #[test]
    fn test_checkpoint_id_generation() {
        let id1 = CheckpointId::new();
        let id2 = CheckpointId::new();
        assert_ne!(id1, id2);
        assert!(!id1.as_str().is_empty());
    }

    #[test]
    fn test_checkpoint_id_from_string() {
        let id = CheckpointId::from_string("test-id-123");
        assert_eq!(id.as_str(), "test-id-123");
        assert_eq!(format!("{id}"), "test-id-123");
    }

    #[test]
    fn test_checkpoint_trigger_display() {
        let manual = CheckpointTrigger::Manual {
            user_id: "user123".to_string(),
        };
        assert!(format!("{manual}").contains("user123"));

        let auto = CheckpointTrigger::AutoPreRiskyOp {
            operation: "rm -rf".to_string(),
        };
        assert!(format!("{auto}").contains("rm -rf"));

        let scheduled = CheckpointTrigger::Scheduled;
        assert!(format!("{scheduled}").contains("Scheduled"));

        let pre_rollback = CheckpointTrigger::PreRollback {
            target_checkpoint_id: CheckpointId::from_string("target-123"),
        };
        assert!(format!("{pre_rollback}").contains("target-123"));
    }

    #[test]
    fn test_checkpoint_components_creation() {
        let components = CheckpointComponents::new("feature-branch", "snap-456", "/data/mem.json");
        assert_eq!(components.git_ref, "feature-branch");
        assert_eq!(components.container_snapshot_id, "snap-456");
        assert_eq!(
            components.memory_backup_path,
            PathBuf::from("/data/mem.json")
        );
        assert!(components.env_snapshot.is_empty());
    }

    #[test]
    fn test_checkpoint_components_filters_secrets() {
        let components = make_components_with_env();

        // Should keep non-secret vars
        assert!(components.env_snapshot.contains_key("PATH"));
        assert!(components.env_snapshot.contains_key("HOME"));
        assert!(components.env_snapshot.contains_key("NORMAL_VAR"));

        // Should filter secret vars
        assert!(!components.env_snapshot.contains_key("API_KEY"));
        assert!(!components.env_snapshot.contains_key("MY_SECRET_TOKEN"));
    }

    #[test]
    fn test_checkpoint_creation() {
        let components = make_components();
        let trigger = CheckpointTrigger::Manual {
            user_id: "user1".to_string(),
        };
        let checkpoint = Checkpoint::new("project-1", "Test checkpoint", trigger, components);

        assert_eq!(checkpoint.project_id, "project-1");
        assert_eq!(checkpoint.description, "Test checkpoint");
        assert!(!checkpoint.id.as_str().is_empty());
    }

    #[test]
    fn test_checkpoint_with_id() {
        let id = CheckpointId::from_string("custom-id");
        let components = make_components();
        let trigger = CheckpointTrigger::Scheduled;
        let checkpoint = Checkpoint::with_id(
            id.clone(),
            "project-2",
            "Custom ID checkpoint",
            trigger,
            components,
        );

        assert_eq!(checkpoint.id, id);
        assert_eq!(checkpoint.project_id, "project-2");
    }

    #[test]
    fn test_checkpoint_display() {
        let components = make_components();
        let trigger = CheckpointTrigger::Scheduled;
        let checkpoint = Checkpoint::new("proj", "My checkpoint", trigger, components);

        let display = format!("{checkpoint}");
        assert!(display.contains("My checkpoint"));
        assert!(display.contains("Scheduled"));
    }

    #[test]
    fn test_checkpoint_config_defaults() {
        let config = CheckpointConfig::default();
        assert_eq!(
            config.max_checkpoints_per_project,
            MAX_CHECKPOINTS_PER_PROJECT
        );
        assert_eq!(
            config.operation_timeout,
            Duration::from_secs(DEFAULT_CHECKPOINT_TIMEOUT_SECS)
        );
    }

    #[test]
    fn test_checkpoint_config_builders() {
        let config = CheckpointConfig::default()
            .with_max_checkpoints(10)
            .with_timeout(Duration::from_secs(60))
            .with_checkpoint_dir("/custom/path")
            .with_auto_trigger("npm uninstall");

        assert_eq!(config.max_checkpoints_per_project, 10);
        assert_eq!(config.operation_timeout, Duration::from_secs(60));
        assert_eq!(config.checkpoint_dir, PathBuf::from("/custom/path"));
        assert!(
            config
                .auto_checkpoint_triggers
                .contains(&"npm uninstall".to_string())
        );
    }

    #[test]
    fn test_should_auto_checkpoint() {
        let config = CheckpointConfig::default();

        assert!(config.should_auto_checkpoint("git reset --hard HEAD"));
        assert!(config.should_auto_checkpoint("git rebase main"));
        assert!(config.should_auto_checkpoint("rm -rf node_modules"));
        assert!(config.should_auto_checkpoint("rm -r build/"));
        assert!(config.should_auto_checkpoint("file delete src/main.rs"));

        assert!(!config.should_auto_checkpoint("git status"));
        assert!(!config.should_auto_checkpoint("cargo build"));
        assert!(!config.should_auto_checkpoint("ls -la"));
    }

    #[test]
    fn test_checkpoint_error_display() {
        let err = CheckpointError::NotFound {
            id: CheckpointId::from_string("missing-123"),
        };
        assert!(err.to_string().contains("missing-123"));

        let err = CheckpointError::Timeout { seconds: 30 };
        assert!(err.to_string().contains("30"));

        let err = CheckpointError::LimitExceeded {
            project_id: "proj".to_string(),
            max: 20,
        };
        assert!(err.to_string().contains("proj"));
        assert!(err.to_string().contains("20"));
    }

    #[test]
    fn test_rollback_result_fully_restored() {
        let components = make_components();
        let checkpoint = Checkpoint::new("proj", "test", CheckpointTrigger::Scheduled, components);

        let result = RollbackResult {
            restored_checkpoint: checkpoint,
            safety_checkpoint: None,
            git_restored: true,
            container_restored: true,
            memory_restored: true,
        };
        assert!(result.fully_restored());

        let partial_result = RollbackResult {
            restored_checkpoint: result.restored_checkpoint.clone(),
            safety_checkpoint: None,
            git_restored: true,
            container_restored: false,
            memory_restored: true,
        };
        assert!(!partial_result.fully_restored());
    }

    #[tokio::test]
    async fn test_in_memory_store_save_and_load() {
        let store = InMemoryCheckpointStore::new();
        let components = make_components();
        let checkpoint = Checkpoint::new(
            "project-1",
            "Test checkpoint",
            CheckpointTrigger::Scheduled,
            components,
        );

        store.save(&checkpoint).await.expect("save should succeed");

        let loaded = store
            .load(&checkpoint.id)
            .await
            .expect("load should succeed");
        assert_eq!(loaded.id, checkpoint.id);
        assert_eq!(loaded.project_id, checkpoint.project_id);
        assert_eq!(loaded.description, checkpoint.description);
    }

    #[tokio::test]
    async fn test_in_memory_store_delete() {
        let store = InMemoryCheckpointStore::new();
        let components = make_components();
        let checkpoint = Checkpoint::new(
            "project-1",
            "Test checkpoint",
            CheckpointTrigger::Scheduled,
            components,
        );

        store.save(&checkpoint).await.expect("save should succeed");
        store
            .delete(&checkpoint.id)
            .await
            .expect("delete should succeed");

        let result = store.load(&checkpoint.id).await;
        assert!(matches!(result, Err(CheckpointError::NotFound { .. })));
    }

    #[tokio::test]
    async fn test_in_memory_store_list_for_project() {
        let store = InMemoryCheckpointStore::new();

        // Create checkpoints for different projects
        for i in 0..3 {
            let components = make_components();
            let checkpoint = Checkpoint::new(
                "project-1",
                format!("Checkpoint {i}"),
                CheckpointTrigger::Scheduled,
                components,
            );
            store.save(&checkpoint).await.expect("save should succeed");
        }

        for i in 0..2 {
            let components = make_components();
            let checkpoint = Checkpoint::new(
                "project-2",
                format!("Other checkpoint {i}"),
                CheckpointTrigger::Scheduled,
                components,
            );
            store.save(&checkpoint).await.expect("save should succeed");
        }

        let project1_checkpoints = store
            .list_for_project("project-1")
            .await
            .expect("list should succeed");
        assert_eq!(project1_checkpoints.len(), 3);

        let project2_checkpoints = store
            .list_for_project("project-2")
            .await
            .expect("list should succeed");
        assert_eq!(project2_checkpoints.len(), 2);

        let project3_checkpoints = store
            .list_for_project("project-3")
            .await
            .expect("list should succeed");
        assert!(project3_checkpoints.is_empty());
    }

    #[tokio::test]
    async fn test_in_memory_store_count_for_project() {
        let store = InMemoryCheckpointStore::new();

        for i in 0..5 {
            let components = make_components();
            let checkpoint = Checkpoint::new(
                "project-1",
                format!("Checkpoint {i}"),
                CheckpointTrigger::Scheduled,
                components,
            );
            store.save(&checkpoint).await.expect("save should succeed");
        }

        let count = store
            .count_for_project("project-1")
            .await
            .expect("count should succeed");
        assert_eq!(count, 5);

        let count = store
            .count_for_project("nonexistent")
            .await
            .expect("count should succeed");
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_in_memory_store_get_oldest() {
        let store = InMemoryCheckpointStore::new();

        // Create checkpoints with slight delays to ensure different timestamps
        let mut oldest_id = None;
        for i in 0..3 {
            let components = make_components();
            let checkpoint = Checkpoint::new(
                "project-1",
                format!("Checkpoint {i}"),
                CheckpointTrigger::Scheduled,
                components,
            );
            if i == 0 {
                oldest_id = Some(checkpoint.id.clone());
            }
            store.save(&checkpoint).await.expect("save should succeed");
        }

        let oldest = store
            .get_oldest_for_project("project-1")
            .await
            .expect("get_oldest should succeed")
            .expect("should have oldest");

        assert_eq!(oldest.id, oldest_id.expect("should have oldest_id"));
    }

    #[tokio::test]
    async fn test_checkpoint_manager_create() {
        let store = InMemoryCheckpointStore::new();
        let manager = CheckpointManager::with_defaults(store);

        let components = make_components();
        let checkpoint = manager
            .create_checkpoint(
                "project-1",
                "Test checkpoint",
                CheckpointTrigger::Manual {
                    user_id: "user1".to_string(),
                },
                components,
            )
            .await
            .expect("create should succeed");

        assert_eq!(checkpoint.project_id, "project-1");
        assert_eq!(checkpoint.description, "Test checkpoint");
    }

    #[tokio::test]
    async fn test_checkpoint_manager_list() {
        let store = InMemoryCheckpointStore::new();
        let manager = CheckpointManager::with_defaults(store);

        for i in 0..3 {
            let components = make_components();
            manager
                .create_checkpoint(
                    "project-1",
                    &format!("Checkpoint {i}"),
                    CheckpointTrigger::Scheduled,
                    components,
                )
                .await
                .expect("create should succeed");
        }

        let checkpoints = manager
            .list_checkpoints("project-1")
            .await
            .expect("list should succeed");
        assert_eq!(checkpoints.len(), 3);
    }

    #[tokio::test]
    async fn test_checkpoint_manager_enforces_limit() {
        let store = InMemoryCheckpointStore::new();
        let config = CheckpointConfig::default().with_max_checkpoints(3);
        let manager = CheckpointManager::new(store, config);

        // Create 5 checkpoints (limit is 3)
        for i in 0..5 {
            let components = make_components();
            manager
                .create_checkpoint(
                    "project-1",
                    &format!("Checkpoint {i}"),
                    CheckpointTrigger::Scheduled,
                    components,
                )
                .await
                .expect("create should succeed");
        }

        // Should only have 3 checkpoints (oldest 2 were rotated out)
        let checkpoints = manager
            .list_checkpoints("project-1")
            .await
            .expect("list should succeed");
        assert_eq!(checkpoints.len(), 3);
    }

    #[tokio::test]
    async fn test_checkpoint_manager_maybe_auto_checkpoint() {
        let store = InMemoryCheckpointStore::new();
        let manager = CheckpointManager::with_defaults(store);

        // Should create checkpoint for risky operation
        let components = make_components();
        let result = manager
            .maybe_auto_checkpoint("project-1", "rm -rf node_modules", components)
            .await
            .expect("auto checkpoint should succeed");
        assert!(result.is_some());

        // Should not create checkpoint for safe operation
        let components = make_components();
        let result = manager
            .maybe_auto_checkpoint("project-1", "cargo build", components)
            .await
            .expect("auto checkpoint should succeed");
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_checkpoint_manager_prepare_rollback() {
        let store = InMemoryCheckpointStore::new();
        let manager = CheckpointManager::with_defaults(store);

        // Create a checkpoint to rollback to
        let components = make_components();
        let checkpoint = manager
            .create_checkpoint(
                "project-1",
                "Target checkpoint",
                CheckpointTrigger::Scheduled,
                components,
            )
            .await
            .expect("create should succeed");

        // Prepare rollback without safety checkpoint
        let (target, safety) = manager
            .prepare_rollback("project-1", &checkpoint.id, false, None)
            .await
            .expect("prepare_rollback should succeed");

        assert_eq!(target.id, checkpoint.id);
        assert!(safety.is_none());

        // Prepare rollback with safety checkpoint
        let current_components = make_components();
        let (target, safety) = manager
            .prepare_rollback("project-1", &checkpoint.id, true, Some(current_components))
            .await
            .expect("prepare_rollback should succeed");

        assert_eq!(target.id, checkpoint.id);
        assert!(safety.is_some());
        let safety = safety.expect("should have safety checkpoint");
        assert!(matches!(
            safety.trigger,
            CheckpointTrigger::PreRollback { .. }
        ));
    }

    #[tokio::test]
    async fn test_checkpoint_manager_get_and_delete() {
        let store = InMemoryCheckpointStore::new();
        let manager = CheckpointManager::with_defaults(store);

        let components = make_components();
        let checkpoint = manager
            .create_checkpoint(
                "project-1",
                "Test checkpoint",
                CheckpointTrigger::Scheduled,
                components,
            )
            .await
            .expect("create should succeed");

        // Get checkpoint
        let loaded = manager
            .get_checkpoint(&checkpoint.id)
            .await
            .expect("get should succeed");
        assert_eq!(loaded.id, checkpoint.id);

        // Delete checkpoint
        manager
            .delete_checkpoint(&checkpoint.id)
            .await
            .expect("delete should succeed");

        // Should not be found
        let result = manager.get_checkpoint(&checkpoint.id).await;
        assert!(matches!(result, Err(CheckpointError::NotFound { .. })));
    }

    #[test]
    fn test_filter_secret_env_vars() {
        let mut env = HashMap::new();
        env.insert("PATH".to_string(), "/usr/bin".to_string());
        env.insert("API_KEY".to_string(), "secret".to_string());
        env.insert("DATABASE_PASSWORD".to_string(), "pass".to_string());
        env.insert("NORMAL".to_string(), "value".to_string());
        env.insert("my_token".to_string(), "tok".to_string());
        env.insert("AWS_SECRET_ACCESS_KEY".to_string(), "aws".to_string());

        let filtered = filter_secret_env_vars(env);

        assert!(filtered.contains_key("PATH"));
        assert!(filtered.contains_key("NORMAL"));
        assert!(!filtered.contains_key("API_KEY"));
        assert!(!filtered.contains_key("DATABASE_PASSWORD"));
        assert!(!filtered.contains_key("my_token"));
        assert!(!filtered.contains_key("AWS_SECRET_ACCESS_KEY"));
    }
}
