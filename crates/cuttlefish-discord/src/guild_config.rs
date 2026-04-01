//! Per-guild configuration for multi-server Discord support.

use std::collections::HashMap;

/// Configuration for a specific Discord guild (server).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GuildConfig {
    /// Whether the bot is enabled in this guild.
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    /// Maximum concurrent projects in this guild.
    #[serde(default = "default_max_projects")]
    pub max_projects: usize,
}

fn default_enabled() -> bool {
    true
}

fn default_max_projects() -> usize {
    10
}

impl Default for GuildConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_projects: 10,
        }
    }
}

/// In-memory store of guild configurations.
pub struct GuildConfigStore {
    configs: HashMap<u64, GuildConfig>,
}

impl GuildConfigStore {
    /// Create a new empty config store.
    pub fn new() -> Self {
        Self {
            configs: HashMap::new(),
        }
    }

    /// Get config for a guild, or return defaults.
    pub fn get(&self, guild_id: u64) -> GuildConfig {
        self.configs.get(&guild_id).cloned().unwrap_or_default()
    }

    /// Set config for a guild.
    pub fn set(&mut self, guild_id: u64, config: GuildConfig) {
        self.configs.insert(guild_id, config);
    }

    /// Check if the bot is enabled for a guild.
    pub fn is_enabled(&self, guild_id: u64) -> bool {
        self.get(guild_id).enabled
    }

    /// Get the number of guilds with configuration.
    pub fn guild_count(&self) -> usize {
        self.configs.len()
    }
}

impl Default for GuildConfigStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_enabled() {
        let store = GuildConfigStore::new();
        assert!(store.is_enabled(12345));
    }

    #[test]
    fn test_set_and_get_guild_config() {
        let mut store = GuildConfigStore::new();
        store.set(
            42,
            GuildConfig {
                enabled: false,
                max_projects: 5,
            },
        );
        let cfg = store.get(42);
        assert!(!cfg.enabled);
        assert_eq!(cfg.max_projects, 5);
    }

    #[test]
    fn test_guild_count() {
        let mut store = GuildConfigStore::new();
        assert_eq!(store.guild_count(), 0);
        store.set(1, GuildConfig::default());
        store.set(2, GuildConfig::default());
        assert_eq!(store.guild_count(), 2);
    }
}
