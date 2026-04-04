//! Agent execution runner with tool calling and timeout enforcement.

use cuttlefish_core::{
    error::AgentError,
    traits::{
        agent::{Agent, AgentContext, AgentOutput},
        provider::{Message, MessageRole, ToolCall},
        sandbox::{Sandbox, SandboxId},
    },
};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::safety::{
    ActionGate, ActionPreview, ActionType, ConfidenceCalculator, ConfidenceScore, FileDiff,
    GateConfig, GateDecision,
};
use crate::tools::ToolRegistry;

/// Maximum model↔tool iterations per invocation.
pub const MAX_ITERATIONS: usize = 25;
/// Default agent invocation timeout in seconds.
pub const DEFAULT_TIMEOUT_SECS: u64 = 300;
/// Default approval timeout in seconds.
pub const DEFAULT_APPROVAL_TIMEOUT_SECS: u64 = 300;

/// Configuration for the agent runner.
#[derive(Debug, Clone)]
pub struct RunnerConfig {
    /// Maximum iterations before forced stop.
    pub max_iterations: usize,
    /// Hard timeout for the entire invocation.
    pub timeout: Duration,
    /// Whether safety gates are enabled.
    pub safety_gates_enabled: bool,
    /// Timeout for waiting on user approval.
    pub approval_timeout: Duration,
}

impl Default for RunnerConfig {
    fn default() -> Self {
        Self {
            max_iterations: MAX_ITERATIONS,
            timeout: Duration::from_secs(DEFAULT_TIMEOUT_SECS),
            safety_gates_enabled: true,
            approval_timeout: Duration::from_secs(DEFAULT_APPROVAL_TIMEOUT_SECS),
        }
    }
}

impl RunnerConfig {
    /// Disable safety gates (for testing or trusted operations).
    pub fn without_safety_gates(mut self) -> Self {
        self.safety_gates_enabled = false;
        self
    }

    /// Set approval timeout.
    pub fn with_approval_timeout(mut self, timeout: Duration) -> Self {
        self.approval_timeout = timeout;
        self
    }
}

/// Result of executing a single tool call.
#[derive(Debug)]
pub struct ToolExecutionResult {
    /// The tool call ID (from model).
    pub id: String,
    /// Tool output content.
    pub content: String,
    /// Whether execution succeeded.
    pub success: bool,
}

/// A pending action awaiting user approval.
#[derive(Debug, Clone)]
pub struct PendingAction {
    /// Unique action ID.
    pub id: String,
    /// The tool call that triggered this action.
    pub tool_call: ToolCall,
    /// Action type.
    pub action_type: ActionType,
    /// Action preview.
    pub preview: ActionPreview,
    /// Confidence score.
    pub confidence: ConfidenceScore,
    /// File diff (if applicable).
    pub diff: Option<FileDiff>,
}

/// Status of a pending action.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PendingActionStatus {
    /// Awaiting user decision.
    Pending,
    /// User approved the action.
    Approved,
    /// User rejected the action.
    Rejected,
    /// Action timed out.
    TimedOut,
}

/// Executes tool calls requested by the model.
pub struct ToolExecutor {
    sandbox: Option<Arc<dyn Sandbox>>,
    sandbox_id: Option<SandboxId>,
    _registry: ToolRegistry,
}

impl ToolExecutor {
    /// Create a new executor optionally backed by a Docker sandbox.
    pub fn new(sandbox: Option<Arc<dyn Sandbox>>, sandbox_id: Option<SandboxId>) -> Self {
        Self {
            sandbox,
            sandbox_id,
            _registry: ToolRegistry::with_defaults(),
        }
    }

