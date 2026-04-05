//! Discord bot event handler and startup logic.
//!
//! This module provides:
//! - `Handler` - Serenity event handler for Discord events
//! - `start_bot` - Start the Discord bot with the given token

use serenity::Client;
use serenity::all::{Context, EventHandler, GatewayIntents, GuildId, Interaction, Ready};
use serenity::async_trait;
use tracing::{error, info, warn};

use crate::commands;

/// Discord bot event handler.
///
/// Handles:
/// - Ready event (bot startup, command registration)
/// - Interaction events (slash commands)
pub struct Handler {
    /// Guild IDs to register commands in (empty = global commands).
    guild_ids: Vec<GuildId>,
}

impl Handler {
    /// Create a new event handler.
    pub fn new(guild_ids: Vec<u64>) -> Self {
        Self {
            guild_ids: guild_ids.into_iter().map(GuildId::new).collect(),
        }
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("Discord bot connected as {}", ready.user.tag());
        info!("Connected to {} guild(s)", ready.guilds.len());

        // Register commands
        if self.guild_ids.is_empty() {
            // Register global commands
            info!("Registering global slash commands...");
            // Note: Global commands can take up to an hour to propagate
            // For development, prefer guild commands
            for guild in &ready.guilds {
                if let Err(e) = commands::register_guild_commands(&ctx, guild.id).await {
                    error!("Failed to register commands in guild {}: {}", guild.id, e);
                }
            }
        } else {
            // Register to specific guilds only
            for guild_id in &self.guild_ids {
                info!("Registering slash commands for guild {}", guild_id);
                if let Err(e) = commands::register_guild_commands(&ctx, *guild_id).await {
                    error!("Failed to register commands in guild {}: {}", guild_id, e);
                }
            }
        }
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        match interaction {
            Interaction::Command(command) => {
                let command_name = command.data.name.clone();
                let user = &command.user;
                let guild_id = command.guild_id.map(|g| g.get()).unwrap_or(0);

                info!(
                    "Command '{}' from {} in guild {}",
                    command_name,
                    user.tag(),
                    guild_id
                );

                if let Err(e) = commands::route_command(&ctx, &command).await {
                    error!("Command '{}' failed: {}", command_name, e);
                }
            }
            Interaction::Component(_component) => {
                // Handle button/select menu interactions in the future
                warn!("Component interaction received but not yet implemented");
            }
            Interaction::Modal(_modal) => {
                // Handle modal submissions in the future
                warn!("Modal interaction received but not yet implemented");
            }
            _ => {
                warn!("Unknown interaction type received");
            }
        }
    }
}

/// Configuration for the Discord bot.
#[derive(Debug, Clone)]
pub struct BotConfig {
    /// Bot token.
    pub token: String,
    /// Guild IDs to restrict commands to (empty = all guilds).
    pub guild_ids: Vec<u64>,
}

impl BotConfig {
    /// Create a new bot configuration.
    pub fn new(token: impl Into<String>) -> Self {
        Self {
            token: token.into(),
            guild_ids: Vec::new(),
        }
    }

    /// Set the guild IDs to register commands in.
    pub fn with_guild_ids(mut self, ids: Vec<u64>) -> Self {
        self.guild_ids = ids;
        self
    }
}

/// Start the Discord bot.
///
/// This function blocks until the bot is stopped.
///
/// # Arguments
///
/// * `config` - Bot configuration
///
/// # Errors
///
/// Returns an error if the bot fails to start or connect.
pub async fn start_bot(config: BotConfig) -> Result<(), serenity::Error> {
    let intents = GatewayIntents::GUILDS
        | GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::DIRECT_MESSAGES;

    let handler = Handler::new(config.guild_ids);

    let mut client = Client::builder(&config.token, intents)
        .event_handler(handler)
        .await?;

    info!("Starting Discord bot...");
    client.start().await
}

/// Start the Discord bot in the background.
///
/// Returns a handle that can be used to abort the bot.
///
/// # Arguments
///
/// * `config` - Bot configuration
pub fn start_bot_background(
    config: BotConfig,
) -> tokio::task::JoinHandle<Result<(), serenity::Error>> {
    tokio::spawn(async move { start_bot(config).await })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bot_config_new() {
        let config = BotConfig::new("test-token");
        assert_eq!(config.token, "test-token");
        assert!(config.guild_ids.is_empty());
    }

    #[test]
    fn test_bot_config_with_guild_ids() {
        let config = BotConfig::new("test-token").with_guild_ids(vec![123, 456]);
        assert_eq!(config.guild_ids, vec![123, 456]);
    }

    #[test]
    fn test_handler_new() {
        let handler = Handler::new(vec![123, 456]);
        assert_eq!(handler.guild_ids.len(), 2);
    }

    #[test]
    fn test_handler_empty_guild_ids() {
        let handler = Handler::new(vec![]);
        assert!(handler.guild_ids.is_empty());
    }
}
