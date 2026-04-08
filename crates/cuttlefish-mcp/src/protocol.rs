//! MCP (Model Context Protocol) message types.
//!
//! MCP uses JSON-RPC 2.0 for communication between clients and servers.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// JSON-RPC 2.0 request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    /// JSON-RPC version (always "2.0").
    pub jsonrpc: String,
    /// Request ID for correlation.
    pub id: RequestId,
    /// Method name.
    pub method: String,
    /// Method parameters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

impl JsonRpcRequest {
    /// Create a new JSON-RPC request.
    pub fn new(id: impl Into<RequestId>, method: impl Into<String>, params: Option<Value>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id: id.into(),
            method: method.into(),
            params,
        }
    }
}

/// JSON-RPC request ID (can be string or number).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(untagged)]
pub enum RequestId {
    /// String ID.
    String(String),
    /// Numeric ID.
    Number(i64),
}

impl From<String> for RequestId {
    fn from(s: String) -> Self {
        Self::String(s)
    }
}

impl From<&str> for RequestId {
    fn from(s: &str) -> Self {
        Self::String(s.to_string())
    }
}

impl From<i64> for RequestId {
    fn from(n: i64) -> Self {
        Self::Number(n)
    }
}

/// JSON-RPC 2.0 response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    /// JSON-RPC version (always "2.0").
    pub jsonrpc: String,
    /// Request ID for correlation.
    pub id: RequestId,
    /// Result (present on success).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    /// Error (present on failure).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

/// JSON-RPC error object.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    /// Error code.
    pub code: i32,
    /// Error message.
    pub message: String,
    /// Additional error data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

/// Standard JSON-RPC error codes.
pub mod error_codes {
    /// Parse error.
    pub const PARSE_ERROR: i32 = -32700;
    /// Invalid request.
    pub const INVALID_REQUEST: i32 = -32600;
    /// Method not found.
    pub const METHOD_NOT_FOUND: i32 = -32601;
    /// Invalid params.
    pub const INVALID_PARAMS: i32 = -32602;
    /// Internal error.
    pub const INTERNAL_ERROR: i32 = -32603;
}

/// JSON-RPC notification (no response expected).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcNotification {
    /// JSON-RPC version (always "2.0").
    pub jsonrpc: String,
    /// Method name.
    pub method: String,
    /// Method parameters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

// === MCP-specific types ===

/// MCP initialization request params.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializeParams {
    /// Protocol version.
    pub protocol_version: String,
    /// Client capabilities.
    pub capabilities: ClientCapabilities,
    /// Client info.
    pub client_info: ClientInfo,
}

/// Client capabilities.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientCapabilities {
    /// Root listing capability.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub roots: Option<RootsCapability>,
    /// Sampling capability.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sampling: Option<Value>,
}

/// Roots capability.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RootsCapability {
    /// Whether the client supports listing roots.
    pub list_changed: bool,
}

/// Client information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientInfo {
    /// Client name.
    pub name: String,
    /// Client version.
    pub version: String,
}

/// MCP initialization result.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializeResult {
    /// Protocol version.
    pub protocol_version: String,
    /// Server capabilities.
    pub capabilities: ServerCapabilities,
    /// Server info.
    pub server_info: ServerInfo,
}

/// Server capabilities.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerCapabilities {
    /// Tool support.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<ToolsCapability>,
    /// Resource support.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resources: Option<ResourcesCapability>,
    /// Prompt support.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompts: Option<PromptsCapability>,
}

/// Tools capability.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolsCapability {
    /// Whether tools list can change.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub list_changed: Option<bool>,
}

/// Resources capability.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourcesCapability {
    /// Whether the server supports subscriptions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subscribe: Option<bool>,
    /// Whether resource list can change.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub list_changed: Option<bool>,
}

/// Prompts capability.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PromptsCapability {
    /// Whether prompts list can change.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub list_changed: Option<bool>,
}

/// Server information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    /// Server name.
    pub name: String,
    /// Server version.
    pub version: String,
}

/// MCP tool definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tool {
    /// Tool name.
    pub name: String,
    /// Tool description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Input schema (JSON Schema).
    pub input_schema: Value,
}

/// Tools list result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolsListResult {
    /// Available tools.
    pub tools: Vec<Tool>,
}

/// Tool call request params.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallToolParams {
    /// Tool name.
    pub name: String,
    /// Tool arguments.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<Value>,
}

