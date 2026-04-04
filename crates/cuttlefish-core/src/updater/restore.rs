//! Task restoration for resuming checkpointed tasks after server restart.
//!
//! This module provides:
//! - [`RestoreError`] for restoration failures
//! - [`RestoreConfig`] for configuring restoration behavior
//! - [`RestoredTask`] representing a task ready to be resumed
//! - [`RestoreResult`] containing all restored tasks and metadata
//! - [`TaskRestorer`] for detecting and restoring checkpointed tasks
//!
//! # Example
//!
//! ```no_run
//! use cuttlefish_core::updater::restore::{TaskRestorer, RestoreConfig};
//!
//! let restorer = TaskRestorer::with_defaults();
//!
//! if restorer.has_pending_restore() {
//!     let result = restorer.restore_tasks().expect("restore failed");
//!     println!("Restored {} tasks from version {}",
//!         result.restored_tasks.len(),
//!         result.previous_version);
//! }
//! ```

use std::path::PathBuf;

use thiserror::Error;
use tracing::{debug, info, warn};

use super::task_state::{ServerState, TaskCheckpointer, TaskState, TaskStateStatus};
use super::DEFAULT_CHECKPOINT_DIR;

/// Errors that can occur during task restoration.
#[derive(Error, Debug)]
pub enum RestoreError {
    /// No checkpoint was found to restore from.
    #[error("No checkpoint found at {path}")]
    CheckpointNotFound {
        /// Path where checkpoint was expected.
        path: PathBuf,
    },

    /// The checkpoint was created by a different version and version mismatch is not allowed.
    #[error(
        "Version mismatch: checkpoint from {checkpoint_version}, current is {current_version}"
    )]
    VersionMismatch {
        /// Version that created the checkpoint.
        checkpoint_version: String,
        /// Current running version.
        current_version: String,
    },

    /// The checkpoint data is corrupted or invalid.
    #[error("Corrupted checkpoint state: {reason}")]
    CorruptedState {
        /// Description of the corruption.
        reason: String,
    },

    /// I/O error during restoration.
    #[error("I/O error during restoration: {reason}")]
    IoError {
        /// Description of the I/O error.
        reason: String,
    },
}

impl From<super::task_state::TaskStateError> for RestoreError {
    fn from(err: super::task_state::TaskStateError) -> Self {
        match err {
            super::task_state::TaskStateError::Io { reason } => Self::IoError { reason },
            super::task_state::TaskStateError::Serialization { reason } => {
                Self::CorruptedState { reason }
            }
            super::task_state::TaskStateError::NotFound { task_id } => Self::CorruptedState {
                reason: format!("Referenced task not found: {task_id}"),
            },
            super::task_state::TaskStateError::DirectoryError { reason } => {
                Self::IoError { reason }
            }
        }
    }
}

/// Configuration for task restoration.
#[derive(Debug, Clone)]
pub struct RestoreConfig {
    /// Directory containing checkpoint files.
    pub checkpoint_dir: PathBuf,
    /// Whether to restore tasks from a different version.
    /// If false, restoration will fail with `VersionMismatch` when versions differ.
    pub allow_version_mismatch: bool,
    /// Whether to delete checkpoint files after successful restoration.
    pub cleanup_after_restore: bool,
}

impl Default for RestoreConfig {
    fn default() -> Self {
        Self {
            checkpoint_dir: PathBuf::from(DEFAULT_CHECKPOINT_DIR),
            allow_version_mismatch: false,
            cleanup_after_restore: true,
        }
    }
}

impl RestoreConfig {
    /// Create a new restore config with the given checkpoint directory.
    pub fn new(checkpoint_dir: impl Into<PathBuf>) -> Self {
        Self {
            checkpoint_dir: checkpoint_dir.into(),
            ..Default::default()
        }
    }

    /// Set whether to allow version mismatch.
    pub fn with_allow_version_mismatch(mut self, allow: bool) -> Self {
        self.allow_version_mismatch = allow;
        self
    }

    /// Set whether to cleanup after restore.
    pub fn with_cleanup_after_restore(mut self, cleanup: bool) -> Self {
        self.cleanup_after_restore = cleanup;
        self
    }
}

