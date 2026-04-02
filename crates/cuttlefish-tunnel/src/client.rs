//! Tunnel client — connects to tunnel server and forwards HTTP requests.
//!
//! The tunnel client establishes an outbound WebSocket connection to the tunnel server,
//! authenticates via link code or JWT, and forwards incoming HTTP requests to a local server.

#![deny(clippy::unwrap_used)]

use crate::error::TunnelError;
use crate::protocol::{ClientMessage, ServerMessage};
use futures_util::{SinkExt, StreamExt};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::sync::{mpsc, RwLock};
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};
use tracing::{debug, error, info, warn};

/// Policy for automatic reconnection with exponential backoff.
#[derive(Debug, Clone)]
pub struct ReconnectPolicy {
    /// Initial delay before first reconnection attempt (default: 1 second)
    pub initial_delay: Duration,
    /// Maximum delay between reconnection attempts (default: 5 minutes)
    pub max_delay: Duration,
    /// Multiplier applied to delay after each failed attempt (default: 2.0)
    pub multiplier: f64,
    /// Maximum number of reconnection attempts (None = infinite)
    pub max_attempts: Option<u32>,
}

impl Default for ReconnectPolicy {
    fn default() -> Self {
        Self {
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(300),
            multiplier: 2.0,
            max_attempts: None,
        }
    }
}

impl ReconnectPolicy {
    /// Create a new reconnect policy with custom settings.
    pub fn new(
        initial_delay: Duration,
        max_delay: Duration,
        multiplier: f64,
        max_attempts: Option<u32>,
    ) -> Self {
        Self {
            initial_delay,
            max_delay,
            multiplier,
            max_attempts,
        }
    }

    /// Calculate the delay for a given attempt number (0-indexed).
    pub fn delay_for_attempt(&self, attempt: u32) -> Duration {
        let multiplied = self.initial_delay.as_secs_f64() * self.multiplier.powi(attempt as i32);
        let capped = multiplied.min(self.max_delay.as_secs_f64());
        Duration::from_secs_f64(capped)
    }

    /// Check if another attempt should be made.
    pub fn should_retry(&self, attempt: u32) -> bool {
        match self.max_attempts {
            Some(max) => attempt < max,
            None => true,
        }
    }
}

/// Events emitted by the reconnecting tunnel client.
#[derive(Debug, Clone)]
pub enum TunnelEvent {
    /// Successfully connected to the tunnel server.
    Connected {
        /// The assigned subdomain.
        subdomain: String,
    },
    /// Disconnected from the tunnel server.
    Disconnected {
        /// Reason for disconnection.
        reason: String,
    },
    /// Attempting to reconnect.
    Reconnecting {
        /// Current attempt number (1-indexed).
        attempt: u32,
        /// Delay before this attempt.
        delay: Duration,
    },
    /// Reconnection failed permanently.
    ReconnectFailed {
        /// Error description.
        error: String,
    },
}

/// Type alias for the WebSocket stream
type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

/// Tunnel client configuration
#[derive(Debug, Clone)]
pub struct TunnelClientConfig {
    /// URL of the tunnel server (e.g., wss://tunnel.cuttlefish.ai)
    pub server_url: String,
    /// Local address to forward requests to
    pub local_addr: SocketAddr,
    /// Heartbeat interval
    pub heartbeat_interval: Duration,
}

impl Default for TunnelClientConfig {
    fn default() -> Self {
        Self {
            server_url: "wss://tunnel.cuttlefish.ai".to_string(),
            local_addr: SocketAddr::from(([127, 0, 0, 1], 8080)),
            heartbeat_interval: Duration::from_secs(30),
        }
    }
}

/// Tunnel client state
pub struct TunnelClient {
    config: TunnelClientConfig,
    jwt: Arc<RwLock<Option<String>>>,
    subdomain: Arc<RwLock<Option<String>>>,
    connected: Arc<RwLock<bool>>,
    http_client: reqwest::Client,
}

impl TunnelClient {
    /// Create a new tunnel client with the given configuration
    pub fn new(config: TunnelClientConfig) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client - this should never fail with default settings");

