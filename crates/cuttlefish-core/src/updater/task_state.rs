//! Task state checkpointing for restart preservation.
//!
//! This module provides:
//! - [`TaskState`] struct capturing running task state
//! - [`TaskProgress`] struct for tracking task progress
//! - [`TaskStateStatus`] enum for task status
//! - [`TaskCheckpointer`] for saving/loading task state to disk
//! - [`ServerState`] for full server state checkpoint
//!
//! Task checkpoints enable the server to resume running tasks after restart.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;
use tracing::{debug, info, warn};

/// Default directory for task checkpoints relative to project root.
pub const DEFAULT_CHECKPOINT_DIR: &str = ".cuttlefish/task_checkpoints";

/// Errors that can occur during task state operations.
#[derive(Error, Debug)]
pub enum TaskStateError {
    /// Failed to read or write checkpoint file.
    #[error("I/O error: {reason}")]
    Io {
        /// Reason for the failure.
        reason: String,
    },

    /// Failed to serialize or deserialize task state.
    #[error("Serialization error: {reason}")]
    Serialization {
        /// Reason for the failure.
        reason: String,
    },

    /// Task checkpoint was not found.
    #[error("Task checkpoint not found: {task_id}")]
    NotFound {
        /// The task ID that was not found.
        task_id: String,
    },

    /// Checkpoint directory does not exist and could not be created.
    #[error("Checkpoint directory error: {reason}")]
    DirectoryError {
        /// Reason for the failure.
        reason: String,
    },
}

impl From<std::io::Error> for TaskStateError {
    fn from(err: std::io::Error) -> Self {
        Self::Io {
            reason: err.to_string(),
        }
    }
}

impl From<serde_json::Error> for TaskStateError {
    fn from(err: serde_json::Error) -> Self {
        Self::Serialization {
            reason: err.to_string(),
        }
    }
}

/// Result type for task state operations.
pub type TaskStateResult<T> = Result<T, TaskStateError>;

/// Status of a checkpointed task.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum TaskStateStatus {
    /// Task is waiting to be executed.
    #[default]
    Pending,
    /// Task is currently running.
    Running,
    /// Task is paused and can be resumed.
    Paused,
}

impl std::fmt::Display for TaskStateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "pending"),
            Self::Running => write!(f, "running"),
            Self::Paused => write!(f, "paused"),
        }
    }
}

/// Progress information for a task.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TaskProgress {
    /// Number of steps completed.
    pub steps_completed: u32,
    /// Total number of steps (0 if unknown).
    pub total_steps: u32,
    /// Description of the current step being executed.
    pub current_step_description: String,
}

impl TaskProgress {
    /// Create a new task progress with the given values.
    pub fn new(steps_completed: u32, total_steps: u32, current_step: impl Into<String>) -> Self {
        Self {
            steps_completed,
            total_steps,
            current_step_description: current_step.into(),
        }
    }

    /// Create progress for a task that just started.
    pub fn started(description: impl Into<String>) -> Self {
        Self {
            steps_completed: 0,
            total_steps: 0,
            current_step_description: description.into(),
        }
    }

    /// Calculate completion percentage (0-100).
    /// Returns 0 if total_steps is 0.
    pub fn percentage(&self) -> u32 {
        if self.total_steps == 0 {
            0
        } else {
            (self.steps_completed * 100) / self.total_steps
        }
    }
}

/// State of a running task that can be checkpointed for restart recovery.
///
/// This struct captures all information needed to resume a task after
/// a server restart. The `context` field stores agent-specific data
/// like conversation history summary, current file being edited, etc.
///
/// # Security Note
/// Do NOT store sensitive data (API keys, tokens, passwords) in the context field.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskState {
    /// Unique task identifier.
    pub task_id: String,
    /// Project this task belongs to.
    pub project_id: String,
    /// Role of the agent executing this task (e.g., "coder", "critic").
    pub agent_role: String,
    /// Current status of the task.
    pub status: TaskStateStatus,
    /// Human-readable description of the task.
    pub description: String,
    /// Agent-specific context data (conversation summary, file state, etc.).
    /// Must be JSON-serializable. Do NOT store secrets here.
    pub context: serde_json::Value,
    /// Progress information for the task.
    pub progress: TaskProgress,
    /// When the task was originally created.
    pub created_at: DateTime<Utc>,
    /// When this checkpoint was last saved.
    pub checkpointed_at: DateTime<Utc>,
}

