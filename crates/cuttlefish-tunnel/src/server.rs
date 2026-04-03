//! Tunnel server — accepts client connections and routes HTTP requests.
//!
//! The tunnel server listens for WebSocket connections from tunnel clients
//! and routes incoming HTTP requests to the appropriate client based on subdomain.

#![deny(clippy::unwrap_used)]

use crate::auth::{validate_jwt, validate_link_code};
use crate::error::TunnelError;
use crate::protocol::{ClientMessage, ServerMessage, generate_request_id};
use chrono::{DateTime, Utc};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{RwLock, mpsc, oneshot};
use tokio_tungstenite::{WebSocketStream, accept_async, tungstenite::Message};
use tracing::{debug, info, warn};

/// Statistics for a tunnel connection
#[derive(Debug, Clone)]
pub struct TunnelStats {
    /// Bytes received from client
    pub bytes_in: Arc<AtomicU64>,
    /// Bytes sent to client
    pub bytes_out: Arc<AtomicU64>,
    /// Number of HTTP requests handled
    pub requests_handled: Arc<AtomicU64>,
    /// When the connection was established
    pub connected_at: DateTime<Utc>,
}

impl TunnelStats {
    /// Create new tunnel statistics
    pub fn new() -> Self {
        Self {
            bytes_in: Arc::new(AtomicU64::new(0)),
            bytes_out: Arc::new(AtomicU64::new(0)),
            requests_handled: Arc::new(AtomicU64::new(0)),
            connected_at: Utc::now(),
        }
    }

    /// Get bytes received
    pub fn get_bytes_in(&self) -> u64 {
        self.bytes_in.load(Ordering::Relaxed)
    }

    /// Get bytes sent
    pub fn get_bytes_out(&self) -> u64 {
        self.bytes_out.load(Ordering::Relaxed)
    }

    /// Get requests handled
    pub fn get_requests_handled(&self) -> u64 {
        self.requests_handled.load(Ordering::Relaxed)
    }

    /// Add bytes received
    pub fn add_bytes_in(&self, bytes: u64) {
        self.bytes_in.fetch_add(bytes, Ordering::Relaxed);
    }

    /// Add bytes sent
    pub fn add_bytes_out(&self, bytes: u64) {
        self.bytes_out.fetch_add(bytes, Ordering::Relaxed);
    }

    /// Increment request count
    pub fn increment_requests(&self) {
        self.requests_handled.fetch_add(1, Ordering::Relaxed);
    }
}

impl Default for TunnelStats {
    fn default() -> Self {
        Self::new()
    }
}

/// Information about a single tunnel
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TunnelInfo {
    /// Subdomain for this tunnel
    pub subdomain: String,
    /// When the tunnel was connected
    pub connected_since: String,
    /// Last heartbeat timestamp
    pub last_heartbeat: String,
    /// Bytes received from client
    pub bytes_in: u64,
    /// Bytes sent to client
    pub bytes_out: u64,
    /// Number of HTTP requests handled
    pub requests_handled: u64,
}

/// Response for tunnel status endpoint
#[derive(Debug, Serialize, Deserialize)]
pub struct TunnelStatusResponse {
    /// Number of active tunnels
    pub active_tunnels: u32,
    /// Details for each tunnel
    pub tunnels: Vec<TunnelInfo>,
}
#[allow(dead_code)]
struct TunnelConnection {
    /// Sender to forward requests to this client
    request_tx: mpsc::Sender<(ServerMessage, oneshot::Sender<ClientMessage>)>,
    /// When the connection was established
    connected_at: Instant,
    /// Last heartbeat received
    last_heartbeat: Instant,
    /// User ID (from JWT claims)
    user_id: String,
    /// Statistics for this tunnel
    stats: TunnelStats,
}

/// Tunnel server configuration
#[derive(Debug, Clone)]
pub struct TunnelServerConfig {
    /// Address to listen on for WebSocket connections
    pub listen_addr: SocketAddr,
    /// Address to listen on for HTTP requests to route
    pub http_addr: SocketAddr,
    /// JWT secret for token validation
    pub jwt_secret: Vec<u8>,
    /// Heartbeat timeout (disconnect if no heartbeat)
    pub heartbeat_timeout: Duration,
    /// Request timeout for forwarded HTTP requests
    pub request_timeout: Duration,
}

impl Default for TunnelServerConfig {
    fn default() -> Self {
        Self {
            listen_addr: SocketAddr::from(([0, 0, 0, 0], 8082)),
            http_addr: SocketAddr::from(([0, 0, 0, 0], 8081)),
            jwt_secret: Vec::new(), // Must be set from env
            heartbeat_timeout: Duration::from_secs(90),
            request_timeout: Duration::from_secs(30),
        }
    }
}

