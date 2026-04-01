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
