//! AWS Bedrock model provider implementation.

use async_trait::async_trait;
use aws_sdk_bedrockruntime::{
    Client,
    types::{ContentBlock, ConversationRole, Message as BedrockMessage, SystemContentBlock},
};
use aws_smithy_types::Document;
use cuttlefish_core::{
    error::ProviderError,
    traits::provider::{
        CompletionRequest, CompletionResponse, MessageRole, ModelProvider, StreamChunk, ToolCall,
    },
};
use futures::stream::{self, BoxStream, StreamExt};
use tracing::{debug, instrument};

/// AWS Bedrock model provider supporting Claude models.
///
/// Uses the Bedrock `converse` API to communicate with Claude models.
/// Supports both synchronous completion and pseudo-streaming (complete + chunk).
pub struct BedrockProvider {
    /// The Bedrock runtime client.
    client: Client,
    /// The model ID (e.g., `anthropic.claude-sonnet-4-6-20260101-v1:0`).
    model_id: String,
}

impl BedrockProvider {
    /// Create a new Bedrock provider with the given model ID.
    ///
    /// Uses the default AWS credential chain (env vars, profile, IAM role).
    pub async fn new(model_id: impl Into<String>) -> Result<Self, ProviderError> {
        let config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
        let client = Client::new(&config);
        Ok(Self {
            client,
            model_id: model_id.into(),
        })
    }

    /// Create with an explicit Bedrock runtime client (useful for testing/custom regions).
    pub fn with_client(client: Client, model_id: impl Into<String>) -> Self {
        Self {
            client,
            model_id: model_id.into(),
        }
    }
}

/// Convert an AWS Smithy [`Document`] to a [`serde_json::Value`].
fn document_to_json(doc: &Document) -> serde_json::Value {
    match doc {
        Document::Object(map) => {
            let obj: serde_json::Map<String, serde_json::Value> = map
                .iter()
                .map(|(k, v)| (k.clone(), document_to_json(v)))
                .collect();
            serde_json::Value::Object(obj)
        }
        Document::Array(arr) => {
            serde_json::Value::Array(arr.iter().map(document_to_json).collect())
        }
        Document::Number(n) => {
            if let Ok(i) = i64::try_from(*n) {
                serde_json::Value::Number(i.into())
            } else {
                let f = (*n).to_f64_lossy();
                serde_json::Number::from_f64(f)
                    .map(serde_json::Value::Number)
                    .unwrap_or(serde_json::Value::Null)
            }
        }
        Document::String(s) => serde_json::Value::String(s.clone()),
        Document::Bool(b) => serde_json::Value::Bool(*b),
        Document::Null => serde_json::Value::Null,
    }
}

/// Convert our [`cuttlefish_core::traits::provider::Message`] to Bedrock's
/// [`BedrockMessage`] type.
///
/// System messages are not valid here — they must be passed via the
/// system parameter on the Bedrock request.
fn convert_message(
    msg: &cuttlefish_core::traits::provider::Message,
) -> Result<BedrockMessage, ProviderError> {
    let role = match msg.role {
        MessageRole::User => ConversationRole::User,
        MessageRole::Assistant => ConversationRole::Assistant,
        MessageRole::System => {
            return Err(ProviderError(
                "System messages must use the system parameter, not the messages array".to_string(),
            ));
        }
    };

    BedrockMessage::builder()
        .role(role)
        .content(ContentBlock::Text(msg.content.clone()))
        .build()
        .map_err(|e| ProviderError(format!("Failed to build message: {e}")))
}

#[async_trait]
impl ModelProvider for BedrockProvider {
    fn name(&self) -> &str {
        "bedrock"
    }

    fn model(&self) -> Option<&str> {
        Some(&self.model_id)
    }