/// Tunnel server that manages client connections
pub struct TunnelServer {
    config: TunnelServerConfig,
    /// Map of subdomain -> connection
    connections: Arc<RwLock<HashMap<String, TunnelConnection>>>,
}

impl TunnelServer {
    /// Create a new tunnel server with the given configuration.
    pub fn new(config: TunnelServerConfig) -> Self {
        Self {
            config,
            connections: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Run the tunnel server.
    ///
    /// This starts:
    /// - WebSocket listener for client connections
    /// - HTTP listener for incoming requests to route
    /// - Background task for cleaning up stale connections
    pub async fn run(&self) -> Result<(), TunnelError> {
        let ws_listener = TcpListener::bind(self.config.listen_addr).await?;
        info!(
            "Tunnel WebSocket server listening on {}",
            self.config.listen_addr
        );

        // Spawn cleanup task
        let connections = Arc::clone(&self.connections);
        let heartbeat_timeout = self.config.heartbeat_timeout;
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30));
            loop {
                interval.tick().await;
                cleanup_stale_connections_inner(&connections, heartbeat_timeout).await;
            }
        });

        // Accept WebSocket connections
        loop {
            let (stream, addr) = ws_listener.accept().await?;
            debug!("New connection from {}", addr);

            let connections = Arc::clone(&self.connections);
            let config = self.config.clone();

            tokio::spawn(async move {
                if let Err(e) = handle_client_connection(stream, addr, connections, config).await {
                    warn!("Client {} disconnected with error: {}", addr, e);
                }
            });
        }
    }

    /// Route an HTTP request to the appropriate tunnel client.
    ///
    /// Returns the response status, headers, and body.
    pub async fn route_request(
        &self,
        subdomain: &str,
        method: &str,
        path: &str,
        headers: Vec<(String, String)>,
        body: Vec<u8>,
    ) -> Result<(u16, Vec<(String, String)>, Vec<u8>), TunnelError> {
        let connections = self.connections.read().await;
        let conn = connections
            .get(subdomain)
            .ok_or_else(|| TunnelError::TunnelNotFound(subdomain.to_string()))?;

        let request_body_len = body.len() as u64;
        let request_id = generate_request_id();
        let request = ServerMessage::HttpRequest {
            request_id,
            method: method.to_string(),
            path: path.to_string(),
            headers,
            body,
        };

        let (response_tx, response_rx) = oneshot::channel();

        conn.request_tx
            .send((request, response_tx))
            .await
            .map_err(|_| TunnelError::ConnectionClosed("Failed to send request".to_string()))?;

        conn.stats.add_bytes_in(request_body_len);
        conn.stats.increment_requests();

        drop(connections); // Release read lock while waiting

        let timeout = self.config.request_timeout;
        let response = tokio::time::timeout(timeout, response_rx)
            .await
            .map_err(|_| TunnelError::Timeout(timeout.as_millis() as u64))?
            .map_err(|_| TunnelError::ConnectionClosed("Response channel closed".to_string()))?;

        match response {
            ClientMessage::HttpResponse {
                status,
                headers,
                body,
                ..
            } => {
                let response_body_len = body.len() as u64;
                let connections = self.connections.read().await;
                if let Some(conn) = connections.get(subdomain) {
                    conn.stats.add_bytes_out(response_body_len);
                }
                Ok((status, headers, body))
            }
            _ => Err(TunnelError::HttpForward(
                "Unexpected response type".to_string(),
            )),
        }
    }

    /// List all active tunnel subdomains.
    pub async fn active_tunnels(&self) -> Vec<String> {
        self.connections.read().await.keys().cloned().collect()
    }

    /// Get the number of active connections.
    pub async fn connection_count(&self) -> usize {
        self.connections.read().await.len()
    }

    /// Manually disconnect a tunnel by subdomain.
    pub async fn disconnect(&self, subdomain: &str) -> bool {
        self.connections.write().await.remove(subdomain).is_some()
    }

    /// Get status information for all active tunnels.
    pub async fn get_status(&self) -> TunnelStatusResponse {
        let connections = self.connections.read().await;
        let mut tunnels = Vec::new();

        for (subdomain, conn) in connections.iter() {
            let connected_since = conn.stats.connected_at.to_rfc3339();
            let last_heartbeat = {
                let elapsed = conn.last_heartbeat.elapsed();
                let heartbeat_time =
                    Utc::now() - chrono::Duration::seconds(elapsed.as_secs() as i64);
                heartbeat_time.to_rfc3339()
            };

            tunnels.push(TunnelInfo {
                subdomain: subdomain.clone(),
                connected_since,
                last_heartbeat,
                bytes_in: conn.stats.get_bytes_in(),
                bytes_out: conn.stats.get_bytes_out(),
                requests_handled: conn.stats.get_requests_handled(),
            });
        }

        TunnelStatusResponse {
            active_tunnels: tunnels.len() as u32,
            tunnels,
        }
    }
}

