//! Cuttlefish server — multi-agent, multi-model agentic coding platform.
//!
//! Entry point that wires together all crates and starts the HTTP/WebSocket server.

use cuttlefish_agents::TokioMessageBus;
use cuttlefish_api::{
    ApiConfig, AuthConfig, WebUiConfig, build_full_app, build_full_app_with_webui,
    create_approval_registry, routes::AppState,
};
use cuttlefish_core::config::CuttlefishConfig;
use cuttlefish_core::traits::provider::ModelProvider;
use cuttlefish_core::updater::{
    AutoUpdateConfig, AutoUpdater, BinaryDownloader, DownloadConfig, RestartState, UpdateChecker,
    UpdateConfig, get_platform_binary_name,
};
use cuttlefish_core::{PricingConfig, TemplateRegistry, TimePeriod, UsageStats};
use cuttlefish_db::Database;
use cuttlefish_providers::ProviderRegistry;
use cuttlefish_tunnel::client::{TunnelClient, TunnelClientConfig};
use dashmap::DashMap;
use sqlx::SqlitePool;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{error, info, warn};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize structured logging first
    cuttlefish_core::tracing::init_tracing();

    // Parse CLI arguments
    let args: Vec<String> = std::env::args().collect();
    let mut config_path: Option<PathBuf> = None;
    let mut i = 1;

    while i < args.len() {
        match args[i].as_str() {
            "--config" | "-c" => {
                if i + 1 < args.len() {
                    config_path = Some(PathBuf::from(&args[i + 1]));
                    i += 2;
                } else {
                    eprintln!("Error: --config requires a path argument");
                    std::process::exit(1);
                }
            }
            "validate-templates" => {
                let dir = args
                    .get(i + 1)
                    .map(PathBuf::from)
                    .unwrap_or_else(|| PathBuf::from("templates"));
                return validate_templates(&dir);
            }
            "tunnel" => {
                if i + 1 >= args.len() {
                    eprintln!("Usage: cuttlefish tunnel <connect|status|disconnect>");
                    std::process::exit(1);
                }
                return match args[i + 1].as_str() {
                    "connect" => tunnel_connect(&args[i + 2..]).await,
                    "status" => tunnel_status().await,
                    "disconnect" => tunnel_disconnect().await,
                    _ => {
                        eprintln!("Unknown tunnel command: {}", args[i + 1]);
                        std::process::exit(1);
                    }
                };
            }
            "usage" | "costs" => {
                return usage_command(&args[i + 1..], &config_path).await;
            }
            "pricing" => {
                return pricing_command(&args[i + 1..], &config_path).await;
            }
            "memory" => {
                let project = args.get(i + 1).map(|s| s.as_str());
                return memory_command(project).await;
            }
            "why" => {
                let file = args.get(i + 1).ok_or_else(|| {
                    eprintln!("Usage: cuttlefish why <file>");
                    std::process::exit(1);
                });
                return why_command(file.expect("file required")).await;
            }
            "branch" => {
                return branch_command(&args[i + 1..]).await;
            }
            "checkpoint" => {
                return checkpoint_command(&args[i + 1..]).await;
            }
            "rollback" => {
                return rollback_command(&args[i + 1..]).await;
            }
            "undo" => {
                let count = args.get(i + 1).and_then(|s| s.parse().ok()).unwrap_or(1);
                return undo_command(count).await;
            }
            "safety" => {
                return safety_command(&args[i + 1..]).await;
            }
            "update" => {
                return update_command(&args[i + 1..]).await;
            }
            "--help" | "-h" => {
                print_help();
                return Ok(());
            }
            "--version" | "-V" => {
                println!("cuttlefish {}", env!("CARGO_PKG_VERSION"));
                return Ok(());
            }
            arg if arg.starts_with('-') => {
                eprintln!("Unknown option: {}", arg);
                eprintln!("Run 'cuttlefish --help' for usage information.");
                std::process::exit(1);
            }
            arg => {
                eprintln!("Unknown command: {}", arg);
                eprintln!("Run 'cuttlefish --help' for usage information.");
                std::process::exit(1);
            }
        }
    }

    // Load configuration
    let config = match &config_path {
        Some(path) => {
            info!("Loading config from: {}", path.display());
            CuttlefishConfig::load_from_path(path).unwrap_or_else(|e| {
                eprintln!("Error loading config from {}: {}", path.display(), e);
                std::process::exit(1);
            })
        }
        None => CuttlefishConfig::load().unwrap_or_else(|_| {
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
                webui: None,
                auto_update: cuttlefish_core::config::AutoUpdateConfigToml::default(),
            }
        }),
    };

    let api_key = config
        .server
        .api_key
        .clone()
        .or_else(|| std::env::var("CUTTLEFISH_API_KEY").ok())
        .unwrap_or_else(|| {
            tracing::warn!("No API key configured. Set CUTTLEFISH_API_KEY or add api_key to [server] in cuttlefish.toml");
            "changeme".to_string()
        });

    let addr = format!("{}:{}", config.server.host, config.server.port);

    info!("🐙 Cuttlefish starting on http://{}", addr);
    info!("WebSocket endpoint: ws://{}/ws", addr);
    info!("Health check: http://{}/health", addr);

    let db = Database::open(&config.database.path)
        .await
        .expect("Failed to open database");
    let db = Arc::new(db);

    let db_url = format!(
        "sqlite://{}?mode=rwc",
        config.database.path.to_string_lossy()
    );
    let pool = SqlitePool::connect(&db_url)
        .await
        .expect("Failed to create SQLite pool");
    let pool = Arc::new(pool);

    cuttlefish_db::usage::run_usage_migrations(&pool)
        .await
        .expect("Failed to run usage migrations");

    // Initialize provider registry
    let mut provider_registry = ProviderRegistry::new();
    let mut default_provider: Option<String> = None;

    // Load providers from config
    for (name, provider_config) in &config.providers {
        match initialize_provider(name, provider_config).await {
            Ok(provider) => {
                info!("Initialized provider: {}", name);
                provider_registry.register(name.clone(), provider);
                if default_provider.is_none() {
                    default_provider = Some(name.clone());
                }
            }
            Err(e) => {
                warn!("Failed to initialize provider '{}': {}", name, e);
            }
        }
    }

    if provider_registry.is_empty() {
        warn!("No providers configured. Add provider configuration to cuttlefish.toml");
        warn!("Example: [providers.anthropic]");
        warn!("         provider_type = \"anthropic\"");
        warn!("         model = \"claude-sonnet-4-6\"");
    } else {
        info!(
            "Loaded {} provider(s): {:?}",
            provider_registry.len(),
            provider_registry.names()
        );
    }

    let provider_registry = Arc::new(provider_registry);

    // Set up prompts directory
    let prompts_dir = std::env::var("CUTTLEFISH_PROMPTS_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("./prompts"));

    if !prompts_dir.exists() {
        info!(
            "Prompts directory not found at {}, using built-in prompts",
            prompts_dir.display()
        );
    }

    let template_registry = Arc::new(TemplateRegistry::new());
    let message_bus = Arc::new(TokioMessageBus::new());
    let active_sessions = Arc::new(DashMap::new());
    let approval_registry = create_approval_registry();

    let state = AppState {
        api_key: api_key.clone(),
        template_registry,
        db,
        provider_registry: provider_registry.clone(),
        active_sessions,
        message_bus,
        prompts_dir,
        default_provider,
        approval_registry,
    };

    let jwt_secret = std::env::var("CUTTLEFISH_JWT_SECRET")
        .unwrap_or_else(|_| api_key.clone())
        .into_bytes();

    let auth_config = AuthConfig::new(jwt_secret)
        .with_legacy_api_key(api_key)
        .with_db((*pool).clone());

    let projects_dir = std::env::var("CUTTLEFISH_PROJECTS_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("./projects"));

    let pricing_config = PricingConfig::with_defaults();

    let api_config = ApiConfig {
        app_state: state,
        pool: pool.clone(),
        auth_config,
        projects_dir,
        pricing_config,
        provider_registry: Some(provider_registry),
    };

    let webui_config = if let Some(ref webui) = config.webui {
        if webui.enabled {
            WebUiConfig::new(&webui.static_dir)
        } else {
            WebUiConfig::disabled()
        }
    } else {
        let default_paths = [
            PathBuf::from("/opt/cuttlefish/webui"),
            PathBuf::from("./webui"),
            PathBuf::from("./cuttlefish-web/.output/public"),
        ];
        default_paths
            .into_iter()
            .find(|p| p.join("index.html").exists())
            .map(WebUiConfig::new)
            .unwrap_or_else(WebUiConfig::disabled)
    };

    let app = if webui_config.enabled && webui_config.is_valid() {
        info!("WebUI enabled: {}", webui_config.static_dir.display());
        build_full_app_with_webui(api_config, webui_config)
    } else {
        if webui_config.enabled {
            info!("WebUI directory not found, serving API only");
        }
        build_full_app(api_config)
    };

    // Start Discord bot if configured
    let _discord_handle = if let Some(ref discord_config) = config.discord {
        let token = std::env::var(&discord_config.token_env_var).ok();
        if let Some(token) = token {
            info!("Starting Discord bot...");
            let bot_config = cuttlefish_discord::BotConfig::new(token)
                .with_guild_ids(discord_config.guild_ids.clone());
            Some(cuttlefish_discord::start_bot_background(bot_config))
        } else {
            warn!(
                "Discord configured but {} env var not set",
                discord_config.token_env_var
            );
            None
        }
    } else {
        None
    };

    // Start auto-updater if enabled
    let auto_updater = if config.auto_update.enabled {
        let mut auto_config = AutoUpdateConfig {
            enabled: true,
            poll_interval_secs: config.auto_update.poll_interval_secs,
            auto_apply: config.auto_update.auto_apply,
            ..AutoUpdateConfig::default()
        };

        // Use custom download_dir from config if specified
        if let Some(ref download_dir) = config.auto_update.download_dir {
            auto_config.download_dir = download_dir.clone();
        }

        let updater = Arc::new(AutoUpdater::new(auto_config));

        // Check for restart state from previous update
        if let Some(restart_state) = updater.load_restart_state().await {
            info!(
                "Resuming after update from v{} to v{}",
                restart_state.from_version, restart_state.to_version
            );
            info!(
                "Restoring {} active sessions",
                restart_state.active_sessions.len()
            );
            // TODO: Restore active sessions and pending approvals
        }

        // Start polling in background
        let polling_handle = updater.clone().start_polling();
        info!("Auto-update polling started");

        Some((updater, polling_handle))
    } else {
        info!("Auto-update is disabled");
        None
    };

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    info!("Server listening on {}", addr);

    // Create shutdown future that listens for both CTRL+C and auto-updater signals
    let shutdown_fut = {
        let updater_shutdown = auto_updater.as_ref().map(|(u, _)| u.shutdown_receiver());
        async move {
            tokio::select! {
                _ = shutdown_signal() => {
                    info!("CTRL+C shutdown");
                    false // Normal shutdown
                }
                result = async {
                    if let Some(mut rx) = updater_shutdown {
                        rx.recv().await
                    } else {
                        std::future::pending::<Result<(), broadcast::error::RecvError>>().await
                    }
                } => {
                    if result.is_ok() {
                        info!("Auto-updater triggered shutdown for update");
                        true // Update shutdown
                    } else {
                        false
                    }
                }
            }
        }
    };

    // Run server with shutdown future
    let (shutdown_reason, _) = tokio::join!(shutdown_fut, axum::serve(listener, app));

    // If shutdown was due to auto-update, apply it
    if shutdown_reason && let Some((updater, _handle)) = &auto_updater {
        info!("Applying update...");
        let state = RestartState {
            active_sessions: vec![],
            pending_approvals: vec![],
            saved_at: chrono::Utc::now().to_rfc3339(),
            from_version: env!("CARGO_PKG_VERSION").to_string(),
            to_version: "unknown".to_string(), // Will be filled by updater
        };
        if let Err(e) = updater.apply_update(state).await {
            error!("Failed to apply update: {}", e);
        }
    }

    // Shutdown auto-updater if running
    if let Some((updater, _handle)) = auto_updater {
        updater.shutdown();
    }

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

/// Initialize a model provider from configuration.
async fn initialize_provider(
    name: &str,
    config: &cuttlefish_core::config::ProviderConfig,
) -> anyhow::Result<Arc<dyn ModelProvider>> {
    use cuttlefish_providers::{anthropic, bedrock, google, ollama, openai};

    let provider_type = config.provider_type.as_str();

    match provider_type {
        "anthropic" => {
            let model = config.model.as_deref().unwrap_or("claude-sonnet-4-6");
            let provider = anthropic::AnthropicProvider::new(model)
                .map_err(|e| anyhow::anyhow!("Failed to create Anthropic provider: {}", e))?;
            Ok(Arc::new(provider) as Arc<dyn ModelProvider>)
        }
        "openai" => {
            let model = config.model.as_deref().unwrap_or("gpt-4o");
            let provider = openai::OpenAiProvider::new(model)
                .map_err(|e| anyhow::anyhow!("Failed to create OpenAI provider: {}", e))?;
            Ok(Arc::new(provider) as Arc<dyn ModelProvider>)
        }
        "bedrock" => {
            // Bedrock model ID includes the full ARN-style format
            let model = config
                .model
                .as_deref()
                .unwrap_or("anthropic.claude-sonnet-4-6-20250514-v1:0");
            let provider = bedrock::BedrockProvider::new(model)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to create Bedrock provider: {}", e))?;
            Ok(Arc::new(provider) as Arc<dyn ModelProvider>)
        }
        "google" | "gemini" => {
            let model = config.model.as_deref().unwrap_or("gemini-2.0-flash");
            let provider = google::GoogleProvider::new(model)
                .map_err(|e| anyhow::anyhow!("Failed to create Google provider: {}", e))?;
            Ok(Arc::new(provider) as Arc<dyn ModelProvider>)
        }
        "ollama" => {
            let model = config.model.as_deref().unwrap_or("llama3.1");
            let provider = if let Some(ref base_url) = config.base_url {
                ollama::OllamaProvider::with_base_url(base_url, model)
            } else {
                ollama::OllamaProvider::new(model)
            };
            Ok(Arc::new(provider) as Arc<dyn ModelProvider>)
        }
        _ => {
            anyhow::bail!(
                "Unknown provider type '{}' for provider '{}'. Supported: anthropic, openai, bedrock, google, ollama",
                provider_type,
                name
            );
        }
    }
}

fn print_help() {
    println!(
        "Cuttlefish {} - Multi-agent, multi-model agentic coding platform",
        env!("CARGO_PKG_VERSION")
    );
    println!();
    println!("USAGE:");
    println!("    cuttlefish-rs [OPTIONS] [COMMAND]");
    println!();
    println!("OPTIONS:");
    println!("    -c, --config <PATH>    Path to configuration file");
    println!("    -h, --help             Print this help message");
    println!("    -V, --version          Print version information");
    println!();
    println!("COMMANDS:");
    println!("    validate-templates     Validate template files in a directory");
    println!("    tunnel <SUBCOMMAND>    Manage tunnel connections");
    println!("    usage [OPTIONS]        Show usage statistics");
    println!("    costs [OPTIONS]        Show cost breakdown (alias for usage)");
    println!("    pricing [SUBCOMMAND]   Show or update pricing configuration");
    println!("    memory [PROJECT]       Show project memory summary");
    println!("    why <FILE>             Show why a file was changed");
    println!("    branch <SUBCOMMAND>    Manage project state branches");
    println!();
    println!("USAGE OPTIONS:");
    println!("    --project <ID>         Show usage for a specific project");
    println!("    --daily                Show daily breakdown");
    println!("    --weekly               Use weekly time period");
    println!("    --monthly              Use monthly time period (default)");
    println!("    --export <PATH>        Export usage data to CSV file");
    println!("    --json                 Output in JSON format");
    println!();
    println!("PRICING SUBCOMMANDS:");
    println!("    pricing                Show current pricing table");
    println!("    pricing set <PROVIDER> <MODEL> <INPUT> <OUTPUT>");
    println!("                           Set pricing (USD per million tokens)");
    println!();
    println!("UPDATE:");
    println!("    update                 Check, download, and apply update automatically");
    println!("    update check           Just check if a new version is available");
    println!();
    println!("BRANCH SUBCOMMANDS:");
    println!("    branch list [PROJECT]  List all branches for a project");
    println!("    branch create <NAME>   Create a new branch");
    println!("    branch restore <NAME>  Restore a branch");
    println!("    branch delete <NAME>   Delete a branch");
    println!();
    println!("CHECKPOINT SUBCOMMANDS:");
    println!("    checkpoint             List checkpoints for current project");
    println!("    checkpoint create [DESCRIPTION]");
    println!("                           Create a manual checkpoint");
    println!("    checkpoint list        List all checkpoints");
    println!();
    println!("ROLLBACK:");
    println!("    rollback <CHECKPOINT_ID>");
    println!("                           Rollback to a specific checkpoint");
    println!("    rollback --latest      Rollback to the most recent checkpoint");
    println!("    rollback --yes         Skip confirmation prompt");
    println!();
    println!("UNDO:");
    println!("    undo [COUNT]           Undo last N operations (default: 1)");
    println!();
    println!("SAFETY SUBCOMMANDS:");
    println!("    safety config          Show current gate configuration");
    println!("    safety config --auto-approve <THRESHOLD>");
    println!("                           Set auto-approve threshold (0.0-1.0)");
    println!("    safety config --prompt <THRESHOLD>");
    println!("                           Set prompt threshold (0.0-1.0)");
    println!();
    println!("CONFIG SEARCH ORDER (if --config not specified):");
    println!("    1. ./cuttlefish.toml");
    println!("    2. /etc/cuttlefish/cuttlefish.toml");
    println!("    3. ~/.config/cuttlefish/config.toml");
    println!();
    println!("ENVIRONMENT VARIABLES:");
    println!("    CUTTLEFISH_API_KEY     API key for authentication");
    println!("    RUST_LOG               Log level (e.g., info, debug, trace)");
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
        println!(
            "✓ Connected! JWT saved to {}",
            get_config_dir().join("tunnel.jwt").display()
        );
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
        println!(
            "🌐 Tunnel available at: https://{}.cuttlefish.ai",
            subdomain
        );
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
                Ok(payload) => match serde_json::from_slice::<serde_json::Value>(&payload) {
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
                },
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

async fn get_db_pool(config_path: &Option<PathBuf>) -> anyhow::Result<SqlitePool> {
    let config = match config_path {
        Some(path) => CuttlefishConfig::load_from_path(path)?,
        None => CuttlefishConfig::load().unwrap_or_else(|_| CuttlefishConfig {
            server: cuttlefish_core::config::ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 8080,
                api_key: None,
            },
            database: cuttlefish_core::config::DatabaseConfig {
                path: PathBuf::from("cuttlefish.db"),
            },
            providers: HashMap::new(),
            agents: HashMap::new(),
            discord: None,
            sandbox: cuttlefish_core::config::SandboxConfig::default(),
            routing: cuttlefish_core::RoutingConfig::default(),
            webui: None,
            auto_update: cuttlefish_core::config::AutoUpdateConfigToml::default(),
        }),
    };

    let db_url = format!(
        "sqlite://{}?mode=rwc",
        config.database.path.to_string_lossy()
    );
    let pool = SqlitePool::connect(&db_url).await?;
    cuttlefish_db::usage::run_usage_migrations(&pool).await?;
    Ok(pool)
}

async fn usage_command(args: &[String], config_path: &Option<PathBuf>) -> anyhow::Result<()> {
    let mut project_id: Option<String> = None;
    let mut period = TimePeriod::Monthly;
    let mut daily_breakdown = false;
    let mut export_path: Option<PathBuf> = None;
    let mut json_output = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--project" => {
                if i + 1 < args.len() {
                    project_id = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    eprintln!("Error: --project requires an ID");
                    std::process::exit(1);
                }
            }
            "--daily" => {
                daily_breakdown = true;
                period = TimePeriod::Daily;
                i += 1;
            }
            "--weekly" => {
                period = TimePeriod::Weekly;
                i += 1;
            }
            "--monthly" => {
                period = TimePeriod::Monthly;
                i += 1;
            }
            "--export" => {
                if i + 1 < args.len() {
                    export_path = Some(PathBuf::from(&args[i + 1]));
                    i += 2;
                } else {
                    eprintln!("Error: --export requires a path");
                    std::process::exit(1);
                }
            }
            "--json" => {
                json_output = true;
                i += 1;
            }
            _ => {
                i += 1;
            }
        }
    }

    let pool = get_db_pool(config_path).await?;
    let pricing = PricingConfig::with_defaults();
    let stats = UsageStats::new(Arc::new(pool.clone()), pricing);

    if let Some(ref path) = export_path {
        return export_usage_csv(&pool, path).await;
    }

    if daily_breakdown {
        let daily = stats.daily_usage(project_id.as_deref(), period).await?;
        if json_output {
            println!("{}", serde_json::to_string_pretty(&daily)?);
        } else {
            println!("Daily Usage Breakdown");
            println!("{:-<60}", "");
            println!(
                "{:<12} {:>15} {:>15} {:>10}",
                "Date", "Input Tokens", "Output Tokens", "Requests"
            );
            println!("{:-<60}", "");
            for day in &daily {
                println!(
                    "{:<12} {:>15} {:>15} {:>10}",
                    day.date, day.input_tokens, day.output_tokens, day.request_count
                );
            }
        }
        return Ok(());
    }

    if let Some(pid) = project_id {
        let summary = stats.project_summary(&pid, period).await?;
        if json_output {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "project_id": summary.project_id,
                    "period": format!("{:?}", summary.period),
                    "total_requests": summary.total_requests,
                    "total_input_tokens": summary.total_input_tokens,
                    "total_output_tokens": summary.total_output_tokens,
                    "total_cost_usd": summary.total_cost_usd,
                    "by_provider": summary.by_provider,
                }))?
            );
        } else {
            print_project_summary(&summary);
        }
    } else {
        let providers = stats.provider_usage(None, period).await?;
        if json_output {
            println!("{}", serde_json::to_string_pretty(&providers)?);
        } else {
            println!("Usage Summary ({:?})", period);
            println!("{:-<70}", "");
            println!(
                "{:<15} {:>15} {:>15} {:>10} {:>12}",
                "Provider", "Input Tokens", "Output Tokens", "Requests", "Est. Cost"
            );
            println!("{:-<70}", "");

            let pricing = PricingConfig::with_defaults();
            let mut total_cost = 0.0;

            for p in &providers {
                let cost = estimate_provider_cost(
                    &pricing,
                    &p.provider,
                    p.input_tokens as u64,
                    p.output_tokens as u64,
                );
                total_cost += cost;
                println!(
                    "{:<15} {:>15} {:>15} {:>10} ${:>10.4}",
                    p.provider, p.input_tokens, p.output_tokens, p.request_count, cost
                );
            }
            println!("{:-<70}", "");
            println!(
                "{:<15} {:>15} {:>15} {:>10} ${:>10.4}",
                "TOTAL", "", "", "", total_cost
            );
        }
    }

    Ok(())
}

