//! Safety API route handlers for checkpoints, approvals, and diff previews.
//!
//! Provides endpoints for:
//! - Checkpoint management (create, list, restore, delete)
//! - Action approval workflow (approve, reject pending actions)
//! - Diff preview for file changes

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    middleware::from_fn_with_state,
    response::Json,
    routing::{delete, get, post},
    Extension, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use cuttlefish_agents::safety::{
    ActionGate, ActionPreview, ActionType, Checkpoint, CheckpointComponents, CheckpointConfig,
    CheckpointId, CheckpointManager, CheckpointTrigger, ConfidenceScore, DiffStats, FileDiff,
    GateConfig, GateDecision, InMemoryCheckpointStore,
};

use crate::middleware::{AuthConfig, AuthenticatedUser};

// ============================================================================
// State
// ============================================================================

/// State for safety routes.
#[derive(Clone)]
pub struct SafetyState {
    /// Checkpoint manager.
    checkpoint_manager: Arc<CheckpointManager<InMemoryCheckpointStore>>,
    /// Pending actions awaiting approval.
    pending_actions: Arc<RwLock<HashMap<String, PendingAction>>>,
    /// Gate configuration.
    gate_config: Arc<RwLock<GateConfig>>,
    /// Base directory for projects.
    projects_dir: PathBuf,
    /// Auth configuration for protected routes.
    auth_config: AuthConfig,
}

impl SafetyState {
    /// Create a new safety state.
    pub fn new(projects_dir: impl Into<PathBuf>, auth_config: AuthConfig) -> Self {
        let store = InMemoryCheckpointStore::new();
        let config = CheckpointConfig::default();
        let manager = CheckpointManager::new(store, config);

        Self {
            checkpoint_manager: Arc::new(manager),
            pending_actions: Arc::new(RwLock::new(HashMap::new())),
            gate_config: Arc::new(RwLock::new(GateConfig::default())),
            projects_dir: projects_dir.into(),
            auth_config,
        }
    }

    /// Get the project path for a project ID.
    fn project_path(&self, project_id: &str) -> PathBuf {
        self.projects_dir.join(project_id)
    }
}

// ============================================================================
// Pending Action Types
// ============================================================================

/// A pending action awaiting user approval.
#[derive(Debug, Clone)]
pub struct PendingAction {
    /// Unique action ID.
    pub id: String,
    /// Project ID.
    pub project_id: String,
    /// Action type.
    pub action_type: ActionType,
    /// Action preview.
    pub preview: ActionPreview,
    /// Confidence score.
    pub confidence: ConfidenceScore,
    /// When the action was queued.
    pub created_at: DateTime<Utc>,
    /// Timeout for approval (in seconds).
    pub timeout_secs: u64,
    /// File diff (if applicable).
    pub diff: Option<FileDiff>,
}

/// Status of a pending action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionStatus {
    /// Awaiting user decision.
    Pending,
    /// User approved the action.
    Approved,
    /// User rejected the action.
    Rejected,
    /// Action timed out.
    TimedOut,
}

// ============================================================================
// Request/Response Types
// ============================================================================

/// Request to create a checkpoint.
#[derive(Debug, Deserialize)]
pub struct CreateCheckpointRequest {
    /// Human-readable description.
    pub description: String,
    /// Git reference (branch or commit).
    #[serde(default = "default_git_ref")]
    pub git_ref: String,
    /// Container snapshot ID (optional).
    pub container_snapshot_id: Option<String>,
}

fn default_git_ref() -> String {
    "HEAD".to_string()
}

/// Response for a checkpoint.
#[derive(Debug, Serialize)]
pub struct CheckpointResponse {
    /// Checkpoint ID.
    pub id: String,
    /// Project ID.
    pub project_id: String,
    /// Creation timestamp.
    pub created_at: String,
    /// Description.
    pub description: String,
    /// Trigger type.
    pub trigger: String,
    /// Git reference.
    pub git_ref: String,
    /// Container snapshot ID.
    pub container_snapshot_id: String,
    /// Memory backup path.
    pub memory_backup_path: String,
}

impl From<Checkpoint> for CheckpointResponse {
    fn from(cp: Checkpoint) -> Self {
        Self {
            id: cp.id.to_string(),
            project_id: cp.project_id,
            created_at: cp.created_at.to_rfc3339(),
            description: cp.description,
            trigger: format!("{}", cp.trigger),
            git_ref: cp.components.git_ref,
            container_snapshot_id: cp.components.container_snapshot_id,
            memory_backup_path: cp.components.memory_backup_path.to_string_lossy().to_string(),
        }
    }
}

