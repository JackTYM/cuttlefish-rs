//! Database model types for all tables.

use serde::{Deserialize, Serialize};

/// A project managed by Cuttlefish.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Project {
    /// Unique project identifier (UUID string).
    pub id: String,
    /// Project name (unique within the system).
    pub name: String,
    /// Project description.
    pub description: String,
    /// Project status (e.g., "active", "completed", "archived").
    pub status: String,
    /// Optional template name used to initialize the project.
    pub template_name: Option<String>,
    /// Optional GitHub repository URL.
    pub github_url: Option<String>,
    /// Optional Discord channel ID for notifications.
    pub discord_channel_id: Option<String>,
    /// Optional Discord guild ID.
    pub discord_guild_id: Option<String>,
    /// Optional Docker container ID.
    pub docker_container_id: Option<String>,
    /// Timestamp when the project was created (ISO 8601 format).
    pub created_at: String,
    /// Timestamp when the project was last updated (ISO 8601 format).
    pub updated_at: String,
}

/// A conversation message associated with a project.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Conversation {
    /// Unique message identifier (UUID string).
    pub id: String,
    /// Project ID this message belongs to.
    pub project_id: String,
    /// Role of the message sender (e.g., "user", "assistant", "system").
    pub role: String,
    /// Message content.
    pub content: String,
    /// Optional model name used to generate the message.
    pub model_used: Option<String>,
    /// Token count for this message.
    pub token_count: i64,
    /// Archive status (0 = active, 1 = archived).
    pub archived: i64,
    /// Timestamp when the message was created (ISO 8601 format).
    pub created_at: String,
}

/// An agent session for a project.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AgentSession {
    /// Unique session identifier (UUID string).
    pub id: String,
    /// Project ID this session belongs to.
    pub project_id: String,
    /// Role of the agent (e.g., "builder", "reviewer", "deployer").
    pub agent_role: String,
    /// Session status (e.g., "running", "completed", "failed").
    pub status: String,
    /// Timestamp when the session started (ISO 8601 format).
    pub started_at: String,
    /// Timestamp when the session completed (ISO 8601 format), if applicable.
    pub completed_at: Option<String>,
}

/// A template for project initialization.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Template {
    /// Unique template identifier (UUID string).
    pub id: String,
    /// Template name (unique within the system).
    pub name: String,
    /// Template description.
    pub description: String,
    /// Template content in Markdown format.
    pub content_md: String,
    /// Programming language or framework the template targets.
    pub language: String,
    /// Timestamp when the template was created (ISO 8601 format).
    pub created_at: String,
}

/// A build log entry for a project.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct BuildLog {
    /// Unique log entry identifier (UUID string).
    pub id: String,
    /// Project ID this build log belongs to.
    pub project_id: String,
    /// Build status (e.g., "running", "success", "failed").
    pub status: String,
    /// Build command that was executed.
    pub command: String,
    /// Build output/logs.
    pub output: String,
    /// Timestamp when the build started (ISO 8601 format).
    pub started_at: String,
    /// Timestamp when the build completed (ISO 8601 format), if applicable.
    pub completed_at: Option<String>,
}

/// A configuration override for a project.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ConfigOverride {
    /// Unique override identifier (UUID string).
    pub id: String,
    /// Project ID this override applies to.
    pub project_id: String,
    /// Configuration key being overridden.
    pub key: String,
    /// Configuration value.
    pub value: String,
}

/// A tunnel link code for initial authentication.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TunnelLinkCode {
    /// Unique identifier.
    pub id: String,
    /// User who created this link code.
    pub user_id: String,
    /// SHA-256 hash of the link code (never store plaintext).
    pub code_hash: String,
    /// Subdomain to assign on successful auth.
    pub subdomain: String,
    /// When the link code was created.
    pub created_at: String,
    /// When the link code expires.
    pub expires_at: String,
    /// When the link code was used (None if unused).
    pub used_at: Option<String>,
}

/// An active tunnel connection.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ActiveTunnel {
    /// Unique identifier.
    pub id: String,
    /// User who owns this tunnel.
    pub user_id: String,
    /// Subdomain for this tunnel.
    pub subdomain: String,
    /// When the tunnel connected.
    pub connected_at: String,
    /// Last heartbeat timestamp.
    pub last_heartbeat: String,
    /// Client version string.
    pub client_version: Option<String>,
    /// Client IP address.
    pub client_ip: Option<String>,
}

