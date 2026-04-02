//! Template registry for loading, caching, and rendering templates.
//!
//! Combines `TemplateManifest` parsing with `TemplateEngine` rendering to provide
//! a unified interface for template management.
//!
//! # Example
//!
//! ```no_run
//! use cuttlefish_core::TemplateRegistry;
//! use std::path::Path;
//! use std::collections::HashMap;
//!
//! let registry = TemplateRegistry::new();
//!
//! // Load templates from a directory
//! let count = registry.load_from_dir(Path::new("templates/")).unwrap();
//! println!("Loaded {} templates", count);
//!
//! // Render a template with variables
//! let mut vars = HashMap::new();
//! vars.insert("project_name".to_string(), "MyApp".to_string());
//! let rendered = registry.render("my-template", &vars).unwrap();
//! ```

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::RwLock;

use crate::template_engine::TemplateEngine;
use crate::template_manifest::{parse_manifest, TemplateError, TemplateManifest};

/// Source of a loaded template.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TemplateSource {
    /// Loaded from local filesystem.
    Local(PathBuf),
    /// Fetched from remote URL.
    Remote(String),
}

/// A fully loaded template with manifest and content.
#[derive(Debug, Clone)]
pub struct LoadedTemplate {
    /// Parsed manifest metadata from YAML frontmatter.
    pub manifest: TemplateManifest,
    /// Template body content (after frontmatter).
    pub content: String,
    /// Where the template was loaded from.
    pub source: TemplateSource,
}

/// Registry that loads, caches, and renders templates.
///
/// Thread-safe via `RwLock` for concurrent access patterns.
pub struct TemplateRegistry {
    templates: RwLock<HashMap<String, LoadedTemplate>>,
    engine: TemplateEngine,
}

