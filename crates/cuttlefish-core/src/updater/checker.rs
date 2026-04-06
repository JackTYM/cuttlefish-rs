//! Update checker that polls GitHub for new releases.

use std::sync::Arc;
use std::time::Duration;

use thiserror::Error;
use tokio::sync::watch;
use tokio::task::JoinHandle;
use tracing::{debug, error, info, warn};

use crate::advanced::{ReleaseAsset, ReleaseInfo, is_newer_version};

/// Errors that can occur during update checking.
#[derive(Error, Debug)]
pub enum UpdateError {
    /// HTTP request failed.
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    /// Failed to parse release response.
    #[error("Failed to parse release: {0}")]
    Parse(String),

    /// No release found.
    #[error("No release found for {owner}/{repo}")]
    NoRelease {
        /// Repository owner.
        owner: String,
        /// Repository name.
        repo: String,
    },

    /// Rate limited by GitHub API.
    #[error("GitHub API rate limited, retry after {retry_after_secs}s")]
    RateLimited {
        /// Seconds until rate limit resets.
        retry_after_secs: u64,
    },
}

/// Configuration for the update checker.
#[derive(Debug, Clone)]
pub struct UpdateConfig {
    /// GitHub repository owner.
    pub owner: String,
    /// GitHub repository name.
    pub repo: String,
    /// Poll interval for background checking.
    pub poll_interval: Duration,
    /// Current version of the application.
    pub current_version: String,
}

impl Default for UpdateConfig {
    fn default() -> Self {
        Self {
            owner: "JackTYM".to_string(),
            repo: "cuttlefish-rs".to_string(),
            poll_interval: Duration::from_secs(3600),
            current_version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}

/// Information about an available update.
#[derive(Debug, Clone)]
pub struct UpdateAvailable {
    /// Current installed version.
    pub current_version: String,
    /// Latest available version.
    pub latest_version: String,
    /// Release notes (body from GitHub release).
    pub release_notes: Option<String>,
    /// Download URL for the release binary.
    pub download_url: Option<String>,
    /// URL for the checksum file (if available).
    pub checksum_url: Option<String>,
}

/// Checks for updates from GitHub releases.
pub struct UpdateChecker {
    config: UpdateConfig,
    client: reqwest::Client,
}

impl UpdateChecker {
    /// Creates a new update checker with the given configuration.
    pub fn new(config: UpdateConfig) -> Self {
        let client = reqwest::Client::builder()
            .user_agent(format!("cuttlefish-rs/{}", env!("CARGO_PKG_VERSION")))
            .timeout(Duration::from_secs(30))
            .build()
            .expect("failed to build HTTP client - this is a bug");

        Self { config, client }
    }

    /// Creates an update checker with default configuration.
    pub fn with_defaults() -> Self {
        Self::new(UpdateConfig::default())
    }

    /// Checks GitHub for the latest release and returns update info if a newer version exists.
    pub async fn check_for_update(&self) -> Result<Option<UpdateAvailable>, UpdateError> {
        // Fetch all releases to find the latest server-v* tag
        let url = format!(
            "https://api.github.com/repos/{}/{}/releases",
            self.config.owner, self.config.repo
        );

        debug!(url = %url, "Checking for updates");

        let response = self.client.get(&url).send().await?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(UpdateError::NoRelease {
                owner: self.config.owner.clone(),
                repo: self.config.repo.clone(),
            });
        }

        if response.status() == reqwest::StatusCode::FORBIDDEN {
            let retry_after = response
                .headers()
                .get("retry-after")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse().ok())
                .unwrap_or(60);
            return Err(UpdateError::RateLimited {
                retry_after_secs: retry_after,
            });
        }

        let releases: Vec<ReleaseInfo> = response
            .json()
            .await
            .map_err(|e| UpdateError::Parse(e.to_string()))?;

        // Find the latest server release in order of preference:
        // 1. server-release-* (current format with version number)
        // 2. server-archive-* (old format, backwards compat)
        // 3. server-v* (older format, backwards compat)
        // 4. v* (very old format)
        // Note: server-latest is skipped as it doesn't contain version in tag name
        let release = releases
            .iter()
            .find(|r| r.tag_name.starts_with("server-release-"))
            .or_else(|| {
                releases
                    .iter()
                    .find(|r| r.tag_name.starts_with("server-archive-"))
            })
            .or_else(|| {
                releases
                    .iter()
                    .find(|r| r.tag_name.starts_with("server-v"))
            })
            .or_else(|| {
                releases
                    .iter()
                    .find(|r| r.tag_name.starts_with('v') && !r.tag_name.starts_with("v0.0"))
            })
            .cloned()
            .ok_or_else(|| UpdateError::NoRelease {
                owner: self.config.owner.clone(),
                repo: self.config.repo.clone(),
            })?;

        let latest_version = release
            .tag_name
            .trim_start_matches("server-release-")
            .trim_start_matches("server-archive-")
            .trim_start_matches("server-v")
            .trim_start_matches('v')
            .to_string();
        let current_version = self.config.current_version.trim_start_matches('v');

        if !is_newer_version(current_version, &latest_version) {
            debug!(
                current = %current_version,
                latest = %latest_version,
                "No update available"
            );
            return Ok(None);
        }

        info!(
            current = %current_version,
            latest = %latest_version,
            "Update available"
        );

        let (download_url, checksum_url) = self.find_asset_urls(&release.assets);

        Ok(Some(UpdateAvailable {
            current_version: self.config.current_version.clone(),
            latest_version,
            release_notes: release.body,
            download_url,
            checksum_url,
        }))
    }

