//! Discord slash command framework for Cuttlefish.
//!
//! This module provides:
//! - Command registration using serenity's `CreateCommand` builder
//! - Command routing to dispatch interactions to handlers
//! - Individual command handlers for project management

use cuttlefish_core::error::DiscordError;
use serenity::all::{CommandInteraction, Context, CreateInteractionResponse};
use serenity::builder::CreateInteractionResponseMessage;
use serenity::model::id::GuildId;

pub mod approve;
pub mod logs;
pub mod new_project;
pub mod reject;
pub mod status;

/// Command names as constants for consistency.
pub mod names {
    /// Create a new project.
    pub const NEW_PROJECT: &str = "new-project";
    /// Get project status.
    pub const STATUS: &str = "status";
    /// Show activity logs.
    pub const LOGS: &str = "logs";
    /// Approve pending action.
    pub const APPROVE: &str = "approve";
    /// Reject pending action.
    pub const REJECT: &str = "reject";
}

/// Register all slash commands with a guild.
///
/// Uses guild commands for faster updates during development.
/// For production, consider using global commands instead.
///
/// # Errors
///
/// Returns an error if command registration fails.
pub async fn register_guild_commands(
    ctx: &Context,
    guild_id: GuildId,
) -> Result<(), DiscordError> {
    let commands = vec![
        new_project::register(),
        status::register(),
        logs::register(),
        approve::register(),
        reject::register(),
    ];

    let command_count = commands.len();

    guild_id
        .set_commands(&ctx.http, commands)
        .await
        .map_err(|e| DiscordError(format!("Failed to register commands: {e}")))?;

    tracing::info!("Registered {} slash commands for guild {}", command_count, guild_id);
    Ok(())
}

/// Route an incoming command interaction to the appropriate handler.
///
/// This function dispatches based on command name and handles:
/// - Deferring replies for long operations
/// - Error responses for unknown commands
/// - Delegation to specific command handlers
///
/// # Errors
///
/// Returns an error if the command handler fails or if responding fails.
pub async fn route_command(ctx: &Context, command: &CommandInteraction) -> Result<(), DiscordError> {
    let command_name = command.data.name.as_str();

    match command_name {
        names::NEW_PROJECT => new_project::run(ctx, command).await,
        names::STATUS => status::run(ctx, command).await,
        names::LOGS => logs::run(ctx, command).await,
        names::APPROVE => approve::run(ctx, command).await,
        names::REJECT => reject::run(ctx, command).await,
        unknown => {
            let response = CreateInteractionResponseMessage::new()
                .content(format!("Unknown command: `{unknown}`"))
                .ephemeral(true);
            command
                .create_response(&ctx.http, CreateInteractionResponse::Message(response))
                .await
                .map_err(|e| DiscordError(format!("Failed to respond: {e}")))?;
            Err(DiscordError(format!("Unknown command: {unknown}")))
        }
    }
}

/// Helper to send an ephemeral error response.
pub async fn send_error_response(
    ctx: &Context,
    command: &CommandInteraction,
    message: &str,
) -> Result<(), DiscordError> {
    let response = CreateInteractionResponseMessage::new()
        .content(format!("❌ {message}"))
        .ephemeral(true);
    command
        .create_response(&ctx.http, CreateInteractionResponse::Message(response))
        .await
        .map_err(|e| DiscordError(format!("Failed to send error response: {e}")))
}

/// Helper to defer a reply for long-running operations.
///
/// Use this when the command handler needs more than 3 seconds to respond.
pub async fn defer_reply(
    ctx: &Context,
    command: &CommandInteraction,
    ephemeral: bool,
) -> Result<(), DiscordError> {
    let response = if ephemeral {
        CreateInteractionResponse::Defer(
            CreateInteractionResponseMessage::new().ephemeral(true),
        )
    } else {
        CreateInteractionResponse::Defer(CreateInteractionResponseMessage::new())
    };

    command
        .create_response(&ctx.http, response)
        .await
        .map_err(|e| DiscordError(format!("Failed to defer reply: {e}")))
}

/// Extract a string option from command options.
pub fn get_string_option(command: &CommandInteraction, name: &str) -> Option<String> {
    use serenity::model::application::ResolvedValue;

    command.data.options().iter().find_map(|opt| {
        if opt.name == name && let ResolvedValue::String(s) = opt.value {
            return Some(s.to_string());
        }
        None
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_names_not_empty() {
        assert!(!names::NEW_PROJECT.is_empty());
        assert!(!names::STATUS.is_empty());
        assert!(!names::LOGS.is_empty());
        assert!(!names::APPROVE.is_empty());
        assert!(!names::REJECT.is_empty());
    }

    #[test]
    fn test_command_names_lowercase_with_hyphens() {
        assert_eq!(names::NEW_PROJECT, names::NEW_PROJECT.to_lowercase());
        assert_eq!(names::STATUS, names::STATUS.to_lowercase());
        assert_eq!(names::LOGS, names::LOGS.to_lowercase());
        assert_eq!(names::APPROVE, names::APPROVE.to_lowercase());
        assert_eq!(names::REJECT, names::REJECT.to_lowercase());
    }
}
