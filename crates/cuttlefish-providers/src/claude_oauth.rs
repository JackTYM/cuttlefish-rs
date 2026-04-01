//! Claude Code OAuth provider implementation.
//!
//! Authenticates with Anthropic's API by emulating the Claude Code CLI client.

use async_trait::async_trait;
use cuttlefish_core::{
    error::ProviderError,
    traits::provider::{
        CompletionRequest, CompletionResponse, MessageRole, ModelProvider, StreamChunk,
    },
};
use futures::stream::{self, BoxStream};
use futures::StreamExt;
use reqwest::Client;
use serde_json::{json, Value};
use tracing::debug;

use crate::oauth_flow::{compute_cch, OAuthTokens, ANTHROPIC_BETA, USER_AGENT};

const API_BASE: &str = "https://api.anthropic.com";
const API_VERSION: &str = "2023-06-01";

/// Claude model provider using Claude Code OAuth authentication.
///
/// Sends requests to the Anthropic Messages API with spoofed
/// Claude Code CLI headers and CCH body signing.
pub struct ClaudeOAuthProvider {
    client: Client,
    tokens: OAuthTokens,
    model: String,
}

impl ClaudeOAuthProvider {
    /// Create a new provider with existing OAuth tokens.
    pub fn new(tokens: OAuthTokens, model: impl Into<String>) -> Self {
        let client = Client::builder()
            .user_agent(USER_AGENT)
            .build()
            .expect("Failed to build HTTP client — TLS backend unavailable");
        Self {
            client,
            tokens,
            model: model.into(),
        }
    }

    fn build_request_body(&self, request: &CompletionRequest) -> Value {
        let messages: Vec<Value> = request
            .messages
            .iter()
            .filter(|m| m.role != MessageRole::System)
            .map(|m| {
                let role = match m.role {
                    MessageRole::User => "user",
                    MessageRole::Assistant => "assistant",
                    MessageRole::System => "system",
                };
                json!({
                    "role": role,
                    "content": m.content
                })
            })
            .collect();

        let system = request.system.clone().or_else(|| {
            request
                .messages
                .iter()
                .find(|m| m.role == MessageRole::System)
                .map(|m| m.content.clone())
        });

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
}

#[async_trait]
impl ModelProvider for ClaudeOAuthProvider {
    fn name(&self) -> &str {
        "claude-oauth"
    }

    async fn complete(
        &self,
        request: CompletionRequest,
    ) -> Result<CompletionResponse, ProviderError> {
        let body = self.build_request_body(&request);
        let body_str = serde_json::to_string(&body)
            .map_err(|e| ProviderError(format!("JSON serialization error: {e}")))?;

        let cch = compute_cch(&body_str);
        let billing_header = format!(
            "cc_version=2.1.87.fingerprint; cc_entrypoint=cli; cch={};",
            cch
        );

        debug!("Sending request to Claude API, model={}", self.model);

        let response = self
            .client
            .post(format!("{}/v1/messages", API_BASE))
            .header(
                "authorization",
                format!("Bearer {}", self.tokens.access_token),
            )
            .header("anthropic-version", API_VERSION)
            .header("anthropic-beta", ANTHROPIC_BETA)
            .header("anthropic-dangerous-direct-browser-access", "true")
            .header("x-anthropic-billing-header", billing_header)
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

        let content = resp_json["content"]
            .as_array()
            .and_then(|arr| arr.first())
            .and_then(|block| block["text"].as_str())
            .unwrap_or("")
            .to_string();

        let input_tokens = resp_json["usage"]["input_tokens"].as_u64().unwrap_or(0) as u32;
        let output_tokens = resp_json["usage"]["output_tokens"].as_u64().unwrap_or(0) as u32;

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
                Ok(resp) => stream::iter(vec![
                    Ok(StreamChunk::Text(resp.content)),
                    Ok(StreamChunk::Usage {
                        input_tokens: resp.input_tokens,
                        output_tokens: resp.output_tokens,
                    }),
                ])
                .boxed(),
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

    fn test_tokens() -> OAuthTokens {
        OAuthTokens {
            access_token: "test-access-token".to_string(),
            refresh_token: Some("test-refresh-token".to_string()),
            expires_at: Some(9999999999),
        }
    }

    #[test]
    fn test_provider_name() {
        let provider = ClaudeOAuthProvider::new(test_tokens(), "claude-sonnet-4-20250514");
        assert_eq!(provider.name(), "claude-oauth");
    }

    #[test]
    fn test_build_request_body_basic() {
        let provider = ClaudeOAuthProvider::new(test_tokens(), "claude-sonnet-4-20250514");
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
        assert_eq!(body["model"], "claude-sonnet-4-20250514");
        assert_eq!(body["max_tokens"], 1024);
        assert_eq!(body["temperature"], 0.5);
        assert!(body["system"].is_null());
    }

    #[test]
    fn test_build_request_body_with_system() {
        let provider = ClaudeOAuthProvider::new(test_tokens(), "claude-sonnet-4-20250514");
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
    }

    #[test]
    fn test_build_request_body_filters_system_messages() {
        let provider = ClaudeOAuthProvider::new(test_tokens(), "claude-sonnet-4-20250514");
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

    #[tokio::test]
    async fn test_count_tokens_returns_estimate() {
        let provider = ClaudeOAuthProvider::new(test_tokens(), "claude-sonnet-4-20250514");
        let count = provider
            .count_tokens("Hello world test")
            .await
            .expect("count");
        assert!(count > 0);
    }
}
