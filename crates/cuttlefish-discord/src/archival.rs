//! Channel archival system for inactive projects.
//!
//! This module provides:
//! - Automatic archival of inactive project channels
//! - Manual archive/unarchive operations
//! - Configurable inactivity timeout
//! - Archive category management

use std::time::Duration;

use cuttlefish_core::error::DiscordError;
use serenity::all::{ChannelId, ChannelType, Context, GuildId};
use serenity::builder::{CreateChannel, CreateEmbed, CreateMessage};
use serenity::model::Colour;

use crate::{ARCHIVED_CATEGORY_NAME, PROJECT_CATEGORY_NAME};

/// Configuration for channel archival behavior.
#[derive(Debug, Clone)]
pub struct ArchiveConfig {
    /// Inactivity timeout before auto-archive (in days).
    pub inactivity_days: u32,
    /// Whether to auto-archive channels.
    pub auto_archive: bool,
    /// Custom archive category name (defaults to ARCHIVED_CATEGORY_NAME).
    pub archive_category_name: Option<String>,
}

impl Default for ArchiveConfig {
    fn default() -> Self {
        Self {
            inactivity_days: 30,
            auto_archive: true,
            archive_category_name: None,
        }
    }
}

impl ArchiveConfig {
    /// Create a new archive config with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the inactivity timeout in days.
    pub fn inactivity_days(mut self, days: u32) -> Self {
        self.inactivity_days = days;
        self
    }

    /// Enable or disable auto-archive.
    pub fn auto_archive(mut self, enabled: bool) -> Self {
        self.auto_archive = enabled;
        self
    }

    /// Set a custom archive category name.
    pub fn archive_category_name(mut self, name: impl Into<String>) -> Self {
        self.archive_category_name = Some(name.into());
        self
    }

    /// Get the inactivity timeout as a Duration.
    pub fn inactivity_duration(&self) -> Duration {
        Duration::from_secs(u64::from(self.inactivity_days) * 24 * 60 * 60)
    }
}

/// Channel archival operations.
pub struct ChannelArchival;

impl ChannelArchival {
    /// Archive a project channel.
    ///
    /// This will:
    /// 1. Move the channel to the archive category
    /// 2. Rename with `archived-` prefix
    /// 3. Send a summary message
    /// 4. Set read-only permissions (optional)
    ///
    /// # Errors
    ///
    /// Returns an error if the channel doesn't exist or Discord API fails.
    pub async fn archive_channel(
        ctx: &Context,
        guild_id: GuildId,
        channel_id: ChannelId,
        config: &ArchiveConfig,
    ) -> Result<ArchivalResult, DiscordError> {
        let guild_channels = guild_id
            .channels(&ctx.http)
            .await
            .map_err(|e| DiscordError(format!("Failed to fetch guild channels: {e}")))?;

        let channel = guild_channels
            .get(&channel_id)
            .ok_or_else(|| DiscordError("Channel not found".to_string()))?;

        if channel.kind != ChannelType::Text {
            return Err(DiscordError("Can only archive text channels".to_string()));
        }

        let original_name = channel.name.clone();
        let archived_name = format!("archived-{}", original_name);

        if original_name.starts_with("archived-") {
            return Err(DiscordError("Channel is already archived".to_string()));
        }

        let archive_category = Self::get_or_create_archive_category(ctx, guild_id, config).await?;

        let edit_builder = serenity::builder::EditChannel::new()
            .name(&archived_name)
            .category(archive_category);

        channel_id
            .edit(&ctx.http, edit_builder)
            .await
            .map_err(|e| DiscordError(format!("Failed to archive channel: {e}")))?;

        Self::send_archive_summary(ctx, channel_id, &original_name).await?;

        tracing::info!(
            channel_id = %channel_id,
            original_name = %original_name,
            archived_name = %archived_name,
            "Archived project channel"
        );

        Ok(ArchivalResult {
            channel_id,
            original_name,
            archived_name,
        })
    }

