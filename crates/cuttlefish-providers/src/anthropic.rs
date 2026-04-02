//! Anthropic direct API provider implementation.

use async_trait::async_trait;
use cuttlefish_core::{
    error::ProviderError,
    traits::provider::{
        CompletionRequest, CompletionResponse, MessageRole, ModelProvider, StreamChunk, ToolCall,
    },
};
use futures::stream::{self, BoxStream, StreamExt};
use reqwest::Client;
use serde_json::{Value, json};
use tracing::{debug, instrument};

const API_BASE: &str = "https://api.anthropic.com/v1/messages";
const API_VERSION: &str = "2023-06-01";

/// Anthropic direct API model provider.
///
/// Supports Claude models via the Anthropic Messages API.
/// Uses pseudo-streaming (complete + chunk) for the stream interface.
pub struct AnthropicProvider {
    /// HTTP client for API requests.
    client: Client,
    /// Anthropic API key.
    api_key: String,
    /// Model identifier (e.g., `claude-opus-4-6`, `claude-sonnet-4-6`, `claude-haiku-4-5`).
    model: String,
}

impl AnthropicProvider {
    /// Create a new Anthropic provider, reading the API key from `ANTHROPIC_API_KEY` env var.
    ///
    /// # Errors
    /// Returns `ProviderError` if `ANTHROPIC_API_KEY` is not set.
    pub fn new(model: impl Into<String>) -> Result<Self, ProviderError> {
        let api_key = std::env::var("ANTHROPIC_API_KEY").map_err(|_| {
            ProviderError("ANTHROPIC_API_KEY environment variable not set".to_string())
        })?;
        Ok(Self::with_api_key(api_key, model))
    }

    /// Create a new Anthropic provider with an explicit API key.
    ///
    /// Useful for testing or when the key is sourced from elsewhere.
    pub fn with_api_key(api_key: impl Into<String>, model: impl Into<String>) -> Self {
        let client = Client::builder()
            .build()
            .expect("Failed to build HTTP client — TLS backend unavailable");
        Self {
            client,
            api_key: api_key.into(),
            model: model.into(),
        }
    }

    /// Build the JSON request body for the Anthropic API.
    fn build_request_body(&self, request: &CompletionRequest) -> Value {
        // Extract system content: prefer explicit system field, then system messages
        let system = request.system.clone().or_else(|| {
            request
                .messages
                .iter()
                .find(|m| m.role == MessageRole::System)
                .map(|m| m.content.clone())
        });

        // Build messages array, filtering out system messages
        let messages: Vec<Value> = request
            .messages
            .iter()
            .filter(|m| m.role != MessageRole::System)
            .map(|m| {
                let role = match m.role {
                    MessageRole::User => "user",
                    MessageRole::Assistant => "assistant",
                    MessageRole::System => "system", // Won't reach here due to filter
                };
                json!({
                    "role": role,
                    "content": m.content
                })
            })
            .collect();

        let mut body = json!({
            "model": self.model,
            "messages": messages,
            "max_tokens": request.max_tokens.unwrap_or(4096),
            "stream": false
        });

        if let Some(sys) = system {
            body["system"] = json!(sys);
        }

        if let Some(temp) = request.temperature {
            body["temperature"] = json!(temp);
        }

        body
    }

    /// Parse the Anthropic API response into a `CompletionResponse`.
    fn parse_response(&self, resp_json: &Value) -> Result<CompletionResponse, ProviderError> {
        let mut content = String::new();
        let mut tool_calls = Vec::new();

        if let Some(content_blocks) = resp_json["content"].as_array() {
            for block in content_blocks {
                match block["type"].as_str() {
                    Some("text") => {
                        if let Some(text) = block["text"].as_str() {
                            content.push_str(text);
                        }
                    }
                    Some("tool_use") => {
                        if let (Some(id), Some(name)) =
                            (block["id"].as_str(), block["name"].as_str())
                        {
                            let input = block["input"].clone();
                            tool_calls.push(ToolCall {
                                id: id.to_string(),
                                name: name.to_string(),
                                input,
                            });
                        }
                    }
                    _ => {}
                }
            }
        }

        let input_tokens = resp_json["usage"]["input_tokens"].as_u64().unwrap_or(0) as u32;
        let output_tokens = resp_json["usage"]["output_tokens"].as_u64().unwrap_or(0) as u32;

        Ok(CompletionResponse {
            content,
            input_tokens,
            output_tokens,
            model: self.model.clone(),
            tool_calls,
        })
    }
}

#[async_trait]
impl ModelProvider for AnthropicProvider {
    fn name(&self) -> &str {
        "anthropic"
    }

    #[instrument(skip(self, request), fields(model = %self.model))]
    async fn complete(
        &self,
        request: CompletionRequest,
    ) -> Result<CompletionResponse, ProviderError> {
        let body = self.build_request_body(&request);
        let body_str = serde_json::to_string(&body)
            .map_err(|e| ProviderError(format!("JSON serialization error: {e}")))?;

        debug!(
            "Sending {} messages to Anthropic model {}",
            request.messages.len(),
            self.model
        );

        let response = self
            .client
            .post(API_BASE)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", API_VERSION)
            .header("content-type", "application/json")
            .body(body_str)
            .send()
            .await
            .map_err(|e| ProviderError(format!("HTTP request failed: {e}")))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(ProviderError(format!("API error {}: {}", status, body)));
        }

        let resp_json: Value = response
            .json()
            .await
            .map_err(|e| ProviderError(format!("JSON parse error: {e}")))?;

