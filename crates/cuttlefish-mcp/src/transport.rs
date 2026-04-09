//! MCP transport implementations.
//!
//! Supports multiple transports for connecting to MCP servers:
//! - Stdio: Communicate with subprocess over stdin/stdout
//! - SSE: Server-Sent Events over HTTP

use std::collections::HashMap;
use std::process::Stdio;
use std::sync::Arc;

use async_trait::async_trait;
use futures::StreamExt;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::{Mutex, mpsc, oneshot};
use tracing::{debug, error, info};

use crate::error::McpError;
use crate::protocol::{JsonRpcNotification, JsonRpcRequest, JsonRpcResponse, RequestId};

/// Transport trait for MCP communication.
#[async_trait]
pub trait McpTransport: Send + Sync {
    /// Send a request and wait for a response.
    async fn request(&self, request: JsonRpcRequest) -> Result<JsonRpcResponse, McpError>;

    /// Send a notification (no response expected).
    async fn notify(&self, notification: JsonRpcNotification) -> Result<(), McpError>;

    /// Close the transport.
    async fn close(&self) -> Result<(), McpError>;
}

/// Stdio transport - communicates with MCP server over stdin/stdout.
pub struct StdioTransport {
    /// Child process.
    child: Arc<Mutex<Option<Child>>>,
    /// Stdin writer.
    stdin_tx: mpsc::Sender<String>,
    /// Pending requests waiting for responses.
    pending: Arc<Mutex<HashMap<RequestId, oneshot::Sender<JsonRpcResponse>>>>,
    /// Shutdown signal.
    shutdown_tx: Option<oneshot::Sender<()>>,
}

impl StdioTransport {
    /// Spawn a new MCP server process.
    pub async fn spawn(command: &str, args: &[&str]) -> Result<Self, McpError> {
        Self::spawn_with_env(command, args, &[]).await
    }

    /// Spawn a new MCP server process with environment variables.
    pub async fn spawn_with_env(
        command: &str,
        args: &[&str],
        env: &[(&str, &str)],
    ) -> Result<Self, McpError> {
        info!("Spawning MCP server: {} {:?}", command, args);

        let mut cmd = Command::new(command);
        cmd.args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        for (key, value) in env {
            cmd.env(key, value);
        }

        let mut child = cmd
            .spawn()
            .map_err(|e| McpError::Transport(e.to_string()))?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| McpError::Transport("Failed to get stdin".to_string()))?;

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| McpError::Transport("Failed to get stdout".to_string()))?;

        // Channel for sending to stdin
        let (stdin_tx, mut stdin_rx) = mpsc::channel::<String>(100);

        // Pending requests
        let pending: Arc<Mutex<HashMap<RequestId, oneshot::Sender<JsonRpcResponse>>>> =
            Arc::new(Mutex::new(HashMap::new()));

        // Shutdown channel
        let (shutdown_tx, mut shutdown_rx) = oneshot::channel::<()>();

        // Spawn stdin writer task
        let mut stdin_writer = stdin;
        tokio::spawn(async move {
            while let Some(msg) = stdin_rx.recv().await {
                if stdin_writer.write_all(msg.as_bytes()).await.is_err() {
                    break;
                }
                if stdin_writer.write_all(b"\n").await.is_err() {
                    break;
                }
                if stdin_writer.flush().await.is_err() {
                    break;
                }
            }
        });

        // Spawn stdout reader task
        let pending_clone = pending.clone();
        tokio::spawn(async move {
            let mut reader = BufReader::new(stdout);
            let mut line = String::new();

            loop {
                tokio::select! {
                    _ = &mut shutdown_rx => {
                        debug!("Stdio reader shutting down");
                        break;
                    }
                    result = reader.read_line(&mut line) => {
                        match result {
                            Ok(0) => {
                                debug!("Stdio EOF");
                                break;
                            }
                            Ok(_) => {
                                let trimmed = line.trim();
                                if !trimmed.is_empty() {
                                    match serde_json::from_str::<JsonRpcResponse>(trimmed) {
                                        Ok(response) => {
                                            let mut pending = pending_clone.lock().await;
                                            if let Some(tx) = pending.remove(&response.id) {
                                                let _ = tx.send(response);
                                            }
                                        }
                                        Err(e) => {
                                            debug!("Failed to parse response: {} - {}", e, trimmed);
                                        }
                                    }
                                }
                                line.clear();
                            }
                            Err(e) => {
                                error!("Stdio read error: {}", e);
                                break;
                            }
                        }
                    }
                }
            }
        });

        Ok(Self {
            child: Arc::new(Mutex::new(Some(child))),
            stdin_tx,
            pending,
            shutdown_tx: Some(shutdown_tx),
        })
    }
}

