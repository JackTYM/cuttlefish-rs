//! REST API routes for sandbox management.
//!
//! Provides endpoints for:
//! - `GET /sandbox` - List all sandboxes
//! - `POST /sandbox` - Create a new sandbox
//! - `GET /sandbox/health` - Health check
//! - `GET /sandbox/{id}` - Get sandbox status
//! - `DELETE /sandbox/{id}` - Remove a sandbox
//! - `POST /sandbox/{id}/execute` - Execute a command
//! - `POST /sandbox/{id}/snapshot` - Create a snapshot

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use axum::{
    Router,
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
};
use cuttlefish_core::traits::sandbox::{
    ContainerConfig, ContainerStatus, HealthChecker, ImageSpec, Language, ResourceLimits,
    SandboxHandle, SandboxLifecycle, SnapshotManager, SnapshotOptions,
};
use cuttlefish_sandbox::{DockerHealthChecker, DockerSandboxLifecycle, DockerSnapshotManager};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

// ============================================================================
// Request/Response Types
// ============================================================================

/// Request to create a new sandbox.
#[derive(Debug, Deserialize)]
pub struct CreateSandboxRequest {
    /// Language: "node", "python", "rust", "go", "ruby", "generic".
    pub language: String,
    /// Resource preset: "light", "standard", "heavy" (default: standard).
    pub resource_preset: Option<String>,
    /// Custom name for the sandbox.
    pub name: Option<String>,
}

/// Response after creating a sandbox.
#[derive(Debug, Serialize)]
pub struct CreateSandboxResponse {
    /// Unique identifier for the sandbox.
    pub id: String,
    /// Name of the sandbox.
    pub name: String,
    /// Current status of the sandbox.
    pub status: String,
}

/// Request to execute a command.
#[derive(Debug, Deserialize)]
pub struct ExecuteRequest {
    /// Command to execute (e.g., ["npm", "install"]).
    pub command: Vec<String>,
    /// Timeout in seconds (default: 60).
    pub timeout_secs: Option<u64>,
}

/// Response from command execution.
#[derive(Debug, Serialize)]
pub struct ExecuteResponse {
    /// Exit code from the command.
    pub exit_code: i64,
    /// Standard output.
    pub stdout: String,
    /// Standard error.
    pub stderr: String,
    /// Duration in milliseconds.
    pub duration_ms: u64,
    /// Whether the command succeeded.
    pub success: bool,
}

/// Request to create a snapshot.
#[derive(Debug, Deserialize)]
pub struct SnapshotRequest {
    /// Optional snapshot name (auto-generated if omitted).
    pub name: Option<String>,
    /// Pause container during snapshot for consistency.
    pub pause: Option<bool>,
}

/// Response after creating a snapshot.
#[derive(Debug, Serialize)]
pub struct SnapshotResponse {
    /// Unique identifier for the snapshot.
    pub id: String,
    /// Name of the snapshot.
    pub name: String,
}

/// Sandbox status response.
#[derive(Debug, Serialize)]
pub struct StatusResponse {
    /// Unique identifier for the sandbox.
    pub id: String,
    /// Name of the sandbox.
    pub name: String,
    /// Current status.
    pub status: String,
    /// Creation timestamp (RFC 3339).
    pub created_at: String,
}

/// Health check response.
#[derive(Debug, Serialize)]
pub struct SandboxHealthResponse {
    /// Whether the sandbox system is healthy.
    pub healthy: bool,
    /// Docker daemon status.
    pub docker_status: String,
    /// Number of active containers.
    pub containers: usize,
    /// Number of volumes.
    pub volumes: usize,
    /// Available images and their status.
    pub images: HashMap<String, bool>,
}

/// List sandboxes response.
#[derive(Debug, Serialize)]
pub struct ListSandboxesResponse {
    /// List of sandboxes.
    pub sandboxes: Vec<SandboxInfo>,
}

/// Information about a sandbox.
#[derive(Debug, Serialize)]
pub struct SandboxInfo {
    /// Unique identifier.
    pub id: String,
    /// Name of the sandbox.
    pub name: String,
    /// Current status.
    pub status: String,
}

/// Error response.
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    /// Error message.
    pub error: String,
    /// Error code.
    pub code: String,
}

// ============================================================================
// State
// ============================================================================

