//! Configuration types for the Cuttlefish platform.

use crate::error::ConfigError;
use crate::routing::RoutingConfig;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Main configuration for Cuttlefish.
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    /// WebUI configuration (optional).
    pub webui: Option<WebUiConfigToml>,
    /// Auto-update configuration (optional).
    #[serde(default)]
    pub auto_update: AutoUpdateConfigToml,
}

/// Auto-update configuration from TOML.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoUpdateConfigToml {
    /// Whether auto-update is enabled.
    #[serde(default)]
    pub enabled: bool,
    /// Poll interval in seconds (default: 3600 = 1 hour).
    #[serde(default = "default_poll_interval")]
    pub poll_interval_secs: u64,
    /// Whether to automatically apply updates.
    #[serde(default = "default_auto_apply")]
    pub auto_apply: bool,
    /// Directory to store downloaded updates.
    /// For systemd services, use /var/cache/cuttlefish.
    pub download_dir: Option<PathBuf>,
}

impl Default for AutoUpdateConfigToml {
    fn default() -> Self {
        Self {
            enabled: false,
            poll_interval_secs: default_poll_interval(),
            auto_apply: default_auto_apply(),
            download_dir: None,
        }
    }
}

fn default_poll_interval() -> u64 {
    3600
}

fn default_auto_apply() -> bool {
    true
}

/// Server configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// Path to the database file.
    #[serde(default = "default_db_path")]
    pub path: PathBuf,
}

/// Provider configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    /// Provider type (e.g., "openai", "anthropic", "bedrock", "google", "ollama").
    pub provider_type: String,
    /// Model name (optional, defaults vary by provider).
    pub model: Option<String>,
    /// Region (for Bedrock, optional).
    pub region: Option<String>,
    /// Base URL (for Ollama, optional).
    pub base_url: Option<String>,
}

/// Agent configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Agent category.
    pub category: String,
    /// Model override (optional).
    pub model_override: Option<String>,
}

/// Discord configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordConfig {
    /// Environment variable name for Discord bot token.
    #[serde(default = "default_discord_token_env")]
    pub token_env_var: String,
    /// Guild IDs to register commands in.
    #[serde(default)]
    pub guild_ids: Vec<u64>,
}

/// Sandbox configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// WebUI configuration from TOML.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebUiConfigToml {
    /// Whether WebUI serving is enabled.
    #[serde(default = "default_webui_enabled")]
    pub enabled: bool,
    /// Path to the directory containing static files.
    #[serde(default = "default_webui_static_dir")]
    pub static_dir: PathBuf,
}

fn default_webui_enabled() -> bool {
    true
}