impl TaskState {
    /// Create a new task state.
    pub fn new(
        task_id: impl Into<String>,
        project_id: impl Into<String>,
        agent_role: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            task_id: task_id.into(),
            project_id: project_id.into(),
            agent_role: agent_role.into(),
            status: TaskStateStatus::Pending,
            description: description.into(),
            context: serde_json::Value::Null,
            progress: TaskProgress::default(),
            created_at: now,
            checkpointed_at: now,
        }
    }

    /// Set the task status.
    pub fn with_status(mut self, status: TaskStateStatus) -> Self {
        self.status = status;
        self
    }

    /// Set the task context.
    pub fn with_context(mut self, context: serde_json::Value) -> Self {
        self.context = context;
        self
    }

    /// Set the task progress.
    pub fn with_progress(mut self, progress: TaskProgress) -> Self {
        self.progress = progress;
        self
    }

    /// Update the checkpointed_at timestamp to now.
    pub fn touch(&mut self) {
        self.checkpointed_at = Utc::now();
    }

    /// Get the age of this task since creation.
    pub fn age(&self) -> chrono::Duration {
        Utc::now().signed_duration_since(self.created_at)
    }

    /// Get the time since last checkpoint.
    pub fn time_since_checkpoint(&self) -> chrono::Duration {
        Utc::now().signed_duration_since(self.checkpointed_at)
    }
}

/// Full server state checkpoint containing all running tasks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerState {
    /// Version of the server that created this checkpoint.
    pub version: String,
    /// All checkpointed tasks.
    pub tasks: Vec<TaskState>,
    /// When this server state was checkpointed.
    pub checkpointed_at: DateTime<Utc>,
}

impl ServerState {
    /// Create a new server state checkpoint.
    pub fn new(version: impl Into<String>, tasks: Vec<TaskState>) -> Self {
        Self {
            version: version.into(),
            tasks,
            checkpointed_at: Utc::now(),
        }
    }

    /// Create an empty server state with the given version.
    pub fn empty(version: impl Into<String>) -> Self {
        Self::new(version, Vec::new())
    }
}

/// Manages task state checkpoints on disk.
///
/// Checkpoints are stored as JSON files in the checkpoint directory,
/// with one file per task named `{task_id}.json`.
pub struct TaskCheckpointer {
    /// Directory where checkpoint files are stored.
    checkpoint_dir: PathBuf,
}

impl TaskCheckpointer {
    /// Create a new checkpointer with the given checkpoint directory.
    ///
    /// The directory will be created if it doesn't exist when saving.
    pub fn new(checkpoint_dir: impl Into<PathBuf>) -> Self {
        Self {
            checkpoint_dir: checkpoint_dir.into(),
        }
    }

    /// Create a checkpointer with the default checkpoint directory.
    pub fn with_default_dir() -> Self {
        Self::new(DEFAULT_CHECKPOINT_DIR)
    }

    /// Get the checkpoint directory path.
    pub fn checkpoint_dir(&self) -> &Path {
        &self.checkpoint_dir
    }

    /// Ensure the checkpoint directory exists.
    fn ensure_dir(&self) -> TaskStateResult<()> {
        if !self.checkpoint_dir.exists() {
            fs::create_dir_all(&self.checkpoint_dir).map_err(|e| {
                TaskStateError::DirectoryError {
                    reason: format!("Failed to create checkpoint directory: {e}"),
                }
            })?;
            debug!(dir = ?self.checkpoint_dir, "Created checkpoint directory");
        }
        Ok(())
    }

    /// Get the path for a task's checkpoint file.
    fn task_path(&self, task_id: &str) -> PathBuf {
        self.checkpoint_dir.join(format!("{task_id}.json"))
    }

    /// Get the path for the server state checkpoint file.
    fn server_state_path(&self) -> PathBuf {
        self.checkpoint_dir.join("_server_state.json")
    }

    /// Save a task state to disk.
    ///
    /// Creates the checkpoint directory if it doesn't exist.
    pub fn save_task_state(&self, state: &TaskState) -> TaskStateResult<()> {
        self.ensure_dir()?;

        let path = self.task_path(&state.task_id);
        let json = serde_json::to_string_pretty(state)?;
        fs::write(&path, json)?;

        debug!(
            task_id = %state.task_id,
            path = ?path,
            "Saved task checkpoint"
        );
        Ok(())
    }

