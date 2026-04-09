#![deny(unsafe_code)]
#![warn(missing_docs)]
//! HTTP/WebSocket API server for Cuttlefish.
//!
//! Provides:
//! - `GET /health` — health check
//! - `GET /ws` — WebSocket upgrade endpoint
//! - `GET /api/templates` — list all templates
//! - `GET /api/templates/:name` — get template details
//! - `POST /api/templates/fetch` — fetch remote template
//! - Authentication middleware and routes
//! - Safety API (checkpoints, approvals, diff preview)

/// REST API route handlers.
pub mod api_routes;
/// Pending approval registry for safety workflow integration.
pub mod approval_registry;
/// API key authentication middleware (legacy).
pub mod auth;
/// Authentication API routes (registration, login, tokens, API keys).
pub mod auth_routes;
/// Collaboration API routes (sharing, invites, handoffs, activity).
pub mod collaboration_routes;
/// Embedded WebUI serving (compiled into binary).
pub mod embedded_webui;
/// Memory, decisions, and branching API endpoints.
pub mod memory_routes;
/// Authentication middleware (JWT, API key validation).
pub mod middleware;
/// Organization API routes (CRUD, members, config, API keys).
pub mod organization_routes;
/// Reverse proxy route registry.
pub mod proxy;
/// HTTP route handlers.
pub mod routes;
/// Safety API routes (checkpoints, approvals, diff preview).
pub mod safety_routes;
/// Sandbox management API endpoints.
pub mod sandbox_routes;
/// System configuration and status API.
pub mod system_routes;
/// Usage tracking and billing API endpoints.
pub mod usage_routes;
/// WebUI static file serving (file-based fallback).
pub mod webui;
/// WebSocket handler and message protocol.
pub mod ws;

use std::path::PathBuf;
use std::sync::Arc;

use axum::{
    Router,
    routing::{any, get, post},
};
use cuttlefish_core::PricingConfig;
use sqlx::SqlitePool;
use tower_http::{cors::CorsLayer, trace::TraceLayer};

pub use approval_registry::{
    ApprovalDecision, ApprovalRegistry, DEFAULT_APPROVAL_TIMEOUT_SECS, PendingApproval,
    SharedApprovalRegistry, create_approval_registry, create_approval_registry_with_db,
};
pub use auth::{auth_middleware, generate_api_key};
pub use auth_routes::AuthState;
pub use collaboration_routes::{CollaborationState, collaboration_router};
pub use memory_routes::{MemoryState, memory_router};
pub use middleware::AuthConfig;
pub use organization_routes::{OrganizationState, organization_router};
pub use proxy::{ProxyRegistry, ProxyRoute};
pub use routes::{AppState, ProjectSession};
pub use safety_routes::{
    SafetyState, is_action_approved, queue_pending_action, safety_router, wait_for_approval,
};
pub use system_routes::{SystemState, system_router};
pub use usage_routes::{UsageState, usage_router};
pub use webui::{WebUiConfig, WebUiState, webui_router};
pub use ws::{ClientMessage, RiskFactor, ServerMessage, execute_with_safety};

/// Configuration for building the full API application.
pub struct ApiConfig {
    /// Core application state.
    pub app_state: AppState,
    /// Database pool for sub-routers.
    pub pool: Arc<SqlitePool>,
    /// Auth configuration for protected routes.
    pub auth_config: AuthConfig,
    /// Base directory for projects.
    pub projects_dir: PathBuf,
    /// Pricing configuration for usage tracking.
    pub pricing_config: PricingConfig,
    /// Provider registry for model providers (optional, for testing).
    pub provider_registry: Option<Arc<cuttlefish_providers::ProviderRegistry>>,
}

