//! Memory API route handlers for project memory, decisions, and branching.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// ============================================================================
// Request/Response Types
// ============================================================================

/// Response for project memory content.
#[derive(Debug, Serialize)]
pub struct MemoryResponse {
    /// Project name.
    pub project_name: String,
    /// Summary section content.
    pub summary: String,
    /// Key decisions.
    pub key_decisions: Vec<DecisionItemResponse>,
    /// Architecture components.
    pub architecture: Vec<ArchitectureItemResponse>,
    /// Gotchas and lessons.
    pub gotchas: Vec<GotchaItemResponse>,
    /// Rejected approaches.
    pub rejected_approaches: Vec<RejectedItemResponse>,
    /// Active context.
    pub active_context: ActiveContextResponse,
}

/// A key decision item response.
#[derive(Debug, Serialize)]
pub struct DecisionItemResponse {
    /// Date of the decision.
    pub date: String,
    /// The decision made.
    pub decision: String,
    /// Rationale for the decision.
    pub rationale: String,
}

/// An architecture component response.
#[derive(Debug, Serialize)]
pub struct ArchitectureItemResponse {
    /// Component name.
    pub component: String,
    /// Component description.
    pub description: String,
}

/// A gotcha item response.
#[derive(Debug, Serialize)]
pub struct GotchaItemResponse {
    /// The gotcha.
    pub gotcha: String,
    /// Context or explanation.
    pub context: String,
}

/// A rejected approach response.
#[derive(Debug, Serialize)]
pub struct RejectedItemResponse {
    /// The approach that was rejected.
    pub approach: String,
    /// Why it was rejected.
    pub reason: String,
}

/// Active context response.
#[derive(Debug, Serialize)]
pub struct ActiveContextResponse {
    /// Current task.
    pub current_task: Option<String>,
    /// Current blockers.
    pub blockers: Option<String>,
    /// Next steps.
    pub next_steps: Option<String>,
}

/// Request to update project memory.
#[derive(Debug, Deserialize)]
pub struct UpdateMemoryRequest {
    /// Summary section content.
    pub summary: Option<String>,
    /// Active context update.
    pub active_context: Option<ActiveContextUpdate>,
}

/// Active context update fields.
#[derive(Debug, Deserialize)]
pub struct ActiveContextUpdate {
    /// Current task.
    pub current_task: Option<String>,
    /// Current blockers.
    pub blockers: Option<String>,
    /// Next steps.
    pub next_steps: Option<String>,
}

/// Decision log entry response.
#[derive(Debug, Serialize)]
pub struct DecisionEntryResponse {
    /// Unique identifier.
    pub id: String,
    /// Timestamp.
    pub timestamp: String,
    /// Conversation ID.
    pub conversation_id: String,
    /// Message ID.
    pub message_id: String,
    /// File path affected (if any).
    pub file_path: Option<String>,
    /// Type of change.
    pub change_type: String,
    /// Summary of the decision.
    pub summary: String,
    /// Reasoning.
    pub reasoning: String,
    /// Agent that made the decision.
    pub agent: String,
    /// Confidence level.
    pub confidence: f32,
}

/// Paginated response for decisions.
#[derive(Debug, Serialize)]
pub struct DecisionListResponse {
    /// Decision entries.
    pub decisions: Vec<DecisionEntryResponse>,
    /// Total count.
    pub total: usize,
    /// Current page.
    pub page: usize,
    /// Items per page.
    pub per_page: usize,
}

/// Pagination query parameters.
#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    /// Page number (1-indexed).
    #[serde(default = "default_page")]
    pub page: usize,
    /// Items per page.
    #[serde(default = "default_per_page")]
    pub per_page: usize,
}

fn default_page() -> usize {
    1
}

fn default_per_page() -> usize {
    50
}

/// Request for why command.
#[derive(Debug, Deserialize)]
pub struct WhyRequest {
    /// Target type: "file" or "decision".
    pub target: String,
    /// Path or ID.
    pub path: String,
}

