//! Core traits for the Cuttlefish platform.

pub mod agent;
pub mod bus;
pub mod provider;
pub mod sandbox;
pub mod vcs;

pub use agent::{Agent, AgentContext, AgentOutput, AgentRole, Category};
pub use bus::{BusMessage, MessageBus};
pub use provider::{
    CompletionRequest, CompletionResponse, Message, MessageRole, ModelProvider, StreamChunk,
    ToolCall,
};
pub use sandbox::{ExecOutput, Sandbox, SandboxConfig, SandboxId};
pub use vcs::{CommitInfo, VersionControl};
