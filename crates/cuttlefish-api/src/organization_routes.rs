//! Organization API route handlers.
//!
//! Provides endpoints for:
//! - Organization CRUD operations
//! - Organization membership management
//! - Organization configuration
//! - Organization API key pool management

use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    middleware::from_fn_with_state,
    response::Json,
    routing::{delete, get, post, put},
    Extension, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use uuid::Uuid;

use cuttlefish_db::{
    org_api_keys::{
        add_org_api_key, delete_org_api_key, get_org_api_keys, OrgApiKeyError, OrgApiKeySummary,
    },
    org_config::{get_org_config, update_org_config, OrgConfig},
    organization::{
        add_member, can_user_access_org, count_members, create_organization, delete_organization,
        get_member, get_org_members, get_organization, get_user_orgs, remove_member,
        update_member_role, update_org_name, OrgError, OrgMember, OrgRole, OrgSummary,
    },
};

use crate::middleware::{AuthConfig, AuthenticatedUser};

/// State for organization routes.
#[derive(Clone)]
pub struct OrganizationState {
    /// Database connection pool.
    pub db: Arc<SqlitePool>,
    /// Auth configuration.
    pub config: AuthConfig,
}

impl OrganizationState {
    /// Create a new organization state.
    pub fn new(db: SqlitePool, config: AuthConfig) -> Self {
        Self {
            db: Arc::new(db),
            config,
        }
    }
}

/// Request to create an organization.
#[derive(Debug, Deserialize)]
pub struct CreateOrgRequest {
    /// Organization name.
    pub name: String,
}

/// Response after creating an organization.
#[derive(Debug, Serialize)]
pub struct CreateOrgResponse {
    /// Organization ID.
    pub id: String,
    /// Organization name.
    pub name: String,
    /// URL-friendly slug.
    pub slug: String,
    /// Message.
    pub message: String,
}

/// Request to update an organization.
#[derive(Debug, Deserialize)]
pub struct UpdateOrgRequest {
    /// New organization name.
    pub name: Option<String>,
}

/// Organization details response.
#[derive(Debug, Serialize)]
pub struct OrgDetailsResponse {
    /// Organization ID.
    pub id: String,
    /// Organization name.
    pub name: String,
    /// URL-friendly slug.
    pub slug: String,
    /// Owner user ID.
    pub owner_id: String,
    /// When created.
    pub created_at: String,
    /// Number of members.
    pub member_count: i64,
    /// User's role in the organization.
    pub user_role: String,
}

/// Request to add a member.
#[derive(Debug, Deserialize)]
pub struct AddMemberRequest {
    /// User ID to add.
    pub user_id: String,
    /// Role to grant.
    pub role: String,
}

/// Response after adding a member.
#[derive(Debug, Serialize)]
pub struct AddMemberResponse {
    /// Membership ID.
    pub id: String,
    /// User ID.
    pub user_id: String,
    /// Role.
    pub role: String,
    /// Message.
    pub message: String,
}

/// Organization member info.
#[derive(Debug, Serialize)]
pub struct OrgMemberInfo {
    /// Membership ID.
    pub id: String,
    /// User ID.
    pub user_id: String,
    /// Role.
    pub role: String,
    /// When joined.
    pub joined_at: String,
}

impl From<OrgMember> for OrgMemberInfo {
    fn from(member: OrgMember) -> Self {
        Self {
            id: member.id,
            user_id: member.user_id,
            role: member.role,
            joined_at: member.joined_at,
        }
    }
}

/// Request to update a member's role.
#[derive(Debug, Deserialize)]
pub struct UpdateOrgRoleRequest {
    /// New role.
    pub role: String,
}

/// Request to add an API key.
#[derive(Debug, Deserialize)]
pub struct AddApiKeyRequest {
    /// Provider name (anthropic, openai, etc.).
    pub provider: String,
    /// Human-readable name.
    pub name: String,
    /// The API key value.
    pub api_key: String,
    /// Optional monthly usage limit.
    pub usage_limit_monthly: Option<f64>,
}

/// Response after adding an API key.
#[derive(Debug, Serialize)]
pub struct AddApiKeyResponse {
    /// Key ID.
    pub id: String,
    /// Provider.
    pub provider: String,
    /// Name.
    pub name: String,
    /// Masked key.
    pub key_masked: String,
    /// Message.
    pub message: String,
}

type ApiError = (StatusCode, Json<serde_json::Value>);

