//! REST API routes for sandbox management.

use axum::{
    extract::Path,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// Request/Response Types
// ============================================================================

/// Request to create a new sandbox
#[derive(Debug, Deserialize)]
pub struct CreateSandboxRequest {
    /// Language: "node", "python", "rust", "go", "ruby"
    pub language: String,
    /// Resource preset: "light", "standard", "heavy" (default: standard)
    pub resource_preset: Option<String>,
    /// Custom name for the sandbox
    pub name: Option<String>,
}

/// Response after creating a sandbox
#[derive(Debug, Serialize)]
pub struct CreateSandboxResponse {
    /// Unique identifier for the sandbox
    pub id: String,
    /// Name of the sandbox
    pub name: String,
    /// Current status of the sandbox
    pub status: String,
}

/// Request to execute a command
#[derive(Debug, Deserialize)]
pub struct ExecuteRequest {
    /// Command to execute (e.g., ["npm", "install"])
    pub command: Vec<String>,
    /// Timeout in seconds (default: 60)
    pub timeout_secs: Option<u64>,
}

/// Response from command execution
#[derive(Debug, Serialize)]
pub struct ExecuteResponse {
    /// Exit code from the command
    pub exit_code: i64,
    /// Standard output
    pub stdout: String,
    /// Standard error
    pub stderr: String,
    /// Duration in milliseconds
    pub duration_ms: u64,
    /// Whether the command succeeded
    pub success: bool,
}

/// Request to create a snapshot
#[derive(Debug, Deserialize)]
pub struct SnapshotRequest {
    /// Optional snapshot name (auto-generated if omitted)
    pub name: Option<String>,
    /// Pause container during snapshot for consistency
    pub pause: Option<bool>,
}

/// Response after creating a snapshot
#[derive(Debug, Serialize)]
pub struct SnapshotResponse {
    /// Unique identifier for the snapshot
    pub id: String,
    /// Name of the snapshot
    pub name: String,
}

/// Sandbox status response
#[derive(Debug, Serialize)]
pub struct StatusResponse {
    /// Unique identifier for the sandbox
    pub id: String,
    /// Name of the sandbox
    pub name: String,
    /// Current status
    pub status: String,
    /// Creation timestamp (RFC 3339)
    pub created_at: String,
}

/// Health check response
#[derive(Debug, Serialize)]
pub struct SandboxHealthResponse {
    /// Whether the sandbox system is healthy
    pub healthy: bool,
    /// Docker daemon status
    pub docker_status: String,
    /// Number of active containers
    pub containers: usize,
    /// Number of volumes
    pub volumes: usize,
    /// Available images and their status
    pub images: HashMap<String, bool>,
}

/// List sandboxes response
#[derive(Debug, Serialize)]
pub struct ListSandboxesResponse {
    /// List of sandboxes
    pub sandboxes: Vec<SandboxInfo>,
}

/// Information about a sandbox
#[derive(Debug, Serialize)]
pub struct SandboxInfo {
    /// Unique identifier
    pub id: String,
    /// Name of the sandbox
    pub name: String,
    /// Current status
    pub status: String,
}

/// Error response
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    /// Error message
    pub error: String,
    /// Error code
    pub code: String,
}

// ============================================================================
// Route Handlers (Stubs)
// ============================================================================

/// POST /sandbox - Create a new sandbox
async fn create_sandbox(
    Json(req): Json<CreateSandboxRequest>,
) -> Result<Json<CreateSandboxResponse>, (StatusCode, Json<ErrorResponse>)> {
    // TODO: Implement with actual sandbox lifecycle
    // For now, return a stub response

    // Validate language
    let valid_languages = ["node", "nodejs", "python", "rust", "go", "ruby"];
    if !valid_languages.contains(&req.language.as_str()) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: format!("Unknown language: {}", req.language),
                code: "INVALID_LANGUAGE".to_string(),
            }),
        ));
    }

    // Generate a mock ID for now
    let id = uuid::Uuid::new_v4().to_string();
    let name = req.name.unwrap_or_else(|| format!("cuttlefish-{}", &id[..8]));

    Ok(Json(CreateSandboxResponse {
        id,
        name,
        status: "created".to_string(),
    }))
}