/// Handle a single client connection.
async fn handle_client_connection(
    stream: TcpStream,
    addr: SocketAddr,
    connections: Arc<RwLock<HashMap<String, TunnelConnection>>>,
    config: TunnelServerConfig,
) -> Result<(), TunnelError> {
    let ws_stream = accept_async(stream).await?;
    info!("WebSocket connection established from {}", addr);

    let (mut write, mut read) = ws_stream.split();

    // Wait for authentication message
    let auth_msg = tokio::time::timeout(Duration::from_secs(10), read.next())
        .await
        .map_err(|_| TunnelError::Timeout(10000))?
        .ok_or_else(|| TunnelError::ConnectionClosed("No auth message received".to_string()))?
        .map_err(TunnelError::from)?;

    let auth_text = match auth_msg {
        Message::Text(t) => t.to_string(),
        _ => {
            return Err(TunnelError::AuthFailed(
                "Expected text message for auth".to_string(),
            ));
        }
    };

    let client_msg = ClientMessage::from_json(&auth_text)?;

    // Validate authentication
    let (jwt, subdomain, user_id) = match client_msg {
        ClientMessage::Auth { link_code } => {
            let result = validate_link_code(&link_code, &config.jwt_secret)?;
            (result.jwt, result.subdomain, result.user_id)
        }
        ClientMessage::AuthToken { jwt } => {
            let claims = validate_jwt(&jwt, &config.jwt_secret)?;
            (jwt, claims.subdomain.clone(), claims.sub.clone())
        }
        _ => {
            let failure = ServerMessage::AuthFailure {
                reason: "Expected Auth or AuthToken message".to_string(),
            };
            let _ = write.send(Message::Text(failure.to_json()?)).await;
            return Err(TunnelError::AuthFailed(
                "Invalid auth message type".to_string(),
            ));
        }
    };

    // Send auth success
    let success = ServerMessage::AuthSuccess {
        jwt: jwt.clone(),
        subdomain: subdomain.clone(),
    };
    write.send(Message::Text(success.to_json()?)).await?;
    info!("Client {} authenticated as subdomain: {}", addr, subdomain);

    // Create request channel
    let (request_tx, mut request_rx) =
        mpsc::channel::<(ServerMessage, oneshot::Sender<ClientMessage>)>(32);

    // Register connection
    {
        let mut conns = connections.write().await;
        // Remove existing connection for this subdomain if any
        if conns.contains_key(&subdomain) {
            warn!("Replacing existing connection for subdomain: {}", subdomain);
        }
        conns.insert(
            subdomain.clone(),
            TunnelConnection {
                request_tx,
                connected_at: Instant::now(),
                last_heartbeat: Instant::now(),
                user_id,
                stats: TunnelStats::new(),
            },
        );
    }

    // Track pending requests
    let pending_requests: Arc<RwLock<HashMap<u64, oneshot::Sender<ClientMessage>>>> =
        Arc::new(RwLock::new(HashMap::new()));

    // Message handling loop
    let result = handle_message_loop(
        &mut write,
        &mut read,
        &mut request_rx,
        &connections,
        &subdomain,
        &pending_requests,
    )
    .await;

    // Cleanup on disconnect
    connections.write().await.remove(&subdomain);
    info!("Client {} disconnected (subdomain: {})", addr, subdomain);

    result
}

