#![deny(clippy::unwrap_used)]

//! Tunnel protocol message types for WebSocket communication.
//!
//! This module defines the message types exchanged between tunnel clients and servers,
//! along with serialization helpers and request ID generation.

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};

/// Global request ID counter for generating unique request identifiers
static REQUEST_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Generate a unique request ID for HTTP requests
pub fn generate_request_id() -> u64 {
    REQUEST_ID_COUNTER.fetch_add(1, Ordering::SeqCst)
}

/// Messages sent from client to server
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessage {
    /// Initial authentication with link code
    Auth {
        /// The link code for authentication
        link_code: String,
    },
    /// Re-authentication with JWT token
    AuthToken {
        /// The JWT token for re-authentication
        jwt: String,
    },
    /// Response to an HTTP request from server
    HttpResponse {
        /// Unique identifier for the request being responded to
        request_id: u64,
        /// HTTP status code
        status: u16,
        /// HTTP response headers
        headers: Vec<(String, String)>,
        /// HTTP response body
        body: Vec<u8>,
    },
    /// Keepalive heartbeat
    Heartbeat,
}

impl ClientMessage {
    /// Serialize this message to JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Deserialize a message from JSON
    pub fn from_json(s: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(s)
    }
}

/// Messages sent from server to client
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessage {
    /// Successful authentication
    AuthSuccess {
        /// JWT token for future authentication
        jwt: String,
        /// Assigned subdomain for the tunnel
        subdomain: String,
    },
    /// Authentication failed
    AuthFailure {
        /// Reason for authentication failure
        reason: String,
    },
    /// HTTP request to forward to local server
    HttpRequest {
        /// Unique identifier for this request
        request_id: u64,
        /// HTTP method (GET, POST, etc.)
        method: String,
        /// Request path
        path: String,
        /// HTTP request headers
        headers: Vec<(String, String)>,
        /// HTTP request body
        body: Vec<u8>,
    },
    /// Heartbeat acknowledgment
    HeartbeatAck,
    /// Server-initiated disconnect
    Disconnect {
        /// Reason for disconnection
        reason: String,
    },
}

impl ServerMessage {
    /// Serialize this message to JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Deserialize a message from JSON
    pub fn from_json(s: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(s)
    }
}

#[cfg(test)]
mod tests {
    #![deny(clippy::unwrap_used)]

    use super::*;