/// State for sandbox routes.
#[derive(Clone)]
pub struct SandboxState {
    /// Sandbox lifecycle manager.
    pub lifecycle: Option<Arc<DockerSandboxLifecycle>>,
    /// Health checker.
    pub health_checker: Option<Arc<DockerHealthChecker>>,
    /// Snapshot manager.
    pub snapshot_manager: Option<Arc<DockerSnapshotManager>>,
    /// Active sandbox handles indexed by ID.
    pub sandboxes: Arc<DashMap<String, SandboxHandle>>,
}

impl Default for SandboxState {
    fn default() -> Self {
        Self::new()
    }
}

impl SandboxState {
    /// Create a new sandbox state, attempting to connect to Docker.
    pub fn new() -> Self {
        let lifecycle = DockerSandboxLifecycle::new().ok().map(Arc::new);
        let health_checker = DockerHealthChecker::new().ok().map(Arc::new);
        let snapshot_manager = DockerSnapshotManager::new().ok().map(Arc::new);

        if lifecycle.is_none() {
            warn!("Docker sandbox unavailable - sandbox features will be disabled");
        }

        Self {
            lifecycle,
            health_checker,
            snapshot_manager,
            sandboxes: Arc::new(DashMap::new()),
        }
    }

    /// Check if the sandbox system is available.
    pub fn is_available(&self) -> bool {
        self.lifecycle.is_some()
    }
}

// ============================================================================
// Helpers
// ============================================================================

fn parse_language(s: &str) -> Option<Language> {
    match s.to_lowercase().as_str() {
        "node" | "nodejs" | "javascript" | "js" => Some(Language::Node),
        "python" | "py" => Some(Language::Python),
        "rust" | "rs" => Some(Language::Rust),
        "go" | "golang" => Some(Language::Go),
        "ruby" | "rb" => Some(Language::Ruby),
        "generic" | "ubuntu" => Some(Language::Generic),
        _ => None,
    }
}

fn language_to_image(lang: Language) -> ImageSpec {
    let (name, tag) = match lang {
        Language::Node => ("node", "22-slim"),
        Language::Python => ("python", "3.12-slim"),
        Language::Rust => ("rust", "1.82-slim"),
        Language::Go => ("golang", "1.22-bookworm"),
        Language::Ruby => ("ruby", "3.3-slim"),
        Language::Generic => ("ubuntu", "22.04"),
    };
    ImageSpec {
        name: name.to_string(),
        tag: tag.to_string(),
        language: lang,
        size_bytes: None,
        created_at: None,
    }
}

fn preset_to_limits(preset: Option<&str>) -> ResourceLimits {
    match preset {
        Some("light") => ResourceLimits {
            memory_bytes: Some(512 * 1024 * 1024), // 512MB
            cpu_quota: Some(50_000),               // 0.5 CPU
            cpu_period: Some(100_000),
            pids_limit: Some(128),
            ..Default::default()
        },
        Some("heavy") => ResourceLimits {
            memory_bytes: Some(4 * 1024 * 1024 * 1024), // 4GB
            cpu_quota: Some(400_000),                   // 4 CPU
            cpu_period: Some(100_000),
            pids_limit: Some(1024),
            ..Default::default()
        },
        _ => ResourceLimits {
            memory_bytes: Some(2 * 1024 * 1024 * 1024), // 2GB
            cpu_quota: Some(200_000),                   // 2 CPU
            cpu_period: Some(100_000),
            pids_limit: Some(512),
            ..Default::default()
        },
    }
}

fn status_to_string(status: ContainerStatus) -> &'static str {
    match status {
        ContainerStatus::Created => "created",
        ContainerStatus::Running => "running",
        ContainerStatus::Paused => "paused",
        ContainerStatus::Stopped => "stopped",
        ContainerStatus::Removed => "removed",
    }
}

fn api_error(code: StatusCode, err: &str, error_code: &str) -> (StatusCode, Json<ErrorResponse>) {
    (
        code,
        Json(ErrorResponse {
            error: err.to_string(),
            code: error_code.to_string(),
        }),
    )
}

// ============================================================================
// Route Handlers
// ============================================================================

