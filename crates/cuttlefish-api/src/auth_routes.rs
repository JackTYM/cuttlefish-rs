//! Authentication API route handlers.
//!
//! Provides endpoints for:
//! - User registration and login
//! - Session management (refresh, logout)
//! - Password management (reset, change)
//! - API key management (CRUD)

use std::sync::Arc;

use axum::{
    Extension, Router,
    extract::{Path, State},
    http::StatusCode,
    middleware::from_fn_with_state,
    response::Json,
    routing::{delete, get, post},
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use uuid::Uuid;

use cuttlefish_core::auth::{
    ApiKeyScope, TokenPair, generate_api_key, generate_reset_token, hash_password,
    hash_reset_token, validate_password_strength, verify_password,
};
use cuttlefish_db::{api_keys, auth, password_reset, sessions};

use crate::middleware::{AuthConfig, AuthenticatedUser};

/// State for authentication routes.
#[derive(Clone)]
pub struct AuthState {
    /// Database connection pool.
    pub db: Arc<SqlitePool>,
    /// Auth configuration (JWT secret, legacy key).
    pub config: AuthConfig,
    /// Whether registration is enabled.
    pub registration_enabled: bool,
}

impl AuthState {
    /// Create a new auth state.
    pub fn new(db: SqlitePool, config: AuthConfig) -> Self {
        Self {
            db: Arc::new(db),
            config,
            registration_enabled: true,
        }
    }

    /// Disable registration (single-user mode).
    pub fn with_registration_disabled(mut self) -> Self {
        self.registration_enabled = false;
        self
    }
}

/// Request to register a new user.
#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    /// User's email address.
    pub email: String,
    /// Password (will be hashed).
    pub password: String,
    /// Optional display name.
    pub display_name: Option<String>,
}

/// Response after successful registration.
#[derive(Debug, Serialize)]
pub struct RegisterResponse {
    /// User ID.
    pub user_id: String,
    /// User's email.
    pub email: String,
    /// Message.
    pub message: String,
}

/// Request to login.
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    /// User's email address.
    pub email: String,
    /// User's password.
    pub password: String,
}

/// Response after successful login.
#[derive(Debug, Serialize)]
pub struct LoginResponse {
    /// Access token.
    pub access_token: String,
    /// Refresh token.
    pub refresh_token: String,
    /// Token type (always "Bearer").
    pub token_type: String,
    /// Expires in seconds.
    pub expires_in: i64,
    /// User info.
    pub user: UserInfo,
}

/// Request to refresh tokens.
#[derive(Debug, Deserialize)]
pub struct RefreshRequest {
    /// The refresh token.
    pub refresh_token: String,
}

/// Response after token refresh.
#[derive(Debug, Serialize)]
pub struct RefreshResponse {
    /// New access token.
    pub access_token: String,
    /// New refresh token.
    pub refresh_token: String,
    /// Token type (always "Bearer").
    pub token_type: String,
    /// Expires in seconds.
    pub expires_in: i64,
}

/// User information response.
#[derive(Debug, Clone, Serialize)]
pub struct UserInfo {
    /// User ID.
    pub id: String,
    /// Email address.
    pub email: String,
    /// Display name.
    pub display_name: Option<String>,
    /// Email verified flag.
    pub email_verified: bool,
    /// Created timestamp.
    pub created_at: String,
}

/// Request to change password.
#[derive(Debug, Deserialize)]
pub struct ChangePasswordRequest {
    /// Current password.
    pub current_password: String,
    /// New password.
    pub new_password: String,
}

/// Request to request password reset.
#[derive(Debug, Deserialize)]
pub struct ResetPasswordRequestBody {
    /// Email address.
    pub email: String,
}

/// Response for password reset request (always same for security).
#[derive(Debug, Serialize)]
pub struct ResetPasswordRequestResponse {
    /// Message (same regardless of whether email exists).
    pub message: String,
}

/// Request to reset password with token.
#[derive(Debug, Deserialize)]
pub struct ResetPasswordBody {
    /// Reset token from email.
    pub token: String,
    /// New password.
    pub new_password: String,
}

/// Request to create an API key.
#[derive(Debug, Deserialize)]
pub struct CreateApiKeyRequest {
    /// Human-readable name.
    pub name: String,
    /// Scopes (read, write, admin).
    pub scopes: Vec<String>,
    /// Optional expiration in days.
    pub expires_in_days: Option<i64>,
}