        self.parse_response(&resp_json)
    }

    fn stream<'a>(
        &'a self,
        request: CompletionRequest,
    ) -> BoxStream<'a, Result<StreamChunk, ProviderError>> {
        let fut = async move {
            match self.complete(request).await {
                Ok(response) => {
                    let chunks: Vec<Result<StreamChunk, ProviderError>> = vec![
                        Ok(StreamChunk::Text(response.content)),
                        Ok(StreamChunk::Usage {
                            input_tokens: response.input_tokens,
                            output_tokens: response.output_tokens,
                        }),
                    ];
                    stream::iter(chunks).boxed()
                }
                Err(e) => stream::iter(vec![Err(e)]).boxed(),
            }
        };
        futures::stream::once(fut).flatten().boxed()
    }

    async fn count_tokens(&self, text: &str) -> Result<usize, ProviderError> {
        Ok(text.len() / 4 + 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cuttlefish_core::traits::provider::{Message, MessageRole};

    #[test]
    fn test_provider_name() {
        let provider = AnthropicProvider::with_api_key("test-key", "claude-sonnet-4-6");
        assert_eq!(provider.name(), "anthropic");
    }

    #[test]
    fn test_build_request_body_basic() {
        let provider = AnthropicProvider::with_api_key("test-key", "claude-sonnet-4-6");
        let request = CompletionRequest {
            messages: vec![Message {
                role: MessageRole::User,
                content: "Hello".to_string(),
            }],
            max_tokens: Some(1024),
            temperature: Some(0.5),
            system: None,
        };

        let body = provider.build_request_body(&request);
        assert_eq!(body["model"], "claude-sonnet-4-6");
        assert_eq!(body["max_tokens"], 1024);
        assert_eq!(body["temperature"], 0.5);
        assert_eq!(body["stream"], false);
        assert!(body["system"].is_null());

        let messages = body["messages"].as_array().expect("messages array");
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0]["role"], "user");
        assert_eq!(messages[0]["content"], "Hello");
    }

    #[test]
    fn test_build_request_body_with_system() {
        let provider = AnthropicProvider::with_api_key("test-key", "claude-sonnet-4-6");
        let request = CompletionRequest {
            messages: vec![Message {
                role: MessageRole::User,
                content: "Hello".to_string(),
            }],
            max_tokens: None,
            temperature: None,
            system: Some("You are helpful.".to_string()),
        };

        let body = provider.build_request_body(&request);
        assert_eq!(body["system"], "You are helpful.");
        assert_eq!(body["max_tokens"], 4096);

        let messages = body["messages"].as_array().expect("messages array");
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0]["role"], "user");
    }

    #[test]
    fn test_build_request_body_filters_system_messages() {
        let provider = AnthropicProvider::with_api_key("test-key", "claude-sonnet-4-6");
        let request = CompletionRequest {
            messages: vec![
                Message {
                    role: MessageRole::System,
                    content: "System prompt".to_string(),
                },
                Message {
                    role: MessageRole::User,
                    content: "Hello".to_string(),
                },
            ],
            max_tokens: None,
            temperature: None,
            system: None,
        };

        let body = provider.build_request_body(&request);
        let messages = body["messages"].as_array().expect("messages array");
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0]["role"], "user");
        assert_eq!(body["system"], "System prompt");
    }

    #[test]
    fn test_parse_response() {
        let provider = AnthropicProvider::with_api_key("test-key", "claude-sonnet-4-6");
        let resp_json = json!({
            "content": [{
                "type": "text",
                "text": "Hello there!"
            }],
            "usage": {
                "input_tokens": 10,
                "output_tokens": 5
            }
        });

        let response = provider.parse_response(&resp_json).expect("parse response");
        assert_eq!(response.content, "Hello there!");
        assert_eq!(response.input_tokens, 10);
        assert_eq!(response.output_tokens, 5);
        assert_eq!(response.model, "claude-sonnet-4-6");
        assert!(response.tool_calls.is_empty());
    }

    #[test]
    fn test_parse_tool_calls() {
        let provider = AnthropicProvider::with_api_key("test-key", "claude-sonnet-4-6");
        let resp_json = json!({
            "content": [
                {
                    "type": "text",
                    "text": "I'll check the weather."
                },
                {
                    "type": "tool_use",
                    "id": "toolu_123",
                    "name": "get_weather",
                    "input": {"location": "NYC"}
                }
            ],
            "usage": {
                "input_tokens": 10,
                "output_tokens": 15
            }
        });

        let response = provider.parse_response(&resp_json).expect("parse response");
        assert_eq!(response.content, "I'll check the weather.");
        assert_eq!(response.tool_calls.len(), 1);
        assert_eq!(response.tool_calls[0].id, "toolu_123");
        assert_eq!(response.tool_calls[0].name, "get_weather");
        assert_eq!(response.tool_calls[0].input["location"], "NYC");
    }

    #[tokio::test]
    async fn test_count_tokens() {
        let provider = AnthropicProvider::with_api_key("test-key", "claude-sonnet-4-6");
        let count = provider.count_tokens("Hello world test").await.expect("count");
        assert!(count > 0);
        assert_eq!(count, 5); // 16 chars / 4 + 1 = 5
    }

    #[test]
    fn test_with_api_key_constructor() {
        let provider = AnthropicProvider::with_api_key("test-key", "claude-sonnet-4-6");
        assert_eq!(provider.model, "claude-sonnet-4-6");
        assert_eq!(provider.api_key, "test-key");
    }
}
