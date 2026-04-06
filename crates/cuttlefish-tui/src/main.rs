//! Cuttlefish terminal user interface.
//!
//! A lightweight TUI client that connects to a remote Cuttlefish server
//! via WebSocket for real-time chat, build logs, and file diffs.

mod app;
mod mascot;
mod ui;
mod updater;

use std::time::Duration;

use app::App;
use clap::Parser;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use futures::{SinkExt, StreamExt};
use ratatui::{Terminal, backend::CrosstermBackend};
use serde::{Deserialize, Serialize};
use tokio_tungstenite::{connect_async, tungstenite::Message as WsMessage};
use url::Url;

/// Cuttlefish TUI client.
#[derive(Parser)]
#[command(name = "cuttlefish-tui", version, about)]
struct Cli {
    /// WebSocket server URL (e.g., ws://localhost:8080).
    #[arg(long, default_value = "ws://localhost:8080")]
    server: String,

    /// API key for authentication.
    #[arg(long, env = "CUTTLEFISH_API_KEY")]
    api_key: Option<String>,

    /// Project ID to connect to (creates new if not specified).
    #[arg(long)]
    project: Option<String>,

    /// Check for updates and exit.
    #[arg(long)]
    check_update: bool,

    /// Download and install the latest update.
    #[arg(long)]
    update: bool,

    /// Skip the automatic update check on startup.
    #[arg(long)]
    no_update_check: bool,
}

/// Outbound message to server.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ClientMessage {
    /// Chat message for a project.
    Chat {
        /// Project ID.
        project_id: String,
        /// Message content.
        content: String,
    },
    /// Ping for connection keepalive.
    Ping,
    /// Subscribe to project updates.
    Subscribe {
        /// Project ID to subscribe to.
        project_id: String,
    },
}

/// Inbound message from server.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[allow(dead_code)] // Fields required for deserialization
enum ServerMessage {
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
    /// An action requires user approval.
    PendingApproval {
        /// Unique action ID.
        action_id: String,
        /// Human-readable description.
        description: String,
        /// Confidence score.
        confidence: f32,
    },
    /// Real-time log entry from agent activity.
    LogEntry {
        /// Agent name.
        agent: String,
        /// Action being performed.
        action: String,
        /// Log level.
        level: String,
        /// Project name.
        project: String,
    },
    /// Pong response.
    Pong,
    /// Error message.
    Error {
        /// Error message.
        message: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Handle update commands first (before setting up terminal)
    if cli.check_update {
        return handle_check_update().await;
    }

    if cli.update {
        return handle_update().await;
    }

    // Set up logging
    tracing::subscriber::set_global_default(tracing_subscriber::fmt().compact().finish())
        .expect("failed to set tracing subscriber");

    // Check for updates in background (unless disabled)
    let update_info = if !cli.no_update_check {
        updater::check_for_update().await
    } else {
        None
    };

    tracing::info!("Cuttlefish TUI starting, connecting to {}", cli.server);

    // Build WebSocket URL
    let ws_url = build_ws_url(&cli.server, cli.api_key.as_deref())?;

    // Initialize terminal
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    let mut app = App::new();
    app.project_id = cli.project.clone();

    // Show update notification if available
    if let Some(ref info) = update_info {
        app.add_message(
            "system",
            format!(
                "Update available: v{} -> v{} (run with --update to install)",
                info.current_version, info.latest_version
            ),
        );
    }

    // Run the app
    let result = run_app(&mut terminal, &mut app, ws_url).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(e) = result {
        eprintln!("Error: {e}");
        return Err(e);
    }

    Ok(())
}

/// Build the WebSocket URL with optional API key.
fn build_ws_url(server: &str, api_key: Option<&str>) -> anyhow::Result<Url> {
    let base = if server.starts_with("ws://") || server.starts_with("wss://") {
        server.to_string()
    } else {
        format!("ws://{server}")
    };

    let mut url = Url::parse(&base)?;
    url.set_path("/ws");

    if let Some(key) = api_key {
        url.query_pairs_mut().append_pair("key", key);
    }

    Ok(url)
}

/// Run the main application event loop.
async fn run_app(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    app: &mut App,
    ws_url: Url,
) -> anyhow::Result<()> {
    // Connect to WebSocket
    let ws_result = connect_async(ws_url.as_str()).await;

    let (ws_write, ws_read) = match ws_result {
        Ok((stream, _response)) => {
            app.connected = true;
            app.add_message("system", "Connected to Cuttlefish server");
            stream.split()
        }
        Err(e) => {
            app.connected = false;
            app.add_message("system", format!("Failed to connect: {e}"));

            // Run in disconnected mode - just show UI
            return run_disconnected(terminal, app).await;
        }
    };

    // Subscribe to project if specified
    if let Some(ref project_id) = app.project_id {
        app.add_message("system", format!("Subscribed to project: {project_id}"));
    }

    // Run the main event loop
    run_event_loop(terminal, app, ws_write, ws_read).await
}

/// Run in disconnected mode (no server connection).
async fn run_disconnected(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    app: &mut App,
) -> anyhow::Result<()> {
    loop {
        // Draw UI
        terminal.draw(|f| ui::render(app, f))?;

        // Poll for events with timeout
        if event::poll(Duration::from_millis(100))?
            && let Event::Key(key) = event::read()?
        {
            // Check for Ctrl+C
            if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
                return Ok(());
            }

            match key.code {
                KeyCode::Tab => app.next_view(),
                KeyCode::Char(c) => app.input.push(c),
                KeyCode::Backspace => {
                    app.input.pop();
                }
                KeyCode::Enter => {
                    if !app.input.is_empty() {
                        let input = std::mem::take(&mut app.input);
                        app.add_message("user", &input);
                        app.add_message("system", "Not connected to server");
                    }
                }
                KeyCode::Esc => return Ok(()),
                _ => {}
            }
        }

        if app.should_exit {
            return Ok(());
        }
    }
}