/// Response for why command.
#[derive(Debug, Serialize)]
pub struct WhyResponse {
    /// Target queried.
    pub target: String,
    /// Related decisions.
    pub decisions: Vec<DecisionEntryResponse>,
    /// Human-readable summary.
    pub summary: String,
}

/// Branch response.
#[derive(Debug, Serialize)]
pub struct BranchResponse {
    /// Branch ID.
    pub id: String,
    /// Branch name.
    pub name: String,
    /// Project ID.
    pub project_id: String,
    /// Creation timestamp.
    pub created_at: String,
    /// Optional description.
    pub description: Option<String>,
    /// Git reference.
    pub git_ref: String,
    /// Container snapshot ID (if any).
    pub container_snapshot: Option<String>,
}

/// Request to create a branch.
#[derive(Debug, Deserialize)]
pub struct CreateBranchRequest {
    /// Branch name.
    pub name: String,
    /// Optional description.
    pub description: Option<String>,
}

/// Request to restore a branch.
#[derive(Debug, Deserialize)]
pub struct RestoreBranchRequest {
    /// Whether to create a backup of current state first.
    #[serde(default)]
    pub create_backup: bool,
}

/// State for memory routes (includes project root mapping).
#[derive(Clone)]
pub struct MemoryState {
    /// Base directory where projects are stored.
    pub projects_dir: PathBuf,
}

impl MemoryState {
    /// Create new memory state.
    pub fn new(projects_dir: impl Into<PathBuf>) -> Self {
        Self {
            projects_dir: projects_dir.into(),
        }
    }

    /// Get the project root path for a project ID.
    fn project_path(&self, project_id: &str) -> PathBuf {
        self.projects_dir.join(project_id)
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

fn error_response(status: StatusCode, message: &str) -> (StatusCode, Json<serde_json::Value>) {
    (status, Json(serde_json::json!({ "error": message })))
}

// ============================================================================
// Route Handlers
// ============================================================================

/// Get project memory file.
///
/// GET /api/projects/:id/memory
pub async fn get_memory(
    State(state): State<MemoryState>,
    Path(project_id): Path<String>,
) -> Result<Json<MemoryResponse>, (StatusCode, Json<serde_json::Value>)> {
    use cuttlefish_agents::memory::ProjectMemory;

    let project_path = state.project_path(&project_id);
    if !project_path.exists() {
        return Err(error_response(
            StatusCode::NOT_FOUND,
            &format!("Project not found: {project_id}"),
        ));
    }

    let memory_path = ProjectMemory::default_path(&project_path);
    let memory = if memory_path.exists() {
        ProjectMemory::load(&memory_path).map_err(|e| {
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                &format!("Failed to load memory: {e}"),
            )
        })?
    } else {
        ProjectMemory::new(&project_id)
    };

    Ok(Json(MemoryResponse {
        project_name: memory.project_name,
        summary: memory.summary,
        key_decisions: memory
            .key_decisions
            .into_iter()
            .map(|d| DecisionItemResponse {
                date: d.date,
                decision: d.decision,
                rationale: d.rationale,
            })
            .collect(),
        architecture: memory
            .architecture
            .into_iter()
            .map(|a| ArchitectureItemResponse {
                component: a.component,
                description: a.description,
            })
            .collect(),
        gotchas: memory
            .gotchas
            .into_iter()
            .map(|g| GotchaItemResponse {
                gotcha: g.gotcha,
                context: g.context,
            })
            .collect(),
        rejected_approaches: memory
            .rejected_approaches
            .into_iter()
            .map(|r| RejectedItemResponse {
                approach: r.approach,
                reason: r.reason,
            })
            .collect(),
        active_context: ActiveContextResponse {
            current_task: memory.active_context.current_task,
            blockers: memory.active_context.blockers,
            next_steps: memory.active_context.next_steps,
        },
    }))
}

/// Update project memory file.
///
/// PUT /api/projects/:id/memory
pub async fn update_memory(
    State(state): State<MemoryState>,
    Path(project_id): Path<String>,
    Json(req): Json<UpdateMemoryRequest>,
) -> Result<Json<MemoryResponse>, (StatusCode, Json<serde_json::Value>)> {
    use cuttlefish_agents::memory::ProjectMemory;
    use cuttlefish_agents::memory::file::ActiveContextData;

    let project_path = state.project_path(&project_id);
    if !project_path.exists() {
        return Err(error_response(
            StatusCode::NOT_FOUND,
            &format!("Project not found: {project_id}"),
        ));
    }

    let memory_path = ProjectMemory::default_path(&project_path);
    let mut memory = if memory_path.exists() {
        ProjectMemory::load(&memory_path).map_err(|e| {
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                &format!("Failed to load memory: {e}"),
            )
        })?
    } else {
        ProjectMemory::new(&project_id)
    };