#[async_trait]
impl McpTransport for StdioTransport {
    async fn request(&self, request: JsonRpcRequest) -> Result<JsonRpcResponse, McpError> {
        let id = request.id.clone();
        let json = serde_json::to_string(&request)
            .map_err(|e| McpError::Protocol(format!("Serialization error: {}", e)))?;

        // Create response channel
        let (tx, rx) = oneshot::channel();
        {
            let mut pending = self.pending.lock().await;
            pending.insert(id.clone(), tx);
        }

        // Send request
        self.stdin_tx
            .send(json)
            .await
            .map_err(|_| McpError::Transport("Failed to send to stdin".to_string()))?;

        // Wait for response with timeout
        match tokio::time::timeout(std::time::Duration::from_secs(30), rx).await {
            Ok(Ok(response)) => Ok(response),
            Ok(Err(_)) => {
                // Channel closed - remove pending
                let mut pending = self.pending.lock().await;
                pending.remove(&id);
                Err(McpError::Transport("Response channel closed".to_string()))
            }
            Err(_) => {
                // Timeout - remove pending
                let mut pending = self.pending.lock().await;
                pending.remove(&id);
                Err(McpError::Timeout)
            }
        }
    }

    async fn notify(&self, notification: JsonRpcNotification) -> Result<(), McpError> {
        let json = serde_json::to_string(&notification)
            .map_err(|e| McpError::Protocol(format!("Serialization error: {}", e)))?;

        self.stdin_tx
            .send(json)
            .await
            .map_err(|_| McpError::Transport("Failed to send notification".to_string()))?;

        Ok(())
    }

    async fn close(&self) -> Result<(), McpError> {
        // Signal shutdown
        // Note: shutdown_tx is consumed on first close, subsequent closes are no-op

        // Kill the child process
        let mut child_guard = self.child.lock().await;
        if let Some(mut child) = child_guard.take() {
            let _ = child.kill().await;
        }

        Ok(())
    }
}

impl Drop for StdioTransport {
    fn drop(&mut self) {
        // Take the shutdown sender to signal shutdown
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
    }
}

/// SSE transport configuration.
#[derive(Debug, Clone)]
pub struct SseConfig {
    /// Base URL for the MCP server.
    pub base_url: String,
    /// Optional authorization header.
    pub auth_header: Option<String>,
}

/// SSE (Server-Sent Events) transport.
pub struct SseTransport {
    config: SseConfig,
    client: reqwest::Client,
    /// Pending requests waiting for responses.
    pending: Arc<Mutex<HashMap<RequestId, oneshot::Sender<JsonRpcResponse>>>>,
    /// Shutdown signal.
    _shutdown_tx: Option<oneshot::Sender<()>>,
}

impl SseTransport {
    /// Create a new SSE transport.
    pub fn new(config: SseConfig) -> Self {
        let client = reqwest::Client::new();

        Self {
            config,
            client,
            pending: Arc::new(Mutex::new(HashMap::new())),
            _shutdown_tx: None,
        }
    }

