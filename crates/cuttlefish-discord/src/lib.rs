#![deny(unsafe_code)]
#![warn(missing_docs)]
//! Discord bot integration for Cuttlefish.
//!
//! Provides a Discord bot that:
//! - Registers slash commands for project management
//! - Creates channels for new projects
//! - Routes messages to the agent system

/// Discord channel management.
pub mod channel_manager;
/// Slash command parsing and definitions.
pub mod commands;
/// Discord message formatting.
pub mod formatter;
/// Per-guild configuration for multi-server support.
pub mod guild_config;
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

pub use channel_manager::ChannelManager;
pub use formatter::{
    format_code_block, format_diff, format_status, split_message, DISCORD_MESSAGE_LIMIT,
};
pub use guild_config::{GuildConfig, GuildConfigStore};