fn print_project_summary(summary: &cuttlefish_core::ProjectUsageSummary) {
    println!("Project Usage Summary: {}", summary.project_id);
    println!("Period: {:?}", summary.period);
    println!("{:-<50}", "");
    println!("Total Requests:      {:>15}", summary.total_requests);
    println!("Total Input Tokens:  {:>15}", summary.total_input_tokens);
    println!("Total Output Tokens: {:>15}", summary.total_output_tokens);
    println!("Estimated Cost:      ${:>14.4}", summary.total_cost_usd);
    println!();
    if !summary.by_provider.is_empty() {
        println!("Cost by Provider:");
        for (provider, cost) in &summary.by_provider {
            println!("  {:<15} ${:.4}", provider, cost);
        }
    }
}

fn estimate_provider_cost(pricing: &PricingConfig, provider: &str, input: u64, output: u64) -> f64 {
    let models = match provider {
        "anthropic" => vec!["claude-sonnet-4-6", "claude-opus-4-6", "claude-haiku-4-5"],
        "openai" => vec!["gpt-5.4", "gpt-4o", "gpt-5-nano"],
        "google" => vec!["gemini-2.0-flash", "gemini-1.5-pro"],
        _ => vec![],
    };

    for model in models {
        if let Some(cost) = pricing.calculate_cost(provider, model, input, output) {
            return cost;
        }
    }
    0.0
}