impl TemplateRegistry {
    /// Create a new empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self {
            templates: RwLock::new(HashMap::new()),
            engine: TemplateEngine::new(),
        }
    }

    /// Load all templates from a directory.
    ///
    /// Iterates over `.md` files in the directory, parses their YAML frontmatter,
    /// and caches them by name.
    ///
    /// # Arguments
    ///
    /// * `dir` - Path to the directory containing template files
    ///
    /// # Returns
    ///
    /// The count of successfully loaded templates.
    ///
    /// # Errors
    ///
    /// Returns `TemplateError::Io` if the directory cannot be read.
    /// Returns `TemplateError::InvalidYaml` if any template has invalid frontmatter.
    pub fn load_from_dir(&self, dir: &Path) -> Result<usize, TemplateError> {
        let entries = std::fs::read_dir(dir)?;
        let mut count = 0;

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("md") {
                continue;
            }

            let content = std::fs::read_to_string(&path)?;
            let (manifest, body) = parse_manifest(&content)?;

            let template = LoadedTemplate {
                manifest: manifest.clone(),
                content: body,
                source: TemplateSource::Local(path),
            };

            self.templates
                .write()
                .expect("RwLock poisoned")
                .insert(manifest.name.clone(), template);
            count += 1;
        }

        Ok(count)
    }

    /// Get a template by name.
    ///
    /// # Arguments
    ///
    /// * `name` - The template name as defined in its manifest
    ///
    /// # Returns
    ///
    /// A cloned copy of the template if found, or `None`.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<LoadedTemplate> {
        self.templates
            .read()
            .expect("RwLock poisoned")
            .get(name)
            .cloned()
    }

    /// List all loaded templates.
    ///
    /// # Returns
    ///
    /// A vector containing cloned copies of all cached templates.
    #[must_use]
    pub fn list(&self) -> Vec<LoadedTemplate> {
        self.templates
            .read()
            .expect("RwLock poisoned")
            .values()
            .cloned()
            .collect()
    }

    /// Render a template with variables.
    ///
    /// # Arguments
    ///
    /// * `name` - The template name to render
    /// * `variables` - Variable substitutions for `{{ variable }}` placeholders
    ///
    /// # Returns
    ///
    /// The rendered template content.
    ///
    /// # Errors
    ///
    /// Returns `TemplateError::NotFound` if the template doesn't exist.
    /// Returns `TemplateError::InvalidYaml` if rendering fails.
    pub fn render(
        &self,
        name: &str,
        variables: &HashMap<String, String>,
    ) -> Result<String, TemplateError> {
        let template = self
            .get(name)
            .ok_or_else(|| TemplateError::NotFound(name.to_string()))?;
        self.engine.render(&template.content, variables)
    }

    /// Add a template directly to the registry.
    ///
    /// Useful for templates loaded from remote sources.
    ///
    /// # Arguments
    ///
    /// * `template` - The fully loaded template to cache
    pub fn add(&self, template: LoadedTemplate) {
        self.templates
            .write()
            .expect("RwLock poisoned")
            .insert(template.manifest.name.clone(), template);
    }

    /// Clear all cached templates.
    pub fn clear(&self) {
        self.templates.write().expect("RwLock poisoned").clear();
    }

    /// Get the number of cached templates.
    #[must_use]
    pub fn len(&self) -> usize {
        self.templates.read().expect("RwLock poisoned").len()
    }

    /// Check if the registry is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Default for TemplateRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_template(name: &str) -> String {
        format!(
            r#"---
name: {name}
description: Test template for {name}
language: rust
docker_image: rust:latest
variables:
  - name: project_name
    description: Name of the project
    required: true
---
# {{{{ project_name }}}}

This is template {name}.
"#
        )
    }

    #[test]
    fn test_load_from_dir() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        // Create test template files
        fs::write(
            temp_dir.path().join("template1.md"),
            create_test_template("template1"),
        )
        .expect("Failed to write file");
        fs::write(
            temp_dir.path().join("template2.md"),
            create_test_template("template2"),
        )
        .expect("Failed to write file");

        // Also create a non-.md file that should be ignored
        fs::write(temp_dir.path().join("readme.txt"), "Should be ignored")
            .expect("Failed to write file");

        let registry = TemplateRegistry::new();
        let count = registry
            .load_from_dir(temp_dir.path())
            .expect("Failed to load templates");

        assert_eq!(count, 2);
        assert_eq!(registry.len(), 2);
    }

    #[test]
    fn test_get_existing() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        fs::write(
            temp_dir.path().join("my-template.md"),
            create_test_template("my-template"),
        )
        .expect("Failed to write file");

        let registry = TemplateRegistry::new();
        registry
            .load_from_dir(temp_dir.path())
            .expect("Failed to load templates");

        let template = registry.get("my-template");
        assert!(template.is_some());

        let template = template.expect("Template should exist");
        assert_eq!(template.manifest.name, "my-template");
        assert_eq!(template.manifest.language, "rust");
        assert!(template.content.contains("This is template my-template."));
    }

    #[test]
    fn test_get_nonexistent() {
        let registry = TemplateRegistry::new();
        let template = registry.get("nonexistent-template");
        assert!(template.is_none());
    }

    #[test]
    fn test_list_all() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        fs::write(
            temp_dir.path().join("alpha.md"),
            create_test_template("alpha"),
        )
        .expect("Failed to write file");
        fs::write(
            temp_dir.path().join("beta.md"),
            create_test_template("beta"),
        )
        .expect("Failed to write file");
        fs::write(
            temp_dir.path().join("gamma.md"),
            create_test_template("gamma"),
        )
        .expect("Failed to write file");

        let registry = TemplateRegistry::new();
        registry
            .load_from_dir(temp_dir.path())
            .expect("Failed to load templates");

        let templates = registry.list();
        assert_eq!(templates.len(), 3);

        let names: Vec<&str> = templates.iter().map(|t| t.manifest.name.as_str()).collect();
        assert!(names.contains(&"alpha"));
        assert!(names.contains(&"beta"));
        assert!(names.contains(&"gamma"));
    }

    #[test]
    fn test_render_with_variables() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        fs::write(
            temp_dir.path().join("render-test.md"),
            create_test_template("render-test"),
        )
        .expect("Failed to write file");

        let registry = TemplateRegistry::new();
        registry
            .load_from_dir(temp_dir.path())
            .expect("Failed to load templates");

        let mut vars = HashMap::new();
        vars.insert("project_name".to_string(), "AwesomeProject".to_string());

        let rendered = registry
            .render("render-test", &vars)
            .expect("Failed to render");

        assert!(rendered.contains("# AwesomeProject"));
        assert!(rendered.contains("This is template render-test."));
    }

    #[test]
    fn test_render_nonexistent() {
        let registry = TemplateRegistry::new();
        let vars = HashMap::new();

        let result = registry.render("does-not-exist", &vars);
        assert!(result.is_err());

        match result {
            Err(TemplateError::NotFound(name)) => {
                assert_eq!(name, "does-not-exist");
            }
            _ => panic!("Expected NotFound error"),
        }
    }

    #[test]
    fn test_clear() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        fs::write(
            temp_dir.path().join("clear-test.md"),
            create_test_template("clear-test"),
        )
        .expect("Failed to write file");

        let registry = TemplateRegistry::new();
        registry
            .load_from_dir(temp_dir.path())
            .expect("Failed to load templates");

        assert!(!registry.is_empty());
        assert_eq!(registry.len(), 1);

        registry.clear();

        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
        assert!(registry.get("clear-test").is_none());
    }

    #[test]
    fn test_add_template_directly() {
        let registry = TemplateRegistry::new();

        let template = LoadedTemplate {
            manifest: TemplateManifest {
                name: "direct-add".to_string(),
                description: "Directly added template".to_string(),
                language: "typescript".to_string(),
                docker_image: "node:20".to_string(),
                variables: vec![],
                author: Some("Test Author".to_string()),
                version: Some("1.0.0".to_string()),
                tags: vec!["test".to_string()],
            },
            content: "# Direct Template\n\nContent here.".to_string(),
            source: TemplateSource::Remote("https://example.com/template.md".to_string()),
        };

        registry.add(template);

        let retrieved = registry.get("direct-add");
        assert!(retrieved.is_some());

        let retrieved = retrieved.expect("Template should exist");
        assert_eq!(retrieved.manifest.name, "direct-add");
        assert_eq!(
            retrieved.source,
            TemplateSource::Remote("https://example.com/template.md".to_string())
        );
    }

    #[test]
    fn test_template_source_equality() {
        let local1 = TemplateSource::Local(PathBuf::from("/path/to/file.md"));
        let local2 = TemplateSource::Local(PathBuf::from("/path/to/file.md"));
        let local3 = TemplateSource::Local(PathBuf::from("/different/path.md"));
        let remote1 = TemplateSource::Remote("https://example.com/template.md".to_string());
        let remote2 = TemplateSource::Remote("https://example.com/template.md".to_string());

        assert_eq!(local1, local2);
        assert_ne!(local1, local3);
        assert_eq!(remote1, remote2);
        assert_ne!(local1, remote1);
    }

    #[test]
    fn test_default_impl() {
        let registry = TemplateRegistry::default();
        assert!(registry.is_empty());
    }
}
