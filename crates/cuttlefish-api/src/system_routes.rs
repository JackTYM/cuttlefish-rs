//! System configuration and status API routes.
//!
//! Provides endpoints for:
//! - `GET /api/system/config` — get system configuration
//! - `PUT /api/system/config` — update system configuration
//! - `GET /api/system/status` — get system status (version, uptime)
//! - `POST /api/system/providers/:id/test` — test provider connection
//! - `POST /api/system/api-key/regenerate` — regenerate API key

use std::sync::Arc;
use std::time::Instant;

use axum::{
    Router,
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
};
use cuttlefish_core::traits::provider::{CompletionRequest, Message, MessageRole};
use cuttlefish_providers::ProviderRegistry;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::middleware::AuthConfig;

/// Provider configuration for the system settings API.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderConfig {
    /// Provider identifier.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Provider type (e.g., "anthropic", "openai").
    #[serde(rename = "type")]
    pub provider_type: String,
    /// Whether the provider is enabled.
    pub enabled: bool,
    /// Masked API key for display.
    #[serde(default)]
    pub api_key: String,
    /// Default model for this provider.
    pub model: String,
    /// Available models.
    pub models: Vec<String>,
    /// Connection status from last test.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connected: Option<bool>,
}

/// Agent category configuration.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentCategoryConfig {
    /// Model category for this agent.
    pub category: String,
}

/// Agent settings for all agent types.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentSettings {
    /// Orchestrator agent configuration.
    pub orchestrator: AgentCategoryConfig,
    /// Coder agent configuration.
    pub coder: AgentCategoryConfig,
    /// Critic agent configuration.
    pub critic: AgentCategoryConfig,
    /// Planner agent configuration.
    pub planner: AgentCategoryConfig,
    /// Explorer agent configuration.
    pub explorer: AgentCategoryConfig,
    /// Librarian agent configuration.
    pub librarian: AgentCategoryConfig,
}

/// Sandbox resource configuration.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SandboxConfig {
    /// Memory limit in MB.
    pub memory_limit_mb: u32,
    /// CPU limit (number of cores).
    pub cpu_limit: f32,
    /// Disk limit in GB.
    pub disk_limit_gb: u32,
    /// Maximum concurrent sandboxes.
    pub max_concurrent: u32,
}

/// Notification settings.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationSettings {
    /// Notify on project creation.
    pub project_created: bool,
    /// Notify on project completion.
    pub project_completed: bool,
    /// Notify on agent errors.
    pub agent_errors: bool,
    /// Send weekly digest.
    pub weekly_digest: bool,
}

/// Full system configuration response.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemConfig {
    /// Masked API key.
    #[serde(default)]
    pub api_key: String,
    /// Configured providers.
    pub providers: Vec<ProviderConfig>,
    /// Agent settings.
    pub agent_settings: AgentSettings,
    /// Notification settings.
    pub notifications: NotificationSettings,
    /// Sandbox settings.
    pub sandbox: SandboxConfig,
    /// Whether tunnel is connected.
    pub tunnel_connected: bool,
    /// Server version.
    pub version: String,
}

impl Default for SystemConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            providers: vec![],
            agent_settings: AgentSettings {
                orchestrator: AgentCategoryConfig { category: "deep".into() },
                coder: AgentCategoryConfig { category: "deep".into() },
                critic: AgentCategoryConfig { category: "unspecified-high".into() },
                planner: AgentCategoryConfig { category: "ultrabrain".into() },
                explorer: AgentCategoryConfig { category: "quick".into() },
                librarian: AgentCategoryConfig { category: "quick".into() },
            },
            notifications: NotificationSettings {
                project_created: true,
                project_completed: true,
                agent_errors: true,
                weekly_digest: false,
            },
            sandbox: SandboxConfig {
                memory_limit_mb: 2048,
                cpu_limit: 2.0,
                disk_limit_gb: 10,
                max_concurrent: 5,
            },
            tunnel_connected: false,
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}

/// System status response.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemStatus {
    /// Server version.
    pub version: String,
    /// Server uptime in seconds.
    pub uptime_seconds: u64,
    /// Whether the server is connected (always true if responding).
    pub connected: bool,
}

/// Response containing a new or regenerated API key.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiKeyResponse {
    /// The (masked) API key.
    pub api_key: String,
}

/// Response from provider connection test.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderTestResponse {
    /// Whether the provider is connected and responding.
    pub connected: bool,
}

/// State for system routes.
#[derive(Clone)]
pub struct SystemState {
    /// System configuration.
    pub config: Arc<RwLock<SystemConfig>>,
    /// Server start time.
    pub start_time: Instant,
    /// Auth configuration.
    #[allow(dead_code)]
    pub auth_config: AuthConfig,
    /// Provider registry for testing connections.
    pub provider_registry: Option<Arc<ProviderRegistry>>,
}

