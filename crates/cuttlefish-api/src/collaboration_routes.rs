//! Collaboration API route handlers.
//!
//! Provides endpoints for:
//! - Project sharing and member management
//! - Project invites
//! - Handoffs for async collaboration
//! - Activity feed

use std::sync::Arc;

use axum::{
    Extension, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    middleware::from_fn_with_state,
    response::Json,
    routing::{delete, get, post, put},
};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use uuid::Uuid;

use cuttlefish_db::{
    activity::{ActivityAction, get_project_activity, log_activity},
    handoffs::{
        ContextSnapshot, CreateHandoffRequest, HandoffError, HandoffPriority, HandoffSummary,
        accept_handoff, create_handoff, get_handoff, get_pending_handoffs, reject_handoff,
    },
    invites::{
        accept_invite, create_invite, get_invite_by_token, get_pending_invites_for_email,
        get_pending_project_invites, invite_exists, mask_email,
    },
    models::{ActivityEntry, ProjectInvite, ProjectRole, ProjectShare},
    sharing::{
        can_user_access, get_project_shares, get_share, get_user_role, is_only_owner, remove_share,
        share_project, update_share_role,
    },
};

use crate::middleware::{AuthConfig, AuthenticatedUser};

/// State for collaboration routes.
#[derive(Clone)]
pub struct CollaborationState {
    /// Database connection pool.
    pub db: Arc<SqlitePool>,
    /// Auth configuration.
    pub config: AuthConfig,
}

impl CollaborationState {
    /// Create a new collaboration state.
    pub fn new(db: SqlitePool, config: AuthConfig) -> Self {
        Self {
            db: Arc::new(db),
            config,
        }
    }
}

// ============================================================================
// Request/Response Types
// ============================================================================

/// Request to share a project with a user.
#[derive(Debug, Deserialize)]
pub struct ShareProjectRequest {
    /// User ID to share with.
    pub user_id: String,
    /// Role to grant.
    pub role: String,
}

/// Response after sharing a project.
#[derive(Debug, Serialize)]
pub struct ShareProjectResponse {
    /// Share ID.
    pub id: String,
    /// Project ID.
    pub project_id: String,
    /// User ID.
    pub user_id: String,
    /// Role granted.
    pub role: String,
    /// Message.
    pub message: String,
}

/// Request to update a member's role.
#[derive(Debug, Deserialize)]
pub struct UpdateRoleRequest {
    /// New role.
    pub role: String,
}

/// Project member info.
#[derive(Debug, Serialize)]
pub struct ProjectMemberInfo {
    /// User ID.
    pub user_id: String,
    /// Role.
    pub role: String,
    /// Who shared with this user.
    pub shared_by: String,
    /// When shared.
    pub created_at: String,
}

impl From<ProjectShare> for ProjectMemberInfo {
    fn from(share: ProjectShare) -> Self {
        Self {
            user_id: share.user_id,
            role: share.role,
            shared_by: share.shared_by,
            created_at: share.created_at,
        }
    }
}

/// Request to create an invite.
#[derive(Debug, Deserialize)]
pub struct CreateInviteRequest {
    /// Email to invite.
    pub email: String,
    /// Role to grant.
    pub role: String,
}

/// Response after creating an invite.
#[derive(Debug, Serialize)]
pub struct CreateInviteResponse {
    /// Invite ID.
    pub id: String,
    /// Invite token (for sharing).
    pub token: String,
    /// Masked email.
    pub email_masked: String,
    /// Role.
    pub role: String,
    /// Expiration timestamp.
    pub expires_at: String,
    /// Message.
    pub message: String,
}

/// Pending invite info.
#[derive(Debug, Serialize)]
pub struct PendingInviteInfo {
    /// Invite ID.
    pub id: String,
    /// Project ID.
    pub project_id: String,
    /// Masked email.
    pub email_masked: String,
    /// Role.
    pub role: String,
    /// Invited by user ID.
    pub invited_by: String,
    /// When created.
    pub created_at: String,
    /// When expires.
    pub expires_at: String,
}

impl From<ProjectInvite> for PendingInviteInfo {
    fn from(invite: ProjectInvite) -> Self {
        Self {
            id: invite.id,
            project_id: invite.project_id,
            email_masked: mask_email(&invite.email),
            role: invite.role,
            invited_by: invite.invited_by,
            created_at: invite.created_at,
            expires_at: invite.expires_at,
        }
    }
}

