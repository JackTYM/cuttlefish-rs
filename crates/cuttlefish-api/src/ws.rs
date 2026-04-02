//! WebSocket handler and message protocol.

use axum::{
    extract::ws::{Message as WsMessage, WebSocket, WebSocketUpgrade},
    response::Response,
};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

/// Inbound message from client to server.
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessage {
    /// Chat message for a project.
    Chat {
        /// Project ID.
        project_id: String,
        /// Message content.
        content: String,
    },
    /// Ping for connection keepalive.
    Ping,
}

/// Outbound message from server to client.
#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessage {
    /// Chat response from an agent.
    Response {
        /// Project ID.
        project_id: String,
        /// Agent name.
        agent: String,
        /// Content.
        content: String,
    },
    /// Streaming build log line.
    BuildLog {
        /// Project ID.
        project_id: String,
        /// Log line content.
        line: String,
    },
    /// File diff update.
    Diff {
        /// Project ID.
        project_id: String,
        /// Unified diff patch.
        patch: String,
    },
    /// Pong response.
    Pong,
    /// Error message.
    Error {
        /// Error message.
        message: String,
    },
}

impl ServerMessage {
    /// Serialize to JSON string.
    pub fn to_json(&self) -> String {
        serde_json::to_string(self)
            .unwrap_or_else(|_| r#"{"type":"error","message":"serialization failed"}"#.to_string())
    }
}

/// Handle a WebSocket upgrade request.
pub async fn ws_handler(ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(handle_socket)
}

/// Handle an established WebSocket connection.
async fn handle_socket(mut socket: WebSocket) {
    info!("New WebSocket connection");

    while let Some(msg) = socket.recv().await {
        match msg {
            Ok(WsMessage::Text(text)) => {
                debug!("Received WebSocket text: {} bytes", text.len());

                match serde_json::from_str::<ClientMessage>(&text) {
                    Ok(ClientMessage::Ping) => {
                        let response = ServerMessage::Pong.to_json();
                        if socket.send(WsMessage::Text(response.into())).await.is_err() {
                            break;
                        }
                    }
                    Ok(ClientMessage::Chat {
                        project_id,
                        content,
                    }) => {
                        debug!(
                            "Chat for project {}: {}",
                            project_id,
                            &content[..content.len().min(50)]
                        );
                        let response = ServerMessage::Response {
                            project_id: project_id.clone(),
                            agent: "orchestrator".to_string(),
                            content: format!("Received: {}", content),
                        }
                        .to_json();
                        if socket.send(WsMessage::Text(response.into())).await.is_err() {
                            break;
                        }
                    }
                    Err(e) => {
                        warn!("Invalid WebSocket message: {}", e);
                        let err = ServerMessage::Error {
                            message: format!("Invalid message: {e}"),
                        }
                        .to_json();
                        if socket.send(WsMessage::Text(err.into())).await.is_err() {
                            break;
                        }
                    }
                }
            }
            Ok(WsMessage::Close(_)) => {
                info!("WebSocket client disconnected");
                break;
            }
            Ok(_) => {} // Binary, ping, pong — ignore
            Err(e) => {
                warn!("WebSocket error: {}", e);
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_message_pong_serializes() {
        let json = ServerMessage::Pong.to_json();
        assert!(json.contains("pong"));
    }

    #[test]
    fn test_server_message_error_serializes() {
        let json = ServerMessage::Error {
            message: "test error".to_string(),
        }
        .to_json();
        assert!(json.contains("test error"));
        assert!(json.contains("error"));
    }

    #[test]
    fn test_server_message_response_serializes() {
        let json = ServerMessage::Response {
            project_id: "proj-1".to_string(),
            agent: "coder".to_string(),
            content: "Done".to_string(),
        }
        .to_json();
        assert!(json.contains("response"));
        assert!(json.contains("proj-1"));
    }

    #[test]
    fn test_client_message_chat_deserializes() {
        let json = r#"{"type":"chat","project_id":"p1","content":"Hello"}"#;
        let msg: ClientMessage = serde_json::from_str(json).expect("parse");
        if let ClientMessage::Chat {
            project_id,
            content,
        } = msg
        {
            assert_eq!(project_id, "p1");
            assert_eq!(content, "Hello");
        } else {
            panic!("Expected Chat variant");
        }
    }

    #[test]
    fn test_client_message_ping_deserializes() {
        let json = r#"{"type":"ping"}"#;
        let msg: ClientMessage = serde_json::from_str(json).expect("parse");
        assert!(matches!(msg, ClientMessage::Ping));
    }
}
