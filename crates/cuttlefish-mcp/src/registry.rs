//! MCP tool registry integration.
//!
//! Bridges MCP servers to Cuttlefish's tool system.

use std::collections::HashMap;

use serde_json::Value;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::client::McpClient;
use crate::error::McpError;
use crate::protocol::Tool;

/// A connected MCP server.
pub struct McpServer {
    /// Server name.
    pub name: String,
    /// MCP client.
    client: McpClient,
    /// Cached tools.
    tools: Vec<Tool>,
}

impl McpServer {
    /// Create a new MCP server connection.
    pub fn new(name: String, client: McpClient) -> Self {
        Self {
            name,
            client,
            tools: Vec::new(),
        }
    }

    /// Get the client.
    pub fn client(&self) -> &McpClient {
        &self.client
    }

    /// Get a mutable reference to the client.
    pub fn client_mut(&mut self) -> &mut McpClient {
        &mut self.client
    }

    /// Get cached tools.
    pub fn tools(&self) -> &[Tool] {
        &self.tools
    }

    /// Refresh the tool list from the server.
    pub async fn refresh_tools(&mut self) -> Result<(), McpError> {
        self.tools = self.client.list_tools().await?;
        debug!(
            "Refreshed tools for server '{}': {} tools",
            self.name,
            self.tools.len()
        );
        Ok(())
    }
}

/// Registry of MCP servers and their tools.
pub struct McpRegistry {
    /// Connected servers.
    servers: RwLock<HashMap<String, McpServer>>,
}

impl McpRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self {
            servers: RwLock::new(HashMap::new()),
        }
    }

    /// Register an MCP server.
    ///
    /// The server should already be initialized before registering.
    pub async fn register(&self, name: String, mut server: McpServer) -> Result<(), McpError> {
        // Refresh tools
        server.refresh_tools().await?;

        let tool_count = server.tools.len();
        let mut servers = self.servers.write().await;
        servers.insert(name.clone(), server);

        info!("Registered MCP server '{}' with {} tools", name, tool_count);
        Ok(())
    }

    /// Unregister an MCP server.
    pub async fn unregister(&self, name: &str) -> Option<McpServer> {
        let mut servers = self.servers.write().await;
        let server = servers.remove(name);
        if server.is_some() {
            info!("Unregistered MCP server '{}'", name);
        }
        server
    }

    /// List all registered server names.
    pub async fn server_names(&self) -> Vec<String> {
        let servers = self.servers.read().await;
        servers.keys().cloned().collect()
    }

    /// Get the number of registered servers.
    pub async fn server_count(&self) -> usize {
        let servers = self.servers.read().await;
        servers.len()
    }

    /// List all available tools across all servers.
    ///
    /// Tool names are prefixed with the server name to avoid conflicts.
    pub async fn list_all_tools(&self) -> Vec<McpToolDefinition> {
        let servers = self.servers.read().await;
        let mut all_tools = Vec::new();

        for (server_name, server) in servers.iter() {
            for tool in &server.tools {
                all_tools.push(McpToolDefinition {
                    server_name: server_name.clone(),
                    tool_name: tool.name.clone(),
                    description: tool.description.clone(),
                    input_schema: tool.input_schema.clone(),
                });
            }
        }

        all_tools
    }

    /// Call a tool on a specific server.
    pub async fn call_tool(
        &self,
        server_name: &str,
        tool_name: &str,
        arguments: Option<Value>,
    ) -> Result<String, McpError> {
        let servers = self.servers.read().await;

        let server = servers
            .get(server_name)
            .ok_or_else(|| McpError::Transport(format!("Server '{}' not found", server_name)))?;

        let result = server.client.call_tool(tool_name, arguments).await?;

        // Convert result to string
        let mut output = String::new();
        for content in result.content {
            match content {
                crate::protocol::ToolContent::Text { text } => {
                    output.push_str(&text);
                }
                crate::protocol::ToolContent::Image { data, mime_type } => {
                    output.push_str(&format!("[Image: {} bytes, {}]\n", data.len(), mime_type));
                }
                crate::protocol::ToolContent::Resource { resource } => {
                    if let Some(text) = resource.text {
                        output.push_str(&text);
                    } else {
                        output.push_str(&format!("[Resource: {}]\n", resource.uri));
                    }
                }
            }
        }

        if result.is_error {
            Err(McpError::Server {
                code: -1,
                message: output,
            })
        } else {
            Ok(output)
        }
    }

    /// Call a tool by its full name (server_name/tool_name).
    pub async fn call_tool_by_full_name(
        &self,
        full_name: &str,
        arguments: Option<Value>,
    ) -> Result<String, McpError> {
        let (server_name, tool_name) = full_name
            .split_once('/')
            .ok_or_else(|| McpError::ToolNotFound(format!("Invalid tool name: {}", full_name)))?;

        self.call_tool(server_name, tool_name, arguments).await
    }

    /// Find a tool by name (searches all servers).
    pub async fn find_tool(&self, tool_name: &str) -> Option<(String, Tool)> {
        let servers = self.servers.read().await;

        for (server_name, server) in servers.iter() {
            if let Some(tool) = server.tools.iter().find(|t| t.name == tool_name) {
                return Some((server_name.clone(), tool.clone()));
            }
        }

        None
    }

    /// Refresh tools for all servers.
    pub async fn refresh_all_tools(&self) -> Result<(), McpError> {
        let mut servers = self.servers.write().await;

        for (name, server) in servers.iter_mut() {
            if let Err(e) = server.refresh_tools().await {
                warn!("Failed to refresh tools for server '{}': {}", name, e);
            }
        }

        Ok(())
    }

    /// Close all server connections.
    pub async fn close_all(&self) -> Result<(), McpError> {
        let servers = self.servers.read().await;

        for (name, server) in servers.iter() {
            if let Err(e) = server.client.close().await {
                warn!("Failed to close server '{}': {}", name, e);
            }
        }

        Ok(())
    }
}

impl Default for McpRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// A tool definition from an MCP server.
#[derive(Debug, Clone)]
pub struct McpToolDefinition {
    /// Server that provides this tool.
    pub server_name: String,
    /// Tool name.
    pub tool_name: String,
    /// Tool description.
    pub description: Option<String>,
    /// Input schema (JSON Schema).
    pub input_schema: Value,
}

impl McpToolDefinition {
    /// Get the full tool name (server_name/tool_name).
    pub fn full_name(&self) -> String {
        format!("{}/{}", self.server_name, self.tool_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_registry_creation() {
        let registry = McpRegistry::new();
        assert_eq!(registry.server_count().await, 0);
    }

    #[test]
    fn test_tool_definition_full_name() {
        let tool = McpToolDefinition {
            server_name: "filesystem".to_string(),
            tool_name: "read_file".to_string(),
            description: Some("Read a file".to_string()),
            input_schema: serde_json::json!({}),
        };
        assert_eq!(tool.full_name(), "filesystem/read_file");
    }
}