/// Run the main event loop with WebSocket connection.
async fn run_event_loop<W, R>(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    app: &mut App,
    mut ws_write: W,
    mut ws_read: R,
) -> anyhow::Result<()>
where
    W: futures::Sink<WsMessage> + Unpin,
    R: futures::Stream<Item = Result<WsMessage, tokio_tungstenite::tungstenite::Error>> + Unpin,
{
    // Channel for sending messages to WebSocket
    let (tx, mut cmd_rx) = tokio::sync::mpsc::channel::<ClientMessage>(32);

    // Subscribe to project if we have one
    if let Some(ref project_id) = app.project_id {
        let msg = ClientMessage::Subscribe {
            project_id: project_id.clone(),
        };
        let _ = tx.send(msg).await;
    }

    // Ping interval
    let mut ping_interval = tokio::time::interval(Duration::from_secs(30));
    ping_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    loop {
        // Draw UI
        terminal.draw(|f| ui::render(app, f))?;

        tokio::select! {
            // Keyboard input
            _ = tokio::time::sleep(Duration::from_millis(50)) => {
                while event::poll(Duration::ZERO)? {
                    if let Event::Key(key) = event::read()? {
                        // Check for Ctrl+C
                        if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
                            return Ok(());
                        }

                        match key.code {
                            KeyCode::Tab => app.next_view(),
                            KeyCode::Char(c) => app.input.push(c),
                            KeyCode::Backspace => { app.input.pop(); }
                            KeyCode::Enter => {
                                if !app.input.is_empty() {
                                    let input = std::mem::take(&mut app.input);
                                    app.add_message("user", &input);

                                    // Get or create project ID
                                    let project_id = app.project_id
                                        .clone()
                                        .unwrap_or_else(|| {
                                            let id = uuid::Uuid::new_v4().to_string();
                                            app.project_id = Some(id.clone());
                                            id
                                        });

                                    // Send chat message
                                    let msg = ClientMessage::Chat {
                                        project_id,
                                        content: input,
                                    };
                                    let _ = tx.send(msg).await;
                                }
                            }
                            KeyCode::Esc => return Ok(()),
                            _ => {}
                        }
                    }
                }
            }

            // WebSocket messages from server
            msg = ws_read.next() => {
                match msg {
                    Some(Ok(WsMessage::Text(text))) => {
                        handle_server_message(app, &text);
                    }
                    Some(Ok(WsMessage::Close(_))) => {
                        app.connected = false;
                        app.add_message("system", "Server closed connection");
                        return Ok(());
                    }
                    Some(Err(e)) => {
                        app.connected = false;
                        app.add_message("system", format!("WebSocket error: {e}"));
                        return Ok(());
                    }
                    None => {
                        app.connected = false;
                        app.add_message("system", "Connection closed");
                        return Ok(());
                    }
                    _ => {}
                }
            }

            // Send queued commands
            Some(cmd) = cmd_rx.recv() => {
                if let Ok(json) = serde_json::to_string(&cmd)
                    && ws_write.send(WsMessage::Text(json.into())).await.is_err()
                {
                    app.connected = false;
                    app.add_message("system", "Failed to send message");
                }
            }

            // Periodic ping
            _ = ping_interval.tick() => {
                let ping_msg = ClientMessage::Ping;
                if let Ok(json) = serde_json::to_string(&ping_msg) {
                    let _ = ws_write.send(WsMessage::Text(json.into())).await;
                }
            }
        }

        if app.should_exit {
            return Ok(());
        }
    }
}

