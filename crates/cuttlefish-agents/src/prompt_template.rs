//! Prompt template engine with placeholder replacement and section overrides.
//!
//! This module provides a flexible template system for system prompts that supports:
//! - Placeholder replacement (e.g., `%AGENT_NAME%` -> "Orchestrator")
//! - Section-based overrides for marketplace plugins
//! - Runtime composition of prompts from multiple sources
//!
//! # Template Format
//!
//! Templates use `%PLACEHOLDER%` syntax for replacements and HTML-style comments
//! for section markers:
//!
//! ```text
//! <!-- SECTION:identity -->
//! You are %AGENT_NAME%, an agent for Cuttlefish.
//! <!-- /SECTION:identity -->
//! ```
//!
//! Sections can be replaced entirely by marketplace plugins or skills.

use std::collections::HashMap;

use regex::Regex;

/// A prompt template with placeholder and section support.
#[derive(Debug, Clone)]
pub struct PromptTemplate {
    /// The raw template content.
    content: String,
    /// Extracted sections from the template.
    sections: HashMap<String, String>,
}

/// Configuration for rendering a prompt template.
#[derive(Debug, Clone, Default)]
pub struct PromptContext {
    /// Placeholder values (e.g., "AGENT_NAME" -> "Orchestrator").
    pub placeholders: HashMap<String, String>,
    /// Section overrides from plugins/marketplace.
    pub section_overrides: HashMap<String, String>,
    /// Sections to exclude from output.
    pub excluded_sections: Vec<String>,
}

impl PromptContext {
    /// Creates a new empty context.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets a placeholder value.
    pub fn set(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.placeholders.insert(key.into(), value.into());
        self
    }

    /// Overrides an entire section with custom content.
    pub fn override_section(
        mut self,
        section: impl Into<String>,
        content: impl Into<String>,
    ) -> Self {
        self.section_overrides
            .insert(section.into(), content.into());
        self
    }

    /// Excludes a section from the output.
    pub fn exclude_section(mut self, section: impl Into<String>) -> Self {
        self.excluded_sections.push(section.into());
        self
    }

    /// Sets the agent name placeholder.
    pub fn agent_name(self, name: impl Into<String>) -> Self {
        self.set("AGENT_NAME", name)
    }

    /// Sets the agent description placeholder.
    pub fn agent_description(self, desc: impl Into<String>) -> Self {
        self.set("AGENT_DESCRIPTION", desc)
    }

    /// Sets the tool list placeholder.
    pub fn tools(self, tools: &[&str]) -> Self {
        self.set("TOOL_LIST", tools.join(", "))
    }

    /// Sets the working directory placeholder.
    pub fn working_dir(self, dir: impl Into<String>) -> Self {
        self.set("WORKING_DIR", dir)
    }

    /// Sets the platform placeholder.
    pub fn platform(self, platform: impl Into<String>) -> Self {
        self.set("PLATFORM", platform)
    }

    /// Sets the datetime placeholder.
    pub fn datetime(self, dt: impl Into<String>) -> Self {
        self.set("DATETIME", dt)
    }

    /// Sets the project name placeholder.
    pub fn project_name(self, name: impl Into<String>) -> Self {
        self.set("PROJECT_NAME", name)
    }

    /// Sets custom instructions to append.
    pub fn custom_instructions(self, instructions: impl Into<String>) -> Self {
        self.set("CUSTOM_INSTRUCTIONS", instructions)
    }
}

impl PromptTemplate {
    /// Creates a new template from raw content.
    pub fn new(content: impl Into<String>) -> Self {
        let content = content.into();
        let sections = Self::extract_sections(&content);
        Self { content, sections }
    }

