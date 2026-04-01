#![deny(unsafe_code)]
#![warn(missing_docs)]
//! Sandbox implementations for Cuttlefish.
//!
//! Provides isolated execution environments for agent code execution.
//! The Docker sandbox uses one container per project context.

/// Docker-based sandbox implementation.
pub mod docker;
/// Docker image registry for template-specific base images.
pub mod images;

pub use cuttlefish_core::traits::sandbox::{ExecOutput, Sandbox, SandboxConfig, SandboxId};
pub use docker::DockerSandbox;
pub use images::ImageRegistry;