fn error_response(status: StatusCode, message: &str) -> ApiError {
    (status, Json(serde_json::json!({ "error": message })))
}

fn org_error_to_api(e: OrgError) -> ApiError {
    match e {
        OrgError::NotFound => error_response(StatusCode::NOT_FOUND, "Organization not found"),
        OrgError::SlugExists => error_response(StatusCode::CONFLICT, "Organization name already taken"),
        OrgError::AlreadyMember => error_response(StatusCode::CONFLICT, "User is already a member"),
        OrgError::NotMember => error_response(StatusCode::NOT_FOUND, "User is not a member"),
        OrgError::LastOwner => error_response(StatusCode::BAD_REQUEST, "Cannot remove the last owner"),
        OrgError::InsufficientPermissions => {
            error_response(StatusCode::FORBIDDEN, "Insufficient permissions")
        }
        OrgError::CannotModifySelf => {
            error_response(StatusCode::BAD_REQUEST, "Cannot modify your own role")
        }
        OrgError::Database(e) => {
            tracing::error!("Database error: {}", e);
            error_response(StatusCode::INTERNAL_SERVER_ERROR, "Database error")
        }
    }
}

fn api_key_error_to_api(e: OrgApiKeyError) -> ApiError {
    match e {
        OrgApiKeyError::OrgNotFound => {
            error_response(StatusCode::NOT_FOUND, "Organization not found")
        }
        OrgApiKeyError::KeyNotFound => error_response(StatusCode::NOT_FOUND, "API key not found"),
        OrgApiKeyError::InsufficientPermissions => {
            error_response(StatusCode::FORBIDDEN, "Insufficient permissions")
        }
        OrgApiKeyError::ProviderKeyExists => {
            error_response(StatusCode::CONFLICT, "API key for this provider already exists")
        }
        OrgApiKeyError::EncryptionError(e) => {
            tracing::error!("Encryption error: {}", e);
            error_response(StatusCode::INTERNAL_SERVER_ERROR, "Encryption error")
        }
        OrgApiKeyError::Database(e) => {
            tracing::error!("Database error: {}", e);
            error_response(StatusCode::INTERNAL_SERVER_ERROR, "Database error")
        }
    }
}

/// Create a new organization.
///
/// POST /api/organizations
pub async fn create_org_handler(
    State(state): State<OrganizationState>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(req): Json<CreateOrgRequest>,
) -> Result<(StatusCode, Json<CreateOrgResponse>), ApiError> {
    let name = req.name.trim();
    if name.is_empty() {
        return Err(error_response(
            StatusCode::BAD_REQUEST,
            "Organization name cannot be empty",
        ));
    }

    let org_id = Uuid::new_v4().to_string();

    let org = create_organization(&state.db, &org_id, name, &user.user_id)
        .await
        .map_err(org_error_to_api)?;

    tracing::info!(
        org_id = %org.id,
        org_name = %org.name,
        owner_id = %user.user_id,
        "Organization created"
    );

    Ok((
        StatusCode::CREATED,
        Json(CreateOrgResponse {
            id: org.id,
            name: org.name,
            slug: org.slug,
            message: "Organization created successfully".to_string(),
        }),
    ))
}

/// List user's organizations.
///
/// GET /api/organizations
pub async fn list_orgs_handler(
    State(state): State<OrganizationState>,
    Extension(user): Extension<AuthenticatedUser>,
) -> Result<Json<Vec<OrgSummary>>, ApiError> {
    let orgs = get_user_orgs(&state.db, &user.user_id)
        .await
        .map_err(|e| {
            tracing::error!("Database error: {}", e);
            error_response(StatusCode::INTERNAL_SERVER_ERROR, "Database error")
        })?;

    Ok(Json(orgs))
}

/// Get organization details.
///
/// GET /api/organizations/{id}
pub async fn get_org_handler(
    State(state): State<OrganizationState>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(org_id): Path<String>,
) -> Result<Json<OrgDetailsResponse>, ApiError> {
    let org = get_organization(&state.db, &org_id)
        .await
        .map_err(|e| {
            tracing::error!("Database error: {}", e);
            error_response(StatusCode::INTERNAL_SERVER_ERROR, "Database error")
        })?
        .ok_or_else(|| error_response(StatusCode::NOT_FOUND, "Organization not found"))?;

    let member = get_member(&state.db, &org_id, &user.user_id)
        .await
        .map_err(|e| {
            tracing::error!("Database error: {}", e);
            error_response(StatusCode::INTERNAL_SERVER_ERROR, "Database error")
        })?
        .ok_or_else(|| {
            error_response(StatusCode::FORBIDDEN, "You are not a member of this organization")
        })?;

    let member_count = count_members(&state.db, &org_id).await.map_err(|e| {
        tracing::error!("Database error: {}", e);
        error_response(StatusCode::INTERNAL_SERVER_ERROR, "Database error")
    })?;

    Ok(Json(OrgDetailsResponse {
        id: org.id,
        name: org.name,
        slug: org.slug,
        owner_id: org.owner_id,
        created_at: org.created_at,
        member_count,
        user_role: member.role,
    }))
}