/// Request to create a handoff.
#[derive(Debug, Deserialize)]
pub struct CreateHandoffApiRequest {
    /// Project ID.
    pub project_id: String,
    /// User ID to assign (optional, None = open to anyone).
    pub to_user_id: Option<String>,
    /// Handoff title.
    pub title: String,
    /// Optional message.
    pub message: Option<String>,
    /// Context snapshot.
    pub context: ContextSnapshotRequest,
    /// Priority level.
    #[serde(default = "default_priority")]
    pub priority: String,
}

fn default_priority() -> String {
    "normal".to_string()
}

/// Context snapshot for handoff request.
#[derive(Debug, Deserialize)]
pub struct ContextSnapshotRequest {
    /// Summary of the conversation.
    pub conversation_summary: String,
    /// Current branch.
    pub current_branch: Option<String>,
    /// Open files.
    #[serde(default)]
    pub open_files: Vec<String>,
    /// Open questions.
    #[serde(default)]
    pub open_questions: Vec<String>,
    /// Suggested next steps.
    #[serde(default)]
    pub suggested_next_steps: Vec<String>,
}

impl From<ContextSnapshotRequest> for ContextSnapshot {
    fn from(req: ContextSnapshotRequest) -> Self {
        let mut ctx = ContextSnapshot::new(req.conversation_summary);
        ctx.current_branch = req.current_branch;
        ctx.open_files = req.open_files;
        ctx.open_questions = req.open_questions;
        ctx.suggested_next_steps = req.suggested_next_steps;
        ctx
    }
}

/// Response after creating a handoff.
#[derive(Debug, Serialize)]
pub struct CreateHandoffResponse {
    /// Handoff ID.
    pub id: String,
    /// Project ID.
    pub project_id: String,
    /// Assigned user ID.
    pub to_user_id: Option<String>,
    /// Title.
    pub title: String,
    /// Priority.
    pub priority: String,
    /// Message.
    pub message: String,
}

/// Request to reject a handoff.
#[derive(Debug, Deserialize)]
pub struct RejectHandoffRequest {
    /// Rejection reason.
    pub reason: Option<String>,
}

/// Query parameters for activity feed.
#[derive(Debug, Deserialize)]
pub struct ActivityQuery {
    /// Maximum number of entries.
    pub limit: Option<i64>,
    /// Cursor for pagination (timestamp).
    pub before: Option<String>,
}

/// Activity entry response.
#[derive(Debug, Serialize)]
pub struct ActivityEntryResponse {
    /// Entry ID.
    pub id: String,
    /// Project ID.
    pub project_id: String,
    /// User ID who performed the action.
    pub user_id: String,
    /// Action type (JSON).
    pub action: String,
    /// Additional details.
    pub details: Option<String>,
    /// When the action occurred.
    pub created_at: String,
}

impl From<ActivityEntry> for ActivityEntryResponse {
    fn from(entry: ActivityEntry) -> Self {
        Self {
            id: entry.id,
            project_id: entry.project_id,
            user_id: entry.user_id,
            action: entry.action,
            details: entry.details,
            created_at: entry.created_at,
        }
    }
}

// ============================================================================
// Error Handling
// ============================================================================

type ApiError = (StatusCode, Json<serde_json::Value>);

fn error_response(status: StatusCode, message: &str) -> ApiError {
    (status, Json(serde_json::json!({ "error": message })))
}

fn handoff_error_to_api(e: HandoffError) -> ApiError {
    match e {
        HandoffError::SelfHandoff => {
            error_response(StatusCode::BAD_REQUEST, "Cannot create handoff to yourself")
        }
        HandoffError::NoProjectAccess => {
            error_response(StatusCode::FORBIDDEN, "User does not have project access")
        }
        HandoffError::NotFound => error_response(StatusCode::NOT_FOUND, "Handoff not found"),
        HandoffError::NotPending => {
            error_response(StatusCode::BAD_REQUEST, "Handoff is not pending")
        }
        HandoffError::NotAssignee => {
            error_response(StatusCode::FORBIDDEN, "You are not the assignee")
        }
        HandoffError::Database(e) => {
            tracing::error!("Database error: {}", e);
            error_response(StatusCode::INTERNAL_SERVER_ERROR, "Database error")
        }
    }
}

// ============================================================================
// Sharing Endpoints
// ============================================================================