    /// Execute a tool call and return the result.
    pub async fn execute(&self, call: &ToolCall) -> ToolExecutionResult {
        debug!("Executing tool: {}", call.name);
        match call.name.as_str() {
            crate::tools::built_in::EXECUTE_COMMAND => {
                let cmd = call.input["command"].as_str().unwrap_or("");
                if let (Some(sb), Some(id)) = (&self.sandbox, &self.sandbox_id) {
                    match sb.exec(id, cmd).await {
                        Ok(out) => ToolExecutionResult {
                            id: call.id.clone(),
                            content: format!(
                                "stdout: {}\nstderr: {}\nexit: {}",
                                out.stdout, out.stderr, out.exit_code
                            ),
                            success: out.success(),
                        },
                        Err(e) => ToolExecutionResult {
                            id: call.id.clone(),
                            content: format!("Error: {e}"),
                            success: false,
                        },
                    }
                } else {
                    ToolExecutionResult {
                        id: call.id.clone(),
                        content: format!("[SIMULATION] Would execute: {}", cmd),
                        success: true,
                    }
                }
            }
            crate::tools::built_in::READ_FILE => {
                let path = call.input["path"].as_str().unwrap_or("");
                if let (Some(sb), Some(id)) = (&self.sandbox, &self.sandbox_id) {
                    match sb.read_file(id, std::path::Path::new(path)).await {
                        Ok(bytes) => ToolExecutionResult {
                            id: call.id.clone(),
                            content: String::from_utf8_lossy(&bytes).to_string(),
                            success: true,
                        },
                        Err(e) => ToolExecutionResult {
                            id: call.id.clone(),
                            content: format!("Error: {e}"),
                            success: false,
                        },
                    }
                } else {
                    ToolExecutionResult {
                        id: call.id.clone(),
                        content: format!("[SIMULATION] Would read: {}", path),
                        success: true,
                    }
                }
            }
            crate::tools::built_in::WRITE_FILE => {
                let path = call.input["path"].as_str().unwrap_or("");
                let content = call.input["content"].as_str().unwrap_or("");
                if let (Some(sb), Some(id)) = (&self.sandbox, &self.sandbox_id) {
                    match sb
                        .write_file(id, std::path::Path::new(path), content.as_bytes())
                        .await
                    {
                        Ok(()) => ToolExecutionResult {
                            id: call.id.clone(),
                            content: format!("Wrote {} bytes to {}", content.len(), path),
                            success: true,
                        },
                        Err(e) => ToolExecutionResult {
                            id: call.id.clone(),
                            content: format!("Error: {e}"),
                            success: false,
                        },
                    }
                } else {
                    ToolExecutionResult {
                        id: call.id.clone(),
                        content: format!(
                            "[SIMULATION] Would write {} bytes to {}",
                            content.len(),
                            path
                        ),
                        success: true,
                    }
                }
            }
            crate::tools::built_in::LIST_DIRECTORY => {
                let path = call.input["path"].as_str().unwrap_or("/workspace");
                if let (Some(sb), Some(id)) = (&self.sandbox, &self.sandbox_id) {
                    match sb.list_files(id, std::path::Path::new(path)).await {
                        Ok(files) => ToolExecutionResult {
                            id: call.id.clone(),
                            content: files.join("\n"),
                            success: true,
                        },
                        Err(e) => ToolExecutionResult {
                            id: call.id.clone(),
                            content: format!("Error: {e}"),
                            success: false,
                        },
                    }
                } else {
                    ToolExecutionResult {
                        id: call.id.clone(),
                        content: format!("[SIMULATION] Would list: {}", path),
                        success: true,
                    }
                }
            }
            crate::tools::built_in::SEARCH_FILES => {
                let pattern = call.input["pattern"].as_str().unwrap_or("*");
                ToolExecutionResult {
                    id: call.id.clone(),
                    content: format!("[SIMULATION] Would search for: {}", pattern),
                    success: true,
                }
            }
            crate::tools::built_in::EDIT_FILE => {
                let path = call.input["path"].as_str().unwrap_or("");
                let edits_json = call.input["edits"].as_array();

                if let (Some(sb), Some(id)) = (&self.sandbox, &self.sandbox_id) {
                    let content = match sb.read_file(id, std::path::Path::new(path)).await {
                        Ok(bytes) => String::from_utf8_lossy(&bytes).to_string(),
                        Err(e) => {
                            return ToolExecutionResult {
                                id: call.id.clone(),
                                content: format!("Error reading file: {e}"),
                                success: false,
                            };
                        }
                    };

                    let edits: Vec<cuttlefish_core::hashline::LineEdit> = match edits_json {
                        Some(arr) => arr
                            .iter()
                            .map(|e| cuttlefish_core::hashline::LineEdit {
                                hash: e["hash"].as_str().unwrap_or("").to_string(),
                                expected_content: e["expected_content"]
                                    .as_str()
                                    .map(|s| s.to_string()),
                                new_content: e["new_content"].as_str().map(|s| s.to_string()),
                            })
                            .collect(),
                        None => {
                            return ToolExecutionResult {
                                id: call.id.clone(),
                                content: "Error: 'edits' array required".to_string(),
                                success: false,
                            };
                        }
                    };

                    match cuttlefish_core::hashline::apply_edits(&content, &edits) {
                        Ok(new_content) => {
                            match sb
                                .write_file(id, std::path::Path::new(path), new_content.as_bytes())
                                .await
                            {
                                Ok(()) => ToolExecutionResult {
                                    id: call.id.clone(),
                                    content: format!("Applied {} edit(s) to {}", edits.len(), path),
                                    success: true,
                                },
                                Err(e) => ToolExecutionResult {
                                    id: call.id.clone(),
                                    content: format!("Error writing file: {e}"),
                                    success: false,
                                },
                            }
                        }
                        Err(e) => ToolExecutionResult {
                            id: call.id.clone(),
                            content: format!("Edit error: {e}"),
                            success: false,
                        },
                    }
                } else {
                    let edit_count = edits_json.map(|a| a.len()).unwrap_or(0);
                    ToolExecutionResult {
                        id: call.id.clone(),
                        content: format!(
                            "[SIMULATION] Would apply {} edit(s) to {}",
                            edit_count, path
                        ),
                        success: true,
                    }
                }
            }
            unknown => {
                warn!("Unknown tool: {}", unknown);
                ToolExecutionResult {
                    id: call.id.clone(),
                    content: format!("Error: Unknown tool '{}'", unknown),
                    success: false,
                }
            }
        }
    }
}