/// Response after creating an API key.
#[derive(Debug, Serialize)]
pub struct CreateApiKeyResponse {
    /// Key ID.
    pub id: String,
    /// The full API key (SHOWN ONCE).
    pub api_key: String,
    /// Prefix for identification.
    pub prefix: String,
    /// Human-readable name.
    pub name: String,
    /// Scopes.
    pub scopes: Vec<String>,
    /// Created timestamp.
    pub created_at: String,
    /// Expiration timestamp (if any).
    pub expires_at: Option<String>,
}

/// API key summary (without the full key).
#[derive(Debug, Serialize)]
pub struct ApiKeySummary {
    /// Key ID.
    pub id: String,
    /// Prefix for identification.
    pub prefix: String,
    /// Human-readable name.
    pub name: String,
    /// Scopes.
    pub scopes: Vec<String>,
    /// Created timestamp.
    pub created_at: String,
    /// Last used timestamp.
    pub last_used_at: Option<String>,
    /// Expiration timestamp (if any).
    pub expires_at: Option<String>,
}

/// Request to update user profile.
#[derive(Debug, Deserialize)]
pub struct UpdateProfileRequest {
    /// Display name.
    pub display_name: Option<String>,
}

type ApiError = (StatusCode, Json<serde_json::Value>);

fn error_response(status: StatusCode, message: &str) -> ApiError {
    (status, Json(serde_json::json!({ "error": message })))
}

fn generate_token_pair(user_id: &str, secret: &[u8]) -> Result<TokenPair, ApiError> {
    cuttlefish_core::auth::jwt::generate_token_pair(user_id, secret).map_err(|e| {
        tracing::error!("Token generation failed: {}", e);
        error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to generate tokens",
        )
    })
}

/// Register a new user.
///
/// POST /api/auth/register
pub async fn register(
    State(state): State<AuthState>,
    Json(req): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<RegisterResponse>), ApiError> {
    if !state.registration_enabled {
        return Err(error_response(
            StatusCode::FORBIDDEN,
            "Registration is disabled",
        ));
    }

    let email = req.email.trim().to_lowercase();
    if !cuttlefish_core::auth::user::validate_email(&email) {
        return Err(error_response(
            StatusCode::BAD_REQUEST,
            "Invalid email format",
        ));
    }

    if auth::email_exists(&state.db, &email).await.map_err(|e| {
        tracing::error!("Database error checking email: {}", e);
        error_response(StatusCode::INTERNAL_SERVER_ERROR, "Database error")
    })? {
        return Err(error_response(
            StatusCode::CONFLICT,
            "Email already registered",
        ));
    }

    validate_password_strength(&req.password).map_err(|e| {
        error_response(
            StatusCode::BAD_REQUEST,
            &format!("Password too weak: {}", e),
        )
    })?;

    let password_hash = hash_password(&req.password).map_err(|e| {
        tracing::error!("Password hashing failed: {}", e);
        error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to process password",
        )
    })?;

    let user_id = Uuid::new_v4().to_string();
    let user = auth::create_user(
        &state.db,
        &user_id,
        &email,
        &password_hash,
        req.display_name.as_deref(),
    )
    .await
    .map_err(|e| {
        tracing::error!("Failed to create user: {}", e);
        error_response(StatusCode::INTERNAL_SERVER_ERROR, "Failed to create user")
    })?;

    tracing::info!(user_id = %user.id, email = %user.email, "User registered");

    Ok((
        StatusCode::CREATED,
        Json(RegisterResponse {
            user_id: user.id,
            email: user.email,
            message: "Registration successful".to_string(),
        }),
    ))
}