/// Tool call result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallToolResult {
    /// Result content.
    pub content: Vec<ToolContent>,
    /// Whether the call resulted in an error.
    #[serde(default)]
    pub is_error: bool,
}

/// Tool result content.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ToolContent {
    /// Text content.
    Text {
        /// Text value.
        text: String,
    },
    /// Image content.
    Image {
        /// Base64-encoded image data.
        data: String,
        /// MIME type.
        mime_type: String,
    },
    /// Resource content.
    Resource {
        /// Resource reference.
        resource: ResourceReference,
    },
}

/// Resource reference.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceReference {
    /// Resource URI.
    pub uri: String,
    /// MIME type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    /// Resource text (for inline content).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}

/// MCP resource definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Resource {
    /// Resource URI.
    pub uri: String,
    /// Resource name.
    pub name: String,
    /// Resource description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// MIME type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
}

/// Resources list result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcesListResult {
    /// Available resources.
    pub resources: Vec<Resource>,
}

/// Read resource params.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadResourceParams {
    /// Resource URI.
    pub uri: String,
}

/// Read resource result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadResourceResult {
    /// Resource contents.
    pub contents: Vec<ResourceContent>,
}

/// Resource content.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceContent {
    /// Resource URI.
    pub uri: String,
    /// MIME type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    /// Text content.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// Binary content (base64).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blob: Option<String>,
}

/// MCP prompt definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prompt {
    /// Prompt name.
    pub name: String,
    /// Prompt description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Prompt arguments.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<Vec<PromptArgument>>,
}

/// Prompt argument.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptArgument {
    /// Argument name.
    pub name: String,
    /// Argument description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Whether the argument is required.
    #[serde(default)]
    pub required: bool,
}

/// Prompts list result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptsListResult {
    /// Available prompts.
    pub prompts: Vec<Prompt>,
}

/// Get prompt params.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetPromptParams {
    /// Prompt name.
    pub name: String,
    /// Prompt arguments.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<Value>,
}

/// Get prompt result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetPromptResult {
    /// Prompt description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Prompt messages.
    pub messages: Vec<PromptMessage>,
}

/// Prompt message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptMessage {
    /// Message role.
    pub role: String,
    /// Message content.
    pub content: PromptContent,
}

/// Prompt content.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum PromptContent {
    /// Text content.
    Text {
        /// Text value.
        text: String,
    },
    /// Image content.
    Image {
        /// Base64-encoded image data.
        data: String,
        /// MIME type.
        mime_type: String,
    },
    /// Resource content.
    Resource {
        /// Resource reference.
        resource: ResourceReference,
    },
}

/// MCP method names.
pub mod methods {
    /// Initialize the connection.
    pub const INITIALIZE: &str = "initialize";
    /// Notify that initialization is complete.
    pub const INITIALIZED: &str = "notifications/initialized";
    /// List available tools.
    pub const TOOLS_LIST: &str = "tools/list";
    /// Call a tool.
    pub const TOOLS_CALL: &str = "tools/call";
    /// List available resources.
    pub const RESOURCES_LIST: &str = "resources/list";
    /// Read a resource.
    pub const RESOURCES_READ: &str = "resources/read";
    /// List available prompts.
    pub const PROMPTS_LIST: &str = "prompts/list";
    /// Get a prompt.
    pub const PROMPTS_GET: &str = "prompts/get";
    /// Ping the server.
    pub const PING: &str = "ping";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_serialization() {
        let req = JsonRpcRequest::new(
            "test-id",
            "tools/list",
            None,
        );
        let json = serde_json::to_string(&req).expect("serialize");
        assert!(json.contains("\"jsonrpc\":\"2.0\""));
        assert!(json.contains("\"method\":\"tools/list\""));
    }

    #[test]
    fn test_response_deserialization() {
        let json = r#"{"jsonrpc":"2.0","id":"test","result":{"tools":[]}}"#;
        let resp: JsonRpcResponse = serde_json::from_str(json).expect("deserialize");
        assert_eq!(resp.id, RequestId::String("test".to_string()));
        assert!(resp.result.is_some());
        assert!(resp.error.is_none());
    }

    #[test]
    fn test_tool_serialization() {
        let tool = Tool {
            name: "read_file".to_string(),
            description: Some("Read a file".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string" }
                },
                "required": ["path"]
            }),
        };
        let json = serde_json::to_string(&tool).expect("serialize");
        assert!(json.contains("read_file"));
        assert!(json.contains("inputSchema"));
    }
}
