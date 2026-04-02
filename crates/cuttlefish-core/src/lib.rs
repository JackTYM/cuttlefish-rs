#![deny(unsafe_code)]
#![warn(missing_docs)]

//! Core infrastructure for the Cuttlefish platform: errors, config, traits, and tracing.

pub mod advanced;
pub mod config;
pub mod context;
pub mod error;
pub mod hashline;
pub mod routing;
pub mod template_engine;
pub mod template_fetcher;
pub mod template_manifest;
pub mod template_registry;
pub mod tracing;
pub mod traits;

pub use advanced::{
    AgentRoutingConfig, GithubAppClaims, ModelConfig, ProjectTemplate, ReleaseAsset, ReleaseInfo,
    WorkflowStatus, find_template, is_newer_version, load_templates_from_dir,
};
pub use config::CuttlefishConfig;
pub use context::{ContextConfig, ContextManager};
pub use error::CuttlefishError;
pub use hashline::{
    EditError, HashedLine, LineEdit, apply_edits, format_with_hashes, hash_file_lines, line_hash,
};
pub use routing::{ModelRouter, RouteConfig, RoutingConfig};
pub use template_engine::TemplateEngine;
pub use template_fetcher::TemplateFetcher;
pub use template_manifest::{TemplateError, TemplateManifest, TemplateVariable, parse_manifest};
pub use template_registry::{LoadedTemplate, TemplateRegistry, TemplateSource};