    /// Extracts all sections from the template.
    fn extract_sections(content: &str) -> HashMap<String, String> {
        let mut sections = HashMap::new();

        // Find all section start markers
        let start_re = Regex::new(r"<!--\s*SECTION:(\w+)\s*-->").expect("Invalid regex");

        for cap in start_re.captures_iter(content) {
            let name = cap.get(1).map(|m| m.as_str()).unwrap_or_default();
            let start_match = cap.get(0).expect("Match should exist");
            let content_start = start_match.end();

            // Find the corresponding end marker
            let end_pattern = format!(r"<!--\s*/SECTION:{}\s*-->", regex::escape(name));
            if let Ok(end_re) = Regex::new(&end_pattern)
                && let Some(end_match) = end_re.find(&content[content_start..])
            {
                let body = content[content_start..content_start + end_match.start()]
                    .trim()
                    .to_string();
                sections.insert(name.to_string(), body);
            }
        }

        sections
    }

    /// Returns the names of all sections in the template.
    pub fn section_names(&self) -> Vec<&str> {
        self.sections.keys().map(|s| s.as_str()).collect()
    }

    /// Gets the content of a specific section.
    pub fn get_section(&self, name: &str) -> Option<&str> {
        self.sections.get(name).map(|s| s.as_str())
    }

    /// Renders the template with the given context.
    ///
    /// This performs:
    /// 1. Section replacements/exclusions
    /// 2. Placeholder substitutions
    /// 3. Cleanup of empty lines and markers
    pub fn render(&self, ctx: &PromptContext) -> String {
        let mut result = self.content.clone();

        // Replace entire sections if overridden
        for (section_name, override_content) in &ctx.section_overrides {
            let pattern = format!(
                r"(?s)<!--\s*SECTION:{}\s*-->.*?<!--\s*/SECTION:{}\s*-->",
                regex::escape(section_name),
                regex::escape(section_name)
            );
            if let Ok(re) = Regex::new(&pattern) {
                let replacement = format!(
                    "<!-- SECTION:{} -->\n{}\n<!-- /SECTION:{} -->",
                    section_name, override_content, section_name
                );
                result = re.replace_all(&result, replacement.as_str()).to_string();
            }
        }

        // Remove excluded sections
        for section_name in &ctx.excluded_sections {
            let pattern = format!(
                r"(?s)<!--\s*SECTION:{}\s*-->.*?<!--\s*/SECTION:{}\s*-->",
                regex::escape(section_name),
                regex::escape(section_name)
            );
            if let Ok(re) = Regex::new(&pattern) {
                result = re.replace_all(&result, "").to_string();
            }
        }

        // Replace placeholders
        for (key, value) in &ctx.placeholders {
            let placeholder = format!("%{}%", key);
            result = result.replace(&placeholder, value);
        }

        // Remove any unreplaced placeholders (set to empty)
        let placeholder_re = Regex::new(r"%[A-Z_]+%").expect("Invalid regex");
        result = placeholder_re.replace_all(&result, "").to_string();

        // Remove section markers from final output
        let marker_re = Regex::new(r"<!--\s*/?SECTION:\w+\s*-->").expect("Invalid regex");
        result = marker_re.replace_all(&result, "").to_string();

        // Remove YAML frontmatter
        if result.trim().starts_with("---")
            && let Some(end) = result[3..].find("\n---")
        {
            result = result[end + 7..].to_string();
        }

        // Clean up excessive newlines
        let newline_re = Regex::new(r"\n{3,}").expect("Invalid regex");
        result = newline_re.replace_all(&result, "\n\n").to_string();

        result.trim().to_string()
    }

    /// Renders only specific sections in order.
    pub fn render_sections(&self, section_names: &[&str], ctx: &PromptContext) -> String {
        let mut parts = Vec::new();

        for name in section_names {
            // Check for override first
            if let Some(override_content) = ctx.section_overrides.get(*name) {
                parts.push(Self::replace_placeholders(
                    override_content,
                    &ctx.placeholders,
                ));
            } else if let Some(section) = self.sections.get(*name)
                && !ctx.excluded_sections.contains(&name.to_string())
            {
                parts.push(Self::replace_placeholders(section, &ctx.placeholders));
            }
        }

        parts.join("\n\n")
    }