        Self {
            config,
            jwt: Arc::new(RwLock::new(None)),
            subdomain: Arc::new(RwLock::new(None)),
            connected: Arc::new(RwLock::new(false)),
            http_client,
        }
    }

    /// Connect to the tunnel server using a link code, returning the JWT on success
    pub async fn connect_with_link_code(&self, code: &str) -> Result<String, TunnelError> {
        info!("Connecting to tunnel server with link code");

        let (mut ws_stream, _) = connect_async(&self.config.server_url).await?;
        debug!("WebSocket connection established");

        let auth_msg = ClientMessage::Auth {
            link_code: code.to_string(),
        };
        let json = auth_msg.to_json()?;
        ws_stream.send(Message::Text(json)).await?;
        debug!("Sent auth message with link code");

        let response = self.wait_for_auth_response(&mut ws_stream).await?;

        match response {
            ServerMessage::AuthSuccess { jwt, subdomain } => {
                info!(subdomain = %subdomain, "Authentication successful");
                *self.jwt.write().await = Some(jwt.clone());
                *self.subdomain.write().await = Some(subdomain);
                *self.connected.write().await = true;

                self.spawn_event_loop(ws_stream).await;

                Ok(jwt)
            }
            ServerMessage::AuthFailure { reason } => {
                error!(reason = %reason, "Authentication failed");
                Err(TunnelError::AuthFailed(reason))
            }
            other => {
                error!(?other, "Unexpected response during authentication");
                Err(TunnelError::AuthFailed(
                    "Unexpected server response".to_string(),
                ))
            }
        }
    }

    /// Connect to the tunnel server using an existing JWT token
    pub async fn connect_with_jwt(&self, jwt: &str) -> Result<(), TunnelError> {
        info!("Connecting to tunnel server with JWT");

        let (mut ws_stream, _) = connect_async(&self.config.server_url).await?;
        debug!("WebSocket connection established");

        let auth_msg = ClientMessage::AuthToken {
            jwt: jwt.to_string(),
        };
        let json = auth_msg.to_json()?;
        ws_stream.send(Message::Text(json)).await?;
        debug!("Sent auth message with JWT");

        let response = self.wait_for_auth_response(&mut ws_stream).await?;

        match response {
            ServerMessage::AuthSuccess {
                jwt: new_jwt,
                subdomain,
            } => {
                info!(subdomain = %subdomain, "Authentication successful");
                *self.jwt.write().await = Some(new_jwt);
                *self.subdomain.write().await = Some(subdomain);
                *self.connected.write().await = true;

                self.spawn_event_loop(ws_stream).await;

                Ok(())
            }
            ServerMessage::AuthFailure { reason } => {
                error!(reason = %reason, "Authentication failed");
                Err(TunnelError::AuthFailed(reason))
            }
            other => {
                error!(?other, "Unexpected response during authentication");
                Err(TunnelError::AuthFailed(
                    "Unexpected server response".to_string(),
                ))
            }
        }
    }

    /// Main event loop (runs internally after authentication)
    pub async fn run(&self) -> Result<(), TunnelError> {
        info!("Tunnel client run() called - event loop is managed internally");
        Ok(())
    }

    /// Get the assigned subdomain, if connected
    pub fn subdomain(&self) -> Option<String> {
        self.subdomain
            .try_read()
            .ok()
            .and_then(|guard| guard.clone())
    }

    /// Check if the client is currently connected
    pub fn is_connected(&self) -> bool {
        self.connected.try_read().ok().is_some_and(|guard| *guard)
    }

    async fn wait_for_auth_response(
        &self,
        ws_stream: &mut WsStream,
    ) -> Result<ServerMessage, TunnelError> {
        let timeout = tokio::time::timeout(Duration::from_secs(30), ws_stream.next()).await;

        match timeout {
            Ok(Some(Ok(Message::Text(text)))) => {
                let msg = ServerMessage::from_json(&text)?;
                Ok(msg)
            }
            Ok(Some(Ok(Message::Close(frame)))) => {
                let reason = frame
                    .map(|f| f.reason.to_string())
                    .unwrap_or_else(|| "Unknown".to_string());
                Err(TunnelError::ConnectionClosed(reason))
            }
            Ok(Some(Ok(_))) => Err(TunnelError::AuthFailed(
                "Unexpected message type".to_string(),
            )),
            Ok(Some(Err(e))) => Err(e.into()),
            Ok(None) => Err(TunnelError::ConnectionClosed(
                "Connection closed before auth response".to_string(),
            )),
            Err(_) => Err(TunnelError::Timeout(30000)),
        }
    }

    async fn spawn_event_loop(&self, ws_stream: WsStream) {
        let config = self.config.clone();
        let connected = Arc::clone(&self.connected);
        let http_client = self.http_client.clone();

        tokio::spawn(async move {
            if let Err(e) = run_event_loop(ws_stream, config, connected.clone(), http_client).await
            {
                error!(error = %e, "Event loop terminated with error");
            }
            *connected.write().await = false;
            info!("Tunnel client disconnected");
        });
    }
}

