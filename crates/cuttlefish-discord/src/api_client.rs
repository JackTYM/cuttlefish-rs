//! HTTP client for communicating with the Cuttlefish API.
//!
//! Provides typed methods for all API endpoints needed by the Discord bot:
//! - Project creation and status
//! - Activity logs
//! - Action approval/rejection

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Configuration for the API client.
#[derive(Debug, Clone)]
pub struct ApiClientConfig {
    /// Base URL of the Cuttlefish API (e.g., "http://localhost:8080").
    pub base_url: String,
    /// API key for authentication.
    pub api_key: String,
    /// Request timeout in seconds.
    pub timeout_secs: u64,
}

impl Default for ApiClientConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:8080".to_string(),
            api_key: String::new(),
            timeout_secs: 30,
        }
    }
}

/// HTTP client for the Cuttlefish API.
#[derive(Debug, Clone)]
pub struct ApiClient {
    config: ApiClientConfig,
    client: reqwest::Client,
}

/// Error type for API client operations.
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    /// HTTP request failed.
    #[error("HTTP request failed: {0}")]
    Request(#[from] reqwest::Error),

    /// API returned an error response.
    #[error("API error ({status}): {message}")]
    ApiResponse {
        /// HTTP status code.
        status: u16,
        /// Error message from API.
        message: String,
    },

    /// Failed to parse API response.
    #[error("Failed to parse response: {0}")]
    Parse(String),

    /// Client not configured.
    #[error("API client not configured: {0}")]
    NotConfigured(String),
}

/// Request body for creating a project.
#[derive(Debug, Serialize)]
pub struct CreateProjectRequest {
    /// Project name.
    pub name: String,
    /// Project description.
    pub description: String,
    /// Template name (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template: Option<String>,
}

/// Response from project creation.
#[derive(Debug, Deserialize)]
pub struct ProjectResponse {
    /// Project ID.
    pub id: String,
    /// Project name.
    pub name: String,
    /// Project status.
    pub status: String,
    /// Template used (if any).
    pub template: Option<String>,
}

/// Project status with agent information.
#[derive(Debug, Deserialize)]
pub struct ProjectStatus {
    /// Project ID.
    pub id: String,
    /// Project name.
    pub name: String,
    /// Current status.
    pub status: String,
    /// Active agents.
    #[serde(default)]
    pub active_agents: Vec<AgentInfo>,
    /// Last activity timestamp.
    pub last_activity: Option<String>,
    /// Current task description.
    pub current_task: Option<String>,
}

/// Information about an active agent.
#[derive(Debug, Deserialize)]
pub struct AgentInfo {
    /// Agent name.
    pub name: String,
    /// Agent status.
    pub status: String,
    /// Current action.
    pub current_action: Option<String>,
}

/// A single log entry.
#[derive(Debug, Deserialize)]
pub struct LogEntry {
    /// Timestamp.
    pub timestamp: String,
    /// Agent name.
    pub agent: String,
    /// Log message.
    pub message: String,
    /// Log level.
    #[serde(default = "default_log_level")]
    pub level: String,
}

fn default_log_level() -> String {
    "info".to_string()
}

/// Response from logs endpoint.
#[derive(Debug, Deserialize)]
pub struct LogsResponse {
    /// Project ID.
    pub project_id: String,
    /// Log entries.
    pub entries: Vec<LogEntry>,
    /// Whether there are more entries.
    pub has_more: bool,
}

/// Request to approve an action.
#[derive(Debug, Serialize)]
pub struct ApproveActionRequest {
    /// Action ID to approve.
    pub action_id: String,
    /// User who approved.
    pub approved_by: String,
}

