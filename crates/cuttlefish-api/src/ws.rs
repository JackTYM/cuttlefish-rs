//! WebSocket handler and message protocol.

use std::sync::Arc;

use axum::{
    extract::{
        State,
        ws::{Message as WsMessage, WebSocket, WebSocketUpgrade},
    },
    response::Response,
};
use cuttlefish_agents::{PromptRegistry, TokioMessageBus, WorkflowEngine};
use cuttlefish_core::traits::provider::{CompletionRequest, Message, MessageRole};
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::routes::{AppState, ProjectSession};

/// Inbound message from client to server.
#[derive(Debug, Clone, Deserialize)]
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
    /// Approve a pending action.
    Approve {
        /// Action ID to approve.
        action_id: String,
    },
    /// Reject a pending action.
    Reject {
        /// Action ID to reject.
        action_id: String,
        /// Optional reason for rejection.
        reason: Option<String>,
    },
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
}

/// Risk factor for a pending action.
#[derive(Debug, Clone, Serialize)]
pub struct RiskFactor {
    /// Type of risk factor.
    #[serde(rename = "type")]
    pub factor_type: String,
    /// Description of the risk.
    pub description: String,
}

/// Outbound message from server to client.
#[derive(Debug, Clone, Serialize)]
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
    /// An action requires user approval.
    PendingApproval {
        /// Unique action ID.
        action_id: String,
        /// Project ID.
        project_id: String,
        /// Type of action (file_write, command_exec, etc.).
        action_type: String,
        /// Human-readable description.
        description: String,
        /// File path if applicable.
        #[serde(skip_serializing_if = "Option::is_none")]
        path: Option<String>,
        /// Command if applicable.
        #[serde(skip_serializing_if = "Option::is_none")]
        command: Option<String>,
        /// Confidence score (0.0-1.0).
        confidence: f32,
        /// Reasoning for the confidence score.
        confidence_reasoning: String,
        /// Risk factors identified.
        #[serde(skip_serializing_if = "Option::is_none")]
        risk_factors: Option<Vec<RiskFactor>>,
        /// When the action was created.
        created_at: String,
        /// Timeout in seconds.
        timeout_secs: u32,
        /// Whether a diff preview is available.
        has_diff: bool,
    },
    /// A pending approval was resolved.
    ApprovalResolved {
        /// Action ID that was resolved.
        action_id: String,
    },
    /// Real-time log entry from agent activity.
    LogEntry {
        /// Unique log entry ID.
        id: String,
        /// Timestamp (ISO8601).
        timestamp: String,
        /// Agent name.
        agent: String,
        /// Action being performed.
        action: String,
        /// Log level (info, warn, error).
        level: String,
        /// Project name or ID.
        project: String,
        /// Additional context.
        #[serde(skip_serializing_if = "Option::is_none")]
        context: Option<String>,
        /// Stack trace for errors.
        #[serde(skip_serializing_if = "Option::is_none")]
        stack_trace: Option<String>,
    },
    /// Streaming text chunk from an agent.
    StreamChunk {
        /// Project ID.
        project_id: String,
        /// Agent name.
        agent: String,
        /// Text delta.
        content: String,
        /// Whether this is the final chunk.
        done: bool,
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
pub async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

/// Handle an established WebSocket connection.
async fn handle_socket(socket: WebSocket, state: AppState) {
    info!("New WebSocket connection");

    // Create a channel for sending messages back to this client
    let (tx, mut rx) = mpsc::channel::<ServerMessage>(100);

    // Split the WebSocket
    let (mut sender, mut receiver) = socket.split();

    // Spawn a task to forward messages from our channel to the WebSocket
    let send_task = tokio::spawn(async move {
        use futures::SinkExt;
        while let Some(msg) = rx.recv().await {
            let json = msg.to_json();
            if sender.send(WsMessage::Text(json.into())).await.is_err() {
                break;
            }
        }
    });

    // Track subscribed projects for this client
    let subscribed_projects: Arc<std::sync::RwLock<Vec<String>>> =
        Arc::new(std::sync::RwLock::new(Vec::new()));

    // Process incoming messages
    use futures::StreamExt;
    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(WsMessage::Text(text)) => {
                debug!("Received WebSocket text: {} bytes", text.len());

                match serde_json::from_str::<ClientMessage>(&text) {
                    Ok(ClientMessage::Ping) => {
                        if tx.send(ServerMessage::Pong).await.is_err() {
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

                        // Send log entry about receiving the message
                        let _ = tx
                            .send(ServerMessage::LogEntry {
                                id: Uuid::new_v4().to_string(),
                                timestamp: chrono::Utc::now().to_rfc3339(),
                                agent: "orchestrator".to_string(),
                                action: "Received chat message".to_string(),
                                level: "info".to_string(),
                                project: project_id.clone(),
                                context: Some(content[..content.len().min(100)].to_string()),
                                stack_trace: None,
                            })
                            .await;

                        // Execute streaming response for real-time token display
                        if let Err(e) =
                            execute_streaming(&state, &project_id, &content, tx.clone()).await
                        {
                            error!("Streaming execution error: {}", e);
                            if tx
                                .send(ServerMessage::Error {
                                    message: e.to_string(),
                                })
                                .await
                                .is_err()
                            {
                                break;
                            }
                        }
                    }
                    Ok(ClientMessage::Subscribe { project_id }) => {
                        info!("Client subscribed to project {}", project_id);
                        if let Ok(mut projects) = subscribed_projects.write()
                            && !projects.contains(&project_id)
                        {
                            projects.push(project_id.clone());
                        }
                        // Add client to session
                        add_client_to_session(&state, &project_id, tx.clone());
                    }
                    Ok(ClientMessage::Unsubscribe { project_id }) => {
                        info!("Client unsubscribed from project {}", project_id);
                        if let Ok(mut projects) = subscribed_projects.write() {
                            projects.retain(|p| p != &project_id);
                        }
                        // Note: We don't remove from session here as it's complex
                        // Sessions clean up on disconnect
                    }
                    Ok(ClientMessage::Approve { action_id }) => {
                        info!("Action approved: {}", action_id);
                        // Wire to approval registry
                        let found = state.approval_registry.approve(&action_id);
                        if !found {
                            warn!("Approval not found: {}", action_id);
                        }
                        if tx
                            .send(ServerMessage::ApprovalResolved {
                                action_id: action_id.clone(),
                            })
                            .await
                            .is_err()
                        {
                            break;
                        }
                    }
                    Ok(ClientMessage::Reject { action_id, reason }) => {
                        info!(
                            "Action rejected: {} (reason: {:?})",
                            action_id,
                            reason.as_deref().unwrap_or("none")
                        );
                        // Wire to approval registry
                        let found = state.approval_registry.reject(&action_id, reason);
                        if !found {
                            warn!("Approval not found for rejection: {}", action_id);
                        }
                        if tx
                            .send(ServerMessage::ApprovalResolved {
                                action_id: action_id.clone(),
                            })
                            .await
                            .is_err()
                        {
                            break;
                        }
                    }
                    Err(e) => {
                        warn!("Invalid WebSocket message: {}", e);
                        let err = ServerMessage::Error {
                            message: format!("Invalid message: {e}"),
                        };
                        if tx.send(err).await.is_err() {
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

    // Clean up: remove client from all subscribed sessions
    if let Ok(projects) = subscribed_projects.read() {
        for project_id in projects.iter() {
            remove_client_from_session(&state, project_id);
        }
    }

    // Wait for send task to finish
    send_task.abort();
    info!("WebSocket connection closed");
}

/// Add a client sender to a project session.
fn add_client_to_session(state: &AppState, project_id: &str, tx: mpsc::Sender<ServerMessage>) {
    state
        .active_sessions
        .entry(project_id.to_string())
        .or_insert_with(|| ProjectSession::new(project_id.to_string()))
        .clients
        .push(tx);
}

/// Remove a client from a project session (simplified - removes last added).
fn remove_client_from_session(state: &AppState, project_id: &str) {
    if let Some(mut session) = state.active_sessions.get_mut(project_id) {
        session.clients.pop();
    }
}

/// Execute streaming chat - streams tokens directly from the provider.
///
/// This provides real-time token-by-token streaming for a responsive UI.
/// Maintains conversation history with automatic context compaction.
/// Messages are persisted to database for crash recovery.
async fn execute_streaming(
    state: &AppState,
    project_id: &str,
    input: &str,
    tx: mpsc::Sender<ServerMessage>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let provider = get_provider(state)?;

    // Create user message
    let user_message = Message {
        role: MessageRole::User,
        content: input.to_string(),
    };

    // Estimate token count for the user message
    let user_token_count = (input.len() / 4).max(1) as i64;

    // Save user message to persistence (if available)
    if let Some(ref persistence) = state.persistence {
        let mut p = persistence.lock().await;
        match p
            .save_message(project_id, &user_message, user_token_count, None)
            .await
        {
            Ok(msg_id) => debug!("User message persisted: {}", msg_id),
            Err(e) => warn!("Failed to persist user message: {}", e),
        }
    }

    // Get or create session and update history
    let (messages, token_count, compacted) = {
        let mut session_entry = state
            .active_sessions
            .entry(project_id.to_string())
            .or_insert_with(|| ProjectSession::new(project_id.to_string()));

        // Add user message to history
        session_entry.messages.push(user_message);

        // Compact history if needed
        // Take messages out temporarily to avoid borrow conflict
        let mut messages = std::mem::take(&mut session_entry.messages);
        let needs_compaction = session_entry.compactor.needs_compaction(&messages);
        let compacted = if needs_compaction {
            let result = session_entry.compactor.compact(&mut messages);
            info!(
                "Context compacted: {} -> {} tokens, {} messages removed",
                result.tokens_before, result.tokens_after, result.messages_removed
            );

            // Record compaction in persistence
            if let Some(ref persistence) = state.persistence {
                let mut p = persistence.lock().await;
                let _ = p.record_compaction(
                    project_id,
                    result.messages_removed,
                    result.tokens_before,
                    result.tokens_after,
                );
            }

            true
        } else {
            false
        };

        // Put messages back and get a copy for the request
        let token_count = session_entry.compactor.current_tokens(&messages);
        let messages_for_request = messages.clone();
        session_entry.messages = messages;

        (messages_for_request, token_count, compacted)
    }; // Session lock released here

    // Send log entry about starting
    let _ = tx
        .send(ServerMessage::LogEntry {
            id: Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            agent: "assistant".to_string(),
            action: format!(
                "Starting streaming response (context: {} tokens, {} messages{})",
                token_count,
                messages.len(),
                if compacted { ", compacted" } else { "" }
            ),
            level: "info".to_string(),
            project: project_id.to_string(),
            context: None,
            stack_trace: None,
        })
        .await;

    // Load system prompt from prompts directory
    let registry = PromptRegistry::new(&state.prompts_dir);
    let system_prompt = registry
        .load_with_system("coder")
        .map(|p| p.body)
        .unwrap_or_else(|_| "You are a helpful AI coding assistant.".to_string());

    // Build the completion request with full history
    let request = CompletionRequest {
        messages,
        max_tokens: Some(4096),
        temperature: Some(0.7),
        system: Some(system_prompt),
    };

    // Stream the response
    let mut stream = provider.stream(request);
    let mut full_content = String::new();

    while let Some(chunk_result) = stream.next().await {
        match chunk_result {
            Ok(chunk) => {
                use cuttlefish_core::traits::provider::StreamChunk;
                match chunk {
                    StreamChunk::Text(text) => {
                        full_content.push_str(&text);
                        let _ = tx
                            .send(ServerMessage::StreamChunk {
                                project_id: project_id.to_string(),
                                agent: "assistant".to_string(),
                                content: text,
                                done: false,
                            })
                            .await;
                    }
                    StreamChunk::Usage {
                        input_tokens,
                        output_tokens,
                    } => {
                        debug!(
                            "Streaming complete: {} input, {} output tokens",
                            input_tokens, output_tokens
                        );
                        // Send final chunk marker
                        let _ = tx
                            .send(ServerMessage::StreamChunk {
                                project_id: project_id.to_string(),
                                agent: "assistant".to_string(),
                                content: String::new(),
                                done: true,
                            })
                            .await;
                    }
                    StreamChunk::ToolCallDelta { .. } => {
                        // Tool calls not yet supported in streaming UI
                    }
                }
            }
            Err(e) => {
                error!("Stream error: {}", e);
                let _ = tx
                    .send(ServerMessage::Error {
                        message: format!("Stream error: {e}"),
                    })
                    .await;
                return Err(Box::new(std::io::Error::other(e.to_string())));
            }
        }
    }

    // Add assistant response to session history and persist
    if !full_content.is_empty() {
        let assistant_message = Message {
            role: MessageRole::Assistant,
            content: full_content.clone(),
        };

        // Save to session
        if let Some(mut session) = state.active_sessions.get_mut(project_id) {
            session.add_message(assistant_message.clone());
        }

        // Persist to database
        let assistant_token_count = (full_content.len() / 4).max(1) as i64;
        if let Some(ref persistence) = state.persistence {
            let mut p = persistence.lock().await;
            match p
                .save_message(
                    project_id,
                    &assistant_message,
                    assistant_token_count,
                    state.default_provider.as_deref(),
                )
                .await
            {
                Ok(msg_id) => debug!("Assistant message persisted: {}", msg_id),
                Err(e) => warn!("Failed to persist assistant message: {}", e),
            }
        }
    }

    // Send log entry about completion
    let _ = tx
        .send(ServerMessage::LogEntry {
            id: Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            agent: "assistant".to_string(),
            action: "Streaming response complete".to_string(),
            level: "info".to_string(),
            project: project_id.to_string(),
            context: Some(format!("{} characters", full_content.len())),
            stack_trace: None,
        })
        .await;

    Ok(())
}

/// Execute the agent workflow for a chat message (non-streaming).
///
/// Use this for complex multi-agent tasks that require orchestration.
/// Currently unused in favor of direct streaming, but kept for future
/// multi-agent workflow support.
#[allow(dead_code)]
async fn execute_workflow(
    state: &AppState,
    project_id: &str,
    input: &str,
    tx: mpsc::Sender<ServerMessage>,
) -> Result<ServerMessage, Box<dyn std::error::Error + Send + Sync>> {
    // Get or create workflow for this project
    let provider = get_provider(state)?;

    // Send log entry about starting workflow
    let _ = tx
        .send(ServerMessage::LogEntry {
            id: Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            agent: "orchestrator".to_string(),
            action: "Starting workflow execution".to_string(),
            level: "info".to_string(),
            project: project_id.to_string(),
            context: None,
            stack_trace: None,
        })
        .await;

    // Create workflow engine
    let bus = TokioMessageBus::new();
    let workflow = WorkflowEngine::with_all_agents(provider, bus, &state.prompts_dir);

    // Parse project ID as UUID (or generate one if it's not a valid UUID)
    let uuid = Uuid::try_parse(project_id).unwrap_or_else(|_| {
        // Create a deterministic UUID from the project_id string
        Uuid::new_v5(&Uuid::NAMESPACE_OID, project_id.as_bytes())
    });

    // Send log entry about running agents
    let _ = tx
        .send(ServerMessage::LogEntry {
            id: Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            agent: "coder".to_string(),
            action: "Executing task".to_string(),
            level: "info".to_string(),
            project: project_id.to_string(),
            context: Some(input[..input.len().min(100)].to_string()),
            stack_trace: None,
        })
        .await;

    // Execute the workflow
    let result = workflow.execute(uuid, input).await.map_err(|e| {
        Box::new(std::io::Error::other(e.to_string())) as Box<dyn std::error::Error + Send + Sync>
    })?;

    // Send log entry about completion
    let _ = tx
        .send(ServerMessage::LogEntry {
            id: Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            agent: "critic".to_string(),
            action: format!(
                "Workflow completed (iterations: {}, success: {})",
                result.iterations, result.success
            ),
            level: if result.success { "info" } else { "warn" }.to_string(),
            project: project_id.to_string(),
            context: result.final_verdict.clone(),
            stack_trace: None,
        })
        .await;

    Ok(ServerMessage::Response {
        project_id: project_id.to_string(),
        agent: "workflow".to_string(),
        content: result.content,
    })
}

/// Execute an action with safety gate evaluation.
///
/// # Lint exception
/// This function takes many parameters because it needs full action context
/// for safety evaluation. A builder pattern would add complexity without benefit.
///
/// This function demonstrates the safety workflow integration:
/// 1. Evaluates the action through the ActionGate
/// 2. If auto-approved, returns Ok immediately
/// 3. If requires prompt, registers a pending approval and waits
/// 4. If blocked, returns an error
///
/// # Arguments
/// * `state` - Application state containing the approval registry
/// * `project_id` - Project this action belongs to
/// * `action_type` - Type of action being performed
/// * `description` - Human-readable description
/// * `confidence` - Confidence score for the action
/// * `tx` - Channel to send pending approval notification to client
///
/// # Returns
/// * `Ok(true)` - Action approved (auto or by user)
/// * `Ok(false)` - Action rejected by user
/// * `Err` - Action blocked by confidence threshold or timed out
#[allow(clippy::too_many_arguments)]
pub async fn execute_with_safety(
    state: &AppState,
    project_id: &str,
    action_type: cuttlefish_agents::ActionType,
    description: &str,
    confidence: cuttlefish_agents::ConfidenceScore,
    path: Option<String>,
    command: Option<String>,
    diff: Option<String>,
    tx: mpsc::Sender<ServerMessage>,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    use crate::approval_registry::PendingApproval;
    use cuttlefish_agents::{ActionGate, ActionPreview, GateDecision};

    let gate = ActionGate::with_defaults();
    let preview = ActionPreview::new(description, action_type);

    match gate.evaluate(action_type, &confidence, preview) {
        GateDecision::AutoApprove => {
            info!("Action auto-approved: {}", description);
            Ok(true)
        }
        GateDecision::PromptUser {
            preview: _,
            confidence: conf,
        } => {
            info!("Action requires user approval: {}", description);

            // Create pending approval
            let mut pending =
                PendingApproval::new(project_id, action_type, description, conf.clone());

            if let Some(p) = path.clone() {
                pending = pending.with_path(p);
            }
            if let Some(c) = command.clone() {
                pending = pending.with_command(c);
            }
            if let Some(d) = diff.clone() {
                pending = pending.with_diff(d);
            }

            let action_id = pending.action_id.clone();

            // Convert confidence factors to risk factor descriptions for the UI
            let risk_factors: Vec<RiskFactor> = conf
                .factors()
                .iter()
                .filter(|f| f.value() < 0.7) // Only include factors that reduce confidence
                .map(|f| RiskFactor {
                    factor_type: f.name().to_string(),
                    description: format!("{}: {:.0}%", f.name(), f.value() * 100.0),
                })
                .collect();

            // Notify client about pending approval
            let _ = tx
                .send(ServerMessage::PendingApproval {
                    action_id: action_id.clone(),
                    project_id: project_id.to_string(),
                    action_type: format!("{:?}", action_type),
                    description: description.to_string(),
                    path: pending.path.clone(),
                    command: pending.command.clone(),
                    confidence: conf.value(),
                    confidence_reasoning: conf.reasoning().to_string(),
                    risk_factors: if risk_factors.is_empty() {
                        None
                    } else {
                        Some(risk_factors)
                    },
                    created_at: chrono::Utc::now().to_rfc3339(),
                    timeout_secs: 300,
                    has_diff: diff.is_some(),
                })
                .await;

            // Wait for user decision
            let decision = state.approval_registry.request_approval(pending).await;

            // Notify client that approval was resolved
            let _ = tx.send(ServerMessage::ApprovalResolved { action_id }).await;

            match decision {
                crate::approval_registry::ApprovalDecision::Approved => {
                    info!("Action approved by user: {}", description);
                    Ok(true)
                }
                crate::approval_registry::ApprovalDecision::Rejected { reason } => {
                    info!(
                        "Action rejected by user: {} (reason: {:?})",
                        description, reason
                    );
                    Ok(false)
                }
                crate::approval_registry::ApprovalDecision::TimedOut => {
                    warn!("Action approval timed out: {}", description);
                    Err("Approval timed out".into())
                }
            }
        }
        GateDecision::Block { reason } => {
            warn!("Action blocked: {} - {}", description, reason);
            Err(format!("Action blocked: {}", reason).into())
        }
    }
}

/// Get the default provider from the registry.
fn get_provider(
    state: &AppState,
) -> Result<
    Arc<dyn cuttlefish_core::traits::provider::ModelProvider>,
    Box<dyn std::error::Error + Send + Sync>,
> {
    // Try to get the default provider
    if let Some(ref default_name) = state.default_provider
        && let Some(provider) = state.provider_registry.get(default_name)
    {
        return Ok(provider);
    }

    // Try common provider names
    for name in ["anthropic", "claude", "openai", "bedrock"] {
        if let Some(provider) = state.provider_registry.get(name) {
            return Ok(provider);
        }
    }

    // Get any available provider
    let names = state.provider_registry.names();
    if let Some(name) = names.first()
        && let Some(provider) = state.provider_registry.get(name)
    {
        return Ok(provider);
    }

    Err("No model providers configured. Add provider configuration to cuttlefish.toml".into())
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
    fn test_server_message_pending_approval_serializes() {
        let json = ServerMessage::PendingApproval {
            action_id: "act-1".to_string(),
            project_id: "proj-1".to_string(),
            action_type: "file_write".to_string(),
            description: "Write to main.rs".to_string(),
            path: Some("src/main.rs".to_string()),
            command: None,
            confidence: 0.65,
            confidence_reasoning: "Modifies source code".to_string(),
            risk_factors: Some(vec![RiskFactor {
                factor_type: "source_modification".to_string(),
                description: "Modifies source files".to_string(),
            }]),
            created_at: "2024-01-15T10:00:00Z".to_string(),
            timeout_secs: 300,
            has_diff: true,
        }
        .to_json();
        assert!(json.contains("pending_approval"));
        assert!(json.contains("file_write"));
        assert!(json.contains("confidence"));
    }

    #[test]
    fn test_server_message_log_entry_serializes() {
        let json = ServerMessage::LogEntry {
            id: "log-1".to_string(),
            timestamp: "2024-01-15T10:00:00Z".to_string(),
            agent: "coder".to_string(),
            action: "Writing code".to_string(),
            level: "info".to_string(),
            project: "proj-1".to_string(),
            context: Some("Creating module".to_string()),
            stack_trace: None,
        }
        .to_json();
        assert!(json.contains("log_entry"));
        assert!(json.contains("coder"));
        assert!(json.contains("Writing code"));
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

    #[test]
    fn test_client_message_approve_deserializes() {
        let json = r#"{"type":"approve","action_id":"act-123"}"#;
        let msg: ClientMessage = serde_json::from_str(json).expect("parse");
        if let ClientMessage::Approve { action_id } = msg {
            assert_eq!(action_id, "act-123");
        } else {
            panic!("Expected Approve variant");
        }
    }

    #[test]
    fn test_client_message_reject_deserializes() {
        let json = r#"{"type":"reject","action_id":"act-123","reason":"Too risky"}"#;
        let msg: ClientMessage = serde_json::from_str(json).expect("parse");
        if let ClientMessage::Reject { action_id, reason } = msg {
            assert_eq!(action_id, "act-123");
            assert_eq!(reason, Some("Too risky".to_string()));
        } else {
            panic!("Expected Reject variant");
        }
    }

    #[test]
    fn test_client_message_subscribe_deserializes() {
        let json = r#"{"type":"subscribe","project_id":"proj-1"}"#;
        let msg: ClientMessage = serde_json::from_str(json).expect("parse");
        if let ClientMessage::Subscribe { project_id } = msg {
            assert_eq!(project_id, "proj-1");
        } else {
            panic!("Expected Subscribe variant");
        }
    }
}