/// POST /sandbox/:id/execute - Execute a command in sandbox
async fn execute_command(
    Path(id): Path<String>,
    Json(req): Json<ExecuteRequest>,
) -> Result<Json<ExecuteResponse>, (StatusCode, Json<ErrorResponse>)> {
    if req.command.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Command cannot be empty".to_string(),
                code: "EMPTY_COMMAND".to_string(),
            }),
        ));
    }

    // TODO: Implement with actual sandbox lifecycle
    Ok(Json(ExecuteResponse {
        exit_code: 0,
        stdout: format!("Executed in sandbox {}: {:?}", id, req.command),
        stderr: String::new(),
        duration_ms: 100,
        success: true,
    }))
}

/// POST /sandbox/:id/snapshot - Create a snapshot
async fn create_snapshot_handler(
    Path(id): Path<String>,
    Json(req): Json<SnapshotRequest>,
) -> Result<Json<SnapshotResponse>, (StatusCode, Json<ErrorResponse>)> {
    let name = req.name.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    // TODO: Implement with actual snapshot manager
    let _ = id; // Suppress unused warning for now
    Ok(Json(SnapshotResponse {
        id: format!("snap-{}", &name[..8.min(name.len())]),
        name,
    }))
}

/// GET /sandbox/:id - Get sandbox status
async fn get_sandbox_status(
    Path(id): Path<String>,
) -> Result<Json<StatusResponse>, (StatusCode, Json<ErrorResponse>)> {
    // TODO: Implement with actual sandbox lifecycle
    Ok(Json(StatusResponse {
        id: id.clone(),
        name: format!("cuttlefish-{}", &id[..8.min(id.len())]),
        status: "running".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
    }))
}

/// DELETE /sandbox/:id - Remove a sandbox
async fn remove_sandbox(
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    // TODO: Implement with actual sandbox lifecycle
    tracing::info!("Removing sandbox: {}", id);
    Ok(StatusCode::NO_CONTENT)
}

/// GET /sandbox - List all sandboxes
async fn list_sandboxes() -> Json<ListSandboxesResponse> {
    // TODO: Implement with actual sandbox lifecycle
    Json(ListSandboxesResponse {
        sandboxes: Vec::new(),
    })
}

/// GET /sandbox/health - Health check
async fn sandbox_health() -> Json<SandboxHealthResponse> {
    // TODO: Implement with actual health checker
    Json(SandboxHealthResponse {
        healthy: true,
        docker_status: "connected".to_string(),
        containers: 0,
        volumes: 0,
        images: HashMap::new(),
    })
}

// ============================================================================
// Router Builder
// ============================================================================

/// Build sandbox routes
pub fn sandbox_router() -> Router {
    Router::new()
        .route("/sandbox", get(list_sandboxes).post(create_sandbox))
        .route("/sandbox/health", get(sandbox_health))
        .route(
            "/sandbox/{id}",
            get(get_sandbox_status).delete(remove_sandbox),
        )
        .route("/sandbox/{id}/execute", post(execute_command))
        .route("/sandbox/{id}/snapshot", post(create_snapshot_handler))
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    fn test_router() -> Router {
        sandbox_router()
    }

    #[tokio::test]
    async fn test_create_sandbox_valid_language() {
        let app = test_router();

        let body = serde_json::json!({
            "language": "node"
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/sandbox")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&body).expect("serialize body")))
                    .expect("build request"),
            )
            .await
            .expect("send request");

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_create_sandbox_invalid_language() {
        let app = test_router();

        let body = serde_json::json!({
            "language": "cobol"
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/sandbox")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&body).expect("serialize body")))
                    .expect("build request"),
            )
            .await
            .expect("send request");

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_execute_empty_command() {
        let app = test_router();

        let body = serde_json::json!({
            "command": []
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/sandbox/test-id/execute")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&body).expect("serialize body")))
                    .expect("build request"),
            )
            .await
            .expect("send request");

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_health_endpoint() {
        let app = test_router();

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/sandbox/health")
                    .body(Body::empty())
                    .expect("build request"),
            )
            .await
            .expect("send request");

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_list_sandboxes() {
        let app = test_router();

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/sandbox")
                    .body(Body::empty())
                    .expect("build request"),
            )
            .await
            .expect("send request");

        assert_eq!(response.status(), StatusCode::OK);
    }
}