/// Request to restore a checkpoint.
#[derive(Debug, Deserialize)]
pub struct RestoreCheckpointRequest {
    /// Whether to create a safety checkpoint before restoring.
    #[serde(default = "default_true")]
    pub create_safety_checkpoint: bool,
}

fn default_true() -> bool {
    true
}

/// Response for restore operation.
#[derive(Debug, Serialize)]
pub struct RestoreResponse {
    /// The restored checkpoint.
    pub restored_checkpoint: CheckpointResponse,
    /// Safety checkpoint created before restore (if any).
    pub safety_checkpoint: Option<CheckpointResponse>,
    /// Message.
    pub message: String,
}

/// Response for a pending action.
#[derive(Debug, Serialize)]
pub struct PendingActionResponse {
    /// Action ID.
    pub id: String,
    /// Project ID.
    pub project_id: String,
    /// Action type.
    pub action_type: String,
    /// Description.
    pub description: String,
    /// Affected path (if any).
    pub path: Option<String>,
    /// Command (if any).
    pub command: Option<String>,
    /// Confidence score (0.0 to 1.0).
    pub confidence: f32,
    /// Confidence reasoning.
    pub confidence_reasoning: String,
    /// When the action was queued.
    pub created_at: String,
    /// Timeout in seconds.
    pub timeout_secs: u64,
    /// Whether a diff is available.
    pub has_diff: bool,
}

impl From<&PendingAction> for PendingActionResponse {
    fn from(action: &PendingAction) -> Self {
        Self {
            id: action.id.clone(),
            project_id: action.project_id.clone(),
            action_type: action.action_type.name().to_string(),
            description: action.preview.description.clone(),
            path: action.preview.path.clone(),
            command: action.preview.command.clone(),
            confidence: action.confidence.value(),
            confidence_reasoning: action.confidence.reasoning().to_string(),
            created_at: action.created_at.to_rfc3339(),
            timeout_secs: action.timeout_secs,
            has_diff: action.diff.is_some(),
        }
    }
}

/// Response for action approval/rejection.
#[derive(Debug, Serialize)]
pub struct ActionDecisionResponse {
    /// Action ID.
    pub action_id: String,
    /// Decision made.
    pub decision: String,
    /// Message.
    pub message: String,
}

/// Response for diff preview.
#[derive(Debug, Serialize)]
pub struct DiffPreviewResponse {
    /// Action ID.
    pub action_id: String,
    /// File path.
    pub path: String,
    /// Whether this is a new file.
    pub is_new_file: bool,
    /// Whether this is a deletion.
    pub is_deletion: bool,
    /// Detected language.
    pub language: Option<String>,
    /// Diff statistics.
    pub stats: DiffStatsResponse,
    /// Unified diff format.
    pub unified_diff: String,
}

/// Diff statistics response.
#[derive(Debug, Serialize)]
pub struct DiffStatsResponse {
    /// Lines added.
    pub lines_added: usize,
    /// Lines removed.
    pub lines_removed: usize,
    /// Number of hunks.
    pub hunks: usize,
}

impl From<DiffStats> for DiffStatsResponse {
    fn from(stats: DiffStats) -> Self {
        Self {
            lines_added: stats.lines_added,
            lines_removed: stats.lines_removed,
            hunks: stats.hunks,
        }
    }
}

/// Request to update gate configuration.
#[derive(Debug, Deserialize)]
pub struct UpdateGateConfigRequest {
    /// Auto-approve threshold (0.0 to 1.0).
    pub auto_approve_threshold: Option<f32>,
    /// Prompt threshold (0.0 to 1.0).
    pub prompt_threshold: Option<f32>,
}

/// Response for gate configuration.
#[derive(Debug, Serialize)]
pub struct GateConfigResponse {
    /// Auto-approve threshold.
    pub auto_approve_threshold: f32,
    /// Prompt threshold.
    pub prompt_threshold: f32,
}

/// Request to undo operations.
#[derive(Debug, Deserialize)]
pub struct UndoRequest {
    /// Number of operations to undo (default: 1).
    #[serde(default = "default_undo_count")]
    pub count: usize,
}

fn default_undo_count() -> usize {
    1
}

