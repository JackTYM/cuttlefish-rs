//! REST API route handlers for project management.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};

use crate::routes::AppState;

/// Request body for creating a project.
#[derive(Debug, Deserialize)]
pub struct CreateProjectRequest {
    /// Project name.
    pub name: String,
    /// Project description.
    pub description: String,
    /// Template name (optional).
    pub template: Option<String>,
}

/// Response body for a project.
#[derive(Debug, Serialize)]
pub struct ProjectResponse {
    /// Project ID.
    pub id: String,
    /// Project name.
    pub name: String,
    /// Status.
    pub status: String,
}

/// Create a new project.
///
/// POST /api/projects
pub async fn create_project(
    State(_state): State<AppState>,
    Json(req): Json<CreateProjectRequest>,
) -> (StatusCode, Json<ProjectResponse>) {
    // In a real implementation this would persist to DB
    let id = uuid::Uuid::new_v4().to_string();
    (
        StatusCode::CREATED,
        Json(ProjectResponse {
            id,
            name: req.name,
            status: "active".to_string(),
        }),
    )
}

/// List all projects.
///
/// GET /api/projects
pub async fn list_projects(
    State(_state): State<AppState>,
) -> Json<Vec<ProjectResponse>> {
    // In a real implementation this would query DB
    Json(vec![])
}

/// Get a project by ID.
///
/// GET /api/projects/:id
pub async fn get_project(
    State(_state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ProjectResponse>, (StatusCode, Json<serde_json::Value>)> {
    // In a real implementation this would query DB
    Err((
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({ "error": format!("Project {} not found", id) })),
    ))
}

/// Cancel/delete a project.
///
/// DELETE /api/projects/:id
pub async fn cancel_project(
    State(_state): State<AppState>,
    Path(id): Path<String>,
) -> (StatusCode, Json<serde_json::Value>) {
    let _ = id;
    (StatusCode::OK, Json(serde_json::json!({ "status": "cancelled" })))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_project_request_deserializes() {
        let json = r#"{"name": "my-app", "description": "A test app"}"#;
        let req: CreateProjectRequest = serde_json::from_str(json).expect("parse");
        assert_eq!(req.name, "my-app");
        assert!(req.template.is_none());
    }

    #[test]
    fn test_project_response_serializes() {
        let resp = ProjectResponse {
            id: "abc".to_string(),
            name: "test".to_string(),
            status: "active".to_string(),
        };
        let json = serde_json::to_string(&resp).expect("serialize");
        assert!(json.contains("active"));
    }
}
