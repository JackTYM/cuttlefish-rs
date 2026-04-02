#![deny(unsafe_code)]
#![warn(missing_docs)]

//! Core infrastructure for the Cuttlefish platform: errors, config, traits, and tracing.

pub mod advanced;
pub mod config;
pub mod context;
pub mod error;
pub mod traits;
pub mod tracing;

pub use advanced::{
    find_template, is_newer_version, load_templates_from_dir, AgentRoutingConfig, GithubAppClaims,
    ModelConfig, ProjectTemplate, ReleaseAsset, ReleaseInfo, WorkflowStatus,
};
pub use config::CuttlefishConfig;
pub use context::{ContextConfig, ContextManager};
pub use error::CuttlefishError;