/// Build the axum application router with all routes.
pub fn build_app(state: AppState) -> Router {
    Router::new()
        .route("/health", get(routes::health_handler))
        .route("/ws", any(ws::ws_handler))
        .route("/api/templates", get(api_routes::list_templates))
        .route("/api/templates/{name}", get(api_routes::get_template))
        .route("/api/templates/fetch", post(api_routes::fetch_template))
        .route(
            "/api/projects",
            get(api_routes::list_projects).post(api_routes::create_project),
        )
        .route(
            "/api/projects/{id}",
            get(api_routes::get_project).delete(api_routes::cancel_project),
        )
        .route(
            "/api/projects/{id}/archive",
            post(api_routes::archive_project),
        )
        .fallback(routes::not_found_handler)
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(state)
}

/// Build the full API application with all modular routers merged.
pub fn build_full_app(config: ApiConfig) -> Router {
    let usage_state = UsageState::new(config.pool.clone(), config.pricing_config.clone());
    let safety_state = SafetyState::new(&config.projects_dir, config.auth_config.clone());
    let memory_state = MemoryState::new(&config.projects_dir);
    let collab_state = CollaborationState::new((*config.pool).clone(), config.auth_config.clone());
    let org_state = OrganizationState::new((*config.pool).clone(), config.auth_config.clone());
    let auth_state = AuthState::new((*config.pool).clone(), config.auth_config.clone());
    let mut system_state = SystemState::new(config.auth_config.clone());
    if let Some(registry) = config.provider_registry.clone() {
        system_state = system_state.with_provider_registry(registry);
    }

    // Build usage router with auth middleware
    let usage_router_with_auth =
        usage_router()
            .with_state(usage_state)
            .layer(axum::middleware::from_fn_with_state(
                config.auth_config.clone(),
                middleware::optional_auth,
            ));

    let router = Router::new()
        .route("/health", get(routes::health_handler))
        .route("/ws", any(ws::ws_handler))
        .route("/api/templates", get(api_routes::list_templates))
        .route("/api/templates/{name}", get(api_routes::get_template))
        .route("/api/templates/fetch", post(api_routes::fetch_template))
        .route(
            "/api/projects",
            get(api_routes::list_projects).post(api_routes::create_project),
        )
        .route(
            "/api/projects/{id}",
            get(api_routes::get_project).delete(api_routes::cancel_project),
        )
        .route(
            "/api/projects/{id}/archive",
            post(api_routes::archive_project),
        )
        .with_state(config.app_state)
        .merge(usage_router_with_auth)
        .merge(safety_router(safety_state))
        .merge(memory_router().with_state(memory_state))
        .merge(system_router(system_state))
        .nest("/api", collaboration_router(collab_state))
        .nest("/api", organization_router(org_state))
        .nest("/api/auth", auth_routes::auth_router(auth_state))
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive());

    // Try embedded WebUI as fallback (for single-binary deployment)
    if let Some(embedded_router) = embedded_webui::embedded_webui_router() {
        tracing::info!("Using embedded WebUI (compiled into binary)");
        router.merge(embedded_router)
    } else {
        router.fallback(routes::not_found_handler)
    }
}

/// Build the axum application router with WebUI static file serving.
pub fn build_app_with_webui(state: AppState, webui_config: WebUiConfig) -> Router {
    use tower_http::services::ServeDir;

    let api_router = Router::new()
        .route("/health", get(routes::health_handler))
        .route("/ws", any(ws::ws_handler))
        .route("/api/templates", get(api_routes::list_templates))
        .route("/api/templates/{name}", get(api_routes::get_template))
        .route("/api/templates/fetch", post(api_routes::fetch_template))
        .route(
            "/api/projects",
            get(api_routes::list_projects).post(api_routes::create_project),
        )
        .route(
            "/api/projects/{id}",
            get(api_routes::get_project).delete(api_routes::cancel_project),
        )
        .route(
            "/api/projects/{id}/archive",
            post(api_routes::archive_project),
        )
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(state);

    // Priority 1: Try embedded WebUI (compiled into binary)
    if let Some(embedded_router) = embedded_webui::embedded_webui_router() {
        tracing::info!("Using embedded WebUI (compiled into binary)");
        return api_router.merge(embedded_router);
    }

    // Priority 2: Try file-based WebUI
    if webui_config.enabled && webui_config.is_valid() {
        tracing::info!(
            static_dir = %webui_config.static_dir.display(),
            "Using file-based WebUI"
        );

        let index_html = webui_config.static_dir.join("index.html");
        let serve_dir = ServeDir::new(&webui_config.static_dir)
            .not_found_service(tower_http::services::ServeFile::new(&index_html));

        return api_router.fallback_service(serve_dir);
    }

    // No WebUI available
    if webui_config.enabled {
        tracing::warn!(
            static_dir = %webui_config.static_dir.display(),
            "WebUI requested but neither embedded nor file-based available"
        );
    }
    api_router.fallback(routes::not_found_handler)
}

