//! Category-based model routing for the Cuttlefish platform.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for a single model route.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RouteConfig {
    /// Provider name in the registry (e.g., "anthropic", "openai").
    pub provider: String,
    /// Model ID to use (e.g., "claude-sonnet-4-6").
    pub model: String,
    /// Fallback route if the primary fails.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fallback: Option<Box<RouteConfig>>,
}

impl RouteConfig {
    /// Create a new route configuration.
    pub fn new(provider: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            provider: provider.into(),
            model: model.into(),
            fallback: None,
        }
    }

    /// Create a route with a fallback.
    pub fn with_fallback(mut self, fallback: RouteConfig) -> Self {
        self.fallback = Some(Box::new(fallback));
        self
    }
}

/// Maps categories and agent roles to model routes.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RoutingConfig {
    /// Category-based routes (e.g., "deep" -> RouteConfig).
    #[serde(default)]
    pub categories: HashMap<String, RouteConfig>,
    /// Agent-specific routes that override category defaults.
    #[serde(default)]
    pub agents: HashMap<String, RouteConfig>,
}

impl Default for RoutingConfig {
    fn default() -> Self {
        let mut categories = HashMap::new();

        categories.insert(
            "deep".to_string(),
            RouteConfig::new("anthropic", "claude-sonnet-4-6"),
        );
        categories.insert(
            "quick".to_string(),
            RouteConfig::new("anthropic", "claude-haiku-4-5"),
        );
        categories.insert(
            "ultrabrain".to_string(),
            RouteConfig::new("anthropic", "claude-opus-4-6"),
        );
        categories.insert(
            "visual".to_string(),
            RouteConfig::new("google", "gemini-2.0-flash"),
        );
        categories.insert(
            "unspecified-high".to_string(),
            RouteConfig::new("anthropic", "claude-sonnet-4-6"),
        );
        categories.insert(
            "unspecified-low".to_string(),
            RouteConfig::new("anthropic", "claude-haiku-4-5"),
        );

        Self {
            categories,
            agents: HashMap::new(),
        }
    }
}

/// Resolves categories or agent roles to route configurations.
pub struct ModelRouter {
    config: RoutingConfig,
}

impl ModelRouter {
    /// Create a new model router with the given configuration.
    pub fn new(config: RoutingConfig) -> Self {
        Self { config }
    }

    /// Get the route for a category.
    pub fn route_category(&self, category: &str) -> Option<&RouteConfig> {
        self.config.categories.get(category)
    }

    /// Get the route for an agent role (overrides category if present).
    pub fn route_agent(&self, agent_role: &str) -> Option<&RouteConfig> {
        self.config.agents.get(agent_role)
    }

    /// Resolve a route for an agent, falling back to category if no agent-specific route.
    pub fn resolve(&self, agent_role: &str, category: &str) -> Option<&RouteConfig> {
        self.route_agent(agent_role)
            .or_else(|| self.route_category(category))
    }

    /// Get the underlying routing configuration.
    pub fn config(&self) -> &RoutingConfig {
        &self.config
    }
}

impl Default for ModelRouter {
    fn default() -> Self {
        Self::new(RoutingConfig::default())
    }
}