impl SystemState {
    /// Create a new system state with auth config.
    pub fn new(auth_config: AuthConfig) -> Self {
        let mut config = SystemConfig::default();
        if let Some(ref key) = auth_config.legacy_api_key {
            config.api_key = mask_api_key(key);
        }
        Self {
            config: Arc::new(RwLock::new(config)),
            start_time: Instant::now(),
            auth_config,
            provider_registry: None,
        }
    }

    /// Add a provider registry for testing provider connections.
    pub fn with_provider_registry(mut self, registry: Arc<ProviderRegistry>) -> Self {
        self.provider_registry = Some(registry);
        self
    }
}

fn mask_api_key(key: &str) -> String {
    if key.len() < 8 {
        return key.to_string();
    }
    format!("{}****{}", &key[..3], &key[key.len()-4..])
}

fn generate_api_key() -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::rng();
    let key: String = (0..32)
        .map(|_| {
            let idx = rng.random_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();
    format!("sk-{key}")
}

/// Get the current system configuration.
pub async fn get_config(State(state): State<SystemState>) -> Json<SystemConfig> {
    let config = state.config.read().await;
    Json(config.clone())
}

/// Update the system configuration.
pub async fn update_config(
    State(state): State<SystemState>,
    Json(new_config): Json<SystemConfig>,
) -> (StatusCode, Json<SystemConfig>) {
    let mut config = state.config.write().await;

    config.providers = new_config.providers;
    config.agent_settings = new_config.agent_settings;
    config.notifications = new_config.notifications;
    config.sandbox = new_config.sandbox;

    tracing::info!("System configuration updated");
    (StatusCode::OK, Json(config.clone()))
}

/// Get the current system status.
pub async fn get_status(State(state): State<SystemState>) -> Json<SystemStatus> {
    Json(SystemStatus {
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: state.start_time.elapsed().as_secs(),
        connected: true,
    })
}

/// Test a provider connection by making a minimal API call.
pub async fn test_provider(
    State(state): State<SystemState>,
    Path(provider_id): Path<String>,
) -> Json<ProviderTestResponse> {
    tracing::info!(provider_id = %provider_id, "Testing provider connection");

    // Check if we have a provider registry
    let Some(ref registry) = state.provider_registry else {
        tracing::warn!("No provider registry configured");
        return Json(ProviderTestResponse { connected: false });
    };

    // Get the provider
    let Some(provider) = registry.get(&provider_id) else {
        tracing::warn!(provider_id = %provider_id, "Provider not found in registry");
        return Json(ProviderTestResponse { connected: false });
    };

    // Create a minimal test request
    let test_request = CompletionRequest {
        messages: vec![Message {
            role: MessageRole::User,
            content: "Say 'ok' if you can hear me.".to_string(),
        }],
        max_tokens: Some(10),
        temperature: Some(0.0),
        system: None,
    };

    // Try to complete the request
    match provider.complete(test_request).await {
        Ok(response) => {
            tracing::info!(
                provider_id = %provider_id,
                tokens_used = response.input_tokens + response.output_tokens,
                "Provider test successful"
            );
            Json(ProviderTestResponse { connected: true })
        }
        Err(e) => {
            tracing::warn!(
                provider_id = %provider_id,
                error = %e,
                "Provider test failed"
            );
            Json(ProviderTestResponse { connected: false })
        }
    }
}

/// Regenerate the API key.
pub async fn regenerate_api_key(State(state): State<SystemState>) -> Json<ApiKeyResponse> {
    let new_key = generate_api_key();
    let masked = mask_api_key(&new_key);
    
    let mut config = state.config.write().await;
    config.api_key = masked.clone();
    
    tracing::info!("API key regenerated");
    Json(ApiKeyResponse { api_key: masked })
}

/// Build the system configuration router.
pub fn system_router(state: SystemState) -> Router {
    Router::new()
        .route("/api/system/config", get(get_config).put(update_config))
        .route("/api/system/status", get(get_status))
        .route("/api/system/providers/{id}/test", post(test_provider))
        .route("/api/system/api-key/regenerate", post(regenerate_api_key))
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = SystemConfig::default();
        assert_eq!(config.agent_settings.orchestrator.category, "deep");
        assert_eq!(config.sandbox.memory_limit_mb, 2048);
    }

    #[test]
    fn test_mask_api_key() {
        assert_eq!(mask_api_key("sk-abcdefghijklmnop"), "sk-****mnop");
        assert_eq!(mask_api_key("short"), "short");
    }

    #[test]
    fn test_generate_api_key() {
        let key = generate_api_key();
        assert!(key.starts_with("sk-"));
        assert_eq!(key.len(), 35);
    }
}
