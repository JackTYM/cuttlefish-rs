//! OpenAI API provider implementation.

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

const API_BASE: &str = "https://api.openai.com/v1/chat/completions";

/// OpenAI API model provider.
///
/// Supports GPT models via the OpenAI Chat Completions API.
/// Uses pseudo-streaming (complete + chunk) for the stream interface.
pub struct OpenAiProvider {
    /// HTTP client for API requests.
    client: Client,
    /// OpenAI API key.
    api_key: String,
    /// Model identifier (e.g., `gpt-5.4`, `gpt-5-nano`, `gpt-4o`).
    model: String,
}

impl OpenAiProvider {
    /// Create a new OpenAI provider, reading the API key from `OPENAI_API_KEY` env var.
    ///
    /// # Errors
    /// Returns `ProviderError` if `OPENAI_API_KEY` is not set.
    pub fn new(model: impl Into<String>) -> Result<Self, ProviderError> {
        let api_key = std::env::var("OPENAI_API_KEY")
            .map_err(|_| ProviderError("OPENAI_API_KEY environment variable not set".to_string()))?;
        Ok(Self::with_api_key(api_key, model))
    }

    /// Create a new OpenAI provider with an explicit API key.
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

    /// Build the JSON request body for the OpenAI API.
    fn build_request_body(&self, request: &CompletionRequest) -> Value {
        let mut messages: Vec<Value> = Vec::new();

        // Add system message if present
        if let Some(sys) = &request.system {
            messages.push(json!({
                "role": "system",
                "content": sys
            }));
        }

        // Add system messages from the messages array first
        for msg in &request.messages {
            if msg.role == MessageRole::System {
                messages.push(json!({
                    "role": "system",
                    "content": msg.content
                }));
            }
        }

        // Add non-system messages
        for msg in &request.messages {
            if msg.role != MessageRole::System {
                let role = match msg.role {
                    MessageRole::User => "user",
                    MessageRole::Assistant => "assistant",
                    MessageRole::System => continue, // Already handled above
                };
                messages.push(json!({
                    "role": role,
                    "content": msg.content
                }));
            }
        }

        let mut body = json!({
            "model": self.model,
            "messages": messages,
            "max_tokens": request.max_tokens.unwrap_or(4096),
        });

        if let Some(temp) = request.temperature {
            body["temperature"] = json!(temp);
        }

        body
    }

    /// Parse the OpenAI API response into a `CompletionResponse`.
    fn parse_response(&self, resp_json: &Value) -> Result<CompletionResponse, ProviderError> {
        let content = resp_json["choices"]
            .get(0)
            .and_then(|c| c["message"]["content"].as_str())
            .unwrap_or("")
            .to_string();

        let input_tokens = resp_json["usage"]["prompt_tokens"].as_u64().unwrap_or(0) as u32;
        let output_tokens = resp_json["usage"]["completion_tokens"].as_u64().unwrap_or(0) as u32;

        // Parse tool calls if present
        let tool_calls = self.parse_tool_calls(resp_json);

        Ok(CompletionResponse {
            content,
            input_tokens,
            output_tokens,
            model: self.model.clone(),
            tool_calls,
        })
    }

    /// Parse tool calls from the OpenAI response.
    fn parse_tool_calls(&self, resp_json: &Value) -> Vec<ToolCall> {
        let mut tool_calls = Vec::new();

        if let Some(calls) = resp_json["choices"]
            .get(0)
            .and_then(|c| c["message"]["tool_calls"].as_array())
        {
            for call in calls {
                if let (Some(id), Some(name), Some(arguments)) = (
                    call["id"].as_str(),
                    call["function"]["name"].as_str(),
                    call["function"]["arguments"].as_str(),
                ) {
                    // Parse arguments as JSON, fallback to string value if invalid
                    let input = serde_json::from_str(arguments)
                        .unwrap_or_else(|_| json!({"raw": arguments}));
                    tool_calls.push(ToolCall {
                        id: id.to_string(),
                        name: name.to_string(),
                        input,
                    });
                }
            }
        }

        tool_calls
    }
}

#[async_trait]
impl ModelProvider for OpenAiProvider {
    fn name(&self) -> &str {
        "openai"
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
            "Sending {} messages to OpenAI model {}",
            request.messages.len(),
            self.model
        );

        let response = self
            .client
            .post(API_BASE)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
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
        let provider = OpenAiProvider::with_api_key("test-key", "gpt-5.4");
        assert_eq!(provider.name(), "openai");
    }

    #[test]
    fn test_build_request_body_basic() {
        let provider = OpenAiProvider::with_api_key("test-key", "gpt-5.4");
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
        assert_eq!(body["model"], "gpt-5.4");
        assert_eq!(body["max_tokens"], 1024);
        assert_eq!(body["temperature"], 0.5);

        let messages = body["messages"].as_array().expect("messages array");
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0]["role"], "user");
        assert_eq!(messages[0]["content"], "Hello");
    }

    #[test]
    fn test_build_request_body_with_system() {
        let provider = OpenAiProvider::with_api_key("test-key", "gpt-5.4");
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
        let messages = body["messages"].as_array().expect("messages array");
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0]["role"], "system");
        assert_eq!(messages[0]["content"], "You are helpful.");
        assert_eq!(messages[1]["role"], "user");
        assert_eq!(body["max_tokens"], 4096);
    }

    #[test]
    fn test_parse_response() {
        let provider = OpenAiProvider::with_api_key("test-key", "gpt-5.4");
        let resp_json = json!({
            "choices": [{
                "message": {
                    "content": "Hello there!"
                }
            }],
            "usage": {
                "prompt_tokens": 10,
                "completion_tokens": 5
            }
        });

        let response = provider.parse_response(&resp_json).expect("parse response");
        assert_eq!(response.content, "Hello there!");
        assert_eq!(response.input_tokens, 10);
        assert_eq!(response.output_tokens, 5);
        assert_eq!(response.model, "gpt-5.4");
        assert!(response.tool_calls.is_empty());
    }

    #[test]
    fn test_parse_tool_calls() {
        let provider = OpenAiProvider::with_api_key("test-key", "gpt-5.4");
        let resp_json = json!({
            "choices": [{
                "message": {
                    "content": "",
                    "tool_calls": [{
                        "id": "call_123",
                        "function": {
                            "name": "get_weather",
                            "arguments": "{\"location\":\"NYC\"}"
                        }
                    }]
                }
            }],
            "usage": {
                "prompt_tokens": 10,
                "completion_tokens": 5
            }
        });

        let tool_calls = provider.parse_tool_calls(&resp_json);
        assert_eq!(tool_calls.len(), 1);
        assert_eq!(tool_calls[0].id, "call_123");
        assert_eq!(tool_calls[0].name, "get_weather");
        assert_eq!(tool_calls[0].input["location"], "NYC");
    }

    #[tokio::test]
    async fn test_count_tokens() {
        let provider = OpenAiProvider::with_api_key("test-key", "gpt-5.4");
        let count = provider.count_tokens("Hello world test").await.expect("count");
        assert!(count > 0);
        // "Hello world test" is 16 chars, so 16/4 + 1 = 5
        assert_eq!(count, 5);
    }

    #[test]
    fn test_with_api_key_constructor() {
        let provider = OpenAiProvider::with_api_key("test-key", "gpt-5.4");
        assert_eq!(provider.model, "gpt-5.4");
        assert_eq!(provider.api_key, "test-key");
    }
}
