//! Cuttlefish terminal user interface.
//!
//! A lightweight TUI client that can operate in two modes:
//! - **Remote mode**: Connects to a Cuttlefish server via WebSocket
//! - **Local mode**: Runs agents directly in-process (no server required)

mod app;
#[cfg(feature = "local")]
mod local;
mod mascot;
mod ui;
mod updater;

use std::time::Duration;

use app::App;
use clap::Parser;
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
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

    /// Run in local mode (no server required).
    /// Uses providers configured in cuttlefish.toml.
    #[cfg(feature = "local")]
    #[arg(long)]
    local: bool,

    /// Path to config file (for local mode).
    #[cfg(feature = "local")]
    #[arg(long, default_value = "cuttlefish.toml")]
    config: std::path::PathBuf,

    /// Working directory for local mode.
    #[cfg(feature = "local")]
    #[arg(long)]
    workdir: Option<std::path::PathBuf>,

    /// API key for authentication (remote mode).
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
    /// Unsubscribe from project updates.
    Unsubscribe {
        /// Project ID to unsubscribe from.
        project_id: String,
    },
    /// Request list of templates.
    ListTemplates,
    /// Request list of projects.
    ListProjects,
    /// Create a new project.
    CreateProject {
        /// Project name.
        name: String,
        /// Template name.
        template: String,
        /// Execution mode (cloud, build_remote_run_local, local).
        execution_mode: String,
        /// Local path for local mode.
        #[serde(skip_serializing_if = "Option::is_none")]
        local_path: Option<String>,
    },
    /// Delete a project.
    DeleteProject {
        /// Project ID to delete.
        project_id: String,
    },
}

/// Template info from server.
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
struct TemplateData {
    /// Template name/ID.
    name: String,
    /// Template description.
    description: String,
    /// Category (builtin, user, community).
    category: String,
    /// Language/framework.
    language: String,
    /// Tags for filtering.
    #[serde(default)]
    tags: Vec<String>,
}