/// Wrapper around `TunnelClient` that provides automatic reconnection with exponential backoff.
pub struct ReconnectingTunnelClient {
    client: TunnelClient,
    policy: ReconnectPolicy,
    event_sender: Option<mpsc::Sender<TunnelEvent>>,
}

impl ReconnectingTunnelClient {
    /// Create a new reconnecting tunnel client.
    pub fn new(config: TunnelClientConfig, policy: ReconnectPolicy) -> Self {
        Self {
            client: TunnelClient::new(config),
            policy,
            event_sender: None,
        }
    }

    /// Set an event sender to receive tunnel events.
    pub fn with_event_sender(mut self, sender: mpsc::Sender<TunnelEvent>) -> Self {
        self.event_sender = Some(sender);
        self
    }

    /// Get a reference to the underlying `TunnelClient`.
    pub fn client(&self) -> &TunnelClient {
        &self.client
    }

    /// Check if currently connected.
    pub fn is_connected(&self) -> bool {
        self.client.is_connected()
    }

    /// Get the current subdomain, if connected.
    pub fn subdomain(&self) -> Option<String> {
        self.client.subdomain()
    }

    /// Run the tunnel with automatic reconnection on network errors.
    ///
    /// This method will:
    /// - Connect to the tunnel server using the provided JWT
    /// - On network disconnect, wait with exponential backoff and reconnect
    /// - Reset the delay counter on successful connections
    /// - Stop retrying on authentication failures (non-recoverable)
    ///
    /// # Errors
    ///
    /// Returns an error only for non-recoverable failures:
    /// - Authentication failures (invalid/expired JWT)
    /// - Maximum retry attempts exceeded (if configured)
    pub async fn run_with_reconnect(&self, jwt: &str) -> Result<(), TunnelError> {
        let mut attempt: u32 = 0;

        loop {
            match self.client.connect_with_jwt(jwt).await {
                Ok(()) => {
                    let subdomain = self.client.subdomain().unwrap_or_default();
                    self.send_event(TunnelEvent::Connected {
                        subdomain: subdomain.clone(),
                    })
                    .await;

                    info!(subdomain = %subdomain, "Connected to tunnel server");
                    attempt = 0;

                    self.wait_for_disconnect().await;

                    let reason = "Connection lost".to_string();
                    self.send_event(TunnelEvent::Disconnected {
                        reason: reason.clone(),
                    })
                    .await;

                    info!(reason = %reason, "Disconnected from tunnel server");
                }
                Err(TunnelError::AuthFailed(reason)) => {
                    error!(reason = %reason, "Authentication failed - not retrying");
                    self.send_event(TunnelEvent::ReconnectFailed {
                        error: format!("Authentication failed: {reason}"),
                    })
                    .await;
                    return Err(TunnelError::AuthFailed(reason));
                }
                Err(e) => {
                    warn!(error = %e, attempt = attempt + 1, "Connection failed");
                }
            }

            if !self.policy.should_retry(attempt) {
                let error_msg = format!(
                    "Max reconnection attempts ({}) exceeded",
                    self.policy.max_attempts.unwrap_or(0)
                );
                error!("{}", error_msg);
                self.send_event(TunnelEvent::ReconnectFailed { error: error_msg.clone() }).await;
                return Err(TunnelError::ConnectionClosed(error_msg));
            }

            let delay = self.policy.delay_for_attempt(attempt);
            attempt = attempt.saturating_add(1);

            self.send_event(TunnelEvent::Reconnecting {
                attempt,
                delay,
            })
            .await;

            info!(
                attempt = attempt,
                delay_secs = delay.as_secs_f64(),
                "Reconnecting after delay"
            );

            tokio::time::sleep(delay).await;
        }
    }