/// A user account in the Cuttlefish system.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    /// Unique user identifier (UUID string).
    pub id: String,
    /// User's email address (unique).
    pub email: String,
    /// Argon2id password hash.
    pub password_hash: String,
    /// Optional display name.
    pub display_name: Option<String>,
    /// When the user was created (ISO 8601 format).
    pub created_at: String,
    /// When the user was last updated (ISO 8601 format).
    pub updated_at: String,
    /// When the email was verified (ISO 8601 format), if applicable.
    pub email_verified_at: Option<String>,
    /// When the user last logged in (ISO 8601 format).
    pub last_login_at: Option<String>,
    /// Whether the user account is active (stored as INTEGER in SQLite).
    pub is_active: i64,
}

impl User {
    /// Check if the user account is active.
    pub fn is_active(&self) -> bool {
        self.is_active != 0
    }
}

/// An authentication session for a user.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Session {
    /// Unique session identifier (UUID string).
    pub id: String,
    /// User ID this session belongs to.
    pub user_id: String,
    /// Hash of the refresh token (never store plaintext).
    pub refresh_token_hash: String,
    /// Client user agent string.
    pub user_agent: Option<String>,
    /// Client IP address.
    pub ip_address: Option<String>,
    /// When the session was created (ISO 8601 format).
    pub created_at: String,
    /// When the session expires (ISO 8601 format).
    pub expires_at: String,
    /// When the session was revoked (ISO 8601 format), if applicable.
    pub revoked_at: Option<String>,
}

impl Session {
    /// Check if the session is revoked.
    pub fn is_revoked(&self) -> bool {
        self.revoked_at.is_some()
    }
}

/// An API key for programmatic access.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ApiKey {
    /// Unique API key identifier (UUID string).
    pub id: String,
    /// User ID this key belongs to.
    pub user_id: String,
    /// Human-readable name for the key.
    pub name: String,
    /// SHA-256 hash of the full key (never store plaintext).
    pub key_hash: String,
    /// First 8 characters of the key for identification.
    pub key_prefix: String,
    /// JSON array of scopes (read, write, admin).
    pub scopes: String,
    /// When the key was created (ISO 8601 format).
    pub created_at: String,
    /// When the key was last used (ISO 8601 format).
    pub last_used_at: Option<String>,
    /// When the key expires (ISO 8601 format), if applicable.
    pub expires_at: Option<String>,
    /// When the key was revoked (ISO 8601 format), if applicable.
    pub revoked_at: Option<String>,
}

impl ApiKey {
    /// Check if the API key is revoked.
    pub fn is_revoked(&self) -> bool {
        self.revoked_at.is_some()
    }

    /// Parse the scopes JSON into a vector of strings.
    pub fn scopes(&self) -> Vec<String> {
        serde_json::from_str(&self.scopes).unwrap_or_default()
    }

    /// Check if the key has a specific scope.
    pub fn has_scope(&self, scope: &str) -> bool {
        self.scopes().iter().any(|s| s == scope)
    }
}

/// A project membership record.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ProjectMember {
    /// Unique membership identifier (UUID string).
    pub id: String,
    /// Project ID.
    pub project_id: String,
    /// User ID.
    pub user_id: String,
    /// Role: owner, admin, member, or viewer.
    pub role: String,
    /// User ID of who invited this member.
    pub invited_by: Option<String>,
    /// When the membership was created (ISO 8601 format).
    pub created_at: String,
}

/// A password reset token.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PasswordResetToken {
    /// Unique token identifier (UUID string).
    pub id: String,
    /// User ID this token is for.
    pub user_id: String,
    /// SHA-256 hash of the reset token (never store plaintext).
    pub token_hash: String,
    /// When the token was created (ISO 8601 format).
    pub created_at: String,
    /// When the token expires (ISO 8601 format).
    pub expires_at: String,
    /// When the token was used (ISO 8601 format), if applicable.
    pub used_at: Option<String>,
}

impl PasswordResetToken {
    /// Check if the token has been used.
    pub fn is_used(&self) -> bool {
        self.used_at.is_some()
    }
}

/// Project role for access control.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProjectRole {
    /// Full control over the project.
    Owner,
    /// Can manage members and settings.
    Admin,
    /// Can work on the project.
    Member,
    /// Read-only access.
    Viewer,
}

