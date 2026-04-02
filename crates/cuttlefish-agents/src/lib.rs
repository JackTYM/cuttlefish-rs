#![deny(unsafe_code)]
#![warn(missing_docs)]
//! Agent system for Cuttlefish: message bus, tool registry, and runner.

/// Message bus implementation using tokio broadcast channels.
pub mod bus;
/// Agent execution runner with tool calling and timeout enforcement.
pub mod runner;
/// Tool registry and built-in tool definitions.
pub mod tools;
/// Orchestrator agent that plans and delegates work.
pub mod orchestrator;
/// Coder agent that writes code, runs builds, and executes tests.
pub mod coder;
/// Critic agent that reviews code changes and provides structured feedback.
pub mod critic;
/// Workflow engine: Orchestrator→Coder→Critic loop.
pub mod workflow;

pub use bus::TokioMessageBus;
pub use runner::{AgentRunner, RunnerConfig, ToolExecutionResult, ToolExecutor, MAX_ITERATIONS, DEFAULT_TIMEOUT_SECS};
pub use tools::{ToolDefinition, ToolRegistry};
pub use orchestrator::OrchestratorAgent;
pub use coder::CoderAgent;
pub use critic::{CriticAgent, ReviewResult, ReviewVerdict};
pub use workflow::{WorkflowEngine, WorkflowResult};

pub use cuttlefish_core::traits::{
    agent::{Agent, AgentContext, AgentOutput, AgentRole, Category},
    bus::{BusMessage, MessageBus},
};

#[cfg(test)]
mod integration_tests;
