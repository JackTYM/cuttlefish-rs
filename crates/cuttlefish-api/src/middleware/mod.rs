//! Authentication and authorization middleware.
//!
//! This module provides middleware for:
//! - JWT token validation
//! - API key validation
//! - Request authentication

pub mod auth;

pub use auth::{AuthConfig, AuthMethod, AuthenticatedUser, optional_auth, require_auth};
