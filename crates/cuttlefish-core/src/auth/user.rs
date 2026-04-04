//! User model and types for authentication.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier for a user.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UserId(pub String);

impl UserId {
    /// Create a new random user ID.
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }

    /// Create a user ID from an existing string.
    pub fn from_string(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    /// Get the inner string value.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for UserId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for UserId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for UserId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for UserId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// A user in the Cuttlefish system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// Unique user identifier.
    pub id: UserId,
    /// User's email address.
    pub email: String,
    /// Optional display name.
    pub display_name: Option<String>,
    /// When the user was created.
    pub created_at: DateTime<Utc>,
    /// When the user was last updated.
    pub updated_at: DateTime<Utc>,
    /// Whether the email has been verified.
    pub email_verified: bool,
    /// When the email was verified, if applicable.
    pub email_verified_at: Option<DateTime<Utc>>,
    /// When the user last logged in.
    pub last_login_at: Option<DateTime<Utc>>,
    /// Whether the user account is active.
    pub is_active: bool,
}

/// Request to create a new user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserRequest {
    /// User's email address.
    pub email: String,
    /// User's password (plaintext, will be hashed).
    pub password: String,
    /// Optional display name.
    pub display_name: Option<String>,
}

impl CreateUserRequest {
    /// Create a new user creation request.
    pub fn new(email: impl Into<String>, password: impl Into<String>) -> Self {
        Self {
            email: email.into(),
            password: password.into(),
            display_name: None,
        }
    }

    /// Set the display name.
    pub fn with_display_name(mut self, name: impl Into<String>) -> Self {
        self.display_name = Some(name.into());
        self
    }
}

/// Validate email format (basic check).
pub fn validate_email(email: &str) -> bool {
    let email = email.trim();
    if email.is_empty() || email.len() > 254 {
        return false;
    }

    let parts: Vec<&str> = email.split('@').collect();
    if parts.len() != 2 {
        return false;
    }

    let local = parts[0];
    let domain = parts[1];

    if local.is_empty() || local.len() > 64 {
        return false;
    }

    if domain.is_empty() || !domain.contains('.') {
        return false;
    }

    let domain_parts: Vec<&str> = domain.split('.').collect();
    if domain_parts.iter().any(|p| p.is_empty()) {
        return false;
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_id_new() {
        let id1 = UserId::new();
        let id2 = UserId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_user_id_from_string() {
        let id = UserId::from_string("test-id");
        assert_eq!(id.as_str(), "test-id");
    }

    #[test]
    fn test_user_id_display() {
        let id = UserId::from_string("display-test");
        assert_eq!(format!("{}", id), "display-test");
    }

    #[test]
    fn test_create_user_request() {
        let req = CreateUserRequest::new("test@example.com", "password123")
            .with_display_name("Test User");
        assert_eq!(req.email, "test@example.com");
        assert_eq!(req.password, "password123");
        assert_eq!(req.display_name, Some("Test User".to_string()));
    }

    #[test]
    fn test_validate_email_valid() {
        assert!(validate_email("test@example.com"));
        assert!(validate_email("user.name@domain.co.uk"));
        assert!(validate_email("user+tag@example.org"));
    }

    #[test]
    fn test_validate_email_invalid() {
        assert!(!validate_email(""));
        assert!(!validate_email("no-at-sign"));
        assert!(!validate_email("@no-local.com"));
        assert!(!validate_email("no-domain@"));
        assert!(!validate_email("no-tld@domain"));
        assert!(!validate_email("double@@at.com"));
    }
}
