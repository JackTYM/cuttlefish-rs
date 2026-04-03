<p align="center">
  <img src=".github/assets/logo.svg" alt="Cuttlefish Logo" width="200"/>
</p>

<h1 align="center">🐙 Cuttlefish</h1>

<p align="center">
  <strong>A portable, multi-agent, multi-model agentic coding platform built in Rust</strong>
</p>

<p align="center">
  <a href="#philosophy">Philosophy</a> •
  <a href="#features">Features</a> •
  <a href="#architecture">Architecture</a> •
  <a href="#installation">Installation</a> •
  <a href="#configuration">Configuration</a> •
  <a href="#usage">Usage</a> •
  <a href="#inspirations">Inspirations</a>
</p>

---

## Philosophy

**Different AI models have different strengths.** A model optimized for speed isn't the same as one optimized for deep reasoning. A model trained for code generation excels at different tasks than one fine-tuned for code review.

Cuttlefish embraces this reality with **category-based model routing**: each agent role can be mapped to the optimal model for its task. The Orchestrator might use a fast model for quick task decomposition, while the Coder uses a powerful model for complex implementations, and the Critic uses yet another model optimized for analytical review.

### Core Principles

1. **Multi-Model by Design**: Route each agent to the model that does its job best
2. **Isolated Execution**: Every project runs in its own Docker sandbox—no cross-contamination
3. **Interface Agnostic**: Same agent system accessible via Discord, WebUI, or TUI
4. **Zero Unsafe Code**: The entire codebase is `#![deny(unsafe_code)]`—memory safety guaranteed
5. **Self-Developing**: Cuttlefish can update itself via GitHub Actions

### Why "Cuttlefish"?

The cuttlefish is nature's ultimate adapter. With millions of chromatophores in their skin, cuttlefish can change color, pattern, and texture in milliseconds—matching their environment or communicating complex messages through dynamic displays.

**This platform works the same way:**

- **Multi-Colored = Multi-Agent**: Just as a cuttlefish displays many colors simultaneously through specialized chromatophores, Cuttlefish deploys multiple specialized agents working in concert. Each agent has its own "color"—its own expertise, tools, and optimal model.

- **Rapid Adaptation = Dynamic Model Routing**: A cuttlefish switches from camouflage to warning display to mating pattern instantly. Cuttlefish (the platform) switches between fast models for quick tasks and powerful models for deep reasoning—adapting its intelligence to match the challenge.

- **Camouflage = Seamless Integration**: Cuttlefish blend into any environment. This platform integrates into your existing workflow—Discord for team chat, WebUI for visual work, TUI for SSH sessions. Same agents, any interface.

- **Giant Brain = Distributed Intelligence**: Cuttlefish have the largest brain-to-body ratio of any invertebrate. This platform distributes intelligence across specialized agents, each contributing their unique cognitive strength to the whole.

- **Self-Evolution**: Cuttlefish don't just adapt to their environment—their species evolves rapidly. This platform can update itself, fix its own bugs, and deploy new versions autonomously. True self-developing software.

```
    🐙 The Cuttlefish Way
    
    Task arrives → Orchestrator assesses → Routes to optimal agent
                                              ↓
    ┌─────────────┬─────────────┬─────────────┬─────────────┐
    │   Planner   │    Coder    │   Critic    │   DevOps    │
    │  ultrabrain │    deep     │    high     │    high     │
    │  "strategy" │   "build"   │  "review"   │  "deploy"   │
    └─────────────┴─────────────┴─────────────┴─────────────┘
                              ↓
    Different models, different strengths, one unified system
```

---

## Features

### 🤖 Multi-Agent System

Cuttlefish implements a **Planner → Coder → Critic** workflow loop inspired by the Sisyphus/Ultraworker pattern:

