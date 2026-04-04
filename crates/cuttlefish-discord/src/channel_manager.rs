//! Discord channel management for project channels.

use cuttlefish_core::error::DiscordError;
use serenity::all::{ChannelId, ChannelType, Context, GuildId};
use serenity::builder::{CreateChannel, CreateEmbed, CreateMessage};
use serenity::model::Colour;

use crate::PROJECT_CATEGORY_NAME;

/// Manages the mapping between Discord channels and Cuttlefish projects.
pub struct ChannelManager;

impl ChannelManager {
    /// Generate the channel name for a project.
    ///
    /// Discord channel names must be lowercase with hyphens.
    pub fn project_channel_name(project_name: &str) -> String {
        let sanitized: String = project_name
            .to_lowercase()
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '-' })
            .collect();
        let cleaned: String = sanitized
            .split('-')
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("-");
        format!("project-{}", cleaned)
    }

    /// Validate a project name for use as a Discord channel.
    ///
    /// # Errors
    ///
    /// Returns an error if the name would result in an invalid channel.
    pub fn validate_project_name(name: &str) -> Result<(), DiscordError> {
        if name.is_empty() {
            return Err(DiscordError("Project name cannot be empty".to_string()));
        }
        if name.len() > 90 {
            return Err(DiscordError(
                "Project name too long (max 90 chars)".to_string(),
            ));
        }
        let channel_name = Self::project_channel_name(name);
        if channel_name.is_empty() || channel_name == "project-" {
            return Err(DiscordError(
                "Project name contains only invalid characters".to_string(),
            ));
        }
        Ok(())
    }

    /// Create a project channel in the specified guild.
    ///
    /// Creates a text channel under the Cuttlefish Projects category.
    /// If the category doesn't exist, it will be created.
    ///
    /// # Errors
    ///
    /// Returns an error if channel creation fails or a channel with the same name exists.
    pub async fn create_project_channel(
        ctx: &Context,
        guild_id: GuildId,
        project_name: &str,
        description: Option<&str>,
    ) -> Result<ChannelId, DiscordError> {
        Self::validate_project_name(project_name)?;

        let channel_name = Self::project_channel_name(project_name);

        let guild_channels = guild_id
            .channels(&ctx.http)
            .await
            .map_err(|e| DiscordError(format!("Failed to fetch guild channels: {e}")))?;

        let existing = guild_channels
            .values()
            .find(|c| c.name == channel_name);

        if existing.is_some() {
            return Err(DiscordError(format!(
                "Channel #{channel_name} already exists for this project"
            )));
        }

        let category_id = Self::get_or_create_category(ctx, guild_id).await?;

        let topic = description.unwrap_or("Cuttlefish project channel");

        let builder = CreateChannel::new(&channel_name)
            .kind(ChannelType::Text)
            .category(category_id)
            .topic(topic);

        let channel = guild_id
            .create_channel(&ctx.http, builder)
            .await
            .map_err(|e| DiscordError(format!("Failed to create channel: {e}")))?;

        Self::send_welcome_message(ctx, channel.id, project_name, description).await?;

        tracing::info!(
            channel_id = %channel.id,
            channel_name = %channel_name,
            guild_id = %guild_id,
            "Created project channel"
        );

        Ok(channel.id)
    }

    async fn get_or_create_category(
        ctx: &Context,
        guild_id: GuildId,
    ) -> Result<ChannelId, DiscordError> {
        let guild_channels = guild_id
            .channels(&ctx.http)
            .await
            .map_err(|e| DiscordError(format!("Failed to fetch guild channels: {e}")))?;

        let existing_category = guild_channels
            .values()
            .find(|c| c.kind == ChannelType::Category && c.name == PROJECT_CATEGORY_NAME);

        if let Some(category) = existing_category {
            return Ok(category.id);
        }

        let builder = CreateChannel::new(PROJECT_CATEGORY_NAME).kind(ChannelType::Category);

        let category = guild_id
            .create_channel(&ctx.http, builder)
            .await
            .map_err(|e| DiscordError(format!("Failed to create category: {e}")))?;

        tracing::info!(
            category_id = %category.id,
            guild_id = %guild_id,
            "Created Cuttlefish Projects category"
        );

        Ok(category.id)
    }

    async fn send_welcome_message(
        ctx: &Context,
        channel_id: ChannelId,
        project_name: &str,
        description: Option<&str>,
    ) -> Result<(), DiscordError> {
        let desc_text = description.unwrap_or("_No description provided_");

        let embed = CreateEmbed::new()
            .title(format!("🐙 Project: {project_name}"))
            .description(format!(
                "{desc_text}\n\n\
                **Quick Commands:**\n\
                • `/status` — View project status\n\
                • `/logs` — View recent activity\n\
                • `/approve` — Approve pending actions\n\
                • `/reject` — Reject with feedback"
            ))
            .colour(Colour::from_rgb(88, 101, 242))
            .footer(serenity::builder::CreateEmbedFooter::new(
                "Cuttlefish — AI Coding Assistant",
            ));

        let message = CreateMessage::new().embed(embed);

        channel_id
            .send_message(&ctx.http, message)
            .await
            .map_err(|e| DiscordError(format!("Failed to send welcome message: {e}")))?;

        Ok(())
    }
}

/// Represents a pending action that requires user approval.
#[derive(Debug, Clone)]
pub struct PendingAction {
    /// Unique identifier for this action.
    pub id: String,
    /// Human-readable description of the action.
    pub description: String,
    /// The type of action pending.
    pub action_type: PendingActionType,
    /// Channel where this action is pending.
    pub channel_id: ChannelId,
    /// Project associated with this action.
    pub project_name: String,
}

/// Types of actions that can be pending approval.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PendingActionType {
    /// Confirm a potentially destructive operation.
    ConfirmDestructive,
    /// Approve a code change.
    ApproveChange,
    /// Answer a question from the agent.
    AnswerQuestion,
    /// Review a diff before applying.
    ReviewDiff,
}

impl PendingAction {
    /// Get the pending action for a channel, if any.
    ///
    /// Returns a mock pending action for testing purposes.
    /// Real implementation will query the database in Wave 5.
    pub fn get_for_channel(channel_id: ChannelId) -> Option<Self> {
        let _ = channel_id;
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_channel_name_lowercase() {
        assert_eq!(
            ChannelManager::project_channel_name("MyApp"),
            "project-myapp"
        );
    }

    #[test]
    fn test_channel_name_spaces_to_hyphens() {
        assert_eq!(
            ChannelManager::project_channel_name("My App"),
            "project-my-app"
        );
    }

    #[test]
    fn test_channel_name_special_chars() {
        assert_eq!(
            ChannelManager::project_channel_name("my.app!"),
            "project-my-app"
        );
    }

    #[test]
    fn test_validate_empty_name() {
        assert!(ChannelManager::validate_project_name("").is_err());
    }

    #[test]
    fn test_validate_valid_name() {
        assert!(ChannelManager::validate_project_name("my-app").is_ok());
    }

    #[test]
    fn test_pending_action_type_equality() {
        assert_eq!(PendingActionType::ApproveChange, PendingActionType::ApproveChange);
        assert_ne!(PendingActionType::ApproveChange, PendingActionType::ReviewDiff);
    }

    #[test]
    fn test_get_pending_action_returns_none() {
        let channel_id = ChannelId::new(123456789);
        assert!(PendingAction::get_for_channel(channel_id).is_none());
    }
}
