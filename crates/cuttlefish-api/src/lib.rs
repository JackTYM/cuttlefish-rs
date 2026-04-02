#![deny(unsafe_code)]
#![warn(missing_docs)]
//! HTTP/WebSocket API server for Cuttlefish.
//!
//! Provides:
//! - `GET /health` — health check
//! - `GET /ws` — WebSocket upgrade endpoint
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
    routing::{any, get},
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
        .fallback(routes::not_found_handler)
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(state)
}