| Agent | Role | Category |
|-------|------|----------|
| **Orchestrator** | Coordinates agents, manages lifecycle | `deep` |
| **Planner** | Creates strategic implementation plans | `ultrabrain` |
| **Coder** | Writes code, runs builds, executes tests | `deep` |
| **Critic** | Reviews code, runs tests, approves/rejects | `unspecified-high` |
| **Explorer** | Searches codebases, finds patterns | `quick` |
| **Librarian** | Finds documentation, retrieves external resources | `quick` |
| **DevOps** | Handles builds, deployments, CI/CD | `unspecified-high` |

### 🎯 Category-Based Model Routing

Configure which models serve which task categories:

| Category | Use Case | Recommended Model |
|----------|----------|-------------------|
| `ultrabrain` | Hard logic, architecture | claude-opus-4-6 |
| `deep` | Complex autonomous work | gpt-5.4 or claude-sonnet-4-6 |
| `quick` | Simple fast tasks | claude-haiku-4-5 |
| `visual` | Frontend, UI/UX | gemini-2.0-flash |
| `unspecified-high` | General higher effort | claude-sonnet-4-6 |
| `unspecified-low` | General lower effort | claude-haiku-4-5 |

### 🐳 Docker Sandboxes

Each project gets an isolated Docker container with:
- Configurable CPU limits (default: 2 cores)
- Configurable memory limits (default: 2GB)
- Configurable disk limits (default: 10GB)
- Network access for package installation
- Automatic cleanup of stale containers

### 🔌 Multi-Interface Access

Access your agents from anywhere:

- **Discord Bot**: Channel-per-project management, slash commands, real-time updates
- **Web UI**: Nuxt-based interface with chat, build logs, and file diffs
- **TUI**: ratatui terminal client for remote development over SSH

### 📦 Model Providers

Cuttlefish supports 11 model providers out of the box:

| Provider | Auth Method | Models | Best For |
|----------|-------------|--------|----------|
| **Anthropic** | `ANTHROPIC_API_KEY` | claude-opus-4-6, claude-sonnet-4-6, claude-haiku-4-5 | General, planning |
| **OpenAI** | `OPENAI_API_KEY` | gpt-5.4, gpt-5-nano, gpt-4o | Coding, reasoning |
| **Google Gemini** | `GOOGLE_API_KEY` | gemini-2.0-flash, gemini-1.5-pro | Visual, frontend |
| **Moonshot (Kimi)** | `MOONSHOT_API_KEY` | kimi-k2.5, moonshot-v1-128k | Claude-like tasks |
| **Zhipu (GLM)** | `ZHIPU_API_KEY` | glm-4-flash, glm-4 | Broad tasks |
| **MiniMax** | `MINIMAX_API_KEY` + `MINIMAX_GROUP_ID` | abab6.5s-chat, abab6.5t-chat | Fast utility |
| **xAI (Grok)** | `XAI_API_KEY` | grok-2, grok-beta | Code search |
| **AWS Bedrock** | IAM / Access Keys | Claude family via Bedrock | Enterprise |
| **Ollama** | None (local) | Any GGUF model | Privacy, offline |
| **Claude OAuth** | PKCE Flow | Claude (via claude.ai) | Personal accounts |
| **ChatGPT OAuth** | Bearer Token | GPT-4o, GPT-4 | Personal accounts |

See [docs/providers/](docs/providers/) for setup guides.

### 🔧 Additional Features

- **Hashline Editing**: Surgical file edits using content-addressable line hashes (5-char xxHash)
- **Context Management**: Sliding window with automatic summarization for long conversations
- **GitHub Integration**: Clone, branch, commit, push, create PRs, monitor Actions
- **Template System**: Project scaffolding from markdown-based templates
- **Self-Update**: Pull new binaries from GitHub Releases and restart automatically

---

## Architecture

### Crate Structure

Cuttlefish is organized as a Cargo workspace with focused, single-responsibility crates:

