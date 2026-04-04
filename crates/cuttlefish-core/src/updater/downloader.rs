//! Binary downloader with checksum verification and progress reporting.

use std::path::Path;

use futures::StreamExt;
use sha2::{Digest, Sha256};
use thiserror::Error;
use tokio::io::AsyncWriteExt;

/// Error type for download operations.
#[derive(Error, Debug)]
pub enum DownloadError {
    /// HTTP request failed.
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    /// File I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Checksum verification failed.
    #[error("Checksum mismatch: expected {expected}, got {actual}")]
    ChecksumMismatch {
        /// Expected SHA256 hash.
        expected: String,
        /// Actual computed hash.
        actual: String,
    },

    /// Invalid URL format.
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    /// Failed to parse checksum file.
    #[error("Invalid checksum format: {0}")]
    InvalidChecksumFormat(String),
}

/// Progress information for an ongoing download.
#[derive(Debug, Clone, Copy)]
pub struct DownloadProgress {
    /// Bytes downloaded so far.
    pub bytes_downloaded: u64,
    /// Total bytes to download (if known).
    pub total_bytes: Option<u64>,
    /// Download percentage (0-100), None if total is unknown.
    pub percent: Option<f64>,
}

/// Configuration for the binary downloader.
#[derive(Debug, Clone)]
pub struct DownloadConfig {
    /// Directory to store downloaded files.
    pub download_dir: std::path::PathBuf,
    /// Whether to verify checksums after download.
    pub verify_checksums: bool,
}

impl Default for DownloadConfig {
    fn default() -> Self {
        Self {
            download_dir: std::env::temp_dir(),
            verify_checksums: true,
        }
    }
}

/// Downloads and verifies binary releases from GitHub.
pub struct BinaryDownloader {
    config: DownloadConfig,
    client: reqwest::Client,
}

impl BinaryDownloader {
    /// Creates a new downloader with the given configuration.
    pub fn new(config: DownloadConfig) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
        }
    }

    /// Creates a new downloader with default configuration.
    pub fn with_defaults() -> Self {
        Self::new(DownloadConfig::default())
    }

    /// Downloads a binary from the given URL to the destination path.
    ///
    /// Streams the download to disk to avoid loading large files into memory.
    pub async fn download_binary(
        &self,
        url: &str,
        dest: &Path,
        progress_callback: Option<&(dyn Fn(DownloadProgress) + Send + Sync)>,
    ) -> Result<(), DownloadError> {
        tracing::info!(url = %url, dest = ?dest, "Starting binary download");

        let response = self
            .client
            .get(url)
            .send()
            .await?
            .error_for_status()
            .map_err(DownloadError::Http)?;

        let total_bytes = response.content_length();
        let mut bytes_downloaded: u64 = 0;

        if let Some(parent) = dest.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let mut file = tokio::fs::File::create(dest).await?;
        let mut stream = response.bytes_stream();

        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result?;
            file.write_all(&chunk).await?;
            bytes_downloaded += chunk.len() as u64;

            if let Some(callback) = progress_callback {
                let progress = DownloadProgress {
                    bytes_downloaded,
                    total_bytes,
                    percent: total_bytes.map(|total| {
                        if total > 0 {
                            (bytes_downloaded as f64 / total as f64) * 100.0
                        } else {
                            0.0
                        }
                    }),
                };
                callback(progress);
            }
        }

        file.flush().await?;
        tracing::info!(bytes = bytes_downloaded, "Download complete");

        Ok(())
    }

    /// Verifies that a file matches the expected SHA256 checksum.
    pub async fn verify_checksum(
        &self,
        file: &Path,
        expected_sha256: &str,
    ) -> Result<(), DownloadError> {
        tracing::debug!(file = ?file, expected = %expected_sha256, "Verifying checksum");

        let contents = tokio::fs::read(file).await?;
        let mut hasher = Sha256::new();
        hasher.update(&contents);
        let actual = hex::encode(hasher.finalize());

        let expected_normalized = expected_sha256.to_lowercase();
        if actual != expected_normalized {
            return Err(DownloadError::ChecksumMismatch {
                expected: expected_normalized,
                actual,
            });
        }

        tracing::debug!("Checksum verified successfully");
        Ok(())
    }

    /// Downloads a binary and its checksum file, then verifies the download.
    ///
    /// The checksum file is expected to be in standard sha256sum format:
    /// `{sha256sum}  {filename}`
    pub async fn download_and_verify(
        &self,
        binary_url: &str,
        checksum_url: &str,
        dest: &Path,
        progress_callback: Option<&(dyn Fn(DownloadProgress) + Send + Sync)>,
    ) -> Result<(), DownloadError> {
        self.download_binary(binary_url, dest, progress_callback)
            .await?;

        if !self.config.verify_checksums {
            tracing::warn!("Checksum verification disabled, skipping");
            return Ok(());
        }

        let checksum_response = self
            .client
            .get(checksum_url)
            .send()
            .await?
            .error_for_status()?;

        let checksum_content = checksum_response.text().await?;
        let expected_sha256 = parse_checksum_file(&checksum_content, dest)?;

        self.verify_checksum(dest, &expected_sha256).await?;

        Ok(())
    }

    /// Returns the download directory from the configuration.
    pub fn download_dir(&self) -> &Path {
        &self.config.download_dir
    }
}