    // Apply updates
    if let Some(summary) = req.summary {
        memory.summary = summary;
    }

    if let Some(ctx) = req.active_context {
        memory.active_context = ActiveContextData {
            current_task: ctx.current_task.or(memory.active_context.current_task),
            blockers: ctx.blockers.or(memory.active_context.blockers),
            next_steps: ctx.next_steps.or(memory.active_context.next_steps),
        };
    }

    memory.save(&memory_path).map_err(|e| {
        error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!("Failed to save memory: {e}"),
        )
    })?;

    // Return updated memory
    get_memory(State(state), Path(project_id)).await
}

/// List decisions with pagination.
///
/// GET /api/projects/:id/decisions
pub async fn list_decisions(
    State(state): State<MemoryState>,
    Path(project_id): Path<String>,
    Query(pagination): Query<PaginationQuery>,
) -> Result<Json<DecisionListResponse>, (StatusCode, Json<serde_json::Value>)> {
    use cuttlefish_agents::memory::DecisionLog;

    let project_path = state.project_path(&project_id);
    if !project_path.exists() {
        return Err(error_response(
            StatusCode::NOT_FOUND,
            &format!("Project not found: {project_id}"),
        ));
    }

    let log_path = DecisionLog::default_path(&project_path);
    let log = DecisionLog::new(&log_path);

    let all_entries = log.read_all().map_err(|e| {
        error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!("Failed to read decisions: {e}"),
        )
    })?;

    let total = all_entries.len();
    let page = pagination.page.max(1);
    let per_page = pagination.per_page.clamp(1, 100);
    let start = (page - 1) * per_page;

    let decisions: Vec<DecisionEntryResponse> = all_entries
        .into_iter()
        .skip(start)
        .take(per_page)
        .map(|e| DecisionEntryResponse {
            id: e.id.to_string(),
            timestamp: e.timestamp.to_rfc3339(),
            conversation_id: e.conversation_id,
            message_id: e.message_id,
            file_path: e.file_path,
            change_type: format!("{:?}", e.change_type).to_lowercase(),
            summary: e.summary,
            reasoning: e.reasoning,
            agent: e.agent,
            confidence: e.confidence,
        })
        .collect();

    Ok(Json(DecisionListResponse {
        decisions,
        total,
        page,
        per_page,
    }))
}

