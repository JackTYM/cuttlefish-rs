//! Moonshot (Kimi) API provider implementation.
//!
//! Provides access to Moonshot's Kimi models via their OpenAI-compatible API.

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
use tracing::debug;

const API_BASE: &str = "https://api.moonshot.cn/v1";

/// Moonshot (Kimi) model provider.
///
/// Uses the Moonshot API (OpenAI-compatible) to communicate with Kimi models.
/// Supports both synchronous completion and pseudo-streaming.
#[derive(Debug)]
pub struct MoonshotProvider {
    /// HTTP client for API requests.
    client: Client,
    /// Moonshot API key.
    api_key: String,
    /// Model name (e.g., `kimi-k2.5`, `moonshot-v1-128k`).
    model: String,
}

impl MoonshotProvider {
    /// Create a new Moonshot provider with the given model.
    ///
    /// Reads the API key from the `MOONSHOT_API_KEY` environment variable.
    ///
    /// # Errors
    /// Returns an error if `MOONSHOT_API_KEY` is not set.
    pub fn new(model: impl Into<String>) -> Result<Self, ProviderError> {
        let api_key = std::env::var("MOONSHOT_API_KEY").map_err(|_| {
            ProviderError("MOONSHOT_API_KEY environment variable not set".to_string())
        })?;
        Ok(Self::with_api_key(api_key, model))
    }

    /// Create a new Moonshot provider with an explicit API key.
    ///
    /// Useful for testing or when the API key is obtained from a different source.
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

    /// Build the request body for the Moonshot API (OpenAI-compatible format).
    fn build_request_body(&self, request: &CompletionRequest) -> Value {
        let mut messages: Vec<Value> = Vec::new();

        // Add system message first if present
        if let Some(sys) = &request.system {
            messages.push(json!({
                "role": "system",
                "content": sys
            }));
        }

        // Add conversation messages
        for m in &request.messages {
            let role = match m.role {
                MessageRole::User => "user",
                MessageRole::Assistant => "assistant",
                MessageRole::System => "system",
            };
            messages.push(json!({
                "role": role,
                "content": m.content
            }));
        }

        let mut body = json!({
            "model": self.model,
            "messages": messages,
            "max_tokens": request.max_tokens.unwrap_or(4096)
        });

        if let Some(temp) = request.temperature {
            body["temperature"] = json!(temp);
        }

        body
    }

    /// Parse the response from the Moonshot API (OpenAI-compatible format).
    fn parse_response(&self, resp_json: &Value) -> Result<CompletionResponse, ProviderError> {
        let content = resp_json["choices"]
            .get(0)
            .and_then(|c| c["message"]["content"].as_str())
            .unwrap_or("")
            .to_string();

        // Extract tool calls if present (OpenAI format)
        let tool_calls: Vec<ToolCall> = resp_json["choices"]
            .get(0)
            .and_then(|c| c["message"]["tool_calls"].as_array())
            .map(|calls| {
                calls
                    .iter()
                    .filter_map(|tc| {
                        let id = tc["id"].as_str()?.to_string();
                        let name = tc["function"]["name"].as_str()?.to_string();
                        let args_str = tc["function"]["arguments"].as_str()?;
                        let input: Value = serde_json::from_str(args_str).ok()?;
                        Some(ToolCall { id, name, input })
                    })
                    .collect()
            })
            .unwrap_or_default();

        let input_tokens = resp_json["usage"]["prompt_tokens"].as_u64().unwrap_or(0) as u32;
        let output_tokens = resp_json["usage"]["completion_tokens"]
            .as_u64()
            .unwrap_or(0) as u32;

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
impl ModelProvider for MoonshotProvider {
    fn name(&self) -> &str {
        "moonshot"
    }

    async fn complete(
        &self,
        request: CompletionRequest,
    ) -> Result<CompletionResponse, ProviderError> {
        let body = self.build_request_body(&request);
        let body_str = serde_json::to_string(&body)
            .map_err(|e| ProviderError(format!("JSON serialization error: {e}")))?;

        let url = format!("{}/chat/completions", API_BASE);

        debug!("Sending request to Moonshot API, model={}", self.model);

        let response = self
            .client
            .post(&url)
            .header("authorization", format!("Bearer {}", self.api_key))
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
        let provider = MoonshotProvider::with_api_key("test-key", "kimi-k2.5");
        assert_eq!(provider.name(), "moonshot");
    }

    #[test]
    fn test_build_request_body_basic() {
        let provider = MoonshotProvider::with_api_key("test-key", "kimi-k2.5");
        let request = CompletionRequest {
            messages: vec![Message {
                role: MessageRole::User,
                content: "Hello".to_string(),
            }],
            max_tokens: Some(1024),
            temperature: Some(0.7),
            system: None,
        };

        let body = provider.build_request_body(&request);

        assert_eq!(body["model"], "kimi-k2.5");
        assert_eq!(body["max_tokens"], 1024);
        let temp = body["temperature"].as_f64().expect("temperature");
        assert!((temp - 0.7).abs() < 0.001);

        let messages = body["messages"].as_array().expect("messages array");
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0]["role"], "user");
        assert_eq!(messages[0]["content"], "Hello");
    }

    #[test]
    fn test_build_request_body_with_system() {
        let provider = MoonshotProvider::with_api_key("test-key", "kimi-k2.5");
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
        let provider = MoonshotProvider::with_api_key("test-key", "kimi-k2.5");
        let resp_json = json!({
            "choices": [{
                "message": {
                    "role": "assistant",
                    "content": "Hello, how can I help?"
                }
            }],
            "usage": {
                "prompt_tokens": 10,
                "completion_tokens": 20
            }
        });

        let response = provider.parse_response(&resp_json).expect("parse");
        assert_eq!(response.content, "Hello, how can I help?");
        assert_eq!(response.input_tokens, 10);
        assert_eq!(response.output_tokens, 20);
        assert!(response.tool_calls.is_empty());
    }

    #[test]
    fn test_parse_response_with_tool_calls() {
        let provider = MoonshotProvider::with_api_key("test-key", "kimi-k2.5");
        let resp_json = json!({
            "choices": [{
                "message": {
                    "role": "assistant",
                    "content": "",
                    "tool_calls": [{
                        "id": "call_123",
                        "type": "function",
                        "function": {
                            "name": "get_weather",
                            "arguments": "{\"location\":\"San Francisco\"}"
                        }
                    }]
                }
            }],
            "usage": {
                "prompt_tokens": 15,
                "completion_tokens": 25
            }
        });

        let response = provider.parse_response(&resp_json).expect("parse");
        assert_eq!(response.tool_calls.len(), 1);
        assert_eq!(response.tool_calls[0].id, "call_123");
        assert_eq!(response.tool_calls[0].name, "get_weather");
        assert_eq!(response.tool_calls[0].input["location"], "San Francisco");
    }

    #[tokio::test]
    async fn test_count_tokens() {
        let provider = MoonshotProvider::with_api_key("test-key", "kimi-k2.5");
        let count = provider
            .count_tokens("Hello world test")
            .await
            .expect("count");
        assert!(count > 0);
    }
}