async fn export_usage_csv(pool: &SqlitePool, path: &PathBuf) -> anyhow::Result<()> {
    let (from, to) = TimePeriod::Monthly.range();
    let records = cuttlefish_db::usage::get_all_usage(pool, &from, &to).await?;

    let mut csv = String::from(
        "id,project_id,session_id,user_id,provider,model,input_tokens,output_tokens,request_type,latency_ms,success,error_type,created_at\n",
    );

    for r in records {
        let project_id = r.project_id.as_deref().unwrap_or("");
        let session_id = r.session_id.as_deref().unwrap_or("");
        let user_id = r.user_id.as_deref().unwrap_or("");
        let latency = r.latency_ms.map(|l| l.to_string()).unwrap_or_default();
        let error_type = r.error_type.as_deref().unwrap_or("");

        csv.push_str(&format!(
            "\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",{},{},\"{}\",{},{},\"{}\",\"{}\"\n",
            escape_csv(&r.id),
            escape_csv(project_id),
            escape_csv(session_id),
            escape_csv(user_id),
            escape_csv(&r.provider),
            escape_csv(&r.model),
            r.input_tokens,
            r.output_tokens,
            escape_csv(&r.request_type),
            latency,
            r.success,
            escape_csv(error_type),
            escape_csv(&r.created_at),
        ));
    }

    std::fs::write(path, csv)?;
    println!("✓ Exported usage data to {}", path.display());
    Ok(())
}