/// Login and receive tokens.
///
/// POST /api/auth/login
pub async fn login(
    State(state): State<AuthState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, ApiError> {
    let email = req.email.trim().to_lowercase();

    let user = auth::get_user_by_email(&state.db, &email)
        .await
        .map_err(|e| {
            tracing::error!("Database error: {}", e);
            error_response(StatusCode::INTERNAL_SERVER_ERROR, "Database error")
        })?
        .ok_or_else(|| error_response(StatusCode::UNAUTHORIZED, "Invalid email or password"))?;

    if !user.is_active() {
        return Err(error_response(
            StatusCode::FORBIDDEN,
            "Account is deactivated",
        ));
    }

    let password_valid = verify_password(&req.password, &user.password_hash).map_err(|e| {
        tracing::error!("Password verification error: {}", e);
        error_response(StatusCode::INTERNAL_SERVER_ERROR, "Authentication error")
    })?;

    if !password_valid {
        return Err(error_response(
            StatusCode::UNAUTHORIZED,
            "Invalid email or password",
        ));
    }

    let tokens = generate_token_pair(&user.id, &state.config.jwt_secret)?;

    sessions::enforce_session_limit(&state.db, &user.id)
        .await
        .map_err(|e| {
            tracing::error!("Session limit enforcement failed: {}", e);
            error_response(StatusCode::INTERNAL_SERVER_ERROR, "Session error")
        })?;

    let session_id = Uuid::new_v4().to_string();
    let refresh_token_hash =
        cuttlefish_core::auth::session::hash_refresh_token(&tokens.refresh_token);
    sessions::create_session(
        &state.db,
        &session_id,
        &user.id,
        &refresh_token_hash,
        None,
        None,
    )
    .await
    .map_err(|e| {
        tracing::error!("Failed to create session: {}", e);
        error_response(StatusCode::INTERNAL_SERVER_ERROR, "Session creation failed")
    })?;

    let _ = auth::update_last_login(&state.db, &user.id).await;

    tracing::info!(user_id = %user.id, "User logged in");

    Ok(Json(LoginResponse {
        access_token: tokens.access_token,
        refresh_token: tokens.refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: tokens.expires_in,
        user: UserInfo {
            id: user.id,
            email: user.email,
            display_name: user.display_name,
            email_verified: user.email_verified_at.is_some(),
            created_at: user.created_at,
        },
    }))
}

/// Refresh tokens.
///
/// POST /api/auth/refresh
pub async fn refresh_tokens(
    State(state): State<AuthState>,
    Json(req): Json<RefreshRequest>,
) -> Result<Json<RefreshResponse>, ApiError> {
    let token_hash = cuttlefish_core::auth::session::hash_refresh_token(&req.refresh_token);

    if sessions::was_token_previously_used(&state.db, &token_hash)
        .await
        .map_err(|e| {
            tracing::error!("Token reuse check failed: {}", e);
            error_response(StatusCode::INTERNAL_SERVER_ERROR, "Token validation error")
        })?
    {
        tracing::warn!("Refresh token reuse detected");
        return Err(error_response(
            StatusCode::UNAUTHORIZED,
            "Invalid refresh token",
        ));
    }

    let session = sessions::get_session_by_token_hash(&state.db, &token_hash)
        .await
        .map_err(|e| {
            tracing::error!("Session lookup failed: {}", e);
            error_response(StatusCode::INTERNAL_SERVER_ERROR, "Session lookup error")
        })?
        .ok_or_else(|| error_response(StatusCode::UNAUTHORIZED, "Invalid refresh token"))?;

    let expires_at = chrono::DateTime::parse_from_rfc3339(&session.expires_at)
        .map_err(|_| error_response(StatusCode::INTERNAL_SERVER_ERROR, "Invalid session data"))?;
    if expires_at < Utc::now() {
        return Err(error_response(
            StatusCode::UNAUTHORIZED,
            "Refresh token expired",
        ));
    }

    let tokens = generate_token_pair(&session.user_id, &state.config.jwt_secret)?;

    let new_token_hash = cuttlefish_core::auth::session::hash_refresh_token(&tokens.refresh_token);
    sessions::update_session_token(&state.db, &session.id, &new_token_hash)
        .await
        .map_err(|e| {
            tracing::error!("Session token update failed: {}", e);
            error_response(StatusCode::INTERNAL_SERVER_ERROR, "Token rotation failed")
        })?;

    tracing::debug!(session_id = %session.id, "Token refreshed");

    Ok(Json(RefreshResponse {
        access_token: tokens.access_token,
        refresh_token: tokens.refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: tokens.expires_in,
    }))
}

/// Logout and invalidate session.
///
/// POST /api/auth/logout
pub async fn logout(
    State(state): State<AuthState>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(req): Json<RefreshRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let token_hash = cuttlefish_core::auth::session::hash_refresh_token(&req.refresh_token);

    if let Some(session) = sessions::get_session_by_token_hash(&state.db, &token_hash)
        .await
        .map_err(|e| {
            tracing::error!("Session lookup failed: {}", e);
            error_response(StatusCode::INTERNAL_SERVER_ERROR, "Logout error")
        })?
        && session.user_id == user.user_id
    {
        sessions::revoke_session(&state.db, &session.id)
            .await
            .map_err(|e| {
                tracing::error!("Session revocation failed: {}", e);
                error_response(StatusCode::INTERNAL_SERVER_ERROR, "Logout error")
            })?;
        tracing::info!(user_id = %user.user_id, session_id = %session.id, "User logged out");
    }

    Ok(Json(
        serde_json::json!({ "message": "Logged out successfully" }),
    ))
}

/// Logout from all sessions.
///
/// POST /api/auth/logout-all
pub async fn logout_all(
    State(state): State<AuthState>,
    Extension(user): Extension<AuthenticatedUser>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let count = sessions::revoke_all_user_sessions(&state.db, &user.user_id)
        .await
        .map_err(|e| {
            tracing::error!("Bulk session revocation failed: {}", e);
            error_response(StatusCode::INTERNAL_SERVER_ERROR, "Logout error")
        })?;

    tracing::info!(user_id = %user.user_id, sessions_revoked = count, "User logged out from all sessions");

    Ok(Json(
        serde_json::json!({ "message": "Logged out from all sessions", "sessions_revoked": count }),
    ))
}

/// Get current user info.
///
/// GET /api/auth/me
pub async fn get_me(
    State(state): State<AuthState>,
    Extension(user): Extension<AuthenticatedUser>,
) -> Result<Json<UserInfo>, ApiError> {
    let db_user = auth::get_user_by_id(&state.db, &user.user_id)
        .await
        .map_err(|e| {
            tracing::error!("User lookup failed: {}", e);
            error_response(StatusCode::INTERNAL_SERVER_ERROR, "Database error")
        })?
        .ok_or_else(|| error_response(StatusCode::NOT_FOUND, "User not found"))?;

    Ok(Json(UserInfo {
        id: db_user.id,
        email: db_user.email,
        display_name: db_user.display_name,
        email_verified: db_user.email_verified_at.is_some(),
        created_at: db_user.created_at,
    }))
}

/// Update current user profile.
///
/// PUT /api/auth/me
pub async fn update_me(
    State(state): State<AuthState>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(req): Json<UpdateProfileRequest>,
) -> Result<Json<UserInfo>, ApiError> {
    auth::update_user_display_name(&state.db, &user.user_id, req.display_name.as_deref())
        .await
        .map_err(|e| {
            tracing::error!("Profile update failed: {}", e);
            error_response(StatusCode::INTERNAL_SERVER_ERROR, "Update failed")
        })?;

    let db_user = auth::get_user_by_id(&state.db, &user.user_id)
        .await
        .map_err(|e| {
            tracing::error!("User lookup failed: {}", e);
            error_response(StatusCode::INTERNAL_SERVER_ERROR, "Database error")
        })?
        .ok_or_else(|| error_response(StatusCode::NOT_FOUND, "User not found"))?;

    Ok(Json(UserInfo {
        id: db_user.id,
        email: db_user.email,
        display_name: db_user.display_name,
        email_verified: db_user.email_verified_at.is_some(),
        created_at: db_user.created_at,
    }))
}

/// Change password.
///
/// POST /api/auth/password
pub async fn change_password(
    State(state): State<AuthState>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(req): Json<ChangePasswordRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let db_user = auth::get_user_by_id(&state.db, &user.user_id)
        .await
        .map_err(|e| {
            tracing::error!("User lookup failed: {}", e);
            error_response(StatusCode::INTERNAL_SERVER_ERROR, "Database error")
        })?
        .ok_or_else(|| error_response(StatusCode::NOT_FOUND, "User not found"))?;

    let password_valid =
        verify_password(&req.current_password, &db_user.password_hash).map_err(|e| {
            tracing::error!("Password verification error: {}", e);
            error_response(StatusCode::INTERNAL_SERVER_ERROR, "Authentication error")
        })?;

    if !password_valid {
        return Err(error_response(
            StatusCode::UNAUTHORIZED,
            "Current password is incorrect",
        ));
    }

    validate_password_strength(&req.new_password).map_err(|e| {
        error_response(
            StatusCode::BAD_REQUEST,
            &format!("Password too weak: {}", e),
        )
    })?;

    let new_hash = hash_password(&req.new_password).map_err(|e| {
        tracing::error!("Password hashing failed: {}", e);
        error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to process password",
        )
    })?;

    auth::update_user_password(&state.db, &user.user_id, &new_hash)
        .await
        .map_err(|e| {
            tracing::error!("Password update failed: {}", e);
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to update password",
            )
        })?;

    sessions::revoke_all_user_sessions(&state.db, &user.user_id)
        .await
        .map_err(|e| {
            tracing::error!("Session revocation failed: {}", e);
            error_response(StatusCode::INTERNAL_SERVER_ERROR, "Session cleanup error")
        })?;

    tracing::info!(user_id = %user.user_id, "Password changed, all sessions revoked");

    Ok(Json(serde_json::json!({
        "message": "Password changed successfully. Please login again."
    })))
}

