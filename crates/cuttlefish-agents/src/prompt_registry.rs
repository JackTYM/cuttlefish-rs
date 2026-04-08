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
//!
//! # System Template Integration
//!
//! The registry supports a `system.md` template that uses the `PromptTemplate` engine
//! for placeholder replacement and section overrides. Use `load_with_context()` to
//! compose prompts with runtime environment values.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::RwLock;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::prompt_template::{PromptContext, PromptTemplate};

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
///
/// The registry supports a "system" prompt that serves as a base for all agents.
/// When using `load_with_system`, the system prompt body is prepended to the
/// agent-specific prompt.
pub struct PromptRegistry {
    prompts_dir: PathBuf,
    cache: RwLock<HashMap<String, AgentPrompt>>,
}

/// The name of the system base prompt file (system.md).
pub const SYSTEM_PROMPT_NAME: &str = "system";

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

    /// Loads an agent prompt with the system base prompt prepended.
    ///
    /// This composes the full prompt by:
    /// 1. Loading the system base prompt (system.md) as a template
    /// 2. Rendering it with default runtime context (working dir, platform, etc.)
    /// 3. Loading the agent-specific prompt
    /// 4. Combining them: rendered system + agent body
    ///
    /// # Arguments
    ///
    /// * `agent_name` - The name of the agent (e.g., "orchestrator").
    ///
    /// # Returns
    ///
    /// The composed `AgentPrompt` with system + agent content, or the agent
    /// prompt alone if the system prompt is not found.
    pub fn load_with_system(&self, agent_name: &str) -> Result<AgentPrompt, PromptError> {
        // Use default context with current working directory
        let ctx = Self::create_default_context(&std::env::current_dir().unwrap_or_default());
        self.load_with_system_and_context(agent_name, &ctx)
    }

    /// Loads an agent prompt with the system base prompt and custom context.
    ///
    /// This is the full-featured version that allows customizing placeholder values.
    ///
    /// # Arguments
    ///
    /// * `agent_name` - The name of the agent (e.g., "orchestrator").
    /// * `ctx` - The context containing placeholder values and section overrides.
    ///
    /// # Returns
    ///
    /// The composed `AgentPrompt` with rendered system + agent content.
    pub fn load_with_system_and_context(
        &self,
        agent_name: &str,
        ctx: &PromptContext,
    ) -> Result<AgentPrompt, PromptError> {
        let agent_prompt = self.load(agent_name)?;

        // Try to load and render system.md with the template engine
        let system_path = self.prompts_dir.join("system.md");
        let rendered_system = if system_path.exists() {
            let system_content = std::fs::read_to_string(&system_path)?;
            let template = PromptTemplate::new(system_content);
            template.render(ctx)
        } else {
            // No system.md, return agent prompt as-is
            return Ok(agent_prompt);
        };

        // Compose: rendered system + separator + agent body
        let composed_body = format!(
            "{}\n\n---\n\n# Agent: {}\n\n{}",
            rendered_system, agent_prompt.metadata.name, agent_prompt.body
        );

        Ok(AgentPrompt {
            metadata: agent_prompt.metadata,
            body: composed_body,
        })
    }

    /// Loads just the system base prompt.
    ///
    /// This is useful when you want to build custom prompts that include
    /// the system foundation but with different agent-specific content.
    pub fn load_system(&self) -> Result<AgentPrompt, PromptError> {
        self.load(SYSTEM_PROMPT_NAME)
    }

    /// Composes a custom prompt with the system base.
    ///
    /// # Arguments
    ///
    /// * `custom_content` - Custom prompt content to append after system.
    /// * `agent_name` - Name for the agent section header.
    ///
    /// # Returns
    ///
    /// A string containing the system prompt followed by the custom content.
    pub fn compose_with_system(
        &self,
        custom_content: &str,
        agent_name: &str,
    ) -> Result<String, PromptError> {
        let system_prompt = self.load_system()?;

        Ok(format!(
            "{}\n\n---\n\n# Agent: {}\n\n{}",
            system_prompt.body, agent_name, custom_content
        ))
    }

    /// Loads an agent prompt with the system template rendered using the given context.
    ///
    /// This is the recommended method for loading prompts in production. It:
    /// 1. Loads `system.md` as a `PromptTemplate`
    /// 2. Renders it with the provided `PromptContext` (placeholder replacement, sections)
    /// 3. Appends the agent-specific prompt body
    ///
    /// # Arguments
    ///
    /// * `agent_name` - The name of the agent (e.g., "orchestrator").
    /// * `ctx` - The context containing placeholder values and section overrides.
    ///
    /// # Returns
    ///
    /// The composed prompt string with all placeholders replaced.
    pub fn load_with_context(
        &self,
        agent_name: &str,
        ctx: &PromptContext,
    ) -> Result<String, PromptError> {
        let agent_prompt = self.load(agent_name)?;

        // Try to load and render system.md with the template engine
        let system_path = self.prompts_dir.join("system.md");
        let rendered_system = if system_path.exists() {
            let system_content = std::fs::read_to_string(&system_path)?;
            let template = PromptTemplate::new(system_content);
            template.render(ctx)
        } else {
            // No system.md, return empty string
            String::new()
        };

        // Compose: rendered system + separator + agent body
        if rendered_system.is_empty() {
            Ok(agent_prompt.body)
        } else {
            Ok(format!(
                "{}\n\n---\n\n# Agent: {}\n\n{}",
                rendered_system, agent_prompt.metadata.name, agent_prompt.body
            ))
        }
    }

    /// Creates a default `PromptContext` with common runtime values.
    ///
    /// This populates placeholders like `%WORKING_DIR%`, `%PLATFORM%`, etc.
    pub fn create_default_context(working_dir: &Path) -> PromptContext {
        PromptContext::new()
            .working_dir(working_dir.display().to_string())
            .platform(std::env::consts::OS)
            .set(
                "SHELL",
                std::env::var("SHELL").unwrap_or_else(|_| "bash".to_string()),
            )
            .set("OS_VERSION", std::env::consts::OS)
            .set("IS_GIT", "true") // Will be overridden by caller if needed
            .set("ADDITIONAL_DIRS", "")
            .set("MODEL_ID", "unknown")
            .set("MODEL_MARKETING_NAME", "AI Assistant")
            .set("KNOWLEDGE_CUTOFF", "2024")
            .set("LANGUAGE", "en")
            .set("MCP_INSTRUCTIONS", "")
            .set("MEMORY", "")
            .set("CUSTOM_INSTRUCTIONS", "")
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

    #[test]
    fn test_load_with_system_renders_template() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        // Create system.md with sections and placeholders
        let system_content = r#"---
name: system
description: Base system prompt
version: 1.0.0
---

<!-- SECTION:identity -->
You are an AI assistant running on %PLATFORM%.
Working directory: %WORKING_DIR%
<!-- /SECTION:identity -->

<!-- SECTION:rules -->
Follow these rules.
<!-- /SECTION:rules -->
"#;
        fs::write(temp_dir.path().join("system.md"), system_content).expect("write system.md");

        // Create agent prompt
        let agent_content = r#"---
name: test_agent
description: Test agent
tools: []
category: deep
---

You are the test agent."#;
        create_test_prompt(temp_dir.path(), "test_agent", agent_content);

        let registry = PromptRegistry::new(temp_dir.path());
        let prompt = registry
            .load_with_system("test_agent")
            .expect("Failed to load prompt");

        // Verify system content was included
        assert!(prompt.body.contains("You are an AI assistant"));
        // Verify placeholders were replaced
        assert!(!prompt.body.contains("%PLATFORM%"));
        assert!(!prompt.body.contains("%WORKING_DIR%"));
        // Verify section markers were removed
        assert!(!prompt.body.contains("<!-- SECTION:"));
        // Verify agent content was appended
        assert!(prompt.body.contains("You are the test agent"));
        // Verify agent header was added
        assert!(prompt.body.contains("# Agent: test_agent"));
    }

    #[test]
    fn test_load_with_system_and_context_custom_placeholders() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        let system_content = r#"---