/// Project info from server.
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
struct ProjectData {
    /// Project ID.
    id: String,
    /// Project name.
    name: String,
    /// Template name used.
    template_name: Option<String>,
    /// Execution mode.
    execution_mode: Option<String>,
    /// Running status.
    running_status: Option<String>,
    /// Local path.
    local_path: Option<String>,
    /// Tunnel URL.
    tunnel_url: Option<String>,
    /// Last activity time (relative).
    last_activity: Option<String>,
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
    /// Streaming chunk from an agent.
    StreamChunk {
        /// Project ID.
        project_id: String,
        /// Agent name.
        agent: String,
        /// Content chunk.
        content: String,
        /// Whether this is the final chunk.
        done: bool,
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
    /// List of available templates.
    TemplateList {
        /// Templates.
        templates: Vec<TemplateData>,
    },
    /// List of projects.
    ProjectList {
        /// Projects list.
        projects: Vec<ProjectData>,
    },
    /// Project created confirmation.
    ProjectCreated {
        /// Project ID.
        project_id: String,
        /// Project name.
        name: String,
    },
    /// Project deleted confirmation.
    ProjectDeleted {
        /// Project ID.
        project_id: String,
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
    execute!(stdout, EnterAlternateScreen)?;
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

    // Run in local or remote mode
    #[cfg(feature = "local")]
    let result = if cli.local {
        run_local_mode(&mut terminal, &mut app, &cli).await
    } else {
        run_app(&mut terminal, &mut app, ws_url).await
    };

    #[cfg(not(feature = "local"))]
    let result = run_app(&mut terminal, &mut app, ws_url).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
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
        url.query_pairs_mut().append_pair("token", key);
    }

    Ok(url)
}

/// Run in local mode (in-process agents, no server).
#[cfg(feature = "local")]
async fn run_local_mode(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    app: &mut App,
    cli: &Cli,
) -> anyhow::Result<()> {
    let workdir = cli
        .workdir
        .clone()
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(".")));

    app.add_message("system", "Initializing local mode...");
    terminal.draw(|f| ui::render(app, f))?;

    // Initialize local runtime
    let runtime = match local::LocalRuntime::new(&cli.config, &workdir).await {
        Ok(rt) => rt,
        Err(e) => {
            app.add_message("error", format!("Failed to initialize local mode: {}", e));
            app.add_message("system", "Tip: Create a cuttlefish.toml with provider configuration");
            app.add_message("system", "Example:");
            app.add_message("system", "  [providers.anthropic]");
            app.add_message("system", "  provider_type = \"anthropic\"");
            app.add_message("system", "  model = \"claude-sonnet-4-6\"");

            // Run in disconnected mode so user can see the error
            return run_disconnected(terminal, app).await;
        }
    };

    local::run_local(terminal, app, runtime).await
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
                KeyCode::BackTab => app.prev_view(),
                KeyCode::Char(c) => {
                    match app.view {
                        app::AppView::History => {}
                        app::AppView::Dashboard => {
                            // N to create new project, D to delete
                            if c == 'n' || c == 'N' {
                                app.start_create_project();
                            } else if c == 'd' || c == 'D' {
                                if let Some(project_id) = app.delete_selected_project() {
                                    app.add_message("system", format!("Deleted project: {}", project_id));
                                }
                            }
                        }
                        app::AppView::CreateProject => {
                            // Text input for name step
                            if app.create_project.step == app::CreateProjectStep::Name {
                                app.create_project.name.push(c);
                                app.create_project.name_message.clear();
                            }
                        }
                        _ => {
                            app.input.push(c);
                        }
                    }
                }
                KeyCode::Backspace => {
                    match app.view {
                        app::AppView::History | app::AppView::Dashboard => {}
                        app::AppView::CreateProject => {
                            if app.create_project.step == app::CreateProjectStep::Name {
                                app.create_project.name.pop();
                            }
                        }
                        _ => {
                            app.input.pop();
                        }
                    }
                }
                KeyCode::Enter => {
                    match app.view {
                        app::AppView::History => {
                            if app.show_restore_options {
                                if let Some((option, msg_index)) = app.confirm_restore() {
                                    app.add_message(
                                        "system",
                                        format!(
                                            "Restore {} at message {} (not yet implemented)",
                                            option.label(),
                                            msg_index
                                        ),
                                    );
                                    app.exit_history_mode();
                                }
                            } else {
                                app.show_restore_popup();
                            }
                        }
                        app::AppView::Dashboard => {
                            // Open selected project
                            if let Some(proj) = app.selected_project().cloned() {
                                app.open_project(&proj.id);
                            }
                        }
                        app::AppView::CreateProject => {
                            // Advance wizard or complete
                            if app.create_project_advance() {
                                // Wizard complete - create the project locally
                                let name = app.create_project.name.clone();
                                let template = app.selected_template().map(|t| t.name.clone());
                                let mode = app.selected_execution_mode();
                                let local_path = if mode == app::ExecutionMode::Local {
                                    Some(app.create_project.local_path.clone())
                                } else {
                                    None
                                };

                                // Create project info and add to list
                                let project_id = uuid::Uuid::new_v4().to_string();
                                app.projects.push(app::ProjectInfo {
                                    id: project_id.clone(),
                                    name: name.clone(),
                                    template_name: template.clone(),
                                    execution_mode: mode,
                                    running_status: app::RunningStatus::Idle,
                                    local_path,
                                    tunnel_url: if mode == app::ExecutionMode::Cloud {
                                        Some(format!("https://{}.cuttlefish.ai", name))
                                    } else {
                                        None
                                    },
                                    last_activity: "now".to_string(),
                                    active: true,
                                });

                                // Mark other projects as inactive
                                for p in app.projects.iter_mut() {
                                    if p.id != project_id {
                                        p.active = false;
                                    }
                                }

                                app.add_message("system", format!("Created project: {}", name));
                                app.open_project(&project_id);
                            }
                        }
                        _ => {
                            if !app.input.is_empty() {
                                let input = std::mem::take(&mut app.input);
                                app.add_to_history(&input);

                                if let Some(cmd) = app.handle_command(&input) {
                                    match cmd {
                                        app::SlashCommand::Help => app.go_to_view(app::AppView::Help),
                                        app::SlashCommand::ListProjects => {
                                            app.go_to_view(app::AppView::Dashboard)
                                        }
                                        app::SlashCommand::ClearChat => app.clear_chat(),
                                        app::SlashCommand::Quit => return Ok(()),
                                        _ => {
                                            app.add_message("system", "Not connected to server");
                                        }
                                    }
                                } else {
                                    app.add_message("user", &input);
                                    app.add_message("system", "Not connected to server");
                                }
                            }
                        }
                    }
                }
                KeyCode::Up => {
                    match app.view {
                        app::AppView::History => app.history_prev(),
                        app::AppView::Dashboard => app.select_prev_project(),
                        app::AppView::CreateProject => app.create_project_prev(),
                        _ => {
                            if app.input.is_empty() {
                                app.history_up();
                            } else {
                                app.scroll_up(1);
                            }
                        }
                    }
                }
                KeyCode::Down => {
                    match app.view {
                        app::AppView::History => app.history_next(),
                        app::AppView::Dashboard => app.select_next_project(),
                        app::AppView::CreateProject => app.create_project_next(),
                        _ => {
                            if app.input.is_empty() {
                                app.history_down();
                            } else {
                                app.scroll_down(1);
                            }
                        }
                    }
                }
                KeyCode::PageUp => app.scroll_up(10),
                KeyCode::PageDown => app.scroll_down(10),
                KeyCode::Home => app.scroll_to_bottom(),
                KeyCode::Esc => {
                    if app.handle_esc() {
                        return Ok(());
                    }
                }
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

    // Request project list and templates for dashboard
    let _ = tx.send(ClientMessage::ListProjects).await;
    let _ = tx.send(ClientMessage::ListTemplates).await;

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
                            KeyCode::BackTab => app.prev_view(),
                            KeyCode::Char(c) => {
                                match app.view {
                                    app::AppView::History => {}
                                    app::AppView::Dashboard => {
                                        if c == 'n' || c == 'N' {
                                            app.start_create_project();
                                            // Request templates from server
                                            let _ = tx.send(ClientMessage::ListTemplates).await;
                                        } else if c == 'd' || c == 'D' {
                                            if let Some(project_id) = app.delete_selected_project() {
                                                // Send delete request to server
                                                let _ = tx.send(ClientMessage::DeleteProject {
                                                    project_id,
                                                }).await;
                                            }
                                        }
                                    }
                                    app::AppView::CreateProject => {
                                        if app.create_project.step == app::CreateProjectStep::Name {
                                            app.create_project.name.push(c);
                                            app.create_project.name_message.clear();
                                        }
                                    }
                                    _ => {
                                        app.input.push(c);
                                    }
                                }
                            }
                            KeyCode::Backspace => {
                                match app.view {
                                    app::AppView::History | app::AppView::Dashboard => {}
                                    app::AppView::CreateProject => {
                                        if app.create_project.step == app::CreateProjectStep::Name {
                                            app.create_project.name.pop();
                                        }
                                    }
                                    _ => {
                                        app.input.pop();
                                    }
                                }
                            }
                            KeyCode::Enter => {
                                match app.view {
                                    app::AppView::History => {
                                        if app.show_restore_options {
                                            if let Some((option, msg_index)) = app.confirm_restore() {
                                                app.add_message(
                                                    "system",
                                                    format!(
                                                        "Restore {} at message {} (not yet implemented)",
                                                        option.label(),
                                                        msg_index
                                                    ),
                                                );
                                                app.exit_history_mode();
                                            }
                                        } else {
                                            app.show_restore_popup();
                                        }
                                    }
                                    app::AppView::Dashboard => {
                                        if let Some(proj) = app.selected_project().cloned() {
                                            // Unsubscribe from old project
                                            if let Some(ref old_id) = app.project_id {
                                                let _ = tx.send(ClientMessage::Unsubscribe {
                                                    project_id: old_id.clone(),
                                                }).await;
                                            }
                                            // Open and subscribe to project
                                            app.open_project(&proj.id);
                                            let _ = tx.send(ClientMessage::Subscribe {
                                                project_id: proj.id,
                                            }).await;
                                        }
                                    }
                                    app::AppView::CreateProject => {
                                        if app.create_project_advance() {
                                            // Wizard complete - send create project to server
                                            let name = app.create_project.name.clone();
                                            let template = app
                                                .selected_template()
                                                .map(|t| t.name.clone())
                                                .unwrap_or_else(|| "blank".to_string());
                                            let mode = app.selected_execution_mode();
                                            let local_path = if mode == app::ExecutionMode::Local {
                                                Some(app.create_project.local_path.clone())
                                            } else {
                                                None
                                            };

                                            // Send create project request to server
                                            let _ = tx.send(ClientMessage::CreateProject {
                                                name: name.clone(),
                                                template,
                                                execution_mode: mode.as_str().to_string(),
                                                local_path,
                                            }).await;

                                            app.add_message("system", format!("Creating project: {}...", name));
                                            // Server will respond with ProjectCreated which will
                                            // add the project to the list and open it
                                            app.view = app::AppView::Dashboard;
                                        }
                                    }
                                    _ => {
                                        if !app.input.is_empty() {
                                            let input = std::mem::take(&mut app.input);
                                            app.add_to_history(&input);

                                            if let Some(cmd) = app.handle_command(&input) {
                                                match cmd {
                                                    app::SlashCommand::Help => app.go_to_view(app::AppView::Help),
                                                    app::SlashCommand::ListProjects => {
                                                        let _ = tx.send(ClientMessage::ListProjects).await;
                                                        app.go_to_view(app::AppView::Dashboard);
                                                    }
                                                    app::SlashCommand::NewProject { name: _ } => {
                                                        // Start the create project wizard instead
                                                        app.start_create_project();
                                                        let _ = tx.send(ClientMessage::ListTemplates).await;
                                                    }
                                                    app::SlashCommand::SwitchProject { id } => {
                                                        if let Some(ref old_id) = app.project_id {
                                                            let _ = tx.send(ClientMessage::Unsubscribe {
                                                                project_id: old_id.clone(),
                                                            }).await;
                                                        }
                                                        app.project_id = Some(id.clone());
                                                        let _ = tx.send(ClientMessage::Subscribe {
                                                            project_id: id.clone(),
                                                        }).await;
                                                        app.add_message("system", format!("Switched to project: {}", id));
                                                    }
                                                    app::SlashCommand::ClearChat => app.clear_chat(),
                                                    app::SlashCommand::Quit => return Ok(()),
                                                }
                                            } else if !input.starts_with('/') {
                                                app.add_message("user", &input);

                                                let project_id = app.project_id
                                                    .clone()
                                                    .unwrap_or_else(|| {
                                                        let id = uuid::Uuid::new_v4().to_string();
                                                        app.project_id = Some(id.clone());
                                                        id
                                                    });

                                                let msg = ClientMessage::Chat {
                                                    project_id,
                                                    content: input,
                                                };
                                                let _ = tx.send(msg).await;
                                            }
                                        }
                                    }
                                }
                            }
                            KeyCode::Up => {
                                match app.view {
                                    app::AppView::History => app.history_prev(),
                                    app::AppView::Dashboard => app.select_prev_project(),
                                    app::AppView::CreateProject => app.create_project_prev(),
                                    _ => {
                                        if app.input.is_empty() {
                                            app.history_up();
                                        } else {
                                            app.scroll_up(1);
                                        }
                                    }
                                }
                            }
                            KeyCode::Down => {
                                match app.view {
                                    app::AppView::History => app.history_next(),
                                    app::AppView::Dashboard => app.select_next_project(),
                                    app::AppView::CreateProject => app.create_project_next(),
                                    _ => {
                                        if app.input.is_empty() {
                                            app.history_down();
                                        } else {
                                            app.scroll_down(1);
                                        }
                                    }
                                }
                            }
                            KeyCode::PageUp => app.scroll_up(10),
                            KeyCode::PageDown => app.scroll_down(10),
                            KeyCode::Home => app.scroll_to_bottom(),
                            KeyCode::Esc => {
                                if app.handle_esc() {
                                    return Ok(());
                                }
                            }
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
            app.streaming = false;
            app.add_message(&agent, &content);
        }
        ServerMessage::StreamChunk {
            agent,
            content,
            done,
            ..
        } => {
            if !app.streaming {
                // First chunk - start new message
                app.streaming = true;
                app.add_message(&agent, &content);
            } else {
                // Subsequent chunks - append to existing message
                app.append_to_message(&agent, &content);
            }
            if done {
                app.streaming = false;
            }
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
        ServerMessage::TemplateList { templates } => {
            // Update templates in create project wizard
            app.create_project.templates = templates
                .into_iter()
                .map(|t| app::TemplateInfo {
                    name: t.name,
                    description: t.description,
                    category: t.category,
                    language: t.language,
                })
                .collect();
            // Add blank template if not present
            if !app
                .create_project
                .templates
                .iter()
                .any(|t| t.name == "blank")
            {
                app.create_project.templates.push(app::TemplateInfo {
                    name: "blank".to_string(),
                    description: "Empty project, you define everything".to_string(),
                    category: "builtin".to_string(),
                    language: "Any".to_string(),
                });
            }
        }
        ServerMessage::ProjectList { projects } => {
            app.projects = projects
                .into_iter()
                .map(|p| app::ProjectInfo {
                    id: p.id.clone(),
                    name: p.name,
                    template_name: p.template_name,
                    execution_mode: p
                        .execution_mode
                        .map(|m| app::ExecutionMode::from_str(&m))
                        .unwrap_or_default(),
                    running_status: p
                        .running_status
                        .map(|s| app::RunningStatus::from_str(&s))
                        .unwrap_or_default(),
                    local_path: p.local_path,
                    tunnel_url: p.tunnel_url,
                    last_activity: p.last_activity.unwrap_or_else(|| "unknown".to_string()),
                    active: app.project_id.as_ref().is_some_and(|id| *id == p.id),
                })
                .collect();
            app.projects_selected = 0;
        }
        ServerMessage::ProjectCreated { project_id, name } => {
            app.add_message("system", format!("Created project: {}", name));
            // Mark all existing projects as inactive
            for p in app.projects.iter_mut() {
                p.active = false;
            }
            // Add new project as active using wizard state
            let mode = app.selected_execution_mode();
            let local_path = if mode == app::ExecutionMode::Local {
                Some(app.create_project.local_path.clone())
            } else {
                None
            };
            app.projects.push(app::ProjectInfo {
                id: project_id.clone(),
                name: name.clone(),
                template_name: app.selected_template().map(|t| t.name.clone()),
                execution_mode: mode,
                running_status: app::RunningStatus::Idle,
                local_path,
                tunnel_url: None,
                last_activity: "now".to_string(),
                active: true,
            });
            // Open the project and switch to chat view
            // Note: subscription is handled separately in the event loop
            app.open_project(&project_id);
        }
        ServerMessage::ProjectDeleted { project_id } => {
            app.add_message("system", format!("Deleted project: {}", project_id));
            // Remove from local list
            app.projects.retain(|p| p.id != project_id);
            // Clear active project if it was deleted
            if app.project_id.as_ref().is_some_and(|id| *id == project_id) {
                app.project_id = None;
            }
            // Adjust selection
            if app.projects_selected >= app.projects.len() && app.projects_selected > 0 {
                app.projects_selected -= 1;
            }
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
        assert_eq!(url.as_str(), "ws://localhost:8080/ws?token=test-key");
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