/// Safety-gated tool executor that checks confidence gates before execution.
pub struct SafetyGatedExecutor {
    inner: ToolExecutor,
    gate: ActionGate,
    confidence_calculator: ConfidenceCalculator,
    pending_actions: Arc<RwLock<HashMap<String, (PendingAction, PendingActionStatus)>>>,
    enabled: bool,
}

impl SafetyGatedExecutor {
    /// Create a new safety-gated executor.
    pub fn new(
        sandbox: Option<Arc<dyn Sandbox>>,
        sandbox_id: Option<SandboxId>,
        gate_config: GateConfig,
    ) -> Self {
        Self {
            inner: ToolExecutor::new(sandbox, sandbox_id),
            gate: ActionGate::new(gate_config),
            confidence_calculator: ConfidenceCalculator::new(),
            pending_actions: Arc::new(RwLock::new(HashMap::new())),
            enabled: true,
        }
    }

    /// Create a safety-gated executor with default configuration.
    pub fn with_defaults(sandbox: Option<Arc<dyn Sandbox>>, sandbox_id: Option<SandboxId>) -> Self {
        Self::new(sandbox, sandbox_id, GateConfig::default())
    }

    /// Disable safety gates (bypass all checks).
    pub fn disable_gates(&mut self) {
        self.enabled = false;
    }

    /// Enable safety gates.
    pub fn enable_gates(&mut self) {
        self.enabled = true;
    }

    /// Check if gates are enabled.
    pub fn gates_enabled(&self) -> bool {
        self.enabled
    }

    /// Get the action type for a tool call.
    fn get_action_type(&self, call: &ToolCall) -> ActionType {
        match call.name.as_str() {
            crate::tools::built_in::WRITE_FILE | crate::tools::built_in::EDIT_FILE => {
                ActionType::FileWrite
            }
            crate::tools::built_in::EXECUTE_COMMAND => {
                let cmd = call.input["command"].as_str().unwrap_or("");
                if cmd.starts_with("git ") {
                    ActionType::GitOperation
                } else if cmd.contains("rm ") || cmd.contains("delete") {
                    ActionType::FileDelete
                } else {
                    ActionType::BashCommand
                }
            }
            _ => ActionType::BashCommand,
        }
    }

