//! Runtime prompt loading from YAML frontmatter markdown files.
//!
//! This module provides a registry for loading agent prompts from `.md` files
//! with YAML frontmatter at runtime. Prompts are cached using `RwLock` for
//! thread-safe access.
//!
//! # File Format
//!
//! Prompt files use YAML frontmatter delimited by `---`:
//!
//! ```markdown
//! ---
//! name: orchestrator
//! description: Plans and delegates work
//! tools: ["Read", "Write", "Bash"]
//! category: deep
//! ---
//!
//! You are the orchestrator agent...
//! ```

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::RwLock;

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Error type for prompt operations.
#[derive(Error, Debug)]
pub enum PromptError {
    /// Prompt file not found.
    #[error("Prompt file not found: {0}")]
    NotFound(String),
    /// Invalid YAML frontmatter.
    #[error("Invalid YAML frontmatter: {0}")]
    InvalidYaml(String),
    /// Missing required body after frontmatter.
    #[error("Missing prompt body after YAML frontmatter")]
    MissingBody,
    /// IO error reading file.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Metadata from YAML frontmatter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptMetadata {
    /// Agent name (e.g., "orchestrator", "coder").
    pub name: String,
    /// One-line description.
    pub description: String,
    /// List of available tools for this agent.
    #[serde(default)]
    pub tools: Vec<String>,
    /// Model category (deep, quick, ultrabrain, etc.).
    pub category: String,
}

/// A loaded agent prompt with metadata and body.
#[derive(Debug, Clone)]
pub struct AgentPrompt {
    /// Parsed YAML metadata.
    pub metadata: PromptMetadata,
    /// The markdown body (system prompt content).
    pub body: String,
}

/// Registry that loads and caches agent prompts from disk.
///
/// The registry uses a `RwLock` to cache loaded prompts, avoiding repeated
/// file reads for the same agent.
pub struct PromptRegistry {
    prompts_dir: PathBuf,
    cache: RwLock<HashMap<String, AgentPrompt>>,
}

impl PromptRegistry {
    /// Creates a new `PromptRegistry` with the given prompts directory.
    ///
    /// # Arguments
    ///
    /// * `prompts_dir` - Path to the directory containing `.md` prompt files.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use cuttlefish_agents::PromptRegistry;
    ///
    /// let registry = PromptRegistry::new("./prompts");
    /// ```
    pub fn new(prompts_dir: impl Into<PathBuf>) -> Self {
        Self {
            prompts_dir: prompts_dir.into(),
            cache: RwLock::new(HashMap::new()),
        }
    }

    /// Loads an agent prompt by name.
    ///
    /// This method first checks the cache. If the prompt is not cached,
    /// it reads the file from disk, parses it, and caches the result.
    ///
    /// # Arguments
    ///
    /// * `agent_name` - The name of the agent (e.g., "orchestrator").
    ///
    /// # Returns
    ///
    /// The loaded `AgentPrompt` or a `PromptError` if loading fails.
    ///
    /// # Errors
    ///
    /// - `PromptError::NotFound` if the file doesn't exist.
    /// - `PromptError::InvalidYaml` if the YAML frontmatter is malformed.
    /// - `PromptError::MissingBody` if there's no content after frontmatter.
    /// - `PromptError::Io` for other I/O errors.
    pub fn load(&self, agent_name: &str) -> Result<AgentPrompt, PromptError> {
        {
            let cache = self
                .cache
                .read()
                .expect("RwLock poisoned - concurrent panic occurred");
            if let Some(prompt) = cache.get(agent_name) {
                return Ok(prompt.clone());
            }
        }

        let file_path = self.prompts_dir.join(format!("{agent_name}.md"));

        if !file_path.exists() {
            return Err(PromptError::NotFound(agent_name.to_string()));
        }

        let content = std::fs::read_to_string(&file_path)?;
        let prompt = Self::parse_prompt(&content)?;

        {
            let mut cache = self
                .cache
                .write()
                .expect("RwLock poisoned - concurrent panic occurred");
            cache.insert(agent_name.to_string(), prompt.clone());
        }

        Ok(prompt)
    }

    /// Parses a prompt file content into an `AgentPrompt`.
    ///
    /// The file format is:
    /// ```text
    /// ---
    /// YAML frontmatter
    /// ---
    /// Markdown body
    /// ```
    fn parse_prompt(content: &str) -> Result<AgentPrompt, PromptError> {
        let content = content.trim();

        if !content.starts_with("---") {
            return Err(PromptError::InvalidYaml(
                "File must start with YAML frontmatter delimiter '---'".to_string(),
            ));
        }

        let after_first_delimiter = &content[3..];
        let closing_pos = after_first_delimiter.find("\n---").ok_or_else(|| {
            PromptError::InvalidYaml("Missing closing '---' delimiter".to_string())
        })?;

        let yaml_content = after_first_delimiter[..closing_pos].trim();
        let body_start = closing_pos + 4;

        if body_start >= after_first_delimiter.len() {
            return Err(PromptError::MissingBody);
        }

        let body = after_first_delimiter[body_start..].trim();

        if body.is_empty() {
            return Err(PromptError::MissingBody);
        }

        let metadata: PromptMetadata = serde_yaml::from_str(yaml_content)
            .map_err(|e| PromptError::InvalidYaml(e.to_string()))?;

        Ok(AgentPrompt {
            metadata,
            body: body.to_string(),
        })
    }

