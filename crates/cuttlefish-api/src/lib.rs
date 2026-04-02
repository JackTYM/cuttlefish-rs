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
//! - Authentication middleware

/// REST API route handlers.
pub mod api_routes;
/// API key authentication middleware.
pub mod auth;
/// Reverse proxy route registry.
pub mod proxy;
/// HTTP route handlers.
pub mod routes;
/// WebSocket handler and message protocol.
pub mod ws;

use axum::{
    Router,
    routing::{any, get, post},
};
use tower_http::{cors::CorsLayer, trace::TraceLayer};

pub use auth::{auth_middleware, generate_api_key};
pub use proxy::{ProxyRegistry, ProxyRoute};
pub use routes::AppState;
pub use ws::{ClientMessage, ServerMessage};

/// Build the axum application router with all routes.
pub fn build_app(state: AppState) -> Router {
    Router::new()
        .route("/health", get(routes::health_handler))
        .route("/ws", any(ws::ws_handler))
        .route("/api/templates", get(api_routes::list_templates))
        .route("/api/templates/{name}", get(api_routes::get_template))
        .route("/api/templates/fetch", post(api_routes::fetch_template))
        .fallback(routes::not_found_handler)
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(state)
}