/// Request password reset.
///
/// POST /api/auth/reset-request
pub async fn request_password_reset(
    State(state): State<AuthState>,
    Json(req): Json<ResetPasswordRequestBody>,
) -> Json<ResetPasswordRequestResponse> {
    let email = req.email.trim().to_lowercase();

    let response = ResetPasswordRequestResponse {
        message: "If an account with that email exists, a password reset link has been sent."
            .to_string(),
    };

    let user = match auth::get_user_by_email(&state.db, &email).await {
        Ok(Some(u)) => u,
        Ok(None) => {
            tracing::debug!(email = %email, "Password reset requested for unknown email");
            return Json(response);
        }
        Err(e) => {
            tracing::error!("Database error during password reset: {}", e);
            return Json(response);
        }
    };

    if let Err(e) = password_reset::invalidate_user_reset_tokens(&state.db, &user.id).await {
        tracing::error!("Failed to invalidate existing reset tokens: {}", e);
    }

    let token = generate_reset_token();
    let token_id = Uuid::new_v4().to_string();

    if let Err(e) =
        password_reset::create_reset_token(&state.db, &token_id, &user.id, &token.hash).await
    {
        tracing::error!("Failed to create reset token: {}", e);
        return Json(response);
    }

    tracing::info!(
        user_id = %user.id,
        token = %token.plaintext,
        "Password reset token generated (would send email)"
    );

    Json(response)
}

