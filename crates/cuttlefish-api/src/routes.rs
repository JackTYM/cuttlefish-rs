//! HTTP route handlers for the API server.

use std::path::PathBuf;
use std::sync::Arc;

use axum::{http::StatusCode, response::Json};
use cuttlefish_agents::{TokioMessageBus, WorkflowEngine};
use cuttlefish_core::TemplateRegistry;
use cuttlefish_db::Database;
use cuttlefish_providers::ProviderRegistry;
use dashmap::DashMap;
use serde::Serialize;
use tokio::sync::mpsc;

use crate::approval_registry::SharedApprovalRegistry;
use crate::ws::ServerMessage;

/// A session for a specific project, tracking active clients and workflow state.
pub struct ProjectSession {
    /// The project ID this session belongs to.
    pub project_id: String,
    /// Workflow engine for this project (if initialized).
    pub workflow: Option<WorkflowEngine>,
    /// Connected WebSocket clients receiving messages for this project.
    pub clients: Vec<mpsc::Sender<ServerMessage>>,
    /// When the session was created.
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl ProjectSession {
    /// Create a new project session.
    pub fn new(project_id: String) -> Self {
        Self {
            project_id,
            workflow: None,
            clients: Vec::new(),
            created_at: chrono::Utc::now(),
        }
    }
}

/// Application state shared across handlers.
#[derive(Clone)]
pub struct AppState {
    /// API key for authentication.
    pub api_key: String,
    /// Template registry for project scaffolding.
    pub template_registry: Arc<TemplateRegistry>,
    /// Database connection.
    pub db: Arc<Database>,
    /// Provider registry for model providers.
    pub provider_registry: Arc<ProviderRegistry>,
    /// Active project sessions indexed by project ID.
    pub active_sessions: Arc<DashMap<String, ProjectSession>>,
    /// Message bus for agent communication.
    pub message_bus: Arc<TokioMessageBus>,
    /// Directory containing agent prompts.
    pub prompts_dir: PathBuf,
    /// Default provider name to use when creating workflows.
    pub default_provider: Option<String>,
    /// Approval registry for safety workflow integration.
    pub approval_registry: SharedApprovalRegistry,
}

/// Health check response.
#[derive(Serialize)]
pub struct HealthResponse {
    /// Service status.
    pub status: &'static str,
    /// Service version.
    pub version: &'static str,
}

/// Health check handler — always returns 200 OK.
pub async fn health_handler() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        version: env!("CARGO_PKG_VERSION"),
    })
}

/// Handler for unknown routes — returns 404.
pub async fn not_found_handler() -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({ "error": "Not found" })),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_health_handler_returns_ok() {
        let response = health_handler().await;
        assert_eq!(response.0.status, "ok");
    }
}
