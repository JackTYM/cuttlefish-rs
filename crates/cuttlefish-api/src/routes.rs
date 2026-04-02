//! HTTP route handlers for the API server.

use std::sync::Arc;

use axum::{http::StatusCode, response::Json};
use cuttlefish_core::TemplateRegistry;
use serde::Serialize;

/// Application state shared across handlers.
#[derive(Clone)]
pub struct AppState {
    /// API key for authentication.
    pub api_key: String,
    /// Template registry for project scaffolding.
    pub template_registry: Arc<TemplateRegistry>,
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
