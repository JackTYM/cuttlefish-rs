#![deny(unsafe_code)]
#![warn(missing_docs)]
//! Agent system for Cuttlefish: message bus, tool registry, and runner.

/// Message bus implementation using tokio broadcast channels.
pub mod bus;
/// Coder agent that writes code, runs builds, and executes tests.
pub mod coder;
/// Context management: token counting and automatic compaction.
pub mod context;
/// Critic agent that reviews code changes and provides structured feedback.
pub mod critic;
/// DevOps agent that handles builds, deployments, and infrastructure.
pub mod devops;
/// Explorer agent that searches codebases for code and patterns.
pub mod explorer;
/// Librarian agent that retrieves documentation for libraries and APIs.
pub mod librarian;
/// Agent memory system for persistent project context.
pub mod memory;
/// Orchestrator agent that plans and delegates work.
pub mod orchestrator;
/// Planner agent that creates detailed implementation plans.
pub mod planner;
/// Runtime prompt loading from YAML frontmatter markdown files.
pub mod prompt_registry;
/// Prompt template engine with placeholder replacement and section overrides.
pub mod prompt_template;
/// Agent execution runner with tool calling and timeout enforcement.
pub mod runner;
/// Safety system: confidence scoring, action gates, and diff generation.
pub mod safety;
/// Session persistence: database storage and crash recovery journaling.
pub mod session;
/// Tool registry and built-in tool definitions.
pub mod tools;
/// Workflow engine: Orchestrator→Coder→Critic loop.
pub mod workflow;

pub use bus::TokioMessageBus;
pub use coder::CoderAgent;
pub use context::{
    CompactionConfig, CompactionResult, ContextCompactor, MessageCategory, TokenCounter,
};
pub use critic::{CriticAgent, ReviewResult, ReviewVerdict};
pub use devops::DevOpsAgent;
pub use explorer::ExplorerAgent;
pub use librarian::LibrarianAgent;
pub use memory::{
    BranchDiff, BranchError, BranchId, BranchStore, ChangeType, ConversationExcerpt, DecisionEntry,
    DecisionIndex, DecisionLog, ExcerptMessage, GitDiffSummary, MAX_BRANCHES_PER_PROJECT,
    MemoryHooks, MemorySection, MemoryTrigger, ProjectMemory, StateBranch, UpdateEvent,
    WhyExplanation, WhyTarget, get_conversation_excerpt, get_excerpts_for_decisions,
    redact_sensitive, why,
};
pub use orchestrator::OrchestratorAgent;
pub use planner::PlannerAgent;
pub use prompt_registry::{AgentPrompt, PromptError, PromptMetadata, PromptRegistry};
pub use prompt_template::{PromptContext, PromptTemplate, load_system_template};
pub use runner::{
    AgentRunner, DEFAULT_TIMEOUT_SECS, MAX_ITERATIONS, RunnerConfig, ToolExecutionResult,
    ToolExecutor,
};
pub use safety::{
    ActionGate, ActionPreview, ActionType, ConfidenceCalculator, ConfidenceFactor, ConfidenceScore,
    DiffHunk, DiffLine, DiffStats, FileDiff, GateConfig, GateDecision, MAX_DIFF_FILE_SIZE,
    QuickDecision, RiskFactor, ThresholdOverride, detect_language,
};
pub use session::{
    ConversationPersistence, JournalEntry, PersistenceConfig, RestoreResult, SessionJournal,
};
pub use tools::{ToolDefinition, ToolRegistry};
pub use workflow::{WorkflowConfig, WorkflowEngine, WorkflowResult};

pub use cuttlefish_core::traits::{
    agent::{Agent, AgentContext, AgentOutput, AgentRole, Category},
    bus::{BusMessage, MessageBus},
};

#[cfg(test)]
mod integration_tests;