/// Request to reject an action.
#[derive(Debug, Serialize)]
pub struct RejectActionRequest {
    /// Action ID to reject.
    pub action_id: String,
    /// User who rejected.
    pub rejected_by: String,
    /// Reason for rejection.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

/// Response from action approval/rejection.
#[derive(Debug, Deserialize)]
pub struct ActionResponse {
    /// Action ID.
    pub action_id: String,
    /// New status.
    pub status: String,
    /// Message.
    pub message: String,
}

/// Pending action from the API.
#[derive(Debug, Deserialize)]
pub struct PendingActionResponse {
    /// Action ID.
    pub id: String,
    /// Project ID.
    pub project_id: String,
    /// Action type.
    pub action_type: String,
    /// Description.
    pub description: String,
    /// Context/details.
    pub context: Option<String>,
    /// Created timestamp.
    pub created_at: String,
}

/// Template summary from the API.
#[derive(Debug, Deserialize)]
pub struct TemplateSummary {
    /// Template name.
    pub name: String,
    /// Description.
    pub description: String,
    /// Language/stack.
    pub language: String,
    /// Tags.
    #[serde(default)]
    pub tags: Vec<String>,
}

/// API error response structure.
#[derive(Debug, Deserialize)]
struct ErrorResponse {
    error: String,
}

impl ApiClient {
    /// Create a new API client with the given configuration.
    pub fn new(config: ApiClientConfig) -> Result<Self, ApiError> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .build()?;

        Ok(Self { config, client })
    }

