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
    /// Execute shell command.
    pub const EXECUTE_COMMAND: &str = "execute_command";
    /// Search files by pattern.
    pub const SEARCH_FILES: &str = "search_files";
    /// List directory contents.
    pub const LIST_DIRECTORY: &str = "list_directory";
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

    /// Create a registry with all 5 default agent tools.
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
    fn test_defaults_has_5_tools() {
        let r = ToolRegistry::with_defaults();
        assert_eq!(r.all_definitions().len(), 5);
        assert!(r.get(built_in::READ_FILE).is_some());
        assert!(r.get(built_in::WRITE_FILE).is_some());
        assert!(r.get(built_in::EXECUTE_COMMAND).is_some());
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
        assert_eq!(r.all_definitions().len(), 5);
    }
}
