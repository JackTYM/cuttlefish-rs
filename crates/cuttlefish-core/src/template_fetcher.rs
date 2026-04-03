//! Template fetcher for downloading templates from remote URLs (GitHub, etc.).
//!
//! This module provides functionality to fetch template files from remote sources,
//! parse their YAML frontmatter, and cache them locally.
//!
//! # Example
//!
//! ```no_run
//! use cuttlefish_core::TemplateFetcher;
//! use std::path::Path;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let fetcher = TemplateFetcher::new(Path::new("/tmp/cache"));
//!
//! // Fetch a template from GitHub
//! let template = fetcher.fetch(
//!     "https://raw.githubusercontent.com/owner/repo/main/template.md"
//! ).await?;
//!
//! println!("Loaded template: {}", template.manifest.name);
//! # Ok(())
//! # }
//! ```

use std::hash::{Hash, Hasher};
use std::path::PathBuf;

use reqwest::Client;

use crate::template_manifest::{TemplateError, parse_manifest};
use crate::template_registry::{LoadedTemplate, TemplateSource};

/// Fetches templates from remote URLs (GitHub, etc.).
///
/// Supports both raw GitHub URLs and regular blob URLs (which are
/// automatically converted to raw URLs). Fetched templates are
/// cached locally to avoid repeated network requests.
pub struct TemplateFetcher {
    client: Client,
    cache_dir: PathBuf,
    github_token: Option<String>,
}

impl TemplateFetcher {
    /// Create a new fetcher with the specified cache directory.
    ///
    /// If the `GITHUB_TOKEN` environment variable is set, it will be used
    /// for authenticated requests (higher rate limits).
    ///
    /// # Arguments
    ///
    /// * `cache_dir` - Directory where fetched templates will be cached
    #[must_use]
    pub fn new(cache_dir: impl Into<PathBuf>) -> Self {
        Self {
            client: Client::new(),
            cache_dir: cache_dir.into(),
            github_token: std::env::var("GITHUB_TOKEN").ok(),
        }
    }

    /// Create a new fetcher with an explicit GitHub token.
    ///
    /// # Arguments
    ///
    /// * `cache_dir` - Directory where fetched templates will be cached
    /// * `github_token` - GitHub personal access token for authenticated requests
    #[must_use]
    pub fn with_token(cache_dir: impl Into<PathBuf>, github_token: Option<String>) -> Self {
        Self {
            client: Client::new(),
            cache_dir: cache_dir.into(),
            github_token,
        }
    }

    /// Fetch a template from a URL.
    ///
    /// Supports:
    /// - Raw GitHub URLs: `https://raw.githubusercontent.com/owner/repo/branch/path.md`
    /// - GitHub blob URLs: `https://github.com/owner/repo/blob/branch/path.md` (converts to raw)
    ///
    /// Fetched templates are cached locally. Subsequent requests for the same URL
    /// will return the cached version.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL to fetch the template from
    ///
    /// # Returns
    ///
    /// A `LoadedTemplate` containing the parsed manifest and content.
    ///
    /// # Errors
    ///
    /// Returns `TemplateError::NotFound` if the URL returns a non-success HTTP status.
    /// Returns `TemplateError::Io` for network errors.
    /// Returns `TemplateError::InvalidYaml` if the template has invalid frontmatter.
    pub async fn fetch(&self, url: &str) -> Result<LoadedTemplate, TemplateError> {
        let raw_url = self.normalize_github_url(url);

        // Check cache first
        let cache_key = self.url_to_cache_key(&raw_url);
        let cache_path = self.cache_dir.join(&cache_key);

        if cache_path.exists() {
            let content = std::fs::read_to_string(&cache_path)?;
            let (manifest, body) = parse_manifest(&content)?;
            return Ok(LoadedTemplate {
                manifest,
                content: body,
                source: TemplateSource::Remote(url.to_string()),
            });
        }

        // Fetch from remote
        let mut request = self.client.get(&raw_url);
        if let Some(ref token) = self.github_token {
            request = request.header("Authorization", format!("token {token}"));
        }

        let response = request
            .send()
            .await
            .map_err(|e| TemplateError::Io(std::io::Error::other(format!("HTTP error: {e}"))))?;

        if !response.status().is_success() {
            let status = response.status();
            return Err(TemplateError::NotFound(format!("HTTP {status}: {url}")));
        }

        let content = response
            .text()
            .await
            .map_err(|e| TemplateError::Io(std::io::Error::other(format!("Read error: {e}"))))?;

        // Parse and validate
        let (manifest, body) = parse_manifest(&content)?;

        // Cache for future use
        if let Some(parent) = cache_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&cache_path, &content)?;