/// Build the full API application with WebUI and all modular routers.
pub fn build_full_app_with_webui(config: ApiConfig, webui_config: WebUiConfig) -> Router {
    use tower_http::services::ServeDir;

    let usage_state = UsageState::new(config.pool.clone(), config.pricing_config.clone());
    let safety_state = SafetyState::new(&config.projects_dir, config.auth_config.clone());
    let memory_state = MemoryState::new(&config.projects_dir);
    let collab_state = CollaborationState::new((*config.pool).clone(), config.auth_config.clone());
    let org_state = OrganizationState::new((*config.pool).clone(), config.auth_config.clone());
    let auth_state = AuthState::new((*config.pool).clone(), config.auth_config.clone());
    let mut system_state = SystemState::new(config.auth_config.clone());
    if let Some(registry) = config.provider_registry.clone() {
        system_state = system_state.with_provider_registry(registry);
    }

    // Build usage router with auth middleware
    let usage_router_with_auth =
        usage_router()
            .with_state(usage_state)
            .layer(axum::middleware::from_fn_with_state(
                config.auth_config.clone(),
                middleware::optional_auth,
            ));

    let api_router = Router::new()
        .route("/health", get(routes::health_handler))
        .route("/ws", any(ws::ws_handler))
        .route("/api/templates", get(api_routes::list_templates))
        .route("/api/templates/{name}", get(api_routes::get_template))
        .route("/api/templates/fetch", post(api_routes::fetch_template))
        .route(
            "/api/projects",
            get(api_routes::list_projects).post(api_routes::create_project),
        )
        .route(
            "/api/projects/{id}",
            get(api_routes::get_project).delete(api_routes::cancel_project),
        )
        .route(
            "/api/projects/{id}/archive",
            post(api_routes::archive_project),
        )
        .with_state(config.app_state)
        .merge(usage_router_with_auth)
        .merge(safety_router(safety_state))
        .merge(memory_router().with_state(memory_state))
        .merge(system_router(system_state))
        .nest("/api", collaboration_router(collab_state))
        .nest("/api", organization_router(org_state))
        .nest("/api/auth", auth_routes::auth_router(auth_state))
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive());

    // Priority 1: Try embedded WebUI (compiled into binary)
    if let Some(embedded_router) = embedded_webui::embedded_webui_router() {
        tracing::info!("Using embedded WebUI (compiled into binary)");
        return api_router.merge(embedded_router);
    }

    // Priority 2: Try file-based WebUI
    if webui_config.enabled && webui_config.is_valid() {
        tracing::info!(
            static_dir = %webui_config.static_dir.display(),
            "Using file-based WebUI"
        );

        let index_html = webui_config.static_dir.join("index.html");
        let serve_dir = ServeDir::new(&webui_config.static_dir)
            .not_found_service(tower_http::services::ServeFile::new(&index_html));

        return api_router.fallback_service(serve_dir);
    }

    // No WebUI available
    if webui_config.enabled {
        tracing::warn!(
            static_dir = %webui_config.static_dir.display(),
            "WebUI requested but neither embedded nor file-based available"
        );
    }
    api_router.fallback(routes::not_found_handler)
}
