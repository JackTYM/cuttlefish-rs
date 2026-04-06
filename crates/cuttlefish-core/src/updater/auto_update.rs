//! Automatic update system with state preservation.
//!
//! Polls for updates in the background and performs seamless updates
//! by persisting state before restart and restoring it after.

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use tokio::sync::{RwLock, broadcast};
use tracing::{error, info, warn};

use super::{
    BinaryDownloader, DownloadConfig, DownloadProgress, UpdateChecker, UpdateConfig,
    get_platform_binary_name,
};

/// Configuration for automatic updates.
#[derive(Debug, Clone)]
pub struct AutoUpdateConfig {
    /// Whether auto-update is enabled.
    pub enabled: bool,
    /// How often to check for updates (in seconds).
    pub poll_interval_secs: u64,
    /// Whether to automatically apply updates when found.
    pub auto_apply: bool,
    /// Directory to store downloaded updates.
    pub download_dir: PathBuf,
    /// Path to state file for persistence across restarts.
    pub state_file: PathBuf,
}

impl Default for AutoUpdateConfig {
    fn default() -> Self {
        // Use /var/cache/cuttlefish for system services (created by install.sh)
        // Fall back to user cache dir for local development
        let download_dir = if PathBuf::from("/var/cache/cuttlefish").exists() {
            PathBuf::from("/var/cache/cuttlefish")
        } else {
            dirs::cache_dir()
                .unwrap_or_else(std::env::temp_dir)
                .join("cuttlefish-updates")
        };

        let data_dir = dirs::data_dir().unwrap_or_else(|| PathBuf::from("/var/lib/cuttlefish"));

        Self {
            enabled: false,
            poll_interval_secs: 3600, // 1 hour
            auto_apply: true,
            download_dir,
            state_file: data_dir.join("cuttlefish/restart_state.json"),
        }
    }
}

/// State to persist across restarts.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct RestartState {
    /// Active session IDs that need to be restored.
    pub active_sessions: Vec<String>,
    /// Pending approvals that were waiting.
    pub pending_approvals: Vec<PendingApprovalState>,
    /// Timestamp when state was saved.
    pub saved_at: String,
    /// Version we're updating from.
    pub from_version: String,
    /// Version we're updating to.
    pub to_version: String,
}

/// Persisted state for a pending approval.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PendingApprovalState {
    /// Unique identifier for this action.
    pub action_id: String,
    /// Project this action belongs to.
    pub project_id: String,
    /// Type of action (e.g., "file_write", "command_exec").
    pub action_type: String,
    /// Human-readable description of the action.
    pub description: String,
}

/// Manages automatic updates with state preservation.
pub struct AutoUpdater {
    config: AutoUpdateConfig,
    checker: UpdateChecker,
    shutdown_tx: broadcast::Sender<()>,
    is_updating: Arc<AtomicBool>,
    state: Arc<RwLock<Option<RestartState>>>,
}

impl AutoUpdater {
    /// Create a new auto-updater.
    pub fn new(config: AutoUpdateConfig) -> Self {
        let (shutdown_tx, _) = broadcast::channel(1);
        let checker = UpdateChecker::new(UpdateConfig::default());

        Self {
            config,
            checker,
            shutdown_tx,
            is_updating: Arc::new(AtomicBool::new(false)),
            state: Arc::new(RwLock::new(None)),
        }
    }

    /// Check if we're currently in the middle of an update.
    pub fn is_updating(&self) -> bool {
        self.is_updating.load(Ordering::SeqCst)
    }

    /// Get a shutdown receiver to listen for update-triggered shutdowns.
    pub fn shutdown_receiver(&self) -> broadcast::Receiver<()> {
        self.shutdown_tx.subscribe()
    }

    /// Load restart state from disk (call on startup).
    pub async fn load_restart_state(&self) -> Option<RestartState> {
        if !self.config.state_file.exists() {
            return None;
        }

        match tokio::fs::read_to_string(&self.config.state_file).await {
            Ok(content) => {
                match serde_json::from_str(&content) {
                    Ok(state) => {
                        // Delete the state file after loading
                        let _ = tokio::fs::remove_file(&self.config.state_file).await;
                        info!("Loaded restart state from previous update");
                        Some(state)
                    }
                    Err(e) => {
                        warn!("Failed to parse restart state: {}", e);
                        let _ = tokio::fs::remove_file(&self.config.state_file).await;
                        None
                    }
                }
            }
            Err(e) => {
                warn!("Failed to read restart state: {}", e);
                None
            }
        }
    }

    /// Save restart state to disk before updating.
    pub async fn save_restart_state(&self, state: RestartState) -> anyhow::Result<()> {
        if let Some(parent) = self.config.state_file.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let content = serde_json::to_string_pretty(&state)?;
        tokio::fs::write(&self.config.state_file, content).await?;

        *self.state.write().await = Some(state);

        info!(
            "Saved restart state to {}",
            self.config.state_file.display()
        );
        Ok(())
    }