/// A task that has been restored from a checkpoint.
#[derive(Debug, Clone)]
pub struct RestoredTask {
    /// The restored task state.
    pub task_state: TaskState,
    /// Whether the task may need to replay its last action.
    /// This is true if the task was `Running` when checkpointed,
    /// as it may have been mid-operation.
    pub needs_replay: bool,
}

impl RestoredTask {
    /// Create a new restored task from a task state.
    ///
    /// The `needs_replay` flag is automatically set based on the task status:
    /// - `Running` -> needs_replay = true (was mid-operation)
    /// - `Pending` or `Paused` -> needs_replay = false (clean state)
    pub fn from_task_state(task_state: TaskState) -> Self {
        let needs_replay = task_state.status == TaskStateStatus::Running;
        Self {
            task_state,
            needs_replay,
        }
    }
}

/// Result of a task restoration operation.
#[derive(Debug, Clone)]
pub struct RestoreResult {
    /// Tasks that were successfully restored.
    pub restored_tasks: Vec<RestoredTask>,
    /// Version of the server that created the checkpoint.
    pub previous_version: String,
    /// Current version of the server.
    pub current_version: String,
    /// Tasks that were skipped during restoration.
    /// Each entry is (task_id, reason for skipping).
    pub skipped_tasks: Vec<(String, String)>,
}

impl RestoreResult {
    /// Returns true if any tasks were restored.
    pub fn has_restored_tasks(&self) -> bool {
        !self.restored_tasks.is_empty()
    }

    /// Returns the count of tasks that need replay.
    pub fn tasks_needing_replay(&self) -> usize {
        self.restored_tasks
            .iter()
            .filter(|t| t.needs_replay)
            .count()
    }

    /// Returns true if the versions match.
    pub fn versions_match(&self) -> bool {
        self.previous_version == self.current_version
    }
}

/// Restores checkpointed tasks after server restart.
///
/// The `TaskRestorer` reads checkpoint files created by [`TaskCheckpointer`]
/// and prepares them for resumption. It handles version checking and
/// optional cleanup of checkpoint files.
///
/// # Workflow
///
/// 1. On startup, check `has_pending_restore()` to see if there are tasks to restore
/// 2. Optionally peek at the server state with `get_server_state()`
/// 3. Call `restore_tasks()` to load and prepare all tasks
/// 4. Resume each `RestoredTask`, paying attention to `needs_replay` flag
/// 5. Checkpoint files are automatically cleaned up if `cleanup_after_restore` is true
pub struct TaskRestorer {
    config: RestoreConfig,
    checkpointer: TaskCheckpointer,
}

impl TaskRestorer {
    /// Create a new task restorer with the given configuration.
    pub fn new(config: RestoreConfig) -> Self {
        let checkpointer = TaskCheckpointer::new(&config.checkpoint_dir);
        Self {
            config,
            checkpointer,
        }
    }

    /// Create a task restorer with default configuration.
    pub fn with_defaults() -> Self {
        Self::new(RestoreConfig::default())
    }

    /// Check if there are pending tasks to restore.
    ///
    /// Returns true if a server state checkpoint file exists.
    pub fn has_pending_restore(&self) -> bool {
        self.checkpointer
            .load_server_state()
            .ok()
            .flatten()
            .is_some()
    }

    /// Get the count of pending tasks to restore.
    ///
    /// Returns 0 if no checkpoint exists or if loading fails.
    pub fn get_pending_task_count(&self) -> usize {
        self.checkpointer
            .load_server_state()
            .ok()
            .flatten()
            .map(|state| state.tasks.len())
            .unwrap_or(0)
    }