    /// Load a task state from disk.
    ///
    /// Returns `None` if the checkpoint file doesn't exist.
    pub fn load_task_state(&self, task_id: &str) -> TaskStateResult<Option<TaskState>> {
        let path = self.task_path(task_id);

        if !path.exists() {
            return Ok(None);
        }

        let json = fs::read_to_string(&path)?;
        let state: TaskState = serde_json::from_str(&json)?;

        debug!(
            task_id = %task_id,
            path = ?path,
            "Loaded task checkpoint"
        );
        Ok(Some(state))
    }

    /// Load all pending/running/paused tasks from the checkpoint directory.
    ///
    /// This is useful for resuming tasks after a server restart.
    pub fn load_all_pending_tasks(&self) -> TaskStateResult<Vec<TaskState>> {
        if !self.checkpoint_dir.exists() {
            return Ok(Vec::new());
        }

        let mut tasks = Vec::new();

        for entry in fs::read_dir(&self.checkpoint_dir)? {
            let entry = entry?;
            let path = entry.path();

            // Skip non-JSON files and the server state file
            if path.extension().is_some_and(|ext| ext == "json")
                && path
                    .file_name()
                    .is_some_and(|name| name != "_server_state.json")
            {
                match fs::read_to_string(&path) {
                    Ok(json) => match serde_json::from_str::<TaskState>(&json) {
                        Ok(state) => {
                            tasks.push(state);
                        }
                        Err(e) => {
                            warn!(
                                path = ?path,
                                error = %e,
                                "Failed to parse task checkpoint, skipping"
                            );
                        }
                    },
                    Err(e) => {
                        warn!(
                            path = ?path,
                            error = %e,
                            "Failed to read task checkpoint, skipping"
                        );
                    }
                }
            }
        }

        // Sort by created_at (oldest first) for consistent ordering
        tasks.sort_by(|a, b| a.created_at.cmp(&b.created_at));

        info!(
            count = tasks.len(),
            dir = ?self.checkpoint_dir,
            "Loaded pending task checkpoints"
        );
        Ok(tasks)
    }

    /// Remove a task checkpoint from disk.
    ///
    /// This should be called when a task completes or is cancelled.
    /// Does nothing if the checkpoint doesn't exist.
    pub fn remove_task_checkpoint(&self, task_id: &str) -> TaskStateResult<()> {
        let path = self.task_path(task_id);

        if path.exists() {
            fs::remove_file(&path)?;
            debug!(
                task_id = %task_id,
                path = ?path,
                "Removed task checkpoint"
            );
        }

        Ok(())
    }

    /// List all checkpointed task IDs.
    pub fn list_checkpointed_tasks(&self) -> TaskStateResult<Vec<String>> {
        if !self.checkpoint_dir.exists() {
            return Ok(Vec::new());
        }

        let mut task_ids = Vec::new();

        for entry in fs::read_dir(&self.checkpoint_dir)? {
            let entry = entry?;
            let path = entry.path();

            if let Some(stem) = path.file_stem()
                && let Some(ext) = path.extension()
                && ext == "json"
            {
                let name = stem.to_string_lossy();
                if name != "_server_state" {
                    task_ids.push(name.into_owned());
                }
            }
        }

        task_ids.sort();
        Ok(task_ids)
    }

    /// Save the full server state to disk.
    pub fn save_server_state(&self, state: &ServerState) -> TaskStateResult<()> {
        self.ensure_dir()?;

        let path = self.server_state_path();
        let json = serde_json::to_string_pretty(state)?;
        fs::write(&path, json)?;

        info!(
            version = %state.version,
            task_count = state.tasks.len(),
            path = ?path,
            "Saved server state checkpoint"
        );
        Ok(())
    }

    /// Load the server state from disk.
    ///
    /// Returns `None` if no server state checkpoint exists.
    pub fn load_server_state(&self) -> TaskStateResult<Option<ServerState>> {
        let path = self.server_state_path();

        if !path.exists() {
            return Ok(None);
        }

        let json = fs::read_to_string(&path)?;
        let state: ServerState = serde_json::from_str(&json)?;

        info!(
            version = %state.version,
            task_count = state.tasks.len(),
            path = ?path,
            "Loaded server state checkpoint"
        );
        Ok(Some(state))
    }