fn default_webui_static_dir() -> PathBuf {
    PathBuf::from("/opt/cuttlefish/webui")
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

    /// Get the default config file path (first existing or default location).
    pub fn default_path() -> PathBuf {
        let local_path = PathBuf::from("cuttlefish.toml");
        if local_path.exists() {
            return local_path;
        }

        let system_path = PathBuf::from("/etc/cuttlefish/cuttlefish.toml");
        if system_path.exists() {
            return system_path;
        }

        if let Ok(home) = std::env::var("HOME") {
            let config_path = PathBuf::from(home)
                .join(".config")
                .join("cuttlefish")
                .join("config.toml");
            if config_path.exists() {
                return config_path;
            }
        }

        // Default to system path if nothing exists
        system_path
    }

    /// Save configuration to a file.
    pub fn save_to_file(&self, path: &std::path::Path) -> Result<(), ConfigError> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| ConfigError(format!("Failed to serialize config: {}", e)))?;
        std::fs::write(path, content)
            .map_err(|e| ConfigError(format!("Failed to write config file: {}", e)))?;
        Ok(())
    }

    /// Update a provider's model in the config.
    pub fn set_provider_model(&mut self, provider_name: &str, model: &str) {
        if let Some(provider) = self.providers.get_mut(provider_name) {
            provider.model = Some(model.to_string());
        }
    }

    /// Create configuration entirely from environment variables.
    ///
    /// This allows running without a config file for development/testing.
    ///
    /// # Environment Variables
    ///
    /// ## Server
    /// - `CUTTLEFISH_HOST` - Server host (default: 127.0.0.1)
    /// - `CUTTLEFISH_PORT` - Server port (default: 8080)
    /// - `CUTTLEFISH_API_KEY` - API key for authentication
    ///
    /// ## Database
    /// - `CUTTLEFISH_DATABASE_PATH` - Database file path (default: ./cuttlefish.db)
    ///
    /// ## Providers (can define multiple with numeric suffix)
    /// - `CUTTLEFISH_PROVIDER_<NAME>_TYPE` - Provider type (anthropic, openai, bedrock, google, ollama)
    /// - `CUTTLEFISH_PROVIDER_<NAME>_MODEL` - Model name
    /// - `CUTTLEFISH_PROVIDER_<NAME>_REGION` - Region (for bedrock)
    /// - `CUTTLEFISH_PROVIDER_<NAME>_BASE_URL` - Base URL (for ollama)
    ///
    /// ## Routing
    /// - `CUTTLEFISH_ROUTE_DEEP` - Provider for deep tasks
    /// - `CUTTLEFISH_ROUTE_QUICK` - Provider for quick tasks
    /// - `CUTTLEFISH_ROUTE_ULTRABRAIN` - Provider for ultrabrain tasks
    /// - `CUTTLEFISH_ROUTE_VISUAL` - Provider for visual tasks
    /// - `CUTTLEFISH_ROUTE_UNSPECIFIED_HIGH` - Provider for unspecified-high tasks
    /// - `CUTTLEFISH_ROUTE_UNSPECIFIED_LOW` - Provider for unspecified-low tasks
    ///
    /// ## WebUI
    /// - `CUTTLEFISH_WEBUI_ENABLED` - Enable WebUI (default: true)
    /// - `CUTTLEFISH_WEBUI_DIR` - WebUI static files directory
    pub fn from_env() -> Result<Self, ConfigError> {
        use std::env;

        // Server config
        let server = ServerConfig {
            host: env::var("CUTTLEFISH_HOST").unwrap_or_else(|_| default_host()),
            port: env::var("CUTTLEFISH_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or_else(default_port),
            api_key: env::var("CUTTLEFISH_API_KEY").ok(),
        };

        // Database config
        let database = DatabaseConfig {
            path: env::var("CUTTLEFISH_DATABASE_PATH")
                .map(PathBuf::from)
                .unwrap_or_else(|_| default_db_path()),
        };

        // Parse providers from env vars
        // Format: CUTTLEFISH_PROVIDER_<NAME>_TYPE, CUTTLEFISH_PROVIDER_<NAME>_MODEL, etc.
        let mut providers = HashMap::new();
        let env_vars: Vec<(String, String)> = env::vars().collect();

        // Find all unique provider names
        let mut provider_names: std::collections::HashSet<String> =
            std::collections::HashSet::new();
        for (key, _) in &env_vars {
            if let Some(rest) = key.strip_prefix("CUTTLEFISH_PROVIDER_")
                && let Some(name_end) = rest.find('_')
            {
                let name = rest[..name_end].to_lowercase();
                provider_names.insert(name);
            }
        }

        for name in provider_names {
            let prefix = format!("CUTTLEFISH_PROVIDER_{}_", name.to_uppercase());
            let provider_type = env::var(format!("{prefix}TYPE")).ok();

            if let Some(ptype) = provider_type {
                let config = ProviderConfig {
                    provider_type: ptype,
                    model: env::var(format!("{prefix}MODEL")).ok(),
                    region: env::var(format!("{prefix}REGION")).ok(),
                    base_url: env::var(format!("{prefix}BASE_URL")).ok(),
                };
                providers.insert(name, config);
            }
        }

        // Routing config - parse CUTTLEFISH_ROUTE_<CATEGORY>=<provider>:<model>
        let mut categories = HashMap::new();
        let route_categories = [
            ("deep", "CUTTLEFISH_ROUTE_DEEP"),
            ("quick", "CUTTLEFISH_ROUTE_QUICK"),
            ("ultrabrain", "CUTTLEFISH_ROUTE_ULTRABRAIN"),
            ("visual", "CUTTLEFISH_ROUTE_VISUAL"),
            ("unspecified-high", "CUTTLEFISH_ROUTE_UNSPECIFIED_HIGH"),
            ("unspecified-low", "CUTTLEFISH_ROUTE_UNSPECIFIED_LOW"),
        ];

        for (category, env_key) in route_categories {
            if let Ok(value) = env::var(env_key) {
                // Format: "provider:model" or just "provider" (uses provider's default model)
                let parts: Vec<&str> = value.splitn(2, ':').collect();
                let provider = parts[0].to_string();
                let model = parts.get(1).map(|s| s.to_string()).unwrap_or_default();
                categories.insert(
                    category.to_string(),
                    crate::routing::RouteConfig::new(provider, model),
                );
            }
        }

        let routing = RoutingConfig {
            categories,
            agents: HashMap::new(),
        };

        // WebUI config
        let webui = if env::var("CUTTLEFISH_WEBUI_ENABLED")
            .map(|v| v != "false" && v != "0")
            .unwrap_or(true)
        {
            Some(WebUiConfigToml {
                enabled: true,
                static_dir: env::var("CUTTLEFISH_WEBUI_DIR")
                    .map(PathBuf::from)
                    .unwrap_or_else(|_| default_webui_static_dir()),
            })
        } else {
            Some(WebUiConfigToml {
                enabled: false,
                static_dir: default_webui_static_dir(),
            })
        };

        Ok(Self {
            server,
            database,
            providers,
            agents: HashMap::new(),
            discord: None,
            sandbox: SandboxConfig::default(),
            routing,
            webui,
            auto_update: AutoUpdateConfigToml::default(),
        })
    }

    /// Load configuration, trying file first then falling back to env vars.
    pub fn load_or_env() -> Result<Self, ConfigError> {
        match Self::load() {
            Ok(config) => Ok(config),
            Err(_) => Self::from_env(),
        }
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