/// Reset password with token.
///
/// POST /api/auth/reset
pub async fn reset_password(
    State(state): State<AuthState>,
    Json(req): Json<ResetPasswordBody>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let token_hash = hash_reset_token(&req.token);

    let reset_token = password_reset::get_reset_token_by_hash(&state.db, &token_hash)
        .await
        .map_err(|e| {
            tracing::error!("Token lookup failed: {}", e);
            error_response(StatusCode::INTERNAL_SERVER_ERROR, "Token validation error")
        })?
        .ok_or_else(|| error_response(StatusCode::BAD_REQUEST, "Invalid or expired reset token"))?;

    let expires_at = chrono::DateTime::parse_from_rfc3339(&reset_token.expires_at)
        .map_err(|_| error_response(StatusCode::INTERNAL_SERVER_ERROR, "Invalid token data"))?;
    if expires_at < Utc::now() {
        return Err(error_response(
            StatusCode::BAD_REQUEST,
            "Reset token has expired",
        ));
    }

    validate_password_strength(&req.new_password).map_err(|e| {
        error_response(
            StatusCode::BAD_REQUEST,
            &format!("Password too weak: {}", e),
        )
    })?;

    let new_hash = hash_password(&req.new_password).map_err(|e| {
        tracing::error!("Password hashing failed: {}", e);
        error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to process password",
        )
    })?;

    auth::update_user_password(&state.db, &reset_token.user_id, &new_hash)
        .await
        .map_err(|e| {
            tracing::error!("Password update failed: {}", e);
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to update password",
            )
        })?;

    password_reset::mark_reset_token_used(&state.db, &reset_token.id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to mark token used: {}", e);
            error_response(StatusCode::INTERNAL_SERVER_ERROR, "Token update error")
        })?;

    sessions::revoke_all_user_sessions(&state.db, &reset_token.user_id)
        .await
        .map_err(|e| {
            tracing::error!("Session revocation failed: {}", e);
            error_response(StatusCode::INTERNAL_SERVER_ERROR, "Session cleanup error")
        })?;

    tracing::info!(user_id = %reset_token.user_id, "Password reset completed");

    Ok(Json(serde_json::json!({
        "message": "Password reset successfully. Please login with your new password."
    })))
}