```
cuttlefish-rs/
├── src/main.rs                    # Server binary entry point
└── crates/
    ├── cuttlefish-core/           # Error types, config, tracing, shared traits
    ├── cuttlefish-db/             # SQLite persistence (sqlx, WAL mode)
    ├── cuttlefish-providers/      # Model providers (Bedrock, Claude OAuth)
    ├── cuttlefish-sandbox/        # Docker container management (bollard)
    ├── cuttlefish-vcs/            # Git operations (git2) + GitHub API
    ├── cuttlefish-agents/         # Agent system (Orchestrator, Coder, Critic)
    ├── cuttlefish-discord/        # Discord bot (serenity)
    ├── cuttlefish-api/            # Axum HTTP/WebSocket server
    └── cuttlefish-tui/            # ratatui terminal client
```

### Dependency Flow

```
cuttlefish-core (no internal deps)
       ↑
cuttlefish-db, cuttlefish-providers, cuttlefish-sandbox, cuttlefish-vcs
       ↑
cuttlefish-agents (depends on all above)
       ↑
cuttlefish-discord, cuttlefish-api, cuttlefish-tui (interface crates)
       ↑
cuttlefish-rs (root binary — wires everything)
```

### Request Flow

```
User (Discord/WebUI/TUI)
        │
        ▼
   cuttlefish-api (WebSocket)
        │
        ▼
   Orchestrator Agent
        │
   ┌────┴────┐
   ▼         ▼
Coder    Critic
   │         │
   └────┬────┘
        ▼
   Docker Sandbox
        │
        ▼
   GitHub (push)
```

---

## Installation

### Quick Start (Recommended)

```bash
# Download and run the guided installer
curl -sSL https://raw.githubusercontent.com/JackTYM/cuttlefish-rs/master/install.sh | bash
```

The installer will:
1. **Detect your platform** (Linux, macOS, or WSL) and configure accordingly
2. **Choose deployment mode**:
   - **Systemd Service** (recommended) — Cuttlefish runs as a system service with Docker sandbox isolation per project
   - **Docker Container** — Simpler setup, no systemd required (no project sandbox isolation)
3. Check and install dependencies (Rust, Docker, Git) for your platform
4. Guide you through **provider selection** (choose which AI providers to configure)
5. Set up API keys for your selected providers
6. For **AWS Bedrock**: select from all regional model variants (default, US, EU, global cross-region inference)
7. Optionally configure **custom model IDs and API endpoints** for any provider
8. Configure server, database, and sandbox settings
9. Optionally set up Discord bot integration
10. Build and install Cuttlefish from source
11. Start the service (systemd mode) or provide Docker run command

### Manual Installation

If you prefer manual setup:

```bash
# Prerequisites: Rust 1.94.0+, Docker, Git
rustup install 1.94.0

# Clone and build
git clone https://github.com/JackTYM/cuttlefish-rs.git
cd cuttlefish-rs
cargo build --release

# Configure
cp cuttlefish.example.toml cuttlefish.toml
# Edit cuttlefish.toml with your settings

# Set API keys for your providers (examples)
export CUTTLEFISH_API_KEY="your-secure-api-key"
export ANTHROPIC_API_KEY="sk-ant-..."      # For Anthropic
export OPENAI_API_KEY="sk-..."              # For OpenAI
export GOOGLE_API_KEY="..."                 # For Google Gemini
# See docs/providers/ for all provider options

# Run
./target/release/cuttlefish-rs
```

### Docker Deployment

```bash
# Build the Docker image
docker build -t cuttlefish .

# Run with configuration
docker run -d \
  -v /var/run/docker.sock:/var/run/docker.sock \
  -v $(pwd)/cuttlefish.toml:/etc/cuttlefish/cuttlefish.toml \
  -e CUTTLEFISH_API_KEY="your-key" \
  -e AWS_ACCESS_KEY_ID="..." \
  -e AWS_SECRET_ACCESS_KEY="..." \
  -p 8080:8080 \
  cuttlefish
```

---

## Configuration

Cuttlefish uses TOML configuration with environment variable overrides for secrets.

### Configuration Files