        Ok(LoadedTemplate {
            manifest,
            content: body,
            source: TemplateSource::Remote(url.to_string()),
        })
    }

    /// Convert GitHub blob URL to raw URL.
    ///
    /// Transforms URLs like:
    /// `https://github.com/owner/repo/blob/branch/path.md`
    /// to:
    /// `https://raw.githubusercontent.com/owner/repo/branch/path.md`
    ///
    /// If the URL is already a raw URL or not a GitHub URL, it's returned unchanged.
    #[must_use]
    pub fn normalize_github_url(&self, url: &str) -> String {
        if url.contains("github.com") && url.contains("/blob/") {
            url.replace("github.com", "raw.githubusercontent.com")
                .replace("/blob/", "/")
        } else {
            url.to_string()
        }
    }

    /// Generate a deterministic cache key from a URL.
    ///
    /// Uses a hash of the URL to create a filename-safe cache key.
    #[must_use]
    pub fn url_to_cache_key(&self, url: &str) -> String {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        url.hash(&mut hasher);
        format!("{:x}.md", hasher.finish())
    }

    /// Clear the template cache.
    ///
    /// Removes the entire cache directory and all cached templates.
    ///
    /// # Errors
    ///
    /// Returns `TemplateError::Io` if the cache directory cannot be removed.
    pub fn clear_cache(&self) -> Result<(), TemplateError> {
        if self.cache_dir.exists() {
            std::fs::remove_dir_all(&self.cache_dir)?;
        }
        Ok(())
    }

    /// Get the cache directory path.
    #[must_use]
    pub fn cache_dir(&self) -> &PathBuf {
        &self.cache_dir
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_normalize_github_url_blob_to_raw() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let fetcher = TemplateFetcher::new(temp_dir.path());

        let blob_url = "https://github.com/owner/repo/blob/main/templates/test.md";
        let raw_url = fetcher.normalize_github_url(blob_url);

        assert_eq!(
            raw_url,
            "https://raw.githubusercontent.com/owner/repo/main/templates/test.md"
        );
    }

    #[test]
    fn test_normalize_github_url_already_raw() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let fetcher = TemplateFetcher::new(temp_dir.path());

        let raw_url = "https://raw.githubusercontent.com/owner/repo/main/templates/test.md";
        let result = fetcher.normalize_github_url(raw_url);

        assert_eq!(result, raw_url);
    }

    #[test]
    fn test_normalize_github_url_non_github() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let fetcher = TemplateFetcher::new(temp_dir.path());

        let url = "https://example.com/templates/test.md";
        let result = fetcher.normalize_github_url(url);

        assert_eq!(result, url);
    }

    #[test]
    fn test_url_to_cache_key_deterministic() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let fetcher = TemplateFetcher::new(temp_dir.path());

        let url = "https://raw.githubusercontent.com/owner/repo/main/template.md";
        let key1 = fetcher.url_to_cache_key(url);
        let key2 = fetcher.url_to_cache_key(url);

        assert_eq!(key1, key2);
        assert!(key1.ends_with(".md"));
    }

    #[test]
    fn test_url_to_cache_key_different_urls() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let fetcher = TemplateFetcher::new(temp_dir.path());

        let url1 = "https://raw.githubusercontent.com/owner/repo/main/template1.md";
        let url2 = "https://raw.githubusercontent.com/owner/repo/main/template2.md";
        let key1 = fetcher.url_to_cache_key(url1);
        let key2 = fetcher.url_to_cache_key(url2);

        assert_ne!(key1, key2);
    }

    #[tokio::test]
    async fn test_fetch_nonexistent_url() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let fetcher = TemplateFetcher::new(temp_dir.path());

        let result = fetcher
            .fetch("https://raw.githubusercontent.com/nonexistent-owner-12345/nonexistent-repo-67890/main/nonexistent.md")
            .await;

        assert!(result.is_err());
        match result {
            Err(TemplateError::NotFound(msg)) => {
                assert!(msg.contains("404") || msg.contains("HTTP"));
            }
            Err(TemplateError::Io(_)) => {
                // Network error is also acceptable
            }
            other => panic!("Expected NotFound or Io error, got: {other:?}"),
        }
    }

    #[test]
    fn test_clear_cache_empty() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let fetcher = TemplateFetcher::new(temp_dir.path().join("cache"));

        // Clear non-existent cache should succeed
        let result = fetcher.clear_cache();
        assert!(result.is_ok());
    }

    #[test]
    fn test_clear_cache_with_content() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let cache_dir = temp_dir.path().join("cache");
        std::fs::create_dir_all(&cache_dir).expect("Failed to create cache dir");
        std::fs::write(cache_dir.join("test.md"), "test content").expect("Failed to write file");

        let fetcher = TemplateFetcher::new(&cache_dir);
        let result = fetcher.clear_cache();

        assert!(result.is_ok());
        assert!(!cache_dir.exists());
    }

    #[test]
    fn test_with_token_constructor() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let fetcher = TemplateFetcher::with_token(temp_dir.path(), Some("test-token".to_string()));

        assert_eq!(fetcher.github_token, Some("test-token".to_string()));
    }

    #[test]
    fn test_cache_dir_accessor() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let fetcher = TemplateFetcher::new(temp_dir.path());

        assert_eq!(fetcher.cache_dir(), temp_dir.path());
    }

    #[tokio::test]
    async fn test_fetch_from_cache() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let fetcher = TemplateFetcher::new(temp_dir.path());

        // Create a cached template file
        let url = "https://example.com/cached-template.md";
        let cache_key = fetcher.url_to_cache_key(&fetcher.normalize_github_url(url));
        let cache_path = temp_dir.path().join(&cache_key);

        let template_content = r#"---
