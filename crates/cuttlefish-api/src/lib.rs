#![deny(unsafe_code)]
#![warn(missing_docs)]
//! HTTP/WebSocket API server for Cuttlefish.
//!
//! Provides:
//! - `GET /health` — health check
//! - `GET /ws` — WebSocket upgrade endpoint
//! - Authentication middleware

/// WebSocket handler and message protocol.
pub mod ws;
/// HTTP route handlers.
pub mod routes;
/// API key authentication middleware.
pub mod auth;
/// REST API route handlers.
pub mod api_routes;
/// Reverse proxy route registry.
pub mod proxy;

use axum::{
    routing::{any, get},
    Router,
};
use tower_http::{cors::CorsLayer, trace::TraceLayer};

pub use routes::AppState;
pub use ws::{ClientMessage, ServerMessage};
pub use auth::{auth_middleware, generate_api_key};
pub use proxy::{ProxyRegistry, ProxyRoute};

/// Build the axum application router with all routes.
pub fn build_app(state: AppState) -> Router {
    Router::new()
        .route("/health", get(routes::health_handler))
        .route("/ws", any(ws::ws_handler))
        .fallback(routes::not_found_handler)
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(state)
}
