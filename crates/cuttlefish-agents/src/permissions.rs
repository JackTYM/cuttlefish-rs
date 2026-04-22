//! Agent role permissions for tool sandboxing.
//!
//! Defines which tools each agent role can use, enforcing separation of concerns:
//! - Planner: Read-only + markdown writes in plans directory
//! - Coder: Full access to all tools
//! - Critic: Read-only access
//! - Explorer: Read-only access
//! - Librarian: Read-only + doc file writes
//! - DevOps: Full access (for infrastructure tasks)
//! - Orchestrator: Read-only (delegates to other agents)

use std::collections::HashSet;

use cuttlefish_core::traits::agent::AgentRole;

use crate::tools::built_in;

/// Extension trait for AgentRole to add permission checking.
pub trait RolePermissions {
    /// Get the allowed tools for this role.
    fn allowed_tools(&self) -> HashSet<&'static str>;

    /// Check if a tool is allowed for this role.
    fn is_tool_allowed(&self, tool_name: &str) -> bool;

    /// Check if a file path is allowed for writing by this role.
    fn is_write_path_allowed(&self, path: &str) -> bool;

    /// Check if a command is allowed for this role.
    fn is_command_allowed(&self, command: &str) -> bool;
}

impl RolePermissions for AgentRole {
    fn allowed_tools(&self) -> HashSet<&'static str> {
        match self {
            Self::Planner | Self::Critic | Self::Explorer | Self::Orchestrator => {
                // Read-only tools
                [
                    built_in::READ_FILE,
                    built_in::LIST_DIRECTORY,
                    built_in::SEARCH_FILES,
                    built_in::GLOB,
                    built_in::GREP,
                    built_in::GIT_STATUS,
                    built_in::GIT_DIFF,
                    built_in::GIT_LOG,
                ]
                .into_iter()
                .collect()
            }
            Self::Librarian => {
                // Read-only + write (for docs)
                [
                    built_in::READ_FILE,
                    built_in::WRITE_FILE,
                    built_in::EDIT_FILE,
                    built_in::EDIT_FILE_REPLACE,
                    built_in::LIST_DIRECTORY,
                    built_in::SEARCH_FILES,
                    built_in::GLOB,
                    built_in::GREP,
                    built_in::GIT_STATUS,
                    built_in::GIT_DIFF,
                    built_in::GIT_LOG,
                ]
                .into_iter()
                .collect()
            }
            Self::Coder | Self::DevOps => {
                // Full access
                [
                    built_in::READ_FILE,
                    built_in::WRITE_FILE,
                    built_in::EDIT_FILE,
                    built_in::EDIT_FILE_REPLACE,
                    built_in::EXECUTE_COMMAND,
                    built_in::LIST_DIRECTORY,
                    built_in::SEARCH_FILES,
                    built_in::GLOB,
                    built_in::GREP,
                    built_in::GIT_STATUS,
                    built_in::GIT_DIFF,
                    built_in::GIT_LOG,
                ]
                .into_iter()
                .collect()
            }
        }
    }

    fn is_tool_allowed(&self, tool_name: &str) -> bool {
        self.allowed_tools().contains(tool_name)
    }

    fn is_write_path_allowed(&self, path: &str) -> bool {
        match self {
            Self::Planner => {
                // Only markdown files in plans directory
                let path_lower = path.to_lowercase();
                (path_lower.contains("/plans/") || path_lower.starts_with("plans/"))
                    && path_lower.ends_with(".md")
            }
            Self::Librarian => {
                // Only documentation files
                let path_lower = path.to_lowercase();
                path_lower.ends_with(".md")
                    || path_lower.contains("/docs/")
                    || path_lower.starts_with("docs/")
                    || path_lower.contains("readme")
            }
            Self::Coder | Self::DevOps => true,
            Self::Critic | Self::Explorer | Self::Orchestrator => false,
        }
    }

    fn is_command_allowed(&self, _command: &str) -> bool {
        matches!(self, Self::Coder | Self::DevOps)
    }
}

/// Result of a permission check.
#[derive(Debug, Clone)]
pub enum PermissionResult {
    /// Action is allowed.
    Allowed,
    /// Tool is not available for this role.
    ToolNotAllowed {
        /// The tool that was denied.
        tool: String,
        /// The role that tried to use it.
        role: AgentRole,
    },
    /// File path is not writable by this role.
    PathNotAllowed {
        /// The path that was denied.
        path: String,
        /// The role that tried to write.
        role: AgentRole,
    },
    /// Command execution is not allowed for this role.
    CommandNotAllowed {
        /// The command that was denied.
        command: String,
        /// The role that tried to execute.
        role: AgentRole,
    },
}

impl PermissionResult {
    /// Check if the result is allowed.
    pub fn is_allowed(&self) -> bool {
        matches!(self, Self::Allowed)
    }

