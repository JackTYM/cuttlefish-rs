//! TUI auto-updater that checks GitHub for new releases.

use std::env;
use std::fs;
use std::path::PathBuf;

/// Information about an available update.
#[derive(Debug, Clone)]
pub struct UpdateInfo {
    /// Current installed version.
    pub current_version: String,
    /// Latest available version.
    pub latest_version: String,
    /// Download URL for the binary.
    pub download_url: String,
}

/// Check GitHub for a newer TUI release.
pub async fn check_for_update() -> Option<UpdateInfo> {
    let current_version = env!("CARGO_PKG_VERSION");

    // Fetch release info from GitHub
    let client = reqwest::Client::builder()
        .user_agent(format!("cuttlefish-tui/{current_version}"))
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .ok()?;

    let url = "https://api.github.com/repos/JackTYM/cuttlefish-rs/releases/tags/tui-latest";
    let response = client.get(url).send().await.ok()?;

    if !response.status().is_success() {
        return None;
    }

    let release: serde_json::Value = response.json().await.ok()?;

    // Extract version from release name (e.g., "TUI Latest (v0.1.0)" -> "0.1.0")
    let name = release["name"].as_str()?;
    let latest_version = extract_version(name)?;

    // Compare versions
    if !is_newer_version(current_version, &latest_version) {
        return None;
    }

    // Find download URL for current platform
    let download_url = find_download_url(&release)?;

    Some(UpdateInfo {
        current_version: current_version.to_string(),
        latest_version,
        download_url,
    })
}

/// Download and apply an update.
pub async fn apply_update(info: &UpdateInfo) -> anyhow::Result<()> {
    let client = reqwest::Client::builder()
        .user_agent(format!("cuttlefish-tui/{}", info.current_version))
        .timeout(std::time::Duration::from_secs(300))
        .build()?;

    println!("Downloading cuttlefish-tui v{}...", info.latest_version);

    let response = client.get(&info.download_url).send().await?;

    if !response.status().is_success() {
        anyhow::bail!("Failed to download update: HTTP {}", response.status());
    }

    let bytes = response.bytes().await?;

    // Get current executable path
    let current_exe = env::current_exe()?;
    let exe_dir = current_exe.parent().unwrap_or(std::path::Path::new("."));

    // Create temp file for download
    let temp_path = exe_dir.join(".cuttlefish-tui-update.tmp");

    // Extract based on file type
    if info.download_url.ends_with(".tar.gz") {
        extract_tar_gz(&bytes, &temp_path)?;
    } else if info.download_url.ends_with(".zip") {
        extract_zip(&bytes, &temp_path)?;
    } else {
        anyhow::bail!("Unknown archive format");
    }

    // Replace current binary
    let backup_path = exe_dir.join(".cuttlefish-tui-backup");

    // Backup current binary
    if current_exe.exists() {
        fs::rename(&current_exe, &backup_path)?;
    }

    // Move new binary into place
    if let Err(e) = fs::rename(&temp_path, &current_exe) {
        // Restore backup on failure
        if backup_path.exists() {
            let _ = fs::rename(&backup_path, &current_exe);
        }
        anyhow::bail!("Failed to install update: {e}");
    }

    // Set executable permissions on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&current_exe)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&current_exe, perms)?;
    }

    // Clean up backup
    let _ = fs::remove_file(&backup_path);

    println!("Successfully updated to v{}", info.latest_version);
    println!("Please restart cuttlefish-tui to use the new version.");

    Ok(())
}

/// Extract version string from release name.
fn extract_version(name: &str) -> Option<String> {
    // Match pattern like "TUI Latest (v0.1.0)" or "v0.1.0"
    let re = regex::Regex::new(r"v?(\d+\.\d+\.\d+)").ok()?;
    let caps = re.captures(name)?;
    Some(caps.get(1)?.as_str().to_string())
}

/// Check if latest version is newer than current.
fn is_newer_version(current: &str, latest: &str) -> bool {
    let parse = |v: &str| -> Option<(u32, u32, u32)> {
        let parts: Vec<&str> = v.split('.').collect();
        if parts.len() != 3 {
            return None;
        }
        Some((
            parts[0].parse().ok()?,
            parts[1].parse().ok()?,
            parts[2].parse().ok()?,
        ))
    };

    let Some(current) = parse(current) else {
        return false;
    };
    let Some(latest) = parse(latest) else {
        return false;
    };

    latest > current
}

/// Find the download URL for the current platform.
fn find_download_url(release: &serde_json::Value) -> Option<String> {
    let assets = release["assets"].as_array()?;

    let target = current_target();

    for asset in assets {
        let name = asset["name"].as_str()?;
        if name.contains(&target) && !name.ends_with(".sha256") {
            return Some(asset["browser_download_url"].as_str()?.to_string());
        }
    }

    None
}

/// Get the target platform string.
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

    format!("{os}-{arch}")
}

/// Extract a tar.gz archive.
fn extract_tar_gz(data: &[u8], dest: &PathBuf) -> anyhow::Result<()> {
    use flate2::read::GzDecoder;
    use tar::Archive;

    let decoder = GzDecoder::new(data);
    let mut archive = Archive::new(decoder);

    for entry in archive.entries()? {
        let mut entry = entry?;
        let path = entry.path()?;

        // Look for the binary
        if path.file_name().is_some_and(|n| n == "cuttlefish-tui") {
            let mut file = fs::File::create(dest)?;
            std::io::copy(&mut entry, &mut file)?;
            return Ok(());
        }
    }

    anyhow::bail!("Binary not found in archive")
}

/// Extract a zip archive.
fn extract_zip(data: &[u8], dest: &PathBuf) -> anyhow::Result<()> {
    use std::io::Cursor;
    use zip::ZipArchive;

    let cursor = Cursor::new(data);
    let mut archive = ZipArchive::new(cursor)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let name = file.name();

        // Look for the binary
        if name.contains("cuttlefish-tui") && !name.ends_with('/') {
            let mut out = fs::File::create(dest)?;
            std::io::copy(&mut file, &mut out)?;
            return Ok(());
        }
    }

    anyhow::bail!("Binary not found in archive")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_version() {
        assert_eq!(
            extract_version("TUI Latest (v0.1.0)"),
            Some("0.1.0".to_string())
        );
        assert_eq!(extract_version("v1.2.3"), Some("1.2.3".to_string()));
        assert_eq!(extract_version("0.1.0"), Some("0.1.0".to_string()));
    }

    #[test]
    fn test_is_newer_version() {
        assert!(is_newer_version("0.1.0", "0.1.1"));
        assert!(is_newer_version("0.1.0", "0.2.0"));
        assert!(is_newer_version("0.1.0", "1.0.0"));
        assert!(!is_newer_version("0.1.1", "0.1.0"));
        assert!(!is_newer_version("0.1.0", "0.1.0"));
    }

    #[test]
    fn test_current_target() {
        let target = current_target();
        assert!(target.contains('-'));
    }
}