    #[instrument(skip(self, request), fields(model = %self.model_id))]
    async fn complete(
        &self,
        request: CompletionRequest,
    ) -> Result<CompletionResponse, ProviderError> {
        let (system_msgs, conv_msgs): (Vec<_>, Vec<_>) = request
            .messages
            .iter()
            .partition(|m| m.role == MessageRole::System);

        let mut system_blocks = Vec::new();
        if let Some(sys) = &request.system {
            system_blocks.push(SystemContentBlock::Text(sys.clone()));
        }
        for sys_msg in &system_msgs {
            system_blocks.push(SystemContentBlock::Text(sys_msg.content.clone()));
        }

        let bedrock_messages: Result<Vec<_>, _> =
            conv_msgs.iter().map(|m| convert_message(m)).collect();
        let bedrock_messages = bedrock_messages?;

        debug!(
            "Sending {} messages to Bedrock model {}",
            bedrock_messages.len(),
            self.model_id
        );

        let mut req_builder = self
            .client
            .converse()
            .model_id(&self.model_id)
            .set_messages(Some(bedrock_messages));

        if !system_blocks.is_empty() {
            req_builder = req_builder.set_system(Some(system_blocks));
        }

        if let Some(max_tokens) = request.max_tokens {
            let mut inference_builder =
                aws_sdk_bedrockruntime::types::InferenceConfiguration::builder()
                    .max_tokens(max_tokens as i32);

            if let Some(temp) = request.temperature {
                inference_builder = inference_builder.temperature(temp);
            }

            let inference_config = inference_builder.build();
            req_builder = req_builder.inference_config(inference_config);
        }

        let response = req_builder.send().await.map_err(|e| {
            // Extract detailed error information from the AWS SDK error
            let (error_code, error_message) = match &e {
                aws_sdk_bedrockruntime::error::SdkError::ServiceError(service_err) => {
                    let err = service_err.err();
                    let code = err
                        .meta()
                        .code()
                        .unwrap_or("UnknownError")
                        .to_string();
                    let msg = err
                        .meta()
                        .message()
                        .unwrap_or("No message provided")
                        .to_string();
                    (code, msg)
                }
                aws_sdk_bedrockruntime::error::SdkError::ConstructionFailure(err) => {
                    ("ConstructionFailure".to_string(), format!("{err:?}"))
                }
                aws_sdk_bedrockruntime::error::SdkError::TimeoutError(_) => {
                    ("TimeoutError".to_string(), "Request timed out".to_string())
                }
                aws_sdk_bedrockruntime::error::SdkError::DispatchFailure(err) => {
                    ("DispatchFailure".to_string(), format!("{err:?}"))
                }
                _ => ("UnknownError".to_string(), format!("{e}")),
            };

            let hint = match error_code.as_str() {
                "AccessDeniedException" => {
                    " (Hint: Request model access in AWS Bedrock console)"
                }
                "ValidationException" => {
                    " (Hint: Check model ID format, e.g., anthropic.claude-sonnet-4-6-20250514-v1:0)"
                }
                "ResourceNotFoundException" => {
                    " (Hint: Model may not be available in this region)"
                }
                "ThrottlingException" => " (Hint: Rate limited, try again later)",
                "ModelNotReadyException" => " (Hint: Model is still warming up, try again)",
                "ModelStreamErrorException" => " (Hint: Streaming error, try non-streaming)",
                _ => "",
            };

            ProviderError(format!(
                "Bedrock API error for model '{}': [{}] {}{hint}",
                self.model_id, error_code, error_message
            ))
        })?;

        let mut content = String::new();
        let mut tool_calls = Vec::new();

        if let Some(output) = response.output {
            use aws_sdk_bedrockruntime::types::ConverseOutput;
            if let ConverseOutput::Message(msg) = output {
                for block in msg.content {
                    match block {
                        ContentBlock::Text(text) => content.push_str(&text),
                        ContentBlock::ToolUse(tool_use) => {
                            let input = document_to_json(&tool_use.input);
                            tool_calls.push(ToolCall {
                                id: tool_use.tool_use_id,
                                name: tool_use.name,
                                input,
                            });
                        }
                        _ => {}
                    }
                }
            }
        }

        let usage = response.usage;

        Ok(CompletionResponse {
            content,
            input_tokens: usage.as_ref().map(|u| u.input_tokens as u32).unwrap_or(0),
            output_tokens: usage.as_ref().map(|u| u.output_tokens as u32).unwrap_or(0),
            model: self.model_id.clone(),
            tool_calls,
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
    fn test_convert_user_message() {
        let msg = Message {
            role: MessageRole::User,
            content: "Hello".to_string(),
        };
        let result = convert_message(&msg);
        assert!(result.is_ok());
    }

    #[test]
    fn test_convert_assistant_message() {
        let msg = Message {
            role: MessageRole::Assistant,
            content: "Hi there".to_string(),
        };
        let result = convert_message(&msg);
        assert!(result.is_ok());
    }

    #[test]
    fn test_convert_system_message_fails() {
        let msg = Message {
            role: MessageRole::System,
            content: "System instructions".to_string(),
        };
        let result = convert_message(&msg);
        assert!(result.is_err());
    }
}
