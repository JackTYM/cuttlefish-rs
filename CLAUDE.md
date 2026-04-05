# Cuttlefish - Developer Guide for AI Assistants

## Project Vision

Cuttlefish is a **persistent, device-agnostic agentic coding platform**. Users can start a project on one device, switch to another, and continue exactly where they left off - including running background processes. It runs on a server and is accessible via multiple client interfaces (WebUI, TUI, Discord, etc.).

**Core Promise**: Multi-agent, multi-model coding assistance with:
- **Project Isolation**: Every project runs in its own Docker container
- **Session Persistence**: Agent memory and state persist between connections
- **Interface Agnostic**: Same backend, any frontend (Discord, WebUI, TUI, etc.)
- **BYOK Model**: Users provide their own API keys - no proxying through Cuttlefish servers

## Project Identity

- **Language**: Rust 2024 edition (MSRV 1.94.0)
- **Architecture**: Cargo workspace with 11 crates
- **Philosophy**: Traits before implementations, zero unsafe code
- **Current Version**: 0.2.0

## Crate Overview

```
cuttlefish-rs/
├── src/main.rs                    # Server binary - wires everything together
├── cuttlefish-web/                # Nuxt 3 WebUI (Vue + TypeScript)
└── crates/
    ├── cuttlefish-core/           # Error types, config, traits, shared types
    ├── cuttlefish-db/             # SQLite persistence (sqlx, WAL mode)
    ├── cuttlefish-providers/      # Model providers (11 providers supported)
    ├── cuttlefish-sandbox/        # Docker container management (bollard)
    ├── cuttlefish-vcs/            # Git operations (git2) + GitHub API
    ├── cuttlefish-agents/         # Agent system (Orchestrator, Coder, Critic, etc.)
    ├── cuttlefish-discord/        # Discord bot (serenity)
    ├── cuttlefish-api/            # Axum HTTP/WebSocket server
    ├── cuttlefish-tui/            # ratatui terminal client
    └── cuttlefish-tunnel/         # WebSocket reverse tunnel for remote access
```

## Dependency Flow

```
cuttlefish-core (no deps on other crates)
       ↑
cuttlefish-db, cuttlefish-providers, cuttlefish-sandbox, cuttlefish-vcs, cuttlefish-tunnel
       ↑
cuttlefish-agents (depends on above)
       ↑
cuttlefish-discord, cuttlefish-api, cuttlefish-tui (interface crates)
       ↑
cuttlefish-rs (root binary — wires everything)
```

---

## CRITICAL: Current Implementation State

### What's Well-Implemented

| Component | Location | Status |
|-----------|----------|--------|
| Database layer | `cuttlefish-db` | Complete - projects, conversations, templates, auth, sessions, usage tracking |
| Core types & config | `cuttlefish-core` | Complete - traits, error types, config parsing |
| Model providers | `cuttlefish-providers` | Complete - 11 providers (Anthropic, OpenAI, Bedrock, Google, etc.) |
| Agent architecture | `cuttlefish-agents` | Complete - Orchestrator, Coder, Critic, Planner, Explorer, Librarian, DevOps |
| Sandbox infrastructure | `cuttlefish-sandbox` | Complete - Docker lifecycle, volumes, snapshots, cleanup |
| Safety system | `cuttlefish-agents/safety` | Complete - confidence scoring, action gates, diff preview |
| Memory system | `cuttlefish-agents/memory` | Complete - project memory, decision log, branching |
| API route structure | `cuttlefish-api` | Routes exist but many are incomplete |
| Usage tracking | `cuttlefish-db/usage` | Complete - token counting, cost calculation, alerts |

### Recently Fixed (Phase 2 Integration)

The following critical integrations have been implemented:

#### 1. WebSocket Now Routes to Agents
- **File**: `crates/cuttlefish-api/src/ws.rs`
- **Status**: FIXED - WebSocket chat messages now invoke the `WorkflowEngine` which runs the Orchestrator -> Coder -> Critic loop
- **How it works**: `execute_workflow()` creates a workflow engine with a provider and executes the full agent pipeline