name: system
description: Base system prompt
version: 1.0.0
---

Project: %PROJECT_NAME%
Model: %MODEL_MARKETING_NAME%
"#;
        fs::write(temp_dir.path().join("system.md"), system_content).expect("write system.md");

        let agent_content = r#"---
name: coder
description: Coder
tools: []
category: deep
---

Code here."#;
        create_test_prompt(temp_dir.path(), "coder", agent_content);

        let registry = PromptRegistry::new(temp_dir.path());
        let ctx = PromptContext::new()
            .set("PROJECT_NAME", "MyProject")
            .set("MODEL_MARKETING_NAME", "Claude");

        let prompt = registry
            .load_with_system_and_context("coder", &ctx)
            .expect("load");

        assert!(prompt.body.contains("Project: MyProject"));
        assert!(prompt.body.contains("Model: Claude"));
    }

    #[test]
    fn test_load_with_system_no_system_file() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        // Only create agent prompt, no system.md
        let agent_content = r#"---
name: standalone
description: Standalone agent
tools: []
category: deep
---

Standalone content."#;
        create_test_prompt(temp_dir.path(), "standalone", agent_content);

        let registry = PromptRegistry::new(temp_dir.path());
        let prompt = registry
            .load_with_system("standalone")
            .expect("Failed to load prompt");

        // Should just return agent body without system prefix
        assert_eq!(prompt.body, "Standalone content.");
    }

    #[test]
    fn test_create_default_context() {
        let ctx = PromptRegistry::create_default_context(std::path::Path::new("/test/dir"));

        // Verify key placeholders are set
        assert!(ctx.placeholders.contains_key("WORKING_DIR"));
        assert!(ctx.placeholders.contains_key("PLATFORM"));
        assert!(ctx.placeholders.contains_key("SHELL"));

        assert_eq!(
            ctx.placeholders.get("WORKING_DIR"),
            Some(&"/test/dir".to_string())
        );
    }
}
