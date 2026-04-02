//! Agent execution runner with tool calling and timeout enforcement.

use cuttlefish_core::{
    error::AgentError,
    traits::{
        agent::{Agent, AgentContext, AgentOutput},
        provider::{Message, MessageRole, ToolCall},
        sandbox::{Sandbox, SandboxId},
    },
};
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, info, warn};

use crate::tools::ToolRegistry;

/// Maximum model↔tool iterations per invocation.
pub const MAX_ITERATIONS: usize = 25;
/// Default agent invocation timeout in seconds.
pub const DEFAULT_TIMEOUT_SECS: u64 = 300;

/// Configuration for the agent runner.
#[derive(Debug, Clone)]
pub struct RunnerConfig {
    /// Maximum iterations before forced stop.
    pub max_iterations: usize,
    /// Hard timeout for the entire invocation.
    pub timeout: Duration,
}

impl Default for RunnerConfig {
    fn default() -> Self {
        Self { max_iterations: MAX_ITERATIONS, timeout: Duration::from_secs(DEFAULT_TIMEOUT_SECS) }
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

/// Executes tool calls requested by the model.
pub struct ToolExecutor {
    sandbox: Option<Arc<dyn Sandbox>>,
    sandbox_id: Option<SandboxId>,
    _registry: ToolRegistry,
}

impl ToolExecutor {
    /// Create a new executor optionally backed by a Docker sandbox.
    pub fn new(sandbox: Option<Arc<dyn Sandbox>>, sandbox_id: Option<SandboxId>) -> Self {
        Self { sandbox, sandbox_id, _registry: ToolRegistry::with_defaults() }
    }

    /// Execute a tool call and return the result.
    pub async fn execute(&self, call: &ToolCall) -> ToolExecutionResult {
        debug!("Executing tool: {}", call.name);
        match call.name.as_str() {
            crate::tools::built_in::EXECUTE_COMMAND => {
                let cmd = call.input["command"].as_str().unwrap_or("");
                if let (Some(sb), Some(id)) = (&self.sandbox, &self.sandbox_id) {
                    match sb.exec(id, cmd).await {
                        Ok(out) => ToolExecutionResult { id: call.id.clone(), content: format!("stdout: {}\nstderr: {}\nexit: {}", out.stdout, out.stderr, out.exit_code), success: out.success() },
                        Err(e) => ToolExecutionResult { id: call.id.clone(), content: format!("Error: {e}"), success: false },
                    }
                } else {
                    ToolExecutionResult { id: call.id.clone(), content: format!("[SIMULATION] Would execute: {}", cmd), success: true }
                }
            }
            crate::tools::built_in::READ_FILE => {
                let path = call.input["path"].as_str().unwrap_or("");
                if let (Some(sb), Some(id)) = (&self.sandbox, &self.sandbox_id) {
                    match sb.read_file(id, std::path::Path::new(path)).await {
                        Ok(bytes) => ToolExecutionResult { id: call.id.clone(), content: String::from_utf8_lossy(&bytes).to_string(), success: true },
                        Err(e) => ToolExecutionResult { id: call.id.clone(), content: format!("Error: {e}"), success: false },
                    }
                } else {
                    ToolExecutionResult { id: call.id.clone(), content: format!("[SIMULATION] Would read: {}", path), success: true }
                }
            }
            crate::tools::built_in::WRITE_FILE => {
                let path = call.input["path"].as_str().unwrap_or("");
                let content = call.input["content"].as_str().unwrap_or("");
                if let (Some(sb), Some(id)) = (&self.sandbox, &self.sandbox_id) {
                    match sb.write_file(id, std::path::Path::new(path), content.as_bytes()).await {
                        Ok(()) => ToolExecutionResult { id: call.id.clone(), content: format!("Wrote {} bytes to {}", content.len(), path), success: true },
                        Err(e) => ToolExecutionResult { id: call.id.clone(), content: format!("Error: {e}"), success: false },
                    }
                } else {
                    ToolExecutionResult { id: call.id.clone(), content: format!("[SIMULATION] Would write {} bytes to {}", content.len(), path), success: true }
                }
            }
            crate::tools::built_in::LIST_DIRECTORY => {
                let path = call.input["path"].as_str().unwrap_or("/workspace");
                if let (Some(sb), Some(id)) = (&self.sandbox, &self.sandbox_id) {
                    match sb.list_files(id, std::path::Path::new(path)).await {
                        Ok(files) => ToolExecutionResult { id: call.id.clone(), content: files.join("\n"), success: true },
                        Err(e) => ToolExecutionResult { id: call.id.clone(), content: format!("Error: {e}"), success: false },
                    }
                } else {
                    ToolExecutionResult { id: call.id.clone(), content: format!("[SIMULATION] Would list: {}", path), success: true }
                }
            }
            crate::tools::built_in::SEARCH_FILES => {
                let pattern = call.input["pattern"].as_str().unwrap_or("*");
                ToolExecutionResult { id: call.id.clone(), content: format!("[SIMULATION] Would search for: {}", pattern), success: true }
            }
            crate::tools::built_in::EDIT_FILE => {
                let path = call.input["path"].as_str().unwrap_or("");
                let edits_json = call.input["edits"].as_array();
                
                if let (Some(sb), Some(id)) = (&self.sandbox, &self.sandbox_id) {
                    let content = match sb.read_file(id, std::path::Path::new(path)).await {
                        Ok(bytes) => String::from_utf8_lossy(&bytes).to_string(),
                        Err(e) => return ToolExecutionResult { id: call.id.clone(), content: format!("Error reading file: {e}"), success: false },
                    };
                    
                    let edits: Vec<cuttlefish_core::hashline::LineEdit> = match edits_json {
                        Some(arr) => arr.iter().map(|e| cuttlefish_core::hashline::LineEdit {
                            hash: e["hash"].as_str().unwrap_or("").to_string(),
                            expected_content: e["expected_content"].as_str().map(|s| s.to_string()),
                            new_content: e["new_content"].as_str().map(|s| s.to_string()),
                        }).collect(),
                        None => return ToolExecutionResult { id: call.id.clone(), content: "Error: 'edits' array required".to_string(), success: false },
                    };
                    
                    match cuttlefish_core::hashline::apply_edits(&content, &edits) {
                        Ok(new_content) => {
                            match sb.write_file(id, std::path::Path::new(path), new_content.as_bytes()).await {
                                Ok(()) => ToolExecutionResult { id: call.id.clone(), content: format!("Applied {} edit(s) to {}", edits.len(), path), success: true },
                                Err(e) => ToolExecutionResult { id: call.id.clone(), content: format!("Error writing file: {e}"), success: false },
                            }
                        }
                        Err(e) => ToolExecutionResult { id: call.id.clone(), content: format!("Edit error: {e}"), success: false },
                    }
                } else {
                    let edit_count = edits_json.map(|a| a.len()).unwrap_or(0);
                    ToolExecutionResult { id: call.id.clone(), content: format!("[SIMULATION] Would apply {} edit(s) to {}", edit_count, path), success: true }
                }
            }
            unknown => {
                warn!("Unknown tool: {}", unknown);
                ToolExecutionResult { id: call.id.clone(), content: format!("Error: Unknown tool '{}'", unknown), success: false }
            }
        }
    }
}

/// Drives an agent through its execution loop with timeout enforcement.
pub struct AgentRunner {
    /// Runner configuration.
    pub config: RunnerConfig,
    /// Tool executor for handling model tool calls.
    pub tool_executor: ToolExecutor,
}

impl AgentRunner {
    /// Create a runner with default config and optional sandbox.
    pub fn new(sandbox: Option<Arc<dyn Sandbox>>, sandbox_id: Option<SandboxId>) -> Self {
        Self { config: RunnerConfig::default(), tool_executor: ToolExecutor::new(sandbox, sandbox_id) }
    }

    /// Run an agent with timeout enforcement.
    pub async fn run(&self, agent: &dyn Agent, ctx: &mut AgentContext, input: &str) -> Result<AgentOutput, AgentError> {
        info!("Running agent {} for project {}", agent.name(), ctx.project_id);
        ctx.messages.push(Message { role: MessageRole::User, content: input.to_string() });
        tokio::time::timeout(self.config.timeout, agent.execute(ctx, input))
            .await
            .map_err(|_| AgentError(format!("Agent {} timed out after {:?}", agent.name(), self.config.timeout)))?
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_unknown_tool_returns_error() {
        let exec = ToolExecutor::new(None, None);
        let call = ToolCall { id: "x".to_string(), name: "bad_tool".to_string(), input: serde_json::json!({}) };
        let r = exec.execute(&call).await;
        assert!(!r.success);
        assert!(r.content.contains("Unknown tool"));
    }

    #[tokio::test]
    async fn test_simulate_command() {
        let exec = ToolExecutor::new(None, None);
        let call = ToolCall { id: "y".to_string(), name: crate::tools::built_in::EXECUTE_COMMAND.to_string(), input: serde_json::json!({"command": "echo hi"}) };
        let r = exec.execute(&call).await;
        assert!(r.success);
        assert!(r.content.contains("SIMULATION"));
    }

    #[test]
    fn test_runner_config_defaults() {
        let c = RunnerConfig::default();
        assert_eq!(c.max_iterations, MAX_ITERATIONS);
        assert_eq!(c.timeout.as_secs(), DEFAULT_TIMEOUT_SECS);
    }
}