impl ProjectRole {
    /// Convert role to string representation.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Owner => "owner",
            Self::Admin => "admin",
            Self::Member => "member",
            Self::Viewer => "viewer",
        }
    }

    /// Parse role from string.
    pub fn parse(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "owner" => Self::Owner,
            "admin" => Self::Admin,
            "member" => Self::Member,
            _ => Self::Viewer,
        }
    }

    /// Get numeric level for comparison (higher = more permissions).
    pub fn level(&self) -> u8 {
        match self {
            Self::Owner => 4,
            Self::Admin => 3,
            Self::Member => 2,
            Self::Viewer => 1,
        }
    }

    /// Check if this role has at least the permissions of another role.
    pub fn has_at_least(&self, other: Self) -> bool {
        self.level() >= other.level()
    }
}

/// A project share record granting access to a user.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ProjectShare {
    /// Unique share identifier (UUID string).
    pub id: String,
    /// Project ID being shared.
    pub project_id: String,
    /// User ID receiving access.
    pub user_id: String,
    /// Role granted to the user.
    pub role: String,
    /// User ID who created this share.
    pub shared_by: String,
    /// When the share was created (ISO 8601 format).
    pub created_at: String,
}

/// A project invite for users who don't have an account yet.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ProjectInvite {
    /// Unique invite identifier (UUID string).
    pub id: String,
    /// Project ID being shared.
    pub project_id: String,
    /// Email address of the invitee.
    pub email: String,
    /// Role to grant upon acceptance.
    pub role: String,
    /// URL-safe token for accepting the invite.
    pub token: String,
    /// User ID who created this invite.
    pub invited_by: String,
    /// When the invite was created (ISO 8601 format).
    pub created_at: String,
    /// When the invite expires (ISO 8601 format).
    pub expires_at: String,
    /// When the invite was accepted (ISO 8601 format), if applicable.
    pub accepted_at: Option<String>,
}

impl ProjectInvite {
    /// Check if the invite has been accepted.
    pub fn is_accepted(&self) -> bool {
        self.accepted_at.is_some()
    }

    /// Check if the invite has expired.
    pub fn is_expired(&self, now: &str) -> bool {
        self.expires_at.as_str() < now
    }
}

/// Activity action types for the activity log.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ActivityAction {
    /// Project was created.
    ProjectCreated,
    /// A member was added to the project.
    MemberAdded {
        /// User ID of the new member.
        member_id: String,
        /// Role granted.
        role: String,
    },
    /// A member was removed from the project.
    MemberRemoved {
        /// User ID of the removed member.
        member_id: String,
    },
    /// A member's role was changed.
    RoleChanged {
        /// User ID of the member.
        member_id: String,
        /// Previous role.
        old_role: String,
        /// New role.
        new_role: String,
    },
    /// An agent task was started.
    AgentTaskStarted {
        /// Description of the task.
        task_description: String,
    },
    /// An agent task was completed.
    AgentTaskCompleted {
        /// Description of the task.
        task_description: String,
        /// Whether the task succeeded.
        success: bool,
    },
    /// A file was changed.
    FileChanged {
        /// Path to the file.
        path: String,
        /// Type of change (created, modified, deleted).
        change_type: String,
    },
    /// A user joined the project (accepted invite).
    UserJoined {
        /// User ID who joined.
        user_id: String,
    },
    /// A user left the project.
    UserLeft {
        /// User ID who left.
        user_id: String,
    },
    /// Project settings were changed.
    SettingsChanged {
        /// Setting that was changed.
        setting: String,
    },
    /// An invite was sent.
    InviteSent {
        /// Email the invite was sent to (masked for privacy).
        email_masked: String,
        /// Role offered.
        role: String,
    },
    /// A handoff was created.
    HandoffCreated {
        /// User ID assigned (None = open to anyone).
        to_user_id: Option<String>,
    },
    /// A handoff was accepted.
    HandoffAccepted {
        /// User ID who accepted.
        by_user_id: String,
    },
}

/// An activity log entry.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ActivityEntry {
    /// Unique entry identifier (UUID string).
    pub id: String,
    /// Project ID this activity belongs to.
    pub project_id: String,
    /// User ID who performed the action.
    pub user_id: String,
    /// Action type (serialized JSON).
    pub action: String,
    /// Additional details (JSON).
    pub details: Option<String>,
    /// When the activity occurred (ISO 8601 format).
    pub created_at: String,
}
