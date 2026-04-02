//! Provider registry for managing named model provider instances.

use cuttlefish_core::traits::provider::ModelProvider;
use std::collections::HashMap;
use std::sync::Arc;

/// Registry that holds named provider instances.
///
/// The registry allows registering model providers by name and retrieving them
/// for use in routing decisions. Providers are stored as `Arc<dyn ModelProvider>`
/// to allow shared ownership across the application.
///
/// # Example
/// ```ignore
/// use cuttlefish_providers::registry::ProviderRegistry;
/// use cuttlefish_providers::mock::MockProvider;
/// use std::sync::Arc;
///
/// let mut registry = ProviderRegistry::new();
/// registry.register("anthropic", Arc::new(MockProvider::new()));
/// let provider = registry.get("anthropic");
/// ```
#[derive(Default)]
pub struct ProviderRegistry {
    providers: HashMap<String, Arc<dyn ModelProvider>>,
}

impl ProviderRegistry {
    /// Create a new empty provider registry.
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
        }
    }

    /// Register a provider with the given name.
    ///
    /// If a provider with the same name already exists, it will be replaced.
    pub fn register(&mut self, name: impl Into<String>, provider: Arc<dyn ModelProvider>) {
        self.providers.insert(name.into(), provider);
    }

    /// Get a provider by name.
    ///
    /// Returns `None` if no provider with the given name is registered.
    pub fn get(&self, name: &str) -> Option<Arc<dyn ModelProvider>> {
        self.providers.get(name).cloned()
    }

    /// Get a list of all registered provider names.
    pub fn names(&self) -> Vec<&str> {
        self.providers.keys().map(|s| s.as_str()).collect()
    }

    /// Check if a provider with the given name is registered.
    pub fn contains(&self, name: &str) -> bool {
        self.providers.contains_key(name)
    }

    /// Get the number of registered providers.
    pub fn len(&self) -> usize {
        self.providers.len()
    }

    /// Check if the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.providers.is_empty()
    }
}

impl std::fmt::Debug for ProviderRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProviderRegistry")
            .field("providers", &self.providers.keys().collect::<Vec<_>>())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::MockModelProvider;

    #[test]
    fn test_new_registry_is_empty() {
        let registry = ProviderRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
    }

    #[test]
    fn test_register_and_get_provider() {
        let mut registry = ProviderRegistry::new();
        let provider = Arc::new(MockModelProvider::new("mock"));

        registry.register("anthropic", provider.clone());

        let retrieved = registry.get("anthropic");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.expect("provider should exist").name(), "mock");
    }

    #[test]
    fn test_get_missing_provider_returns_none() {
        let registry = ProviderRegistry::new();
        assert!(registry.get("nonexistent").is_none());
    }

    #[test]
    fn test_names_returns_all_registered_names() {
        let mut registry = ProviderRegistry::new();
        registry.register("anthropic", Arc::new(MockModelProvider::new("mock")));
        registry.register("openai", Arc::new(MockModelProvider::new("mock")));
        registry.register("google", Arc::new(MockModelProvider::new("mock")));

        let mut names = registry.names();
        names.sort();

        assert_eq!(names, vec!["anthropic", "google", "openai"]);
    }

    #[test]
    fn test_contains() {
        let mut registry = ProviderRegistry::new();
        registry.register("anthropic", Arc::new(MockModelProvider::new("mock")));

        assert!(registry.contains("anthropic"));
        assert!(!registry.contains("openai"));
    }

    #[test]
    fn test_len_and_is_empty() {
        let mut registry = ProviderRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);

        registry.register("anthropic", Arc::new(MockModelProvider::new("mock")));
        assert!(!registry.is_empty());
        assert_eq!(registry.len(), 1);

        registry.register("openai", Arc::new(MockModelProvider::new("mock")));
        assert_eq!(registry.len(), 2);
    }

    #[test]
    fn test_register_replaces_existing() {
        let mut registry = ProviderRegistry::new();
        let provider1 = Arc::new(MockModelProvider::new("mock"));
        let provider2 = Arc::new(MockModelProvider::new("mock"));

        registry.register("anthropic", provider1);
        registry.register("anthropic", provider2);

        assert_eq!(registry.len(), 1);
    }

    #[test]
    fn test_default_creates_empty_registry() {
        let registry = ProviderRegistry::default();
        assert!(registry.is_empty());
    }

    #[test]
    fn test_debug_impl() {
        let mut registry = ProviderRegistry::new();
        registry.register("anthropic", Arc::new(MockModelProvider::new("mock")));

        let debug_str = format!("{:?}", registry);
        assert!(debug_str.contains("ProviderRegistry"));
        assert!(debug_str.contains("anthropic"));
    }
}