    /// Returns the prompts directory path.
    pub fn prompts_dir(&self) -> &Path {
        &self.prompts_dir
    }

    /// Clears the cache, forcing prompts to be reloaded from disk.
    pub fn clear_cache(&self) {
        let mut cache = self
            .cache
            .write()
            .expect("RwLock poisoned - concurrent panic occurred");
        cache.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_prompt(dir: &Path, name: &str, content: &str) {
        let file_path = dir.join(format!("{name}.md"));
        fs::write(file_path, content).expect("Failed to write test file");
    }

    #[test]
    fn test_load_valid_prompt() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let content = r#"---
name: orchestrator
description: Plans and delegates work
tools:
  - Read
  - Write
  - Bash
category: deep
---

You are the orchestrator agent. Your job is to plan and delegate work."#;

        create_test_prompt(temp_dir.path(), "orchestrator", content);

        let registry = PromptRegistry::new(temp_dir.path());
        let prompt = registry
            .load("orchestrator")
            .expect("Failed to load prompt");

        assert_eq!(prompt.metadata.name, "orchestrator");
        assert_eq!(prompt.metadata.description, "Plans and delegates work");
        assert_eq!(prompt.metadata.tools, vec!["Read", "Write", "Bash"]);
        assert_eq!(prompt.metadata.category, "deep");
        assert!(prompt.body.contains("You are the orchestrator agent"));
    }

    #[test]
    fn test_load_missing_prompt() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let registry = PromptRegistry::new(temp_dir.path());

        let result = registry.load("nonexistent");
        assert!(matches!(result, Err(PromptError::NotFound(_))));

        if let Err(PromptError::NotFound(name)) = result {
            assert_eq!(name, "nonexistent");
        }
    }

    #[test]
    fn test_load_invalid_yaml() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let content = r#"---
name: [invalid yaml
description: missing bracket
---

Body content here."#;

        create_test_prompt(temp_dir.path(), "invalid", content);

        let registry = PromptRegistry::new(temp_dir.path());
        let result = registry.load("invalid");

        assert!(matches!(result, Err(PromptError::InvalidYaml(_))));
    }

    #[test]
    fn test_cache_works() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let content = r#"---
name: coder
description: Writes code
tools: []
category: deep
---

You are the coder agent."#;

        create_test_prompt(temp_dir.path(), "coder", content);

        let registry = PromptRegistry::new(temp_dir.path());

        let prompt1 = registry.load("coder").expect("Failed to load prompt");

        fs::remove_file(temp_dir.path().join("coder.md")).expect("Failed to delete file");

        let prompt2 = registry.load("coder").expect("Failed to load from cache");

        assert_eq!(prompt1.metadata.name, prompt2.metadata.name);
        assert_eq!(prompt1.body, prompt2.body);

        registry.clear_cache();
        let result = registry.load("coder");
        assert!(matches!(result, Err(PromptError::NotFound(_))));
    }

    #[test]
    fn test_missing_body() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let content = r#"---
name: empty
description: No body
tools: []
category: quick
---
"#;

        create_test_prompt(temp_dir.path(), "empty", content);

        let registry = PromptRegistry::new(temp_dir.path());
        let result = registry.load("empty");

        assert!(matches!(result, Err(PromptError::MissingBody)));
    }

    #[test]
    fn test_missing_frontmatter_delimiter() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let content = r#"---
name: broken
description: Missing closing delimiter
tools: []
category: quick

Body without closing delimiter."#;

        create_test_prompt(temp_dir.path(), "broken", content);

        let registry = PromptRegistry::new(temp_dir.path());
        let result = registry.load("broken");

        assert!(matches!(result, Err(PromptError::InvalidYaml(_))));
    }

    #[test]
    fn test_no_frontmatter_start() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let content = r#"Just plain markdown without frontmatter."#;

        create_test_prompt(temp_dir.path(), "plain", content);

        let registry = PromptRegistry::new(temp_dir.path());
        let result = registry.load("plain");

        assert!(matches!(result, Err(PromptError::InvalidYaml(_))));
    }

    #[test]
    fn test_empty_tools_default() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let content = r#"---
name: minimal
description: Minimal agent
category: quick
---

Minimal body."#;

        create_test_prompt(temp_dir.path(), "minimal", content);

        let registry = PromptRegistry::new(temp_dir.path());
        let prompt = registry.load("minimal").expect("Failed to load prompt");

        assert!(prompt.metadata.tools.is_empty());
    }
}
