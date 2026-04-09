//! MCP client for connecting to MCP servers.

use std::sync::Arc;
use std::sync::atomic::{AtomicI64, Ordering};

use serde_json::Value;
use tracing::{debug, info};

use crate::error::McpError;
use crate::protocol::{
    CallToolParams, CallToolResult, ClientCapabilities, ClientInfo, GetPromptParams,
    GetPromptResult, InitializeParams, InitializeResult, JsonRpcNotification, JsonRpcRequest,
    Prompt, PromptsListResult, ReadResourceParams, ReadResourceResult, RequestId, Resource,
    ResourcesListResult, ServerCapabilities, Tool, ToolsListResult, methods,
};
use crate::transport::McpTransport;

/// MCP protocol version.
pub const PROTOCOL_VERSION: &str = "2024-11-05";

/// MCP client for interacting with an MCP server.
pub struct McpClient {
    /// Transport layer.
    transport: Arc<dyn McpTransport>,
    /// Request ID counter.
    request_id: AtomicI64,
    /// Server capabilities (set after initialization).
    server_capabilities: Option<ServerCapabilities>,
    /// Server info (set after initialization).
    server_info: Option<crate::protocol::ServerInfo>,
    /// Whether the client has been initialized.
    initialized: bool,
}

impl McpClient {
    /// Create a new MCP client with the given transport.
    pub fn new(transport: impl McpTransport + 'static) -> Self {
        Self {
            transport: Arc::new(transport),
            request_id: AtomicI64::new(1),
            server_capabilities: None,
            server_info: None,
            initialized: false,
        }
    }