| File | Purpose |
|------|---------|
| `cuttlefish.toml` | Main configuration (gitignored) |
| `cuttlefish.example.toml` | Example with documented defaults |
| `/etc/cuttlefish/cuttlefish.toml` | System-wide config (production) |
| `~/.config/cuttlefish/config.toml` | User config (alternative) |

### Environment Variables

**Required:**
```bash
CUTTLEFISH_API_KEY      # API key for WebUI/TUI authentication
```

**For AWS Bedrock:**
```bash
AWS_ACCESS_KEY_ID       # AWS credentials
AWS_SECRET_ACCESS_KEY
AWS_DEFAULT_REGION      # e.g., us-east-1
```

**For Discord:**
```bash
DISCORD_BOT_TOKEN       # Discord bot token
```

### Full Configuration Reference

```toml
# =============================================================================
# Server Configuration
# =============================================================================
[server]
host = "127.0.0.1"      # Bind address (use 0.0.0.0 for external access)
port = 8080             # HTTP/WebSocket port

# =============================================================================
# Database Configuration
# =============================================================================
[database]
path = "cuttlefish.db"  # SQLite database path (WAL mode enabled)

# =============================================================================
# Docker Sandbox Configuration
# =============================================================================
[sandbox]
docker_socket = "unix:///var/run/docker.sock"
memory_limit_mb = 2048  # Per-container memory limit
cpu_limit = 2.0         # Per-container CPU limit (cores)
disk_limit_gb = 10      # Per-container disk limit
max_concurrent = 5      # Maximum concurrent sandboxes

# =============================================================================
# Model Providers
# =============================================================================

# AWS Bedrock (recommended for production)
[providers.claude]
provider_type = "bedrock"
model = "anthropic.claude-sonnet-4-6-20260101-v1:0"
region = "us-east-1"

# Fast model for quick tasks
[providers.claude-fast]
provider_type = "bedrock"
model = "anthropic.claude-haiku-4-5-20260101-v1:0"
region = "us-east-1"

# Powerful model for complex reasoning
[providers.claude-opus]
provider_type = "bedrock"
model = "anthropic.claude-opus-4-6-20260101-v1:0"
region = "us-east-1"

# =============================================================================
# Agent Configuration
# =============================================================================

# Map agents to categories (which then map to providers)
[agents.orchestrator]
category = "deep"               # Uses claude (Sonnet)

[agents.coder]
category = "deep"               # Uses claude (Sonnet)

[agents.critic]
category = "unspecified-high"   # Can use a different model

# Optional: Override the model directly
# [agents.coder]
# category = "deep"
# model_override = "anthropic.claude-opus-4-6-20260101-v1:0"

# =============================================================================
# Discord Configuration (Optional)
# =============================================================================
[discord]
token_env_var = "DISCORD_BOT_TOKEN"  # Env var containing the token
guild_ids = []                        # Empty = global commands, or specify guild IDs
```

### Model Recommendations by Task

| Task Type | Recommended Model | Why |
|-----------|-------------------|-----|
| **Planning & Orchestration** | Claude Sonnet 4.6 | Good balance of speed and reasoning |
| **Code Generation** | Claude Sonnet 4.6 or Opus 4.6 | Strong at code, handles complexity |
| **Code Review** | Claude Sonnet 4.6 | Analytical, catches issues |
| **Quick Searches** | Claude Haiku 4.5 | Fast, cheap, good enough for simple tasks |
| **Architecture Decisions** | Claude Opus 4.6 | Deepest reasoning capability |
| **Frontend/UI Work** | Gemini 2.0 Flash | Strong visual understanding |

---

## Usage

### Starting the Server

```bash
# Development
cargo run

# Production
./target/release/cuttlefish-rs

# With systemd (after install.sh)
sudo systemctl start cuttlefish
```

### Endpoints

| Endpoint | Purpose |
|----------|---------|
| `http://localhost:8080/health` | Health check |
| `ws://localhost:8080/ws` | WebSocket for clients |
| `http://localhost:8080/api/...` | REST API |

