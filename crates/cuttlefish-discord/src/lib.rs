#![deny(unsafe_code)]
#![warn(missing_docs)]
//! Discord bot integration for Cuttlefish.
//!
//! Provides a Discord bot that:
//! - Registers slash commands for project management
//! - Creates channels for new projects
//! - Routes messages to the agent system

/// HTTP client for communicating with the Cuttlefish API.
pub mod api_client;
/// Channel archival system for inactive projects.
pub mod archival;
/// Discord channel management.
pub mod channel_manager;
/// Slash command handlers for the new command framework.
pub mod commands;
/// Rich embed builders for Discord messages.
pub mod embeds;
/// Discord message formatting.
pub mod formatter;
/// Per-guild configuration for multi-server support.
pub mod guild_config;
/// Legacy command parsing (deprecated, use `commands` module instead).
#[deprecated(
    since = "0.2.0",
    note = "Use the `commands` module for slash command handling"
)]
#[allow(deprecated)]
pub mod legacy_commands;

/// Discord configuration.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct DiscordConfig {
    /// Bot token (from DISCORD_BOT_TOKEN env var).
    pub token: String,
    /// Guild IDs to restrict command registration (empty = all guilds).
    #[serde(default)]
    pub guild_ids: Vec<u64>,
}

/// The name used for the Cuttlefish project category in Discord.
pub const PROJECT_CATEGORY_NAME: &str = "🐙 Cuttlefish Projects";
/// The name used for the archived projects category.
pub const ARCHIVED_CATEGORY_NAME: &str = "📦 Archived Projects";

pub use api_client::{ApiClient, ApiClientConfig, ApiError, get_api_client, init_api_client};
pub use archival::{ArchiveConfig, ChannelArchival};
pub use channel_manager::{ChannelManager, PendingAction, PendingActionType};
pub use embeds::{
    AgentStatus, AgentStatusEmbed, AgentType, BuildProgressEmbed, ErrorEmbed, ProgressEmbed,
    ProgressStep, QuestionEmbed, StepStatus, SubAgentStatus, TaskCompletionEmbed, TestResults,
};
pub use formatter::{
    DISCORD_MESSAGE_LIMIT, format_code_block, format_diff, format_status, split_message,
};
pub use guild_config::{GuildConfig, GuildConfigStore};