    /// Initialize the connection with the MCP server.
    pub async fn initialize(&mut self) -> Result<InitializeResult, McpError> {
        let params = InitializeParams {
            protocol_version: PROTOCOL_VERSION.to_string(),
            capabilities: ClientCapabilities::default(),
            client_info: ClientInfo {
                name: "cuttlefish".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
        };

        let request = self.make_request(
            methods::INITIALIZE,
            Some(serde_json::to_value(&params).expect("serialize params")),
        );

        let response = self.transport.request(request).await?;
        let result = self.parse_response::<InitializeResult>(response)?;

        // Store server capabilities
        self.server_capabilities = Some(result.capabilities.clone());
        self.server_info = Some(result.server_info.clone());

        // Send initialized notification
        let notification = JsonRpcNotification {
            jsonrpc: "2.0".to_string(),
            method: methods::INITIALIZED.to_string(),
            params: None,
        };
        self.transport.notify(notification).await?;

        self.initialized = true;

        info!(
            "MCP client initialized with server: {} v{}",
            result.server_info.name, result.server_info.version
        );

        Ok(result)
    }

    /// Check if the client has been initialized.
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Get server capabilities.
    pub fn server_capabilities(&self) -> Option<&ServerCapabilities> {
        self.server_capabilities.as_ref()
    }

    /// Get server info.
    pub fn server_info(&self) -> Option<&crate::protocol::ServerInfo> {
        self.server_info.as_ref()
    }

    /// List available tools.
    pub async fn list_tools(&self) -> Result<Vec<Tool>, McpError> {
        self.ensure_initialized()?;

        let request = self.make_request(methods::TOOLS_LIST, None);
        let response = self.transport.request(request).await?;
        let result = self.parse_response::<ToolsListResult>(response)?;

        debug!("Listed {} tools", result.tools.len());
        Ok(result.tools)
    }

    /// Call a tool.
    pub async fn call_tool(
        &self,
        name: &str,
        arguments: Option<Value>,
    ) -> Result<CallToolResult, McpError> {
        self.ensure_initialized()?;

        let params = CallToolParams {
            name: name.to_string(),
            arguments,
        };

        let request = self.make_request(
            methods::TOOLS_CALL,
            Some(serde_json::to_value(&params).expect("serialize params")),
        );

        let response = self.transport.request(request).await?;
        let result = self.parse_response::<CallToolResult>(response)?;

        debug!("Called tool '{}', is_error: {}", name, result.is_error);
        Ok(result)
    }

    /// List available resources.
    pub async fn list_resources(&self) -> Result<Vec<Resource>, McpError> {
        self.ensure_initialized()?;

        let request = self.make_request(methods::RESOURCES_LIST, None);
        let response = self.transport.request(request).await?;
        let result = self.parse_response::<ResourcesListResult>(response)?;

        debug!("Listed {} resources", result.resources.len());
        Ok(result.resources)
    }

    /// Read a resource.
    pub async fn read_resource(&self, uri: &str) -> Result<ReadResourceResult, McpError> {
        self.ensure_initialized()?;

        let params = ReadResourceParams {
            uri: uri.to_string(),
        };

        let request = self.make_request(
            methods::RESOURCES_READ,
            Some(serde_json::to_value(&params).expect("serialize params")),
        );

        let response = self.transport.request(request).await?;
        let result = self.parse_response::<ReadResourceResult>(response)?;

        debug!(
            "Read resource '{}', {} contents",
            uri,
            result.contents.len()
        );
        Ok(result)
    }

    /// List available prompts.
    pub async fn list_prompts(&self) -> Result<Vec<Prompt>, McpError> {
        self.ensure_initialized()?;

        let request = self.make_request(methods::PROMPTS_LIST, None);
        let response = self.transport.request(request).await?;
        let result = self.parse_response::<PromptsListResult>(response)?;

        debug!("Listed {} prompts", result.prompts.len());
        Ok(result.prompts)
    }

    /// Get a prompt.
    pub async fn get_prompt(
        &self,
        name: &str,
        arguments: Option<Value>,
    ) -> Result<GetPromptResult, McpError> {
        self.ensure_initialized()?;

        let params = GetPromptParams {
            name: name.to_string(),
            arguments,
        };

        let request = self.make_request(
            methods::PROMPTS_GET,
            Some(serde_json::to_value(&params).expect("serialize params")),
        );

        let response = self.transport.request(request).await?;
        let result = self.parse_response::<GetPromptResult>(response)?;

        debug!("Got prompt '{}', {} messages", name, result.messages.len());
        Ok(result)
    }

    /// Ping the server.
    pub async fn ping(&self) -> Result<(), McpError> {
        let request = self.make_request(methods::PING, None);
        let response = self.transport.request(request).await?;

        if response.error.is_some() {
            return Err(self.error_from_response(&response));
        }

        Ok(())
    }

    /// Close the client connection.
    pub async fn close(&self) -> Result<(), McpError> {
        self.transport.close().await
    }

    /// Create a JSON-RPC request.
    fn make_request(&self, method: &str, params: Option<Value>) -> JsonRpcRequest {
        let id = self.request_id.fetch_add(1, Ordering::SeqCst);
        JsonRpcRequest::new(RequestId::Number(id), method, params)
    }

    /// Parse a JSON-RPC response into the expected type.
    fn parse_response<T: serde::de::DeserializeOwned>(
        &self,
        response: crate::protocol::JsonRpcResponse,
    ) -> Result<T, McpError> {
        if let Some(error) = response.error {
            return Err(McpError::Server {
                code: error.code,
                message: error.message,
            });
        }

        let result = response
            .result
            .ok_or_else(|| McpError::InvalidResponse("Missing result in response".to_string()))?;

        serde_json::from_value(result)
            .map_err(|e| McpError::InvalidResponse(format!("Failed to parse result: {}", e)))
    }

    /// Convert a response error to McpError.
    fn error_from_response(&self, response: &crate::protocol::JsonRpcResponse) -> McpError {
        if let Some(ref error) = response.error {
            McpError::Server {
                code: error.code,
                message: error.message.clone(),
            }
        } else {
            McpError::InvalidResponse("Expected error response".to_string())
        }
    }

    /// Ensure the client has been initialized.
    fn ensure_initialized(&self) -> Result<(), McpError> {
        if !self.initialized {
            return Err(McpError::NotInitialized);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: Integration tests would require a real MCP server
    // Unit tests focus on request construction

    #[test]
    fn test_protocol_version() {
        assert!(!PROTOCOL_VERSION.is_empty());
    }
}
