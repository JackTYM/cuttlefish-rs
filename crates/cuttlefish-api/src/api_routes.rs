//! REST API route handlers for project and template management.

use std::collections::HashMap;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};

use crate::routes::AppState;

// ============================================================================
// Template Types
// ============================================================================

/// Response for a template summary (used in list endpoint).
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TemplateSummary {
    /// Template ID (same as name for now).
    pub id: String,
    /// Template name.
    pub name: String,
    /// One-line description.
    pub description: String,
    /// Full description (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub full_description: Option<String>,
    /// Programming language/stack.
    pub language: String,
    /// Template category.
    pub category: String,
    /// Tags for categorization.
    pub tags: Vec<String>,
    /// Number of times used.
    pub use_count: u32,
    /// Star count (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stars: Option<u32>,
    /// Last updated timestamp.
    pub updated_at: String,
    /// Source URL (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_url: Option<String>,
    /// File structure preview (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_structure: Option<String>,
}

/// Response for full template details.
#[derive(Debug, Serialize)]
pub struct TemplateDetail {
    /// Template name.
    pub name: String,
    /// One-line description.
    pub description: String,
    /// Programming language/stack.
    pub language: String,
    /// Docker base image.
    pub docker_image: String,
    /// Template variables.
    pub variables: Vec<TemplateVariableResponse>,
    /// Tags for categorization.
    pub tags: Vec<String>,
    /// First 500 characters of content (preview).
    pub content_preview: String,
}

/// Template variable in response.
#[derive(Debug, Serialize)]
pub struct TemplateVariableResponse {
    /// Variable name.
    pub name: String,
    /// Human-readable description.
    pub description: String,
    /// Default value if not provided.
    pub default: Option<String>,
    /// Whether this variable is required.
    pub required: bool,
}

/// Request to fetch a template from URL.
#[derive(Debug, Deserialize)]
pub struct FetchTemplateRequest {
    /// Remote URL to fetch template from.
    pub url: String,
}

// ============================================================================
// Template Handlers
// ============================================================================

/// List all available templates.
///
/// GET /api/templates
pub async fn list_templates(State(state): State<AppState>) -> Json<Vec<TemplateSummary>> {
    let templates = state.template_registry.list();
    Json(
        templates
            .into_iter()
            .map(|t| {
                // Derive category from language or tags
                let category = if t.manifest.tags.iter().any(|tag| tag == "cli") {
                    "CLI".to_string()
                } else if t.manifest.tags.iter().any(|tag| tag == "web" || tag == "frontend") {
                    "Web".to_string()
                } else if t.manifest.tags.iter().any(|tag| tag == "library" || tag == "lib") {
                    "Library".to_string()
                } else if t.manifest.tags.iter().any(|tag| tag == "api" || tag == "backend") {
                    "API".to_string()
                } else {
                    "Other".to_string()
                };

                TemplateSummary {
                    id: t.manifest.name.clone(),
                    name: t.manifest.name,
                    description: t.manifest.description,
                    full_description: None,
                    language: t.manifest.language,
                    category,
                    tags: t.manifest.tags,
                    use_count: 0,
                    stars: None,
                    updated_at: chrono::Utc::now().to_rfc3339(),
                    source_url: None,
                    file_structure: None,
                }
            })
            .collect(),
    )
}

/// Get a single template by name.
///
/// GET /api/templates/:name
pub async fn get_template(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<TemplateDetail>, (StatusCode, Json<serde_json::Value>)> {
    let template = state.template_registry.get(&name).ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": format!("Template not found: {}", name) })),
        )
    })?;

    Ok(Json(TemplateDetail {
        name: template.manifest.name,
        description: template.manifest.description,
        language: template.manifest.language,
        docker_image: template.manifest.docker_image,
        variables: template
            .manifest
            .variables
            .into_iter()
            .map(|v| TemplateVariableResponse {
                name: v.name,
                description: v.description,
                default: v.default,
                required: v.required,
            })
            .collect(),
        tags: template.manifest.tags,
        content_preview: template.content.chars().take(500).collect(),
    }))
}

/// Fetch a template from a remote URL and add to registry.
///
/// POST /api/templates/fetch
pub async fn fetch_template(
    State(_state): State<AppState>,
    Json(_req): Json<FetchTemplateRequest>,
) -> Result<(StatusCode, Json<TemplateSummary>), (StatusCode, Json<serde_json::Value>)> {
    // This will use TemplateFetcher from T6
    // For now, return not implemented
    Err((
        StatusCode::NOT_IMPLEMENTED,
        Json(serde_json::json!({ "error": "Remote fetching not yet implemented" })),
    ))
}

// ============================================================================
// Project Types
// ============================================================================

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
#[serde(rename_all = "camelCase")]
pub struct ProjectResponse {
    /// Project ID.
    pub id: String,
    /// Project name.
    pub name: String,
    /// Project description.
    pub description: String,
    /// Status.
    pub status: String,
    /// Template used (if any).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template: Option<String>,
    /// Whether the project is archived.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_archived: Option<bool>,
    /// Last updated timestamp.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
    /// Number of messages.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_count: Option<u32>,
}

