//! Update checking, binary download, and task state checkpointing functionality.

mod checker;
mod downloader;
pub mod restore;
mod shutdown;
mod task_state;

pub use checker::{UpdateAvailable, UpdateChecker, UpdateConfig, UpdateError};
pub use downloader::{
    BinaryDownloader, DownloadConfig, DownloadError, DownloadProgress, get_platform_binary_name,
};
pub use restore::{
    RestoreConfig, RestoreError, RestoreResult, RestoredTask, TaskRestorer, should_resume_tasks,
};
pub use shutdown::{
    PauseCallback, ShutdownConfig, ShutdownError, ShutdownSignal, ShutdownState, TaskProvider,
    UpdateCoordinator, backup_binary, exec_replace,
};
pub use task_state::{
    ServerState, TaskCheckpointer, TaskProgress, TaskState, TaskStateError, TaskStateResult,
    TaskStateStatus, DEFAULT_CHECKPOINT_DIR,
};
