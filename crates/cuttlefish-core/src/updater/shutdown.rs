//! Graceful shutdown, task checkpointing, and process replacement for updates.
//!
//! This module provides:
//! - [`ShutdownError`] for shutdown-related errors
//! - [`ShutdownConfig`] for configuring shutdown behavior
//! - [`ShutdownState`] for tracking shutdown progress
//! - [`ShutdownSignal`] for signaling shutdown to running tasks
//! - [`UpdateCoordinator`] for orchestrating the full update flow

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use thiserror::Error;
use tokio::sync::watch;
use tracing::{debug, info, warn};

use super::downloader::{BinaryDownloader, DownloadError};
use super::task_state::{ServerState, TaskCheckpointer, TaskState, TaskStateError};
use super::UpdateChecker;

/// Errors that can occur during graceful shutdown.
#[derive(Error, Debug)]
pub enum ShutdownError {
    /// Failed to checkpoint task state.
    #[error("Checkpoint failed: {reason}")]
    CheckpointFailed {
        /// Reason for the failure.
        reason: String,
    },

    /// Failed to execute the new binary.
    #[error("Exec failed: {reason}")]
    ExecFailed {
        /// Reason for the failure.
        reason: String,
    },

    /// I/O error during shutdown operations.
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// Download failed during update.
    #[error("Download failed: {0}")]
    DownloadFailed(#[from] DownloadError),

    /// Timed out waiting for tasks to drain.
    #[error("Task drain timeout after {timeout_secs}s")]
    TaskDrainTimeout {
        /// Timeout duration in seconds.
        timeout_secs: u64,
    },

    /// Task state error.
    #[error("Task state error: {0}")]
    TaskState(#[from] TaskStateError),
}

/// Configuration for graceful shutdown behavior.
#[derive(Debug, Clone)]
pub struct ShutdownConfig {
    /// How long to wait for tasks to complete before forcing shutdown.
    pub drain_timeout: Duration,
    /// Directory for storing task checkpoints.
    pub checkpoint_dir: PathBuf,
    /// Whether to backup the current binary before replacement.
    pub backup_current_binary: bool,
}

impl Default for ShutdownConfig {
    fn default() -> Self {
        Self {
            drain_timeout: Duration::from_secs(30),
            checkpoint_dir: PathBuf::from(".cuttlefish/task_checkpoints"),
            backup_current_binary: true,
        }
    }
}

/// Current state of the shutdown process.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ShutdownState {
    /// Normal operation, no shutdown in progress.
    #[default]
    Running,
    /// Waiting for tasks to pause/complete.
    Draining,
    /// Saving task state to disk.
    Checkpointing,
    /// About to replace the process with new binary.
    Restarting,
}

impl std::fmt::Display for ShutdownState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Running => write!(f, "running"),
            Self::Draining => write!(f, "draining"),
            Self::Checkpointing => write!(f, "checkpointing"),
            Self::Restarting => write!(f, "restarting"),
        }
    }
}

/// Signal for coordinating shutdown across tasks.
///
/// Tasks can check `is_shutdown_requested()` in their loops to gracefully
/// pause when a shutdown is initiated.
#[derive(Debug, Clone)]
pub struct ShutdownSignal {
    shutdown_requested: Arc<AtomicBool>,
}

impl Default for ShutdownSignal {
    fn default() -> Self {
        Self::new()
    }
}

