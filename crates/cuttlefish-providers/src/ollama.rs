//! Ollama local model provider implementation.
//!
//! Provides access to locally running Ollama models.

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

const DEFAULT_BASE_URL: &str = "http://localhost:11434";

/// Ollama local model provider.
///
/// Sends requests to a local Ollama server using Ollama's native API format.
pub struct OllamaProvider {
    /// HTTP client.
    client: Client,
    /// Base URL of the Ollama server.
    base_url: String,
    /// Model ID (e.g., `llama3.1`, `codellama`).
    model: String,
}

impl OllamaProvider {
    /// Create a new provider using default localhost URL.
    ///
    /// Connects to `http://localhost:11434` by default.
    pub fn new(model: impl Into<String>) -> Self {
        Self::with_base_url(DEFAULT_BASE_URL, model)
    }

    /// Create a new provider with a custom base URL.
    ///
    /// # Arguments
    /// * `base_url` - Base URL of the Ollama server (e.g., `http://192.168.1.100:11434`)
    /// * `model` - Model ID (e.g., `llama3.1`, `codellama`)
    pub fn with_base_url(base_url: impl Into<String>, model: impl Into<String>) -> Self {
        let client = Client::builder()
            .build()
            .expect("Failed to build HTTP client — TLS backend unavailable");
        Self {
            client,
            base_url: base_url.into(),
            model: model.into(),
        }
    }

    /// Build the request body in Ollama's native format.
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
            "stream": false,
        });

        let mut options = json!({});
        if let Some(temp) = request.temperature {
            options["temperature"] = json!(temp);
        }

        if !options.as_object().is_none_or(|o| o.is_empty()) {
            body["options"] = options;
        }

        body
    }

    /// Parse the response from Ollama's API.
    fn parse_response(&self, resp_json: &Value) -> Result<CompletionResponse, ProviderError> {
        let content = resp_json["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();

        let input_tokens = resp_json["prompt_eval_count"].as_u64().unwrap_or(0) as u32;
        let output_tokens = resp_json["eval_count"].as_u64().unwrap_or(0) as u32;

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
impl ModelProvider for OllamaProvider {
    fn name(&self) -> &str {
        "ollama"
    }

    async fn complete(
        &self,
        request: CompletionRequest,
    ) -> Result<CompletionResponse, ProviderError> {
        let body = self.build_request_body(&request);
        let body_str = serde_json::to_string(&body)
            .map_err(|e| ProviderError(format!("JSON serialization error: {e}")))?;

        let api_url = format!("{}/api/chat", self.base_url);
        debug!(
            "Sending request to Ollama API at {}, model={}",
            api_url, self.model
        );

        let response = self
            .client
            .post(&api_url)
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
        let provider = OllamaProvider::new("llama3.1");
        assert_eq!(provider.name(), "ollama");
    }

    #[test]
    fn test_build_request_body() {
        let provider = OllamaProvider::new("llama3.1");
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
        assert_eq!(body["model"], "llama3.1");
        assert_eq!(body["stream"], false);
        let temp = body["options"]["temperature"]
            .as_f64()
            .expect("temperature");
        assert!((temp - 0.7).abs() < 0.001);
    }

    #[test]
    fn test_build_request_body_with_system() {
        let provider = OllamaProvider::with_base_url("http://custom:11434", "codellama");
        let request = CompletionRequest {
            messages: vec![Message {
                role: MessageRole::User,
                content: "Hello".to_string(),
            }],
            max_tokens: None,
            temperature: None,
            system: Some("You are a coding assistant.".to_string()),
        };

        let body = provider.build_request_body(&request);
        let messages = body["messages"].as_array().expect("messages array");
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0]["role"], "system");
        assert_eq!(messages[0]["content"], "You are a coding assistant.");
    }

    #[test]
    fn test_parse_response() {
        let provider = OllamaProvider::new("llama3.1");
        let resp_json = json!({
            "message": {
                "role": "assistant",
                "content": "Hi! How can I help you?"
            },
            "prompt_eval_count": 10,
            "eval_count": 50
        });

        let response = provider.parse_response(&resp_json).expect("parse");
        assert_eq!(response.content, "Hi! How can I help you?");
        assert_eq!(response.input_tokens, 10);
        assert_eq!(response.output_tokens, 50);
        assert_eq!(response.model, "llama3.1");
        assert!(response.tool_calls.is_empty());
    }

    #[tokio::test]
    async fn test_count_tokens() {
        let provider = OllamaProvider::new("llama3.1");
        let count = provider
            .count_tokens("Hello world test")
            .await
            .expect("count");
        assert!(count > 0);
    }
}
