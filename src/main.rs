//! Cuttlefish server — multi-agent, multi-model agentic coding platform.
//!
//! Entry point that wires together all crates and starts the HTTP/WebSocket server.

use cuttlefish_api::{build_app, routes::AppState};
use cuttlefish_core::config::CuttlefishConfig;
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize structured logging first
    cuttlefish_core::tracing::init_tracing();

    // Load configuration (falls back gracefully if no config file)
    let config = CuttlefishConfig::load().unwrap_or_else(|_| {
        // Default config if no file found — useful for first-time run
        tracing::warn!(
            "No cuttlefish.toml found, using defaults. Copy cuttlefish.example.toml to get started."
        );
        CuttlefishConfig {
            server: cuttlefish_core::config::ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 8080,
                api_key: std::env::var("CUTTLEFISH_API_KEY").ok(),
            },
            database: cuttlefish_core::config::DatabaseConfig {
                path: PathBuf::from("cuttlefish.db"),
            },
            providers: HashMap::new(),
            agents: HashMap::new(),
            discord: None,
            sandbox: cuttlefish_core::config::SandboxConfig::default(),
        }
    });

    let api_key = config
        .server
        .api_key
        .or_else(|| std::env::var("CUTTLEFISH_API_KEY").ok())
        .unwrap_or_else(|| {
            tracing::warn!("No API key configured. Set CUTTLEFISH_API_KEY or add api_key to [server] in cuttlefish.toml");
            "changeme".to_string()
        });

    let addr = format!("{}:{}", config.server.host, config.server.port);

    info!("🐙 Cuttlefish starting on http://{}", addr);
    info!("WebSocket endpoint: ws://{}/ws", addr);
    info!("Health check: http://{}/health", addr);

    let state = AppState { api_key };
    let app = build_app(state);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    info!("Server listening on {}", addr);

    let serve_future = axum::serve(listener, app).with_graceful_shutdown(shutdown_signal());
    serve_future.await?;

    info!("Cuttlefish server stopped gracefully");
    Ok(())
}

/// Wait for Ctrl+C signal for graceful shutdown.
async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to install CTRL+C signal handler");
    tracing::info!("Shutdown signal received");
}