/// POST /sandbox - Create a new sandbox.
async fn create_sandbox(
    State(state): State<SandboxState>,
    Json(req): Json<CreateSandboxRequest>,
) -> Result<Json<CreateSandboxResponse>, (StatusCode, Json<ErrorResponse>)> {
    let lifecycle = state.lifecycle.as_ref().ok_or_else(|| {
        api_error(
            StatusCode::SERVICE_UNAVAILABLE,
            "Docker sandbox unavailable",
            "SANDBOX_UNAVAILABLE",
        )
    })?;

    let language = parse_language(&req.language).ok_or_else(|| {
        api_error(
            StatusCode::BAD_REQUEST,
            &format!(
                "Unknown language: {}. Valid options: node, python, rust, go, ruby, generic",
                req.language
            ),
            "INVALID_LANGUAGE",
        )
    })?;

    let image = language_to_image(language);
    let limits = preset_to_limits(req.resource_preset.as_deref());

    let config = ContainerConfig {
        image,
        name: req.name,
        resource_limits: limits,
        ..Default::default()
    };

    let handle = lifecycle.create(config).await.map_err(|e| {
        api_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!("Failed to create sandbox: {e}"),
            "CREATE_FAILED",
        )
    })?;

    // Start the container
    lifecycle.start(&handle).await.map_err(|e| {
        api_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!("Failed to start sandbox: {e}"),
            "START_FAILED",
        )
    })?;

    let id = handle.id.clone();
    let name = handle.name.clone();

    // Track the sandbox
    state.sandboxes.insert(id.clone(), handle);

    info!("Created sandbox {} ({})", name, id);

    Ok(Json(CreateSandboxResponse {
        id,
        name,
        status: "running".to_string(),
    }))
}

/// POST /sandbox/{id}/execute - Execute a command in sandbox.
async fn execute_command(
    State(state): State<SandboxState>,
    Path(id): Path<String>,
    Json(req): Json<ExecuteRequest>,
) -> Result<Json<ExecuteResponse>, (StatusCode, Json<ErrorResponse>)> {
    if req.command.is_empty() {
        return Err(api_error(
            StatusCode::BAD_REQUEST,
            "Command cannot be empty",
            "EMPTY_COMMAND",
        ));
    }

    let lifecycle = state.lifecycle.as_ref().ok_or_else(|| {
        api_error(
            StatusCode::SERVICE_UNAVAILABLE,
            "Docker sandbox unavailable",
            "SANDBOX_UNAVAILABLE",
        )
    })?;

    let handle = state.sandboxes.get(&id).ok_or_else(|| {
        api_error(
            StatusCode::NOT_FOUND,
            &format!("Sandbox not found: {id}"),
            "NOT_FOUND",
        )
    })?;

    let timeout_secs = req.timeout_secs.unwrap_or(60);
    let timeout = Duration::from_secs(timeout_secs);

    let result = lifecycle
        .execute(&handle, &req.command, timeout)
        .await
        .map_err(|e| {
            api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                &format!("Execution failed: {e}"),
                "EXEC_FAILED",
            )
        })?;

    let success = result.success();
    Ok(Json(ExecuteResponse {
        exit_code: result.exit_code,
        stdout: result.stdout,
        stderr: result.stderr,
        duration_ms: result.duration.as_millis() as u64,
        success,
    }))
}

/// POST /sandbox/{id}/snapshot - Create a snapshot.
async fn create_snapshot_handler(
    State(state): State<SandboxState>,
    Path(id): Path<String>,
    Json(req): Json<SnapshotRequest>,
) -> Result<Json<SnapshotResponse>, (StatusCode, Json<ErrorResponse>)> {
    let snapshot_manager = state.snapshot_manager.as_ref().ok_or_else(|| {
        api_error(
            StatusCode::SERVICE_UNAVAILABLE,
            "Snapshot manager unavailable",
            "SNAPSHOT_UNAVAILABLE",
        )
    })?;

    let handle = state.sandboxes.get(&id).ok_or_else(|| {
        api_error(
            StatusCode::NOT_FOUND,
            &format!("Sandbox not found: {id}"),
            "NOT_FOUND",
        )
    })?;

    let options = SnapshotOptions {
        name: req.name,
        pause_container: req.pause.unwrap_or(false),
        ..Default::default()
    };

    let snapshot = snapshot_manager
        .create_snapshot(&handle, options)
        .await
        .map_err(|e| {
            api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                &format!("Failed to create snapshot: {e}"),
                "SNAPSHOT_FAILED",
            )
        })?;

    info!("Created snapshot {} for sandbox {}", snapshot.name, id);

    Ok(Json(SnapshotResponse {
        id: snapshot.id,
        name: snapshot.name,
    }))
}

