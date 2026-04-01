# 🐙 Cuttlefish

A portable, multi-agent, multi-model agentic coding platform built in Rust.

## Philosophy

Different AI models have different strengths. Cuttlefish routes each agent role to the optimal model via configurable category-based dispatch — inspired by the OmO/Sisyphus multi-model philosophy.

## Architecture

Each project gets:
- **Agent Orchestra**: Orchestrator → Coder → Critic workflow loop
- **Docker Sandbox**: Isolated container per project/context
- **Persistent Context**: SQLite with sliding window + summarization
- **Multi-Interface**: Nuxt WebUI, Discord bot, ratatui TUI — all via shared WebSocket

## Crate Structure

| Crate | Purpose |
|-------|---------|
| `cuttlefish-rs` (root) | Server binary entry point |
| `cuttlefish-core` | Error types, config, tracing, shared traits |
| `cuttlefish-db` | SQLite persistence (sqlx) |
| `cuttlefish-providers` | Model providers (Bedrock, Claude OAuth) |
| `cuttlefish-sandbox` | Docker container management (bollard) |
| `cuttlefish-vcs` | Git operations (git2) + GitHub API |
| `cuttlefish-agents` | Agent system (Orchestrator, Coder, Critic) |
| `cuttlefish-discord` | Discord bot (serenity) |
| `cuttlefish-api` | Axum HTTP/WebSocket server |
| `cuttlefish-tui` | ratatui terminal client binary |

## Getting Started

### Prerequisites
- Rust 1.94.0+ (`rustup install 1.94.0`)
- Docker (`docker --version`)
- SQLite (included via sqlx)

### Build
```bash
cargo build --release
```

### Configure
Copy the example config and edit:
```bash
cp cuttlefish.example.toml cuttlefish.toml
# Edit cuttlefish.toml with your settings
export CUTTLEFISH_API_KEY="your-api-key"
export DISCORD_BOT_TOKEN="your-discord-token"   # if using Discord
```

### Run
```bash
./target/release/cuttlefish-rs
```

## License

MIT