#### 2. WebSocket Protocol Extended
- **Status**: FIXED - Server now sends all message types WebUI expects
- **New message types**: `PendingApproval`, `ApprovalResolved`, `LogEntry`
- **Log streaming**: Agent activity is streamed via `LogEntry` messages during workflow execution

#### 3. Provider Testing Works
- **File**: `crates/cuttlefish-api/src/system_routes.rs`
- **Status**: FIXED - `test_provider()` now makes a real API call to verify the provider works

#### 4. Providers Initialized from Config
- **File**: `src/main.rs`
- **Status**: FIXED - Providers are loaded from `cuttlefish.toml` at startup
- **Supported**: anthropic, openai, bedrock, google, ollama

### Additional Integrations (Phase 3+)

#### 5. TUI Now Functional
- **File**: `crates/cuttlefish-tui/src/main.rs`
- **Status**: DONE - Full TUI with WebSocket connection, keyboard input, view switching
- **Features**: Chat view, diff view, log view, mascot view, graceful disconnection handling

#### 6. Sandbox API Routes Implemented
- **File**: `crates/cuttlefish-api/src/sandbox_routes.rs`
- **Status**: DONE - Real Docker sandbox integration
- **Endpoints**: Create, list, delete sandboxes; execute commands; health check; snapshots
- **Features**: Resource presets (light/standard/heavy), graceful degradation if Docker unavailable

#### 7. Discord Bot Startup
- **File**: `crates/cuttlefish-discord/src/bot.rs`
- **Status**: DONE - Event handler with command registration and routing
- **Integration**: Spawned from main.rs when Discord config present

#### 8. Safety Workflow Integration
- **File**: `crates/cuttlefish-api/src/approval_registry.rs` (NEW)
- **File**: `crates/cuttlefish-api/src/ws.rs` (UPDATED)
- **Status**: DONE - Approval registry with async wait, WebSocket handlers wired
- **Features**:
  - `ApprovalRegistry` tracks pending approvals with oneshot channels for resolution
  - `execute_with_safety()` evaluates actions through `ActionGate` and waits for approval
  - WebSocket `Approve`/`Reject` handlers resolve pending approvals
  - Timeout handling with configurable duration (default 5 minutes)

#### 9. Architecture Documentation
- **Directory**: `docs/architecture/`
- **Status**: DONE - Protocol specs and architecture overview created
- **Files**:
  - `overview.md` - System diagram, request lifecycle, data flow
  - `websocket-protocol.md` - Full WebSocket message specification
  - `rest-api.md` - REST API endpoint documentation
  - `agent-contracts.md` - Agent roles, workflow, and tool definitions

#### 10. Integration Tests
- **Directory**: `tests/integration/`
- **Status**: DONE - End-to-end tests for API and approval workflow
- **Tests**:
  - `health_test.rs` - Health endpoint and 404 handling
  - `approval_test.rs` - Approval registry workflow tests

### What's Still Incomplete

#### 1. Logs Page Verification
- **File**: `cuttlefish-web/pages/logs.vue`
- **Status**: Needs verification that WebSocket log messages are received and displayed correctly

---

## API Contracts (Source of Truth)

When implementing features that span client/server, these contracts MUST be documented first.

### WebSocket Protocol

**Server Messages** (`crates/cuttlefish-api/src/ws.rs`):
```rust
pub enum ServerMessage {
    Response { project_id, agent, content },
    BuildLog { project_id, line },
    Diff { project_id, patch },
    PendingApproval { action_id, project_id, action_type, description, confidence, ... },
    ApprovalResolved { action_id },
    LogEntry { id, timestamp, agent, action, level, project, context?, stack_trace? },
    Pong,
    Error { message },
}
```

**Client Messages**:
```rust
pub enum ClientMessage {
    Chat { project_id, content },
    Ping,
    Approve { action_id },
    Reject { action_id, reason? },
    Subscribe { project_id },
    Unsubscribe { project_id },
}
```

