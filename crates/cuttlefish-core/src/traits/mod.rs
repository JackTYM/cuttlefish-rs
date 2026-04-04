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
pub use sandbox::{
    CleanupManager, CleanupPolicy, CleanupResult, ContainerConfig, ContainerStatus, ExecOutput,
    ExecutionResult, HealthChecker, ImageBuildOptions, ImageRegistry, ImageSpec, Language,
    ResourceLimits, ResourceLimitsBuilder, Sandbox, SandboxConfig, SandboxHandle, SandboxHealth,
    SandboxId, SandboxLifecycle, SandboxUsage, Snapshot, SnapshotManager, SnapshotOptions,
    VolumeHandle, VolumeManager, VolumeMount,
};
pub use vcs::{CommitInfo, VersionControl};