    /// Starts a background task that periodically checks for updates.
    ///
    /// Returns a `JoinHandle` for the spawned task and a receiver that emits
    /// `Some(UpdateAvailable)` when an update is found.
    pub fn start_background_poll(
        self: Arc<Self>,
    ) -> (JoinHandle<()>, watch::Receiver<Option<UpdateAvailable>>) {
        let (tx, rx) = watch::channel(None);
        let poll_interval = self.config.poll_interval;

        let handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(poll_interval);
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            loop {
                interval.tick().await;

                match self.check_for_update().await {
                    Ok(Some(update)) => {
                        info!(
                            latest = %update.latest_version,
                            "Background check found update"
                        );
                        if tx.send(Some(update)).is_err() {
                            debug!("Update receiver dropped, stopping background poll");
                            break;
                        }
                    }
                    Ok(None) => {
                        debug!("Background check: no update available");
                    }
                    Err(UpdateError::RateLimited { retry_after_secs }) => {
                        warn!(
                            retry_after = retry_after_secs,
                            "Rate limited, will retry later"
                        );
                    }
                    Err(e) => {
                        error!(error = %e, "Background update check failed");
                    }
                }
            }
        });

        (handle, rx)
    }

    fn find_asset_urls(&self, assets: &[ReleaseAsset]) -> (Option<String>, Option<String>) {
        let target = Self::current_target();

        // Try new naming convention first (cuttlefish-server-*), then fall back to old (cuttlefish-*)
        let download_url = assets
            .iter()
            .find(|a| {
                a.name.contains(&target)
                    && !a.name.ends_with(".sha256")
                    && a.name.starts_with("cuttlefish-server-")
            })
            .or_else(|| {
                assets.iter().find(|a| {
                    a.name.contains(&target)
                        && !a.name.ends_with(".sha256")
                        && a.name.starts_with("cuttlefish-")
                })
            })
            .map(|a| a.browser_download_url.clone());

        let checksum_url = assets
            .iter()
            .find(|a| a.name.contains(&target) && a.name.ends_with(".sha256"))
            .map(|a| a.browser_download_url.clone());

        (download_url, checksum_url)
    }

    fn current_target() -> String {
        let os = if cfg!(target_os = "linux") {
            "linux"
        } else if cfg!(target_os = "macos") {
            "darwin"
        } else if cfg!(target_os = "windows") {
            "windows"
        } else {
            "unknown"
        };

        let arch = if cfg!(target_arch = "x86_64") {
            "x86_64"
        } else if cfg!(target_arch = "aarch64") {
            "aarch64"
        } else {
            "unknown"
        };

        format!("{}-{}", os, arch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_config_default() {
        let config = UpdateConfig::default();
        assert_eq!(config.owner, "JackTYM");
        assert_eq!(config.repo, "cuttlefish-rs");
        assert_eq!(config.poll_interval, Duration::from_secs(3600));
    }

    #[test]
    fn test_update_checker_creation() {
        let config = UpdateConfig {
            owner: "test".to_string(),
            repo: "repo".to_string(),
            poll_interval: Duration::from_secs(60),
            current_version: "0.1.0".to_string(),
        };
        let checker = UpdateChecker::new(config.clone());
        assert_eq!(checker.config.owner, "test");
        assert_eq!(checker.config.repo, "repo");
    }

    #[test]
    fn test_update_checker_with_defaults() {
        let checker = UpdateChecker::with_defaults();
        assert_eq!(checker.config.owner, "JackTYM");
        assert_eq!(checker.config.repo, "cuttlefish-rs");
    }

    #[test]
    fn test_current_target_format() {
        let target = UpdateChecker::current_target();
        assert!(target.contains('-'));
        let parts: Vec<&str> = target.split('-').collect();
        assert_eq!(parts.len(), 2);
    }

    #[test]
    fn test_find_asset_urls_empty() {
        let config = UpdateConfig::default();
        let checker = UpdateChecker::new(config);
        let (download, checksum) = checker.find_asset_urls(&[]);
        assert!(download.is_none());
        assert!(checksum.is_none());
    }

    #[test]
    fn test_find_asset_urls_with_matching_assets() {
        let config = UpdateConfig::default();
        let checker = UpdateChecker::new(config);

        let target = UpdateChecker::current_target();
        let assets = vec![
            ReleaseAsset {
                name: format!("cuttlefish-{}.tar.gz", target),
                browser_download_url: "https://example.com/binary.tar.gz".to_string(),
            },
            ReleaseAsset {
                name: format!("cuttlefish-{}.tar.gz.sha256", target),
                browser_download_url: "https://example.com/binary.tar.gz.sha256".to_string(),
            },
        ];

        let (download, checksum) = checker.find_asset_urls(&assets);
        assert_eq!(
            download,
            Some("https://example.com/binary.tar.gz".to_string())
        );
        assert_eq!(
            checksum,
            Some("https://example.com/binary.tar.gz.sha256".to_string())
        );
    }

    #[test]
    fn test_update_error_display() {
        let err = UpdateError::NoRelease {
            owner: "test".to_string(),
            repo: "repo".to_string(),
        };
        assert_eq!(err.to_string(), "No release found for test/repo");

        let err = UpdateError::RateLimited {
            retry_after_secs: 60,
        };
        assert_eq!(err.to_string(), "GitHub API rate limited, retry after 60s");

        let err = UpdateError::Parse("invalid json".to_string());
        assert_eq!(err.to_string(), "Failed to parse release: invalid json");
    }

    #[test]
    fn test_update_available_fields() {
        let update = UpdateAvailable {
            current_version: "0.1.0".to_string(),
            latest_version: "0.2.0".to_string(),
            release_notes: Some("Bug fixes".to_string()),
            download_url: Some("https://example.com/download".to_string()),
            checksum_url: Some("https://example.com/checksum".to_string()),
        };

        assert_eq!(update.current_version, "0.1.0");
        assert_eq!(update.latest_version, "0.2.0");
        assert_eq!(update.release_notes, Some("Bug fixes".to_string()));
    }

    #[tokio::test]
    async fn test_check_for_update_no_release() {
        let config = UpdateConfig {
            owner: "nonexistent-owner-12345".to_string(),
            repo: "nonexistent-repo-67890".to_string(),
            poll_interval: Duration::from_secs(60),
            current_version: "0.1.0".to_string(),
        };
        let checker = UpdateChecker::new(config);

        let result = checker.check_for_update().await;
        assert!(result.is_err());
        match result {
            Err(UpdateError::NoRelease { owner, repo }) => {
                assert_eq!(owner, "nonexistent-owner-12345");
                assert_eq!(repo, "nonexistent-repo-67890");
            }
            Err(UpdateError::Http(_)) => {}
            Err(UpdateError::RateLimited { .. }) => {}
            Err(UpdateError::Parse(_)) => {}
            _ => panic!("Expected NoRelease, Http, RateLimited, or Parse error"),
        }
    }

    #[tokio::test]
    async fn test_background_poll_receiver() {
        let config = UpdateConfig {
            owner: "test".to_string(),
            repo: "repo".to_string(),
            poll_interval: Duration::from_millis(100),
            current_version: "0.1.0".to_string(),
        };
        let checker = Arc::new(UpdateChecker::new(config));

        let (handle, rx) = checker.start_background_poll();

        assert!(rx.borrow().is_none());

        handle.abort();
    }
}
