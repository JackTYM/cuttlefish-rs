//! GitHub API client for repository and workflow management.
//!
//! Provides a [`GitHubClient`] that wraps the GitHub REST API with PAT
//! authentication, supporting repository creation, pull request management,
//! and GitHub Actions workflow monitoring.

use cuttlefish_core::error::VcsError;
use serde::{Deserialize, Serialize};
use tracing::{debug, instrument};

/// Base URL for the GitHub REST API.
const GITHUB_API_BASE: &str = "https://api.github.com";

/// GitHub API version header value.
const GITHUB_API_VERSION: &str = "2022-11-28";

/// Result type for GitHub operations.
type GitHubResult<T> = Result<T, VcsError>;

/// GitHub repository creation request.
#[derive(Debug, Serialize)]
pub struct CreateRepoRequest {
    /// Repository name.
    pub name: String,
    /// Repository description.
    pub description: String,
    /// Whether the repository is private.
    pub private: bool,
    /// Initialize with README.
    pub auto_init: bool,
}

/// GitHub repository information.
#[derive(Debug, Deserialize)]
pub struct RepoInfo {
    /// Full repository name (owner/repo).
    pub full_name: String,
    /// Clone URL (HTTPS).
    pub clone_url: String,
    /// HTML URL for browser access.
    pub html_url: String,
    /// Default branch name.
    pub default_branch: String,
}

/// GitHub workflow run information.
#[derive(Debug, Clone, Deserialize)]
pub struct WorkflowRun {
    /// Run ID.
    pub id: u64,
    /// Workflow name.
    pub name: Option<String>,
    /// Run status (queued, in_progress, completed).
    pub status: String,
    /// Conclusion (success, failure, cancelled, etc.). None if not completed.
    pub conclusion: Option<String>,
    /// HTML URL.
    pub html_url: String,
    /// Creation timestamp.
    pub created_at: String,
}

/// GitHub pull request creation request.
#[derive(Debug, Serialize)]
pub struct CreatePrRequest {
    /// PR title.
    pub title: String,
    /// PR body/description.
    pub body: String,
    /// Head branch (source).
    pub head: String,
    /// Base branch (target).
    pub base: String,
}

/// GitHub pull request information.
#[derive(Debug, Deserialize)]
pub struct PrInfo {
    /// PR number.
    pub number: u64,
    /// PR title.
    pub title: String,
    /// HTML URL.
    pub html_url: String,
    /// PR state (open, closed).
    pub state: String,
}

/// Client for GitHub REST API operations.
///
/// Authenticates via a Personal Access Token (PAT) and scopes all
/// operations to a specific owner (user or organization).
pub struct GitHubClient {
    client: reqwest::Client,
    token: String,
    owner: String,
}

impl GitHubClient {
    /// Create a new GitHub client for the given owner (user or org).
    ///
    /// # Arguments
    /// * `token` — Personal access token (classic or fine-grained)
    /// * `owner` — GitHub user or organization name
    pub fn new(token: impl Into<String>, owner: impl Into<String>) -> Self {
        let client = reqwest::Client::builder()
            .user_agent("cuttlefish-rs/0.1.0")
            .build()
            .expect("Failed to build HTTP client — TLS backend unavailable");
        Self {
            client,
            token: token.into(),
            owner: owner.into(),
        }
    }

    /// Build the Bearer authorization header value.
    fn auth_header(&self) -> String {
        format!("Bearer {}", self.token)
    }

    /// Create a new repository under the owner.
    #[instrument(skip(self))]
    pub async fn create_repo(&self, request: &CreateRepoRequest) -> GitHubResult<RepoInfo> {
        debug!("Creating repo {}/{}", self.owner, request.name);

        let url = format!("{GITHUB_API_BASE}/user/repos");
        let response = self
            .client
            .post(&url)
            .header("Authorization", self.auth_header())
            .header("X-GitHub-Api-Version", GITHUB_API_VERSION)
            .json(request)
            .send()
            .await
            .map_err(|e| VcsError(format!("GitHub API request failed: {e}")))?;

        self.handle_response(response).await
    }

    /// Get repository information.
    pub async fn get_repo(&self, repo: &str) -> GitHubResult<RepoInfo> {
        let url = format!("{GITHUB_API_BASE}/repos/{}/{repo}", self.owner);
        let response = self
            .client
            .get(&url)
            .header("Authorization", self.auth_header())
            .header("X-GitHub-Api-Version", GITHUB_API_VERSION)
            .send()
            .await
            .map_err(|e| VcsError(format!("GitHub API request failed: {e}")))?;

        self.handle_response(response).await
    }

    /// Create a pull request.
    #[instrument(skip(self))]
    pub async fn create_pull_request(
        &self,
        repo: &str,
        request: &CreatePrRequest,
    ) -> GitHubResult<PrInfo> {
        debug!("Creating PR in {}/{repo}: {}", self.owner, request.title);

        let url = format!("{GITHUB_API_BASE}/repos/{}/{repo}/pulls", self.owner);
        let response = self
            .client
            .post(&url)
            .header("Authorization", self.auth_header())
            .header("X-GitHub-Api-Version", GITHUB_API_VERSION)
            .json(request)
            .send()
            .await
            .map_err(|e| VcsError(format!("GitHub API request failed: {e}")))?;

        self.handle_response(response).await
    }