/// GET /sandbox/{id} - Get sandbox status.
async fn get_sandbox_status(
    State(state): State<SandboxState>,
    Path(id): Path<String>,
) -> Result<Json<StatusResponse>, (StatusCode, Json<ErrorResponse>)> {
    let lifecycle = state.lifecycle.as_ref().ok_or_else(|| {
        api_error(
            StatusCode::SERVICE_UNAVAILABLE,
            "Docker sandbox unavailable",
            "SANDBOX_UNAVAILABLE",
        )
    })?;

    let handle = state.sandboxes.get(&id).ok_or_else(|| {
        api_error(
            StatusCode::NOT_FOUND,
            &format!("Sandbox not found: {id}"),
            "NOT_FOUND",
        )
    })?;

    let status = lifecycle.status(&handle).await.map_err(|e| {
        api_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!("Failed to get status: {e}"),
            "STATUS_FAILED",
        )
    })?;

    Ok(Json(StatusResponse {
        id: handle.id.clone(),
        name: handle.name.clone(),
        status: status_to_string(status).to_string(),
        created_at: handle.created_at.to_rfc3339(),
    }))
}

/// DELETE /sandbox/{id} - Remove a sandbox.
async fn remove_sandbox(
    State(state): State<SandboxState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let lifecycle = state.lifecycle.as_ref().ok_or_else(|| {
        api_error(
            StatusCode::SERVICE_UNAVAILABLE,
            "Docker sandbox unavailable",
            "SANDBOX_UNAVAILABLE",
        )
    })?;

    let handle = state.sandboxes.remove(&id).ok_or_else(|| {
        api_error(
            StatusCode::NOT_FOUND,
            &format!("Sandbox not found: {id}"),
            "NOT_FOUND",
        )
    })?;

    // Stop and remove the container
    let _ = lifecycle.stop(&handle.1, 10).await;
    lifecycle.remove(&handle.1).await.map_err(|e| {
        api_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!("Failed to remove sandbox: {e}"),
            "REMOVE_FAILED",
        )
    })?;

    info!("Removed sandbox {} ({})", handle.1.name, id);
    Ok(StatusCode::NO_CONTENT)
}

/// GET /sandbox - List all sandboxes.
async fn list_sandboxes(State(state): State<SandboxState>) -> Json<ListSandboxesResponse> {
    let lifecycle = match &state.lifecycle {
        Some(l) => l,
        None => {
            return Json(ListSandboxesResponse {
                sandboxes: Vec::new(),
            });
        }
    };

    let mut sandboxes = Vec::new();
    for entry in state.sandboxes.iter() {
        let handle = entry.value();
        let status = lifecycle
            .status(handle)
            .await
            .map(status_to_string)
            .unwrap_or("unknown");

        sandboxes.push(SandboxInfo {
            id: handle.id.clone(),
            name: handle.name.clone(),
            status: status.to_string(),
        });
    }

    Json(ListSandboxesResponse { sandboxes })
}

/// GET /sandbox/health - Health check.
async fn sandbox_health(State(state): State<SandboxState>) -> Json<SandboxHealthResponse> {
    let health_checker = match &state.health_checker {
        Some(h) => h,
        None => {
            return Json(SandboxHealthResponse {
                healthy: false,
                docker_status: "unavailable".to_string(),
                containers: 0,
                volumes: 0,
                images: HashMap::new(),
            });
        }
    };

    match health_checker.check_health().await {
        Ok(health) => {
            let images: HashMap<String, bool> = health
                .images_available
                .iter()
                .map(|(lang, available)| (format!("{lang:?}").to_lowercase(), *available))
                .collect();

            Json(SandboxHealthResponse {
                healthy: health.is_healthy(),
                docker_status: if health.docker_healthy {
                    "connected"
                } else {
                    "disconnected"
                }
                .to_string(),
                containers: health.resource_usage.container_count,
                volumes: health.resource_usage.volume_count,
                images,
            })
        }
        Err(e) => {
            warn!("Health check failed: {e}");
            Json(SandboxHealthResponse {
                healthy: false,
                docker_status: format!("error: {e}"),
                containers: 0,
                volumes: 0,
                images: HashMap::new(),
            })
        }
    }
}