/// Parses a sha256sum-format checksum file and extracts the hash for the given filename.
///
/// Format: `{sha256sum}  {filename}` (two spaces between hash and filename)
fn parse_checksum_file(content: &str, file_path: &Path) -> Result<String, DownloadError> {
    let filename = file_path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| DownloadError::InvalidUrl("Invalid file path".to_string()))?;

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        // sha256sum format: hash followed by two spaces and filename
        // Also handle single space for compatibility
        let parts: Vec<&str> = line.splitn(2, ' ').collect();
        if parts.len() != 2 {
            continue;
        }

        let hash = parts[0].trim();
        let file = parts[1].trim();

        // Handle the case where there might be a leading * for binary mode
        let file = file.trim_start_matches('*');

        if file == filename {
            if hash.len() != 64 || !hash.chars().all(|c| c.is_ascii_hexdigit()) {
                return Err(DownloadError::InvalidChecksumFormat(format!(
                    "Invalid SHA256 hash: {hash}"
                )));
            }
            return Ok(hash.to_lowercase());
        }
    }

    Err(DownloadError::InvalidChecksumFormat(format!(
        "No checksum found for file: {filename}"
    )))
}

/// Returns the platform-specific binary name for the current system.
///
/// Format: `cuttlefish-{os}-{arch}`
/// - os: linux, darwin, windows
/// - arch: x86_64, aarch64
pub fn get_platform_binary_name() -> String {
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

    format!("cuttlefish-{os}-{arch}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_platform_binary_name() {
        let name = get_platform_binary_name();
        assert!(name.starts_with("cuttlefish-"));

        #[cfg(target_os = "linux")]
        assert!(name.contains("linux"));

        #[cfg(target_os = "macos")]
        assert!(name.contains("darwin"));

        #[cfg(target_arch = "x86_64")]
        assert!(name.contains("x86_64"));

        #[cfg(target_arch = "aarch64")]
        assert!(name.contains("aarch64"));
    }

    #[test]
    fn test_parse_checksum_file_standard_format() {
        let content =
            "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08  cuttlefish-linux-x86_64\n\
             e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  cuttlefish-darwin-arm64";

        let path = Path::new("/tmp/cuttlefish-linux-x86_64");
        let result = parse_checksum_file(content, path);
        assert!(result.is_ok(), "parse failed: {:?}", result);
        assert_eq!(
            result.expect("should parse"),
            "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08"
        );
    }

    #[test]
    fn test_parse_checksum_file_binary_mode() {
        let content =
            "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08 *cuttlefish-linux-x86_64";

        let path = Path::new("/tmp/cuttlefish-linux-x86_64");
        let result = parse_checksum_file(content, path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_checksum_file_not_found() {
        let content =
            "abc123def456abc123def456abc123def456abc123def456abc123def456abc123de  other-file";

        let path = Path::new("/tmp/cuttlefish-linux-x86_64");
        let result = parse_checksum_file(content, path);
        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(DownloadError::InvalidChecksumFormat(_))
        ));
    }

    #[test]
    fn test_parse_checksum_file_invalid_hash() {
        let content = "not-a-valid-hash  cuttlefish-linux-x86_64";

        let path = Path::new("/tmp/cuttlefish-linux-x86_64");
        let result = parse_checksum_file(content, path);
        assert!(result.is_err());
    }

    #[test]
    fn test_download_config_default() {
        let config = DownloadConfig::default();
        assert!(config.verify_checksums);
        assert_eq!(config.download_dir, std::env::temp_dir());
    }

    #[test]
    fn test_download_progress() {
        let progress = DownloadProgress {
            bytes_downloaded: 500,
            total_bytes: Some(1000),
            percent: Some(50.0),
        };
        assert_eq!(progress.bytes_downloaded, 500);
        assert_eq!(progress.total_bytes, Some(1000));
        assert!((progress.percent.expect("should have percent") - 50.0).abs() < f64::EPSILON);
    }

    #[tokio::test]
    async fn test_verify_checksum_success() {
        let temp_dir = tempfile::tempdir().expect("should create temp dir");
        let file_path = temp_dir.path().join("test_file");
        tokio::fs::write(&file_path, b"test content")
            .await
            .expect("should write file");

        // SHA256 of "test content"
        let expected_hash = "6ae8a75555209fd6c44157c0aed8016e763ff435a19cf186f76863140143ff72";

        let downloader = BinaryDownloader::with_defaults();
        let result = downloader.verify_checksum(&file_path, expected_hash).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_verify_checksum_mismatch() {
        let temp_dir = tempfile::tempdir().expect("should create temp dir");
        let file_path = temp_dir.path().join("test_file");
        tokio::fs::write(&file_path, b"test content")
            .await
            .expect("should write file");

        let wrong_hash = "0000000000000000000000000000000000000000000000000000000000000000";

        let downloader = BinaryDownloader::with_defaults();
        let result = downloader.verify_checksum(&file_path, wrong_hash).await;
        assert!(result.is_err());
        assert!(matches!(result, Err(DownloadError::ChecksumMismatch { .. })));
    }

    #[test]
    fn test_binary_downloader_new() {
        let config = DownloadConfig {
            download_dir: std::path::PathBuf::from("/custom/path"),
            verify_checksums: false,
        };
        let downloader = BinaryDownloader::new(config);
        assert_eq!(
            downloader.download_dir(),
            Path::new("/custom/path")
        );
    }
}