    /// Create an action preview for a tool call.
    fn create_preview(&self, call: &ToolCall, action_type: ActionType) -> ActionPreview {
        let description = match call.name.as_str() {
            crate::tools::built_in::WRITE_FILE => {
                let path = call.input["path"].as_str().unwrap_or("unknown");
                format!("Write to file: {}", path)
            }
            crate::tools::built_in::EDIT_FILE => {
                let path = call.input["path"].as_str().unwrap_or("unknown");
                format!("Edit file: {}", path)
            }
            crate::tools::built_in::EXECUTE_COMMAND => {
                let cmd = call.input["command"].as_str().unwrap_or("unknown");
                format!("Execute: {}", cmd)
            }
            _ => format!("Execute tool: {}", call.name),
        };

        let mut preview = ActionPreview::new(description, action_type);

        if let Some(path) = call.input["path"].as_str() {
            preview = preview.with_path(path);
        }
        if let Some(cmd) = call.input["command"].as_str() {
            preview = preview.with_command(cmd);
        }

        preview
    }

    /// Execute a tool call with safety gate checks.
    pub async fn execute(&self, call: &ToolCall) -> Result<ToolExecutionResult, GatedExecutionError> {
        if !self.enabled {
            return Ok(self.inner.execute(call).await);
        }

        let action_type = self.get_action_type(call);

        if matches!(
            call.name.as_str(),
            crate::tools::built_in::READ_FILE
                | crate::tools::built_in::LIST_DIRECTORY
                | crate::tools::built_in::SEARCH_FILES
        ) {
            return Ok(self.inner.execute(call).await);
        }

        let confidence = self.confidence_calculator.calculate_for_tool_call(call);
        let preview = self.create_preview(call, action_type);

        let decision = self.gate.evaluate(action_type, &confidence, preview.clone());

        match decision {
            GateDecision::AutoApprove => {
                debug!(
                    tool = %call.name,
                    confidence = %confidence.value(),
                    "Action auto-approved"
                );
                Ok(self.inner.execute(call).await)
            }
            GateDecision::PromptUser { preview, confidence } => {
                let action_id = uuid::Uuid::new_v4().to_string();
                let pending = PendingAction {
                    id: action_id.clone(),
                    tool_call: call.clone(),
                    action_type,
                    preview,
                    confidence: confidence.clone(),
                    diff: None,
                };

                {
                    let mut actions = self.pending_actions.write().await;
                    actions.insert(action_id.clone(), (pending, PendingActionStatus::Pending));
                }

                info!(
                    action_id = %action_id,
                    tool = %call.name,
                    confidence = %confidence.value(),
                    "Action queued for approval"
                );

                Err(GatedExecutionError::RequiresApproval { action_id })
            }
            GateDecision::Block { reason } => {
                warn!(
                    tool = %call.name,
                    confidence = %confidence.value(),
                    reason = %reason,
                    "Action blocked"
                );
                Err(GatedExecutionError::Blocked { reason })
            }
        }
    }

    /// Approve a pending action and execute it.
    pub async fn approve_action(&self, action_id: &str) -> Result<ToolExecutionResult, GatedExecutionError> {
        let pending = {
            let mut actions = self.pending_actions.write().await;
            if let Some((action, status)) = actions.get_mut(action_id) {
                if *status != PendingActionStatus::Pending {
                    return Err(GatedExecutionError::ActionNotPending {
                        action_id: action_id.to_string(),
                    });
                }
                *status = PendingActionStatus::Approved;
                Some(action.clone())
            } else {
                None
            }
        };

        match pending {
            Some(action) => {
                info!(action_id = %action_id, "Action approved, executing");
                Ok(self.inner.execute(&action.tool_call).await)
            }
            None => Err(GatedExecutionError::ActionNotFound {
                action_id: action_id.to_string(),
            }),
        }
    }