    /// Create a new API client from environment variables.
    ///
    /// Reads:
    /// - `CUTTLEFISH_API_URL` (default: http://localhost:8080)
    /// - `CUTTLEFISH_API_KEY` (required)
    pub fn from_env() -> Result<Self, ApiError> {
        let base_url = std::env::var("CUTTLEFISH_API_URL")
            .unwrap_or_else(|_| "http://localhost:8080".to_string());

        let api_key = std::env::var("CUTTLEFISH_API_KEY").map_err(|_| {
            ApiError::NotConfigured("CUTTLEFISH_API_KEY environment variable not set".to_string())
        })?;

        let timeout_secs = std::env::var("CUTTLEFISH_API_TIMEOUT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(30);

        Self::new(ApiClientConfig {
            base_url,
            api_key,
            timeout_secs,
        })
    }

    /// Build a request with authentication headers.
    fn request(&self, method: reqwest::Method, path: &str) -> reqwest::RequestBuilder {
        let url = format!("{}{}", self.config.base_url, path);
        self.client
            .request(method, &url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
    }

    /// Handle API response, extracting errors if present.
    async fn handle_response<T: for<'de> Deserialize<'de>>(
        response: reqwest::Response,
    ) -> Result<T, ApiError> {
        let status = response.status();

        if status.is_success() {
            response
                .json::<T>()
                .await
                .map_err(|e| ApiError::Parse(e.to_string()))
        } else {
            let error_text = response.text().await.unwrap_or_default();
            let message = serde_json::from_str::<ErrorResponse>(&error_text)
                .map(|e| e.error)
                .unwrap_or(error_text);

            Err(ApiError::ApiResponse {
                status: status.as_u16(),
                message,
            })
        }
    }

    // =========================================================================
    // Project Endpoints
    // =========================================================================

    /// Create a new project.
    ///
    /// POST /api/projects
    pub async fn create_project(&self, request: CreateProjectRequest) -> Result<ProjectResponse, ApiError> {
        let response = self
            .request(reqwest::Method::POST, "/api/projects")
            .json(&request)
            .send()
            .await?;

        Self::handle_response(response).await
    }

    /// Get project status by ID.
    ///
    /// GET /api/projects/:id
    pub async fn get_project(&self, project_id: &str) -> Result<ProjectStatus, ApiError> {
        let response = self
            .request(reqwest::Method::GET, &format!("/api/projects/{project_id}"))
            .send()
            .await?;

        Self::handle_response(response).await
    }

    /// Get project status by name.
    ///
    /// GET /api/projects?name=:name
    pub async fn get_project_by_name(&self, name: &str) -> Result<ProjectStatus, ApiError> {
        let response = self
            .request(reqwest::Method::GET, "/api/projects")
            .query(&[("name", name)])
            .send()
            .await?;

        // API returns a list, we take the first match
        let projects: Vec<ProjectStatus> = Self::handle_response(response).await?;
        projects.into_iter().next().ok_or_else(|| ApiError::ApiResponse {
            status: 404,
            message: format!("Project not found: {name}"),
        })
    }

    /// List all projects.
    ///
    /// GET /api/projects
    pub async fn list_projects(&self) -> Result<Vec<ProjectResponse>, ApiError> {
        let response = self
            .request(reqwest::Method::GET, "/api/projects")
            .send()
            .await?;

        Self::handle_response(response).await
    }

    // =========================================================================
    // Logs Endpoints
    // =========================================================================

    /// Get activity logs for a project.
    ///
    /// GET /api/projects/:id/logs?limit=:limit
    pub async fn get_project_logs(
        &self,
        project_id: &str,
        limit: u32,
    ) -> Result<LogsResponse, ApiError> {
        let response = self
            .request(
                reqwest::Method::GET,
                &format!("/api/projects/{project_id}/logs"),
            )
            .query(&[("limit", limit.to_string())])
            .send()
            .await?;

        Self::handle_response(response).await
    }

    // =========================================================================
    // Action Endpoints
    // =========================================================================

    /// Get pending action for a project.
    ///
    /// GET /api/projects/:id/pending-action
    pub async fn get_pending_action(
        &self,
        project_id: &str,
    ) -> Result<Option<PendingActionResponse>, ApiError> {
        let response = self
            .request(
                reqwest::Method::GET,
                &format!("/api/projects/{project_id}/pending-action"),
            )
            .send()
            .await?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(None);
        }

        Self::handle_response(response).await.map(Some)
    }

    /// Approve a pending action.
    ///
    /// POST /api/projects/:id/approve
    pub async fn approve_action(
        &self,
        project_id: &str,
        request: ApproveActionRequest,
    ) -> Result<ActionResponse, ApiError> {
        let response = self
            .request(
                reqwest::Method::POST,
                &format!("/api/projects/{project_id}/approve"),
            )
            .json(&request)
            .send()
            .await?;

        Self::handle_response(response).await
    }

    /// Reject a pending action.
    ///
    /// POST /api/projects/:id/reject
    pub async fn reject_action(
        &self,
        project_id: &str,
        request: RejectActionRequest,
    ) -> Result<ActionResponse, ApiError> {
        let response = self
            .request(
                reqwest::Method::POST,
                &format!("/api/projects/{project_id}/reject"),
            )
            .json(&request)
            .send()
            .await?;

        Self::handle_response(response).await
    }

    // =========================================================================
    // Template Endpoints
    // =========================================================================

    /// List available templates.
    ///
    /// GET /api/templates
    pub async fn list_templates(&self) -> Result<Vec<TemplateSummary>, ApiError> {
        let response = self
            .request(reqwest::Method::GET, "/api/templates")
            .send()
            .await?;

        Self::handle_response(response).await
    }
}

/// Global API client instance for the Discord bot.
///
/// This is initialized lazily when first accessed.
static API_CLIENT: std::sync::OnceLock<Result<ApiClient, String>> = std::sync::OnceLock::new();

/// Get the global API client instance.
///
/// Initializes from environment variables on first call.
pub fn get_api_client() -> Result<&'static ApiClient, ApiError> {
    let result = API_CLIENT.get_or_init(|| {
        ApiClient::from_env().map_err(|e| e.to_string())
    });

    match result {
        Ok(client) => Ok(client),
        Err(e) => Err(ApiError::NotConfigured(e.clone())),
    }
}