    /// Unarchive a project channel.
    ///
    /// This will:
    /// 1. Move the channel back to the projects category
    /// 2. Remove the `archived-` prefix
    /// 3. Restore write permissions
    ///
    /// # Errors
    ///
    /// Returns an error if the channel doesn't exist or isn't archived.
    pub async fn unarchive_channel(
        ctx: &Context,
        guild_id: GuildId,
        channel_id: ChannelId,
    ) -> Result<ArchivalResult, DiscordError> {
        let guild_channels = guild_id
            .channels(&ctx.http)
            .await
            .map_err(|e| DiscordError(format!("Failed to fetch guild channels: {e}")))?;

        let channel = guild_channels
            .get(&channel_id)
            .ok_or_else(|| DiscordError("Channel not found".to_string()))?;

        if channel.kind != ChannelType::Text {
            return Err(DiscordError("Can only unarchive text channels".to_string()));
        }

        let archived_name = channel.name.clone();

        if !archived_name.starts_with("archived-") {
            return Err(DiscordError("Channel is not archived".to_string()));
        }

        let original_name = archived_name
            .strip_prefix("archived-")
            .expect("already checked prefix")
            .to_string();

        let projects_category = Self::get_or_create_projects_category(ctx, guild_id).await?;

        let edit_builder = serenity::builder::EditChannel::new()
            .name(&original_name)
            .category(projects_category);

        channel_id
            .edit(&ctx.http, edit_builder)
            .await
            .map_err(|e| DiscordError(format!("Failed to unarchive channel: {e}")))?;

        Self::send_unarchive_message(ctx, channel_id, &original_name).await?;

        tracing::info!(
            channel_id = %channel_id,
            original_name = %original_name,
            archived_name = %archived_name,
            "Unarchived project channel"
        );

        Ok(ArchivalResult {
            channel_id,
            original_name,
            archived_name,
        })
    }

    /// Check if a channel is archived.
    pub fn is_archived(channel_name: &str) -> bool {
        channel_name.starts_with("archived-")
    }

    /// Get the original name from an archived channel name.
    pub fn get_original_name(archived_name: &str) -> Option<&str> {
        archived_name.strip_prefix("archived-")
    }

    async fn get_or_create_archive_category(
        ctx: &Context,
        guild_id: GuildId,
        config: &ArchiveConfig,
    ) -> Result<ChannelId, DiscordError> {
        let category_name = config
            .archive_category_name
            .as_deref()
            .unwrap_or(ARCHIVED_CATEGORY_NAME);

        let guild_channels = guild_id
            .channels(&ctx.http)
            .await
            .map_err(|e| DiscordError(format!("Failed to fetch guild channels: {e}")))?;

        let existing = guild_channels
            .values()
            .find(|c| c.kind == ChannelType::Category && c.name == category_name);

        if let Some(category) = existing {
            return Ok(category.id);
        }

        let builder = CreateChannel::new(category_name).kind(ChannelType::Category);

        let category = guild_id
            .create_channel(&ctx.http, builder)
            .await
            .map_err(|e| DiscordError(format!("Failed to create archive category: {e}")))?;

        tracing::info!(
            category_id = %category.id,
            guild_id = %guild_id,
            "Created archive category"
        );

        Ok(category.id)
    }

    async fn get_or_create_projects_category(
        ctx: &Context,
        guild_id: GuildId,
    ) -> Result<ChannelId, DiscordError> {
        let guild_channels = guild_id
            .channels(&ctx.http)
            .await
            .map_err(|e| DiscordError(format!("Failed to fetch guild channels: {e}")))?;

        let existing = guild_channels
            .values()
            .find(|c| c.kind == ChannelType::Category && c.name == PROJECT_CATEGORY_NAME);

        if let Some(category) = existing {
            return Ok(category.id);
        }

        let builder = CreateChannel::new(PROJECT_CATEGORY_NAME).kind(ChannelType::Category);

        let category = guild_id
            .create_channel(&ctx.http, builder)
            .await
            .map_err(|e| DiscordError(format!("Failed to create projects category: {e}")))?;

        Ok(category.id)
    }