/// Create a new project.
///
/// POST /api/projects
pub async fn create_project(
    State(state): State<AppState>,
    Json(req): Json<CreateProjectRequest>,
) -> Result<(StatusCode, Json<ProjectResponse>), (StatusCode, Json<serde_json::Value>)> {
    let _template_content = if let Some(ref template_name) = req.template {
        let _template = state.template_registry.get(template_name).ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                Json(
                    serde_json::json!({ "error": format!("Unknown template: {}", template_name) }),
                ),
            )
        })?;

        let mut vars = HashMap::new();
        vars.insert("project_name".to_string(), req.name.clone());
        vars.insert("description".to_string(), req.description.clone());

        let rendered = state
            .template_registry
            .render(template_name, &vars)
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": format!("Template error: {}", e) })),
                )
            })?;

        Some(rendered)
    } else {
        None
    };

    let id = uuid::Uuid::new_v4().to_string();
    let project = state
        .db
        .create_project(&id, &req.name, &req.description, req.template.as_deref())
        .await
        .map_err(|e| {
            tracing::error!("Failed to create project: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "Failed to create project" })),
            )
        })?;

    Ok((
        StatusCode::CREATED,
        Json(ProjectResponse {
            id: project.id,
            name: project.name,
            description: project.description,
            status: project.status.clone(),
            template: project.template_name,
            is_archived: Some(project.status == "archived"),
            updated_at: Some(project.updated_at),
            message_count: Some(0),
        }),
    ))
}

/// List all projects.
///
/// GET /api/projects
pub async fn list_projects(State(state): State<AppState>) -> Json<Vec<ProjectResponse>> {
    match state.db.list_active_projects().await {
        Ok(projects) => Json(
            projects
                .into_iter()
                .map(|p| ProjectResponse {
                    id: p.id,
                    name: p.name,
                    description: p.description,
                    status: p.status.clone(),
                    template: p.template_name,
                    is_archived: Some(p.status == "archived"),
                    updated_at: Some(p.updated_at),
                    message_count: None,
                })
                .collect(),
        ),
        Err(e) => {
            tracing::error!("Failed to list projects: {}", e);
            Json(vec![])
        }
    }
}

/// Get a project by ID.
///
/// GET /api/projects/:id
pub async fn get_project(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ProjectResponse>, (StatusCode, Json<serde_json::Value>)> {
    let project = state
        .db
        .get_project(&id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get project: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "Database error" })),
            )
        })?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": format!("Project {} not found", id) })),
            )
        })?;

    Ok(Json(ProjectResponse {
        id: project.id,
        name: project.name,
        description: project.description,
        status: project.status.clone(),
        template: project.template_name,
        is_archived: Some(project.status == "archived"),
        updated_at: Some(project.updated_at),
        message_count: None,
    }))
}

/// Cancel/delete a project.
///
/// DELETE /api/projects/:id
pub async fn cancel_project(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> (StatusCode, Json<serde_json::Value>) {
    match state.db.update_project_status(&id, "cancelled").await {
        Ok(()) => (
            StatusCode::OK,
            Json(serde_json::json!({ "status": "cancelled" })),
        ),
        Err(e) => {
            tracing::error!("Failed to cancel project: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "Failed to cancel project" })),
            )
        }
    }
}

/// Archive a project.
///
/// POST /api/projects/:id/archive
pub async fn archive_project(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> (StatusCode, Json<serde_json::Value>) {
    match state.db.update_project_status(&id, "archived").await {
        Ok(()) => (
            StatusCode::OK,
            Json(serde_json::json!({ "status": "archived" })),
        ),
        Err(e) => {
            tracing::error!("Failed to archive project: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "Failed to archive project" })),
            )
        }
    }
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
            description: "A test project".to_string(),
            status: "active".to_string(),
            template: None,
            is_archived: Some(false),
            updated_at: None,
            message_count: None,
        };
        let json = serde_json::to_string(&resp).expect("serialize");
        assert!(json.contains("active"));
        assert!(json.contains("isArchived"));
    }

    #[test]
    fn test_template_summary_serializes() {
        let summary = TemplateSummary {
            id: "rust-web".to_string(),
            name: "rust-web".to_string(),
            description: "Rust web project".to_string(),
            full_description: None,
            language: "rust".to_string(),
            category: "Web".to_string(),
            tags: vec!["backend".to_string(), "web".to_string()],
            use_count: 42,
            stars: Some(10),
            updated_at: "2025-01-01T00:00:00Z".to_string(),
            source_url: None,
            file_structure: None,
        };
        let json = serde_json::to_string(&summary).expect("serialize");
        assert!(json.contains("rust-web"));
        assert!(json.contains("backend"));
        assert!(json.contains("useCount"));
        assert!(!json.contains("docker_image"));
    }

    #[test]
    fn test_template_detail_serializes() {
        let detail = TemplateDetail {
            name: "nuxt-app".to_string(),
            description: "Nuxt 3 application".to_string(),
            language: "typescript".to_string(),
            docker_image: "node:20".to_string(),
            variables: vec![TemplateVariableResponse {
                name: "project_name".to_string(),
                description: "Name of the project".to_string(),
                default: None,
                required: true,
            }],
            tags: vec!["frontend".to_string()],
            content_preview: "# Template content...".to_string(),
        };
        let json = serde_json::to_string(&detail).expect("serialize");
        assert!(json.contains("nuxt-app"));
        assert!(json.contains("node:20"));
        assert!(json.contains("project_name"));
        assert!(json.contains("content_preview"));
    }

    #[test]
    fn test_fetch_template_request_deserializes() {
        let json = r#"{"url": "https://example.com/template.md"}"#;
        let req: FetchTemplateRequest = serde_json::from_str(json).expect("parse");
        assert_eq!(req.url, "https://example.com/template.md");
    }
}