/// Get a single decision by ID.
///
/// GET /api/projects/:id/decisions/:decision_id
pub async fn get_decision(
    State(state): State<MemoryState>,
    Path((project_id, decision_id)): Path<(String, String)>,
) -> Result<Json<DecisionEntryResponse>, (StatusCode, Json<serde_json::Value>)> {
    use cuttlefish_agents::memory::DecisionLog;

    let project_path = state.project_path(&project_id);
    if !project_path.exists() {
        return Err(error_response(
            StatusCode::NOT_FOUND,
            &format!("Project not found: {project_id}"),
        ));
    }

    let log_path = DecisionLog::default_path(&project_path);
    let log = DecisionLog::new(&log_path);

    let all_entries = log.read_all().map_err(|e| {
        error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!("Failed to read decisions: {e}"),
        )
    })?;

    let entry = all_entries
        .into_iter()
        .find(|e| e.id.to_string() == decision_id)
        .ok_or_else(|| {
            error_response(
                StatusCode::NOT_FOUND,
                &format!("Decision not found: {decision_id}"),
            )
        })?;

    Ok(Json(DecisionEntryResponse {
        id: entry.id.to_string(),
        timestamp: entry.timestamp.to_rfc3339(),
        conversation_id: entry.conversation_id,
        message_id: entry.message_id,
        file_path: entry.file_path,
        change_type: format!("{:?}", entry.change_type).to_lowercase(),
        summary: entry.summary,
        reasoning: entry.reasoning,
        agent: entry.agent,
        confidence: entry.confidence,
    }))
}

/// Query why command.
///
/// POST /api/projects/:id/why
pub async fn query_why(
    State(state): State<MemoryState>,
    Path(project_id): Path<String>,
    Json(req): Json<WhyRequest>,
) -> Result<Json<WhyResponse>, (StatusCode, Json<serde_json::Value>)> {
    use cuttlefish_agents::memory::{DecisionIndex, DecisionLog, WhyTarget, why};

    let project_path = state.project_path(&project_id);
    if !project_path.exists() {
        return Err(error_response(
            StatusCode::NOT_FOUND,
            &format!("Project not found: {project_id}"),
        ));
    }

    let log_path = DecisionLog::default_path(&project_path);
    let log = DecisionLog::new(&log_path);

    let index = DecisionIndex::from_log(&log).map_err(|e| {
        error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!("Failed to build decision index: {e}"),
        )
    })?;

    let target = match req.target.as_str() {
        "file" => WhyTarget::File(req.path.clone()),
        "decision" => WhyTarget::Decision(req.path.clone()),
        _ => {
            return Err(error_response(
                StatusCode::BAD_REQUEST,
                "Invalid target type. Use 'file' or 'decision'",
            ));
        }
    };

    let explanation = why(&index, target);

    Ok(Json(WhyResponse {
        target: explanation.target,
        decisions: explanation
            .decisions
            .into_iter()
            .map(|e| DecisionEntryResponse {
                id: e.id.to_string(),
                timestamp: e.timestamp.to_rfc3339(),
                conversation_id: e.conversation_id,
                message_id: e.message_id,
                file_path: e.file_path,
                change_type: format!("{:?}", e.change_type).to_lowercase(),
                summary: e.summary,
                reasoning: e.reasoning,
                agent: e.agent,
                confidence: e.confidence,
            })
            .collect(),
        summary: explanation.summary,
    }))
}

/// List branches for a project.
///
/// GET /api/projects/:id/branches
pub async fn list_branches(
    State(state): State<MemoryState>,
    Path(project_id): Path<String>,
) -> Result<Json<Vec<BranchResponse>>, (StatusCode, Json<serde_json::Value>)> {
    use cuttlefish_agents::memory::BranchStore;

    let project_path = state.project_path(&project_id);
    if !project_path.exists() {
        return Err(error_response(
            StatusCode::NOT_FOUND,
            &format!("Project not found: {project_id}"),
        ));
    }

    let mut store = BranchStore::new(&project_path);
    store.load().map_err(|e| {
        error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!("Failed to load branches: {e}"),
        )
    })?;

    let branches: Vec<BranchResponse> = store
        .list_branches(&project_id)
        .into_iter()
        .map(|b| BranchResponse {
            id: b.id.to_string(),
            name: b.name.clone(),
            project_id: b.project_id.clone(),
            created_at: b.created_at.to_rfc3339(),
            description: b.description.clone(),
            git_ref: b.git_ref.clone(),
            container_snapshot: b.container_snapshot.clone(),
        })
        .collect();

    Ok(Json(branches))
}