    /// Get an error message if not allowed.
    pub fn error_message(&self) -> Option<String> {
        match self {
            Self::Allowed => None,
            Self::ToolNotAllowed { tool, role } => Some(format!(
                "Tool '{}' is not available for {:?} role. This agent can only use read-only tools.",
                tool, role
            )),
            Self::PathNotAllowed { path, role } => Some(format!(
                "Path '{}' is not writable by {:?} role. {}",
                path,
                role,
                match role {
                    AgentRole::Planner =>
                        "Planner can only write .md files in the plans/ directory.",
                    AgentRole::Librarian =>
                        "Librarian can only write documentation files (.md, docs/, README).",
                    _ => "This agent has read-only access.",
                }
            )),
            Self::CommandNotAllowed { command, role } => Some(format!(
                "Command execution is not allowed for {:?} role. Command: '{}'",
                role,
                if command.len() > 50 {
                    format!("{}...", &command[..50])
                } else {
                    command.clone()
                }
            )),
        }
    }
}

/// Check permissions for a tool call.
pub fn check_tool_permission(
    role: AgentRole,
    tool_name: &str,
    path: Option<&str>,
    command: Option<&str>,
) -> PermissionResult {
    // First check if the tool is allowed
    if !role.is_tool_allowed(tool_name) {
        return PermissionResult::ToolNotAllowed {
            tool: tool_name.to_string(),
            role,
        };
    }

    // For write operations, check path permissions
    if matches!(
        tool_name,
        built_in::WRITE_FILE | built_in::EDIT_FILE | built_in::EDIT_FILE_REPLACE
    ) && let Some(p) = path
        && !role.is_write_path_allowed(p)
    {
        return PermissionResult::PathNotAllowed {
            path: p.to_string(),
            role,
        };
    }

    // For command execution, check command permissions
    if tool_name == built_in::EXECUTE_COMMAND
        && let Some(cmd) = command
        && !role.is_command_allowed(cmd)
    {
        return PermissionResult::CommandNotAllowed {
            command: cmd.to_string(),
            role,
        };
    }

    PermissionResult::Allowed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_planner_read_only() {
        let role = AgentRole::Planner;
        assert!(role.is_tool_allowed(built_in::READ_FILE));
        assert!(role.is_tool_allowed(built_in::GLOB));
        assert!(!role.is_tool_allowed(built_in::WRITE_FILE));
        assert!(!role.is_tool_allowed(built_in::EXECUTE_COMMAND));
    }

    #[test]
    fn test_planner_path_restrictions() {
        let role = AgentRole::Planner;
        assert!(role.is_write_path_allowed("plans/implementation.md"));
        assert!(role.is_write_path_allowed("/workspace/plans/design.md"));
        assert!(!role.is_write_path_allowed("src/main.rs"));
        assert!(!role.is_write_path_allowed("plans/script.py"));
    }

    #[test]
    fn test_coder_full_access() {
        let role = AgentRole::Coder;
        assert!(role.is_tool_allowed(built_in::READ_FILE));
        assert!(role.is_tool_allowed(built_in::WRITE_FILE));
        assert!(role.is_tool_allowed(built_in::EXECUTE_COMMAND));
        assert!(role.is_write_path_allowed("src/main.rs"));
        assert!(role.is_command_allowed("cargo build"));
    }

    #[test]
    fn test_critic_read_only() {
        let role = AgentRole::Critic;
        assert!(role.is_tool_allowed(built_in::READ_FILE));
        assert!(!role.is_tool_allowed(built_in::WRITE_FILE));
        assert!(!role.is_write_path_allowed("any/path.rs"));
    }

    #[test]
    fn test_librarian_docs_only() {
        let role = AgentRole::Librarian;
        assert!(role.is_tool_allowed(built_in::WRITE_FILE));
        assert!(role.is_write_path_allowed("docs/api.md"));
        assert!(role.is_write_path_allowed("README.md"));
        assert!(!role.is_write_path_allowed("src/lib.rs"));
    }

    #[test]
    fn test_check_permission_integration() {
        // Planner trying to write code - tool not allowed
        let result = check_tool_permission(
            AgentRole::Planner,
            built_in::WRITE_FILE,
            Some("src/main.rs"),
            None,
        );
        assert!(!result.is_allowed());

        // Coder writing code - allowed
        let result = check_tool_permission(
            AgentRole::Coder,
            built_in::WRITE_FILE,
            Some("src/main.rs"),
            None,
        );
        assert!(result.is_allowed());

        // Critic trying to execute command - tool not allowed
        let result = check_tool_permission(
            AgentRole::Critic,
            built_in::EXECUTE_COMMAND,
            None,
            Some("rm -rf /"),
        );
        assert!(!result.is_allowed());

        // Librarian writing docs - allowed
        let result = check_tool_permission(
            AgentRole::Librarian,
            built_in::WRITE_FILE,
            Some("docs/api.md"),
            None,
        );
        assert!(result.is_allowed());

        // Librarian writing code - path not allowed
        let result = check_tool_permission(
            AgentRole::Librarian,
            built_in::WRITE_FILE,
            Some("src/lib.rs"),
            None,
        );
        assert!(!result.is_allowed());
    }
}