name: cached-template
description: A cached template
language: rust
docker_image: rust:latest
---
# Cached Template

This is the body.
"#;
        std::fs::write(&cache_path, template_content).expect("Failed to write cache file");

        // Fetch should use cache
        let result = fetcher.fetch(url).await;
        assert!(result.is_ok());

        let template = result.expect("Should load from cache");
        assert_eq!(template.manifest.name, "cached-template");
        assert_eq!(template.manifest.language, "rust");
        assert!(template.content.contains("# Cached Template"));
        assert_eq!(template.source, TemplateSource::Remote(url.to_string()));
    }

    /// Integration test that fetches a real template from GitHub.
    /// Marked as `ignore` to avoid CI failures due to rate limiting.
    #[tokio::test]
    #[ignore]
    async fn test_fetch_real_github_template() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let fetcher = TemplateFetcher::new(temp_dir.path());

        // Use a well-known public file that's unlikely to change
        let result = fetcher
            .fetch("https://raw.githubusercontent.com/rust-lang/rust/master/README.md")
            .await;

        // This should fail with InvalidYaml since README.md doesn't have YAML frontmatter
        // But it proves the network request succeeded
        match result {
            Ok(_) => panic!("Expected error for README without frontmatter"),
            Err(TemplateError::MissingBody | TemplateError::InvalidYaml(_)) => {
                // Expected - file doesn't have valid template frontmatter
            }
            Err(e) => panic!("Unexpected error type: {e}"),
        }
    }
}