/// Response for undo operation.
#[derive(Debug, Serialize)]
pub struct UndoResponse {
    /// Number of operations undone.
    pub operations_undone: usize,
    /// Message.
    pub message: String,
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
    20
}

// ============================================================================
// Error Handling
// ============================================================================

type ApiError = (StatusCode, Json<serde_json::Value>);

fn error_response(status: StatusCode, message: &str) -> ApiError {
    (status, Json(serde_json::json!({ "error": message })))
}

// ============================================================================
// Checkpoint Handlers
// ============================================================================

/// Create a checkpoint for a project.
///
/// POST /api/projects/:id/checkpoints
pub async fn create_checkpoint(
    State(state): State<SafetyState>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(project_id): Path<String>,
    Json(req): Json<CreateCheckpointRequest>,
) -> Result<(StatusCode, Json<CheckpointResponse>), ApiError> {
    let project_path = state.project_path(&project_id);
    if !project_path.exists() {
        return Err(error_response(
            StatusCode::NOT_FOUND,
            &format!("Project not found: {project_id}"),
        ));
    }

    let memory_backup_path = project_path.join(".cuttlefish/checkpoints/memory_backup.json");
    let container_snapshot_id = req
        .container_snapshot_id
        .unwrap_or_else(|| format!("snapshot-{}", Uuid::new_v4()));

    let components = CheckpointComponents::new(req.git_ref, container_snapshot_id, memory_backup_path);

    let trigger = CheckpointTrigger::Manual {
        user_id: user.user_id.clone(),
    };

    let checkpoint = state
        .checkpoint_manager
        .create_checkpoint(&project_id, &req.description, trigger, components)
        .await
        .map_err(|e| {
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                &format!("Failed to create checkpoint: {e}"),
            )
        })?;

    tracing::info!(
        project_id = %project_id,
        checkpoint_id = %checkpoint.id,
        user_id = %user.user_id,
        "Checkpoint created"
    );

    Ok((StatusCode::CREATED, Json(checkpoint.into())))
}

/// List checkpoints for a project.
///
/// GET /api/projects/:id/checkpoints
pub async fn list_checkpoints(
    State(state): State<SafetyState>,
    Path(project_id): Path<String>,
    Query(pagination): Query<PaginationQuery>,
) -> Result<Json<Vec<CheckpointResponse>>, ApiError> {
    let project_path = state.project_path(&project_id);
    if !project_path.exists() {
        return Err(error_response(
            StatusCode::NOT_FOUND,
            &format!("Project not found: {project_id}"),
        ));
    }

    let checkpoints = state
        .checkpoint_manager
        .list_checkpoints(&project_id)
        .await
        .map_err(|e| {
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                &format!("Failed to list checkpoints: {e}"),
            )
        })?;

    let page = pagination.page.max(1);
    let per_page = pagination.per_page.clamp(1, 100);
    let start = (page - 1) * per_page;

    let paginated: Vec<CheckpointResponse> = checkpoints
        .into_iter()
        .skip(start)
        .take(per_page)
        .map(CheckpointResponse::from)
        .collect();

    Ok(Json(paginated))
}

/// Restore a checkpoint (rollback).
///
/// POST /api/projects/:id/checkpoints/:checkpoint_id/restore
pub async fn restore_checkpoint(
    State(state): State<SafetyState>,
    Extension(user): Extension<AuthenticatedUser>,
    Path((project_id, checkpoint_id)): Path<(String, String)>,
    Json(req): Json<RestoreCheckpointRequest>,
) -> Result<Json<RestoreResponse>, ApiError> {
    let project_path = state.project_path(&project_id);
    if !project_path.exists() {
        return Err(error_response(
            StatusCode::NOT_FOUND,
            &format!("Project not found: {project_id}"),
        ));
    }

    let checkpoint_id = CheckpointId::from_string(&checkpoint_id);

    // Prepare current components for safety checkpoint if requested
    let current_components = if req.create_safety_checkpoint {
        let memory_backup_path = project_path.join(".cuttlefish/checkpoints/pre_rollback_memory.json");
        Some(CheckpointComponents::new(
            "HEAD",
            format!("pre-rollback-{}", Uuid::new_v4()),
            memory_backup_path,
        ))
    } else {
        None
    };

    let (target, safety) = state
        .checkpoint_manager
        .prepare_rollback(&project_id, &checkpoint_id, req.create_safety_checkpoint, current_components)
        .await
        .map_err(|e| {
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                &format!("Failed to prepare rollback: {e}"),
            )
        })?;

    tracing::info!(
        project_id = %project_id,
        checkpoint_id = %checkpoint_id,
        user_id = %user.user_id,
        safety_checkpoint = safety.is_some(),
        "Rollback prepared"
    );

    Ok(Json(RestoreResponse {
        restored_checkpoint: target.into(),
        safety_checkpoint: safety.map(CheckpointResponse::from),
        message: "Rollback prepared. Execute git checkout and container restore to complete.".to_string(),
    }))
}