    /// Start the background update polling task.
    pub fn start_polling(self: Arc<Self>) -> tokio::task::JoinHandle<()> {
        let updater = self.clone();

        tokio::spawn(async move {
            if !updater.config.enabled {
                info!("Auto-update is disabled");
                return;
            }

            info!(
                "Auto-update enabled, polling every {} seconds",
                updater.config.poll_interval_secs
            );

            let mut interval =
                tokio::time::interval(Duration::from_secs(updater.config.poll_interval_secs));
            let mut shutdown_rx = updater.shutdown_tx.subscribe();

            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        if let Err(e) = updater.check_and_update().await {
                            warn!("Auto-update check failed: {}", e);
                        }
                    }
                    _ = shutdown_rx.recv() => {
                        info!("Auto-updater shutting down");
                        break;
                    }
                }
            }
        })
    }

    /// Check for updates and apply if available.
    async fn check_and_update(&self) -> anyhow::Result<()> {
        // Don't check if already updating
        if self.is_updating.load(Ordering::SeqCst) {
            return Ok(());
        }

        let update = match self.checker.check_for_update().await? {
            Some(update) => update,
            None => return Ok(()),
        };

        info!(
            "Update available: v{} -> v{}",
            update.current_version, update.latest_version
        );

        if !self.config.auto_apply {
            info!("Auto-apply disabled, skipping automatic update");
            return Ok(());
        }

        // Set updating flag
        self.is_updating.store(true, Ordering::SeqCst);

        let download_url = match update.download_url {
            Some(url) => url,
            None => {
                warn!("No download URL for platform, skipping update");
                self.is_updating.store(false, Ordering::SeqCst);
                return Ok(());
            }
        };

        // Download the update
        info!("Downloading update...");

        tokio::fs::create_dir_all(&self.config.download_dir).await?;

        let binary_name = get_platform_binary_name();
        let dest_path = self.config.download_dir.join(&binary_name);

        let dl_config = DownloadConfig {
            download_dir: self.config.download_dir.clone(),
            verify_checksums: update.checksum_url.is_some(),
        };
        let downloader = BinaryDownloader::new(dl_config);

        if let Some(ref checksum_url) = update.checksum_url {
            downloader
                .download_and_verify(
                    &download_url,
                    checksum_url,
                    &dest_path,
                    None::<&(dyn Fn(DownloadProgress) + Send + Sync)>,
                )
                .await?;
        } else {
            downloader
                .download_binary(
                    &download_url,
                    &dest_path,
                    None::<&(dyn Fn(DownloadProgress) + Send + Sync)>,
                )
                .await?;
        }

        info!("Update downloaded to {}", dest_path.display());

        // Signal that we're ready to restart
        // The main application should listen for this and:
        // 1. Save state
        // 2. Call apply_update
        let _ = self.shutdown_tx.send(());

        Ok(())
    }

    /// Apply the downloaded update. Call this after saving state.
    pub async fn apply_update(&self, state: RestartState) -> anyhow::Result<()> {
        // Save state first
        self.save_restart_state(state).await?;

        let binary_name = get_platform_binary_name();
        let downloaded_path = self.config.download_dir.join(&binary_name);

        if !downloaded_path.exists() {
            anyhow::bail!(
                "Downloaded update not found at {}",
                downloaded_path.display()
            );
        }

        let current_exe = std::env::current_exe()?;
        let backup_path = current_exe.with_extension("bak");

        info!("Applying update...");
        info!("  Current: {}", current_exe.display());
        info!("  New: {}", downloaded_path.display());

        // Create backup
        std::fs::copy(&current_exe, &backup_path)?;

        // On Unix, we can use a clever trick: rename is atomic
        // Copy new binary to a temp location next to current, then rename
        let temp_path = current_exe.with_extension("new");
        std::fs::copy(&downloaded_path, &temp_path)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&temp_path)?.permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&temp_path, perms)?;
        }

        // Atomic rename (works even if binary is running on most Unix systems)
        std::fs::rename(&temp_path, &current_exe)?;

        info!("Update applied, signaling restart...");

        // Clean up
        let _ = std::fs::remove_file(&downloaded_path);

        // Signal systemd to restart us, or exec ourselves
        self.trigger_restart().await
    }

    /// Trigger a restart of the service.
    async fn trigger_restart(&self) -> anyhow::Result<()> {
        // Try systemd first
        let systemd_result = std::process::Command::new("systemctl")
            .args(["restart", "cuttlefish"])
            .status();

        if let Ok(status) = systemd_result
            && status.success()
        {
            info!("Triggered systemd restart");
            // Give systemd a moment to kill us
            tokio::time::sleep(Duration::from_secs(5)).await;
            return Ok(());
        }

        // If systemd isn't available/doesn't work, use exec to replace ourselves
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            let current_exe = std::env::current_exe()?;
            let args: Vec<String> = std::env::args().collect();

            info!("Executing new binary directly");

            let err = std::process::Command::new(&current_exe)
                .args(&args[1..])
                .exec();

            // If we get here, exec failed
            error!("Failed to exec new binary: {}", err);
            anyhow::bail!("Failed to restart: {}", err);
        }

        #[cfg(not(unix))]
        {
            anyhow::bail!("Auto-restart not supported on this platform. Please restart manually.");
        }
    }

    /// Shutdown the auto-updater.
    pub fn shutdown(&self) {
        let _ = self.shutdown_tx.send(());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AutoUpdateConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.poll_interval_secs, 3600);
        assert!(config.auto_apply);
    }

    #[test]
    fn test_restart_state_serialization() {
        let state = RestartState {
            active_sessions: vec!["session1".to_string(), "session2".to_string()],
            pending_approvals: vec![PendingApprovalState {
                action_id: "action1".to_string(),
                project_id: "project1".to_string(),
                action_type: "file_write".to_string(),
                description: "Write to file".to_string(),
            }],
            saved_at: "2024-01-01T00:00:00Z".to_string(),
            from_version: "0.2.0".to_string(),
            to_version: "0.2.1".to_string(),
        };

        let json = serde_json::to_string(&state).unwrap();
        let restored: RestartState = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.active_sessions.len(), 2);
        assert_eq!(restored.pending_approvals.len(), 1);
    }
}