fn escape_csv(s: &str) -> String {
    s.replace('"', "\"\"")
}

async fn pricing_command(args: &[String], config_path: &Option<PathBuf>) -> anyhow::Result<()> {
    if args.is_empty() {
        let pricing = PricingConfig::with_defaults();
        println!("Current Pricing (USD per million tokens)");
        println!("{:-<65}", "");
        println!(
            "{:<15} {:<25} {:>10} {:>12}",
            "Provider", "Model", "Input", "Output"
        );
        println!("{:-<65}", "");

        for provider in pricing.providers() {
            if let Some(models) = pricing.models(provider) {
                for model in models {
                    if let Some(price) = pricing.get_price(provider, model) {
                        println!(
                            "{:<15} {:<25} ${:>9.2} ${:>11.2}",
                            provider, model, price.input_per_million, price.output_per_million
                        );
                    }
                }
            }
        }
        return Ok(());
    }

    if args[0] == "set" {
        if args.len() < 5 {
            eprintln!("Usage: cuttlefish pricing set <PROVIDER> <MODEL> <INPUT> <OUTPUT>");
            eprintln!("  INPUT and OUTPUT are prices in USD per million tokens");
            std::process::exit(1);
        }

        let provider = &args[1];
        let model = &args[2];
        let input: f64 = args[3]
            .parse()
            .map_err(|_| anyhow::anyhow!("Invalid input price"))?;
        let output: f64 = args[4]
            .parse()
            .map_err(|_| anyhow::anyhow!("Invalid output price"))?;

        let pool = get_db_pool(config_path).await?;
        let pricing = cuttlefish_db::usage::ModelPricing {
            id: uuid::Uuid::new_v4().to_string(),
            provider: provider.clone(),
            model: model.clone(),
            input_price_per_million: input,
            output_price_per_million: output,
            effective_from: chrono::Utc::now().to_rfc3339(),
            created_at: chrono::Utc::now().to_rfc3339(),
        };

        cuttlefish_db::usage::upsert_pricing(&pool, &pricing).await?;
        println!("✓ Set pricing for {}/{}:", provider, model);
        println!("  Input:  ${:.2} per million tokens", input);
        println!("  Output: ${:.2} per million tokens", output);
        return Ok(());
    }

    eprintln!("Unknown pricing command: {}", args[0]);
    eprintln!("Use 'cuttlefish pricing' to show pricing or 'cuttlefish pricing set ...' to update");
    std::process::exit(1);
}