/// Share a project with a user.
///
/// POST /api/projects/{id}/share
pub async fn share_project_handler(
    State(state): State<CollaborationState>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(project_id): Path<String>,
    Json(req): Json<ShareProjectRequest>,
) -> Result<(StatusCode, Json<ShareProjectResponse>), ApiError> {
    // Verify actor has admin+ access
    let can_share = can_user_access(&state.db, &user.user_id, &project_id, ProjectRole::Admin)
        .await
        .map_err(|e| {
            tracing::error!("Database error: {}", e);
            error_response(StatusCode::INTERNAL_SERVER_ERROR, "Database error")
        })?;

    if !can_share {
        return Err(error_response(
            StatusCode::FORBIDDEN,
            "You don't have permission to share this project",
        ));
    }

    // Check if user already has access
    let existing = get_share(&state.db, &project_id, &req.user_id)
        .await
        .map_err(|e| {
            tracing::error!("Database error: {}", e);
            error_response(StatusCode::INTERNAL_SERVER_ERROR, "Database error")
        })?;

    if existing.is_some() {
        return Err(error_response(
            StatusCode::CONFLICT,
            "User already has access to this project",
        ));
    }

    let role = ProjectRole::parse(&req.role);
    let share_id = Uuid::new_v4().to_string();

    let share = share_project(
        &state.db,
        &share_id,
        &project_id,
        &req.user_id,
        role,
        &user.user_id,
    )
    .await
    .map_err(|e| {
        tracing::error!("Failed to share project: {}", e);
        error_response(StatusCode::INTERNAL_SERVER_ERROR, "Failed to share project")
    })?;

    // Log activity
    let activity_id = Uuid::new_v4().to_string();
    let _ = log_activity(
        &state.db,
        &activity_id,
        &project_id,
        &user.user_id,
        &ActivityAction::MemberAdded {
            member_id: req.user_id.clone(),
            role: role.as_str().to_string(),
        },
        None,
    )
    .await;

    tracing::info!(
        project_id = %project_id,
        user_id = %req.user_id,
        role = %role.as_str(),
        "Project shared"
    );

    Ok((
        StatusCode::CREATED,
        Json(ShareProjectResponse {
            id: share.id,
            project_id: share.project_id,
            user_id: share.user_id,
            role: share.role,
            message: "Project shared successfully".to_string(),
        }),
    ))
}

/// List project members.
///
/// GET /api/projects/{id}/members
pub async fn list_members_handler(
    State(state): State<CollaborationState>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(project_id): Path<String>,
) -> Result<Json<Vec<ProjectMemberInfo>>, ApiError> {
    // Verify actor has viewer+ access
    let can_view = can_user_access(&state.db, &user.user_id, &project_id, ProjectRole::Viewer)
        .await
        .map_err(|e| {
            tracing::error!("Database error: {}", e);
            error_response(StatusCode::INTERNAL_SERVER_ERROR, "Database error")
        })?;

    if !can_view {
        return Err(error_response(
            StatusCode::FORBIDDEN,
            "You don't have access to this project",
        ));
    }

    let shares = get_project_shares(&state.db, &project_id)
        .await
        .map_err(|e| {
            tracing::error!("Database error: {}", e);
            error_response(StatusCode::INTERNAL_SERVER_ERROR, "Database error")
        })?;

    Ok(Json(
        shares.into_iter().map(ProjectMemberInfo::from).collect(),
    ))
}

