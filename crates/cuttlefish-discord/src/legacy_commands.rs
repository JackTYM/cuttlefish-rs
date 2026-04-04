//! Legacy Discord slash command definitions and parsing.
//!
//! **Deprecated**: Use the `commands` module instead for the new slash command framework.

use cuttlefish_core::error::DiscordError;

/// The set of slash commands registered by the bot.
pub mod slash {
    /// Create a new project.
    pub const PROJECT_CREATE: &str = "project_create";
    /// List all projects.
    pub const PROJECT_LIST: &str = "project_list";
    /// Get project status.
    pub const PROJECT_STATUS: &str = "project_status";
    /// Cancel a project.
    pub const PROJECT_CANCEL: &str = "project_cancel";
    /// Show help.
    pub const HELP: &str = "help";
}

/// A parsed slash command interaction.
#[derive(Debug, Clone)]
pub enum ParsedCommand {
    /// Create a new project with name and description.
    ProjectCreate {
        /// Project name.
        name: String,
        /// Project description.
        description: String,
    },
    /// List all active projects.
    ProjectList,
    /// Get status of a named project.
    ProjectStatus {
        /// Project name.
        name: String,
    },
    /// Cancel a named project.
    ProjectCancel {
        /// Project name.
        name: String,
    },
    /// Show help.
    Help,
}

/// Parse a serenity interaction into a ParsedCommand.
///
/// Returns None if the command name is unrecognized.
pub fn parse_command(
    command_name: &str,
    options: &[serenity::model::application::CommandDataOption],
) -> Result<ParsedCommand, DiscordError> {
    use serenity::model::application::CommandDataOptionValue;

    let get_str = |name: &str| -> Option<String> {
        options.iter().find(|o| o.name == name).and_then(|o| {
            if let CommandDataOptionValue::String(s) = &o.value {
                Some(s.clone())
            } else {
                None
            }
        })
    };

    match command_name {
        slash::PROJECT_CREATE => {
            let name = get_str("name")
                .ok_or_else(|| DiscordError("Missing 'name' parameter".to_string()))?;
            let description =
                get_str("description").unwrap_or_else(|| "No description".to_string());
            Ok(ParsedCommand::ProjectCreate { name, description })
        }
        slash::PROJECT_LIST => Ok(ParsedCommand::ProjectList),
        slash::PROJECT_STATUS => {
            let name = get_str("name")
                .ok_or_else(|| DiscordError("Missing 'name' parameter".to_string()))?;
            Ok(ParsedCommand::ProjectStatus { name })
        }
        slash::PROJECT_CANCEL => {
            let name = get_str("name")
                .ok_or_else(|| DiscordError("Missing 'name' parameter".to_string()))?;
            Ok(ParsedCommand::ProjectCancel { name })
        }
        slash::HELP => Ok(ParsedCommand::Help),
        unknown => Err(DiscordError(format!("Unknown command: {}", unknown))),
    }
}

/// Format a response for a help command.
pub fn help_text() -> &'static str {
    "🐙 **Cuttlefish** — AI Coding Assistant\n\
    \n\
    **Commands:**\n\
    `/project_create name:<name> description:<desc>` — Create a new coding project\n\
    `/project_list` — List all active projects\n\
    `/project_status name:<name>` — Check project status\n\
    `/project_cancel name:<name>` — Cancel a project\n\
    `/help` — Show this help message"
}

/// Format a message that is too long for Discord (2000 char limit).
///
/// Splits the message into chunks respecting the limit.
pub fn split_for_discord(text: &str, max_len: usize) -> Vec<String> {
    if text.len() <= max_len {
        return vec![text.to_string()];
    }
    text.chars()
        .collect::<Vec<_>>()
        .chunks(max_len)
        .map(|c| c.iter().collect())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_help_text_nonempty() {
        assert!(!help_text().is_empty());
        assert!(help_text().contains("Cuttlefish"));
    }

    #[test]
    fn test_split_short_message() {
        let parts = split_for_discord("Hello world", 2000);
        assert_eq!(parts.len(), 1);
        assert_eq!(parts[0], "Hello world");
    }

    #[test]
    fn test_split_long_message() {
        let long = "a".repeat(5000);
        let parts = split_for_discord(&long, 2000);
        assert_eq!(parts.len(), 3); // ceil(5000/2000)
        assert!(parts.iter().all(|p| p.len() <= 2000));
    }

    #[test]
    fn test_slash_constants_nonempty() {
        assert!(!slash::PROJECT_CREATE.is_empty());
        assert!(!slash::HELP.is_empty());
    }
}