/// Initialize the global API client with a specific configuration.
///
/// This should be called during bot startup if custom configuration is needed.
/// Returns an error if the client has already been initialized.
pub fn init_api_client(config: ApiClientConfig) -> Result<(), ApiError> {
    let client = ApiClient::new(config)?;
    API_CLIENT
        .set(Ok(client))
        .map_err(|_| ApiError::NotConfigured("API client already initialized".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_client_config_default() {
        let config = ApiClientConfig::default();
        assert_eq!(config.base_url, "http://localhost:8080");
        assert!(config.api_key.is_empty());
        assert_eq!(config.timeout_secs, 30);
    }

    #[test]
    fn test_create_project_request_serializes() {
        let req = CreateProjectRequest {
            name: "test-project".to_string(),
            description: "A test project".to_string(),
            template: Some("rust".to_string()),
        };
        let json = serde_json::to_string(&req).expect("should serialize");
        assert!(json.contains("test-project"));
        assert!(json.contains("rust"));
    }

    #[test]
    fn test_create_project_request_without_template() {
        let req = CreateProjectRequest {
            name: "test".to_string(),
            description: "desc".to_string(),
            template: None,
        };
        let json = serde_json::to_string(&req).expect("should serialize");
        assert!(!json.contains("template"));
    }

    #[test]
    fn test_project_response_deserializes() {
        let json = r#"{"id": "abc123", "name": "my-project", "status": "active", "template": null}"#;
        let resp: ProjectResponse = serde_json::from_str(json).expect("should deserialize");
        assert_eq!(resp.id, "abc123");
        assert_eq!(resp.name, "my-project");
        assert_eq!(resp.status, "active");
        assert!(resp.template.is_none());
    }

    #[test]
    fn test_project_status_deserializes() {
        let json = r#"{
            "id": "abc123",
            "name": "my-project",
            "status": "active",
            "active_agents": [{"name": "Coder", "status": "working", "current_action": "Writing code"}],
            "last_activity": "2024-01-15T14:30:00Z",
            "current_task": "Implementing feature X"
        }"#;
        let status: ProjectStatus = serde_json::from_str(json).expect("should deserialize");
        assert_eq!(status.active_agents.len(), 1);
        assert_eq!(status.active_agents[0].name, "Coder");
    }

    #[test]
    fn test_log_entry_deserializes() {
        let json = r#"{"timestamp": "2024-01-15T14:30:00Z", "agent": "Coder", "message": "Started task"}"#;
        let entry: LogEntry = serde_json::from_str(json).expect("should deserialize");
        assert_eq!(entry.agent, "Coder");
        assert_eq!(entry.level, "info"); // default
    }

    #[test]
    fn test_approve_action_request_serializes() {
        let req = ApproveActionRequest {
            action_id: "action-123".to_string(),
            approved_by: "user-456".to_string(),
        };
        let json = serde_json::to_string(&req).expect("should serialize");
        assert!(json.contains("action-123"));
        assert!(json.contains("user-456"));
    }

    #[test]
    fn test_reject_action_request_serializes() {
        let req = RejectActionRequest {
            action_id: "action-123".to_string(),
            rejected_by: "user-456".to_string(),
            reason: Some("Not ready yet".to_string()),
        };
        let json = serde_json::to_string(&req).expect("should serialize");
        assert!(json.contains("Not ready yet"));
    }

    #[test]
    fn test_reject_action_request_without_reason() {
        let req = RejectActionRequest {
            action_id: "action-123".to_string(),
            rejected_by: "user-456".to_string(),
            reason: None,
        };
        let json = serde_json::to_string(&req).expect("should serialize");
        assert!(!json.contains("reason"));
    }

    #[test]
    fn test_api_error_display() {
        let err = ApiError::ApiResponse {
            status: 404,
            message: "Not found".to_string(),
        };
        assert!(err.to_string().contains("404"));
        assert!(err.to_string().contains("Not found"));
    }

    #[test]
    fn test_pending_action_response_deserializes() {
        let json = r#"{
            "id": "action-123",
            "project_id": "proj-456",
            "action_type": "ApproveChange",
            "description": "Apply code changes",
            "context": "Modified 3 files",
            "created_at": "2024-01-15T14:30:00Z"
        }"#;
        let action: PendingActionResponse = serde_json::from_str(json).expect("should deserialize");
        assert_eq!(action.id, "action-123");
        assert_eq!(action.action_type, "ApproveChange");
    }
}
