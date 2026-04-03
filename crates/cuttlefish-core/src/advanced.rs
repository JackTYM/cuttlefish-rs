//! Advanced platform features: self-update, template system, and model routing.

use std::collections::HashMap;
use std::path::Path;
use tracing::{debug, info};

// ============= SELF-UPDATE (Task 38) =============

/// Release information from GitHub.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct ReleaseInfo {
    /// Version tag (e.g., "v0.2.0").
    pub tag_name: String,
    /// Release description.
    pub body: Option<String>,
    /// Download assets.
    pub assets: Vec<ReleaseAsset>,
}

/// A downloadable release asset.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct ReleaseAsset {
    /// Asset name.
    pub name: String,
    /// Download URL.
    pub browser_download_url: String,
}

/// Check if a version string is newer than the current version.
///
/// Compares semantic versions (v1.2.3 format).
pub fn is_newer_version(current: &str, latest: &str) -> bool {
    let parse = |v: &str| -> (u32, u32, u32) {
        let v = v.trim_start_matches('v');
        let parts: Vec<u32> = v.split('.').filter_map(|s| s.parse().ok()).collect();
        (
            parts.first().copied().unwrap_or(0),
            parts.get(1).copied().unwrap_or(0),
            parts.get(2).copied().unwrap_or(0),
        )
    };
    parse(latest) > parse(current)
}

// ============= TEMPLATE SYSTEM (Task 40) =============

/// A project template loaded from a `.md` file.
#[derive(Debug, Clone)]
pub struct ProjectTemplate {
    /// Template name (e.g., "nuxt-cloudflare").
    pub name: String,
    /// Natural language description for the agent.
    pub content: String,
    /// The language/stack (e.g., "typescript", "rust").
    pub language: String,
    /// Docker base image to use for this template.
    pub docker_image: String,
}

/// Load all templates from a directory of `.md` files.
///
/// Each `.md` file's stem becomes the template name.
/// The first line is expected to be `# Template: <name>`.
/// The full content is used as the agent instruction.
pub fn load_templates_from_dir(dir: &Path) -> Vec<ProjectTemplate> {
    let mut templates = Vec::new();

    let Ok(entries) = std::fs::read_dir(dir) else {
        debug!("Template directory not found: {}", dir.display());
        return templates;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }

        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        // Determine language from template name
        let language =
            if name.contains("nuxt") || name.contains("node") || name.contains("typescript") {
                "typescript"
            } else if name.contains("python") || name.contains("fastapi") {
                "python"
            } else if name.contains("rust") || name.contains("axum") {
                "rust"
            } else if name.contains("go") {
                "go"
            } else {
                "generic"
            };

        let docker_image = match language {
            "typescript" => "node:22-slim",
            "python" => "python:3.12-slim",
            "rust" => "rust:1.82-slim",
            "go" => "golang:1.22-bookworm",
            _ => "ubuntu:22.04",
        };

        templates.push(ProjectTemplate {
            name,
            content,
            language: language.to_string(),
            docker_image: docker_image.to_string(),
        });
    }

    info!(
        "Loaded {} templates from {}",
        templates.len(),
        dir.display()
    );
    templates
}

/// Get a template by name, returning None if not found.
pub fn find_template<'a>(
    templates: &'a [ProjectTemplate],
    name: &str,
) -> Option<&'a ProjectTemplate> {
    templates.iter().find(|t| t.name == name)
}

// ============= MODEL ROUTING (Task 43) =============

/// Configuration for a single model (provider + model ID).
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct ModelConfig {
    /// Provider name (e.g., "bedrock", "claude-oauth").
    pub provider: String,
    /// Model ID (e.g., "anthropic.claude-sonnet-4-6-20260101-v1:0").
    pub model_id: String,
    /// Temperature for sampling (0.0-1.0).
    #[serde(default)]
    pub temperature: f32,
    /// Maximum tokens to generate.
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,
}

fn default_max_tokens() -> u32 {
    4096
}

/// Agent role to category mapping.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct AgentRoutingConfig {
    /// Map of category name → ModelConfig.
    pub categories: HashMap<String, ModelConfig>,
    /// Map of agent role name → category name.
    pub agents: HashMap<String, String>,
}

impl AgentRoutingConfig {
    /// Resolve an agent role to its ModelConfig.
    pub fn resolve_agent(&self, role: &str) -> Option<&ModelConfig> {
        let category = self.agents.get(role)?;
        self.categories.get(category)
    }

    /// Resolve a category directly to its ModelConfig.
    pub fn resolve_category(&self, category: &str) -> Option<&ModelConfig> {
        self.categories.get(category)
    }

