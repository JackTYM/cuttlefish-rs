//! Tool registry and built-in tool definitions for agents.

use std::collections::HashMap;

/// Definition of a tool an agent can invoke.
#[derive(Debug, Clone)]
pub struct ToolDefinition {
    /// Tool name as called by the model.
    pub name: String,
    /// Description shown to the model.
    pub description: String,
    /// JSON Schema for input parameters.
    pub input_schema: serde_json::Value,
}

/// Built-in tool name constants.
pub mod built_in {
    /// Read file contents.
    pub const READ_FILE: &str = "read_file";
    /// Write file contents.
    pub const WRITE_FILE: &str = "write_file";
    /// Edit file using line hashes (Hashline).
    pub const EDIT_FILE: &str = "edit_file";
    /// Edit file using old_string -> new_string replacement (surgical).
    pub const EDIT_FILE_REPLACE: &str = "edit_file_replace";
    /// Execute shell command.
    pub const EXECUTE_COMMAND: &str = "execute_command";
    /// Search files by pattern.
    pub const SEARCH_FILES: &str = "search_files";
    /// List directory contents.
    pub const LIST_DIRECTORY: &str = "list_directory";
    /// Fast file pattern matching (glob).
    pub const GLOB: &str = "glob";
    /// Content search with regex (grep).
    pub const GREP: &str = "grep";
    /// Git log.
    pub const GIT_LOG: &str = "git_log";
    /// Git diff.
    pub const GIT_DIFF: &str = "git_diff";
    /// Git status.
    pub const GIT_STATUS: &str = "git_status";
}

/// Registry of tools available to agents.
pub struct ToolRegistry {
    tools: HashMap<String, ToolDefinition>,
}