/// Delete a checkpoint.
///
/// DELETE /api/projects/:id/checkpoints/:checkpoint_id
pub async fn delete_checkpoint(
    State(state): State<SafetyState>,
    Extension(user): Extension<AuthenticatedUser>,
    Path((project_id, checkpoint_id)): Path<(String, String)>,
) -> Result<StatusCode, ApiError> {
    let project_path = state.project_path(&project_id);
    if !project_path.exists() {
        return Err(error_response(
            StatusCode::NOT_FOUND,
            &format!("Project not found: {project_id}"),
        ));
    }

    let checkpoint_id = CheckpointId::from_string(&checkpoint_id);

    state
        .checkpoint_manager
        .delete_checkpoint(&checkpoint_id)
        .await
        .map_err(|e| {
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                &format!("Failed to delete checkpoint: {e}"),
            )
        })?;

    tracing::info!(
        project_id = %project_id,
        checkpoint_id = %checkpoint_id,
        user_id = %user.user_id,
        "Checkpoint deleted"
    );

    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Action Approval Handlers
// ============================================================================

/// List pending actions for a project.
///
/// GET /api/projects/:id/actions/pending
pub async fn list_pending_actions(
    State(state): State<SafetyState>,
    Path(project_id): Path<String>,
) -> Result<Json<Vec<PendingActionResponse>>, ApiError> {
    let project_path = state.project_path(&project_id);
    if !project_path.exists() {
        return Err(error_response(
            StatusCode::NOT_FOUND,
            &format!("Project not found: {project_id}"),
        ));
    }

    let pending = state.pending_actions.read().await;
    let actions: Vec<PendingActionResponse> = pending
        .values()
        .filter(|a| a.project_id == project_id)
        .map(PendingActionResponse::from)
        .collect();

    Ok(Json(actions))
}

/// Approve a pending action.
///
/// POST /api/actions/:action_id/approve
pub async fn approve_action(
    State(state): State<SafetyState>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(action_id): Path<String>,
) -> Result<Json<ActionDecisionResponse>, ApiError> {
    let mut pending = state.pending_actions.write().await;

    let action = pending.remove(&action_id).ok_or_else(|| {
        error_response(
            StatusCode::NOT_FOUND,
            &format!("Pending action not found: {action_id}"),
        )
    })?;

    tracing::info!(
        action_id = %action_id,
        project_id = %action.project_id,
        user_id = %user.user_id,
        action_type = %action.action_type.name(),
        "Action approved"
    );

    Ok(Json(ActionDecisionResponse {
        action_id,
        decision: "approved".to_string(),
        message: format!("Action approved by user {}", user.user_id),
    }))
}

/// Reject a pending action.
///
/// POST /api/actions/:action_id/reject
pub async fn reject_action(
    State(state): State<SafetyState>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(action_id): Path<String>,
) -> Result<Json<ActionDecisionResponse>, ApiError> {
    let mut pending = state.pending_actions.write().await;

    let action = pending.remove(&action_id).ok_or_else(|| {
        error_response(
            StatusCode::NOT_FOUND,
            &format!("Pending action not found: {action_id}"),
        )
    })?;

    tracing::info!(
        action_id = %action_id,
        project_id = %action.project_id,
        user_id = %user.user_id,
        action_type = %action.action_type.name(),
        "Action rejected"
    );

    Ok(Json(ActionDecisionResponse {
        action_id,
        decision: "rejected".to_string(),
        message: format!("Action rejected by user {}", user.user_id),
    }))
}

