//! ChatGPT OAuth provider implementation.
//!
//! Uses OpenAI's chat completions API with OAuth access tokens.

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

const API_BASE: &str = "https://api.openai.com/v1/chat/completions";

/// ChatGPT model provider using OAuth access tokens.
///
/// Sends requests to the OpenAI Chat Completions API using a Bearer token
/// obtained through OAuth flow.
pub struct ChatGptOAuthProvider {
    /// HTTP client.
    client: Client,
    /// OAuth access token.
    access_token: String,
    /// Model ID (e.g., `gpt-4o`, `gpt-4`).
    model: String,
}

impl ChatGptOAuthProvider {
    /// Create a new provider with an existing OAuth access token.
    ///
    /// # Arguments
    /// * `access_token` - OAuth access token from OpenAI
    /// * `model` - Model ID (e.g., `gpt-4o`, `gpt-4`)
    pub fn with_token(access_token: impl Into<String>, model: impl Into<String>) -> Self {
        let client = Client::builder()
            .build()
            .expect("Failed to build HTTP client — TLS backend unavailable");
        Self {
            client,
            access_token: access_token.into(),
            model: model.into(),
        }
    }

    /// Build the request body for OpenAI's chat completions API.
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
}

#[async_trait]
impl ModelProvider for ChatGptOAuthProvider {
    fn name(&self) -> &str {
        "chatgpt-oauth"
    }

    async fn complete(
        &self,
        request: CompletionRequest,
    ) -> Result<CompletionResponse, ProviderError> {
        let body = self.build_request_body(&request);
        let body_str = serde_json::to_string(&body)
            .map_err(|e| ProviderError(format!("JSON serialization error: {e}")))?;

        debug!("Sending request to ChatGPT API, model={}", self.model);

        let response = self
            .client
            .post(API_BASE)
            .header("Authorization", format!("Bearer {}", self.access_token))
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

        let content = resp_json["choices"]
            .as_array()
            .and_then(|arr| arr.first())
            .and_then(|choice| choice["message"]["content"].as_str())
            .unwrap_or("")
            .to_string();

        let input_tokens = resp_json["usage"]["prompt_tokens"].as_u64().unwrap_or(0) as u32;
        let output_tokens = resp_json["usage"]["completion_tokens"].as_u64().unwrap_or(0) as u32;

        Ok(CompletionResponse {
            content,
            input_tokens,
            output_tokens,
            model: self.model.clone(),
            tool_calls: Vec::new(),
        })
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
        let provider = ChatGptOAuthProvider::with_token("test-token", "gpt-4o");
        assert_eq!(provider.name(), "chatgpt-oauth");
    }

    #[test]
    fn test_build_request_body() {
        let provider = ChatGptOAuthProvider::with_token("test-token", "gpt-4o");
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
        assert_eq!(body["model"], "gpt-4o");
        assert_eq!(body["max_tokens"], 1024);
        let temp = body["temperature"].as_f64().expect("temperature");
        assert!((temp - 0.7).abs() < 0.001);
    }

    #[test]
    fn test_build_request_body_with_system() {
        let provider = ChatGptOAuthProvider::with_token("test-token", "gpt-4");
        let request = CompletionRequest {
            messages: vec![Message {
                role: MessageRole::User,
                content: "Hello".to_string(),
            }],
            max_tokens: None,
            temperature: None,
            system: Some("You are a helpful assistant.".to_string()),
        };

        let body = provider.build_request_body(&request);
        let messages = body["messages"].as_array().expect("messages array");
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0]["role"], "system");
        assert_eq!(messages[0]["content"], "You are a helpful assistant.");
        assert_eq!(messages[1]["role"], "user");
    }

    #[tokio::test]
    async fn test_count_tokens() {
        let provider = ChatGptOAuthProvider::with_token("test-token", "gpt-4o");
        let count = provider
            .count_tokens("Hello world test")
            .await
            .expect("count");
        assert!(count > 0);
    }
}