    async fn send_event(&self, event: TunnelEvent) {
        if let Some(sender) = &self.event_sender
            && let Err(e) = sender.send(event).await
        {
            debug!(error = %e, "Failed to send tunnel event");
        }
    }

    async fn wait_for_disconnect(&self) {
        while self.client.is_connected() {
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }
}

/// Run the main event loop for the tunnel client
async fn run_event_loop(
    ws_stream: WsStream,
    config: TunnelClientConfig,
    connected: Arc<RwLock<bool>>,
    http_client: reqwest::Client,
) -> Result<(), TunnelError> {
    let (mut ws_sink, mut ws_source) = ws_stream.split();
    let mut heartbeat_interval = tokio::time::interval(config.heartbeat_interval);

    loop {
        tokio::select! {
            msg = ws_source.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        match ServerMessage::from_json(&text) {
                            Ok(server_msg) => {
                                handle_server_message(
                                    server_msg,
                                    &mut ws_sink,
                                    &config,
                                    &http_client,
                                ).await?;
                            }
                            Err(e) => {
                                warn!(error = %e, "Failed to parse server message");
                            }
                        }
                    }
                    Some(Ok(Message::Close(frame))) => {
                        let reason = frame
                            .map(|f| f.reason.to_string())
                            .unwrap_or_else(|| "Unknown".to_string());
                        info!(reason = %reason, "Server closed connection");
                        return Err(TunnelError::ConnectionClosed(reason));
                    }
                    Some(Ok(Message::Ping(data))) => {
                        debug!("Received ping, sending pong");
                        ws_sink.send(Message::Pong(data)).await?;
                    }
                    Some(Ok(Message::Pong(_))) => {
                        debug!("Received pong");
                    }
                    Some(Ok(_)) => {}
                    Some(Err(e)) => {
                        error!(error = %e, "WebSocket error");
                        return Err(e.into());
                    }
                    None => {
                        info!("WebSocket stream ended");
                        return Err(TunnelError::ConnectionClosed("Stream ended".to_string()));
                    }
                }
            }

            _ = heartbeat_interval.tick() => {
                if !*connected.read().await {
                    break;
                }
                debug!("Sending heartbeat");
                let heartbeat = ClientMessage::Heartbeat;
                let json = heartbeat.to_json()?;
                ws_sink.send(Message::Text(json)).await?;
            }
        }
    }

    Ok(())
}

async fn handle_server_message<S>(
    msg: ServerMessage,
    ws_sink: &mut S,
    config: &TunnelClientConfig,
    http_client: &reqwest::Client,
) -> Result<(), TunnelError>
where
    S: SinkExt<Message, Error = tokio_tungstenite::tungstenite::Error> + Unpin,
{
    match msg {
        ServerMessage::HttpRequest {
            request_id,
            method,
            path,
            headers,
            body,
        } => {
            debug!(request_id = request_id, method = %method, path = %path, "Received HTTP request");

            let http_client = http_client.clone();
            let local_addr = config.local_addr;

            let response = tokio::spawn(async move {
                forward_http_request(&http_client, local_addr, request_id, method, path, headers, body).await
            })
            .await
            .map_err(|e| TunnelError::HttpForward(format!("Task join error: {e}")))?;

            let response_msg = response?;
            let json = response_msg.to_json()?;
            ws_sink
                .send(Message::Text(json))
                .await
                .map_err(TunnelError::from)?;
        }
        ServerMessage::HeartbeatAck => {
            debug!("Received heartbeat acknowledgment");
        }
        ServerMessage::Disconnect { reason } => {
            info!(reason = %reason, "Server requested disconnect");
            return Err(TunnelError::ConnectionClosed(reason));
        }
        ServerMessage::AuthSuccess { .. } | ServerMessage::AuthFailure { .. } => {
            warn!("Received unexpected auth message after connection established");
        }
    }

    Ok(())
}

