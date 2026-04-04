#![deny(unsafe_code)]
#![warn(missing_docs)]

//! Core infrastructure for the Cuttlefish platform: errors, config, traits, and tracing.

pub mod advanced;
pub mod alerts;
pub mod auth;
pub mod config;
pub mod context;
pub mod costs;
pub mod error;
pub mod hashline;
pub mod pricing;
pub mod routing;
pub mod stats;
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
pub use alerts::{AlertChecker, AlertError, AlertNotifier, LogNotifier, NoopNotifier, TriggeredAlert};
pub use auth::{
    Action, ApiKeyScope, AuthError, CreateUserRequest, GeneratedApiKey, GeneratedResetToken,
    RefreshResult, Role, RoleError, SessionError, SessionInfo, SessionMetadata, TokenClaims,
    TokenPair, TokenType, User, UserId, can_perform, generate_access_token, generate_api_key,
    generate_refresh_token, generate_refresh_token_value, generate_reset_token, has_required_scope,
    hash_api_key, hash_password, hash_refresh_token, hash_reset_token, validate_api_key_format,
    validate_password_strength, validate_token, verify_password, verify_refresh_token,
    verify_reset_token,
};
pub use config::CuttlefishConfig;
pub use context::{ContextConfig, ContextManager};
pub use costs::{CostBreakdown, CostCalculator, CostError};
pub use error::CuttlefishError;
pub use hashline::{
    EditError, HashedLine, LineEdit, apply_edits, format_with_hashes, hash_file_lines, line_hash,
};
pub use pricing::{ModelPrice, PricingConfig, PricingSection};
pub use routing::{ModelRouter, RouteConfig, RoutingConfig};
pub use stats::{ProjectUsageSummary, StatsError, TimePeriod, UsageStats, UserUsageSummary};
pub use template_engine::TemplateEngine;
pub use template_fetcher::TemplateFetcher;
pub use template_manifest::{TemplateError, TemplateManifest, TemplateVariable, parse_manifest};
pub use template_registry::{LoadedTemplate, TemplateRegistry, TemplateSource};
