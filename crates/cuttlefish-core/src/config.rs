//! Configuration types for the Cuttlefish platform.

use crate::error::ConfigError;
use crate::routing::RoutingConfig;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

/// Main configuration for Cuttlefish.
#[derive(Debug, Deserialize)]
pub struct CuttlefishConfig {
    /// Server configuration.
    pub server: ServerConfig,
    /// Database configuration.
    pub database: DatabaseConfig,
    /// Provider configurations.
    #[serde(default)]
    pub providers: HashMap<String, ProviderConfig>,
    /// Agent configurations.
    #[serde(default)]
    pub agents: HashMap<String, AgentConfig>,
    /// Discord configuration (optional).
    pub discord: Option<DiscordConfig>,
    /// Sandbox configuration.
    #[serde(default)]
    pub sandbox: SandboxConfig,
    /// Model routing configuration.
    #[serde(default)]
    pub routing: RoutingConfig,
}

/// Server configuration.
#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    /// Host to bind to.
    #[serde(default = "default_host")]
    pub host: String,
    /// Port to bind to.
    #[serde(default = "default_port")]
    pub port: u16,
    /// API key (loaded from CUTTLEFISH_API_KEY env var, not stored in config).
    #[serde(skip)]
    pub api_key: Option<String>,
}

/// Database configuration.
#[derive(Debug, Deserialize)]
pub struct DatabaseConfig {
    /// Path to the database file.
    #[serde(default = "default_db_path")]
    pub path: PathBuf,
}

/// Provider configuration.
#[derive(Debug, Deserialize)]
pub struct ProviderConfig {
    /// Provider type (e.g., "openai", "anthropic").
    pub provider_type: String,
    /// Model name.
    pub model: String,
    /// Region (optional).
    pub region: Option<String>,
}

/// Agent configuration.
#[derive(Debug, Deserialize)]
pub struct AgentConfig {
    /// Agent category.
    pub category: String,
    /// Model override (optional).
    pub model_override: Option<String>,
}

/// Discord configuration.
#[derive(Debug, Deserialize)]
pub struct DiscordConfig {
    /// Environment variable name for Discord bot token.
    #[serde(default = "default_discord_token_env")]
    pub token_env_var: String,
    /// Guild IDs to register commands in.
    #[serde(default)]
    pub guild_ids: Vec<u64>,
}

/// Sandbox configuration.
#[derive(Debug, Deserialize)]
pub struct SandboxConfig {
    /// Docker socket path.
    #[serde(default = "default_docker_socket")]
    pub docker_socket: String,
    /// Memory limit in MB.
    #[serde(default = "default_memory_limit")]
    pub memory_limit_mb: u64,
    /// CPU limit.
    #[serde(default = "default_cpu_limit")]
    pub cpu_limit: f64,
    /// Disk limit in GB.
    #[serde(default = "default_disk_limit")]
    pub disk_limit_gb: u64,
    /// Maximum concurrent sandboxes.
    #[serde(default = "default_max_concurrent")]
    pub max_concurrent: usize,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            docker_socket: default_docker_socket(),
            memory_limit_mb: default_memory_limit(),
            cpu_limit: default_cpu_limit(),
            disk_limit_gb: default_disk_limit(),
            max_concurrent: default_max_concurrent(),
        }
    }
}

fn default_host() -> String {
    "127.0.0.1".to_string()
}

fn default_port() -> u16 {
    8080
}

fn default_db_path() -> PathBuf {
    PathBuf::from("cuttlefish.db")
}

fn default_docker_socket() -> String {
    "unix:///var/run/docker.sock".to_string()
}

fn default_memory_limit() -> u64 {
    2048
}

fn default_cpu_limit() -> f64 {
    2.0
}

fn default_disk_limit() -> u64 {
    10
}

fn default_max_concurrent() -> usize {
    5
}

fn default_discord_token_env() -> String {
    "DISCORD_BOT_TOKEN".to_string()
}