// ============================================================================
// Router Builder
// ============================================================================

/// Build sandbox routes without state (for testing).
pub fn sandbox_router() -> Router {
    sandbox_router_with_state(SandboxState::new())
}

/// Build sandbox routes with provided state.
pub fn sandbox_router_with_state(state: SandboxState) -> Router {
    Router::new()
        .route("/sandbox", get(list_sandboxes).post(create_sandbox))
        .route("/sandbox/health", get(sandbox_health))
        .route(
            "/sandbox/{id}",
            get(get_sandbox_status).delete(remove_sandbox),
        )
        .route("/sandbox/{id}/execute", post(execute_command))
        .route("/sandbox/{id}/snapshot", post(create_snapshot_handler))
        .with_state(state)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_language_node() {
        assert_eq!(parse_language("node"), Some(Language::Node));
        assert_eq!(parse_language("nodejs"), Some(Language::Node));
        assert_eq!(parse_language("JavaScript"), Some(Language::Node));
    }

    #[test]
    fn test_parse_language_python() {
        assert_eq!(parse_language("python"), Some(Language::Python));
        assert_eq!(parse_language("py"), Some(Language::Python));
    }

    #[test]
    fn test_parse_language_rust() {
        assert_eq!(parse_language("rust"), Some(Language::Rust));
        assert_eq!(parse_language("rs"), Some(Language::Rust));
    }

    #[test]
    fn test_parse_language_go() {
        assert_eq!(parse_language("go"), Some(Language::Go));
        assert_eq!(parse_language("golang"), Some(Language::Go));
    }

    #[test]
    fn test_parse_language_ruby() {
        assert_eq!(parse_language("ruby"), Some(Language::Ruby));
        assert_eq!(parse_language("rb"), Some(Language::Ruby));
    }

    #[test]
    fn test_parse_language_generic() {
        assert_eq!(parse_language("generic"), Some(Language::Generic));
        assert_eq!(parse_language("ubuntu"), Some(Language::Generic));
    }

    #[test]
    fn test_parse_language_invalid() {
        assert_eq!(parse_language("cobol"), None);
        assert_eq!(parse_language(""), None);
    }

    #[test]
    fn test_language_to_image_node() {
        let image = language_to_image(Language::Node);
        assert_eq!(image.name, "node");
        assert_eq!(image.tag, "22-slim");
        assert_eq!(image.language, Language::Node);
    }

    #[test]
    fn test_preset_to_limits_light() {
        let limits = preset_to_limits(Some("light"));
        assert_eq!(limits.memory_bytes, Some(512 * 1024 * 1024));
        assert_eq!(limits.cpu_quota, Some(50_000));
    }

    #[test]
    fn test_preset_to_limits_heavy() {
        let limits = preset_to_limits(Some("heavy"));
        assert_eq!(limits.memory_bytes, Some(4 * 1024 * 1024 * 1024));
        assert_eq!(limits.cpu_quota, Some(400_000));
    }

    #[test]
    fn test_preset_to_limits_standard() {
        let limits = preset_to_limits(Some("standard"));
        assert_eq!(limits.memory_bytes, Some(2 * 1024 * 1024 * 1024));
        assert_eq!(limits.cpu_quota, Some(200_000));
    }

    #[test]
    fn test_preset_to_limits_default() {
        let limits = preset_to_limits(None);
        assert_eq!(limits.memory_bytes, Some(2 * 1024 * 1024 * 1024));
    }

    #[test]
    fn test_status_to_string() {
        assert_eq!(status_to_string(ContainerStatus::Created), "created");
        assert_eq!(status_to_string(ContainerStatus::Running), "running");
        assert_eq!(status_to_string(ContainerStatus::Paused), "paused");
        assert_eq!(status_to_string(ContainerStatus::Stopped), "stopped");
        assert_eq!(status_to_string(ContainerStatus::Removed), "removed");
    }

    #[test]
    fn test_sandbox_state_default() {
        // Just test that it doesn't panic - actual Docker connection may or may not work
        let _ = SandboxState::default();
    }
}