    /// Connect and start listening for SSE events.
    pub async fn connect(&mut self) -> Result<(), McpError> {
        let sse_url = format!("{}/sse", self.config.base_url);
        info!("Connecting to SSE endpoint: {}", sse_url);

        let mut request = self.client.get(&sse_url);
        if let Some(ref auth) = self.config.auth_header {
            request = request.header("Authorization", auth);
        }

        let (shutdown_tx, mut shutdown_rx) = oneshot::channel::<()>();
        self._shutdown_tx = Some(shutdown_tx);

        let pending = self.pending.clone();

        // Spawn SSE listener
        tokio::spawn(async move {
            let response = match request.send().await {
                Ok(r) => r,
                Err(e) => {
                    error!("SSE connection failed: {}", e);
                    return;
                }
            };

            let mut stream = response.bytes_stream();
            let mut buffer = String::new();

            loop {
                tokio::select! {
                    _ = &mut shutdown_rx => {
                        debug!("SSE listener shutting down");
                        break;
                    }
                    chunk = stream.next() => {
                        match chunk {
                            Some(Ok(bytes)) => {
                                buffer.push_str(&String::from_utf8_lossy(&bytes));

                                // Process complete events
                                while let Some(event_end) = buffer.find("\n\n") {
                                    let event = buffer[..event_end].to_string();
                                    buffer = buffer[event_end + 2..].to_string();

                                    // Parse SSE event
                                    if let Some(data_line) = event.lines().find(|l| l.starts_with("data: ")) {
                                        let data = &data_line[6..];
                                        if let Ok(response) = serde_json::from_str::<JsonRpcResponse>(data) {
                                            let mut pending = pending.lock().await;
                                            if let Some(tx) = pending.remove(&response.id) {
                                                let _ = tx.send(response);
                                            }
                                        }
                                    }
                                }
                            }
                            Some(Err(e)) => {
                                error!("SSE stream error: {}", e);
                                break;
                            }
                            None => {
                                debug!("SSE stream ended");
                                break;
                            }
                        }
                    }
                }
            }
        });

        Ok(())
    }
}

#[async_trait]
impl McpTransport for SseTransport {
    async fn request(&self, request: JsonRpcRequest) -> Result<JsonRpcResponse, McpError> {
        let id = request.id.clone();

        // Create response channel
        let (tx, rx) = oneshot::channel();
        {
            let mut pending = self.pending.lock().await;
            pending.insert(id.clone(), tx);
        }

        // Send request via POST
        let url = format!("{}/message", self.config.base_url);
        let mut http_request = self.client.post(&url).json(&request);

        if let Some(ref auth) = self.config.auth_header {
            http_request = http_request.header("Authorization", auth);
        }

        http_request
            .send()
            .await
            .map_err(|e| McpError::Transport(e.to_string()))?;

        // Wait for response via SSE
        match tokio::time::timeout(std::time::Duration::from_secs(30), rx).await {
            Ok(Ok(response)) => Ok(response),
            Ok(Err(_)) => {
                let mut pending = self.pending.lock().await;
                pending.remove(&id);
                Err(McpError::Transport("Response channel closed".to_string()))
            }
            Err(_) => {
                let mut pending = self.pending.lock().await;
                pending.remove(&id);
                Err(McpError::Timeout)
            }
        }
    }

    async fn notify(&self, notification: JsonRpcNotification) -> Result<(), McpError> {
        let url = format!("{}/message", self.config.base_url);
        let mut request = self.client.post(&url).json(&notification);

        if let Some(ref auth) = self.config.auth_header {
            request = request.header("Authorization", auth);
        }

        request
            .send()
            .await
            .map_err(|e| McpError::Transport(e.to_string()))?;

        Ok(())
    }

    async fn close(&self) -> Result<(), McpError> {
        // Shutdown is handled via the shutdown channel when dropped
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sse_config() {
        let config = SseConfig {
            base_url: "http://localhost:8080".to_string(),
            auth_header: Some("Bearer token".to_string()),
        };
        assert_eq!(config.base_url, "http://localhost:8080");
    }
}