    /// Reject a pending action.
    pub async fn reject_action(&self, action_id: &str) -> Result<(), GatedExecutionError> {
        let mut actions = self.pending_actions.write().await;
        if let Some((_, status)) = actions.get_mut(action_id) {
            if *status != PendingActionStatus::Pending {
                return Err(GatedExecutionError::ActionNotPending {
                    action_id: action_id.to_string(),
                });
            }
            *status = PendingActionStatus::Rejected;
            info!(action_id = %action_id, "Action rejected");
            Ok(())
        } else {
            Err(GatedExecutionError::ActionNotFound {
                action_id: action_id.to_string(),
            })
        }
    }

    /// Get a pending action by ID.
    pub async fn get_pending_action(&self, action_id: &str) -> Option<PendingAction> {
        let actions = self.pending_actions.read().await;
        actions.get(action_id).map(|(a, _)| a.clone())
    }

    /// List all pending actions.
    pub async fn list_pending_actions(&self) -> Vec<PendingAction> {
        let actions = self.pending_actions.read().await;
        actions
            .values()
            .filter(|(_, status)| *status == PendingActionStatus::Pending)
            .map(|(a, _)| a.clone())
            .collect()
    }
}

/// Errors that can occur during gated execution.
#[derive(Debug, thiserror::Error)]
pub enum GatedExecutionError {
    /// Action requires user approval before proceeding.
    #[error("Action requires approval: {action_id}")]
    RequiresApproval {
        /// The ID of the pending action.
        action_id: String,
    },

    /// Action was blocked due to low confidence.
    #[error("Action blocked: {reason}")]
    Blocked {
        /// Reason for blocking.
        reason: String,
    },

    /// Action was not found.
    #[error("Action not found: {action_id}")]
    ActionNotFound {
        /// The action ID that was not found.
        action_id: String,
    },

    /// Action is not in pending state.
    #[error("Action not pending: {action_id}")]
    ActionNotPending {
        /// The action ID.
        action_id: String,
    },
}

/// Drives an agent through its execution loop with timeout enforcement.
pub struct AgentRunner {
    /// Runner configuration.
    pub config: RunnerConfig,
    /// Tool executor for handling model tool calls.
    pub tool_executor: ToolExecutor,
    /// Safety-gated executor (optional).
    pub gated_executor: Option<SafetyGatedExecutor>,
}

impl AgentRunner {
    /// Create a runner with default config and optional sandbox.
    pub fn new(sandbox: Option<Arc<dyn Sandbox>>, sandbox_id: Option<SandboxId>) -> Self {
        Self {
            config: RunnerConfig::default(),
            tool_executor: ToolExecutor::new(sandbox.clone(), sandbox_id.clone()),
            gated_executor: Some(SafetyGatedExecutor::with_defaults(sandbox, sandbox_id)),
        }
    }

    /// Create a runner without safety gates.
    pub fn without_safety_gates(sandbox: Option<Arc<dyn Sandbox>>, sandbox_id: Option<SandboxId>) -> Self {
        Self {
            config: RunnerConfig::default().without_safety_gates(),
            tool_executor: ToolExecutor::new(sandbox, sandbox_id),
            gated_executor: None,
        }
    }

    /// Get the gated executor if available.
    pub fn gated_executor(&self) -> Option<&SafetyGatedExecutor> {
        self.gated_executor.as_ref()
    }

    /// Get a mutable reference to the gated executor if available.
    pub fn gated_executor_mut(&mut self) -> Option<&mut SafetyGatedExecutor> {
        self.gated_executor.as_mut()
    }

