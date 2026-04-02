//! Cuttlefish terminal user interface.
//!
//! A lightweight TUI client that connects to a remote Cuttlefish server
//! via WebSocket for real-time chat, build logs, and file diffs.

mod app;
mod mascot;
mod ui;

use clap::Parser;

/// Cuttlefish TUI client.
#[derive(Parser)]
#[command(name = "cuttlefish-tui", version, about)]
struct Cli {
    /// WebSocket server URL (e.g., ws://localhost:8080).
    #[arg(long, default_value = "ws://localhost:8080")]
    server: String,

    /// API key for authentication.
    #[arg(long, env = "CUTTLEFISH_API_KEY")]
    api_key: Option<String>,
}

fn main() {
    let cli = Cli::parse();
    tracing::subscriber::set_global_default(tracing_subscriber::fmt().compact().finish())
        .expect("failed to set tracing subscriber");

    tracing::info!("Cuttlefish TUI starting, connecting to {}", cli.server);
    tracing::info!("Note: full TUI UI requires a running Cuttlefish server");
    tracing::info!("Press Ctrl+C to exit");
}