/// Get diff preview for an action.
///
/// GET /api/actions/:action_id/diff
pub async fn get_action_diff(
    State(state): State<SafetyState>,
    Path(action_id): Path<String>,
) -> Result<Json<DiffPreviewResponse>, ApiError> {
    let pending = state.pending_actions.read().await;

    let action = pending.get(&action_id).ok_or_else(|| {
        error_response(
            StatusCode::NOT_FOUND,
            &format!("Pending action not found: {action_id}"),
        )
    })?;

    let diff = action.diff.as_ref().ok_or_else(|| {
        error_response(
            StatusCode::NOT_FOUND,
            "No diff available for this action",
        )
    })?;

    Ok(Json(DiffPreviewResponse {
        action_id,
        path: diff.path.clone(),
        is_new_file: diff.is_new_file(),
        is_deletion: diff.is_deletion(),
        language: diff.language.clone(),
        stats: diff.stats.clone().into(),
        unified_diff: diff.to_unified_diff(),
    }))
}

// ============================================================================
// Gate Configuration Handlers
// ============================================================================

/// Get gate configuration for a project.
///
/// GET /api/projects/:id/safety/config
pub async fn get_gate_config(
    State(state): State<SafetyState>,
    Path(project_id): Path<String>,
) -> Result<Json<GateConfigResponse>, ApiError> {
    let project_path = state.project_path(&project_id);
    if !project_path.exists() {
        return Err(error_response(
            StatusCode::NOT_FOUND,
            &format!("Project not found: {project_id}"),
        ));
    }

    let config = state.gate_config.read().await;

    Ok(Json(GateConfigResponse {
        auto_approve_threshold: config.auto_approve_threshold,
        prompt_threshold: config.prompt_threshold,
    }))
}

/// Update gate configuration for a project.
///
/// PUT /api/projects/:id/safety/config
pub async fn update_gate_config(
    State(state): State<SafetyState>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(project_id): Path<String>,
    Json(req): Json<UpdateGateConfigRequest>,
) -> Result<Json<GateConfigResponse>, ApiError> {
    let project_path = state.project_path(&project_id);
    if !project_path.exists() {
        return Err(error_response(
            StatusCode::NOT_FOUND,
            &format!("Project not found: {project_id}"),
        ));
    }

    let mut config = state.gate_config.write().await;

    if let Some(threshold) = req.auto_approve_threshold {
        config.auto_approve_threshold = threshold.clamp(0.0, 1.0);
    }

    if let Some(threshold) = req.prompt_threshold {
        config.prompt_threshold = threshold.clamp(0.0, 1.0);
    }

    tracing::info!(
        project_id = %project_id,
        user_id = %user.user_id,
        auto_approve = config.auto_approve_threshold,
        prompt = config.prompt_threshold,
        "Gate config updated"
    );

    Ok(Json(GateConfigResponse {
        auto_approve_threshold: config.auto_approve_threshold,
        prompt_threshold: config.prompt_threshold,
    }))
}

// ============================================================================
// Undo Handler
// ============================================================================

/// Undo last N operations.
///
/// POST /api/projects/:id/undo
pub async fn undo_operations(
    State(state): State<SafetyState>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(project_id): Path<String>,
    Json(req): Json<UndoRequest>,
) -> Result<Json<UndoResponse>, ApiError> {
    let project_path = state.project_path(&project_id);
    if !project_path.exists() {
        return Err(error_response(
            StatusCode::NOT_FOUND,
            &format!("Project not found: {project_id}"),
        ));
    }

    // For now, undo is a placeholder that would integrate with an operation journal
    // In a full implementation, this would track file operations and reverse them
    let count = req.count.min(10); // Cap at 10 operations

    tracing::info!(
        project_id = %project_id,
        user_id = %user.user_id,
        count = count,
        "Undo requested"
    );

    Ok(Json(UndoResponse {
        operations_undone: 0, // Placeholder - would be actual count
        message: format!(
            "Undo functionality requires operation journal. Requested {} operations.",
            count
        ),
    }))
}

// ============================================================================
// Router
// ============================================================================