    #[test]
    fn test_client_message_auth_roundtrip() {
        let original = ClientMessage::Auth {
            link_code: "test-link-code-123".to_string(),
        };

        let json = original
            .to_json()
            .expect("Failed to serialize ClientMessage::Auth");
        let deserialized =
            ClientMessage::from_json(&json).expect("Failed to deserialize ClientMessage::Auth");

        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_client_message_auth_token_roundtrip() {
        let original = ClientMessage::AuthToken {
            jwt: "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9".to_string(),
        };

        let json = original
            .to_json()
            .expect("Failed to serialize ClientMessage::AuthToken");
        let deserialized = ClientMessage::from_json(&json)
            .expect("Failed to deserialize ClientMessage::AuthToken");

        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_client_message_http_response_roundtrip() {
        let original = ClientMessage::HttpResponse {
            request_id: 42,
            status: 200,
            headers: vec![
                ("content-type".to_string(), "application/json".to_string()),
                ("x-custom-header".to_string(), "custom-value".to_string()),
            ],
            body: b"Hello, World!".to_vec(),
        };

        let json = original
            .to_json()
            .expect("Failed to serialize ClientMessage::HttpResponse");
        let deserialized = ClientMessage::from_json(&json)
            .expect("Failed to deserialize ClientMessage::HttpResponse");

        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_client_message_heartbeat_roundtrip() {
        let original = ClientMessage::Heartbeat;

        let json = original
            .to_json()
            .expect("Failed to serialize ClientMessage::Heartbeat");
        let deserialized = ClientMessage::from_json(&json)
            .expect("Failed to deserialize ClientMessage::Heartbeat");

        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_server_message_auth_success_roundtrip() {
        let original = ServerMessage::AuthSuccess {
            jwt: "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9".to_string(),
            subdomain: "my-tunnel".to_string(),
        };

        let json = original
            .to_json()
            .expect("Failed to serialize ServerMessage::AuthSuccess");
        let deserialized = ServerMessage::from_json(&json)
            .expect("Failed to deserialize ServerMessage::AuthSuccess");

        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_server_message_auth_failure_roundtrip() {
        let original = ServerMessage::AuthFailure {
            reason: "Invalid link code".to_string(),
        };

        let json = original
            .to_json()
            .expect("Failed to serialize ServerMessage::AuthFailure");
        let deserialized = ServerMessage::from_json(&json)
            .expect("Failed to deserialize ServerMessage::AuthFailure");

        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_server_message_http_request_roundtrip() {
        let original = ServerMessage::HttpRequest {
            request_id: 123,
            method: "POST".to_string(),
            path: "/api/users".to_string(),
            headers: vec![
                ("content-type".to_string(), "application/json".to_string()),
                ("authorization".to_string(), "Bearer token123".to_string()),
            ],
            body: br#"{"name":"John","age":30}"#.to_vec(),
        };

        let json = original
            .to_json()
            .expect("Failed to serialize ServerMessage::HttpRequest");
        let deserialized = ServerMessage::from_json(&json)
            .expect("Failed to deserialize ServerMessage::HttpRequest");

        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_server_message_heartbeat_ack_roundtrip() {
        let original = ServerMessage::HeartbeatAck;

        let json = original
            .to_json()
            .expect("Failed to serialize ServerMessage::HeartbeatAck");
        let deserialized = ServerMessage::from_json(&json)
            .expect("Failed to deserialize ServerMessage::HeartbeatAck");

        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_server_message_disconnect_roundtrip() {
        let original = ServerMessage::Disconnect {
            reason: "Server maintenance".to_string(),
        };

        let json = original
            .to_json()
            .expect("Failed to serialize ServerMessage::Disconnect");
        let deserialized = ServerMessage::from_json(&json)
            .expect("Failed to deserialize ServerMessage::Disconnect");

        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_invalid_json_returns_error() {
        let invalid_json = r#"{"type":"invalid_type","data":"test"}"#;
        let result = ClientMessage::from_json(invalid_json);
        assert!(result.is_err(), "Expected error for invalid JSON");
    }

    #[test]
    fn test_malformed_json_returns_error() {
        let malformed_json = r#"{"type":"auth","link_code":}"#;
        let result = ClientMessage::from_json(malformed_json);
        assert!(result.is_err(), "Expected error for malformed JSON");
    }

    #[test]
    fn test_generate_request_id_increments() {
        let id1 = generate_request_id();
        let id2 = generate_request_id();
        let id3 = generate_request_id();

        assert!(id2 > id1, "Request IDs should increment");
        assert!(id3 > id2, "Request IDs should increment");
    }

    #[test]
    fn test_empty_headers_and_body() {
        let original = ClientMessage::HttpResponse {
            request_id: 1,
            status: 204,
            headers: vec![],
            body: vec![],
        };

        let json = original
            .to_json()
            .expect("Failed to serialize empty response");
        let deserialized =
            ClientMessage::from_json(&json).expect("Failed to deserialize empty response");

        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_binary_body_preservation() {
        let binary_body = vec![0u8, 1, 2, 255, 254, 253];
        let original = ClientMessage::HttpResponse {
            request_id: 99,
            status: 200,
            headers: vec![(
                "content-type".to_string(),
                "application/octet-stream".to_string(),
            )],
            body: binary_body.clone(),
        };

        let json = original.to_json().expect("Failed to serialize binary body");
        let deserialized =
            ClientMessage::from_json(&json).expect("Failed to deserialize binary body");

        assert_eq!(original, deserialized);
        if let ClientMessage::HttpResponse { body, .. } = deserialized {
            assert_eq!(body, binary_body, "Binary body should be preserved");
        } else {
            panic!("Expected HttpResponse variant");
        }
    }
}