impl ToolRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    /// Create a registry with all 6 default agent tools.
    pub fn with_defaults() -> Self {
        let mut r = Self::new();
        r.register(ToolDefinition {
            name: built_in::READ_FILE.to_string(),
            description: "Read a file from the workspace".to_string(),
            input_schema: serde_json::json!({"type":"object","properties":{"path":{"type":"string"}},"required":["path"]}),
        });
        r.register(ToolDefinition {
            name: built_in::WRITE_FILE.to_string(),
            description: "Write content to a file in the workspace".to_string(),
            input_schema: serde_json::json!({"type":"object","properties":{"path":{"type":"string"},"content":{"type":"string"}},"required":["path","content"]}),
        });
        r.register(ToolDefinition {
            name: built_in::EDIT_FILE.to_string(),
            description: "Edit specific lines in a file using line hashes. Use read_file first to get hashes.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {"type": "string", "description": "Path to the file to edit"},
                    "edits": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "hash": {"type": "string", "description": "5-char hash of the line to edit"},
                                "new_content": {"type": "string", "description": "New content for the line (omit to delete)"}
                            },
                            "required": ["hash"]
                        },
                        "description": "List of edits to apply"
                    }
                },
                "required": ["path", "edits"]
            }),
        });
        r.register(ToolDefinition {
            name: built_in::EXECUTE_COMMAND.to_string(),
            description: "Execute a shell command in the sandbox".to_string(),
            input_schema: serde_json::json!({"type":"object","properties":{"command":{"type":"string"},"timeout_secs":{"type":"integer"}},"required":["command"]}),
        });
        r.register(ToolDefinition {
            name: built_in::SEARCH_FILES.to_string(),
            description: "Search for files matching a pattern".to_string(),
            input_schema: serde_json::json!({"type":"object","properties":{"pattern":{"type":"string"},"directory":{"type":"string"}},"required":["pattern"]}),
        });
        r.register(ToolDefinition {
            name: built_in::LIST_DIRECTORY.to_string(),
            description: "List files in a directory".to_string(),
            input_schema: serde_json::json!({"type":"object","properties":{"path":{"type":"string"}},"required":["path"]}),
        });
        r.register(ToolDefinition {
            name: built_in::GLOB.to_string(),
            description: "Fast file pattern matching. Returns files matching a glob pattern (e.g., '**/*.rs', 'src/**/*.ts'). Results sorted by modification time (most recent first).".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "pattern": {"type": "string", "description": "Glob pattern to match (e.g., '**/*.rs', 'src/*.py')"},
                    "path": {"type": "string", "description": "Root directory to search from (default: current dir)"}
                },
                "required": ["pattern"]
            }),
        });
        r.register(ToolDefinition {
            name: built_in::GREP.to_string(),
            description: "Search file contents using regex patterns. Returns matching lines with context.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "pattern": {"type": "string", "description": "Regex pattern to search for"},
                    "path": {"type": "string", "description": "File or directory to search in (default: current dir)"},
                    "context": {"type": "integer", "description": "Lines of context before/after match (default: 0)"},
                    "glob": {"type": "string", "description": "Only search files matching this glob (e.g., '*.rs')"},
                    "max_results": {"type": "integer", "description": "Maximum matches to return (default: 100)"}
                },
                "required": ["pattern"]
            }),
        });
        r.register(ToolDefinition {
            name: built_in::EDIT_FILE_REPLACE.to_string(),
            description: "Surgical file edit using string replacement. More token-efficient than write_file for small changes. The old_string must match exactly (including whitespace).".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {"type": "string", "description": "Path to the file to edit"},
                    "old_string": {"type": "string", "description": "Exact string to find and replace"},
                    "new_string": {"type": "string", "description": "Replacement string"},
                    "replace_all": {"type": "boolean", "description": "Replace all occurrences (default: false, replaces first only)"}
                },
                "required": ["path", "old_string", "new_string"]
            }),
        });
        r.register(ToolDefinition {
            name: built_in::GIT_STATUS.to_string(),
            description: "Get git repository status (modified, staged, untracked files).".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {"type": "string", "description": "Repository path (default: current dir)"}
                }
            }),
        });
        r.register(ToolDefinition {
            name: built_in::GIT_DIFF.to_string(),
            description: "Get git diff (unstaged changes, or between commits/branches).".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {"type": "string", "description": "Repository path (default: current dir)"},
                    "staged": {"type": "boolean", "description": "Show staged changes (default: false)"},
                    "commit": {"type": "string", "description": "Compare against specific commit/branch"}
                }
            }),
        });
        r.register(ToolDefinition {
            name: built_in::GIT_LOG.to_string(),
            description: "Get git commit history.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {"type": "string", "description": "Repository path (default: current dir)"},
                    "max_count": {"type": "integer", "description": "Maximum commits to return (default: 10)"},
                    "since": {"type": "string", "description": "Show commits after date (e.g., '2024-01-01')"},
                    "author": {"type": "string", "description": "Filter by author name/email"}
                }
            }),
        });
        r
    }

    /// Register a tool definition.
    pub fn register(&mut self, tool: ToolDefinition) {
        self.tools.insert(tool.name.clone(), tool);
    }

    /// Get a tool by name.
    pub fn get(&self, name: &str) -> Option<&ToolDefinition> {
        self.tools.get(name)
    }

    /// Get all tool definitions.
    pub fn all_definitions(&self) -> Vec<&ToolDefinition> {
        self.tools.values().collect()
    }

    /// List all registered tool names.
    pub fn tool_names(&self) -> Vec<&str> {
        self.tools.keys().map(|s| s.as_str()).collect()
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::with_defaults()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults_has_all_tools() {
        let r = ToolRegistry::with_defaults();
        // 6 original + 6 new = 12 tools
        assert_eq!(r.all_definitions().len(), 12);
        assert!(r.get(built_in::READ_FILE).is_some());
        assert!(r.get(built_in::WRITE_FILE).is_some());
        assert!(r.get(built_in::EDIT_FILE).is_some());
        assert!(r.get(built_in::EXECUTE_COMMAND).is_some());
        assert!(r.get(built_in::GLOB).is_some());
        assert!(r.get(built_in::GREP).is_some());
        assert!(r.get(built_in::EDIT_FILE_REPLACE).is_some());
        assert!(r.get(built_in::GIT_STATUS).is_some());
        assert!(r.get(built_in::GIT_DIFF).is_some());
        assert!(r.get(built_in::GIT_LOG).is_some());
    }

    #[test]
    fn test_register_custom() {
        let mut r = ToolRegistry::new();
        r.register(ToolDefinition {
            name: "my_tool".to_string(),
            description: "test".to_string(),
            input_schema: serde_json::json!({}),
        });
        assert!(r.get("my_tool").is_some());
    }

    #[test]
    fn test_all_definitions_count() {
        let r = ToolRegistry::with_defaults();
        assert_eq!(r.all_definitions().len(), 12);
    }
}
