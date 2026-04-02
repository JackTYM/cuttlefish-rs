//! Template manifest parser for YAML frontmatter in template files.
//!
//! This module provides parsing of markdown template files with YAML frontmatter,
//! extracting rich metadata about templates including variables, author, version, and tags.
//!
//! # File Format
//!
//! Template files use YAML frontmatter delimited by `---`:
//!
//! ```markdown
//! ---
//! name: nuxt-cloudflare
//! description: Nuxt 3 project on Cloudflare Pages
//! language: typescript
//! docker_image: node:20-alpine
//! author: Cuttlefish Team
//! version: 1.0.0
//! tags: [frontend, nuxt, cloudflare]
//! variables:
//!   - name: project_name
//!     description: Name of the project
//!     required: true
//!   - name: port
//!     description: Development server port
//!     default: "3000"
//!     required: false
//! ---
//!
//! # Nuxt 3 + Cloudflare Template
//! ...
//! ```

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Error type for template manifest operations.
#[derive(Error, Debug)]
pub enum TemplateError {
    /// Template file not found.
    #[error("Template not found: {0}")]
    NotFound(String),
    /// Invalid YAML frontmatter.
    #[error("Invalid YAML frontmatter: {0}")]
    InvalidYaml(String),
    /// Missing required body after frontmatter.
    #[error("Missing template body after frontmatter")]
    MissingBody,
    /// IO error reading file.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// A template variable definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateVariable {
    /// Variable name (e.g., "project_name").
    pub name: String,
    /// Human-readable description.
    pub description: String,
    /// Default value if not provided.
    #[serde(default)]
    pub default: Option<String>,
    /// Whether this variable is required.
    #[serde(default = "default_required")]
    pub required: bool,
}

/// Default value for required field (true).
fn default_required() -> bool {
    true
}