async fn memory_command(project: Option<&str>) -> anyhow::Result<()> {
    use cuttlefish_agents::memory::ProjectMemory;

    let project_path = match project {
        Some(p) => PathBuf::from(p),
        None => std::env::current_dir()?,
    };

    let memory_path = ProjectMemory::default_path(&project_path);
    if !memory_path.exists() {
        println!("No memory file found for this project.");
        println!("Memory file would be at: {}", memory_path.display());
        return Ok(());
    }

    let memory = ProjectMemory::load(&memory_path)?;

    println!("# Project Memory: {}", memory.project_name);
    println!();

    if !memory.summary.is_empty() {
        println!("## Summary");
        println!("{}", memory.summary);
        println!();
    }

    if !memory.key_decisions.is_empty() {
        println!("## Key Decisions ({} total)", memory.key_decisions.len());
        for decision in memory.key_decisions.iter().take(5) {
            println!(
                "  - [{}] {} — {}",
                decision.date, decision.decision, decision.rationale
            );
        }
        if memory.key_decisions.len() > 5 {
            println!("  ... and {} more", memory.key_decisions.len() - 5);
        }
        println!();
    }

    if !memory.architecture.is_empty() {
        println!("## Architecture ({} components)", memory.architecture.len());
        for item in memory.architecture.iter().take(5) {
            println!("  - {}: {}", item.component, item.description);
        }
        if memory.architecture.len() > 5 {
            println!("  ... and {} more", memory.architecture.len() - 5);
        }
        println!();
    }

    if !memory.gotchas.is_empty() {
        println!("## Gotchas ({} total)", memory.gotchas.len());
        for item in memory.gotchas.iter().take(3) {
            println!("  - {}: {}", item.gotcha, item.context);
        }
        if memory.gotchas.len() > 3 {
            println!("  ... and {} more", memory.gotchas.len() - 3);
        }
        println!();
    }

    if let Some(ref task) = memory.active_context.current_task {
        println!("## Active Context");
        println!("  Currently working on: {}", task);
        if let Some(ref blockers) = memory.active_context.blockers {
            println!("  Blockers: {}", blockers);
        }
        if let Some(ref next) = memory.active_context.next_steps {
            println!("  Next steps: {}", next);
        }
    }

    Ok(())
}

async fn why_command(file: &str) -> anyhow::Result<()> {
    use cuttlefish_agents::memory::{DecisionIndex, DecisionLog, WhyTarget, why};

    let project_path = std::env::current_dir()?;
    let log_path = DecisionLog::default_path(&project_path);

    if !log_path.exists() {
        println!("No decision log found. Memory system may not be initialized.");
        return Ok(());
    }

    let log = DecisionLog::new(&log_path);
    let index = DecisionIndex::from_log(&log)?;

    let target = WhyTarget::parse(file);
    let explanation = why(&index, target);

    println!("{}", explanation.to_markdown());

    Ok(())
}

async fn branch_command(args: &[String]) -> anyhow::Result<()> {
    use cuttlefish_agents::memory::BranchStore;

    if args.is_empty() {
        eprintln!("Usage: cuttlefish branch <list|create|restore|delete> [args]");
        std::process::exit(1);
    }

    let project_path = std::env::current_dir()?;
    let project_id = project_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    match args[0].as_str() {
        "list" => {
            let mut store = BranchStore::new(&project_path);
            store.load()?;

            let branches = store.list_branches(&project_id);
            if branches.is_empty() {
                println!("No branches found for this project.");
            } else {
                println!("Branches for project '{}':", project_id);
                println!("{:-<60}", "");
                for branch in branches {
                    let desc = branch.description.as_deref().unwrap_or("-");
                    println!(
                        "  {} (created {})",
                        branch.name,
                        branch.created_at.format("%Y-%m-%d %H:%M")
                    );
                    println!("    Description: {}", desc);
                    println!("    Git ref: {}", branch.git_ref);
                    println!();
                }
            }
        }
        "create" => {
            let name = args.get(1).ok_or_else(|| {
                eprintln!("Usage: cuttlefish branch create <name>");
                anyhow::anyhow!("branch name required")
            })?;

            let description = args.get(2).map(|s| s.as_str());

            let mut store = BranchStore::new(&project_path);
            store.load()?;

            let branch = store.create_branch(&project_id, name, description, "HEAD")?;
            println!(
                "✓ Created branch '{}' at {}",
                branch.name,
                branch.created_at.format("%Y-%m-%d %H:%M")
            );
            println!(
                "  Memory and decisions backed up to .cuttlefish/branches/{}/",
                name
            );
        }
        "restore" => {
            let name = args.get(1).ok_or_else(|| {
                eprintln!("Usage: cuttlefish branch restore <name>");
                anyhow::anyhow!("branch name required")
            })?;

            let create_backup = args.get(2).is_some_and(|s| s == "--backup");

            let mut store = BranchStore::new(&project_path);
            store.load()?;

            let branch = store.restore_branch(&project_id, name, create_backup)?;
            println!("✓ Restored branch '{}'", branch.name);
            if create_backup {
                println!("  Current state backed up before restore");
            }
            println!("  Memory and decisions restored from branch");
            println!("  Note: Git checkout may be needed for full restore");
        }
        "delete" => {
            let name = args.get(1).ok_or_else(|| {
                eprintln!("Usage: cuttlefish branch delete <name>");
                anyhow::anyhow!("branch name required")
            })?;

            let mut store = BranchStore::new(&project_path);
            store.load()?;

            store.delete_branch(name)?;
            println!("✓ Deleted branch '{}'", name);
        }
        _ => {
            eprintln!("Unknown branch command: {}", args[0]);
            eprintln!("Available: list, create, restore, delete");
            std::process::exit(1);
        }
    }

    Ok(())
}