async fn forward_http_request(
    http_client: &reqwest::Client,
    local_addr: SocketAddr,
    request_id: u64,
    method: String,
    path: String,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
) -> Result<ClientMessage, TunnelError> {
    let url = format!("http://{}{}", local_addr, path);
    debug!(url = %url, method = %method, "Forwarding request to local server");

    let method = method
        .parse::<reqwest::Method>()
        .map_err(|e| TunnelError::HttpForward(format!("Invalid HTTP method: {e}")))?;

    let mut request = http_client.request(method, &url);

    // Skip hop-by-hop headers per RFC 2616 Section 13.5.1
    for (name, value) in headers {
        let name_lower = name.to_lowercase();
        if matches!(
            name_lower.as_str(),
            "host" | "connection" | "keep-alive" | "transfer-encoding" | "upgrade"
        ) {
            continue;
        }

        if let (Ok(header_name), Ok(header_value)) = (
            reqwest::header::HeaderName::try_from(&name),
            reqwest::header::HeaderValue::try_from(&value),
        ) {
            request = request.header(header_name, header_value);
        }
    }

    if !body.is_empty() {
        request = request.body(body);
    }

    let response = match request.send().await {
        Ok(resp) => resp,
        Err(e) => {
            warn!(error = %e, "Failed to forward request to local server");
            return Ok(ClientMessage::HttpResponse {
                request_id,
                status: 502,
                headers: vec![("content-type".to_string(), "text/plain".to_string())],
                body: format!("Failed to connect to local server: {e}").into_bytes(),
            });
        }
    };

    let status = response.status().as_u16();
    let response_headers: Vec<(String, String)> = response
        .headers()
        .iter()
        .filter_map(|(name, value)| {
            value
                .to_str()
                .ok()
                .map(|v| (name.to_string(), v.to_string()))
        })
        .collect();

    let response_body = response
        .bytes()
        .await
        .map_err(|e| TunnelError::HttpForward(format!("Failed to read response body: {e}")))?
        .to_vec();

    debug!(
        request_id = request_id,
        status = status,
        body_len = response_body.len(),
        "Received response from local server"
    );

    Ok(ClientMessage::HttpResponse {
        request_id,
        status,
        headers: response_headers,
        body: response_body,
    })
}

#[cfg(test)]
mod tests {
    #![deny(clippy::unwrap_used)]

    use super::*;

    #[test]
    fn test_client_config_default() {
        let config = TunnelClientConfig::default();

        assert_eq!(config.server_url, "wss://tunnel.cuttlefish.ai");
        assert_eq!(config.local_addr, SocketAddr::from(([127, 0, 0, 1], 8080)));
        assert_eq!(config.heartbeat_interval, Duration::from_secs(30));
    }

    #[test]
    fn test_client_config_custom() {
        let config = TunnelClientConfig {
            server_url: "wss://custom.example.com".to_string(),
            local_addr: SocketAddr::from(([192, 168, 1, 1], 3000)),
            heartbeat_interval: Duration::from_secs(60),
        };

        assert_eq!(config.server_url, "wss://custom.example.com");
        assert_eq!(
            config.local_addr,
            SocketAddr::from(([192, 168, 1, 1], 3000))
        );
        assert_eq!(config.heartbeat_interval, Duration::from_secs(60));
    }

    #[test]
    fn test_client_new() {
        let config = TunnelClientConfig::default();
        let client = TunnelClient::new(config);

        assert!(!client.is_connected());
        assert!(client.subdomain().is_none());
    }

    #[test]
    fn test_client_new_with_custom_config() {
        let config = TunnelClientConfig {
            server_url: "wss://test.example.com".to_string(),
            local_addr: SocketAddr::from(([127, 0, 0, 1], 9000)),
            heartbeat_interval: Duration::from_secs(15),
        };
        let client = TunnelClient::new(config);

        assert!(!client.is_connected());
        assert!(client.subdomain().is_none());
    }