    /// Create a default routing config using Bedrock Claude models.
    pub fn default_bedrock() -> Self {
        let mut categories = HashMap::new();
        categories.insert(
            "deep".to_string(),
            ModelConfig {
                provider: "bedrock".to_string(),
                model_id: "anthropic.claude-sonnet-4-6-20260101-v1:0".to_string(),
                temperature: 0.1,
                max_tokens: 4096,
            },
        );
        categories.insert(
            "quick".to_string(),
            ModelConfig {
                provider: "bedrock".to_string(),
                model_id: "anthropic.claude-haiku-4-5-20260101-v1:0".to_string(),
                temperature: 0.3,
                max_tokens: 2048,
            },
        );
        categories.insert(
            "unspecified-high".to_string(),
            ModelConfig {
                provider: "bedrock".to_string(),
                model_id: "anthropic.claude-sonnet-4-6-20260101-v1:0".to_string(),
                temperature: 0.2,
                max_tokens: 4096,
            },
        );

        let mut agents = HashMap::new();
        agents.insert("orchestrator".to_string(), "deep".to_string());
        agents.insert("coder".to_string(), "deep".to_string());
        agents.insert("critic".to_string(), "unspecified-high".to_string());
        agents.insert("planner".to_string(), "deep".to_string());

        Self { categories, agents }
    }
}

// ============= GITHUB ACTIONS MONITORING (Task 39) =============

/// Status of a GitHub Actions workflow run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkflowStatus {
    /// Queued or waiting.
    Queued,
    /// Currently running.
    InProgress,
    /// Completed successfully.
    Success,
    /// Completed with failure.
    Failure,
    /// Cancelled.
    Cancelled,
    /// Unknown status.
    Unknown(String),
}

impl WorkflowStatus {
    /// Parse from GitHub API status/conclusion strings.
    pub fn from_api(status: &str, conclusion: Option<&str>) -> Self {
        match status {
            "queued" => Self::Queued,
            "in_progress" => Self::InProgress,
            "completed" => match conclusion {
                Some("success") => Self::Success,
                Some("failure") | Some("timed_out") => Self::Failure,
                Some("cancelled") => Self::Cancelled,
                Some(other) => Self::Unknown(other.to_string()),
                None => Self::Unknown("completed without conclusion".to_string()),
            },
            other => Self::Unknown(other.to_string()),
        }
    }

    /// Returns true if this is a terminal (non-running) state.
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            Self::Success | Self::Failure | Self::Cancelled | Self::Unknown(_)
        )
    }
}

// ============= GITHUB APP AUTH (Task 42) =============

/// JWT claims for GitHub App authentication.
#[derive(Debug, serde::Serialize)]
pub struct GithubAppClaims {
    /// Issued at (Unix timestamp).
    pub iat: i64,
    /// Expiration (Unix timestamp).
    pub exp: i64,
    /// Issuer (App ID as string).
    pub iss: String,
}

impl GithubAppClaims {
    /// Create claims valid for 10 minutes from now.
    pub fn new(app_id: u64) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        Self {
            iat: now - 60,  // 1 min in the past for clock skew
            exp: now + 600, // 10 minutes
            iss: app_id.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Version comparison tests ---
    #[test]
    fn test_newer_version() {
        assert!(is_newer_version("v0.1.0", "v0.2.0"));
        assert!(is_newer_version("0.1.0", "0.1.1"));
        assert!(!is_newer_version("v0.2.0", "v0.1.0"));
        assert!(!is_newer_version("v1.0.0", "v1.0.0")); // same = not newer
    }

    // --- Template loading tests ---
    #[test]
    fn test_find_template_missing() {
        let templates = vec![];
        assert!(find_template(&templates, "nonexistent").is_none());
    }

    #[test]
    fn test_find_template_found() {
        let templates = vec![ProjectTemplate {
            name: "nuxt-cloudflare".to_string(),
            content: "# Nuxt template".to_string(),
            language: "typescript".to_string(),
            docker_image: "node:22-slim".to_string(),
        }];
        assert!(find_template(&templates, "nuxt-cloudflare").is_some());
    }

    // --- Model routing tests ---
    #[test]
    fn test_default_bedrock_routing() {
        let config = AgentRoutingConfig::default_bedrock();
        assert!(config.resolve_agent("orchestrator").is_some());
        assert!(config.resolve_agent("coder").is_some());
        assert!(config.resolve_category("quick").is_some());
    }

    #[test]
    fn test_resolve_unknown_agent_returns_none() {
        let config = AgentRoutingConfig::default_bedrock();
        assert!(config.resolve_agent("nonexistent_agent").is_none());
    }

    // --- Workflow status tests ---
    #[test]
    fn test_workflow_status_parsing() {
        assert_eq!(
            WorkflowStatus::from_api("queued", None),
            WorkflowStatus::Queued
        );
        assert_eq!(
            WorkflowStatus::from_api("in_progress", None),
            WorkflowStatus::InProgress
        );
        assert_eq!(
            WorkflowStatus::from_api("completed", Some("success")),
            WorkflowStatus::Success
        );
        assert_eq!(
            WorkflowStatus::from_api("completed", Some("failure")),
            WorkflowStatus::Failure
        );
    }

    #[test]
    fn test_terminal_states() {
        assert!(WorkflowStatus::Success.is_terminal());
        assert!(WorkflowStatus::Failure.is_terminal());
        assert!(!WorkflowStatus::InProgress.is_terminal());
        assert!(!WorkflowStatus::Queued.is_terminal());
    }

    // --- GitHub App claims ---
    #[test]
    fn test_github_app_claims_expiry() {
        let claims = GithubAppClaims::new(12345);
        assert_eq!(claims.iss, "12345");
        assert!(claims.exp > claims.iat);
        assert!(claims.exp - claims.iat >= 600);
    }
}
