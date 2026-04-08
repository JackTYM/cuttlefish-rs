#![deny(unsafe_code)]
#![warn(missing_docs)]
//! MCP (Model Context Protocol) client implementation for Cuttlefish.
//!
//! This crate provides:
//! - MCP protocol types (JSON-RPC based)
//! - Transport implementations (Stdio, SSE)
//! - Client for interacting with MCP servers
//! - Registry for managing multiple MCP servers
//!
//! # Example
//!
//! ```ignore
//! use cuttlefish_mcp::{McpClient, McpRegistry, StdioTransport};
//!
//! // Spawn an MCP server process
//! let transport = StdioTransport::spawn("npx", &["-y", "@modelcontextprotocol/server-filesystem"]).await?;
//!
//! // Create client and initialize
//! let mut client = McpClient::new(transport);
//! client.initialize().await?;
//!
//! // List available tools
//! let tools = client.list_tools().await?;
//!
//! // Call a tool
//! let result = client.call_tool("read_file", Some(serde_json::json!({"path": "/etc/hosts"}))).await?;
//! ```

/// MCP client.
pub mod client;
/// Error types.
pub mod error;
/// MCP protocol types.
pub mod protocol;
/// MCP server registry.
pub mod registry;
/// Transport implementations.
pub mod transport;

pub use client::{McpClient, PROTOCOL_VERSION};
pub use error::McpError;
pub use protocol::{
    CallToolParams, CallToolResult, ClientCapabilities, ClientInfo, InitializeParams,
    InitializeResult, JsonRpcError, JsonRpcNotification, JsonRpcRequest, JsonRpcResponse, Prompt,
    RequestId, Resource, ServerCapabilities, ServerInfo, Tool, ToolContent,
};
pub use registry::{McpRegistry, McpServer, McpToolDefinition};
pub use transport::{McpTransport, SseConfig, SseTransport, StdioTransport};
