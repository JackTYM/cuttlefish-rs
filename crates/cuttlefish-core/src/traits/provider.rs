//! Model provider trait and supporting types.

use async_trait::async_trait;
use futures::stream::BoxStream;
use serde::{Deserialize, Serialize};

use crate::error::ProviderError;

/// Result type for provider operations.
pub type ProviderResult<T> = Result<T, ProviderError>;

/// A single message in a conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// The role of the message sender.
    pub role: MessageRole,
    /// The message content.
    pub content: String,
}

/// Role of a message sender.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    /// System message (instructions).
    System,
    /// User message.
    User,
    /// Assistant (model) message.
    Assistant,
}

/// A completion request to a model provider.
#[derive(Debug, Clone)]
pub struct CompletionRequest {
    /// Conversation messages.
    pub messages: Vec<Message>,
    /// Maximum tokens to generate.
    pub max_tokens: Option<u32>,
    /// Temperature for sampling.
    pub temperature: Option<f32>,
    /// System prompt (prepended before messages if set separately).
    pub system: Option<String>,
}

/// A complete response from the model.
#[derive(Debug, Clone)]
pub struct CompletionResponse {
    /// The generated text content.
    pub content: String,
    /// Tokens used in the request.
    pub input_tokens: u32,
    /// Tokens generated in the response.
    pub output_tokens: u32,
    /// The model that generated the response.
    pub model: String,
    /// Tool calls requested by the model (if any).
    pub tool_calls: Vec<ToolCall>,
}

/// A tool call requested by the model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    /// Tool call ID.
    pub id: String,
    /// Tool name.
    pub name: String,
    /// Tool input as JSON string.
    pub input: serde_json::Value,
}

/// A streaming chunk from the model.
#[derive(Debug, Clone)]
pub enum StreamChunk {
    /// Text delta.
    Text(String),
    /// Tool call delta.
    ToolCallDelta {
        /// Tool call ID.
        id: String,
        /// Tool name.
        name: String,
        /// Partial input JSON.
        input_delta: String,
    },
    /// Usage statistics (usually final chunk).
    Usage {
        /// Tokens used in the request.
        input_tokens: u32,
        /// Tokens generated in the response.
        output_tokens: u32,
    },
}

/// A model provider (Bedrock, Claude OAuth, etc.).
#[async_trait]
pub trait ModelProvider: Send + Sync {
    /// Provider name (e.g., "bedrock", "claude-oauth").
    fn name(&self) -> &str;

    /// Complete a conversation with the model.
    async fn complete(&self, request: CompletionRequest) -> ProviderResult<CompletionResponse>;

    /// Stream a completion from the model.
    fn stream<'a>(
        &'a self,
        request: CompletionRequest,
    ) -> BoxStream<'a, ProviderResult<StreamChunk>>;

    /// Estimate token count for text.
    async fn count_tokens(&self, text: &str) -> ProviderResult<usize>;
}