/// Remove a member from a project.
///
/// DELETE /api/projects/{id}/members/{user_id}
pub async fn remove_member_handler(
    State(state): State<CollaborationState>,
    Extension(user): Extension<AuthenticatedUser>,
    Path((project_id, target_user_id)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // Get actor's role
    let actor_role = get_user_role(&state.db, &user.user_id, &project_id)
        .await
        .map_err(|e| {
            tracing::error!("Database error: {}", e);
            error_response(StatusCode::INTERNAL_SERVER_ERROR, "Database error")
        })?
        .ok_or_else(|| {
            error_response(
                StatusCode::FORBIDDEN,
                "You don't have access to this project",
            )
        })?;

    // Get target's role
    let target_role = get_user_role(&state.db, &target_user_id, &project_id)
        .await
        .map_err(|e| {
            tracing::error!("Database error: {}", e);
            error_response(StatusCode::INTERNAL_SERVER_ERROR, "Database error")
        })?
        .ok_or_else(|| error_response(StatusCode::NOT_FOUND, "Member not found"))?;

    // Users can remove themselves, otherwise need admin+
    if target_user_id != user.user_id {
        if !actor_role.has_at_least(ProjectRole::Admin) {
            return Err(error_response(
                StatusCode::FORBIDDEN,
                "You don't have permission to remove members",
            ));
        }

        // Can't remove someone with equal or higher role
        if target_role.level() >= actor_role.level() {
            return Err(error_response(
                StatusCode::FORBIDDEN,
                "Cannot remove a member with equal or higher role",
            ));
        }
    }

    // Check if removing the last owner
    if target_role == ProjectRole::Owner {
        let is_only = is_only_owner(&state.db, &project_id, &target_user_id)
            .await
            .map_err(|e| {
                tracing::error!("Database error: {}", e);
                error_response(StatusCode::INTERNAL_SERVER_ERROR, "Database error")
            })?;

        if is_only {
            return Err(error_response(
                StatusCode::BAD_REQUEST,
                "Cannot remove the last owner",
            ));
        }
    }

    let removed = remove_share(&state.db, &project_id, &target_user_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to remove member: {}", e);
            error_response(StatusCode::INTERNAL_SERVER_ERROR, "Failed to remove member")
        })?;

    if !removed {
        return Err(error_response(StatusCode::NOT_FOUND, "Member not found"));
    }

    // Log activity
    let activity_id = Uuid::new_v4().to_string();
    let _ = log_activity(
        &state.db,
        &activity_id,
        &project_id,
        &user.user_id,
        &ActivityAction::MemberRemoved {
            member_id: target_user_id.clone(),
        },
        None,
    )
    .await;

    tracing::info!(
        project_id = %project_id,
        removed_user_id = %target_user_id,
        "Member removed"
    );

    Ok(Json(
        serde_json::json!({ "message": "Member removed successfully" }),
    ))
}

/// Update a member's role.
///
/// PUT /api/projects/{id}/members/{user_id}/role
pub async fn update_role_handler(
    State(state): State<CollaborationState>,
    Extension(user): Extension<AuthenticatedUser>,
    Path((project_id, target_user_id)): Path<(String, String)>,
    Json(req): Json<UpdateRoleRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // Can't modify own role
    if target_user_id == user.user_id {
        return Err(error_response(
            StatusCode::BAD_REQUEST,
            "Cannot modify your own role",
        ));
    }

    // Get actor's role
    let actor_role = get_user_role(&state.db, &user.user_id, &project_id)
        .await
        .map_err(|e| {
            tracing::error!("Database error: {}", e);
            error_response(StatusCode::INTERNAL_SERVER_ERROR, "Database error")
        })?
        .ok_or_else(|| {
            error_response(
                StatusCode::FORBIDDEN,
                "You don't have access to this project",
            )
        })?;

    // Get target's current role
    let target_role = get_user_role(&state.db, &target_user_id, &project_id)
        .await
        .map_err(|e| {
            tracing::error!("Database error: {}", e);
            error_response(StatusCode::INTERNAL_SERVER_ERROR, "Database error")
        })?
        .ok_or_else(|| error_response(StatusCode::NOT_FOUND, "Member not found"))?;

    let new_role = ProjectRole::parse(&req.role);

    // Check permissions using the sharing module's logic
    if !cuttlefish_db::sharing::can_modify_role(actor_role, target_role, new_role) {
        return Err(error_response(
            StatusCode::FORBIDDEN,
            "You don't have permission to make this role change",
        ));
    }

    // Check if demoting the last owner
    if target_role == ProjectRole::Owner && new_role != ProjectRole::Owner {
        let is_only = is_only_owner(&state.db, &project_id, &target_user_id)
            .await
            .map_err(|e| {
                tracing::error!("Database error: {}", e);
                error_response(StatusCode::INTERNAL_SERVER_ERROR, "Database error")
            })?;

        if is_only {
            return Err(error_response(
                StatusCode::BAD_REQUEST,
                "Cannot demote the last owner",
            ));
        }
    }

    let updated = update_share_role(&state.db, &project_id, &target_user_id, new_role)
        .await
        .map_err(|e| {
            tracing::error!("Failed to update role: {}", e);
            error_response(StatusCode::INTERNAL_SERVER_ERROR, "Failed to update role")
        })?;

    if !updated {
        return Err(error_response(StatusCode::NOT_FOUND, "Member not found"));
    }

    // Log activity
    let activity_id = Uuid::new_v4().to_string();
    let _ = log_activity(
        &state.db,
        &activity_id,
        &project_id,
        &user.user_id,
        &ActivityAction::RoleChanged {
            member_id: target_user_id.clone(),
            old_role: target_role.as_str().to_string(),
            new_role: new_role.as_str().to_string(),
        },
        None,
    )
    .await;

    tracing::info!(
        project_id = %project_id,
        target_user_id = %target_user_id,
        old_role = %target_role.as_str(),
        new_role = %new_role.as_str(),
        "Role updated"
    );

    Ok(Json(serde_json::json!({
        "message": "Role updated successfully",
        "new_role": new_role.as_str()
    })))
}

