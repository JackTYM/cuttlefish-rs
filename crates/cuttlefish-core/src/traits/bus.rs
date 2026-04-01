//! Message bus trait and supporting types.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::AgentError;

/// A message on the bus.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusMessage {
    /// Unique message ID.
    pub id: Uuid,
    /// Message type/topic.
    pub topic: String,
    /// Message payload as JSON.
    pub payload: serde_json::Value,
    /// Timestamp (Unix millis).
    pub timestamp_ms: u64,
}

impl BusMessage {
    /// Create a new message.
    pub fn new(topic: impl Into<String>, payload: serde_json::Value) -> Self {
        let timestamp_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        Self {
            id: Uuid::new_v4(),
            topic: topic.into(),
            payload,
            timestamp_ms,
        }
    }
}

/// An async message bus for agent communication.
#[async_trait]
pub trait MessageBus: Send + Sync {
    /// Publish a message to a topic.
    async fn publish(&self, message: BusMessage) -> Result<(), AgentError>;

    /// Subscribe to a topic, returning a receiver for messages.
    async fn subscribe(
        &self,
        topic: &str,
    ) -> Result<tokio::sync::broadcast::Receiver<BusMessage>, AgentError>;
}