/// Handle the main message loop for a connected client.
async fn handle_message_loop(
    write: &mut futures_util::stream::SplitSink<WebSocketStream<TcpStream>, Message>,
    read: &mut futures_util::stream::SplitStream<WebSocketStream<TcpStream>>,
    request_rx: &mut mpsc::Receiver<(ServerMessage, oneshot::Sender<ClientMessage>)>,
    connections: &Arc<RwLock<HashMap<String, TunnelConnection>>>,
    subdomain: &str,
    pending_requests: &Arc<RwLock<HashMap<u64, oneshot::Sender<ClientMessage>>>>,
) -> Result<(), TunnelError> {
    loop {
        tokio::select! {
            // Handle incoming messages from client
            msg = read.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        let client_msg = ClientMessage::from_json(&text)?;
                        match client_msg {
                            ClientMessage::Heartbeat => {
                                // Update last heartbeat
                                {
                                    let mut conns = connections.write().await;
                                    if let Some(conn) = conns.get_mut(subdomain) {
                                        conn.last_heartbeat = Instant::now();
                                    }
                                }
                                // Send ack
                                let ack = ServerMessage::HeartbeatAck;
                                write.send(Message::Text(ack.to_json()?)).await?;
                            }
                            ClientMessage::HttpResponse { request_id, .. } => {
                                // Route response to waiting request
                                let mut pending = pending_requests.write().await;
                                if let Some(tx) = pending.remove(&request_id) {
                                    let _ = tx.send(client_msg);
                                } else {
                                    warn!("Received response for unknown request: {}", request_id);
                                }
                            }
                            _ => {
                                debug!("Ignoring unexpected message type from client");
                            }
                        }
                    }
                    Some(Ok(Message::Close(_))) => {
                        debug!("Client sent close frame");
                        break;
                    }
                    Some(Ok(Message::Ping(data))) => {
                        write.send(Message::Pong(data)).await?;
                    }
                    Some(Ok(_)) => {
                        // Ignore other message types
                    }
                    Some(Err(e)) => {
                        return Err(TunnelError::from(e));
                    }
                    None => {
                        debug!("WebSocket stream ended");
                        break;
                    }
                }
            }

            // Handle outgoing requests to client
            request = request_rx.recv() => {
                match request {
                    Some((server_msg, response_tx)) => {
                        // Extract request_id for tracking
                        if let ServerMessage::HttpRequest { request_id, .. } = &server_msg {
                            pending_requests.write().await.insert(*request_id, response_tx);
                        }
                        write.send(Message::Text(server_msg.to_json()?)).await?;
                    }
                    None => {
                        // Channel closed, server is shutting down
                        break;
                    }
                }
            }
        }
    }

    Ok(())
}

/// Clean up connections that have exceeded the heartbeat timeout.
async fn cleanup_stale_connections_inner(
    connections: &Arc<RwLock<HashMap<String, TunnelConnection>>>,
    timeout: Duration,
) {
    let now = Instant::now();
    let mut to_remove = Vec::new();

    {
        let conns = connections.read().await;
        for (subdomain, conn) in conns.iter() {
            if now.duration_since(conn.last_heartbeat) > timeout {
                to_remove.push(subdomain.clone());
            }
        }
    }

    if !to_remove.is_empty() {
        let mut conns = connections.write().await;
        for subdomain in to_remove {
            info!("Removing stale connection for subdomain: {}", subdomain);
            conns.remove(&subdomain);
        }
    }
}

#[cfg(test)]
mod tests {
    #![deny(clippy::unwrap_used)]

    use super::*;

    #[test]
    fn test_tunnel_server_config_default() {
        let config = TunnelServerConfig::default();
        assert_eq!(config.listen_addr.port(), 8082);
        assert_eq!(config.http_addr.port(), 8081);
        assert_eq!(config.heartbeat_timeout, Duration::from_secs(90));
        assert!(config.jwt_secret.is_empty());
    }

    #[test]
    fn test_tunnel_server_new() {
        let config = TunnelServerConfig {
            jwt_secret: b"test-secret".to_vec(),
            ..Default::default()
        };
        let server = TunnelServer::new(config);
        assert_eq!(server.config.jwt_secret, b"test-secret".to_vec());
    }

    #[tokio::test]
    async fn test_active_tunnels_empty() {
        let server = TunnelServer::new(TunnelServerConfig::default());
        let tunnels = server.active_tunnels().await;
        assert!(tunnels.is_empty());
    }

    #[tokio::test]
    async fn test_connection_count_empty() {
        let server = TunnelServer::new(TunnelServerConfig::default());
        assert_eq!(server.connection_count().await, 0);
    }

    #[tokio::test]
    async fn test_route_request_tunnel_not_found() {
        let server = TunnelServer::new(TunnelServerConfig::default());
        let result = server
            .route_request("nonexistent", "GET", "/", vec![], vec![])
            .await;
        assert!(matches!(result, Err(TunnelError::TunnelNotFound(_))));
    }

    #[tokio::test]
    async fn test_disconnect_nonexistent() {
        let server = TunnelServer::new(TunnelServerConfig::default());
        assert!(!server.disconnect("nonexistent").await);
    }

