//! Simple reverse proxy for exposing project dev servers.

use std::collections::HashMap;

/// A proxy route mapping project name to target address.
#[derive(Debug, Clone)]
pub struct ProxyRoute {
    /// Project name.
    pub project_name: String,
    /// Target host:port (e.g., "localhost:3000").
    pub target: String,
}

/// Registry of active proxy routes.
#[derive(Default)]
pub struct ProxyRegistry {
    /// Map of project name → target address.
    routes: HashMap<String, String>,
}

impl ProxyRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self {
            routes: HashMap::new(),
        }
    }

    /// Register a proxy route for a project.
    pub fn register(&mut self, project_name: impl Into<String>, target: impl Into<String>) {
        self.routes.insert(project_name.into(), target.into());
    }

    /// Remove a proxy route.
    pub fn unregister(&mut self, project_name: &str) {
        self.routes.remove(project_name);
    }

    /// Resolve a project name to its proxy target.
    pub fn resolve(&self, project_name: &str) -> Option<&str> {
        self.routes.get(project_name).map(|s| s.as_str())
    }

    /// List all active proxy routes.
    pub fn list_routes(&self) -> Vec<ProxyRoute> {
        self.routes
            .iter()
            .map(|(k, v)| ProxyRoute {
                project_name: k.clone(),
                target: v.clone(),
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_and_resolve() {
        let mut reg = ProxyRegistry::new();
        reg.register("my-app", "localhost:3000");
        assert_eq!(reg.resolve("my-app"), Some("localhost:3000"));
    }

    #[test]
    fn test_unregister() {
        let mut reg = ProxyRegistry::new();
        reg.register("app", "localhost:8080");
        reg.unregister("app");
        assert!(reg.resolve("app").is_none());
    }

    #[test]
    fn test_resolve_unknown_returns_none() {
        let reg = ProxyRegistry::new();
        assert!(reg.resolve("nonexistent").is_none());
    }

    #[test]
    fn test_list_routes() {
        let mut reg = ProxyRegistry::new();
        reg.register("app1", "localhost:3000");
        reg.register("app2", "localhost:3001");
        let routes = reg.list_routes();
        assert_eq!(routes.len(), 2);
    }
}
