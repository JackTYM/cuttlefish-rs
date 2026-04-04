//! Cuttlefish server — multi-agent, multi-model agentic coding platform.
//!
//! Entry point that wires together all crates and starts the HTTP/WebSocket server.

use cuttlefish_api::{build_app, routes::AppState};
use cuttlefish_core::{PricingConfig, TemplateRegistry, TimePeriod, UsageStats};
use cuttlefish_core::config::CuttlefishConfig;
use cuttlefish_tunnel::client::{TunnelClient, TunnelClientConfig};
use sqlx::SqlitePool;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::info;

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
            "--help" | "-h" => {
                print_help();
                return Ok(());
            }
            _ => {
                i += 1;
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
            }
        }),
    };

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

fn print_help() {
    println!("Cuttlefish - Multi-agent, multi-model agentic coding platform");
    println!();
    println!("USAGE:");
    println!("    cuttlefish-rs [OPTIONS] [COMMAND]");
    println!();
    println!("OPTIONS:");
    println!("    -c, --config <PATH>    Path to configuration file");
    println!("    -h, --help             Print this help message");
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
        }),
    };

    let db_url = format!("sqlite://{}?mode=rwc", config.database.path.to_string_lossy());
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
            println!("{:<12} {:>15} {:>15} {:>10}", "Date", "Input Tokens", "Output Tokens", "Requests");
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
            println!("{}", serde_json::to_string_pretty(&serde_json::json!({
                "project_id": summary.project_id,
                "period": format!("{:?}", summary.period),
                "total_requests": summary.total_requests,
                "total_input_tokens": summary.total_input_tokens,
                "total_output_tokens": summary.total_output_tokens,
                "total_cost_usd": summary.total_cost_usd,
                "by_provider": summary.by_provider,
            }))?);
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
            println!("{:<15} {:>15} {:>15} {:>10} {:>12}", "Provider", "Input Tokens", "Output Tokens", "Requests", "Est. Cost");
            println!("{:-<70}", "");
            
            let pricing = PricingConfig::with_defaults();
            let mut total_cost = 0.0;
            
            for p in &providers {
                let cost = estimate_provider_cost(&pricing, &p.provider, p.input_tokens as u64, p.output_tokens as u64);
                total_cost += cost;
                println!(
                    "{:<15} {:>15} {:>15} {:>10} ${:>10.4}",
                    p.provider, p.input_tokens, p.output_tokens, p.request_count, cost
                );
            }
            println!("{:-<70}", "");
            println!("{:<15} {:>15} {:>15} {:>10} ${:>10.4}", "TOTAL", "", "", "", total_cost);
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
    
    let mut csv = String::from("id,project_id,session_id,user_id,provider,model,input_tokens,output_tokens,request_type,latency_ms,success,error_type,created_at\n");
    
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
        println!("{:<15} {:<25} {:>10} {:>12}", "Provider", "Model", "Input", "Output");
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
        let input: f64 = args[3].parse().map_err(|_| anyhow::anyhow!("Invalid input price"))?;
        let output: f64 = args[4].parse().map_err(|_| anyhow::anyhow!("Invalid output price"))?;
        
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
            println!("  - [{}] {} — {}", decision.date, decision.decision, decision.rationale);
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
            println!("✓ Created branch '{}' at {}", branch.name, branch.created_at.format("%Y-%m-%d %H:%M"));
            println!("  Memory and decisions backed up to .cuttlefish/branches/{}/", name);
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
            println!(
                "{:<36} {:<20} {}",
                "ID", "Created", "Description"
            );
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
            let description = args.get(1).map(|s| s.as_str()).unwrap_or("Manual checkpoint");

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
            println!("  Created at: {}", checkpoint.created_at.format("%Y-%m-%d %H:%M:%S UTC"));
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
        checkpoints.iter().find(|c| c.id.as_str() == id || c.id.as_str().starts_with(&id)).cloned()
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
    println!("  Created: {}", target.created_at.format("%Y-%m-%d %H:%M:%S UTC"));
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
        let mut i = if args.first().is_some_and(|s| s == "config") { 1 } else { 0 };
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
        println!("  Auto-approve threshold: {:.2}", config.auto_approve_threshold);
        println!("  Prompt threshold:       {:.2}", config.prompt_threshold);
        println!();
        println!("Actions with confidence >= {:.2} are auto-approved.", config.auto_approve_threshold);
        println!("Actions with confidence >= {:.2} but < {:.2} prompt for approval.", config.prompt_threshold, config.auto_approve_threshold);
        println!("Actions with confidence < {:.2} are blocked.", config.prompt_threshold);

        return Ok(());
    }

    eprintln!("Unknown safety command: {}", args[0]);
    eprintln!("Available: config");
    std::process::exit(1);
}