    /// Replaces placeholders in a string.
    fn replace_placeholders(content: &str, placeholders: &HashMap<String, String>) -> String {
        let mut result = content.to_string();
        for (key, value) in placeholders {
            let placeholder = format!("%{}%", key);
            result = result.replace(&placeholder, value);
        }
        // Remove unreplaced placeholders
        let placeholder_re = Regex::new(r"%[A-Z_]+%").expect("Invalid regex");
        placeholder_re.replace_all(&result, "").to_string()
    }
}

/// Loads the default system prompt template from the prompts directory.
pub fn load_system_template(prompts_dir: &std::path::Path) -> std::io::Result<PromptTemplate> {
    let path = prompts_dir.join("system.md");
    let content = std::fs::read_to_string(path)?;
    Ok(PromptTemplate::new(content))
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_TEMPLATE: &str = r#"---
name: test
---

<!-- SECTION:identity -->
You are %AGENT_NAME%. %AGENT_DESCRIPTION%
<!-- /SECTION:identity -->

<!-- SECTION:tools -->
Available tools: %TOOL_LIST%
<!-- /SECTION:tools -->

<!-- SECTION:custom -->
%CUSTOM_INSTRUCTIONS%
<!-- /SECTION:custom -->
"#;

    #[test]
    fn test_extract_sections() {
        let template = PromptTemplate::new(TEST_TEMPLATE);
        assert!(template.sections.contains_key("identity"));
        assert!(template.sections.contains_key("tools"));
        assert!(template.sections.contains_key("custom"));
    }

    #[test]
    fn test_placeholder_replacement() {
        let template = PromptTemplate::new(TEST_TEMPLATE);
        let ctx = PromptContext::new()
            .agent_name("TestAgent")
            .agent_description("A test agent.")
            .tools(&["Read", "Write", "Bash"]);

        let result = template.render(&ctx);
        assert!(result.contains("You are TestAgent"));
        assert!(result.contains("A test agent"));
        assert!(result.contains("Read, Write, Bash"));
    }

    #[test]
    fn test_section_override() {
        let template = PromptTemplate::new(TEST_TEMPLATE);
        let ctx = PromptContext::new()
            .agent_name("TestAgent")
            .override_section("identity", "CUSTOM IDENTITY SECTION");

        let result = template.render(&ctx);
        assert!(result.contains("CUSTOM IDENTITY SECTION"));
        assert!(!result.contains("You are TestAgent"));
    }

    #[test]
    fn test_section_exclusion() {
        let template = PromptTemplate::new(TEST_TEMPLATE);
        let ctx = PromptContext::new()
            .agent_name("TestAgent")
            .exclude_section("tools");

        let result = template.render(&ctx);
        assert!(!result.contains("Available tools"));
    }

    #[test]
    fn test_render_specific_sections() {
        let template = PromptTemplate::new(TEST_TEMPLATE);
        let ctx = PromptContext::new()
            .agent_name("TestAgent")
            .agent_description("Desc");

        let result = template.render_sections(&["identity"], &ctx);
        assert!(result.contains("You are TestAgent"));
        assert!(!result.contains("Available tools"));
    }

    #[test]
    fn test_unreplaced_placeholders_removed() {
        let template = PromptTemplate::new("Hello %NAME%, your ID is %ID%.");
        let ctx = PromptContext::new().set("NAME", "Alice");

        let result = template.render(&ctx);
        assert_eq!(result, "Hello Alice, your ID is .");
    }

    #[test]
    fn test_section_names() {
        let template = PromptTemplate::new(TEST_TEMPLATE);
        let names = template.section_names();
        assert!(names.contains(&"identity"));
        assert!(names.contains(&"tools"));
        assert!(names.contains(&"custom"));
    }
}