### REST API Routes

**Implemented and Working**:
- `GET /health` - Health check
- `GET /ws` - WebSocket upgrade
- `GET /api/templates` - List templates
- `GET /api/templates/{name}` - Get template
- `POST /api/templates/fetch` - Fetch remote template
- `GET /api/projects` - List projects
- `POST /api/projects` - Create project
- `GET /api/projects/{id}` - Get project
- `DELETE /api/projects/{id}` - Cancel project
- `POST /api/projects/{id}/archive` - Archive project
- `GET /api/system/config` - Get config
- `PUT /api/system/config` - Update config
- `GET /api/system/status` - Get status
- `POST /api/system/api-key/regenerate` - Regenerate API key

**Recently Fixed**:
- `POST /api/system/providers/{id}/test` - Now tests actual provider connection

**Remaining (WebUI expects via REST)**:
- Safety approval routes (currently handled via WebSocket)
- Log history endpoint (currently streamed via WebSocket)

---

## Non-Negotiable Code Rules

### Code Quality
- `#![deny(unsafe_code)]` workspace-wide - NEVER add unsafe
- `#![deny(clippy::unwrap_used)]` on lib crates - use `?` or `expect("reason")`
- `#![warn(missing_docs)]` - every public item needs `///` documentation
- No `println!` debugging - use `tracing::debug!`, `tracing::info!`, etc.
- No global mutable state - no `lazy_static!`, no `once_cell::sync::Lazy`

### Error Handling
- Library crates use `thiserror` - define specific error types
- Binary crates (main.rs) may use `anyhow` for top-level error handling
- Error types live in `cuttlefish-core::error`

### Architecture
- Traits defined in `cuttlefish-core::traits` - no trait definitions in impl crates
- Concrete implementations depend on trait interfaces, not concrete types
- All I/O behind traits for testability

---

## Development Workflow

1. **Write the test first** (TDD)
2. Implement minimally to make the test pass
3. Run: `cargo clippy -p <crate> -- -D warnings` - must be clean
4. Run: `cargo test -p <crate>` - all pass
5. Commit: `<type>(<crate>): <description>`

### Commit Convention
Format: `<type>(<crate>): <description>`
Types: `feat`, `fix`, `refactor`, `test`, `docs`, `chore`
Example: `feat(api): wire WebSocket to orchestrator agent`

### Testing
- Unit tests: in `#[cfg(test)]` blocks at bottom of each module
- Integration tests requiring real services: behind `#[cfg(feature = "integration")]`
- Mock implementations live alongside the trait (e.g., `MockModelProvider`)
- All async tests use `#[tokio::test]`

---

## WebUI Development

The WebUI is a Nuxt 3 + Vue 3 + TypeScript application in `cuttlefish-web/`.

### Key Files
- `pages/*.vue` - Page components
- `composables/use*.ts` - Shared composables (API clients, WebSocket)
- `components/` - Reusable components

### API Base
Configured via `NUXT_PUBLIC_API_BASE` env var or `runtimeConfig.public.apiBase`.
Default: relative URLs (same host as WebUI).

### Build & Serve
```bash
cd cuttlefish-web
npm install
npm run build           # Production build to .output/
npm run dev             # Development server
```

The Rust server serves the built WebUI from `.output/public/` when available.

---

## Key Configuration

### Environment Variables
```bash
CUTTLEFISH_API_KEY      # API key for WebUI/TUI authentication
CUTTLEFISH_JWT_SECRET   # JWT signing secret (defaults to API key)
CUTTLEFISH_PROJECTS_DIR # Base directory for project files
DISCORD_BOT_TOKEN       # Discord bot token
RUST_LOG                # Log level (e.g., info, debug, trace)

# Provider-specific
ANTHROPIC_API_KEY
OPENAI_API_KEY
GOOGLE_API_KEY
AWS_ACCESS_KEY_ID
AWS_SECRET_ACCESS_KEY
AWS_DEFAULT_REGION
# ... etc for other providers
```

