//! Message bus implementation using tokio broadcast channels.

use async_trait::async_trait;
use cuttlefish_core::{
    error::AgentError,
    traits::bus::{BusMessage, MessageBus},
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, broadcast};
use tracing::debug;

const CHANNEL_CAPACITY: usize = 256;

/// Tokio broadcast-based message bus for agent communication.
#[derive(Clone)]
pub struct TokioMessageBus {
    channels: Arc<Mutex<HashMap<String, broadcast::Sender<BusMessage>>>>,
}

impl TokioMessageBus {
    /// Create a new message bus.
    pub fn new() -> Self {
        Self {
            channels: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    async fn get_or_create(&self, topic: &str) -> broadcast::Sender<BusMessage> {
        let mut channels = self.channels.lock().await;
        if let Some(s) = channels.get(topic) {
            s.clone()
        } else {
            let (s, _) = broadcast::channel(CHANNEL_CAPACITY);
            channels.insert(topic.to_string(), s.clone());
            s
        }
    }
}

impl Default for TokioMessageBus {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl MessageBus for TokioMessageBus {
    async fn publish(&self, message: BusMessage) -> Result<(), AgentError> {
        let s = self.get_or_create(&message.topic).await;
        debug!("Publishing to {}", message.topic);
        let _ = s.send(message);
        Ok(())
    }

    async fn subscribe(&self, topic: &str) -> Result<broadcast::Receiver<BusMessage>, AgentError> {
        let s = self.get_or_create(topic).await;
        debug!("Subscribing to {}", topic);
        Ok(s.subscribe())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_subscribe_then_publish() {
        let bus = TokioMessageBus::new();
        let mut rx = bus.subscribe("t").await.expect("sub");
        let msg = BusMessage::new("t", serde_json::json!({}));
        bus.publish(msg).await.expect("pub");
        assert!(rx.recv().await.is_ok());
    }

    #[tokio::test]
    async fn test_multiple_subscribers() {
        let bus = TokioMessageBus::new();
        let mut rx1 = bus.subscribe("c").await.expect("s1");
        let mut rx2 = bus.subscribe("c").await.expect("s2");
        bus.publish(BusMessage::new("c", serde_json::json!({})))
            .await
            .expect("pub");
        assert!(rx1.recv().await.is_ok());
        assert!(rx2.recv().await.is_ok());
    }

    #[tokio::test]
    async fn test_isolated_topics() {
        let bus = TokioMessageBus::new();
        let mut rx_a = bus.subscribe("a").await.expect("sub");
        let _rx_b = bus.subscribe("b").await.expect("sub");
        bus.publish(BusMessage::new("b", serde_json::json!({})))
            .await
            .expect("pub");
        assert!(rx_a.try_recv().is_err());
    }
}
