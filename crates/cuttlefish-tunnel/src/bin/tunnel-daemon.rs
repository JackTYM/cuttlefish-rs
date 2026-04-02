//! Tunnel daemon binary — runs the tunnel server.
//!
//! This binary starts the tunnel server that accepts WebSocket connections
//! from tunnel clients and routes HTTP requests to them.

use cuttlefish_tunnel::server::{TunnelServer, TunnelServerConfig};
use std::net::SocketAddr;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let jwt_secret = std::env::var("TUNNEL_JWT_SECRET")
        .expect("TUNNEL_JWT_SECRET environment variable must be set");

    let ws_port: u16 = std::env::var("TUNNEL_WS_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8082);

    let http_port: u16 = std::env::var("TUNNEL_HTTP_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8081);

    let config = TunnelServerConfig {
        listen_addr: SocketAddr::from(([0, 0, 0, 0], ws_port)),
        http_addr: SocketAddr::from(([0, 0, 0, 0], http_port)),
        jwt_secret: jwt_secret.into_bytes(),
        ..Default::default()
    };

    info!(
        ws_addr = %config.listen_addr,
        http_addr = %config.http_addr,
        "Starting tunnel daemon"
    );

    let server = TunnelServer::new(config);
    server.run().await?;

    Ok(())
}
