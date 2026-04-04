//! Authentication module for Cuttlefish.
//!
//! This module provides:
//! - User model and types
//! - Password hashing with Argon2id
//! - JWT token generation and validation
//! - Session management and refresh token rotation
//! - API key generation and validation
//! - Role-based access control
//! - Password reset flow

pub mod api_key;
pub mod jwt;
pub mod password;
pub mod reset;
pub mod role;
pub mod session;
pub mod user;

pub use api_key::{
    ApiKeyScope, GeneratedApiKey, generate_api_key, has_required_scope, hash_api_key,
    validate_api_key_format,
};
pub use jwt::{
    TokenClaims, TokenPair, TokenType, generate_access_token, generate_refresh_token,
    validate_token,
};
pub use password::{hash_password, validate_password_strength, verify_password};
pub use reset::{GeneratedResetToken, generate_reset_token, hash_reset_token, verify_reset_token};
pub use role::{Action, Role, RoleError, can_perform};
pub use session::{
    RefreshResult, SessionError, SessionInfo, SessionMetadata, generate_refresh_token_value,
    hash_refresh_token, verify_refresh_token,
};
pub use user::{CreateUserRequest, User, UserId};

use thiserror::Error;

/// Authentication-related errors.
#[derive(Error, Debug)]
pub enum AuthError {
    /// Password hashing failed.
    #[error("Password hashing failed: {0}")]
    HashingFailed(String),

    /// Password verification failed.
    #[error("Password verification failed: {0}")]
    VerificationFailed(String),

    /// Password does not meet strength requirements.
    #[error("Password too weak: {0}")]
    WeakPassword(String),

    /// JWT token generation failed.
    #[error("Token generation failed: {0}")]
    TokenGenerationFailed(String),

    /// JWT token validation failed.
    #[error("Token validation failed: {0}")]
    TokenValidationFailed(String),

    /// Token has expired.
    #[error("Token has expired")]
    TokenExpired,

    /// Invalid token type.
    #[error("Invalid token type: expected {expected}, got {got}")]
    InvalidTokenType {
        /// Expected token type.
        expected: String,
        /// Actual token type.
        got: String,
    },

    /// User not found.
    #[error("User not found: {0}")]
    UserNotFound(String),

    /// Email already exists.
    #[error("Email already exists: {0}")]
    EmailAlreadyExists(String),

    /// Invalid email format.
    #[error("Invalid email format: {0}")]
    InvalidEmail(String),

    /// Database error.
    #[error("Database error: {0}")]
    Database(String),

    /// Invalid API key.
    #[error("Invalid API key: {0}")]
    InvalidApiKey(String),

    /// API key expired.
    #[error("API key has expired")]
    ApiKeyExpired,

    /// API key revoked.
    #[error("API key has been revoked")]
    ApiKeyRevoked,

    /// Insufficient scope.
    #[error("Insufficient scope: required {required}")]
    InsufficientScope {
        /// The required scope.
        required: String,
    },

    /// Invalid reset token.
    #[error("Invalid or expired reset token")]
    InvalidResetToken,

    /// Reset token already used.
    #[error("Reset token has already been used")]
    ResetTokenUsed,
}