    /// Run an agent with timeout enforcement.
    pub async fn run(
        &self,
        agent: &dyn Agent,
        ctx: &mut AgentContext,
        input: &str,
    ) -> Result<AgentOutput, AgentError> {
        info!(
            "Running agent {} for project {}",
            agent.name(),
            ctx.project_id
        );
        ctx.messages.push(Message {
            role: MessageRole::User,
            content: input.to_string(),
        });
        tokio::time::timeout(self.config.timeout, agent.execute(ctx, input))
            .await
            .map_err(|_| {
                AgentError(format!(
                    "Agent {} timed out after {:?}",
                    agent.name(),
                    self.config.timeout
                ))
            })?
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_unknown_tool_returns_error() {
        let exec = ToolExecutor::new(None, None);
        let call = ToolCall {
            id: "x".to_string(),
            name: "bad_tool".to_string(),
            input: serde_json::json!({}),
        };
        let r = exec.execute(&call).await;
        assert!(!r.success);
        assert!(r.content.contains("Unknown tool"));
    }

    #[tokio::test]
    async fn test_simulate_command() {
        let exec = ToolExecutor::new(None, None);
        let call = ToolCall {
            id: "y".to_string(),
            name: crate::tools::built_in::EXECUTE_COMMAND.to_string(),
            input: serde_json::json!({"command": "echo hi"}),
        };
        let r = exec.execute(&call).await;
        assert!(r.success);
        assert!(r.content.contains("SIMULATION"));
    }

    #[test]
    fn test_runner_config_defaults() {
        let c = RunnerConfig::default();
        assert_eq!(c.max_iterations, MAX_ITERATIONS);
        assert_eq!(c.timeout.as_secs(), DEFAULT_TIMEOUT_SECS);
        assert!(c.safety_gates_enabled);
    }

    #[test]
    fn test_runner_config_without_safety_gates() {
        let c = RunnerConfig::default().without_safety_gates();
        assert!(!c.safety_gates_enabled);
    }

    #[tokio::test]
    async fn test_gated_executor_read_bypasses_gates() {
        let exec = SafetyGatedExecutor::with_defaults(None, None);
        let call = ToolCall {
            id: "r".to_string(),
            name: crate::tools::built_in::READ_FILE.to_string(),
            input: serde_json::json!({"path": "/test"}),
        };
        let result = exec.execute(&call).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_gated_executor_disabled_bypasses_all() {
        let mut exec = SafetyGatedExecutor::with_defaults(None, None);
        exec.disable_gates();

        let call = ToolCall {
            id: "w".to_string(),
            name: crate::tools::built_in::WRITE_FILE.to_string(),
            input: serde_json::json!({"path": "/test", "content": "data"}),
        };
        let result = exec.execute(&call).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_gated_executor_action_type_detection() {
        let exec = SafetyGatedExecutor::with_defaults(None, None);

        let write_call = ToolCall {
            id: "1".to_string(),
            name: crate::tools::built_in::WRITE_FILE.to_string(),
            input: serde_json::json!({}),
        };
        assert_eq!(exec.get_action_type(&write_call), ActionType::FileWrite);

        let git_call = ToolCall {
            id: "2".to_string(),
            name: crate::tools::built_in::EXECUTE_COMMAND.to_string(),
            input: serde_json::json!({"command": "git push"}),
        };
        assert_eq!(exec.get_action_type(&git_call), ActionType::GitOperation);

        let rm_call = ToolCall {
            id: "3".to_string(),
            name: crate::tools::built_in::EXECUTE_COMMAND.to_string(),
            input: serde_json::json!({"command": "rm -rf /tmp/test"}),
        };
        assert_eq!(exec.get_action_type(&rm_call), ActionType::FileDelete);
    }

    #[tokio::test]
    async fn test_gated_executor_approve_reject() {
        let exec = SafetyGatedExecutor::new(None, None, GateConfig::strict());

        let call = ToolCall {
            id: "w".to_string(),
            name: crate::tools::built_in::WRITE_FILE.to_string(),
            input: serde_json::json!({"path": "/test", "content": "data"}),
        };

        let result = exec.execute(&call).await;
        let action_id = match result {
            Err(GatedExecutionError::RequiresApproval { action_id }) => action_id,
            _ => panic!("Expected RequiresApproval"),
        };

        let pending = exec.list_pending_actions().await;
        assert_eq!(pending.len(), 1);

        exec.reject_action(&action_id).await.expect("reject should succeed");

        let pending = exec.list_pending_actions().await;
        assert!(pending.is_empty());
    }
}
