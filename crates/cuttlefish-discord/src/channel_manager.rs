//! Discord channel management for project channels.

use cuttlefish_core::error::DiscordError;

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
        // Remove consecutive hyphens and leading/trailing hyphens
        let cleaned: String = sanitized
            .split('-')
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("-");
        format!("project-{}", cleaned)
    }

    /// Validate a project name for use as a Discord channel.
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
}