async fn checkpoint_command(args: &[String]) -> anyhow::Result<()> {
    use cuttlefish_agents::safety::{
        Checkpoint, CheckpointComponents, CheckpointConfig, CheckpointManager, CheckpointStore,
        CheckpointTrigger, InMemoryCheckpointStore,
    };

    let project_path = std::env::current_dir()?;
    let project_id = project_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    let checkpoint_dir = project_path.join(".cuttlefish/checkpoints");
    std::fs::create_dir_all(&checkpoint_dir)?;

    let store = InMemoryCheckpointStore::new();
    let config = CheckpointConfig::default().with_checkpoint_dir(&checkpoint_dir);
    let manager = CheckpointManager::new(store, config);

    let checkpoints_file = checkpoint_dir.join("checkpoints.json");
    let existing_checkpoints: Vec<Checkpoint> = if checkpoints_file.exists() {
        let content = std::fs::read_to_string(&checkpoints_file)?;
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        Vec::new()
    };

    for cp in &existing_checkpoints {
        manager.store().save(cp).await?;
    }

    if args.is_empty() || args[0] == "list" {
        let checkpoints = manager.list_checkpoints(&project_id).await?;
        if checkpoints.is_empty() {
            println!("No checkpoints found for project '{}'.", project_id);
            println!("Create one with: cuttlefish checkpoint create [description]");
        } else {
            println!("Checkpoints for project '{}':", project_id);
            println!("{:-<80}", "");
            println!("{:<36} {:<20} Description", "ID", "Created");
            println!("{:-<80}", "");
            for cp in checkpoints {
                println!(
                    "{:<36} {:<20} {}",
                    cp.id,
                    cp.created_at.format("%Y-%m-%d %H:%M"),
                    cp.description
                );
            }
        }
        return Ok(());
    }

    match args[0].as_str() {
        "create" => {
            let description = args
                .get(1)
                .map(|s| s.as_str())
                .unwrap_or("Manual checkpoint");

            let git_ref = std::process::Command::new("git")
                .args(["rev-parse", "HEAD"])
                .current_dir(&project_path)
                .output()
                .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
                .unwrap_or_else(|_| "HEAD".to_string());

            let memory_backup_path = checkpoint_dir.join(format!(
                "memory_backup_{}.json",
                chrono::Utc::now().format("%Y%m%d_%H%M%S")
            ));

            let memory_file = project_path.join(".cuttlefish/memory.json");
            if memory_file.exists() {
                std::fs::copy(&memory_file, &memory_backup_path)?;
            }

            let components = CheckpointComponents::new(
                git_ref,
                format!("cli-checkpoint-{}", uuid::Uuid::new_v4()),
                memory_backup_path,
            );

            let trigger = CheckpointTrigger::Manual {
                user_id: whoami::username(),
            };

            let checkpoint = manager
                .create_checkpoint(&project_id, description, trigger, components)
                .await?;

            let all_checkpoints = manager.list_checkpoints(&project_id).await?;
            let json = serde_json::to_string_pretty(&all_checkpoints)?;
            std::fs::write(&checkpoints_file, json)?;

            println!("✓ Created checkpoint: {}", checkpoint.id);
            println!("  Description: {}", checkpoint.description);
            println!("  Git ref: {}", checkpoint.components.git_ref);
            println!(
                "  Created at: {}",
                checkpoint.created_at.format("%Y-%m-%d %H:%M:%S UTC")
            );
        }
        "list" => {}
        _ => {
            eprintln!("Unknown checkpoint command: {}", args[0]);
            eprintln!("Available: create, list");
            std::process::exit(1);
        }
    }

    Ok(())
}

async fn rollback_command(args: &[String]) -> anyhow::Result<()> {
    use cuttlefish_agents::safety::{
        Checkpoint, CheckpointComponents, CheckpointConfig, CheckpointManager, CheckpointStore,
        InMemoryCheckpointStore,
    };

    let project_path = std::env::current_dir()?;
    let project_id = project_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    let checkpoint_dir = project_path.join(".cuttlefish/checkpoints");
    let checkpoints_file = checkpoint_dir.join("checkpoints.json");

    if !checkpoints_file.exists() {
        eprintln!("No checkpoints found. Create one first with: cuttlefish checkpoint create");
        std::process::exit(1);
    }

    let content = std::fs::read_to_string(&checkpoints_file)?;
    let checkpoints: Vec<Checkpoint> = serde_json::from_str(&content)?;

    if checkpoints.is_empty() {
        eprintln!("No checkpoints found. Create one first with: cuttlefish checkpoint create");
        std::process::exit(1);
    }

    let mut checkpoint_id: Option<String> = None;
    let mut use_latest = false;
    let mut skip_confirm = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--latest" => {
                use_latest = true;
                i += 1;
            }
            "--yes" | "-y" => {
                skip_confirm = true;
                i += 1;
            }
            _ => {
                if checkpoint_id.is_none() && !args[i].starts_with('-') {
                    checkpoint_id = Some(args[i].clone());
                }
                i += 1;
            }
        }
    }

    let target = if use_latest {
        checkpoints.first().cloned()
    } else if let Some(id) = checkpoint_id {
        checkpoints
            .iter()
            .find(|c| c.id.as_str() == id || c.id.as_str().starts_with(&id))
            .cloned()
    } else {
        eprintln!("Usage: cuttlefish rollback <checkpoint-id> [--yes]");
        eprintln!("       cuttlefish rollback --latest [--yes]");
        std::process::exit(1);
    };

    let target = target.ok_or_else(|| {
        eprintln!("Checkpoint not found");
        anyhow::anyhow!("checkpoint not found")
    })?;

    println!("Rollback to checkpoint:");
    println!("  ID: {}", target.id);
    println!("  Description: {}", target.description);
    println!(
        "  Created: {}",
        target.created_at.format("%Y-%m-%d %H:%M:%S UTC")
    );
    println!("  Git ref: {}", target.components.git_ref);
    println!();

    if !skip_confirm {
        print!("Are you sure you want to rollback? This will restore git state and memory. [y/N] ");
        use std::io::Write;
        std::io::stdout().flush()?;

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Rollback cancelled.");
            return Ok(());
        }
    }

    let store = InMemoryCheckpointStore::new();
    let config = CheckpointConfig::default().with_checkpoint_dir(&checkpoint_dir);
    let manager = CheckpointManager::new(store, config);

    for cp in &checkpoints {
        manager.store().save(cp).await?;
    }

    let current_git_ref = std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(&project_path)
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "HEAD".to_string());

    let safety_memory_path = checkpoint_dir.join(format!(
        "pre_rollback_memory_{}.json",
        chrono::Utc::now().format("%Y%m%d_%H%M%S")
    ));

    let memory_file = project_path.join(".cuttlefish/memory.json");
    if memory_file.exists() {
        std::fs::copy(&memory_file, &safety_memory_path)?;
    }

    let current_components = CheckpointComponents::new(
        current_git_ref,
        format!("pre-rollback-{}", uuid::Uuid::new_v4()),
        safety_memory_path,
    );

    let (_, safety) = manager
        .prepare_rollback(&project_id, &target.id, true, Some(current_components))
        .await?;

    println!("Restoring git state...");
    let git_result = std::process::Command::new("git")
        .args(["checkout", &target.components.git_ref])
        .current_dir(&project_path)
        .status();

    let git_restored = git_result.map(|s| s.success()).unwrap_or(false);
    if git_restored {
        println!("  ✓ Git state restored to {}", target.components.git_ref);
    } else {
        println!("  ⚠ Git checkout failed (you may need to commit or stash changes first)");
    }

    println!("Restoring memory state...");
    let memory_restored = if target.components.memory_backup_path.exists() {
        std::fs::copy(&target.components.memory_backup_path, &memory_file).is_ok()
    } else {
        false
    };

    if memory_restored {
        println!("  ✓ Memory state restored");
    } else {
        println!("  ⚠ Memory backup not found or restore failed");
    }

    let all_checkpoints = manager.list_checkpoints(&project_id).await?;
    let json = serde_json::to_string_pretty(&all_checkpoints)?;
    std::fs::write(&checkpoints_file, json)?;

    println!();
    println!("✓ Rollback complete");
    if let Some(safety) = safety {
        println!("  Safety checkpoint created: {}", safety.id);
        println!("  (Use this ID to undo the rollback if needed)");
    }

    Ok(())
}