    /// Remove the server state checkpoint from disk.
    pub fn remove_server_state(&self) -> TaskStateResult<()> {
        let path = self.server_state_path();

        if path.exists() {
            fs::remove_file(&path)?;
            debug!(path = ?path, "Removed server state checkpoint");
        }

        Ok(())
    }

    /// Clear all checkpoints (tasks and server state).
    pub fn clear_all(&self) -> TaskStateResult<()> {
        if !self.checkpoint_dir.exists() {
            return Ok(());
        }

        for entry in fs::read_dir(&self.checkpoint_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                fs::remove_file(&path)?;
            }
        }

        info!(dir = ?self.checkpoint_dir, "Cleared all checkpoints");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tempfile::TempDir;

    fn make_task_state(task_id: &str) -> TaskState {
        TaskState::new(task_id, "project-1", "coder", "Test task description")
    }

    #[test]
    fn test_task_state_status_default() {
        let status = TaskStateStatus::default();
        assert_eq!(status, TaskStateStatus::Pending);
    }

    #[test]
    fn test_task_state_status_display() {
        assert_eq!(TaskStateStatus::Pending.to_string(), "pending");
        assert_eq!(TaskStateStatus::Running.to_string(), "running");
        assert_eq!(TaskStateStatus::Paused.to_string(), "paused");
    }

    #[test]
    fn test_task_progress_new() {
        let progress = TaskProgress::new(5, 10, "Step 5 of 10");
        assert_eq!(progress.steps_completed, 5);
        assert_eq!(progress.total_steps, 10);
        assert_eq!(progress.current_step_description, "Step 5 of 10");
    }

    #[test]
    fn test_task_progress_started() {
        let progress = TaskProgress::started("Starting task");
        assert_eq!(progress.steps_completed, 0);
        assert_eq!(progress.total_steps, 0);
        assert_eq!(progress.current_step_description, "Starting task");
    }

    #[test]
    fn test_task_progress_percentage() {
        let progress = TaskProgress::new(5, 10, "");
        assert_eq!(progress.percentage(), 50);

        let progress = TaskProgress::new(3, 4, "");
        assert_eq!(progress.percentage(), 75);

        let progress = TaskProgress::new(0, 0, "");
        assert_eq!(progress.percentage(), 0);

        let progress = TaskProgress::new(10, 10, "");
        assert_eq!(progress.percentage(), 100);
    }

    #[test]
    fn test_task_state_new() {
        let state = TaskState::new("task-1", "project-1", "coder", "Build feature X");
        assert_eq!(state.task_id, "task-1");
        assert_eq!(state.project_id, "project-1");
        assert_eq!(state.agent_role, "coder");
        assert_eq!(state.description, "Build feature X");
        assert_eq!(state.status, TaskStateStatus::Pending);
        assert!(state.context.is_null());
    }

    #[test]
    fn test_task_state_builders() {
        let context = json!({"file": "main.rs", "line": 42});
        let progress = TaskProgress::new(2, 5, "Editing file");

        let state = TaskState::new("task-1", "project-1", "coder", "Test")
            .with_status(TaskStateStatus::Running)
            .with_context(context.clone())
            .with_progress(progress);

        assert_eq!(state.status, TaskStateStatus::Running);
        assert_eq!(state.context, context);
        assert_eq!(state.progress.steps_completed, 2);
        assert_eq!(state.progress.total_steps, 5);
    }

    #[test]
    fn test_task_state_touch() {
        let mut state = make_task_state("task-1");
        let original = state.checkpointed_at;

        // Small delay to ensure timestamp changes
        std::thread::sleep(std::time::Duration::from_millis(10));
        state.touch();

        assert!(state.checkpointed_at > original);
    }

    #[test]
    fn test_task_state_serialization() {
        let state = TaskState::new("task-1", "project-1", "coder", "Test task")
            .with_status(TaskStateStatus::Running)
            .with_context(json!({"key": "value"}))
            .with_progress(TaskProgress::new(1, 3, "Step 1"));

        let json = serde_json::to_string(&state).expect("serialize");
        let deserialized: TaskState = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(deserialized.task_id, state.task_id);
        assert_eq!(deserialized.project_id, state.project_id);
        assert_eq!(deserialized.agent_role, state.agent_role);
        assert_eq!(deserialized.status, state.status);
        assert_eq!(deserialized.description, state.description);
        assert_eq!(deserialized.context, state.context);
        assert_eq!(
            deserialized.progress.steps_completed,
            state.progress.steps_completed
        );
    }