/// List user's API keys.
///
/// GET /api/auth/api-keys
pub async fn list_api_keys(
    State(state): State<AuthState>,
    Extension(user): Extension<AuthenticatedUser>,
) -> Result<Json<Vec<ApiKeySummary>>, ApiError> {
    let keys = api_keys::list_user_api_keys(&state.db, &user.user_id)
        .await
        .map_err(|e| {
            tracing::error!("API key list failed: {}", e);
            error_response(StatusCode::INTERNAL_SERVER_ERROR, "Database error")
        })?;

    Ok(Json(
        keys.into_iter()
            .map(|k| {
                let scopes = k.scopes();
                ApiKeySummary {
                    id: k.id,
                    prefix: k.key_prefix,
                    name: k.name,
                    scopes,
                    created_at: k.created_at,
                    last_used_at: k.last_used_at,
                    expires_at: k.expires_at,
                }
            })
            .collect(),
    ))
}

/// Create a new API key.
///
/// POST /api/auth/api-keys
pub async fn create_api_key_handler(
    State(state): State<AuthState>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(req): Json<CreateApiKeyRequest>,
) -> Result<(StatusCode, Json<CreateApiKeyResponse>), ApiError> {
    for scope in &req.scopes {
        if ApiKeyScope::parse(scope).is_none() {
            return Err(error_response(
                StatusCode::BAD_REQUEST,
                &format!("Invalid scope: {}", scope),
            ));
        }
    }

    let generated = generate_api_key();
    let key_id = Uuid::new_v4().to_string();

    let expires_at = req
        .expires_in_days
        .map(|days| (Utc::now() + chrono::Duration::days(days)).to_rfc3339());

    let db_key = api_keys::create_api_key(
        &state.db,
        &key_id,
        &user.user_id,
        &req.name,
        &generated.hash,
        &generated.prefix,
        &req.scopes,
        expires_at.as_deref(),
    )
    .await
    .map_err(|e| {
        tracing::error!("API key creation failed: {}", e);
        error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to create API key",
        )
    })?;

    tracing::info!(
        user_id = %user.user_id,
        key_id = %key_id,
        key_prefix = %generated.prefix,
        "API key created"
    );

    let scopes = db_key.scopes();
    Ok((
        StatusCode::CREATED,
        Json(CreateApiKeyResponse {
            id: db_key.id,
            api_key: generated.plaintext,
            prefix: db_key.key_prefix,
            name: db_key.name,
            scopes,
            created_at: db_key.created_at,
            expires_at: db_key.expires_at,
        }),
    ))
}

/// Delete (revoke) an API key.
///
/// DELETE /api/auth/api-keys/:id
pub async fn delete_api_key(
    State(state): State<AuthState>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(key_id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let key = api_keys::get_api_key_by_id(&state.db, &key_id)
        .await
        .map_err(|e| {
            tracing::error!("API key lookup failed: {}", e);
            error_response(StatusCode::INTERNAL_SERVER_ERROR, "Database error")
        })?
        .ok_or_else(|| error_response(StatusCode::NOT_FOUND, "API key not found"))?;

    if key.user_id != user.user_id {
        return Err(error_response(StatusCode::FORBIDDEN, "Access denied"));
    }

    if key.is_revoked() {
        return Err(error_response(
            StatusCode::BAD_REQUEST,
            "API key already revoked",
        ));
    }

    api_keys::revoke_api_key(&state.db, &key_id)
        .await
        .map_err(|e| {
            tracing::error!("API key revocation failed: {}", e);
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to revoke API key",
            )
        })?;

    tracing::info!(user_id = %user.user_id, key_id = %key_id, "API key revoked");

    Ok(Json(
        serde_json::json!({ "message": "API key revoked successfully" }),
    ))
}