impl ShutdownSignal {
    /// Create a new shutdown signal.
    pub fn new() -> Self {
        Self {
            shutdown_requested: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Check if shutdown has been requested.
    ///
    /// Tasks should call this periodically and pause gracefully when it returns true.
    pub fn is_shutdown_requested(&self) -> bool {
        self.shutdown_requested.load(Ordering::SeqCst)
    }

    /// Request shutdown. All tasks checking this signal will see the request.
    pub fn request_shutdown(&self) {
        self.shutdown_requested.store(true, Ordering::SeqCst);
        debug!("Shutdown requested via signal");
    }

    /// Reset the shutdown signal (e.g., after a cancelled shutdown).
    pub fn reset(&self) {
        self.shutdown_requested.store(false, Ordering::SeqCst);
        debug!("Shutdown signal reset");
    }
}

/// Callback type for getting current tasks to checkpoint.
pub type TaskProvider = Box<dyn Fn() -> Vec<TaskState> + Send + Sync>;

/// Callback type for signaling tasks to pause.
pub type PauseCallback = Box<dyn Fn() + Send + Sync>;

/// Orchestrates the full update flow: drain tasks, checkpoint, and restart.
pub struct UpdateCoordinator {
    checker: UpdateChecker,
    downloader: BinaryDownloader,
    checkpointer: TaskCheckpointer,
    config: ShutdownConfig,
    state: watch::Sender<ShutdownState>,
    state_rx: watch::Receiver<ShutdownState>,
    shutdown_signal: ShutdownSignal,
}

impl UpdateCoordinator {
    /// Create a new update coordinator.
    pub fn new(
        checker: UpdateChecker,
        downloader: BinaryDownloader,
        checkpointer: TaskCheckpointer,
        config: ShutdownConfig,
    ) -> Self {
        let (state_tx, state_rx) = watch::channel(ShutdownState::Running);
        Self {
            checker,
            downloader,
            checkpointer,
            config,
            state: state_tx,
            state_rx,
            shutdown_signal: ShutdownSignal::new(),
        }
    }

    /// Get the current shutdown state.
    pub fn state(&self) -> ShutdownState {
        *self.state_rx.borrow()
    }

    /// Get a receiver for state changes.
    pub fn state_receiver(&self) -> watch::Receiver<ShutdownState> {
        self.state_rx.clone()
    }

    /// Get the shutdown signal for tasks to monitor.
    pub fn shutdown_signal(&self) -> ShutdownSignal {
        self.shutdown_signal.clone()
    }

    /// Get a reference to the update checker.
    pub fn checker(&self) -> &UpdateChecker {
        &self.checker
    }

    /// Get a reference to the binary downloader.
    pub fn downloader(&self) -> &BinaryDownloader {
        &self.downloader
    }

    /// Get a reference to the task checkpointer.
    pub fn checkpointer(&self) -> &TaskCheckpointer {
        &self.checkpointer
    }

    /// Initiate the update process with a new binary.
    ///
    /// This method:
    /// 1. Sets state to Draining and signals tasks to pause
    /// 2. Waits for drain_timeout or all tasks paused
    /// 3. Sets state to Checkpointing and saves all task states
    /// 4. Backs up current binary if configured
    /// 5. Sets state to Restarting
    /// 6. Calls exec_replace to replace the current process
    ///
    /// # Arguments
    /// * `new_binary_path` - Path to the new binary to execute
    /// * `pause_callback` - Called to signal tasks to pause
    /// * `task_provider` - Called to get current tasks for checkpointing
    /// * `is_drained` - Called to check if all tasks have paused
    ///
    /// # Errors
    /// Returns error if checkpointing fails, backup fails, or exec fails.
    pub async fn initiate_update<F>(
        &self,
        new_binary_path: &Path,
        pause_callback: Option<&PauseCallback>,
        task_provider: &TaskProvider,
        is_drained: F,
    ) -> Result<(), ShutdownError>
    where
        F: Fn() -> bool,
    {
        info!(
            new_binary = ?new_binary_path,
            "Initiating update process"
        );

        // Step 1: Set state to Draining
        self.set_state(ShutdownState::Draining);
        self.shutdown_signal.request_shutdown();

        // Signal tasks to pause
        if let Some(callback) = pause_callback {
            callback();
        }

        // Step 2: Wait for drain_timeout or all tasks paused
        let drain_start = std::time::Instant::now();
        let drain_timeout = self.config.drain_timeout;

        while !is_drained() {
            if drain_start.elapsed() >= drain_timeout {
                warn!(
                    timeout_secs = drain_timeout.as_secs(),
                    "Task drain timeout reached, proceeding with checkpoint"
                );
                break;
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        debug!(
            elapsed_ms = drain_start.elapsed().as_millis(),
            "Drain phase complete"
        );

        // Step 3: Set state to Checkpointing
        self.set_state(ShutdownState::Checkpointing);

        // Get current tasks and checkpoint them
        let tasks = task_provider();
        let server_state = ServerState::new(env!("CARGO_PKG_VERSION"), tasks);

        self.checkpointer
            .save_server_state(&server_state)
            .map_err(|e| ShutdownError::CheckpointFailed {
                reason: e.to_string(),
            })?;

        info!(
            task_count = server_state.tasks.len(),
            "Checkpointed server state"
        );

        // Step 4: Backup current binary if configured
        if self.config.backup_current_binary {
            let current_exe = std::env::current_exe()?;
            let backup_dir = self.config.checkpoint_dir.join("binary_backup");
            backup_binary(&current_exe, &backup_dir)?;
        }

        // Step 5: Set state to Restarting
        self.set_state(ShutdownState::Restarting);

        // Step 6: Replace the process
        info!(
            binary = ?new_binary_path,
            "Executing new binary"
        );

        exec_replace(new_binary_path)?;

        // This line should never be reached on Unix (exec replaces the process)
        // On Windows, we spawn and exit, so this also shouldn't be reached
        Ok(())
    }

    /// Checkpoint all tasks and shutdown without replacement.
    ///
    /// Use this for clean shutdown without updating.
    pub fn checkpoint_and_shutdown(
        &self,
        task_provider: &TaskProvider,
    ) -> Result<(), ShutdownError> {
        info!("Initiating clean shutdown with checkpoint");

        self.set_state(ShutdownState::Checkpointing);
        self.shutdown_signal.request_shutdown();

        let tasks = task_provider();
        let server_state = ServerState::new(env!("CARGO_PKG_VERSION"), tasks);

        self.checkpointer
            .save_server_state(&server_state)
            .map_err(|e| ShutdownError::CheckpointFailed {
                reason: e.to_string(),
            })?;

        info!(
            task_count = server_state.tasks.len(),
            "Checkpointed server state for clean shutdown"
        );

        Ok(())
    }

    fn set_state(&self, state: ShutdownState) {
        debug!(state = %state, "Shutdown state changed");
        // Ignore send error - receivers may have been dropped
        let _ = self.state.send(state);
    }
}

/// Replace the current process with a new binary.
///
/// On Unix, uses `exec()` to replace the current process in-place.
/// On Windows, spawns the new process and exits the current one.
///
/// # Arguments
/// * `binary_path` - Path to the new binary to execute
///
/// # Errors
/// Returns error if the exec/spawn fails.
pub fn exec_replace(binary_path: &Path) -> Result<(), ShutdownError> {
    // Verify the binary exists and is executable
    if !binary_path.exists() {
        return Err(ShutdownError::ExecFailed {
            reason: format!("Binary not found: {}", binary_path.display()),
        });
    }

    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;

        info!(binary = ?binary_path, "Executing new binary via exec (Unix)");

        let err = Command::new(binary_path)
            .arg("--resume-tasks")
            .exec();

        // exec() only returns if it fails
        Err(ShutdownError::ExecFailed {
            reason: err.to_string(),
        })
    }

    #[cfg(windows)]
    {
        info!(binary = ?binary_path, "Spawning new binary and exiting (Windows)");

        Command::new(binary_path)
            .arg("--resume-tasks")
            .spawn()
            .map_err(|e| ShutdownError::ExecFailed {
                reason: e.to_string(),
            })?;

        // Exit the current process
        std::process::exit(0);
    }

    #[cfg(not(any(unix, windows)))]
    {
        Err(ShutdownError::ExecFailed {
            reason: "Unsupported platform for process replacement".to_string(),
        })
    }
}

/// Backup the current binary to a backup directory.
///
/// Creates a timestamped backup of the binary for rollback purposes.
///
/// # Arguments
/// * `current` - Path to the current binary
/// * `backup_dir` - Directory to store the backup
///
/// # Errors
/// Returns error if backup fails.
pub fn backup_binary(current: &Path, backup_dir: &Path) -> Result<PathBuf, ShutdownError> {
    std::fs::create_dir_all(backup_dir)?;

    let filename = current
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("cuttlefish");

    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let backup_name = format!("{filename}.backup.{timestamp}");
    let backup_path = backup_dir.join(backup_name);

    std::fs::copy(current, &backup_path)?;

    info!(
        source = ?current,
        backup = ?backup_path,
        "Backed up current binary"
    );

    Ok(backup_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::updater::{DownloadConfig, UpdateConfig};
    use std::sync::atomic::AtomicUsize;
    use tempfile::TempDir;

    fn make_test_coordinator(temp_dir: &Path) -> UpdateCoordinator {
        let checker = UpdateChecker::new(UpdateConfig {
            owner: "test".to_string(),
            repo: "repo".to_string(),
            poll_interval: Duration::from_secs(3600),
            current_version: "0.1.0".to_string(),
        });

        let downloader = BinaryDownloader::new(DownloadConfig {
            download_dir: temp_dir.to_path_buf(),
            verify_checksums: false,
        });

        let checkpointer = TaskCheckpointer::new(temp_dir.join("checkpoints"));

        let config = ShutdownConfig {
            drain_timeout: Duration::from_millis(100),
            checkpoint_dir: temp_dir.to_path_buf(),
            backup_current_binary: false, // Don't backup in tests
        };

        UpdateCoordinator::new(checker, downloader, checkpointer, config)
    }

    #[test]
    fn test_shutdown_state_default() {
        let state = ShutdownState::default();
        assert_eq!(state, ShutdownState::Running);
    }

    #[test]
    fn test_shutdown_state_display() {
        assert_eq!(ShutdownState::Running.to_string(), "running");
        assert_eq!(ShutdownState::Draining.to_string(), "draining");
        assert_eq!(ShutdownState::Checkpointing.to_string(), "checkpointing");
        assert_eq!(ShutdownState::Restarting.to_string(), "restarting");
    }

    #[test]
    fn test_shutdown_config_default() {
        let config = ShutdownConfig::default();
        assert_eq!(config.drain_timeout, Duration::from_secs(30));
        assert!(config.backup_current_binary);
    }

    #[test]
    fn test_shutdown_signal_new() {
        let signal = ShutdownSignal::new();
        assert!(!signal.is_shutdown_requested());
    }

    #[test]
    fn test_shutdown_signal_request() {
        let signal = ShutdownSignal::new();
        assert!(!signal.is_shutdown_requested());

        signal.request_shutdown();
        assert!(signal.is_shutdown_requested());
    }

    #[test]
    fn test_shutdown_signal_reset() {
        let signal = ShutdownSignal::new();
        signal.request_shutdown();
        assert!(signal.is_shutdown_requested());

        signal.reset();
        assert!(!signal.is_shutdown_requested());
    }

    #[test]
    fn test_shutdown_signal_clone() {
        let signal1 = ShutdownSignal::new();
        let signal2 = signal1.clone();

        signal1.request_shutdown();
        assert!(signal2.is_shutdown_requested());
    }

    #[test]
    fn test_shutdown_signal_default() {
        let signal = ShutdownSignal::default();
        assert!(!signal.is_shutdown_requested());
    }

    #[test]
    fn test_update_coordinator_state() {
        let temp_dir = TempDir::new().expect("temp dir");
        let coordinator = make_test_coordinator(temp_dir.path());

        assert_eq!(coordinator.state(), ShutdownState::Running);
    }

    #[test]
    fn test_update_coordinator_shutdown_signal() {
        let temp_dir = TempDir::new().expect("temp dir");
        let coordinator = make_test_coordinator(temp_dir.path());

        let signal = coordinator.shutdown_signal();
        assert!(!signal.is_shutdown_requested());
    }

    #[test]
    fn test_update_coordinator_accessors() {
        let temp_dir = TempDir::new().expect("temp dir");
        let coordinator = make_test_coordinator(temp_dir.path());

        // Just verify these don't panic
        let _ = coordinator.checker();
        let _ = coordinator.downloader();
        let _ = coordinator.checkpointer();
    }

    #[test]
    fn test_backup_binary() {
        let temp_dir = TempDir::new().expect("temp dir");

        // Create a fake binary file
        let fake_binary = temp_dir.path().join("fake_binary");
        std::fs::write(&fake_binary, b"fake binary content").expect("write fake binary");

        let backup_dir = temp_dir.path().join("backups");
        let backup_path = backup_binary(&fake_binary, &backup_dir).expect("backup");

        assert!(backup_path.exists());
        assert!(backup_path.to_string_lossy().contains("backup"));

        let backup_content = std::fs::read(&backup_path).expect("read backup");
        assert_eq!(backup_content, b"fake binary content");
    }

    #[test]
    fn test_backup_binary_creates_dir() {
        let temp_dir = TempDir::new().expect("temp dir");

        let fake_binary = temp_dir.path().join("fake_binary");
        std::fs::write(&fake_binary, b"content").expect("write");

        let nested_backup_dir = temp_dir.path().join("nested").join("backup").join("dir");
        assert!(!nested_backup_dir.exists());

        let backup_path = backup_binary(&fake_binary, &nested_backup_dir).expect("backup");
        assert!(backup_path.exists());
        assert!(nested_backup_dir.exists());
    }

    #[test]
    fn test_exec_replace_nonexistent_binary() {
        let result = exec_replace(Path::new("/nonexistent/binary/path"));
        assert!(result.is_err());

        match result {
            Err(ShutdownError::ExecFailed { reason }) => {
                assert!(reason.contains("not found"));
            }
            _ => panic!("Expected ExecFailed error"),
        }
    }

    #[test]
    fn test_shutdown_error_display() {
        let err = ShutdownError::CheckpointFailed {
            reason: "disk full".to_string(),
        };
        assert!(err.to_string().contains("disk full"));

        let err = ShutdownError::ExecFailed {
            reason: "permission denied".to_string(),
        };
        assert!(err.to_string().contains("permission denied"));

        let err = ShutdownError::TaskDrainTimeout { timeout_secs: 30 };
        assert!(err.to_string().contains("30"));
    }

    #[test]
    fn test_shutdown_error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err: ShutdownError = io_err.into();
        assert!(matches!(err, ShutdownError::IoError(_)));
    }

    #[tokio::test]
    async fn test_checkpoint_and_shutdown() {
        let temp_dir = TempDir::new().expect("temp dir");
        let coordinator = make_test_coordinator(temp_dir.path());

        let task_provider: TaskProvider = Box::new(|| {
            vec![TaskState::new(
                "task-1",
                "project-1",
                "coder",
                "Test task",
            )]
        });

        let result = coordinator.checkpoint_and_shutdown(&task_provider);
        assert!(result.is_ok());

        // Verify checkpoint was saved
        let loaded = coordinator
            .checkpointer()
            .load_server_state()
            .expect("load")
            .expect("should exist");

        assert_eq!(loaded.tasks.len(), 1);
        assert_eq!(loaded.tasks[0].task_id, "task-1");
    }

    #[tokio::test]
    async fn test_initiate_update_checkpoints_tasks() {
        let temp_dir = TempDir::new().expect("temp dir");
        let coordinator = make_test_coordinator(temp_dir.path());

        // Create a fake binary to "update" to
        let fake_new_binary = temp_dir.path().join("new_binary");
        std::fs::write(&fake_new_binary, b"#!/bin/bash\nexit 0").expect("write");

        // Make it executable on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&fake_new_binary)
                .expect("metadata")
                .permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&fake_new_binary, perms).expect("set permissions");
        }

        let pause_called = Arc::new(AtomicBool::new(false));
        let pause_called_clone = pause_called.clone();

        let pause_callback: PauseCallback = Box::new(move || {
            pause_called_clone.store(true, Ordering::SeqCst);
        });

        let task_provider: TaskProvider = Box::new(|| {
            vec![TaskState::new(
                "task-1",
                "project-1",
                "coder",
                "Test task",
            )]
        });

        let drain_count = Arc::new(AtomicUsize::new(0));
        let drain_count_clone = drain_count.clone();

        // We can't actually test exec_replace without replacing the test process,
        // so we'll test up to the checkpoint phase by using a binary that doesn't exist
        // after verifying the checkpoint was created.

        // First, let's test that the checkpoint is created even if exec fails
        let nonexistent_binary = temp_dir.path().join("nonexistent");

        let result = coordinator
            .initiate_update(
                &nonexistent_binary,
                Some(&pause_callback),
                &task_provider,
                move || {
                    // Return true after a few calls to simulate tasks draining
                    drain_count_clone.fetch_add(1, Ordering::SeqCst) >= 2
                },
            )
            .await;

        // Should fail because binary doesn't exist
        assert!(result.is_err());

        // But pause should have been called
        assert!(pause_called.load(Ordering::SeqCst));

        // And checkpoint should have been saved
        let loaded = coordinator
            .checkpointer()
            .load_server_state()
            .expect("load")
            .expect("should exist");

        assert_eq!(loaded.tasks.len(), 1);
    }

    #[tokio::test]
    async fn test_state_transitions() {
        let temp_dir = TempDir::new().expect("temp dir");
        let coordinator = make_test_coordinator(temp_dir.path());

        let state_rx = coordinator.state_receiver();

        // Initial state
        assert_eq!(*state_rx.borrow(), ShutdownState::Running);

        // Start checkpoint_and_shutdown which changes state
        let task_provider: TaskProvider = Box::new(Vec::new);
        coordinator
            .checkpoint_and_shutdown(&task_provider)
            .expect("checkpoint");

        // State should have changed
        assert_eq!(*state_rx.borrow(), ShutdownState::Checkpointing);
    }

    #[test]
    fn test_shutdown_signal_thread_safety() {
        use std::thread;

        let signal = ShutdownSignal::new();
        let signal_clone = signal.clone();

        let handle = thread::spawn(move || {
            signal_clone.request_shutdown();
        });

        handle.join().expect("thread join");
        assert!(signal.is_shutdown_requested());
    }
}