async fn undo_command(count: usize) -> anyhow::Result<()> {
    println!("Undo last {} operation(s)", count);
    println!();
    println!("Note: Full undo functionality requires an operation journal.");
    println!("For now, you can use checkpoints to restore previous states:");
    println!();
    println!("  cuttlefish checkpoint list     # List available checkpoints");
    println!("  cuttlefish rollback --latest   # Rollback to most recent checkpoint");
    println!();
    println!("To enable automatic checkpoints before risky operations,");
    println!("the agent system will create them automatically.");

    Ok(())
}

async fn safety_command(args: &[String]) -> anyhow::Result<()> {
    use cuttlefish_agents::safety::GateConfig;

    let project_path = std::env::current_dir()?;
    let config_path = project_path.join(".cuttlefish/safety_config.json");

    let mut config: GateConfig = if config_path.exists() {
        let content = std::fs::read_to_string(&config_path)?;
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        GateConfig::default()
    };

    if args.is_empty() || args[0] == "config" {
        let mut i = if args.first().is_some_and(|s| s == "config") {
            1
        } else {
            0
        };
        let mut updated = false;

        while i < args.len() {
            match args[i].as_str() {
                "--auto-approve" => {
                    if i + 1 < args.len() {
                        let threshold: f32 = args[i + 1].parse().map_err(|_| {
                            anyhow::anyhow!("Invalid threshold value: {}", args[i + 1])
                        })?;
                        config.auto_approve_threshold = threshold.clamp(0.0, 1.0);
                        updated = true;
                        i += 2;
                    } else {
                        eprintln!("Error: --auto-approve requires a threshold value (0.0-1.0)");
                        std::process::exit(1);
                    }
                }
                "--prompt" => {
                    if i + 1 < args.len() {
                        let threshold: f32 = args[i + 1].parse().map_err(|_| {
                            anyhow::anyhow!("Invalid threshold value: {}", args[i + 1])
                        })?;
                        config.prompt_threshold = threshold.clamp(0.0, 1.0);
                        updated = true;
                        i += 2;
                    } else {
                        eprintln!("Error: --prompt requires a threshold value (0.0-1.0)");
                        std::process::exit(1);
                    }
                }
                _ => {
                    i += 1;
                }
            }
        }

        if updated {
            std::fs::create_dir_all(config_path.parent().expect("parent dir"))?;
            let json = serde_json::to_string_pretty(&config)?;
            std::fs::write(&config_path, json)?;
            println!("✓ Safety configuration updated");
        }

        println!("Safety Gate Configuration:");
        println!("{:-<50}", "");
        println!(
            "  Auto-approve threshold: {:.2}",
            config.auto_approve_threshold
        );
        println!("  Prompt threshold:       {:.2}", config.prompt_threshold);
        println!();
        println!(
            "Actions with confidence >= {:.2} are auto-approved.",
            config.auto_approve_threshold
        );
        println!(
            "Actions with confidence >= {:.2} but < {:.2} prompt for approval.",
            config.prompt_threshold, config.auto_approve_threshold
        );
        println!(
            "Actions with confidence < {:.2} are blocked.",
            config.prompt_threshold
        );

        return Ok(());
    }

    eprintln!("Unknown safety command: {}", args[0]);
    eprintln!("Available: config");
    std::process::exit(1);
}

/// Stop cuttlefish systemd service if running. Returns true if it was running.
fn stop_cuttlefish_service() -> bool {
    // Check if systemctl exists
    if std::process::Command::new("systemctl")
        .args(["--version"])
        .output()
        .is_err()
    {
        return false;
    }

    // Check if service is active
    let is_active = std::process::Command::new("systemctl")
        .args(["is-active", "--quiet", "cuttlefish"])
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    if is_active {
        println!("Stopping cuttlefish service...");
        let _ = std::process::Command::new("systemctl")
            .args(["stop", "cuttlefish"])
            .status();
        // Wait a moment for the process to fully stop
        std::thread::sleep(std::time::Duration::from_secs(2));
        true
    } else {
        false
    }
}

/// Start cuttlefish systemd service.
fn start_cuttlefish_service() {
    let _ = std::process::Command::new("systemctl")
        .args(["start", "cuttlefish"])
        .status();
}