/// Create a new branch.
///
/// POST /api/projects/:id/branches
pub async fn create_branch(
    State(state): State<MemoryState>,
    Path(project_id): Path<String>,
    Json(req): Json<CreateBranchRequest>,
) -> Result<(StatusCode, Json<BranchResponse>), (StatusCode, Json<serde_json::Value>)> {
    use cuttlefish_agents::memory::BranchStore;

    let project_path = state.project_path(&project_id);
    if !project_path.exists() {
        return Err(error_response(
            StatusCode::NOT_FOUND,
            &format!("Project not found: {project_id}"),
        ));
    }

    let mut store = BranchStore::new(&project_path);
    store.load().map_err(|e| {
        error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!("Failed to load branches: {e}"),
        )
    })?;

    // Use "HEAD" as git ref for now (actual git operations handled elsewhere)
    let branch = store
        .create_branch(&project_id, &req.name, req.description.as_deref(), "HEAD")
        .map_err(|e| {
            error_response(
                StatusCode::BAD_REQUEST,
                &format!("Failed to create branch: {e}"),
            )
        })?;

    Ok((
        StatusCode::CREATED,
        Json(BranchResponse {
            id: branch.id.to_string(),
            name: branch.name,
            project_id: branch.project_id,
            created_at: branch.created_at.to_rfc3339(),
            description: branch.description,
            git_ref: branch.git_ref,
            container_snapshot: branch.container_snapshot,
        }),
    ))
}

/// Restore a branch.
///
/// POST /api/projects/:id/branches/:name/restore
pub async fn restore_branch(
    State(state): State<MemoryState>,
    Path((project_id, branch_name)): Path<(String, String)>,
    Json(req): Json<RestoreBranchRequest>,
) -> Result<Json<BranchResponse>, (StatusCode, Json<serde_json::Value>)> {
    use cuttlefish_agents::memory::BranchStore;

    let project_path = state.project_path(&project_id);
    if !project_path.exists() {
        return Err(error_response(
            StatusCode::NOT_FOUND,
            &format!("Project not found: {project_id}"),
        ));
    }

    let mut store = BranchStore::new(&project_path);
    store.load().map_err(|e| {
        error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!("Failed to load branches: {e}"),
        )
    })?;

    let branch = store
        .restore_branch(&project_id, &branch_name, req.create_backup)
        .map_err(|e| {
            error_response(
                StatusCode::BAD_REQUEST,
                &format!("Failed to restore branch: {e}"),
            )
        })?;

    Ok(Json(BranchResponse {
        id: branch.id.to_string(),
        name: branch.name,
        project_id: branch.project_id,
        created_at: branch.created_at.to_rfc3339(),
        description: branch.description,
        git_ref: branch.git_ref,
        container_snapshot: branch.container_snapshot,
    }))
}

/// Delete a branch.
///
/// DELETE /api/projects/:id/branches/:name
pub async fn delete_branch(
    State(state): State<MemoryState>,
    Path((project_id, branch_name)): Path<(String, String)>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    use cuttlefish_agents::memory::BranchStore;

    let project_path = state.project_path(&project_id);
    if !project_path.exists() {
        return Err(error_response(
            StatusCode::NOT_FOUND,
            &format!("Project not found: {project_id}"),
        ));
    }

    let mut store = BranchStore::new(&project_path);
    store.load().map_err(|e| {
        error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!("Failed to load branches: {e}"),
        )
    })?;

    store.delete_branch(&branch_name).map_err(|e| {
        error_response(
            StatusCode::BAD_REQUEST,
            &format!("Failed to delete branch: {e}"),
        )
    })?;

    Ok(StatusCode::NO_CONTENT)
}