/// Handle --check-update flag.
async fn handle_check_update() -> anyhow::Result<()> {
    println!("Checking for updates...");

    match updater::check_for_update().await {
        Some(info) => {
            println!(
                "Update available: v{} -> v{}",
                info.current_version, info.latest_version
            );
            println!("Run with --update to install");
        }
        None => {
            println!(
                "You are running the latest version (v{})",
                env!("CARGO_PKG_VERSION")
            );
        }
    }

    Ok(())
}

/// Handle --update flag.
async fn handle_update() -> anyhow::Result<()> {
    println!("Checking for updates...");

    match updater::check_for_update().await {
        Some(info) => {
            println!(
                "Update available: v{} -> v{}",
                info.current_version, info.latest_version
            );
            updater::apply_update(&info).await?;
        }
        None => {
            println!(
                "You are already running the latest version (v{})",
                env!("CARGO_PKG_VERSION")
            );
        }
    }

    Ok(())
}

/// Handle a server message and update app state.
fn handle_server_message(app: &mut App, text: &str) {
    let Ok(msg) = serde_json::from_str::<ServerMessage>(text) else {
        tracing::warn!("Failed to parse server message: {}", text);
        return;
    };

    match msg {
        ServerMessage::Response { agent, content, .. } => {
            app.add_message(&agent, &content);
        }
        ServerMessage::BuildLog { line, .. } => {
            app.add_log_line(&line);
        }
        ServerMessage::Diff { patch, .. } => {
            app.diff_content = patch;
        }
        ServerMessage::PendingApproval {
            description,
            confidence,
            ..
        } => {
            app.add_message(
                "system",
                format!(
                    "Approval needed ({:.0}% confidence): {description}",
                    confidence * 100.0
                ),
            );
        }
        ServerMessage::LogEntry {
            agent,
            action,
            level,
            project,
        } => {
            app.add_log_line(format!("[{level}] [{project}] {agent}: {action}"));
        }
        ServerMessage::Pong => {
            // Connection alive, no action needed
        }
        ServerMessage::Error { message } => {
            app.add_message("error", &message);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_ws_url_simple() {
        let url = build_ws_url("localhost:8080", None).unwrap();
        assert_eq!(url.as_str(), "ws://localhost:8080/ws");
    }

    #[test]
    fn test_build_ws_url_with_api_key() {
        let url = build_ws_url("localhost:8080", Some("test-key")).unwrap();
        assert_eq!(url.as_str(), "ws://localhost:8080/ws?key=test-key");
    }

    #[test]
    fn test_build_ws_url_with_scheme() {
        let url = build_ws_url("wss://example.com", None).unwrap();
        assert_eq!(url.as_str(), "wss://example.com/ws");
    }

    #[test]
    fn test_client_message_serialization() {
        let msg = ClientMessage::Chat {
            project_id: "test".to_string(),
            content: "hello".to_string(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("chat"));
        assert!(json.contains("test"));
        assert!(json.contains("hello"));
    }

    #[test]
    fn test_server_message_deserialization() {
        let json = r#"{"type":"response","project_id":"p1","agent":"coder","content":"done"}"#;
        let msg: ServerMessage = serde_json::from_str(json).unwrap();
        match msg {
            ServerMessage::Response { agent, content, .. } => {
                assert_eq!(agent, "coder");
                assert_eq!(content, "done");
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_server_message_error() {
        let json = r#"{"type":"error","message":"test error"}"#;
        let msg: ServerMessage = serde_json::from_str(json).unwrap();
        match msg {
            ServerMessage::Error { message } => {
                assert_eq!(message, "test error");
            }
            _ => panic!("Wrong message type"),
        }
    }
}