### TUI Client

```bash
# Build the TUI
cargo build --release -p cuttlefish-tui

# Connect to server
./target/release/cuttlefish-tui \
  --server ws://localhost:8080 \
  --api-key "$CUTTLEFISH_API_KEY"
```

### Discord Bot

1. Create a Discord application at https://discord.com/developers
2. Create a bot and copy the token
3. Set `DISCORD_BOT_TOKEN` environment variable
4. Enable Discord in config
5. Invite bot to your server with appropriate permissions
6. Use `/cuttlefish` slash commands

---

## Inspirations

Cuttlefish stands on the shoulders of giants. This project draws inspiration from several innovative AI coding platforms:

### OmO / Sisyphus Labs

The **OhMyOpenCode (OmO)** project and its **Sisyphus** agent system pioneered many concepts used in Cuttlefish:

- **Category-based model routing**: Different tasks routed to different models based on task category (visual, deep, quick, ultrabrain)
- **Multi-agent orchestration**: Planner → Coder → Critic workflow loop (Ultraworker pattern)
- **Hashline editing**: Content-addressable line hashes for surgical file edits
- **Agent dispatch with skills**: Loading specialized knowledge per-agent

*"Different AI models have different strengths"* — the core philosophy that Cuttlefish inherits.

### OpenClaw

**OpenClaw** contributed architectural patterns:

- **Gateway control plane**: Multi-channel message routing
- **Skills platform**: Bundled, managed, and workspace-level skills
- **Multi-interface design**: Single backend, multiple frontends

### Other Influences

- **Moltis**: Large Rust workspace architecture (46 crates), zero unsafe code, SQLite patterns
- **Hermes Agent**: Personality systems, dynamic agent switching, toolset distributions
- **IronClaw**: Sandbox isolation concepts, capability-based permissions

---

## Development

### Building

```bash
# Debug build
cargo build --workspace

# Release build
cargo build --release --workspace

# Run tests
cargo test --workspace

# Run clippy
cargo clippy --workspace -- -D warnings

# Format code
cargo fmt --all
```

### Project Conventions

- **Edition**: Rust 2024
- **MSRV**: 1.94.0
- **No unsafe**: `#![deny(unsafe_code)]` workspace-wide
- **No unwrap**: `#![deny(clippy::unwrap_used)]` in library crates
- **Errors**: Use `thiserror` for library errors, `anyhow` for binaries
- **Logging**: `tracing` macros only, no `println!`

### Contributing

1. Fork the repository
2. Create a feature branch
3. Write tests (TDD preferred)
4. Ensure `cargo test --workspace` passes
5. Ensure `cargo clippy --workspace -- -D warnings` is clean
6. Submit a pull request

---

## Roadmap

### v1.0 (Current)
- [x] Core agent system (Orchestrator, Coder, Critic)
- [x] AWS Bedrock provider
- [x] Claude OAuth provider
- [x] ChatGPT OAuth provider
- [x] OpenAI API provider
- [x] Google Gemini provider
- [x] Moonshot/Kimi provider
- [x] Zhipu/GLM provider
- [x] MiniMax provider
- [x] xAI/Grok provider
- [x] Ollama provider
- [x] Docker sandboxes
- [x] Discord bot
- [x] Web UI (Nuxt)
- [x] TUI client
- [x] GitHub integration
- [x] Hashline editing
- [x] Planner agent
- [x] Explorer agent
- [x] Librarian agent
- [x] DevOps agent

### v1.1 (Planned)
- [ ] WASM sandbox option
- [ ] Hook system for customization
- [ ] Skill-embedded MCPs

### v2.0 (Future)
- [ ] Code editor in WebUI
- [ ] JetBrains plugin
- [ ] Self-modifying prompts
- [ ] RAG with vector embeddings

---

## License

MIT License — see [LICENSE](LICENSE) for details.

---

<p align="center">
  Made with 🦀 and 🐙
</p>