/// Build the safety routes router.
pub fn safety_router(state: SafetyState) -> Router {
    let auth_config = state.auth_config.clone();

    // All safety routes require authentication
    Router::new()
        // Checkpoint routes
        .route(
            "/api/projects/{id}/checkpoints",
            get(list_checkpoints).post(create_checkpoint),
        )
        .route(
            "/api/projects/{id}/checkpoints/{checkpoint_id}/restore",
            post(restore_checkpoint),
        )
        .route(
            "/api/projects/{id}/checkpoints/{checkpoint_id}",
            delete(delete_checkpoint),
        )
        // Pending action routes
        .route(
            "/api/projects/{id}/actions/pending",
            get(list_pending_actions),
        )
        // Action approval routes
        .route("/api/actions/{action_id}/approve", post(approve_action))
        .route("/api/actions/{action_id}/reject", post(reject_action))
        .route("/api/actions/{action_id}/diff", get(get_action_diff))
        // Gate configuration routes
        .route(
            "/api/projects/{id}/safety/config",
            get(get_gate_config).put(update_gate_config),
        )
        // Undo route
        .route("/api/projects/{id}/undo", post(undo_operations))
        .layer(from_fn_with_state(auth_config, crate::middleware::require_auth))
        .with_state(state)
}

// ============================================================================
// Helper Functions for Agent Integration
// ============================================================================

/// Queue an action for approval.
///
/// Returns the action ID if queued, or None if auto-approved.
pub async fn queue_pending_action(
    state: &SafetyState,
    project_id: &str,
    action_type: ActionType,
    preview: ActionPreview,
    confidence: ConfidenceScore,
    diff: Option<FileDiff>,
) -> Option<String> {
    let config = state.gate_config.read().await;
    let gate = ActionGate::new(config.clone());

    let decision = gate.evaluate(action_type, &confidence, preview.clone());

    match decision {
        GateDecision::AutoApprove => {
            tracing::debug!(
                project_id = %project_id,
                action_type = %action_type.name(),
                confidence = confidence.value(),
                "Action auto-approved"
            );
            None
        }
        GateDecision::PromptUser { .. } => {
            let action_id = Uuid::new_v4().to_string();
            let confidence_value = confidence.value();
            let action = PendingAction {
                id: action_id.clone(),
                project_id: project_id.to_string(),
                action_type,
                preview,
                confidence,
                created_at: Utc::now(),
                timeout_secs: 300, // 5 minutes default
                diff,
            };

            let mut pending = state.pending_actions.write().await;
            pending.insert(action_id.clone(), action);

            tracing::info!(
                project_id = %project_id,
                action_id = %action_id,
                action_type = %action_type.name(),
                confidence = confidence_value,
                "Action queued for approval"
            );

            Some(action_id)
        }
        GateDecision::Block { reason } => {
            tracing::warn!(
                project_id = %project_id,
                action_type = %action_type.name(),
                confidence = confidence.value(),
                reason = %reason,
                "Action blocked"
            );
            None
        }
    }
}

/// Check if an action has been approved.
pub async fn is_action_approved(state: &SafetyState, action_id: &str) -> bool {
    let pending = state.pending_actions.read().await;
    !pending.contains_key(action_id)
}

