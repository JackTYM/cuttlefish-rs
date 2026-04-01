//! Docker image registry for Cuttlefish template stacks.

use std::collections::HashMap;

/// Mapping from template/language name to Docker image reference.
///
/// Images can be:
/// - Pre-built public images (e.g., "node:22-slim")  
/// - Cuttlefish base images built from Dockerfiles in docker/ directory
#[derive(Debug, Clone)]
pub struct ImageRegistry {
    /// Map of template name → Docker image name.
    images: HashMap<String, String>,
}

impl ImageRegistry {
    /// Create an `ImageRegistry` with the default Cuttlefish base images.
    ///
    /// Defaults use public official images directly:
    /// - `node` / `nuxt` / `typescript` → `node:22-slim`
    /// - `python` / `fastapi` → `python:3.12-slim`  
    /// - `rust` / `axum` → `rust:1.82-slim`
    /// - `go` / `golang` → `golang:1.22-bookworm`
    /// - `generic` / `bash` / `shell` → `ubuntu:22.04`
    pub fn default_registry() -> Self {
        let mut images = HashMap::new();

        // Node.js family
        images.insert("node".to_string(), "node:22-slim".to_string());
        images.insert("nuxt".to_string(), "node:22-slim".to_string());
        images.insert("nuxt-cloudflare".to_string(), "node:22-slim".to_string());
        images.insert("typescript".to_string(), "node:22-slim".to_string());
        images.insert("node-express".to_string(), "node:22-slim".to_string());

        // Python family
        images.insert("python".to_string(), "python:3.12-slim".to_string());
        images.insert("python-fastapi".to_string(), "python:3.12-slim".to_string());
        images.insert("fastapi".to_string(), "python:3.12-slim".to_string());

        // Rust family
        images.insert("rust".to_string(), "rust:1.82-slim".to_string());
        images.insert("rust-axum".to_string(), "rust:1.82-slim".to_string());
        images.insert("axum".to_string(), "rust:1.82-slim".to_string());

        // Go family
        images.insert("go".to_string(), "golang:1.22-bookworm".to_string());
        images.insert("golang".to_string(), "golang:1.22-bookworm".to_string());

        // Generic
        images.insert("generic".to_string(), "ubuntu:22.04".to_string());
        images.insert("static-site".to_string(), "node:22-slim".to_string());

        Self { images }
    }

    /// Resolve a template name to a Docker image.
    ///
    /// Falls back to the generic Ubuntu image if the template is unknown.
    pub fn resolve(&self, template_name: &str) -> &str {
        self.images
            .get(template_name)
            .map(|s| s.as_str())
            .unwrap_or("ubuntu:22.04")
    }

    /// Register a custom template → image mapping.
    pub fn register(&mut self, template_name: impl Into<String>, image: impl Into<String>) {
        self.images.insert(template_name.into(), image.into());
    }

    /// Check if a template has a registered image.
    pub fn has_template(&self, template_name: &str) -> bool {
        self.images.contains_key(template_name)
    }

    /// List all registered template names.
    pub fn list_templates(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self.images.keys().map(|s| s.as_str()).collect();
        names.sort();
        names
    }

    /// Get the Dockerfile path for a template (relative to workspace root).
    ///
    /// Returns the path to the Dockerfile if one exists for the template's language family.
    pub fn dockerfile_path(&self, template_name: &str) -> Option<std::path::PathBuf> {
        let image = self.resolve(template_name);
        let dockerfile_name = if image.starts_with("node:") {
            "node-base.Dockerfile"
        } else if image.starts_with("python:") {
            "python-base.Dockerfile"
        } else if image.starts_with("rust:") {
            "rust-base.Dockerfile"
        } else if image.starts_with("golang:") {
            "go-base.Dockerfile"
        } else {
            "generic-base.Dockerfile"
        };
        Some(std::path::PathBuf::from("docker").join(dockerfile_name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_registry_resolves_node() {
        let registry = ImageRegistry::default_registry();
        assert_eq!(registry.resolve("node"), "node:22-slim");
        assert_eq!(registry.resolve("nuxt"), "node:22-slim");
    }

    #[test]
    fn test_default_registry_resolves_python() {
        let registry = ImageRegistry::default_registry();
        assert_eq!(registry.resolve("python"), "python:3.12-slim");
        assert_eq!(registry.resolve("fastapi"), "python:3.12-slim");
    }

    #[test]
    fn test_default_registry_resolves_rust() {
        let registry = ImageRegistry::default_registry();
        assert_eq!(registry.resolve("rust"), "rust:1.82-slim");
    }

    #[test]
    fn test_unknown_template_returns_generic() {
        let registry = ImageRegistry::default_registry();
        assert_eq!(registry.resolve("unknown-language"), "ubuntu:22.04");
    }

    #[test]
    fn test_register_custom_template() {
        let mut registry = ImageRegistry::default_registry();
        registry.register("elixir", "elixir:1.16-slim");
        assert_eq!(registry.resolve("elixir"), "elixir:1.16-slim");
    }

    #[test]
    fn test_has_template() {
        let registry = ImageRegistry::default_registry();
        assert!(registry.has_template("node"));
        assert!(!registry.has_template("cobol"));
    }

    #[test]
    fn test_list_templates_sorted() {
        let registry = ImageRegistry::default_registry();
        let templates = registry.list_templates();
        assert!(!templates.is_empty());
        let mut sorted = templates.clone();
        sorted.sort();
        assert_eq!(templates, sorted);
    }

    #[test]
    fn test_dockerfile_path_for_node() {
        let registry = ImageRegistry::default_registry();
        let path = registry.dockerfile_path("node").expect("path");
        assert_eq!(path.to_str().expect("str"), "docker/node-base.Dockerfile");
    }
}
