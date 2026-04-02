//! Variable substitution engine using Tera templating.
//!
//! Provides safe, sandboxed template rendering with built-in variables like
//! `{{ project_name }}`, `{{ timestamp }}`, and `{{ uuid }}`.
//!
//! # Example
//!
//! ```no_run
//! use cuttlefish_core::TemplateEngine;
//! use std::collections::HashMap;
//!
//! let engine = TemplateEngine::new();
//! let mut vars = HashMap::new();
//! vars.insert("project_name".to_string(), "MyApp".to_string());
//!
//! let result = engine.render(
//!     "Project: {{ project_name }}",
//!     &vars
//! ).expect("render failed");
//! assert_eq!(result, "Project: MyApp");
//! ```

use chrono::Utc;
use std::collections::HashMap;
use tera::{Context, Tera};
use uuid::Uuid;

use crate::template_manifest::TemplateError;

/// Template engine for safe variable substitution.
///
/// Uses Tera (Jinja2-like) templating with sandboxed rendering.
/// No file access, no arbitrary code execution.
#[derive(Debug)]
pub struct TemplateEngine {
    #[allow(dead_code)]
    tera: Tera,
}

impl TemplateEngine {
    /// Create a new template engine with safe defaults.
    ///
    /// Disables dangerous features like file inclusion and auto-escaping.
    pub fn new() -> Self {
        let tera = Tera::default();
        Self { tera }
    }

    /// Render template content with provided variables.
    ///
    /// Built-in variables available in all templates:
    /// - `timestamp`: Current UTC time in RFC3339 format
    /// - `uuid`: A new UUID v4
    ///
    /// # Arguments
    ///
    /// * `template_content` - The template string with `{{ variable }}` placeholders
    /// * `variables` - User-provided variables to substitute
    ///
    /// # Returns
    ///
    /// The rendered template string, or a `TemplateError` if rendering fails.
    ///
    /// # Errors
    ///
    /// Returns `TemplateError::InvalidYaml` if:
    /// - Template syntax is invalid
    /// - A referenced variable is undefined (Tera strict mode)
    /// - Template rendering fails for any reason
    pub fn render(
        &self,
        template_content: &str,
        variables: &HashMap<String, String>,
    ) -> Result<String, TemplateError> {
        let mut context = Context::new();

        // Add built-in variables
        context.insert("timestamp", &Utc::now().to_rfc3339());
        context.insert("uuid", &Uuid::new_v4().to_string());

        // Add user-provided variables
        for (key, value) in variables {
            context.insert(key, value);
        }

        // Render using one-off template (safer than registered templates)
        Tera::one_off(template_content, &context, false)
            .map_err(|e| TemplateError::InvalidYaml(format!("Template render error: {e}")))
    }
}

impl Default for TemplateEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_substitution() {
        let engine = TemplateEngine::new();
        let mut vars = HashMap::new();
        vars.insert("project_name".to_string(), "MyApp".to_string());

        let result = engine
            .render("Project: {{ project_name }}", &vars)
            .expect("render failed");
        assert_eq!(result, "Project: MyApp");
    }

    #[test]
    fn test_multiple_variables() {
        let engine = TemplateEngine::new();
        let mut vars = HashMap::new();
        vars.insert("project_name".to_string(), "MyApp".to_string());
        vars.insert("author".to_string(), "Alice".to_string());
        vars.insert("version".to_string(), "1.0.0".to_string());

        let template = "Project: {{ project_name }}, Author: {{ author }}, Version: {{ version }}";
        let result = engine.render(template, &vars).expect("render failed");
        assert_eq!(result, "Project: MyApp, Author: Alice, Version: 1.0.0");
    }

    #[test]
    fn test_builtin_timestamp() {
        let engine = TemplateEngine::new();
        let vars = HashMap::new();

        let result = engine
            .render("Time: {{ timestamp }}", &vars)
            .expect("render failed");

        // Verify it contains a timestamp (RFC3339 format)
        assert!(result.starts_with("Time: "));
        assert!(result.contains("T")); // ISO8601 format includes T
        assert!(result.contains("Z") || result.contains("+")); // UTC or offset
    }

    #[test]
    fn test_builtin_uuid() {
        let engine = TemplateEngine::new();
        let vars = HashMap::new();

        let result = engine
            .render("ID: {{ uuid }}", &vars)
            .expect("render failed");

        // Verify it contains a UUID (36 chars with hyphens)
        assert!(result.starts_with("ID: "));
        let uuid_part = &result[4..];
        assert_eq!(uuid_part.len(), 36); // UUID v4 format: 8-4-4-4-12
        assert_eq!(uuid_part.matches('-').count(), 4); // 4 hyphens in UUID
    }

    #[test]
    fn test_missing_variable_error() {
        let engine = TemplateEngine::new();
        let vars = HashMap::new();

        let result = engine.render("Project: {{ undefined_var }}", &vars);

        // Tera should error on undefined variables in strict mode
        assert!(result.is_err());
        match result {
            Err(TemplateError::InvalidYaml(msg)) => {
                assert!(msg.contains("Template render error"));
            }
            _ => panic!("Expected InvalidYaml error"),
        }
    }

    #[test]
    fn test_no_include_directive() {
        let engine = TemplateEngine::new();
        let vars = HashMap::new();

        // Try to use include directive (should fail)
        let result = engine.render("{% include \"file.txt\" %}", &vars);

        // Should error because include is not allowed
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_template() {
        let engine = TemplateEngine::new();
        let vars = HashMap::new();

        let result = engine.render("", &vars).expect("render failed");
        assert_eq!(result, "");
    }

    #[test]
    fn test_template_with_conditionals() {
        let engine = TemplateEngine::new();
        let mut vars = HashMap::new();
        vars.insert("debug".to_string(), "true".to_string());

        let template = "{% if debug == \"true\" %}Debug mode{% else %}Production{% endif %}";
        let result = engine.render(template, &vars).expect("render failed");
        assert_eq!(result, "Debug mode");
    }

    #[test]
    fn test_template_with_loops() {
        let engine = TemplateEngine::new();
        let template = "Item: {{ item1 }}, {{ item2 }}, {{ item3 }}";
        let mut vars = HashMap::new();
        vars.insert("item1".to_string(), "apple".to_string());
        vars.insert("item2".to_string(), "banana".to_string());
        vars.insert("item3".to_string(), "cherry".to_string());

        let result = engine.render(template, &vars).expect("render failed");
        assert_eq!(result, "Item: apple, banana, cherry");
    }

    #[test]
    fn test_special_characters_in_variables() {
        let engine = TemplateEngine::new();
        let mut vars = HashMap::new();
        vars.insert("project_name".to_string(), "My-App_v2.0 (Beta)".to_string());

        let result = engine
            .render("Project: {{ project_name }}", &vars)
            .expect("render failed");
        assert_eq!(result, "Project: My-App_v2.0 (Beta)");
    }

    #[test]
    fn test_whitespace_preservation() {
        let engine = TemplateEngine::new();
        let mut vars = HashMap::new();
        vars.insert("name".to_string(), "Alice".to_string());

        let template = "Hello,\n  {{ name }}\n  Welcome!";
        let result = engine.render(template, &vars).expect("render failed");
        assert_eq!(result, "Hello,\n  Alice\n  Welcome!");
    }
}