### Config Files
- `cuttlefish.toml` - Main config (gitignored)
- `cuttlefish.example.toml` - Example with documented defaults
- `/etc/cuttlefish/cuttlefish.toml` - System-wide (production)
- `~/.config/cuttlefish/config.toml` - User config (alternative)

---

## Architecture Patterns

### Agent System
The agent system uses a Planner -> Coder -> Critic loop:

1. **Orchestrator** receives task, assesses scope
2. **Planner** creates implementation plan (for complex tasks)
3. **Coder** implements the plan
4. **Critic** reviews and approves/rejects
5. Loop until Critic approves or max iterations

Each agent has a **category** that maps to a model:
- `ultrabrain` - Hardest logic (claude-opus)
- `deep` - Complex work (claude-sonnet, gpt-4)
- `quick` - Fast tasks (claude-haiku)
- `visual` - Frontend/UI (gemini)
- `unspecified-high/low` - General fallbacks

### Message Bus
Agents communicate via `TokioMessageBus` (`cuttlefish-agents/bus.rs`).
Messages have `from`, `to`, `kind`, and `payload` fields.

### Safety Gates
Actions above a confidence threshold auto-approve.
Actions in the middle prompt the user.
Actions below are blocked.

---

## Priority Fix Order

When working on this project, tackle issues in this order:

1. ~~**Wire WebSocket to Agents**~~ DONE
   - WebSocket now routes to WorkflowEngine
   - Providers initialized from config at startup

2. ~~**Add Missing WebSocket Message Types**~~ DONE
   - `PendingApproval`, `ApprovalResolved`, `LogEntry` added
   - Log streaming implemented during workflow execution

3. ~~**Implement TUI**~~ DONE
   - Full terminal client with WebSocket connection
   - Chat, diff, log, and mascot views with keyboard navigation

4. ~~**Add Sandbox API Routes**~~ DONE
   - Real Docker integration with DockerSandboxLifecycle
   - Create, execute, snapshot, health check endpoints

5. ~~**Test Provider Connection**~~ DONE
   - Provider testing now works via actual API call

6. ~~**Discord Bot Startup**~~ DONE
   - Event handler with slash command registration
   - Spawns from main.rs when configured

7. ~~**Wire Safety Approval to Workflow**~~ DONE
   - ApprovalRegistry tracks pending approvals with async wait
   - execute_with_safety() evaluates actions through ActionGate
   - WebSocket Approve/Reject handlers resolve pending approvals

8. ~~**Architecture Documentation**~~ DONE
   - Created `/docs/architecture/` with protocol specs
   - overview.md, websocket-protocol.md, rest-api.md, agent-contracts.md

9. ~~**Integration Tests**~~ DONE
   - Created `tests/integration/` with end-to-end tests
   - health_test.rs, approval_test.rs

10. **Logs Page Verification** (Next priority)
    - Verify WebUI logs page displays WebSocket log messages correctly
   - Currently approvals are acknowledged but don't block execution

8. **Create Architecture Documentation**
   - `/docs/architecture/` with protocol specs and diagrams

9. **Add Integration Tests**
   - End-to-end tests in `tests/integration/`

---

## File Quick Reference

| Need to... | Look at... |
|------------|------------|
| Add WebSocket message type | `crates/cuttlefish-api/src/ws.rs` |
| Add REST API route | `crates/cuttlefish-api/src/*.rs` |
| Modify agent behavior | `crates/cuttlefish-agents/src/*.rs` |
| Add database table | `crates/cuttlefish-db/src/lib.rs` (migrations) |
| Add provider | `crates/cuttlefish-providers/src/*.rs` |
| Modify sandbox | `crates/cuttlefish-sandbox/src/*.rs` |
| WebUI pages | `cuttlefish-web/pages/*.vue` |
| WebUI API calls | `cuttlefish-web/composables/*.ts` |