impl std::fmt::Debug for ModelRouter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ModelRouter")
            .field("config", &self.config)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_route_config_new() {
        let route = RouteConfig::new("anthropic", "claude-sonnet-4-6");
        assert_eq!(route.provider, "anthropic");
        assert_eq!(route.model, "claude-sonnet-4-6");
        assert!(route.fallback.is_none());
    }

    #[test]
    fn test_route_config_with_fallback() {
        let fallback = RouteConfig::new("openai", "gpt-4");
        let route = RouteConfig::new("anthropic", "claude-sonnet-4-6").with_fallback(fallback);

        assert!(route.fallback.is_some());
        let fb = route.fallback.as_ref().expect("fallback should exist");
        assert_eq!(fb.provider, "openai");
        assert_eq!(fb.model, "gpt-4");
    }

    #[test]
    fn test_routing_config_default() {
        let config = RoutingConfig::default();

        assert!(config.categories.contains_key("deep"));
        assert!(config.categories.contains_key("quick"));
        assert!(config.categories.contains_key("ultrabrain"));
        assert!(config.categories.contains_key("visual"));
        assert!(config.categories.contains_key("unspecified-high"));
        assert!(config.categories.contains_key("unspecified-low"));

        let deep = config.categories.get("deep").expect("deep should exist");
        assert_eq!(deep.provider, "anthropic");
        assert_eq!(deep.model, "claude-sonnet-4-6");
    }

    #[test]
    fn test_model_router_route_category() {
        let router = ModelRouter::default();

        let route = router.route_category("deep");
        assert!(route.is_some());
        let r = route.expect("route should exist");
        assert_eq!(r.provider, "anthropic");
        assert_eq!(r.model, "claude-sonnet-4-6");
    }

    #[test]
    fn test_model_router_route_category_missing() {
        let router = ModelRouter::default();
        assert!(router.route_category("nonexistent").is_none());
    }

    #[test]
    fn test_model_router_route_agent() {
        let mut config = RoutingConfig::default();
        config.agents.insert(
            "coder".to_string(),
            RouteConfig::new("anthropic", "claude-opus-4-6"),
        );

        let router = ModelRouter::new(config);

        let route = router.route_agent("coder");
        assert!(route.is_some());
        let r = route.expect("route should exist");
        assert_eq!(r.model, "claude-opus-4-6");
    }

    #[test]
    fn test_model_router_route_agent_missing() {
        let router = ModelRouter::default();
        assert!(router.route_agent("nonexistent").is_none());
    }

    #[test]
    fn test_model_router_resolve_prefers_agent() {
        let mut config = RoutingConfig::default();
        config.agents.insert(
            "coder".to_string(),
            RouteConfig::new("anthropic", "claude-opus-4-6"),
        );

        let router = ModelRouter::new(config);

        let route = router.resolve("coder", "deep");
        assert!(route.is_some());
        let r = route.expect("route should exist");
        assert_eq!(r.model, "claude-opus-4-6");
    }

    #[test]
    fn test_model_router_resolve_falls_back_to_category() {
        let router = ModelRouter::default();

        let route = router.resolve("unknown_agent", "deep");
        assert!(route.is_some());
        let r = route.expect("route should exist");
        assert_eq!(r.model, "claude-sonnet-4-6");
    }

    #[test]
    fn test_model_router_resolve_returns_none_when_both_missing() {
        let router = ModelRouter::new(RoutingConfig {
            categories: HashMap::new(),
            agents: HashMap::new(),
        });

        assert!(router.resolve("unknown", "unknown").is_none());
    }

    #[test]
    fn test_routing_config_serialization() {
        let config = RoutingConfig::default();
        let json = serde_json::to_string(&config).expect("serialization should succeed");
        let deserialized: RoutingConfig =
            serde_json::from_str(&json).expect("deserialization should succeed");
        assert_eq!(config, deserialized);
    }

    #[test]
    fn test_route_config_with_nested_fallback() {
        let fallback2 = RouteConfig::new("local", "llama-3");
        let fallback1 = RouteConfig::new("openai", "gpt-4").with_fallback(fallback2);
        let route = RouteConfig::new("anthropic", "claude-sonnet-4-6").with_fallback(fallback1);

        let fb1 = route.fallback.as_ref().expect("first fallback");
        assert_eq!(fb1.provider, "openai");

        let fb2 = fb1.fallback.as_ref().expect("second fallback");
        assert_eq!(fb2.provider, "local");
    }

    #[test]
    fn test_model_router_config_accessor() {
        let router = ModelRouter::default();
        let config = router.config();
        assert!(!config.categories.is_empty());
    }

    #[test]
    fn test_model_router_default() {
        let router = ModelRouter::default();
        assert!(router.route_category("deep").is_some());
    }

    #[test]
    fn test_model_router_debug() {
        let router = ModelRouter::default();
        let debug_str = format!("{:?}", router);
        assert!(debug_str.contains("ModelRouter"));
    }
}