    /// Restore all checkpointed tasks.
    ///
    /// This method:
    /// 1. Loads the server state from checkpoint
    /// 2. Compares versions (fails if different and `allow_version_mismatch` is false)
    /// 3. Converts each task to a `RestoredTask` with appropriate `needs_replay` flag
    /// 4. Optionally cleans up checkpoint files
    ///
    /// # Errors
    ///
    /// - `CheckpointNotFound` if no server state checkpoint exists
    /// - `VersionMismatch` if versions differ and `allow_version_mismatch` is false
    /// - `CorruptedState` if checkpoint data is invalid
    /// - `IoError` for file system errors
    pub fn restore_tasks(&self) -> Result<RestoreResult, RestoreError> {
        let current_version = env!("CARGO_PKG_VERSION").to_string();

        // Load server state
        let server_state = self.checkpointer.load_server_state()?.ok_or_else(|| {
            RestoreError::CheckpointNotFound {
                path: self.config.checkpoint_dir.clone(),
            }
        })?;

        info!(
            previous_version = %server_state.version,
            current_version = %current_version,
            task_count = server_state.tasks.len(),
            "Restoring checkpointed tasks"
        );

        // Check version compatibility
        if server_state.version != current_version {
            if self.config.allow_version_mismatch {
                warn!(
                    previous = %server_state.version,
                    current = %current_version,
                    "Restoring tasks from different version (allowed by config)"
                );
            } else {
                return Err(RestoreError::VersionMismatch {
                    checkpoint_version: server_state.version,
                    current_version,
                });
            }
        }

        // Convert tasks to RestoredTask
        let mut restored_tasks = Vec::new();
        let mut skipped_tasks = Vec::new();

        for task_state in server_state.tasks {
            // Validate task has required fields
            if task_state.task_id.is_empty() {
                skipped_tasks.push(("<empty>".to_string(), "Task has empty task_id".to_string()));
                continue;
            }

            debug!(
                task_id = %task_state.task_id,
                status = %task_state.status,
                "Restoring task"
            );

            restored_tasks.push(RestoredTask::from_task_state(task_state));
        }

        let result = RestoreResult {
            restored_tasks,
            previous_version: server_state.version,
            current_version: current_version.clone(),
            skipped_tasks,
        };

        // Cleanup if configured
        if self.config.cleanup_after_restore {
            self.cleanup_checkpoints()?;
        }

        info!(
            restored = result.restored_tasks.len(),
            skipped = result.skipped_tasks.len(),
            needs_replay = result.tasks_needing_replay(),
            "Task restoration complete"
        );

        Ok(result)
    }

    /// Manually cleanup all checkpoint files.
    ///
    /// This removes the server state and all individual task checkpoints.
    pub fn cleanup_checkpoints(&self) -> Result<(), RestoreError> {
        self.checkpointer.clear_all()?;
        debug!(dir = ?self.config.checkpoint_dir, "Cleaned up checkpoint files");
        Ok(())
    }

    /// Peek at the server state without restoring.
    ///
    /// This allows inspecting the checkpoint without triggering restoration
    /// or cleanup. Returns `None` if no checkpoint exists.
    pub fn get_server_state(&self) -> Option<ServerState> {
        self.checkpointer.load_server_state().ok().flatten()
    }
}