// ============================================================================
// Invite Endpoints
// ============================================================================

/// Create a project invite.
///
/// POST /api/projects/{id}/invites
pub async fn create_invite_handler(
    State(state): State<CollaborationState>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(project_id): Path<String>,
    Json(req): Json<CreateInviteRequest>,
) -> Result<(StatusCode, Json<CreateInviteResponse>), ApiError> {
    // Verify actor has admin+ access
    let can_invite = can_user_access(&state.db, &user.user_id, &project_id, ProjectRole::Admin)
        .await
        .map_err(|e| {
            tracing::error!("Database error: {}", e);
            error_response(StatusCode::INTERNAL_SERVER_ERROR, "Database error")
        })?;

    if !can_invite {
        return Err(error_response(
            StatusCode::FORBIDDEN,
            "You don't have permission to invite users",
        ));
    }

    let email = req.email.trim().to_lowercase();

    // Check if invite already exists
    let exists = invite_exists(&state.db, &project_id, &email)
        .await
        .map_err(|e| {
            tracing::error!("Database error: {}", e);
            error_response(StatusCode::INTERNAL_SERVER_ERROR, "Database error")
        })?;

    if exists {
        return Err(error_response(
            StatusCode::CONFLICT,
            "An invite for this email already exists",
        ));
    }

    let role = ProjectRole::parse(&req.role);
    let invite_id = Uuid::new_v4().to_string();

    let invite = create_invite(
        &state.db,
        &invite_id,
        &project_id,
        &email,
        role,
        &user.user_id,
    )
    .await
    .map_err(|e| {
        tracing::error!("Failed to create invite: {}", e);
        error_response(StatusCode::INTERNAL_SERVER_ERROR, "Failed to create invite")
    })?;

    // Log activity
    let activity_id = Uuid::new_v4().to_string();
    let _ = log_activity(
        &state.db,
        &activity_id,
        &project_id,
        &user.user_id,
        &ActivityAction::InviteSent {
            email_masked: mask_email(&email),
            role: role.as_str().to_string(),
        },
        None,
    )
    .await;

    tracing::info!(
        project_id = %project_id,
        email_masked = %mask_email(&email),
        role = %role.as_str(),
        "Invite created"
    );

    Ok((
        StatusCode::CREATED,
        Json(CreateInviteResponse {
            id: invite.id,
            token: invite.token,
            email_masked: mask_email(&email),
            role: invite.role,
            expires_at: invite.expires_at,
            message: "Invite created successfully".to_string(),
        }),
    ))
}

/// Get pending invites for the current user.
///
/// GET /api/invites/pending
pub async fn get_pending_invites_handler(
    State(state): State<CollaborationState>,
    Extension(user): Extension<AuthenticatedUser>,
) -> Result<Json<Vec<PendingInviteInfo>>, ApiError> {
    // Get user's email from the database
    let db_user = cuttlefish_db::auth::get_user_by_id(&state.db, &user.user_id)
        .await
        .map_err(|e| {
            tracing::error!("Database error: {}", e);
            error_response(StatusCode::INTERNAL_SERVER_ERROR, "Database error")
        })?
        .ok_or_else(|| error_response(StatusCode::NOT_FOUND, "User not found"))?;

    let invites = get_pending_invites_for_email(&state.db, &db_user.email)
        .await
        .map_err(|e| {
            tracing::error!("Database error: {}", e);
            error_response(StatusCode::INTERNAL_SERVER_ERROR, "Database error")
        })?;

    Ok(Json(
        invites.into_iter().map(PendingInviteInfo::from).collect(),
    ))
}

/// Get pending invites for a project.
///
/// GET /api/projects/{id}/invites
pub async fn get_project_invites_handler(
    State(state): State<CollaborationState>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(project_id): Path<String>,
) -> Result<Json<Vec<PendingInviteInfo>>, ApiError> {
    // Verify actor has admin+ access
    let can_view = can_user_access(&state.db, &user.user_id, &project_id, ProjectRole::Admin)
        .await
        .map_err(|e| {
            tracing::error!("Database error: {}", e);
            error_response(StatusCode::INTERNAL_SERVER_ERROR, "Database error")
        })?;

    if !can_view {
        return Err(error_response(
            StatusCode::FORBIDDEN,
            "You don't have permission to view invites",
        ));
    }

    let invites = get_pending_project_invites(&state.db, &project_id)
        .await
        .map_err(|e| {
            tracing::error!("Database error: {}", e);
            error_response(StatusCode::INTERNAL_SERVER_ERROR, "Database error")
        })?;

    Ok(Json(
        invites.into_iter().map(PendingInviteInfo::from).collect(),
    ))
}

