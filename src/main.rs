//! Cuttlefish server — multi-agent, multi-model agentic coding platform.
//!
//! Entry point that wires together all crates and starts the HTTP/WebSocket server.

use cuttlefish_api::{build_app, routes::AppState};
use cuttlefish_core::config::CuttlefishConfig;
use cuttlefish_core::TemplateRegistry;
use cuttlefish_tunnel::client::{TunnelClient, TunnelClientConfig};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize structured logging first
    cuttlefish_core::tracing::init_tracing();

    // Check for CLI commands
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        match args[1].as_str() {
            "validate-templates" => {
                let dir = args
                    .get(2)
                    .map(PathBuf::from)
                    .unwrap_or_else(|| PathBuf::from("templates"));
                return validate_templates(&dir);
            }
            "tunnel" => {
                if args.len() < 3 {
                    eprintln!("Usage: cuttlefish tunnel <connect|status|disconnect>");
                    std::process::exit(1);
                }
                return match args[2].as_str() {
                    "connect" => tunnel_connect(&args[3..]).await,
                    "status" => tunnel_status().await,
                    "disconnect" => tunnel_disconnect().await,
                    _ => {
                        eprintln!("Unknown tunnel command: {}", args[2]);
                        std::process::exit(1);
                    }
                };
            }
            _ => {}
        }
    }

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
            routing: cuttlefish_core::RoutingConfig::default(),
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

    let template_registry = Arc::new(TemplateRegistry::new());
    let state = AppState {
        api_key,
        template_registry,
    };
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

/// Validate template files in a directory.
fn validate_templates(dir: &PathBuf) -> anyhow::Result<()> {
    println!("Validating templates in {:?}...", dir);

    let registry = TemplateRegistry::new();

    match registry.load_from_dir(dir) {
        Ok(count) => {
            println!("✓ Successfully loaded {} templates", count);
            let templates = registry.list();
            for template in templates {
                println!(
                    "  - {} ({})",
                    template.manifest.name, template.manifest.language
                );
            }
            std::process::exit(0);
        }
        Err(e) => {
            eprintln!("✗ Validation failed: {}", e);
            std::process::exit(1);
        }
    }
}

/// Get the Cuttlefish config directory
fn get_config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("cuttlefish")
}

/// Save JWT to config directory
fn save_jwt(jwt: &str) -> anyhow::Result<()> {
    let dir = get_config_dir();
    std::fs::create_dir_all(&dir)?;
    std::fs::write(dir.join("tunnel.jwt"), jwt)?;
    Ok(())
}

/// Load saved JWT from config directory
fn load_saved_jwt() -> anyhow::Result<String> {
    let path = get_config_dir().join("tunnel.jwt");
    Ok(std::fs::read_to_string(path)?)
}

/// Parse tunnel connect arguments
fn parse_tunnel_args(args: &[String]) -> anyhow::Result<(Option<String>, Option<PathBuf>)> {
    if args.is_empty() {
        return Ok((None, None));
    }

    if args[0] == "--jwt" {
        if args.len() < 2 {
            anyhow::bail!("--jwt requires a path argument");
        }
        Ok((None, Some(PathBuf::from(&args[1]))))
    } else {
        Ok((Some(args[0].clone()), None))
    }
}

/// Connect to tunnel with link code or JWT
async fn tunnel_connect(args: &[String]) -> anyhow::Result<()> {
    let (link_code, jwt_path) = parse_tunnel_args(args)?;

    let config = TunnelClientConfig {
        server_url: std::env::var("CUTTLEFISH_TUNNEL_URL")
            .unwrap_or_else(|_| "wss://tunnel.cuttlefish.ai".to_string()),
        local_addr: SocketAddr::from(([127, 0, 0, 1], 8080)),
        ..Default::default()
    };

    let client = TunnelClient::new(config);

    // Connect with link code or saved JWT
    if let Some(code) = link_code {
        let jwt = client.connect_with_link_code(&code).await?;
        save_jwt(&jwt)?;
        println!("✓ Connected! JWT saved to {}", get_config_dir().join("tunnel.jwt").display());
    } else if let Some(path) = jwt_path {
        let jwt = std::fs::read_to_string(&path)?;
        client.connect_with_jwt(jwt.trim()).await?;
        println!("✓ Connected with JWT from {}", path.display());
    } else {
        // Try loading saved JWT
        let jwt = load_saved_jwt()?;
        client.connect_with_jwt(&jwt).await?;
        println!("✓ Connected with saved JWT");
    }

    if let Some(subdomain) = client.subdomain() {
        println!("🌐 Tunnel available at: https://{}.cuttlefish.ai", subdomain);
    }

    // Run until Ctrl+C
    tokio::select! {
        result = client.run() => {
            if let Err(e) = result {
                eprintln!("✗ Tunnel error: {}", e);
                return Err(anyhow::anyhow!("{}", e));
            }
        }
        _ = tokio::signal::ctrl_c() => {
            println!("\n✓ Disconnecting...");
        }
    }

    Ok(())
}

/// Show tunnel status
async fn tunnel_status() -> anyhow::Result<()> {
    let jwt_path = get_config_dir().join("tunnel.jwt");
    if jwt_path.exists() {
        let jwt = std::fs::read_to_string(&jwt_path)?;
        // Decode JWT to show subdomain (without validation)
        let parts: Vec<&str> = jwt.trim().split('.').collect();
        if parts.len() == 3 {
            use base64::Engine;
            match base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(parts[1]) {
                Ok(payload) => {
                    match serde_json::from_slice::<serde_json::Value>(&payload) {
                        Ok(claims) => {
                            if let Some(subdomain) = claims["subdomain"].as_str() {
                                println!("✓ Connected");
                                println!("  Subdomain: {}", subdomain);
                                println!("  URL: https://{}.cuttlefish.ai", subdomain);
                            }
                            if let Some(exp) = claims["exp"].as_i64() {
                                println!("  Expires: {}", exp);
                            }
                        }
                        Err(_) => {
                            println!("✓ JWT saved at: {}", jwt_path.display());
                        }
                    }
                }
                Err(_) => {
                    println!("✓ JWT saved at: {}", jwt_path.display());
                }
            }
        } else {
            println!("✓ JWT saved at: {}", jwt_path.display());
        }
    } else {
        println!("✗ Not connected. Run `cuttlefish tunnel connect <link-code>` first.");
        std::process::exit(1);
    }
    Ok(())
}

/// Disconnect tunnel
async fn tunnel_disconnect() -> anyhow::Result<()> {
    let jwt_path = get_config_dir().join("tunnel.jwt");
    if jwt_path.exists() {
        std::fs::remove_file(&jwt_path)?;
        println!("✓ Tunnel JWT removed");
    } else {
        println!("✗ No tunnel connection found");
        std::process::exit(1);
    }
    Ok(())
}
