//! Local mode runtime for running agents without a server.
//!
//! This module provides in-process agent execution, allowing the TUI
//! to function as a standalone tool without connecting to a remote server.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use futures::StreamExt;
use ratatui::{Terminal, backend::CrosstermBackend};
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

use crate::app::{self, App, AppView};
use crate::ui;

use cuttlefish_agents::{PromptRegistry, TokioMessageBus, WorkflowConfig, WorkflowEngine};
use cuttlefish_core::config::CuttlefishConfig;
use cuttlefish_core::traits::provider::ModelProvider;
use cuttlefish_db::Database;
use cuttlefish_providers::ProviderRegistry;

/// Runtime for local (in-process) agent execution.
pub struct LocalRuntime {
    /// Database for persistence.
    pub db: Arc<Database>,
    /// Provider registry with loaded providers.
    pub providers: Arc<ProviderRegistry>,
    /// Default provider name.
    pub default_provider: String,
    /// Working directory.
    pub workdir: PathBuf,
    /// Prompts directory.
    pub prompts_dir: PathBuf,
}

impl LocalRuntime {
    /// Create a new local runtime from config.
    pub async fn new(config_path: &Path, workdir: &Path) -> anyhow::Result<Self> {
        // Load config
        let config = if config_path.exists() {
            info!("Loading config from: {}", config_path.display());
            CuttlefishConfig::load_from_path(config_path)?
        } else {
            // Try default locations
            CuttlefishConfig::load().or_else(|_| CuttlefishConfig::from_env())?
        };

        // Open/create local database
        let db_path = workdir.join(".cuttlefish/local.db");
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let db = Database::open(&db_path).await?;
        let db = Arc::new(db);

        // Load providers from config
        let mut provider_registry = ProviderRegistry::new();
        let mut default_provider: Option<String> = None;

        for (name, provider_config) in &config.providers {
            match initialize_provider(name, provider_config).await {
                Ok(provider) => {
                    info!("Initialized provider: {}", name);
                    provider_registry.register(name.clone(), provider);
                    if default_provider.is_none() {
                        default_provider = Some(name.clone());
                    }
                }
                Err(e) => {
                    warn!("Failed to initialize provider '{}': {}", name, e);
                }
            }
        }

        let default_provider = default_provider.ok_or_else(|| {
            anyhow::anyhow!(
                "No providers configured. Add provider configuration to cuttlefish.toml\n\
                 Example:\n\
                 [providers.anthropic]\n\
                 provider_type = \"anthropic\"\n\
                 model = \"claude-sonnet-4-6\""
            )
        })?;

        info!(
            "Loaded {} provider(s), default: {}",
            provider_registry.len(),
            default_provider
        );

        // Find prompts directory
        let prompts_dir = std::env::var("CUTTLEFISH_PROMPTS_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| workdir.join("prompts"));

        Ok(Self {
            db,
            providers: Arc::new(provider_registry),
            default_provider,
            workdir: workdir.to_path_buf(),
            prompts_dir,
        })
    }

    /// Get the default provider.
    pub fn provider(&self) -> Option<Arc<dyn ModelProvider>> {
        self.providers.get(&self.default_provider)
    }

    /// Create a workflow engine.
    #[allow(dead_code)]
    pub fn workflow_engine(&self) -> Option<WorkflowEngine> {
        let provider = self.provider()?;
        let bus = TokioMessageBus::new();
        Some(WorkflowEngine::with_config(
            provider,
            bus,
            &self.prompts_dir,
            WorkflowConfig::default(),
        ))
    }
}

/// Initialize a model provider from configuration.
async fn initialize_provider(
    name: &str,
    config: &cuttlefish_core::config::ProviderConfig,
) -> anyhow::Result<Arc<dyn ModelProvider>> {
    use cuttlefish_providers::{anthropic, bedrock, google, ollama, openai};

    let provider_type = config.provider_type.as_str();

    match provider_type {
        "anthropic" => {
            let model = config.model.as_deref().unwrap_or("claude-sonnet-4-6");
            let provider = anthropic::AnthropicProvider::new(model)
                .map_err(|e| anyhow::anyhow!("Failed to create Anthropic provider: {}", e))?;
            Ok(Arc::new(provider) as Arc<dyn ModelProvider>)
        }
        "openai" => {
            let model = config.model.as_deref().unwrap_or("gpt-4o");
            let provider = openai::OpenAiProvider::new(model)
                .map_err(|e| anyhow::anyhow!("Failed to create OpenAI provider: {}", e))?;
            Ok(Arc::new(provider) as Arc<dyn ModelProvider>)
        }
        "bedrock" => {
            let model = config
                .model
                .as_deref()
                .unwrap_or("anthropic.claude-sonnet-4-6-20250514-v1:0");
            let provider = bedrock::BedrockProvider::new(model)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to create Bedrock provider: {}", e))?;
            Ok(Arc::new(provider) as Arc<dyn ModelProvider>)
        }
        "google" | "gemini" => {
            let model = config.model.as_deref().unwrap_or("gemini-2.0-flash");
            let provider = google::GoogleProvider::new(model)
                .map_err(|e| anyhow::anyhow!("Failed to create Google provider: {}", e))?;
            Ok(Arc::new(provider) as Arc<dyn ModelProvider>)
        }
        "ollama" => {
            let model = config.model.as_deref().unwrap_or("llama3.1");
            let provider = if let Some(ref base_url) = config.base_url {
                ollama::OllamaProvider::with_base_url(base_url, model)
            } else {
                ollama::OllamaProvider::new(model)
            };
            Ok(Arc::new(provider) as Arc<dyn ModelProvider>)
        }
        _ => {
            anyhow::bail!(
                "Unknown provider type '{}' for provider '{}'. \
                 Supported: anthropic, openai, bedrock, google, ollama",
                provider_type,
                name
            );
        }
    }
}

/// Internal event for local mode communication.
#[derive(Debug)]
pub enum LocalEvent {
    /// Agent produced a streaming chunk.
    StreamChunk {
        agent: String,
        content: String,
        done: bool,
    },
    /// Agent needs approval for an action.
    NeedsApproval {
        action_id: String,
        description: String,
        confidence: f32,
    },
    /// Log entry from agent activity.
    Log {
        agent: String,
        action: String,
        level: String,
    },
    /// Error occurred during execution.
    Error { message: String },
}

/// Run the TUI in local mode (no server connection).
pub async fn run_local(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    app: &mut App,
    runtime: LocalRuntime,
) -> anyhow::Result<()> {
    // Set up for local mode - creates project and switches to Chat view
    app.setup_local_mode(&runtime.workdir);
    app.connected = true;

    app.add_message("system", "Running in local mode");
    app.add_message(
        "system",
        format!("Provider: {}", runtime.default_provider),
    );
    app.add_message(
        "system",
        format!("Working directory: {}", runtime.workdir.display()),
    );

    // Channel for agent events
    let (event_tx, mut event_rx) = mpsc::channel::<LocalEvent>(100);

    let project_id = app.project_id.clone().expect("project set by setup_local_mode");

    // Store conversation history for context
    let mut conversation_history: Vec<cuttlefish_core::traits::provider::Message> = Vec::new();

    loop {
        terminal.draw(|f| ui::render(app, f))?;

        tokio::select! {
            // Keyboard input
            _ = tokio::time::sleep(Duration::from_millis(50)) => {
                while event::poll(Duration::ZERO)? {
                    if let Event::Key(key) = event::read()? {
                        // Check for Ctrl+C
                        if key.code == KeyCode::Char('c')
                            && key.modifiers.contains(KeyModifiers::CONTROL)
                        {
                            return Ok(());
                        }

                        match key.code {
                            KeyCode::Tab => app.next_view(),
                            KeyCode::Char(c) => {
                                match app.view {
                                    AppView::History | AppView::Dashboard => {
                                        // In local mode, Dashboard just shows single project
                                        // N key not needed since there's only one project
                                    }
                                    AppView::CreateProject => {
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
                                    AppView::History | AppView::Dashboard => {}
                                    AppView::CreateProject => {
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
                                    AppView::History => {
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
                                    AppView::Dashboard => {
                                        // Local mode uses single implicit project
                                        app.go_to_view(AppView::Chat);
                                    }
                                    AppView::CreateProject => {
                                        // Not really used in local mode
                                        app.view = AppView::Chat;
                                    }
                                    _ => {
                                        if !app.input.is_empty() {
                                            let input = std::mem::take(&mut app.input);
                                            app.add_to_history(&input);

                                            if let Some(cmd) = app.handle_command(&input) {
                                                match cmd {
                                                    app::SlashCommand::Help => app.go_to_view(AppView::Help),
                                                    app::SlashCommand::ListProjects => {
                                                        app.add_message("system", "Local mode uses a single implicit project");
                                                        app.go_to_view(AppView::Dashboard);
                                                    }
                                                    app::SlashCommand::NewProject { name } => {
                                                        app.add_message("system", format!(
                                                            "Project naming not supported in local mode. Current project: {}",
                                                            name.unwrap_or_else(|| project_id.clone())
                                                        ));
                                                    }
                                                    app::SlashCommand::SwitchProject { id } => {
                                                        app.add_message("system", format!(
                                                            "Project switching not supported in local mode. Current project: {}",
                                                            id
                                                        ));
                                                    }
                                                    app::SlashCommand::ClearChat => {
                                                        app.clear_chat();
                                                        conversation_history.clear();
                                                    }
                                                    app::SlashCommand::Quit => return Ok(()),
                                                }
                                            } else if !input.starts_with('/') {
                                                app.add_message("user", &input);

                                                conversation_history.push(cuttlefish_core::traits::provider::Message {
                                                    role: cuttlefish_core::traits::provider::MessageRole::User,
                                                    content: input.clone(),
                                                });

                                                let tx = event_tx.clone();
                                                let provider = runtime.provider();
                                                let prompts_dir = runtime.prompts_dir.clone();
                                                let history = conversation_history.clone();

                                                if let Some(provider) = provider {
                                                    tokio::spawn(async move {
                                                        execute_local_chat(
                                                            provider,
                                                            &prompts_dir,
                                                            history,
                                                            tx,
                                                        ).await;
                                                    });
                                                } else {
                                                    app.add_message("error", "No provider available");
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            KeyCode::Up => {
                                match app.view {
                                    AppView::History => app.history_prev(),
                                    AppView::Dashboard => app.select_prev_project(),
                                    AppView::CreateProject => app.create_project_prev(),
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
                                    AppView::History => app.history_next(),
                                    AppView::Dashboard => app.select_next_project(),
                                    AppView::CreateProject => app.create_project_next(),
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

            // Agent events
            Some(event) = event_rx.recv() => {
                match event {
                    LocalEvent::StreamChunk { agent, content, done } => {
                        if !app.streaming {
                            app.streaming = true;
                            app.add_message(&agent, &content);
                        } else {
                            app.append_to_message(&agent, &content);
                        }
                        if done {
                            app.streaming = false;

                            // Get the full response and add to conversation history
                            if let Some(last_msg) = app.messages.back() {
                                if last_msg.sender != "user" && last_msg.sender != "system" {
                                    conversation_history.push(cuttlefish_core::traits::provider::Message {
                                        role: cuttlefish_core::traits::provider::MessageRole::Assistant,
                                        content: last_msg.content.clone(),
                                    });
                                }
                            }
                        }
                    }
                    LocalEvent::NeedsApproval { action_id, description, confidence } => {
                        app.add_message("system", format!(
                            "[APPROVAL NEEDED] {} (confidence: {:.0}%)\n\
                             Type /approve {} or /reject {}",
                            description, confidence * 100.0, action_id, action_id
                        ));
                    }
                    LocalEvent::Log { agent, action, level } => {
                        app.add_log_line(format!("[{}] {}: {}", level.to_uppercase(), agent, action));
                    }
                    LocalEvent::Error { message } => {
                        app.streaming = false;
                        app.add_message("error", &message);
                    }
                }
            }
        }

        if app.should_exit {
            return Ok(());
        }
    }
}

/// Execute a chat message locally using the provider directly.
async fn execute_local_chat(
    provider: Arc<dyn ModelProvider>,
    prompts_dir: &Path,
    history: Vec<cuttlefish_core::traits::provider::Message>,
    tx: mpsc::Sender<LocalEvent>,
) {
    use cuttlefish_core::traits::provider::{CompletionRequest, StreamChunk};

    let _ = tx
        .send(LocalEvent::Log {
            agent: "assistant".to_string(),
            action: format!("Starting response ({} messages in context)", history.len()),
            level: "info".to_string(),
        })
        .await;

    // Load system prompt
    let registry = PromptRegistry::new(prompts_dir);
    let system_prompt = registry
        .load_with_system("coder")
        .map(|p| p.body)
        .unwrap_or_else(|_| {
            "You are a helpful AI coding assistant. You help users write, debug, and understand code. \
             Be concise and provide working code examples when appropriate.".to_string()
        });

    // Build request with full conversation history
    let request = CompletionRequest {
        messages: history,
        max_tokens: Some(4096),
        temperature: Some(0.7),
        system: Some(system_prompt),
    };

    // Stream response
    let mut stream = provider.stream(request);

    while let Some(chunk_result) = stream.next().await {
        match chunk_result {
            Ok(StreamChunk::Text(text)) => {
                let _ = tx
                    .send(LocalEvent::StreamChunk {
                        agent: "assistant".to_string(),
                        content: text,
                        done: false,
                    })
                    .await;
            }
            Ok(StreamChunk::Usage {
                input_tokens,
                output_tokens,
            }) => {
                debug!(
                    "Streaming complete: {} input, {} output tokens",
                    input_tokens, output_tokens
                );
                // Send final chunk marker
                let _ = tx
                    .send(LocalEvent::StreamChunk {
                        agent: "assistant".to_string(),
                        content: String::new(),
                        done: true,
                    })
                    .await;

                let _ = tx
                    .send(LocalEvent::Log {
                        agent: "assistant".to_string(),
                        action: format!(
                            "Response complete ({} input, {} output tokens)",
                            input_tokens, output_tokens
                        ),
                        level: "info".to_string(),
                    })
                    .await;
            }
            Ok(StreamChunk::ToolCallDelta { .. }) => {
                // Tool calls not yet supported in local TUI mode
            }
            Err(e) => {
                error!("Stream error: {}", e);
                let _ = tx
                    .send(LocalEvent::Error {
                        message: format!("Stream error: {}", e),
                    })
                    .await;
                return;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_local_event_debug() {
        let event = LocalEvent::StreamChunk {
            agent: "test".to_string(),
            content: "hello".to_string(),
            done: false,
        };
        assert!(format!("{:?}", event).contains("StreamChunk"));
    }

    #[test]
    fn test_local_event_error() {
        let event = LocalEvent::Error {
            message: "test error".to_string(),
        };
        if let LocalEvent::Error { message } = event {
            assert_eq!(message, "test error");
        } else {
            panic!("Expected Error variant");
        }
    }
}