    #[tokio::test]
    async fn test_client_initial_state() {
        let config = TunnelClientConfig::default();
        let client = TunnelClient::new(config);

        assert!(!client.is_connected());
        assert!(client.subdomain().is_none());

        let jwt = client.jwt.read().await;
        assert!(jwt.is_none());

        let subdomain = client.subdomain.read().await;
        assert!(subdomain.is_none());

        let connected = client.connected.read().await;
        assert!(!*connected);
    }

    #[tokio::test]
    async fn test_client_state_updates() {
        let config = TunnelClientConfig::default();
        let client = TunnelClient::new(config);

        *client.jwt.write().await = Some("test-jwt-token".to_string());
        *client.subdomain.write().await = Some("my-tunnel".to_string());
        *client.connected.write().await = true;

        assert!(client.is_connected());
        assert_eq!(client.subdomain(), Some("my-tunnel".to_string()));

        let jwt = client.jwt.read().await;
        assert_eq!(jwt.as_deref(), Some("test-jwt-token"));
    }

    #[test]
    fn test_http_response_message_creation() {
        let response = ClientMessage::HttpResponse {
            request_id: 42,
            status: 200,
            headers: vec![("content-type".to_string(), "application/json".to_string())],
            body: b"{}".to_vec(),
        };

        let json = response.to_json().expect("Failed to serialize response");
        assert!(json.contains("http_response"));
        assert!(json.contains("42"));
        assert!(json.contains("200"));
    }

    #[test]
    fn test_heartbeat_message_creation() {
        let heartbeat = ClientMessage::Heartbeat;
        let json = heartbeat.to_json().expect("Failed to serialize heartbeat");
        assert!(json.contains("heartbeat"));
    }

    #[test]
    fn test_reconnect_policy_default() {
        let policy = ReconnectPolicy::default();

        assert_eq!(policy.initial_delay, Duration::from_secs(1));
        assert_eq!(policy.max_delay, Duration::from_secs(300));
        assert!((policy.multiplier - 2.0).abs() < f64::EPSILON);
        assert!(policy.max_attempts.is_none());
    }

    #[test]
    fn test_reconnect_policy_custom() {
        let policy = ReconnectPolicy::new(
            Duration::from_millis(500),
            Duration::from_secs(60),
            1.5,
            Some(10),
        );

        assert_eq!(policy.initial_delay, Duration::from_millis(500));
        assert_eq!(policy.max_delay, Duration::from_secs(60));
        assert!((policy.multiplier - 1.5).abs() < f64::EPSILON);
        assert_eq!(policy.max_attempts, Some(10));
    }

    #[test]
    fn test_reconnect_policy_delay_calculation() {
        let policy = ReconnectPolicy::new(
            Duration::from_secs(1),
            Duration::from_secs(300),
            2.0,
            None,
        );

        assert_eq!(policy.delay_for_attempt(0), Duration::from_secs(1));
        assert_eq!(policy.delay_for_attempt(1), Duration::from_secs(2));
        assert_eq!(policy.delay_for_attempt(2), Duration::from_secs(4));
        assert_eq!(policy.delay_for_attempt(3), Duration::from_secs(8));
        assert_eq!(policy.delay_for_attempt(4), Duration::from_secs(16));
    }

    #[test]
    fn test_reconnect_policy_delay_capped_at_max() {
        let policy = ReconnectPolicy::new(
            Duration::from_secs(1),
            Duration::from_secs(10),
            2.0,
            None,
        );

        assert_eq!(policy.delay_for_attempt(0), Duration::from_secs(1));
        assert_eq!(policy.delay_for_attempt(1), Duration::from_secs(2));
        assert_eq!(policy.delay_for_attempt(2), Duration::from_secs(4));
        assert_eq!(policy.delay_for_attempt(3), Duration::from_secs(8));
        assert_eq!(policy.delay_for_attempt(4), Duration::from_secs(10));
        assert_eq!(policy.delay_for_attempt(5), Duration::from_secs(10));
        assert_eq!(policy.delay_for_attempt(100), Duration::from_secs(10));
    }

    #[test]
    fn test_reconnect_policy_should_retry_infinite() {
        let policy = ReconnectPolicy::new(
            Duration::from_secs(1),
            Duration::from_secs(60),
            2.0,
            None,
        );

        assert!(policy.should_retry(0));
        assert!(policy.should_retry(100));
        assert!(policy.should_retry(u32::MAX - 1));
    }