    /// List recent workflow runs for a repository.
    pub async fn list_workflow_runs(
        &self,
        repo: &str,
        limit: u32,
    ) -> GitHubResult<Vec<WorkflowRun>> {
        let url = format!(
            "{GITHUB_API_BASE}/repos/{}/{repo}/actions/runs?per_page={limit}",
            self.owner
        );
        let response = self
            .client
            .get(&url)
            .header("Authorization", self.auth_header())
            .header("X-GitHub-Api-Version", GITHUB_API_VERSION)
            .send()
            .await
            .map_err(|e| VcsError(format!("GitHub API request failed: {e}")))?;

        #[derive(Deserialize)]
        struct RunsResponse {
            workflow_runs: Vec<WorkflowRun>,
        }

        let runs_resp: RunsResponse = self.handle_response(response).await?;
        Ok(runs_resp.workflow_runs)
    }

    /// Get a specific workflow run by ID.
    pub async fn get_workflow_run(&self, repo: &str, run_id: u64) -> GitHubResult<WorkflowRun> {
        let url = format!(
            "{GITHUB_API_BASE}/repos/{}/{repo}/actions/runs/{run_id}",
            self.owner
        );
        let response = self
            .client
            .get(&url)
            .header("Authorization", self.auth_header())
            .header("X-GitHub-Api-Version", GITHUB_API_VERSION)
            .send()
            .await
            .map_err(|e| VcsError(format!("GitHub API request failed: {e}")))?;

        self.handle_response(response).await
    }

    /// Get the download URL for workflow run logs.
    ///
    /// GitHub responds with a 3xx redirect to a signed URL for the log archive.
    /// This method captures the redirect `Location` header instead of following it,
    /// returning the signed URL that can be used to download logs.
    pub async fn get_workflow_run_logs_url(
        &self,
        repo: &str,
        run_id: u64,
    ) -> GitHubResult<String> {
        let url = format!(
            "{GITHUB_API_BASE}/repos/{}/{repo}/actions/runs/{run_id}/logs",
            self.owner
        );

        let no_redirect_client = reqwest::Client::builder()
            .user_agent("cuttlefish-rs/0.1.0")
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|e| VcsError(format!("Failed to build HTTP client: {e}")))?;

        let resp = no_redirect_client
            .get(&url)
            .header("Authorization", self.auth_header())
            .header("X-GitHub-Api-Version", GITHUB_API_VERSION)
            .send()
            .await
            .map_err(|e| VcsError(format!("GitHub API request failed: {e}")))?;

        if resp.status().is_redirection() {
            resp.headers()
                .get("location")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string())
                .ok_or_else(|| {
                    VcsError("No Location header in redirect response".to_string())
                })
        } else {
            Err(VcsError(format!(
                "Expected redirect, got {}",
                resp.status()
            )))
        }
    }

    /// Handle an API response — check status and deserialize JSON.
    async fn handle_response<T: for<'de> serde::Deserialize<'de>>(
        &self,
        response: reqwest::Response,
    ) -> GitHubResult<T> {
        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(VcsError(format!("GitHub API error {status}: {body}")));
        }
        response
            .json::<T>()
            .await
            .map_err(|e| VcsError(format!("Failed to parse GitHub response: {e}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_github_client_creation() {
        let client = GitHubClient::new("test-token", "test-owner");
        assert_eq!(client.token, "test-token");
        assert_eq!(client.owner, "test-owner");
    }

    #[test]
    fn test_auth_header_format() {
        let client = GitHubClient::new("ghp_abc123", "myorg");
        let header = client.auth_header();
        assert_eq!(header, "Bearer ghp_abc123");
    }

    #[test]
    fn test_workflow_run_is_failed() {
        let run = WorkflowRun {
            id: 123,
            name: Some("CI".to_string()),
            status: "completed".to_string(),
            conclusion: Some("failure".to_string()),
            html_url: "https://github.com".to_string(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
        };
        assert_eq!(run.conclusion.as_deref(), Some("failure"));
        assert_eq!(run.status, "completed");
    }

    #[test]
    fn test_workflow_run_in_progress() {
        let run = WorkflowRun {
            id: 456,
            name: None,
            status: "in_progress".to_string(),
            conclusion: None,
            html_url: "https://github.com".to_string(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
        };
        assert!(run.conclusion.is_none());
        assert_eq!(run.status, "in_progress");
    }

    #[test]
    fn test_create_repo_request_serializes() {
        let req = CreateRepoRequest {
            name: "my-app".to_string(),
            description: "A test app".to_string(),
            private: false,
            auto_init: true,
        };
        let json = serde_json::to_string(&req).expect("serialize");
        assert!(json.contains("my-app"));
        assert!(json.contains("auto_init"));
    }
}
