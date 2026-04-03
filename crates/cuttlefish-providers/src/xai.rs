//! xAI (Grok) model provider implementation.
//!
//! Provides access to xAI's Grok models via their OpenAI-compatible API.

use async_trait::async_trait;
use cuttlefish_core::{
    error::ProviderError,
    traits::provider::{
        CompletionRequest, CompletionResponse, MessageRole, ModelProvider, StreamChunk,
    },
};
use futures::stream::{self, BoxStream, StreamExt};
use reqwest::Client;
use serde_json::{Value, json};
use tracing::debug;

const API_BASE: &str = "https://api.x.ai/v1/chat/completions";

/// xAI (Grok) model provider.
///
/// Sends requests to xAI's chat completions API using OpenAI-compatible format.
pub struct XaiProvider {
    /// HTTP client.
    client: Client,
    /// API key.
    api_key: String,
    /// Model ID (e.g., `grok-2`, `grok-beta`).
    model: String,
}

impl XaiProvider {
    /// Create a new provider, reading API key from `XAI_API_KEY` environment variable.
    ///
    /// # Errors
    /// Returns an error if `XAI_API_KEY` is not set.
    pub fn new(model: impl Into<String>) -> Result<Self, ProviderError> {
        let api_key = std::env::var("XAI_API_KEY")
            .map_err(|_| ProviderError("XAI_API_KEY environment variable not set".to_string()))?;
        Ok(Self::with_api_key(api_key, model))
    }

    /// Create a new provider with an explicit API key.
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

    /// Build the request body in OpenAI-compatible format.
    fn build_request_body(&self, request: &CompletionRequest) -> Value {
        let mut messages: Vec<Value> = Vec::new();

        let system = request.system.clone().or_else(|| {
            request
                .messages
                .iter()
                .find(|m| m.role == MessageRole::System)
                .map(|m| m.content.clone())
        });

        if let Some(sys) = system {
            messages.push(json!({
                "role": "system",
                "content": sys
            }));
        }

        for msg in &request.messages {
            if msg.role == MessageRole::System {
                continue;
            }
            let role = match msg.role {
                MessageRole::User => "user",
                MessageRole::Assistant => "assistant",
                MessageRole::System => continue,
            };
            messages.push(json!({
                "role": role,
                "content": msg.content
            }));
        }

        let mut body = json!({
            "model": self.model,
            "messages": messages,
        });

        if let Some(max_tokens) = request.max_tokens {
            body["max_tokens"] = json!(max_tokens);
        }

        if let Some(temp) = request.temperature {
            body["temperature"] = json!(temp);
        }

        body
    }

    /// Parse the response from xAI's API (OpenAI-compatible format).
    fn parse_response(&self, resp_json: &Value) -> Result<CompletionResponse, ProviderError> {
        let content = resp_json["choices"]
            .as_array()
            .and_then(|arr| arr.first())
            .and_then(|choice| choice["message"]["content"].as_str())
            .unwrap_or("")
            .to_string();

        let input_tokens = resp_json["usage"]["prompt_tokens"].as_u64().unwrap_or(0) as u32;
        let output_tokens = resp_json["usage"]["completion_tokens"]
            .as_u64()
            .unwrap_or(0) as u32;

        Ok(CompletionResponse {
            content,
            input_tokens,
            output_tokens,
            model: self.model.clone(),
            tool_calls: Vec::new(),
        })
    }
}

#[async_trait]
impl ModelProvider for XaiProvider {
    fn name(&self) -> &str {
        "xai"
    }

    async fn complete(
        &self,
        request: CompletionRequest,
    ) -> Result<CompletionResponse, ProviderError> {
        let body = self.build_request_body(&request);
        let body_str = serde_json::to_string(&body)
            .map_err(|e| ProviderError(format!("JSON serialization error: {e}")))?;

        debug!("Sending request to xAI API, model={}", self.model);

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
        let provider = XaiProvider::with_api_key("test-key", "grok-2");
        assert_eq!(provider.name(), "xai");
    }

    #[test]
    fn test_build_request_body() {
        let provider = XaiProvider::with_api_key("test-key", "grok-2");
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
        assert_eq!(body["model"], "grok-2");
        assert_eq!(body["max_tokens"], 1024);
        let temp = body["temperature"].as_f64().expect("temperature");
        assert!((temp - 0.7).abs() < 0.001);
    }

    #[test]
    fn test_parse_response() {
        let provider = XaiProvider::with_api_key("test-key", "grok-beta");
        let resp_json = json!({
            "choices": [{
                "message": {
                    "role": "assistant",
                    "content": "Hello! I'm Grok."
                }
            }],
            "usage": {
                "prompt_tokens": 10,
                "completion_tokens": 5
            }
        });

        let response = provider.parse_response(&resp_json).expect("parse");
        assert_eq!(response.content, "Hello! I'm Grok.");
        assert_eq!(response.input_tokens, 10);
        assert_eq!(response.output_tokens, 5);
        assert_eq!(response.model, "grok-beta");
    }

    #[tokio::test]
    async fn test_count_tokens() {
        let provider = XaiProvider::with_api_key("test-key", "grok-2");
        let count = provider
            .count_tokens("Hello world test")
            .await
            .expect("count");
        assert!(count > 0);
    }
}
