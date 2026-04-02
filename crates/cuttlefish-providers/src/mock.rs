//! Mock model provider for testing.

use async_trait::async_trait;
use cuttlefish_core::{
    error::ProviderError,
    traits::provider::{CompletionRequest, CompletionResponse, ModelProvider, StreamChunk},
};
use futures::StreamExt;
use futures::stream::{self, BoxStream};
use std::sync::{Arc, Mutex};

/// A mock model provider for use in tests.
///
/// Can be configured with canned responses that are returned in order.
/// When all canned responses are consumed, returns a default response.
pub struct MockModelProvider {
    /// Pre-configured responses to return (in order).
    responses: Arc<Mutex<Vec<String>>>,
    /// Name of this mock provider.
    name: String,
}

impl MockModelProvider {
    /// Create a new mock provider with the given name.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            responses: Arc::new(Mutex::new(Vec::new())),
            name: name.into(),
        }
    }

    /// Add a canned response that will be returned on the next `complete()` call.
    pub fn add_response(&self, response: impl Into<String>) {
        let mut responses = self.responses.lock().expect("mutex poisoned");
        responses.push(response.into());
    }
}

impl Default for MockModelProvider {
    fn default() -> Self {
        let mock = Self::new("mock");
        mock.add_response("Mock response from model.");
        mock
    }
}

#[async_trait]
impl ModelProvider for MockModelProvider {
    fn name(&self) -> &str {
        &self.name
    }

    async fn complete(
        &self,
        _request: CompletionRequest,
    ) -> Result<CompletionResponse, ProviderError> {
        let content = {
            let mut responses = self.responses.lock().expect("mutex poisoned");
            if responses.is_empty() {
                "Default mock response.".to_string()
            } else {
                responses.remove(0)
            }
        };

        Ok(CompletionResponse {
            content,
            input_tokens: 10,
            output_tokens: 5,
            model: self.name.clone(),
            tool_calls: Vec::new(),
        })
    }

    fn stream<'a>(
        &'a self,
        request: CompletionRequest,
    ) -> BoxStream<'a, Result<StreamChunk, ProviderError>> {
        let content = {
            let mut responses = self.responses.lock().expect("mutex poisoned");
            if responses.is_empty() {
                "Default mock stream response.".to_string()
            } else {
                responses.remove(0)
            }
        };
        let _ = request;
        stream::iter(vec![
            Ok(StreamChunk::Text(content)),
            Ok(StreamChunk::Usage {
                input_tokens: 10,
                output_tokens: 5,
            }),
        ])
        .boxed()
    }

    async fn count_tokens(&self, text: &str) -> Result<usize, ProviderError> {
        Ok(text.len() / 4 + 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cuttlefish_core::traits::provider::{CompletionRequest, Message, MessageRole};
    use futures::StreamExt;

    fn test_request() -> CompletionRequest {
        CompletionRequest {
            messages: vec![Message {
                role: MessageRole::User,
                content: "Test message".to_string(),
            }],
            max_tokens: Some(100),
            temperature: Some(0.7),
            system: None,
        }
    }

    #[tokio::test]
    async fn test_mock_complete_with_canned_response() {
        let mock = MockModelProvider::new("test-mock");
        mock.add_response("This is the test response");
        let result = mock.complete(test_request()).await.expect("complete");
        assert_eq!(result.content, "This is the test response");
        assert_eq!(result.model, "test-mock");
    }

    #[tokio::test]
    async fn test_mock_complete_returns_default_when_empty() {
        let mock = MockModelProvider::new("test-mock");
        let result = mock.complete(test_request()).await.expect("complete");
        assert_eq!(result.content, "Default mock response.");
    }

    #[tokio::test]
    async fn test_mock_stream_emits_text_and_usage() {
        let mock = MockModelProvider::new("test-mock");
        mock.add_response("Streamed response");
        let mut stream = mock.stream(test_request());

        let chunk1 = stream.next().await.expect("chunk1").expect("ok");
        assert!(matches!(chunk1, StreamChunk::Text(_)));

        let chunk2 = stream.next().await.expect("chunk2").expect("ok");
        assert!(matches!(chunk2, StreamChunk::Usage { .. }));
    }

    #[tokio::test]
    async fn test_mock_count_tokens() {
        let mock = MockModelProvider::default();
        let count = mock.count_tokens("Hello world").await.expect("count");
        assert!(count > 0);
    }

    #[tokio::test]
    async fn test_mock_multiple_responses_consumed_in_order() {
        let mock = MockModelProvider::new("test-mock");
        mock.add_response("First");
        mock.add_response("Second");
        mock.add_response("Third");

        let r1 = mock.complete(test_request()).await.expect("r1");
        let r2 = mock.complete(test_request()).await.expect("r2");
        let r3 = mock.complete(test_request()).await.expect("r3");

        assert_eq!(r1.content, "First");
        assert_eq!(r2.content, "Second");
        assert_eq!(r3.content, "Third");
    }

    #[tokio::test]
    async fn test_mock_name() {
        let mock = MockModelProvider::new("my-provider");
        assert_eq!(mock.name(), "my-provider");
    }

    #[tokio::test]
    async fn test_mock_default_has_response() {
        let mock = MockModelProvider::default();
        let result = mock.complete(test_request()).await.expect("complete");
        assert_eq!(result.content, "Mock response from model.");
    }
}