    #[test]
    fn test_server_state_new() {
        let tasks = vec![make_task_state("task-1"), make_task_state("task-2")];
        let state = ServerState::new("1.0.0", tasks);

        assert_eq!(state.version, "1.0.0");
        assert_eq!(state.tasks.len(), 2);
    }

    #[test]
    fn test_server_state_empty() {
        let state = ServerState::empty("1.0.0");
        assert_eq!(state.version, "1.0.0");
        assert!(state.tasks.is_empty());
    }

    #[test]
    fn test_server_state_serialization() {
        let tasks = vec![make_task_state("task-1")];
        let state = ServerState::new("1.0.0", tasks);

        let json = serde_json::to_string(&state).expect("serialize");
        let deserialized: ServerState = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(deserialized.version, state.version);
        assert_eq!(deserialized.tasks.len(), 1);
    }

    #[test]
    fn test_checkpointer_new() {
        let checkpointer = TaskCheckpointer::new("/tmp/checkpoints");
        assert_eq!(checkpointer.checkpoint_dir(), Path::new("/tmp/checkpoints"));
    }

    #[test]
    fn test_checkpointer_with_default_dir() {
        let checkpointer = TaskCheckpointer::with_default_dir();
        assert_eq!(
            checkpointer.checkpoint_dir(),
            Path::new(DEFAULT_CHECKPOINT_DIR)
        );
    }

    #[test]
    fn test_checkpointer_save_and_load_task() {
        let temp_dir = TempDir::new().expect("temp dir");
        let checkpointer = TaskCheckpointer::new(temp_dir.path());

        let state = TaskState::new("task-123", "project-1", "coder", "Test task")
            .with_status(TaskStateStatus::Running)
            .with_context(json!({"file": "main.rs"}));

        checkpointer.save_task_state(&state).expect("save");

        let loaded = checkpointer
            .load_task_state("task-123")
            .expect("load")
            .expect("should exist");

        assert_eq!(loaded.task_id, "task-123");
        assert_eq!(loaded.project_id, "project-1");
        assert_eq!(loaded.agent_role, "coder");
        assert_eq!(loaded.status, TaskStateStatus::Running);
        assert_eq!(loaded.context["file"], "main.rs");
    }

    #[test]
    fn test_checkpointer_load_nonexistent() {
        let temp_dir = TempDir::new().expect("temp dir");
        let checkpointer = TaskCheckpointer::new(temp_dir.path());

        let result = checkpointer
            .load_task_state("nonexistent")
            .expect("should not error");
        assert!(result.is_none());
    }

    #[test]
    fn test_checkpointer_remove_task() {
        let temp_dir = TempDir::new().expect("temp dir");
        let checkpointer = TaskCheckpointer::new(temp_dir.path());

        let state = make_task_state("task-to-remove");
        checkpointer.save_task_state(&state).expect("save");

        // Verify it exists
        assert!(checkpointer
            .load_task_state("task-to-remove")
            .expect("load")
            .is_some());

        // Remove it
        checkpointer
            .remove_task_checkpoint("task-to-remove")
            .expect("remove");

        // Verify it's gone
        assert!(checkpointer
            .load_task_state("task-to-remove")
            .expect("load")
            .is_none());
    }

    #[test]
    fn test_checkpointer_remove_nonexistent() {
        let temp_dir = TempDir::new().expect("temp dir");
        let checkpointer = TaskCheckpointer::new(temp_dir.path());

        // Should not error when removing nonexistent checkpoint
        checkpointer
            .remove_task_checkpoint("nonexistent")
            .expect("remove should succeed");
    }

    #[test]
    fn test_checkpointer_list_tasks() {
        let temp_dir = TempDir::new().expect("temp dir");
        let checkpointer = TaskCheckpointer::new(temp_dir.path());

        // Save multiple tasks
        for i in 1..=3 {
            let state = make_task_state(&format!("task-{i}"));
            checkpointer.save_task_state(&state).expect("save");
        }

        let task_ids = checkpointer.list_checkpointed_tasks().expect("list");
        assert_eq!(task_ids.len(), 3);
        assert!(task_ids.contains(&"task-1".to_string()));
        assert!(task_ids.contains(&"task-2".to_string()));
        assert!(task_ids.contains(&"task-3".to_string()));
    }