/// Parsed template manifest from YAML frontmatter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateManifest {
    /// Template name (e.g., "nuxt-cloudflare").
    pub name: String,
    /// One-line description of the template.
    pub description: String,
    /// Programming language/stack (e.g., "typescript", "rust").
    pub language: String,
    /// Docker base image for this template.
    pub docker_image: String,
    /// Template variables that can be substituted.
    #[serde(default)]
    pub variables: Vec<TemplateVariable>,
    /// Template author.
    #[serde(default)]
    pub author: Option<String>,
    /// Template version (semantic versioning).
    #[serde(default)]
    pub version: Option<String>,
    /// Tags for categorization (e.g., ["frontend", "nuxt"]).
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Parse YAML frontmatter from template content.
///
/// Extracts the YAML frontmatter between `---` delimiters and returns
/// both the parsed manifest and the remaining body content.
///
/// # Arguments
///
/// * `content` - The full template file content (frontmatter + body)
///
/// # Returns
///
/// A tuple of `(TemplateManifest, String)` containing the parsed manifest
/// and the body content after the frontmatter.
///
/// # Errors
///
/// Returns `TemplateError` if:
/// - The YAML frontmatter is invalid
/// - The file has no body content after frontmatter
/// - Required fields are missing from the manifest
///
/// # Example
///
/// ```no_run
/// use cuttlefish_core::template_manifest::parse_manifest;
///
/// let content = r#"---
/// name: my-template
/// description: A test template
/// language: rust
/// docker_image: rust:latest
/// ---
/// # Template body
/// "#;
///
/// let (manifest, body) = parse_manifest(content).expect("Failed to parse");
/// assert_eq!(manifest.name, "my-template");
/// ```
pub fn parse_manifest(content: &str) -> Result<(TemplateManifest, String), TemplateError> {
    // Find the first --- delimiter
    let first_delimiter = content.find("---").ok_or(TemplateError::MissingBody)?;

    // Find the second --- delimiter after the first one
    let after_first = &content[first_delimiter + 3..];
    let second_delimiter = after_first.find("---").ok_or(TemplateError::MissingBody)?;

    // Extract YAML content between delimiters
    let yaml_content = &after_first[..second_delimiter].trim();

    // Parse YAML
    let manifest: TemplateManifest = serde_yaml::from_str(yaml_content)
        .map_err(|e| TemplateError::InvalidYaml(e.to_string()))?;

    // Extract body content after the second delimiter
    let body_start = first_delimiter + 3 + second_delimiter + 3;
    let body = if body_start < content.len() {
        content[body_start..].trim_start().to_string()
    } else {
        String::new()
    };

    Ok((manifest, body))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_manifest() {
        let content = r#"---
name: nuxt-cloudflare
description: Nuxt 3 on Cloudflare Pages
language: typescript
docker_image: node:20-alpine
author: Cuttlefish Team
version: 1.0.0
tags: [frontend, nuxt]
variables:
  - name: project_name
    description: Project name
    required: true
  - name: port
    description: Dev server port
    default: "3000"
    required: false
---
# Nuxt 3 Template
This is the template body.
"#;

        let (manifest, body) = parse_manifest(content).expect("Failed to parse");

        assert_eq!(manifest.name, "nuxt-cloudflare");
        assert_eq!(manifest.description, "Nuxt 3 on Cloudflare Pages");
        assert_eq!(manifest.language, "typescript");
        assert_eq!(manifest.docker_image, "node:20-alpine");
        assert_eq!(manifest.author, Some("Cuttlefish Team".to_string()));
        assert_eq!(manifest.version, Some("1.0.0".to_string()));
        assert_eq!(manifest.tags, vec!["frontend", "nuxt"]);
        assert_eq!(manifest.variables.len(), 2);
        assert_eq!(manifest.variables[0].name, "project_name");
        assert!(manifest.variables[0].required);
        assert_eq!(manifest.variables[1].default, Some("3000".to_string()));
        assert!(!manifest.variables[1].required);
        assert!(body.contains("# Nuxt 3 Template"));
    }

    #[test]
    fn test_parse_invalid_yaml() {
        let content = r#"---
name: test
description: test
language: rust
docker_image: rust:latest
variables:
  - name: invalid
    description: test
    required: not_a_bool
---
Body
"#;

        let result = parse_manifest(content);
        assert!(result.is_err());
        match result {
            Err(TemplateError::InvalidYaml(_)) => (),
            _ => panic!("Expected InvalidYaml error"),
        }
    }

    #[test]
    fn test_parse_missing_body() {
        let content = r#"---
name: test
description: test
language: rust
docker_image: rust:latest
---"#;

        let (manifest, body) = parse_manifest(content).expect("Failed to parse");
        assert_eq!(manifest.name, "test");
        assert_eq!(body, "");
    }

    #[test]
    fn test_parse_no_frontmatter() {
        let content = "Just some content without frontmatter";

        let result = parse_manifest(content);
        assert!(result.is_err());
        match result {
            Err(TemplateError::MissingBody) => (),
            _ => panic!("Expected MissingBody error"),
        }
    }

    #[test]
    fn test_variables_default() {
        let content = r#"---
name: minimal
description: Minimal template
language: rust
docker_image: rust:latest
---
Body content
"#;

        let (manifest, _) = parse_manifest(content).expect("Failed to parse");
        assert_eq!(manifest.variables.len(), 0);
        assert_eq!(manifest.author, None);
        assert_eq!(manifest.version, None);
        assert_eq!(manifest.tags.len(), 0);
    }

    #[test]
    fn test_parse_with_whitespace() {
        let content = r#"---
name: test-whitespace
description: Test with whitespace
language: python
docker_image: python:3.11

variables:
  - name: var1
    description: First variable
    required: true

---

# Body with leading whitespace
Some content here
"#;

        let (manifest, body) = parse_manifest(content).expect("Failed to parse");
        assert_eq!(manifest.name, "test-whitespace");
        assert_eq!(manifest.variables.len(), 1);
        assert!(body.starts_with("# Body"));
    }
}