/// Accept an invite.
///
/// POST /api/invites/{token}/accept
pub async fn accept_invite_handler(
    State(state): State<CollaborationState>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(token): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // Get the invite
    let invite = get_invite_by_token(&state.db, &token)
        .await
        .map_err(|e| {
            tracing::error!("Database error: {}", e);
            error_response(StatusCode::INTERNAL_SERVER_ERROR, "Database error")
        })?
        .ok_or_else(|| error_response(StatusCode::NOT_FOUND, "Invite not found or expired"))?;

    // Verify the invite is for this user's email
    let db_user = cuttlefish_db::auth::get_user_by_id(&state.db, &user.user_id)
        .await
        .map_err(|e| {
            tracing::error!("Database error: {}", e);
            error_response(StatusCode::INTERNAL_SERVER_ERROR, "Database error")
        })?
        .ok_or_else(|| error_response(StatusCode::NOT_FOUND, "User not found"))?;

    if db_user.email.to_lowercase() != invite.email.to_lowercase() {
        return Err(error_response(
            StatusCode::FORBIDDEN,
            "This invite is not for your email address",
        ));
    }

    // Accept the invite
    let accepted = accept_invite(&state.db, &token)
        .await
        .map_err(|e| {
            tracing::error!("Failed to accept invite: {}", e);
            error_response(StatusCode::INTERNAL_SERVER_ERROR, "Failed to accept invite")
        })?
        .ok_or_else(|| {
            error_response(
                StatusCode::BAD_REQUEST,
                "Invite already accepted or expired",
            )
        })?;

    // Create the share
    let role = ProjectRole::parse(&accepted.role);
    let share_id = Uuid::new_v4().to_string();

    share_project(
        &state.db,
        &share_id,
        &accepted.project_id,
        &user.user_id,
        role,
        &accepted.invited_by,
    )
    .await
    .map_err(|e| {
        tracing::error!("Failed to create share: {}", e);
        error_response(StatusCode::INTERNAL_SERVER_ERROR, "Failed to join project")
    })?;

    // Log activity
    let activity_id = Uuid::new_v4().to_string();
    let _ = log_activity(
        &state.db,
        &activity_id,
        &accepted.project_id,
        &user.user_id,
        &ActivityAction::UserJoined {
            user_id: user.user_id.clone(),
        },
        Some("Joined via invite"),
    )
    .await;

    tracing::info!(
        project_id = %accepted.project_id,
        user_id = %user.user_id,
        "Invite accepted"
    );

    Ok(Json(serde_json::json!({
        "message": "Invite accepted successfully",
        "project_id": accepted.project_id,
        "role": accepted.role
    })))
}

// ============================================================================
// Handoff Endpoints
// ============================================================================

/// Create a handoff.
///
/// POST /api/handoffs
pub async fn create_handoff_handler(
    State(state): State<CollaborationState>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(req): Json<CreateHandoffApiRequest>,
) -> Result<(StatusCode, Json<CreateHandoffResponse>), ApiError> {
    let handoff_id = Uuid::new_v4().to_string();
    let context: ContextSnapshot = req.context.into();
    let priority = HandoffPriority::parse(&req.priority);

    let request = CreateHandoffRequest {
        id: &handoff_id,
        project_id: &req.project_id,
        from_user_id: &user.user_id,
        to_user_id: req.to_user_id.as_deref(),
        title: &req.title,
        message: req.message.as_deref(),
        context: &context,
        priority,
    };

    let handoff = create_handoff(&state.db, request)
        .await
        .map_err(handoff_error_to_api)?;

    tracing::info!(
        handoff_id = %handoff.id,
        project_id = %handoff.project_id,
        to_user_id = ?handoff.to_user_id,
        "Handoff created"
    );

    Ok((
        StatusCode::CREATED,
        Json(CreateHandoffResponse {
            id: handoff.id,
            project_id: handoff.project_id,
            to_user_id: handoff.to_user_id,
            title: handoff.title,
            priority: handoff.priority,
            message: "Handoff created successfully".to_string(),
        }),
    ))
}