/// Update an organization.
///
/// PUT /api/organizations/{id}
pub async fn update_org_handler(
    State(state): State<OrganizationState>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(org_id): Path<String>,
    Json(req): Json<UpdateOrgRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let can_update = can_user_access_org(&state.db, &user.user_id, &org_id, OrgRole::Admin)
        .await
        .map_err(|e| {
            tracing::error!("Database error: {}", e);
            error_response(StatusCode::INTERNAL_SERVER_ERROR, "Database error")
        })?;

    if !can_update {
        return Err(error_response(
            StatusCode::FORBIDDEN,
            "You don't have permission to update this organization",
        ));
    }

    if let Some(name) = req.name {
        let name = name.trim();
        if name.is_empty() {
            return Err(error_response(
                StatusCode::BAD_REQUEST,
                "Organization name cannot be empty",
            ));
        }

        update_org_name(&state.db, &org_id, name)
            .await
            .map_err(org_error_to_api)?;

        tracing::info!(org_id = %org_id, new_name = %name, "Organization name updated");
    }

    Ok(Json(serde_json::json!({ "message": "Organization updated successfully" })))
}

/// Delete an organization.
///
/// DELETE /api/organizations/{id}
pub async fn delete_org_handler(
    State(state): State<OrganizationState>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(org_id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    delete_organization(&state.db, &org_id, &user.user_id)
        .await
        .map_err(org_error_to_api)?;

    tracing::info!(org_id = %org_id, "Organization deleted");

    Ok(Json(serde_json::json!({ "message": "Organization deleted successfully" })))
}

/// Add a member to an organization.
///
/// POST /api/organizations/{id}/members
pub async fn add_member_handler(
    State(state): State<OrganizationState>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(org_id): Path<String>,
    Json(req): Json<AddMemberRequest>,
) -> Result<(StatusCode, Json<AddMemberResponse>), ApiError> {
    let role = OrgRole::parse(&req.role);
    let member_id = Uuid::new_v4().to_string();

    let member = add_member(
        &state.db,
        &member_id,
        &org_id,
        &req.user_id,
        role,
        &user.user_id,
    )
    .await
    .map_err(org_error_to_api)?;

    tracing::info!(
        org_id = %org_id,
        user_id = %req.user_id,
        role = %role.as_str(),
        "Member added to organization"
    );

    Ok((
        StatusCode::CREATED,
        Json(AddMemberResponse {
            id: member.id,
            user_id: member.user_id,
            role: member.role,
            message: "Member added successfully".to_string(),
        }),
    ))
}

/// List organization members.
///
/// GET /api/organizations/{id}/members
pub async fn list_members_handler(
    State(state): State<OrganizationState>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(org_id): Path<String>,
) -> Result<Json<Vec<OrgMemberInfo>>, ApiError> {
    let can_view = can_user_access_org(&state.db, &user.user_id, &org_id, OrgRole::Member)
        .await
        .map_err(|e| {
            tracing::error!("Database error: {}", e);
            error_response(StatusCode::INTERNAL_SERVER_ERROR, "Database error")
        })?;

    if !can_view {
        return Err(error_response(
            StatusCode::FORBIDDEN,
            "You are not a member of this organization",
        ));
    }

    let members = get_org_members(&state.db, &org_id).await.map_err(|e| {
        tracing::error!("Database error: {}", e);
        error_response(StatusCode::INTERNAL_SERVER_ERROR, "Database error")
    })?;

    Ok(Json(members.into_iter().map(OrgMemberInfo::from).collect()))
}

/// Remove a member from an organization.
///
/// DELETE /api/organizations/{id}/members/{user_id}
pub async fn remove_member_handler(
    State(state): State<OrganizationState>,
    Extension(user): Extension<AuthenticatedUser>,
    Path((org_id, target_user_id)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, ApiError> {
    remove_member(&state.db, &org_id, &target_user_id, &user.user_id)
        .await
        .map_err(org_error_to_api)?;

    tracing::info!(
        org_id = %org_id,
        removed_user_id = %target_user_id,
        "Member removed from organization"
    );

    Ok(Json(serde_json::json!({ "message": "Member removed successfully" })))
}

/// Update a member's role.
///
/// PUT /api/organizations/{id}/members/{user_id}/role
pub async fn update_role_handler(
    State(state): State<OrganizationState>,
    Extension(user): Extension<AuthenticatedUser>,
    Path((org_id, target_user_id)): Path<(String, String)>,
    Json(req): Json<UpdateOrgRoleRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let new_role = OrgRole::parse(&req.role);

    update_member_role(&state.db, &org_id, &target_user_id, new_role, &user.user_id)
        .await
        .map_err(org_error_to_api)?;

    tracing::info!(
        org_id = %org_id,
        target_user_id = %target_user_id,
        new_role = %new_role.as_str(),
        "Member role updated"
    );

    Ok(Json(serde_json::json!({
        "message": "Role updated successfully",
        "new_role": new_role.as_str()
    })))
}

/// Get organization configuration.
///
/// GET /api/organizations/{id}/config
pub async fn get_config_handler(
    State(state): State<OrganizationState>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(org_id): Path<String>,
) -> Result<Json<OrgConfig>, ApiError> {
    let can_view = can_user_access_org(&state.db, &user.user_id, &org_id, OrgRole::Member)
        .await
        .map_err(|e| {
            tracing::error!("Database error: {}", e);
            error_response(StatusCode::INTERNAL_SERVER_ERROR, "Database error")
        })?;

    if !can_view {
        return Err(error_response(
            StatusCode::FORBIDDEN,
            "You are not a member of this organization",
        ));
    }

    let config = get_org_config(&state.db, &org_id)
        .await
        .map_err(org_error_to_api)?;

    Ok(Json(config))
}

/// Update organization configuration.
///
/// PUT /api/organizations/{id}/config
pub async fn update_config_handler(
    State(state): State<OrganizationState>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(org_id): Path<String>,
    Json(config): Json<OrgConfig>,
) -> Result<Json<serde_json::Value>, ApiError> {
    update_org_config(&state.db, &org_id, &config, &user.user_id)
        .await
        .map_err(org_error_to_api)?;

    tracing::info!(org_id = %org_id, "Organization config updated");

    Ok(Json(serde_json::json!({ "message": "Configuration updated successfully" })))
}

/// Add an API key to the organization pool.
///
/// POST /api/organizations/{id}/api-keys
pub async fn add_api_key_handler(
    State(state): State<OrganizationState>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(org_id): Path<String>,
    Json(req): Json<AddApiKeyRequest>,
) -> Result<(StatusCode, Json<AddApiKeyResponse>), ApiError> {
    let key_id = Uuid::new_v4().to_string();

    let summary = add_org_api_key(
        &state.db,
        &key_id,
        &org_id,
        &req.provider,
        &req.name,
        &req.api_key,
        &user.user_id,
        req.usage_limit_monthly,
    )
    .await
    .map_err(api_key_error_to_api)?;

    tracing::info!(
        org_id = %org_id,
        provider = %req.provider,
        key_id = %key_id,
        "API key added to organization"
    );

    Ok((
        StatusCode::CREATED,
        Json(AddApiKeyResponse {
            id: summary.id,
            provider: summary.provider,
            name: summary.name,
            key_masked: summary.key_masked,
            message: "API key added successfully".to_string(),
        }),
    ))
}

/// List organization API keys (masked).
///
/// GET /api/organizations/{id}/api-keys
pub async fn list_api_keys_handler(
    State(state): State<OrganizationState>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(org_id): Path<String>,
) -> Result<Json<Vec<OrgApiKeySummary>>, ApiError> {
    let keys = get_org_api_keys(&state.db, &org_id, &user.user_id)
        .await
        .map_err(api_key_error_to_api)?;

    Ok(Json(keys))
}

/// Delete an organization API key.
///
/// DELETE /api/organizations/{id}/api-keys/{key_id}
pub async fn delete_api_key_handler(
    State(state): State<OrganizationState>,
    Extension(user): Extension<AuthenticatedUser>,
    Path((org_id, key_id)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let deleted = delete_org_api_key(&state.db, &org_id, &key_id, &user.user_id)
        .await
        .map_err(api_key_error_to_api)?;

    if !deleted {
        return Err(error_response(StatusCode::NOT_FOUND, "API key not found"));
    }

    tracing::info!(
        org_id = %org_id,
        key_id = %key_id,
        "API key deleted from organization"
    );

    Ok(Json(serde_json::json!({ "message": "API key deleted successfully" })))
}

/// Build the organization routes router.
pub fn organization_router(state: OrganizationState) -> Router {
    let auth_config = state.config.clone();

    Router::new()
        .route("/organizations", post(create_org_handler).get(list_orgs_handler))
        .route(
            "/organizations/{id}",
            get(get_org_handler)
                .put(update_org_handler)
                .delete(delete_org_handler),
        )
        .route(
            "/organizations/{id}/members",
            post(add_member_handler).get(list_members_handler),
        )
        .route(
            "/organizations/{id}/members/{user_id}",
            delete(remove_member_handler),
        )
        .route(
            "/organizations/{id}/members/{user_id}/role",
            put(update_role_handler),
        )
        .route(
            "/organizations/{id}/config",
            get(get_config_handler).put(update_config_handler),
        )
        .route(
            "/organizations/{id}/api-keys",
            post(add_api_key_handler).get(list_api_keys_handler),
        )
        .route(
            "/organizations/{id}/api-keys/{key_id}",
            delete(delete_api_key_handler),
        )
        .layer(from_fn_with_state(auth_config, crate::middleware::require_auth))
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_org_request_deserialize() {
        let json = r#"{"name": "Acme Corp"}"#;
        let req: CreateOrgRequest = serde_json::from_str(json).expect("parse");
        assert_eq!(req.name, "Acme Corp");
    }

    #[test]
    fn test_update_org_request_deserialize() {
        let json = r#"{"name": "New Name"}"#;
        let req: UpdateOrgRequest = serde_json::from_str(json).expect("parse");
        assert_eq!(req.name, Some("New Name".to_string()));
    }

    #[test]
    fn test_update_org_request_empty() {
        let json = r#"{}"#;
        let req: UpdateOrgRequest = serde_json::from_str(json).expect("parse");
        assert!(req.name.is_none());
    }

    #[test]
    fn test_add_member_request_deserialize() {
        let json = r#"{"user_id": "user-123", "role": "admin"}"#;
        let req: AddMemberRequest = serde_json::from_str(json).expect("parse");
        assert_eq!(req.user_id, "user-123");
        assert_eq!(req.role, "admin");
    }

    #[test]
    fn test_add_api_key_request_deserialize() {
        let json = r#"{
            "provider": "anthropic",
            "name": "Production Key",
            "api_key": "sk-ant-api03-xxx",
            "usage_limit_monthly": 100.0
        }"#;
        let req: AddApiKeyRequest = serde_json::from_str(json).expect("parse");
        assert_eq!(req.provider, "anthropic");
        assert_eq!(req.name, "Production Key");
        assert_eq!(req.api_key, "sk-ant-api03-xxx");
        assert_eq!(req.usage_limit_monthly, Some(100.0));
    }

    #[test]
    fn test_add_api_key_request_no_limit() {
        let json = r#"{
            "provider": "openai",
            "name": "Dev Key",
            "api_key": "sk-xxx"
        }"#;
        let req: AddApiKeyRequest = serde_json::from_str(json).expect("parse");
        assert!(req.usage_limit_monthly.is_none());
    }

    #[test]
    fn test_org_member_info_serialize() {
        let info = OrgMemberInfo {
            id: "member-1".to_string(),
            user_id: "user-1".to_string(),
            role: "admin".to_string(),
            joined_at: "2024-01-01T00:00:00Z".to_string(),
        };
        let json = serde_json::to_string(&info).expect("serialize");
        assert!(json.contains("member-1"));
        assert!(json.contains("admin"));
    }

    #[test]
    fn test_org_details_response_serialize() {
        let resp = OrgDetailsResponse {
            id: "org-1".to_string(),
            name: "Test Org".to_string(),
            slug: "test-org".to_string(),
            owner_id: "user-1".to_string(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
            member_count: 5,
            user_role: "owner".to_string(),
        };
        let json = serde_json::to_string(&resp).expect("serialize");
        assert!(json.contains("test-org"));
        assert!(json.contains("member_count"));
    }
}
