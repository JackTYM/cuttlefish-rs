#![deny(unsafe_code)]
#![warn(missing_docs)]
//! Sandbox implementations for Cuttlefish.
//!
//! Provides isolated execution environments for agent code execution.
//! The Docker sandbox uses one container per project context.

/// Cleanup and garbage collection for sandbox resources.
pub mod cleanup;
/// Docker-based sandbox implementation.
pub mod docker;
/// Health checking for the sandbox system.
pub mod health;
/// Docker image registry for template-specific base images.
pub mod images;
/// Container lifecycle management for Docker sandboxes.
pub mod lifecycle;
/// Resource usage monitoring and enforcement.
pub mod resources;
/// Container snapshot management using Docker image commits.
pub mod snapshots;
/// Docker volume management for persistent storage.
pub mod volumes;

pub use cleanup::DockerCleanupManager;
pub use cuttlefish_core::traits::sandbox::{
    ExecOutput, ImageRegistry, Sandbox, SandboxConfig, SandboxId,
};
pub use docker::DockerSandbox;
pub use health::DockerHealthChecker;
pub use images::{DockerImageRegistry, TemplateImageMap};
pub use lifecycle::DockerSandboxLifecycle;
pub use resources::{ResourceEnforcer, ResourceUsage};
pub use snapshots::DockerSnapshotManager;
pub use volumes::DockerVolumeManager;