/// Build memory routes router.
pub fn memory_router() -> axum::Router<MemoryState> {
    use axum::routing::{delete, get, post};

    axum::Router::new()
        .route(
            "/api/projects/{id}/memory",
            get(get_memory).put(update_memory),
        )
        .route("/api/projects/{id}/decisions", get(list_decisions))
        .route(
            "/api/projects/{id}/decisions/{decision_id}",
            get(get_decision),
        )
        .route("/api/projects/{id}/why", post(query_why))
        .route(
            "/api/projects/{id}/branches",
            get(list_branches).post(create_branch),
        )
        .route(
            "/api/projects/{id}/branches/{name}/restore",
            post(restore_branch),
        )
        .route("/api/projects/{id}/branches/{name}", delete(delete_branch))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pagination_defaults() {
        let json = r#"{}"#;
        let query: PaginationQuery = serde_json::from_str(json).expect("parse");
        assert_eq!(query.page, 1);
        assert_eq!(query.per_page, 50);
    }

    #[test]
    fn test_why_request_deserialize() {
        let json = r#"{"target": "file", "path": "src/main.rs"}"#;
        let req: WhyRequest = serde_json::from_str(json).expect("parse");
        assert_eq!(req.target, "file");
        assert_eq!(req.path, "src/main.rs");
    }

    #[test]
    fn test_create_branch_request_deserialize() {
        let json = r#"{"name": "pre-refactor", "description": "Before major refactor"}"#;
        let req: CreateBranchRequest = serde_json::from_str(json).expect("parse");
        assert_eq!(req.name, "pre-refactor");
        assert_eq!(req.description, Some("Before major refactor".to_string()));
    }

    #[test]
    fn test_restore_branch_request_deserialize() {
        let json = r#"{"create_backup": true}"#;
        let req: RestoreBranchRequest = serde_json::from_str(json).expect("parse");
        assert!(req.create_backup);

        let json = r#"{}"#;
        let req: RestoreBranchRequest = serde_json::from_str(json).expect("parse");
        assert!(!req.create_backup);
    }

    #[test]
    fn test_memory_response_serializes() {
        let resp = MemoryResponse {
            project_name: "test-project".to_string(),
            summary: "A test project".to_string(),
            key_decisions: vec![],
            architecture: vec![],
            gotchas: vec![],
            rejected_approaches: vec![],
            active_context: ActiveContextResponse {
                current_task: Some("Testing".to_string()),
                blockers: None,
                next_steps: None,
            },
        };
        let json = serde_json::to_string(&resp).expect("serialize");
        assert!(json.contains("test-project"));
        assert!(json.contains("Testing"));
    }

    #[test]
    fn test_decision_entry_response_serializes() {
        let resp = DecisionEntryResponse {
            id: "abc-123".to_string(),
            timestamp: "2024-01-15T10:00:00Z".to_string(),
            conversation_id: "conv-1".to_string(),
            message_id: "msg-1".to_string(),
            file_path: Some("src/main.rs".to_string()),
            change_type: "create".to_string(),
            summary: "Created main entry point".to_string(),
            reasoning: "Project needs an entry point".to_string(),
            agent: "coder".to_string(),
            confidence: 0.95,
        };
        let json = serde_json::to_string(&resp).expect("serialize");
        assert!(json.contains("abc-123"));
        assert!(json.contains("src/main.rs"));
    }

    #[test]
    fn test_branch_response_serializes() {
        let resp = BranchResponse {
            id: "branch-123".to_string(),
            name: "pre-refactor".to_string(),
            project_id: "project-1".to_string(),
            created_at: "2024-01-15T10:00:00Z".to_string(),
            description: Some("Before refactor".to_string()),
            git_ref: "HEAD".to_string(),
            container_snapshot: None,
        };
        let json = serde_json::to_string(&resp).expect("serialize");
        assert!(json.contains("pre-refactor"));
        assert!(json.contains("project-1"));
    }

    #[test]
    fn test_memory_state_project_path() {
        let state = MemoryState::new("/projects");
        let path = state.project_path("my-project");
        assert_eq!(path.to_string_lossy(), "/projects/my-project");
    }
}
