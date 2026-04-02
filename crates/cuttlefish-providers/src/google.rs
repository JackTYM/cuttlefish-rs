//! Google Gemini API provider implementation.
//!
//! Provides access to Google's Gemini models via the Generative Language API.

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
use std::sync::atomic::{AtomicU64, Ordering};

static CALL_COUNTER: AtomicU64 = AtomicU64::new(0);

const API_BASE: &str = "https://generativelanguage.googleapis.com/v1beta";

/// Google Gemini model provider.
///
/// Uses the Generative Language API to communicate with Gemini models.
/// Supports both synchronous completion and pseudo-streaming.
#[derive(Debug)]
pub struct GoogleProvider {
    /// HTTP client for API requests.
    client: Client,
    /// Google API key.
    api_key: String,
    /// Model name (e.g., `gemini-2.0-flash`).
    model: String,
}

impl GoogleProvider {
    /// Create a new Google provider with the given model.
    ///
    /// Reads the API key from the `GOOGLE_API_KEY` environment variable.
    ///
    /// # Errors
    /// Returns an error if `GOOGLE_API_KEY` is not set.
    pub fn new(model: impl Into<String>) -> Result<Self, ProviderError> {
        let api_key = std::env::var("GOOGLE_API_KEY")
            .map_err(|_| ProviderError("GOOGLE_API_KEY environment variable not set".to_string()))?;
        Ok(Self::with_api_key(api_key, model))
    }

    /// Create a new Google provider with an explicit API key.
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

    /// Build the request body for the Gemini API.
    fn build_request_body(&self, request: &CompletionRequest) -> Value {
        // Separate system messages from conversation messages
        let system_text: Option<String> = request.system.clone().or_else(|| {
            request
                .messages
                .iter()
                .find(|m| m.role == MessageRole::System)
                .map(|m| m.content.clone())
        });

        // Build contents array (excluding system messages)
        let contents: Vec<Value> = request
            .messages
            .iter()
            .filter(|m| m.role != MessageRole::System)
            .map(|m| {
                let role = match m.role {
                    MessageRole::User => "user",
                    MessageRole::Assistant => "model",
                    MessageRole::System => "user", // Should not reach here due to filter
                };
                json!({
                    "role": role,
                    "parts": [{"text": m.content}]
                })
            })
            .collect();

        let mut body = json!({
            "contents": contents,
            "generationConfig": {
                "maxOutputTokens": request.max_tokens.unwrap_or(4096)
            }
        });

        // Add system instruction if present
        if let Some(sys) = system_text {
            body["systemInstruction"] = json!({
                "parts": [{"text": sys}]
            });
        }

        // Add temperature if specified
        if let Some(temp) = request.temperature {
            body["generationConfig"]["temperature"] = json!(temp);
        }

        body
    }