    async fn send_archive_summary(
        ctx: &Context,
        channel_id: ChannelId,
        original_name: &str,
    ) -> Result<(), DiscordError> {
        let embed = CreateEmbed::new()
            .title("📦 Channel Archived")
            .description(format!(
                "This project channel has been archived due to inactivity.\n\n\
                 **Original name:** `{original_name}`\n\n\
                 Use `/unarchive` to restore this channel if needed."
            ))
            .colour(Colour::from_rgb(128, 128, 128))
            .footer(serenity::builder::CreateEmbedFooter::new(
                "Archived by Cuttlefish",
            ));

        let message = CreateMessage::new().embed(embed);

        channel_id
            .send_message(&ctx.http, message)
            .await
            .map_err(|e| DiscordError(format!("Failed to send archive summary: {e}")))?;

        Ok(())
    }

    async fn send_unarchive_message(
        ctx: &Context,
        channel_id: ChannelId,
        original_name: &str,
    ) -> Result<(), DiscordError> {
        let embed = CreateEmbed::new()
            .title("🔄 Channel Restored")
            .description(format!(
                "This project channel has been unarchived and is now active.\n\n\
                 **Channel name:** `{original_name}`"
            ))
            .colour(Colour::from_rgb(87, 242, 135))
            .footer(serenity::builder::CreateEmbedFooter::new(
                "Restored by Cuttlefish",
            ));

        let message = CreateMessage::new().embed(embed);

        channel_id
            .send_message(&ctx.http, message)
            .await
            .map_err(|e| DiscordError(format!("Failed to send unarchive message: {e}")))?;

        Ok(())
    }
}

/// Result of an archive or unarchive operation.
#[derive(Debug, Clone)]
pub struct ArchivalResult {
    /// Channel ID.
    pub channel_id: ChannelId,
    /// Original channel name (without archived- prefix).
    pub original_name: String,
    /// Archived channel name (with archived- prefix).
    pub archived_name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_archive_config_defaults() {
        let config = ArchiveConfig::default();
        assert_eq!(config.inactivity_days, 30);
        assert!(config.auto_archive);
        assert!(config.archive_category_name.is_none());
    }

    #[test]
    fn test_archive_config_builder() {
        let config = ArchiveConfig::new()
            .inactivity_days(14)
            .auto_archive(false)
            .archive_category_name("Old Projects");

        assert_eq!(config.inactivity_days, 14);
        assert!(!config.auto_archive);
        assert_eq!(
            config.archive_category_name,
            Some("Old Projects".to_string())
        );
    }

    #[test]
    fn test_inactivity_duration() {
        let config = ArchiveConfig::new().inactivity_days(7);
        let duration = config.inactivity_duration();
        assert_eq!(duration, Duration::from_secs(7 * 24 * 60 * 60));
    }

    #[test]
    fn test_is_archived() {
        assert!(ChannelArchival::is_archived("archived-project-myapp"));
        assert!(ChannelArchival::is_archived("archived-"));
        assert!(!ChannelArchival::is_archived("project-myapp"));
        assert!(!ChannelArchival::is_archived("archivedproject"));
    }

    #[test]
    fn test_get_original_name() {
        assert_eq!(
            ChannelArchival::get_original_name("archived-project-myapp"),
            Some("project-myapp")
        );
        assert_eq!(ChannelArchival::get_original_name("archived-"), Some(""));
        assert_eq!(ChannelArchival::get_original_name("project-myapp"), None);
    }

    #[test]
    fn test_archival_result() {
        let channel_id = ChannelId::new(123456789);
        let result = ArchivalResult {
            channel_id,
            original_name: "project-myapp".to_string(),
            archived_name: "archived-project-myapp".to_string(),
        };

        assert_eq!(result.channel_id, channel_id);
        assert_eq!(result.original_name, "project-myapp");
        assert_eq!(result.archived_name, "archived-project-myapp");
    }
}