/// Wait for action approval with timeout.
pub async fn wait_for_approval(
    state: &SafetyState,
    action_id: &str,
    timeout_secs: u64,
) -> Result<bool, &'static str> {
    let timeout = std::time::Duration::from_secs(timeout_secs);
    let start = std::time::Instant::now();
    let poll_interval = std::time::Duration::from_millis(500);

    while start.elapsed() < timeout {
        if is_action_approved(state, action_id).await {
            return Ok(true);
        }
        tokio::time::sleep(poll_interval).await;
    }

    // Timeout - remove from pending
    let mut pending = state.pending_actions.write().await;
    pending.remove(action_id);

    Err("Action approval timed out")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_state() -> SafetyState {
        let auth_config = AuthConfig::new(vec![0u8; 32]);
        SafetyState::new("/tmp/test_projects", auth_config)
    }

    #[test]
    fn test_checkpoint_response_from() {
        let components = CheckpointComponents::new("main", "snap-123", "/tmp/mem.json");
        let checkpoint = Checkpoint::new(
            "project-1",
            "Test checkpoint",
            CheckpointTrigger::Scheduled,
            components,
        );

        let response: CheckpointResponse = checkpoint.into();
        assert_eq!(response.project_id, "project-1");
        assert_eq!(response.description, "Test checkpoint");
        assert_eq!(response.git_ref, "main");
    }

    #[test]
    fn test_pending_action_response_from() {
        let preview = ActionPreview::new("Write file", ActionType::FileWrite)
            .with_path("src/main.rs");
        let confidence = ConfidenceScore::medium("Test confidence");

        let action = PendingAction {
            id: "action-123".to_string(),
            project_id: "project-1".to_string(),
            action_type: ActionType::FileWrite,
            preview,
            confidence,
            created_at: Utc::now(),
            timeout_secs: 300,
            diff: None,
        };

        let response = PendingActionResponse::from(&action);
        assert_eq!(response.id, "action-123");
        assert_eq!(response.project_id, "project-1");
        assert_eq!(response.action_type, "File Write");
        assert!(!response.has_diff);
    }

    #[test]
    fn test_diff_stats_response_from() {
        let stats = DiffStats {
            lines_added: 10,
            lines_removed: 5,
            hunks: 2,
        };

        let response: DiffStatsResponse = stats.into();
        assert_eq!(response.lines_added, 10);
        assert_eq!(response.lines_removed, 5);
        assert_eq!(response.hunks, 2);
    }

    #[test]
    fn test_create_checkpoint_request_deserialize() {
        let json = r#"{"description": "Before refactor"}"#;
        let req: CreateCheckpointRequest = serde_json::from_str(json).expect("parse");
        assert_eq!(req.description, "Before refactor");
        assert_eq!(req.git_ref, "HEAD");
        assert!(req.container_snapshot_id.is_none());
    }

    #[test]
    fn test_restore_checkpoint_request_deserialize() {
        let json = r#"{}"#;
        let req: RestoreCheckpointRequest = serde_json::from_str(json).expect("parse");
        assert!(req.create_safety_checkpoint);

        let json = r#"{"create_safety_checkpoint": false}"#;
        let req: RestoreCheckpointRequest = serde_json::from_str(json).expect("parse");
        assert!(!req.create_safety_checkpoint);
    }

    #[test]
    fn test_update_gate_config_request_deserialize() {
        let json = r#"{"auto_approve_threshold": 0.85}"#;
        let req: UpdateGateConfigRequest = serde_json::from_str(json).expect("parse");
        assert_eq!(req.auto_approve_threshold, Some(0.85));
        assert!(req.prompt_threshold.is_none());
    }

    #[test]
    fn test_undo_request_deserialize() {
        let json = r#"{}"#;
        let req: UndoRequest = serde_json::from_str(json).expect("parse");
        assert_eq!(req.count, 1);

        let json = r#"{"count": 5}"#;
        let req: UndoRequest = serde_json::from_str(json).expect("parse");
        assert_eq!(req.count, 5);
    }

    #[test]
    fn test_pagination_query_defaults() {
        let json = r#"{}"#;
        let query: PaginationQuery = serde_json::from_str(json).expect("parse");
        assert_eq!(query.page, 1);
        assert_eq!(query.per_page, 20);
    }

    #[tokio::test]
    async fn test_queue_pending_action_auto_approve() {
        let state = make_test_state();
        let preview = ActionPreview::new("Safe action", ActionType::FileWrite);
        let confidence = ConfidenceScore::high("Very confident");

        let result = queue_pending_action(
            &state,
            "project-1",
            ActionType::FileWrite,
            preview,
            confidence,
            None,
        )
        .await;

        // High confidence should auto-approve
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_queue_pending_action_needs_approval() {
        let state = make_test_state();
        let preview = ActionPreview::new("Risky action", ActionType::FileWrite);
        let confidence = ConfidenceScore::medium("Somewhat confident");

        let result = queue_pending_action(
            &state,
            "project-1",
            ActionType::FileWrite,
            preview,
            confidence,
            None,
        )
        .await;

        // Medium confidence should queue for approval
        assert!(result.is_some());

        // Verify it's in the pending queue
        let pending = state.pending_actions.read().await;
        assert!(pending.contains_key(&result.expect("action_id")));
    }

    #[tokio::test]
    async fn test_is_action_approved() {
        let state = make_test_state();

        // Non-existent action is considered "approved" (not pending)
        assert!(is_action_approved(&state, "nonexistent").await);

        // Add a pending action
        let preview = ActionPreview::new("Test", ActionType::FileWrite);
        let confidence = ConfidenceScore::medium("Test");
        let action_id = queue_pending_action(
            &state,
            "project-1",
            ActionType::FileWrite,
            preview,
            confidence,
            None,
        )
        .await
        .expect("should queue");

        // Should not be approved yet
        assert!(!is_action_approved(&state, &action_id).await);

        // Remove it (simulating approval)
        {
            let mut pending = state.pending_actions.write().await;
            pending.remove(&action_id);
        }

        // Now should be approved
        assert!(is_action_approved(&state, &action_id).await);
    }
}