async fn update_command(args: &[String]) -> anyhow::Result<()> {
    let config = UpdateConfig::default();
    let checker = UpdateChecker::new(config);

    let subcommand = args.first().map(|s| s.as_str()).unwrap_or("");

    match subcommand {
        "check" => {
            println!(
                "Checking for updates... (current: v{})",
                env!("CARGO_PKG_VERSION")
            );

            match checker.check_for_update().await {
                Ok(Some(update)) => {
                    println!();
                    println!("✓ Update available!");
                    println!("  Current version: v{}", update.current_version);
                    println!("  Latest version:  v{}", update.latest_version);
                    if let Some(ref notes) = update.release_notes {
                        println!();
                        println!("Release notes:");
                        for line in notes.lines().take(10) {
                            println!("  {}", line);
                        }
                        if notes.lines().count() > 10 {
                            println!("  ...(truncated)");
                        }
                    }
                    println!();
                    println!("To update: cuttlefish update");
                }
                Ok(None) => {
                    println!(
                        "✓ You are running the latest version (v{})",
                        env!("CARGO_PKG_VERSION")
                    );
                }
                Err(e) => {
                    eprintln!("✗ Failed to check for updates: {}", e);
                    std::process::exit(1);
                }
            }
        }
        "" => {
            // Default: full automatic update
            println!(
                "Checking for updates... (current: v{})",
                env!("CARGO_PKG_VERSION")
            );

            let update = match checker.check_for_update().await {
                Ok(Some(update)) => update,
                Ok(None) => {
                    println!(
                        "✓ You are running the latest version (v{})",
                        env!("CARGO_PKG_VERSION")
                    );
                    return Ok(());
                }
                Err(e) => {
                    eprintln!("✗ Failed to check for updates: {}", e);
                    std::process::exit(1);
                }
            };

            println!();
            println!(
                "✓ Update available: v{} → v{}",
                update.current_version, update.latest_version
            );

            let download_url = match update.download_url {
                Some(url) => url,
                None => {
                    eprintln!(
                        "✗ No download URL found for your platform ({})",
                        get_platform_binary_name()
                    );
                    eprintln!("  Please download manually from GitHub releases.");
                    std::process::exit(1);
                }
            };

            // Download
            let download_dir = dirs::cache_dir()
                .unwrap_or_else(std::env::temp_dir)
                .join("cuttlefish-updates");
            std::fs::create_dir_all(&download_dir)?;

            let binary_name = get_platform_binary_name();
            let tarball_path = download_dir.join(format!("{}.tar.gz", binary_name));

            println!("Downloading v{}...", update.latest_version);

            let dl_config = DownloadConfig {
                download_dir: download_dir.clone(),
                verify_checksums: update.checksum_url.is_some(),
            };
            let downloader = BinaryDownloader::new(dl_config);

            let progress_callback = |progress: cuttlefish_core::updater::DownloadProgress| {
                if let Some(percent) = progress.percent {
                    print!("\r  Progress: {:.1}%  ", percent);
                    use std::io::Write;
                    std::io::stdout().flush().ok();
                }
            };

            if let Some(ref checksum_url) = update.checksum_url {
                downloader
                    .download_and_verify(
                        &download_url,
                        checksum_url,
                        &tarball_path,
                        Some(&progress_callback),
                    )
                    .await?;
            } else {
                downloader
                    .download_binary(&download_url, &tarball_path, Some(&progress_callback))
                    .await?;
            }
            println!("\r✓ Downloaded v{}         ", update.latest_version);

            // Extract binary from tarball
            let extract_dir = download_dir.join("extract");
            std::fs::create_dir_all(&extract_dir)?;

            let tarball_file = std::fs::File::open(&tarball_path)?;
            let tar_decoder = flate2::read::GzDecoder::new(tarball_file);
            let mut archive = tar::Archive::new(tar_decoder);
            archive.unpack(&extract_dir)?;

            let extracted_binary = extract_dir.join("cuttlefish-rs");
            if !extracted_binary.exists() {
                anyhow::bail!(
                    "Binary not found in tarball. Expected: {}",
                    extracted_binary.display()
                );
            }

            // Stop service if running
            let service_was_running = stop_cuttlefish_service();

            // Apply update using atomic rename (works even if binary is running)
            let current_exe = std::env::current_exe()?;
            let backup_path = current_exe.with_extension("bak");
            let temp_path = current_exe.with_extension("new");

            println!("Applying update...");

            // Create backup of current binary
            std::fs::copy(&current_exe, &backup_path)?;

            // Copy extracted binary to temp location
            std::fs::copy(&extracted_binary, &temp_path)?;

            // Make executable on Unix
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = std::fs::metadata(&temp_path)?.permissions();
                perms.set_mode(0o755);
                std::fs::set_permissions(&temp_path, perms)?;
            }

            // Atomic rename - this works even if the binary is running on Unix
            // The running process keeps the old inode, but the path points to new file
            std::fs::rename(&temp_path, &current_exe)?;

            // Clean up downloaded and extracted files
            std::fs::remove_file(&tarball_path).ok();
            std::fs::remove_dir_all(&extract_dir).ok();

            println!("✓ Updated to v{}", update.latest_version);

            // Restart service if it was running
            if service_was_running {
                println!("Restarting service...");
                start_cuttlefish_service();
                println!("✓ Service restarted");
            } else {
                println!();
                println!("Restart cuttlefish to use the new version.");
            }
        }
        "download" => {
            println!(
                "Checking for updates... (current: v{})",
                env!("CARGO_PKG_VERSION")
            );

            let update = match checker.check_for_update().await {
                Ok(Some(update)) => update,
                Ok(None) => {
                    println!(
                        "✓ You are running the latest version (v{})",
                        env!("CARGO_PKG_VERSION")
                    );
                    return Ok(());
                }
                Err(e) => {
                    eprintln!("✗ Failed to check for updates: {}", e);
                    std::process::exit(1);
                }
            };

            let download_url = match update.download_url {
                Some(url) => url,
                None => {
                    eprintln!(
                        "✗ No download URL found for your platform ({})",
                        get_platform_binary_name()
                    );
                    eprintln!("  Please download manually from GitHub releases.");
                    std::process::exit(1);
                }
            };

            let download_dir = dirs::cache_dir()
                .unwrap_or_else(std::env::temp_dir)
                .join("cuttlefish-updates");
            std::fs::create_dir_all(&download_dir)?;

            let binary_name = get_platform_binary_name();
            let tarball_path = download_dir.join(format!("{}.tar.gz", binary_name));

            println!("Downloading v{}...", update.latest_version);

            let dl_config = DownloadConfig {
                download_dir: download_dir.clone(),
                verify_checksums: update.checksum_url.is_some(),
            };
            let downloader = BinaryDownloader::new(dl_config);

            let progress_callback = |progress: cuttlefish_core::updater::DownloadProgress| {
                if let Some(percent) = progress.percent {
                    print!("\r  Progress: {:.1}%  ", percent);
                    use std::io::Write;
                    std::io::stdout().flush().ok();
                }
            };

            if let Some(ref checksum_url) = update.checksum_url {
                downloader
                    .download_and_verify(
                        &download_url,
                        checksum_url,
                        &tarball_path,
                        Some(&progress_callback),
                    )
                    .await?;
                println!("\r  Progress: 100.0%  ");
                println!("✓ Downloaded and verified v{}", update.latest_version);
            } else {
                downloader
                    .download_binary(&download_url, &tarball_path, Some(&progress_callback))
                    .await?;
                println!("\r  Progress: 100.0%  ");
                println!(
                    "✓ Downloaded v{} (no checksum available)",
                    update.latest_version
                );
            }

            // Extract binary from tarball
            let extract_dir = download_dir.join("extract");
            std::fs::create_dir_all(&extract_dir)?;

            let tarball_file = std::fs::File::open(&tarball_path)?;
            let tar_decoder = flate2::read::GzDecoder::new(tarball_file);
            let mut archive = tar::Archive::new(tar_decoder);
            archive.unpack(&extract_dir)?;

            let extracted_binary = extract_dir.join("cuttlefish-rs");

            println!();
            println!("Downloaded to: {}", extracted_binary.display());
            println!("To apply: cuttlefish update apply");
        }
        "apply" => {
            let download_dir = dirs::cache_dir()
                .unwrap_or_else(std::env::temp_dir)
                .join("cuttlefish-updates");
            let extracted_binary = download_dir.join("extract").join("cuttlefish-rs");

            if !extracted_binary.exists() {
                eprintln!("✗ No downloaded update found.");
                eprintln!("  Run 'cuttlefish update download' first.");
                std::process::exit(1);
            }

            let current_exe = std::env::current_exe()?;
            let backup_path = current_exe.with_extension("bak");

            println!("Applying update...");
            println!("  Current binary: {}", current_exe.display());
            println!("  New binary: {}", extracted_binary.display());
            println!("  Backup will be created at: {}", backup_path.display());
            println!();

            // Confirm before applying
            print!("Continue with update? [y/N] ");
            use std::io::Write;
            std::io::stdout().flush()?;

            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            if !input.trim().eq_ignore_ascii_case("y") {
                println!("Update cancelled.");
                return Ok(());
            }

            // Create backup
            std::fs::copy(&current_exe, &backup_path)?;
            println!("✓ Backup created");

            // Copy extracted binary
            std::fs::copy(&extracted_binary, &current_exe)?;

            // Make executable on Unix
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = std::fs::metadata(&current_exe)?.permissions();
                perms.set_mode(0o755);
                std::fs::set_permissions(&current_exe, perms)?;
            }

            println!("✓ Update applied successfully!");
            println!();
            println!("Please restart cuttlefish to use the new version.");
            println!("If issues occur, restore from: {}", backup_path.display());

            // Clean up extracted files
            std::fs::remove_dir_all(download_dir.join("extract")).ok();
            // Remove tarball too
            let binary_name = get_platform_binary_name();
            std::fs::remove_file(download_dir.join(format!("{}.tar.gz", binary_name))).ok();
        }
        _ => {
            eprintln!("Unknown update command: {}", subcommand);
            eprintln!("Available: check, download, apply");
            std::process::exit(1);
        }
    }

    Ok(())
}