/// Check if the `--resume-tasks` CLI flag was passed.
///
/// This is a helper function for startup hooks to determine if
/// task restoration should be attempted.
///
/// # Note
///
/// This function checks `std::env::args()` for the flag. In production,
/// you may want to use a proper CLI argument parser like `clap`.
pub fn should_resume_tasks() -> bool {
    std::env::args().any(|arg| arg == "--resume-tasks")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::updater::task_state::{TaskProgress, TaskState, TaskStateStatus};
    use serde_json::json;
    use tempfile::TempDir;

    fn make_task_state(task_id: &str, status: TaskStateStatus) -> TaskState {
        TaskState::new(task_id, "project-1", "coder", "Test task")
            .with_status(status)
            .with_context(json!({"key": "value"}))
            .with_progress(TaskProgress::new(1, 3, "Step 1"))
    }

    fn setup_checkpoint(
        temp_dir: &TempDir,
        version: &str,
        tasks: Vec<TaskState>,
    ) -> TaskCheckpointer {
        let checkpointer = TaskCheckpointer::new(temp_dir.path());
        let server_state = ServerState::new(version, tasks);
        checkpointer
            .save_server_state(&server_state)
            .expect("save server state");
        checkpointer
    }

    #[test]
    fn test_restore_error_display() {
        let err = RestoreError::CheckpointNotFound {
            path: PathBuf::from("/tmp/checkpoints"),
        };
        assert!(err.to_string().contains("/tmp/checkpoints"));

        let err = RestoreError::VersionMismatch {
            checkpoint_version: "1.0.0".to_string(),
            current_version: "2.0.0".to_string(),
        };
        assert!(err.to_string().contains("1.0.0"));
        assert!(err.to_string().contains("2.0.0"));

        let err = RestoreError::CorruptedState {
            reason: "invalid json".to_string(),
        };
        assert!(err.to_string().contains("invalid json"));

        let err = RestoreError::IoError {
            reason: "permission denied".to_string(),
        };
        assert!(err.to_string().contains("permission denied"));
    }

    #[test]
    fn test_restore_config_default() {
        let config = RestoreConfig::default();
        assert_eq!(config.checkpoint_dir, PathBuf::from(DEFAULT_CHECKPOINT_DIR));
        assert!(!config.allow_version_mismatch);
        assert!(config.cleanup_after_restore);
    }

    #[test]
    fn test_restore_config_new() {
        let config = RestoreConfig::new("/custom/path");
        assert_eq!(config.checkpoint_dir, PathBuf::from("/custom/path"));
    }

    #[test]
    fn test_restore_config_builders() {
        let config = RestoreConfig::default()
            .with_allow_version_mismatch(true)
            .with_cleanup_after_restore(false);

        assert!(config.allow_version_mismatch);
        assert!(!config.cleanup_after_restore);
    }

    #[test]
    fn test_restored_task_from_pending() {
        let task = make_task_state("task-1", TaskStateStatus::Pending);
        let restored = RestoredTask::from_task_state(task);

        assert!(!restored.needs_replay);
        assert_eq!(restored.task_state.task_id, "task-1");
    }

    #[test]
    fn test_restored_task_from_running() {
        let task = make_task_state("task-1", TaskStateStatus::Running);
        let restored = RestoredTask::from_task_state(task);

        assert!(restored.needs_replay);
        assert_eq!(restored.task_state.task_id, "task-1");
    }

    #[test]
    fn test_restored_task_from_paused() {
        let task = make_task_state("task-1", TaskStateStatus::Paused);
        let restored = RestoredTask::from_task_state(task);

        assert!(!restored.needs_replay);
    }

    #[test]
    fn test_restore_result_has_restored_tasks() {
        let result = RestoreResult {
            restored_tasks: vec![],
            previous_version: "1.0.0".to_string(),
            current_version: "1.0.0".to_string(),
            skipped_tasks: vec![],
        };
        assert!(!result.has_restored_tasks());

        let task = make_task_state("task-1", TaskStateStatus::Pending);
        let result = RestoreResult {
            restored_tasks: vec![RestoredTask::from_task_state(task)],
            previous_version: "1.0.0".to_string(),
            current_version: "1.0.0".to_string(),
            skipped_tasks: vec![],
        };
        assert!(result.has_restored_tasks());
    }

    #[test]
    fn test_restore_result_tasks_needing_replay() {
        let pending = make_task_state("task-1", TaskStateStatus::Pending);
        let running = make_task_state("task-2", TaskStateStatus::Running);
        let paused = make_task_state("task-3", TaskStateStatus::Paused);

        let result = RestoreResult {
            restored_tasks: vec![
                RestoredTask::from_task_state(pending),
                RestoredTask::from_task_state(running),
                RestoredTask::from_task_state(paused),
            ],
            previous_version: "1.0.0".to_string(),
            current_version: "1.0.0".to_string(),
            skipped_tasks: vec![],
        };

        assert_eq!(result.tasks_needing_replay(), 1);
    }

    #[test]
    fn test_restore_result_versions_match() {
        let result = RestoreResult {
            restored_tasks: vec![],
            previous_version: "1.0.0".to_string(),
            current_version: "1.0.0".to_string(),
            skipped_tasks: vec![],
        };
        assert!(result.versions_match());

        let result = RestoreResult {
            restored_tasks: vec![],
            previous_version: "1.0.0".to_string(),
            current_version: "2.0.0".to_string(),
            skipped_tasks: vec![],
        };
        assert!(!result.versions_match());
    }

    #[test]
    fn test_task_restorer_new() {
        let config = RestoreConfig::new("/tmp/checkpoints");
        let restorer = TaskRestorer::new(config);
        assert_eq!(
            restorer.config.checkpoint_dir,
            PathBuf::from("/tmp/checkpoints")
        );
    }

    #[test]
    fn test_task_restorer_with_defaults() {
        let restorer = TaskRestorer::with_defaults();
        assert_eq!(
            restorer.config.checkpoint_dir,
            PathBuf::from(DEFAULT_CHECKPOINT_DIR)
        );
    }

    #[test]
    fn test_task_restorer_has_pending_restore_false() {
        let temp_dir = TempDir::new().expect("temp dir");
        let config = RestoreConfig::new(temp_dir.path());
        let restorer = TaskRestorer::new(config);

        assert!(!restorer.has_pending_restore());
    }

    #[test]
    fn test_task_restorer_has_pending_restore_true() {
        let temp_dir = TempDir::new().expect("temp dir");
        let tasks = vec![make_task_state("task-1", TaskStateStatus::Running)];
        setup_checkpoint(&temp_dir, env!("CARGO_PKG_VERSION"), tasks);

        let config = RestoreConfig::new(temp_dir.path());
        let restorer = TaskRestorer::new(config);

        assert!(restorer.has_pending_restore());
    }

    #[test]
    fn test_task_restorer_get_pending_task_count() {
        let temp_dir = TempDir::new().expect("temp dir");
        let config = RestoreConfig::new(temp_dir.path());
        let restorer = TaskRestorer::new(config);

        // No checkpoint
        assert_eq!(restorer.get_pending_task_count(), 0);

        // With checkpoint
        let tasks = vec![
            make_task_state("task-1", TaskStateStatus::Running),
            make_task_state("task-2", TaskStateStatus::Pending),
        ];
        setup_checkpoint(&temp_dir, env!("CARGO_PKG_VERSION"), tasks);

        let config = RestoreConfig::new(temp_dir.path());
        let restorer = TaskRestorer::new(config);
        assert_eq!(restorer.get_pending_task_count(), 2);
    }

    #[test]
    fn test_task_restorer_restore_tasks_no_checkpoint() {
        let temp_dir = TempDir::new().expect("temp dir");
        let config = RestoreConfig::new(temp_dir.path());
        let restorer = TaskRestorer::new(config);

        let result = restorer.restore_tasks();
        assert!(matches!(
            result,
            Err(RestoreError::CheckpointNotFound { .. })
        ));
    }

    #[test]
    fn test_task_restorer_restore_tasks_version_mismatch() {
        let temp_dir = TempDir::new().expect("temp dir");
        let tasks = vec![make_task_state("task-1", TaskStateStatus::Running)];
        setup_checkpoint(&temp_dir, "0.0.0-different", tasks);

        let config = RestoreConfig::new(temp_dir.path()).with_cleanup_after_restore(false);
        let restorer = TaskRestorer::new(config);

        let result = restorer.restore_tasks();
        assert!(matches!(result, Err(RestoreError::VersionMismatch { .. })));
    }

    #[test]
    fn test_task_restorer_restore_tasks_version_mismatch_allowed() {
        let temp_dir = TempDir::new().expect("temp dir");
        let tasks = vec![make_task_state("task-1", TaskStateStatus::Running)];
        setup_checkpoint(&temp_dir, "0.0.0-different", tasks);

        let config = RestoreConfig::new(temp_dir.path())
            .with_allow_version_mismatch(true)
            .with_cleanup_after_restore(false);
        let restorer = TaskRestorer::new(config);

        let result = restorer.restore_tasks().expect("should succeed");
        assert_eq!(result.restored_tasks.len(), 1);
        assert_eq!(result.previous_version, "0.0.0-different");
        assert!(!result.versions_match());
    }

    #[test]
    fn test_task_restorer_restore_tasks_success() {
        let temp_dir = TempDir::new().expect("temp dir");
        let tasks = vec![
            make_task_state("task-1", TaskStateStatus::Pending),
            make_task_state("task-2", TaskStateStatus::Running),
            make_task_state("task-3", TaskStateStatus::Paused),
        ];
        setup_checkpoint(&temp_dir, env!("CARGO_PKG_VERSION"), tasks);

        let config = RestoreConfig::new(temp_dir.path()).with_cleanup_after_restore(false);
        let restorer = TaskRestorer::new(config);

        let result = restorer.restore_tasks().expect("should succeed");

        assert_eq!(result.restored_tasks.len(), 3);
        assert!(result.versions_match());
        assert_eq!(result.tasks_needing_replay(), 1); // Only the Running task

        // Verify task states
        let task_ids: Vec<_> = result
            .restored_tasks
            .iter()
            .map(|t| t.task_state.task_id.as_str())
            .collect();
        assert!(task_ids.contains(&"task-1"));
        assert!(task_ids.contains(&"task-2"));
        assert!(task_ids.contains(&"task-3"));
    }

    #[test]
    fn test_task_restorer_restore_tasks_with_cleanup() {
        let temp_dir = TempDir::new().expect("temp dir");
        let tasks = vec![make_task_state("task-1", TaskStateStatus::Running)];
        setup_checkpoint(&temp_dir, env!("CARGO_PKG_VERSION"), tasks);

        let config = RestoreConfig::new(temp_dir.path()).with_cleanup_after_restore(true);
        let restorer = TaskRestorer::new(config);

        // Verify checkpoint exists
        assert!(restorer.has_pending_restore());

        // Restore (should cleanup)
        let result = restorer.restore_tasks().expect("should succeed");
        assert_eq!(result.restored_tasks.len(), 1);

        // Verify checkpoint is gone
        assert!(!restorer.has_pending_restore());
    }

    #[test]
    fn test_task_restorer_cleanup_checkpoints() {
        let temp_dir = TempDir::new().expect("temp dir");
        let tasks = vec![make_task_state("task-1", TaskStateStatus::Running)];
        setup_checkpoint(&temp_dir, env!("CARGO_PKG_VERSION"), tasks);

        let config = RestoreConfig::new(temp_dir.path()).with_cleanup_after_restore(false);
        let restorer = TaskRestorer::new(config);

        // Verify checkpoint exists
        assert!(restorer.has_pending_restore());

        // Manual cleanup
        restorer.cleanup_checkpoints().expect("cleanup");

        // Verify checkpoint is gone
        assert!(!restorer.has_pending_restore());
    }

    #[test]
    fn test_task_restorer_get_server_state() {
        let temp_dir = TempDir::new().expect("temp dir");
        let config = RestoreConfig::new(temp_dir.path());
        let restorer = TaskRestorer::new(config);

        // No checkpoint
        assert!(restorer.get_server_state().is_none());

        // With checkpoint
        let tasks = vec![make_task_state("task-1", TaskStateStatus::Running)];
        setup_checkpoint(&temp_dir, "1.2.3", tasks);

        let config = RestoreConfig::new(temp_dir.path());
        let restorer = TaskRestorer::new(config);

        let state = restorer.get_server_state().expect("should exist");
        assert_eq!(state.version, "1.2.3");
        assert_eq!(state.tasks.len(), 1);
    }

    #[test]
    fn test_task_restorer_skips_empty_task_id() {
        let temp_dir = TempDir::new().expect("temp dir");

        // Create a task with empty task_id
        let mut bad_task = make_task_state("", TaskStateStatus::Running);
        bad_task.task_id = String::new();

        let tasks = vec![
            make_task_state("task-1", TaskStateStatus::Running),
            bad_task,
        ];
        setup_checkpoint(&temp_dir, env!("CARGO_PKG_VERSION"), tasks);

        let config = RestoreConfig::new(temp_dir.path()).with_cleanup_after_restore(false);
        let restorer = TaskRestorer::new(config);

        let result = restorer.restore_tasks().expect("should succeed");

        assert_eq!(result.restored_tasks.len(), 1);
        assert_eq!(result.skipped_tasks.len(), 1);
        assert_eq!(result.skipped_tasks[0].0, "<empty>");
    }

    #[test]
    fn test_should_resume_tasks_false() {
        // In test environment, --resume-tasks is not passed
        // This test just verifies the function doesn't panic
        let _ = should_resume_tasks();
    }

    #[test]
    fn test_restore_error_from_task_state_error() {
        use super::super::task_state::TaskStateError;

        let err: RestoreError = TaskStateError::Io {
            reason: "test".to_string(),
        }
        .into();
        assert!(matches!(err, RestoreError::IoError { .. }));

        let err: RestoreError = TaskStateError::Serialization {
            reason: "test".to_string(),
        }
        .into();
        assert!(matches!(err, RestoreError::CorruptedState { .. }));

        let err: RestoreError = TaskStateError::NotFound {
            task_id: "task-1".to_string(),
        }
        .into();
        assert!(matches!(err, RestoreError::CorruptedState { .. }));

        let err: RestoreError = TaskStateError::DirectoryError {
            reason: "test".to_string(),
        }
        .into();
        assert!(matches!(err, RestoreError::IoError { .. }));
    }
}
