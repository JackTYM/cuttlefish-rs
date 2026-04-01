//! Agent trait and supporting types.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::AgentError;

/// Result type for agent operations.
pub type AgentResult<T> = Result<T, AgentError>;

/// Roles an agent can fulfill.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AgentRole {
    /// Orchestrates tasks and delegates to other agents.
    Orchestrator,
    /// Creates strategic plans.
    Planner,
    /// Writes and modifies code.
    Coder,
    /// Reviews code and provides feedback.
    Critic,
    /// Searches codebases and retrieves information.
    Explorer,
    /// Finds documentation and external resources.
    Librarian,
    /// Handles builds, deployments, and infrastructure.
    DevOps,
}

/// Model/task category for routing decisions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Category {
    /// Frontend, UI/UX, visual work.
    Visual,
    /// Deep research and autonomous execution.
    Deep,
    /// Simple, quick tasks.
    Quick,
    /// Hard logic and architecture decisions.
    UltraBrain,
    /// General tasks, lower effort.
    UnspecifiedLow,
    /// General tasks, higher effort.
    UnspecifiedHigh,
}

/// Context passed to every agent invocation.
pub struct AgentContext {
    /// Unique ID for this agent invocation.
    pub invocation_id: Uuid,
    /// Project ID this agent is working on.
    pub project_id: Uuid,
    /// Current working directory inside the sandbox.
    pub working_dir: std::path::PathBuf,
    /// Tool registry access (names of available tools).
    pub available_tools: Vec<String>,
    /// Conversation history for this invocation.
    pub messages: Vec<crate::traits::provider::Message>,
}

/// Output from an agent invocation.
#[derive(Debug, Clone)]
pub struct AgentOutput {
    /// Final text output.
    pub content: String,
    /// Files modified during this invocation.
    pub files_changed: Vec<std::path::PathBuf>,
    /// Commands that were executed.
    pub commands_run: Vec<String>,
    /// Whether the task was completed successfully.
    pub success: bool,
    /// Optional structured metadata.
    pub metadata: serde_json::Value,
}

/// An agent that can execute tasks.
#[async_trait]
pub trait Agent: Send + Sync {
    /// Agent display name.
    fn name(&self) -> &str;

    /// Agent's role.
    fn role(&self) -> AgentRole;

    /// Execute a task with the given context and input.
    async fn execute(&self, ctx: &mut AgentContext, input: &str) -> AgentResult<AgentOutput>;
}