    #[test]
    fn test_checkpointer_list_empty() {
        let temp_dir = TempDir::new().expect("temp dir");
        let checkpointer = TaskCheckpointer::new(temp_dir.path());

        let task_ids = checkpointer.list_checkpointed_tasks().expect("list");
        assert!(task_ids.is_empty());
    }

    #[test]
    fn test_checkpointer_list_nonexistent_dir() {
        let checkpointer = TaskCheckpointer::new("/nonexistent/path/checkpoints");

        let task_ids = checkpointer.list_checkpointed_tasks().expect("list");
        assert!(task_ids.is_empty());
    }

    #[test]
    fn test_checkpointer_load_all_pending() {
        let temp_dir = TempDir::new().expect("temp dir");
        let checkpointer = TaskCheckpointer::new(temp_dir.path());

        // Save tasks with different statuses
        let pending = make_task_state("task-pending").with_status(TaskStateStatus::Pending);
        let running = make_task_state("task-running").with_status(TaskStateStatus::Running);
        let paused = make_task_state("task-paused").with_status(TaskStateStatus::Paused);

        checkpointer.save_task_state(&pending).expect("save");
        checkpointer.save_task_state(&running).expect("save");
        checkpointer.save_task_state(&paused).expect("save");

        let tasks = checkpointer.load_all_pending_tasks().expect("load all");
        assert_eq!(tasks.len(), 3);
    }

    #[test]
    fn test_checkpointer_load_all_empty() {
        let temp_dir = TempDir::new().expect("temp dir");
        let checkpointer = TaskCheckpointer::new(temp_dir.path());

        let tasks = checkpointer.load_all_pending_tasks().expect("load all");
        assert!(tasks.is_empty());
    }

    #[test]
    fn test_checkpointer_load_all_nonexistent_dir() {
        let checkpointer = TaskCheckpointer::new("/nonexistent/path/checkpoints");

        let tasks = checkpointer.load_all_pending_tasks().expect("load all");
        assert!(tasks.is_empty());
    }

    #[test]
    fn test_checkpointer_save_and_load_server_state() {
        let temp_dir = TempDir::new().expect("temp dir");
        let checkpointer = TaskCheckpointer::new(temp_dir.path());

        let tasks = vec![
            make_task_state("task-1").with_status(TaskStateStatus::Running),
            make_task_state("task-2").with_status(TaskStateStatus::Paused),
        ];
        let state = ServerState::new("1.2.3", tasks);

        checkpointer.save_server_state(&state).expect("save");

        let loaded = checkpointer
            .load_server_state()
            .expect("load")
            .expect("should exist");

        assert_eq!(loaded.version, "1.2.3");
        assert_eq!(loaded.tasks.len(), 2);
        assert_eq!(loaded.tasks[0].task_id, "task-1");
        assert_eq!(loaded.tasks[1].task_id, "task-2");
    }

    #[test]
    fn test_checkpointer_load_server_state_nonexistent() {
        let temp_dir = TempDir::new().expect("temp dir");
        let checkpointer = TaskCheckpointer::new(temp_dir.path());

        let result = checkpointer.load_server_state().expect("should not error");
        assert!(result.is_none());
    }

    #[test]
    fn test_checkpointer_remove_server_state() {
        let temp_dir = TempDir::new().expect("temp dir");
        let checkpointer = TaskCheckpointer::new(temp_dir.path());

        let state = ServerState::empty("1.0.0");
        checkpointer.save_server_state(&state).expect("save");

        // Verify it exists
        assert!(checkpointer.load_server_state().expect("load").is_some());

        // Remove it
        checkpointer.remove_server_state().expect("remove");

        // Verify it's gone
        assert!(checkpointer.load_server_state().expect("load").is_none());
    }

    #[test]
    fn test_checkpointer_clear_all() {
        let temp_dir = TempDir::new().expect("temp dir");
        let checkpointer = TaskCheckpointer::new(temp_dir.path());

        // Save some tasks and server state
        for i in 1..=3 {
            let state = make_task_state(&format!("task-{i}"));
            checkpointer.save_task_state(&state).expect("save");
        }
        let server_state = ServerState::empty("1.0.0");
        checkpointer.save_server_state(&server_state).expect("save");

        // Verify they exist
        assert_eq!(
            checkpointer.list_checkpointed_tasks().expect("list").len(),
            3
        );
        assert!(checkpointer.load_server_state().expect("load").is_some());

        // Clear all
        checkpointer.clear_all().expect("clear");

        // Verify everything is gone
        assert!(checkpointer
            .list_checkpointed_tasks()
            .expect("list")
            .is_empty());
        assert!(checkpointer.load_server_state().expect("load").is_none());
    }