/// Build the auth routes router.
pub fn auth_router(state: AuthState) -> Router {
    let auth_config = state.config.clone();

    let public_routes = Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/refresh", post(refresh_tokens))
        .route("/reset-request", post(request_password_reset))
        .route("/reset", post(reset_password));

    let protected_routes = Router::new()
        .route("/logout", post(logout))
        .route("/logout-all", post(logout_all))
        .route("/me", get(get_me))
        .route("/me", axum::routing::put(update_me))
        .route("/password", post(change_password))
        .route("/api-keys", get(list_api_keys).post(create_api_key_handler))
        .route("/api-keys/{id}", delete(delete_api_key))
        .layer(from_fn_with_state(
            auth_config,
            crate::middleware::require_auth,
        ));

    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_request_deserialize() {
        let json = r#"{"email": "test@example.com", "password": "SecurePass123!"}"#;
        let req: RegisterRequest = serde_json::from_str(json).expect("parse");
        assert_eq!(req.email, "test@example.com");
        assert_eq!(req.password, "SecurePass123!");
        assert!(req.display_name.is_none());
    }

    #[test]
    fn test_register_request_with_display_name() {
        let json =
            r#"{"email": "test@example.com", "password": "Pass123!", "display_name": "Test"}"#;
        let req: RegisterRequest = serde_json::from_str(json).expect("parse");
        assert_eq!(req.display_name, Some("Test".to_string()));
    }

    #[test]
    fn test_login_request_deserialize() {
        let json = r#"{"email": "test@example.com", "password": "password123"}"#;
        let req: LoginRequest = serde_json::from_str(json).expect("parse");
        assert_eq!(req.email, "test@example.com");
    }

    #[test]
    fn test_login_response_serialize() {
        let resp = LoginResponse {
            access_token: "access".to_string(),
            refresh_token: "refresh".to_string(),
            token_type: "Bearer".to_string(),
            expires_in: 86400,
            user: UserInfo {
                id: "123".to_string(),
                email: "test@example.com".to_string(),
                display_name: None,
                email_verified: false,
                created_at: "2024-01-01T00:00:00Z".to_string(),
            },
        };
        let json = serde_json::to_string(&resp).expect("serialize");
        assert!(json.contains("Bearer"));
        assert!(json.contains("access"));
    }

    #[test]
    fn test_refresh_request_deserialize() {
        let json = r#"{"refresh_token": "my-refresh-token"}"#;
        let req: RefreshRequest = serde_json::from_str(json).expect("parse");
        assert_eq!(req.refresh_token, "my-refresh-token");
    }

    #[test]
    fn test_create_api_key_request_deserialize() {
        let json = r#"{"name": "My Key", "scopes": ["read", "write"], "expires_in_days": 30}"#;
        let req: CreateApiKeyRequest = serde_json::from_str(json).expect("parse");
        assert_eq!(req.name, "My Key");
        assert_eq!(req.scopes, vec!["read", "write"]);
        assert_eq!(req.expires_in_days, Some(30));
    }

    #[test]
    fn test_api_key_summary_serialize() {
        let summary = ApiKeySummary {
            id: "key-1".to_string(),
            prefix: "cfish_abc".to_string(),
            name: "Test Key".to_string(),
            scopes: vec!["read".to_string()],
            created_at: "2024-01-01T00:00:00Z".to_string(),
            last_used_at: None,
            expires_at: None,
        };
        let json = serde_json::to_string(&summary).expect("serialize");
        assert!(json.contains("cfish_abc"));
        assert!(!json.contains("api_key"));
    }

    #[test]
    fn test_change_password_request_deserialize() {
        let json = r#"{"current_password": "old", "new_password": "new"}"#;
        let req: ChangePasswordRequest = serde_json::from_str(json).expect("parse");
        assert_eq!(req.current_password, "old");
        assert_eq!(req.new_password, "new");
    }

    #[test]
    fn test_reset_password_request_deserialize() {
        let json = r#"{"email": "reset@example.com"}"#;
        let req: ResetPasswordRequestBody = serde_json::from_str(json).expect("parse");
        assert_eq!(req.email, "reset@example.com");
    }

    #[test]
    fn test_reset_password_body_deserialize() {
        let json = r#"{"token": "abc123", "new_password": "NewPass123!"}"#;
        let req: ResetPasswordBody = serde_json::from_str(json).expect("parse");
        assert_eq!(req.token, "abc123");
        assert_eq!(req.new_password, "NewPass123!");
    }

    #[test]
    fn test_user_info_serialize() {
        let info = UserInfo {
            id: "user-1".to_string(),
            email: "user@example.com".to_string(),
            display_name: Some("User".to_string()),
            email_verified: true,
            created_at: "2024-01-01T00:00:00Z".to_string(),
        };
        let json = serde_json::to_string(&info).expect("serialize");
        assert!(json.contains("user@example.com"));
        assert!(json.contains("email_verified"));
    }
}