    /// Parse the response from the Gemini API.
    fn parse_response(&self, resp_json: &Value) -> Result<CompletionResponse, ProviderError> {
        // Extract text content
        let content = resp_json["candidates"]
            .get(0)
            .and_then(|c| c["content"]["parts"].as_array())
            .and_then(|parts| {
                parts
                    .iter()
                    .filter_map(|p| p["text"].as_str())
                    .collect::<Vec<_>>()
                    .first()
                    .map(|s| s.to_string())
            })
            .unwrap_or_default();

        // Extract tool calls if present
        let tool_calls: Vec<ToolCall> = resp_json["candidates"]
            .get(0)
            .and_then(|c| c["content"]["parts"].as_array())
            .map(|parts| {
                parts
                    .iter()
                    .filter_map(|p| {
                        p["functionCall"].as_object().map(|fc| {
                            let call_id = CALL_COUNTER.fetch_add(1, Ordering::Relaxed);
                            ToolCall {
                                id: format!("call_{call_id}"),
                                name: fc
                                    .get("name")
                                    .and_then(|n| n.as_str())
                                    .unwrap_or("")
                                    .to_string(),
                                input: fc.get("args").cloned().unwrap_or(json!({})),
                            }
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        // Extract usage metadata
        let input_tokens = resp_json["usageMetadata"]["promptTokenCount"]
            .as_u64()
            .unwrap_or(0) as u32;
        let output_tokens = resp_json["usageMetadata"]["candidatesTokenCount"]
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
impl ModelProvider for GoogleProvider {
    fn name(&self) -> &str {
        "google"
    }

    async fn complete(
        &self,
        request: CompletionRequest,
    ) -> Result<CompletionResponse, ProviderError> {
        let body = self.build_request_body(&request);
        let body_str = serde_json::to_string(&body)
            .map_err(|e| ProviderError(format!("JSON serialization error: {e}")))?;

        let url = format!(
            "{}/models/{}:generateContent?key={}",
            API_BASE, self.model, self.api_key
        );

        debug!("Sending request to Google Gemini API, model={}", self.model);

        let response = self
            .client
            .post(&url)
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
        // Rough estimate: ~4 characters per token
        Ok(text.len() / 4 + 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cuttlefish_core::traits::provider::{Message, MessageRole};

    #[test]
    fn test_provider_name() {
        let provider = GoogleProvider::with_api_key("test-key", "gemini-2.0-flash");
        assert_eq!(provider.name(), "google");
    }

    #[test]
    fn test_build_request_body_basic() {
        let provider = GoogleProvider::with_api_key("test-key", "gemini-2.0-flash");
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

        assert!(body["contents"].is_array());
        let contents = body["contents"].as_array().expect("contents array");
        assert_eq!(contents.len(), 1);
        assert_eq!(contents[0]["role"], "user");
        assert_eq!(contents[0]["parts"][0]["text"], "Hello");
        assert_eq!(body["generationConfig"]["maxOutputTokens"], 1024);
        assert_eq!(body["generationConfig"]["temperature"], 0.5);
        assert!(body["systemInstruction"].is_null());
    }

    #[test]
    fn test_build_request_body_with_system() {
        let provider = GoogleProvider::with_api_key("test-key", "gemini-2.0-flash");
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

        assert_eq!(body["systemInstruction"]["parts"][0]["text"], "You are helpful.");
        assert_eq!(body["generationConfig"]["maxOutputTokens"], 4096);
    }

    #[test]
    fn test_role_mapping() {
        let provider = GoogleProvider::with_api_key("test-key", "gemini-2.0-flash");
        let request = CompletionRequest {
            messages: vec![
                Message {
                    role: MessageRole::User,
                    content: "Hello".to_string(),
                },
                Message {
                    role: MessageRole::Assistant,
                    content: "Hi there!".to_string(),
                },
                Message {
                    role: MessageRole::User,
                    content: "How are you?".to_string(),
                },
            ],
            max_tokens: None,
            temperature: None,
            system: None,
        };

        let body = provider.build_request_body(&request);
        let contents = body["contents"].as_array().expect("contents array");

        assert_eq!(contents.len(), 3);
        assert_eq!(contents[0]["role"], "user");
        assert_eq!(contents[1]["role"], "model"); // Assistant maps to "model"
        assert_eq!(contents[2]["role"], "user");
    }

    #[test]
    fn test_system_message_in_messages_array() {
        let provider = GoogleProvider::with_api_key("test-key", "gemini-2.0-flash");
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

        // System message should be extracted to systemInstruction
        assert_eq!(body["systemInstruction"]["parts"][0]["text"], "System prompt");

        // Contents should only have the user message
        let contents = body["contents"].as_array().expect("contents array");
        assert_eq!(contents.len(), 1);
        assert_eq!(contents[0]["role"], "user");
    }

    #[tokio::test]
    async fn test_count_tokens() {
        let provider = GoogleProvider::with_api_key("test-key", "gemini-2.0-flash");
        let count = provider
            .count_tokens("Hello world test")
            .await
            .expect("count");
        assert!(count > 0);
    }

    #[test]
    fn test_parse_response_basic() {
        let provider = GoogleProvider::with_api_key("test-key", "gemini-2.0-flash");
        let resp_json = json!({
            "candidates": [{
                "content": {
                    "parts": [{"text": "Hello, how can I help?"}],
                    "role": "model"
                }
            }],
            "usageMetadata": {
                "promptTokenCount": 10,
                "candidatesTokenCount": 20
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
        let provider = GoogleProvider::with_api_key("test-key", "gemini-2.0-flash");
        let resp_json = json!({
            "candidates": [{
                "content": {
                    "parts": [{
                        "functionCall": {
                            "name": "get_weather",
                            "args": {"location": "San Francisco"}
                        }
                    }],
                    "role": "model"
                }
            }],
            "usageMetadata": {
                "promptTokenCount": 15,
                "candidatesTokenCount": 25
            }
        });

        let response = provider.parse_response(&resp_json).expect("parse");
        assert_eq!(response.tool_calls.len(), 1);
        assert_eq!(response.tool_calls[0].name, "get_weather");
        assert_eq!(response.tool_calls[0].input["location"], "San Francisco");
    }
}