    #[test]
    fn test_reconnect_policy_should_retry_limited() {
        let policy = ReconnectPolicy::new(
            Duration::from_secs(1),
            Duration::from_secs(60),
            2.0,
            Some(5),
        );

        assert!(policy.should_retry(0));
        assert!(policy.should_retry(1));
        assert!(policy.should_retry(4));
        assert!(!policy.should_retry(5));
        assert!(!policy.should_retry(6));
    }

    #[test]
    fn test_reconnect_policy_zero_max_attempts() {
        let policy = ReconnectPolicy::new(
            Duration::from_secs(1),
            Duration::from_secs(60),
            2.0,
            Some(0),
        );

        assert!(!policy.should_retry(0));
    }

    #[test]
    fn test_reconnect_policy_fractional_multiplier() {
        let policy = ReconnectPolicy::new(
            Duration::from_secs(10),
            Duration::from_secs(300),
            1.5,
            None,
        );

        assert_eq!(policy.delay_for_attempt(0), Duration::from_secs(10));
        assert_eq!(policy.delay_for_attempt(1), Duration::from_secs(15));
        let delay_2 = policy.delay_for_attempt(2);
        assert!(delay_2 >= Duration::from_millis(22400) && delay_2 <= Duration::from_millis(22600));
    }

    #[test]
    fn test_tunnel_event_variants() {
        let connected = TunnelEvent::Connected {
            subdomain: "test-sub".to_string(),
        };
        let disconnected = TunnelEvent::Disconnected {
            reason: "timeout".to_string(),
        };
        let reconnecting = TunnelEvent::Reconnecting {
            attempt: 3,
            delay: Duration::from_secs(4),
        };
        let failed = TunnelEvent::ReconnectFailed {
            error: "max attempts".to_string(),
        };

        assert!(matches!(connected, TunnelEvent::Connected { .. }));
        assert!(matches!(disconnected, TunnelEvent::Disconnected { .. }));
        assert!(matches!(reconnecting, TunnelEvent::Reconnecting { .. }));
        assert!(matches!(failed, TunnelEvent::ReconnectFailed { .. }));
    }

    #[test]
    fn test_reconnecting_client_creation() {
        let config = TunnelClientConfig::default();
        let policy = ReconnectPolicy::default();
        let client = ReconnectingTunnelClient::new(config, policy);

        assert!(!client.is_connected());
        assert!(client.subdomain().is_none());
    }

    #[test]
    fn test_reconnecting_client_with_custom_policy() {
        let config = TunnelClientConfig::default();
        let policy = ReconnectPolicy::new(
            Duration::from_millis(100),
            Duration::from_secs(30),
            1.5,
            Some(3),
        );
        let client = ReconnectingTunnelClient::new(config, policy);

        assert!(!client.is_connected());
    }

    #[tokio::test]
    async fn test_reconnecting_client_with_event_sender() {
        let config = TunnelClientConfig::default();
        let policy = ReconnectPolicy::default();
        let (tx, mut rx) = mpsc::channel(10);

        let client = ReconnectingTunnelClient::new(config, policy).with_event_sender(tx);

        assert!(client.event_sender.is_some());

        client
            .send_event(TunnelEvent::Connected {
                subdomain: "test".to_string(),
            })
            .await;

        let event = rx.recv().await.expect("Should receive event");
        assert!(matches!(event, TunnelEvent::Connected { subdomain } if subdomain == "test"));
    }

    #[test]
    fn test_reconnect_policy_exponential_sequence() {
        let policy = ReconnectPolicy::default();
        let delays: Vec<u64> = (0..10).map(|i| policy.delay_for_attempt(i).as_secs()).collect();

        assert_eq!(delays[0], 1);
        assert_eq!(delays[1], 2);
        assert_eq!(delays[2], 4);
        assert_eq!(delays[3], 8);
        assert_eq!(delays[4], 16);
        assert_eq!(delays[5], 32);
        assert_eq!(delays[6], 64);
        assert_eq!(delays[7], 128);
        assert_eq!(delays[8], 256);
        assert_eq!(delays[9], 300);
    }
}