/// Get pending handoffs for the current user.
///
/// GET /api/handoffs/pending
pub async fn get_pending_handoffs_handler(
    State(state): State<CollaborationState>,
    Extension(user): Extension<AuthenticatedUser>,
) -> Result<Json<Vec<HandoffSummary>>, ApiError> {
    let handoffs = get_pending_handoffs(&state.db, &user.user_id)
        .await
        .map_err(|e| {
            tracing::error!("Database error: {}", e);
            error_response(StatusCode::INTERNAL_SERVER_ERROR, "Database error")
        })?;

    Ok(Json(handoffs))
}

/// Accept a handoff.
///
/// POST /api/handoffs/{id}/accept
pub async fn accept_handoff_handler(
    State(state): State<CollaborationState>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(handoff_id): Path<String>,
) -> Result<Json<ContextSnapshot>, ApiError> {
    let context = accept_handoff(&state.db, &handoff_id, &user.user_id)
        .await
        .map_err(handoff_error_to_api)?;

    tracing::info!(
        handoff_id = %handoff_id,
        user_id = %user.user_id,
        "Handoff accepted"
    );

    Ok(Json(context))
}

/// Reject a handoff.
///
/// POST /api/handoffs/{id}/reject
pub async fn reject_handoff_handler(
    State(state): State<CollaborationState>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(handoff_id): Path<String>,
    Json(req): Json<RejectHandoffRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    reject_handoff(&state.db, &handoff_id, &user.user_id, req.reason.as_deref())
        .await
        .map_err(handoff_error_to_api)?;

    tracing::info!(
        handoff_id = %handoff_id,
        user_id = %user.user_id,
        "Handoff rejected"
    );

    Ok(Json(serde_json::json!({ "message": "Handoff rejected" })))
}

/// Get a specific handoff.
///
/// GET /api/handoffs/{id}
pub async fn get_handoff_handler(
    State(state): State<CollaborationState>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(handoff_id): Path<String>,
) -> Result<Json<HandoffSummary>, ApiError> {
    let handoff = get_handoff(&state.db, &handoff_id)
        .await
        .map_err(|e| {
            tracing::error!("Database error: {}", e);
            error_response(StatusCode::INTERNAL_SERVER_ERROR, "Database error")
        })?
        .ok_or_else(|| error_response(StatusCode::NOT_FOUND, "Handoff not found"))?;

    // Verify user has access to the project
    let can_view = can_user_access(
        &state.db,
        &user.user_id,
        &handoff.project_id,
        ProjectRole::Viewer,
    )
    .await
    .map_err(|e| {
        tracing::error!("Database error: {}", e);
        error_response(StatusCode::INTERNAL_SERVER_ERROR, "Database error")
    })?;

    if !can_view {
        return Err(error_response(
            StatusCode::FORBIDDEN,
            "You don't have access to this handoff",
        ));
    }

    Ok(Json(HandoffSummary::from(handoff)))
}

// ============================================================================
// Activity Endpoints
// ============================================================================

/// Get project activity feed.
///
/// GET /api/projects/{id}/activity
pub async fn get_activity_handler(
    State(state): State<CollaborationState>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(project_id): Path<String>,
    Query(query): Query<ActivityQuery>,
) -> Result<Json<Vec<ActivityEntryResponse>>, ApiError> {
    // Verify actor has viewer+ access
    let can_view = can_user_access(&state.db, &user.user_id, &project_id, ProjectRole::Viewer)
        .await
        .map_err(|e| {
            tracing::error!("Database error: {}", e);
            error_response(StatusCode::INTERNAL_SERVER_ERROR, "Database error")
        })?;

    if !can_view {
        return Err(error_response(
            StatusCode::FORBIDDEN,
            "You don't have access to this project",
        ));
    }

    let activities =
        get_project_activity(&state.db, &project_id, query.limit, query.before.as_deref())
            .await
            .map_err(|e| {
                tracing::error!("Database error: {}", e);
                error_response(StatusCode::INTERNAL_SERVER_ERROR, "Database error")
            })?;

    Ok(Json(
        activities
            .into_iter()
            .map(ActivityEntryResponse::from)
            .collect(),
    ))
}

// ============================================================================
// Router
// ============================================================================