    #[tokio::test]
    async fn test_cleanup_stale_connections() {
        let connections: Arc<RwLock<HashMap<String, TunnelConnection>>> =
            Arc::new(RwLock::new(HashMap::new()));

        // Add a connection with old heartbeat
        let (tx, _rx) = mpsc::channel(1);
        {
            let mut conns = connections.write().await;
            conns.insert(
                "stale-subdomain".to_string(),
                TunnelConnection {
                    request_tx: tx,
                    connected_at: Instant::now(),
                    last_heartbeat: Instant::now() - Duration::from_secs(120),
                    user_id: "user-1".to_string(),
                    stats: TunnelStats::new(),
                },
            );
        }

        // Run cleanup with 90 second timeout
        cleanup_stale_connections_inner(&connections, Duration::from_secs(90)).await;

        // Connection should be removed
        assert!(connections.read().await.is_empty());
    }

    #[tokio::test]
    async fn test_cleanup_keeps_fresh_connections() {
        let connections: Arc<RwLock<HashMap<String, TunnelConnection>>> =
            Arc::new(RwLock::new(HashMap::new()));

        // Add a fresh connection
        let (tx, _rx) = mpsc::channel(1);
        {
            let mut conns = connections.write().await;
            conns.insert(
                "fresh-subdomain".to_string(),
                TunnelConnection {
                    request_tx: tx,
                    connected_at: Instant::now(),
                    last_heartbeat: Instant::now(),
                    user_id: "user-1".to_string(),
                    stats: TunnelStats::new(),
                },
            );
        }

        // Run cleanup with 90 second timeout
        cleanup_stale_connections_inner(&connections, Duration::from_secs(90)).await;

        // Connection should still exist
        assert_eq!(connections.read().await.len(), 1);
    }

    #[test]
    fn test_tunnel_stats_new() {
        let stats = TunnelStats::new();
        assert_eq!(stats.get_bytes_in(), 0);
        assert_eq!(stats.get_bytes_out(), 0);
        assert_eq!(stats.get_requests_handled(), 0);
    }

    #[test]
    fn test_tunnel_stats_add_bytes_in() {
        let stats = TunnelStats::new();
        stats.add_bytes_in(100);
        assert_eq!(stats.get_bytes_in(), 100);
        stats.add_bytes_in(50);
        assert_eq!(stats.get_bytes_in(), 150);
    }

    #[test]
    fn test_tunnel_stats_add_bytes_out() {
        let stats = TunnelStats::new();
        stats.add_bytes_out(200);
        assert_eq!(stats.get_bytes_out(), 200);
        stats.add_bytes_out(75);
        assert_eq!(stats.get_bytes_out(), 275);
    }

    #[test]
    fn test_tunnel_stats_increment_requests() {
        let stats = TunnelStats::new();
        stats.increment_requests();
        assert_eq!(stats.get_requests_handled(), 1);
        stats.increment_requests();
        stats.increment_requests();
        assert_eq!(stats.get_requests_handled(), 3);
    }

    #[tokio::test]
    async fn test_get_status_empty() {
        let server = TunnelServer::new(TunnelServerConfig::default());
        let status = server.get_status().await;
        assert_eq!(status.active_tunnels, 0);
        assert!(status.tunnels.is_empty());
    }

    #[tokio::test]
    async fn test_get_status_with_tunnel() {
        let connections: Arc<RwLock<HashMap<String, TunnelConnection>>> =
            Arc::new(RwLock::new(HashMap::new()));

        let (tx, _rx) = mpsc::channel(1);
        let stats = TunnelStats::new();
        stats.add_bytes_in(1000);
        stats.add_bytes_out(2000);
        stats.increment_requests();

        {
            let mut conns = connections.write().await;
            conns.insert(
                "test-subdomain".to_string(),
                TunnelConnection {
                    request_tx: tx,
                    connected_at: Instant::now(),
                    last_heartbeat: Instant::now(),
                    user_id: "user-1".to_string(),
                    stats,
                },
            );
        }

        let server = TunnelServer {
            config: TunnelServerConfig::default(),
            connections,
        };

        let status = server.get_status().await;
        assert_eq!(status.active_tunnels, 1);
        assert_eq!(status.tunnels.len(), 1);
        assert_eq!(status.tunnels[0].subdomain, "test-subdomain");
        assert_eq!(status.tunnels[0].bytes_in, 1000);
        assert_eq!(status.tunnels[0].bytes_out, 2000);
        assert_eq!(status.tunnels[0].requests_handled, 1);
    }
}