    #[test]
    fn test_checkpointer_clear_all_nonexistent_dir() {
        let checkpointer = TaskCheckpointer::new("/nonexistent/path/checkpoints");

        // Should not error when clearing nonexistent directory
        checkpointer.clear_all().expect("clear should succeed");
    }

    #[test]
    fn test_checkpointer_creates_directory() {
        let temp_dir = TempDir::new().expect("temp dir");
        let nested_path = temp_dir.path().join("nested").join("checkpoints");
        let checkpointer = TaskCheckpointer::new(&nested_path);

        // Directory doesn't exist yet
        assert!(!nested_path.exists());

        // Save a task - should create the directory
        let state = make_task_state("task-1");
        checkpointer.save_task_state(&state).expect("save");

        // Directory should now exist
        assert!(nested_path.exists());
    }

    #[test]
    fn test_checkpointer_ignores_non_json_files() {
        let temp_dir = TempDir::new().expect("temp dir");
        let checkpointer = TaskCheckpointer::new(temp_dir.path());

        // Save a valid task
        let state = make_task_state("task-1");
        checkpointer.save_task_state(&state).expect("save");

        // Create a non-JSON file
        std::fs::write(temp_dir.path().join("readme.txt"), "not json").expect("write");

        // List should only return the task
        let task_ids = checkpointer.list_checkpointed_tasks().expect("list");
        assert_eq!(task_ids.len(), 1);
        assert_eq!(task_ids[0], "task-1");
    }

    #[test]
    fn test_checkpointer_handles_invalid_json() {
        let temp_dir = TempDir::new().expect("temp dir");
        let checkpointer = TaskCheckpointer::new(temp_dir.path());

        // Save a valid task
        let state = make_task_state("task-valid");
        checkpointer.save_task_state(&state).expect("save");

        // Create an invalid JSON file
        std::fs::write(temp_dir.path().join("task-invalid.json"), "not valid json").expect("write");

        // load_all_pending_tasks should skip the invalid file
        let tasks = checkpointer.load_all_pending_tasks().expect("load all");
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].task_id, "task-valid");
    }

    #[test]
    fn test_checkpointer_server_state_not_in_task_list() {
        let temp_dir = TempDir::new().expect("temp dir");
        let checkpointer = TaskCheckpointer::new(temp_dir.path());

        // Save a task and server state
        let task = make_task_state("task-1");
        checkpointer.save_task_state(&task).expect("save task");

        let server_state = ServerState::empty("1.0.0");
        checkpointer
            .save_server_state(&server_state)
            .expect("save server state");

        // List should not include _server_state
        let task_ids = checkpointer.list_checkpointed_tasks().expect("list");
        assert_eq!(task_ids.len(), 1);
        assert_eq!(task_ids[0], "task-1");

        // load_all_pending_tasks should not include server state
        let tasks = checkpointer.load_all_pending_tasks().expect("load all");
        assert_eq!(tasks.len(), 1);
    }

    #[test]
    fn test_task_state_error_display() {
        let err = TaskStateError::NotFound {
            task_id: "task-123".to_string(),
        };
        assert!(err.to_string().contains("task-123"));

        let err = TaskStateError::Io {
            reason: "file not found".to_string(),
        };
        assert!(err.to_string().contains("file not found"));

        let err = TaskStateError::Serialization {
            reason: "invalid json".to_string(),
        };
        assert!(err.to_string().contains("invalid json"));

        let err = TaskStateError::DirectoryError {
            reason: "permission denied".to_string(),
        };
        assert!(err.to_string().contains("permission denied"));
    }

    #[test]
    fn test_task_state_error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err: TaskStateError = io_err.into();
        assert!(matches!(err, TaskStateError::Io { .. }));
    }

    #[test]
    fn test_task_state_error_from_serde() {
        let json_err = serde_json::from_str::<TaskState>("invalid").expect_err("should fail");
        let err: TaskStateError = json_err.into();
        assert!(matches!(err, TaskStateError::Serialization { .. }));
    }
}