impl CuttlefishConfig {
    /// Load configuration from a file.
    pub fn load_from_file(path: &std::path::Path) -> Result<Self, ConfigError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| ConfigError(format!("Failed to read config file: {}", e)))?;
        let mut config: Self = toml::from_str(&content)
            .map_err(|e| ConfigError(format!("Failed to parse TOML: {}", e)))?;

        config.server.api_key = std::env::var("CUTTLEFISH_API_KEY").ok();

        Ok(config)
    }

    /// Load configuration from default locations.
    ///
    /// Tries in order:
    /// 1. `./cuttlefish.toml`
    /// 2. `/etc/cuttlefish/cuttlefish.toml`
    /// 3. `~/.config/cuttlefish/config.toml`
    pub fn load() -> Result<Self, ConfigError> {
        let local_path = PathBuf::from("cuttlefish.toml");
        if local_path.exists() {
            return Self::load_from_file(&local_path);
        }

        let system_path = PathBuf::from("/etc/cuttlefish/cuttlefish.toml");
        if system_path.exists() {
            return Self::load_from_file(&system_path);
        }

        if let Ok(home) = std::env::var("HOME") {
            let config_path = PathBuf::from(home)
                .join(".config")
                .join("cuttlefish")
                .join("config.toml");
            if config_path.exists() {
                return Self::load_from_file(&config_path);
            }
        }

        Err(ConfigError(
            "No configuration file found at ./cuttlefish.toml, /etc/cuttlefish/cuttlefish.toml, or ~/.config/cuttlefish/config.toml"
                .to_string(),
        ))
    }

    /// Load from a specific file path
    pub fn load_from_path(path: &std::path::Path) -> Result<Self, ConfigError> {
        Self::load_from_file(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_load_from_file_with_all_sections() {
        let toml_content = r#"
[server]
host = "0.0.0.0"
port = 9000

[database]
path = "test.db"

[sandbox]
docker_socket = "unix:///custom/docker.sock"
memory_limit_mb = 4096
cpu_limit = 4.0
disk_limit_gb = 20
max_concurrent = 10

[providers.openai]
provider_type = "openai"
model = "gpt-4"
region = "us-east-1"

[agents.default]
category = "general"
model_override = "gpt-4-turbo"

[discord]
token_env_var = "DISCORD_TOKEN"
guild_ids = [123456789, 987654321]
"#;

        let mut temp_file = tempfile::NamedTempFile::new().expect("Failed to create temp file");
        temp_file
            .write_all(toml_content.as_bytes())
            .expect("Failed to write to temp file");

        let config =
            CuttlefishConfig::load_from_file(temp_file.path()).expect("Failed to load config");

        assert_eq!(config.server.host, "0.0.0.0");
        assert_eq!(config.server.port, 9000);
        assert_eq!(config.database.path, PathBuf::from("test.db"));
        assert_eq!(config.sandbox.memory_limit_mb, 4096);
        assert_eq!(config.sandbox.cpu_limit, 4.0);
        assert_eq!(config.sandbox.disk_limit_gb, 20);
        assert_eq!(config.sandbox.max_concurrent, 10);
        assert!(config.providers.contains_key("openai"));
        assert!(config.agents.contains_key("default"));
        assert!(config.discord.is_some());
    }

    #[test]
    fn test_default_values() {
        let toml_content = r#"
[server]

[database]
"#;

        let mut temp_file = tempfile::NamedTempFile::new().expect("Failed to create temp file");
        temp_file
            .write_all(toml_content.as_bytes())
            .expect("Failed to write to temp file");

        let config =
            CuttlefishConfig::load_from_file(temp_file.path()).expect("Failed to load config");

        assert_eq!(config.server.host, "127.0.0.1");
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.database.path, PathBuf::from("cuttlefish.db"));
        assert_eq!(config.sandbox.docker_socket, "unix:///var/run/docker.sock");
        assert_eq!(config.sandbox.memory_limit_mb, 2048);
        assert_eq!(config.sandbox.cpu_limit, 2.0);
        assert_eq!(config.sandbox.disk_limit_gb, 10);
        assert_eq!(config.sandbox.max_concurrent, 5);
    }

    #[test]
    fn test_invalid_toml() {
        let toml_content = "invalid toml [[[";

        let mut temp_file = tempfile::NamedTempFile::new().expect("Failed to create temp file");
        temp_file
            .write_all(toml_content.as_bytes())
            .expect("Failed to write to temp file");

        let result = CuttlefishConfig::load_from_file(temp_file.path());
        assert!(result.is_err());
    }
}