/// Build the collaboration routes router.
pub fn collaboration_router(state: CollaborationState) -> Router {
    let auth_config = state.config.clone();

    Router::new()
        // Sharing endpoints
        .route("/projects/{id}/share", post(share_project_handler))
        .route("/projects/{id}/members", get(list_members_handler))
        .route(
            "/projects/{id}/members/{user_id}",
            delete(remove_member_handler),
        )
        .route(
            "/projects/{id}/members/{user_id}/role",
            put(update_role_handler),
        )
        // Invite endpoints
        .route(
            "/projects/{id}/invites",
            post(create_invite_handler).get(get_project_invites_handler),
        )
        .route("/invites/pending", get(get_pending_invites_handler))
        .route("/invites/{token}/accept", post(accept_invite_handler))
        // Handoff endpoints
        .route("/handoffs", post(create_handoff_handler))
        .route("/handoffs/pending", get(get_pending_handoffs_handler))
        .route("/handoffs/{id}", get(get_handoff_handler))
        .route("/handoffs/{id}/accept", post(accept_handoff_handler))
        .route("/handoffs/{id}/reject", post(reject_handoff_handler))
        // Activity endpoints
        .route("/projects/{id}/activity", get(get_activity_handler))
        .layer(from_fn_with_state(
            auth_config,
            crate::middleware::require_auth,
        ))
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_share_project_request_deserialize() {
        let json = r#"{"user_id": "user-123", "role": "member"}"#;
        let req: ShareProjectRequest = serde_json::from_str(json).expect("parse");
        assert_eq!(req.user_id, "user-123");
        assert_eq!(req.role, "member");
    }

    #[test]
    fn test_create_invite_request_deserialize() {
        let json = r#"{"email": "test@example.com", "role": "viewer"}"#;
        let req: CreateInviteRequest = serde_json::from_str(json).expect("parse");
        assert_eq!(req.email, "test@example.com");
        assert_eq!(req.role, "viewer");
    }

    #[test]
    fn test_create_handoff_request_deserialize() {
        let json = r#"{
            "project_id": "proj-1",
            "to_user_id": "user-2",
            "title": "Continue feature X",
            "message": "Please finish this",
            "context": {
                "conversation_summary": "Working on feature X",
                "current_branch": "feature/x",
                "open_files": ["src/main.rs"],
                "open_questions": ["How to handle edge case?"],
                "suggested_next_steps": ["Add tests"]
            },
            "priority": "high"
        }"#;
        let req: CreateHandoffApiRequest = serde_json::from_str(json).expect("parse");
        assert_eq!(req.project_id, "proj-1");
        assert_eq!(req.to_user_id, Some("user-2".to_string()));
        assert_eq!(req.title, "Continue feature X");
        assert_eq!(req.priority, "high");
        assert_eq!(req.context.conversation_summary, "Working on feature X");
    }

    #[test]
    fn test_create_handoff_request_defaults() {
        let json = r#"{
            "project_id": "proj-1",
            "title": "Task",
            "context": {
                "conversation_summary": "Summary"
            }
        }"#;
        let req: CreateHandoffApiRequest = serde_json::from_str(json).expect("parse");
        assert!(req.to_user_id.is_none());
        assert!(req.message.is_none());
        assert_eq!(req.priority, "normal");
        assert!(req.context.open_files.is_empty());
    }

    #[test]
    fn test_activity_query_deserialize() {
        let json = r#"{"limit": 20, "before": "2024-01-01T00:00:00Z"}"#;
        let query: ActivityQuery = serde_json::from_str(json).expect("parse");
        assert_eq!(query.limit, Some(20));
        assert_eq!(query.before, Some("2024-01-01T00:00:00Z".to_string()));
    }

    #[test]
    fn test_project_member_info_serialize() {
        let info = ProjectMemberInfo {
            user_id: "user-1".to_string(),
            role: "member".to_string(),
            shared_by: "user-owner".to_string(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
        };
        let json = serde_json::to_string(&info).expect("serialize");
        assert!(json.contains("user-1"));
        assert!(json.contains("member"));
    }

    #[test]
    fn test_context_snapshot_conversion() {
        let req = ContextSnapshotRequest {
            conversation_summary: "Summary".to_string(),
            current_branch: Some("main".to_string()),
            open_files: vec!["file.rs".to_string()],
            open_questions: vec!["Q1".to_string()],
            suggested_next_steps: vec!["Step 1".to_string()],
        };
        let ctx: ContextSnapshot = req.into();
        assert_eq!(ctx.conversation_summary, "Summary");
        assert_eq!(ctx.current_branch, Some("main".to_string()));
        assert_eq!(ctx.open_files.len(), 1);
    }
}
