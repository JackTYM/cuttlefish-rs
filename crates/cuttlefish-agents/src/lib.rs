#![deny(unsafe_code)]
#![warn(missing_docs)]
//! Agent system for Cuttlefish: message bus, tool registry, and runner.

/// Message bus implementation using tokio broadcast channels.
pub mod bus;
/// Coder agent that writes code, runs builds, and executes tests.
pub mod coder;
/// Critic agent that reviews code changes and provides structured feedback.
pub mod critic;
/// DevOps agent that handles builds, deployments, and infrastructure.
pub mod devops;
/// Explorer agent that searches codebases for code and patterns.
pub mod explorer;
/// Librarian agent that retrieves documentation for libraries and APIs.
pub mod librarian;
/// Orchestrator agent that plans and delegates work.
pub mod orchestrator;
/// Planner agent that creates detailed implementation plans.
pub mod planner;
/// Runtime prompt loading from YAML frontmatter markdown files.
pub mod prompt_registry;
/// Agent execution runner with tool calling and timeout enforcement.
pub mod runner;
/// Tool registry and built-in tool definitions.
pub mod tools;
/// Workflow engine: Orchestrator→Coder→Critic loop.
pub mod workflow;

pub use bus::TokioMessageBus;
pub use coder::CoderAgent;
pub use critic::{CriticAgent, ReviewResult, ReviewVerdict};
pub use devops::DevOpsAgent;
pub use explorer::ExplorerAgent;
pub use librarian::LibrarianAgent;
pub use orchestrator::OrchestratorAgent;
pub use planner::PlannerAgent;
pub use prompt_registry::{AgentPrompt, PromptError, PromptMetadata, PromptRegistry};
pub use runner::{
    AgentRunner, DEFAULT_TIMEOUT_SECS, MAX_ITERATIONS, RunnerConfig, ToolExecutionResult,
    ToolExecutor,
};
pub use tools::{ToolDefinition, ToolRegistry};
pub use workflow::{WorkflowConfig, WorkflowEngine, WorkflowResult};

pub use cuttlefish_core::traits::{
    agent::{Agent, AgentContext, AgentOutput, AgentRole, Category},
    bus::{BusMessage, MessageBus},
};

#[cfg(test)]
mod integration_tests;
