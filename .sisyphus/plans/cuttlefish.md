# Cuttlefish — Multi-Agent, Multi-Model Agentic Coding Platform

## TL;DR

> **Quick Summary**: Build a portable, multi-agent agentic coding platform in Rust where different AI models serve specialized agent roles (Orchestrator/Coder/Critic), accessed via WebUI (Nuxt), Discord, and TUI, executing code in Docker sandboxes, managing GitHub repos end-to-end, with self-development capabilities.
> 
> **Deliverables**:
> - Cargo workspace with 10+ crates
> - Multi-model provider layer (Bedrock, Claude Code OAuth, ChatGPT OAuth)
> - Agent system (Orchestrator, Coder, Critic — expandable to 7+ roles)
> - Docker sandbox per project/context
> - Discord bot with channel-per-project management
> - Nuxt WebUI with real-time chat, build logs, file diffs
> - ratatui TUI client for remote development
> - GitHub integration (clone, branch, commit, push, PR, Actions monitoring)
> - Template system (.MD-based project standards)
> - Self-update via GitHub Actions
> - Reverse proxy for dev server exposure
> 
> **Estimated Effort**: XL (20+ weeks, 40+ tasks)
> **Parallel Execution**: YES — 10 waves
> **Critical Path**: Workspace → Core Traits → DB → Bedrock Provider → Docker Sandbox → Agent System → Discord → WebUI

---

## Context

### Original Request
Build "Cuttlefish" — a portable, multi-agent, multi-model agentic coding tool in Rust inspired by OpenClaw, Hermes, OmO, Sisyphus Labs, Moltis, and IronClaw. Accessible from Discord, WebUI, and TUI. Each agent personality runs on a different model matched to its strengths. Projects get isolated Docker sandboxes. Full GitHub workflow support. Self-developing.

### Interview Summary
**Key Discussions**:
- **Deployment**: Runs in its own KVM, portable to run anywhere. Linux-only target.
- **Sandboxing**: Docker container per project/context window
- **MVP Priority**: Discord → Agent → GitHub core loop
- **Storage**: SQLite (WAL mode) + filesystem
- **Discord**: Multi-server support
- **Model Routing**: OmO-style category-based (visual/deep/quick/ultrabrain) — user configures which models serve which roles via config
- **GitHub**: Both PAT (quick setup) + GitHub App (production)
- **Templates**: .MD files describing project standards — agent reads and follows them
- **Self-Update**: GitHub Actions builds → Cuttlefish pulls new binary → restarts
- **Client Protocol**: WebSocket for all client ↔ server communication
- **WebUI**: Nuxt frontend — primary rich interface for chat, diffs, builds
- **TUI Auth**: API key
- **Agent Roles**: Full Sisyphus-style suite (Orchestrator, Planner, Coder, Critic, Explorer, Librarian, DevOps) — v1 ships Orchestrator, Coder, Critic
- **Context**: Sliding window + summaries
- **Docker Images**: Template-specific base images
- **Project Structure**: Cargo workspace with multiple crates
- **Test Strategy**: TDD from start
- **Scope**: EVERYTHING — one plan

### Research Findings
- **OpenClaw**: Gateway control plane architecture, multi-channel message routing, 3-tier skills platform (bundled/managed/workspace)
- **Hermes Agent**: Personality system with dynamic switching, toolset distributions per task type, closed learning loop
- **OmO**: Category-based model routing, discipline agents, hash-anchored edits (Hashline), skill-embedded MCPs, todo continuation enforcer
- **Sisyphus Labs**: Planner→Executor→Critic loop (Ultraworker), parallel wave execution, agent dispatch with categories/skills
- **Moltis**: 46-crate Rust workspace (~196K LoC), zero unsafe code, SQLite+FTS+vector, 15 lifecycle hooks, MCP support
- **IronClaw**: WASM sandbox with capability-based permissions, credential injection at host boundary, prompt injection defense layers
- **Claude Code OAuth**: PKCE flow with client_id `9d1c250a-e61b-44d9-88ed-5944d1962f5e`, CCH body signing via xxHash64, beta headers required, SSE streaming

### Metis Review
**Identified Gaps** (addressed):
- Secret storage: Use env vars + restricted-permissions secrets file (NOT SQLite)
- Agent lifecycle: Max 5 iterations per cycle, 5-min hard timeout per action
- Resource limits: Max concurrent containers configurable, default 2GB RAM / 10GB disk per container
- Error recovery: Graceful degradation when providers/Docker/GitHub unavailable
- Logging: tracing + tracing-subscriber from day 1
- Config format: TOML for human-editable config
- MSRV: Pin to rust-version = "1.94.0"
- Phasing: Strict phase gates — no phase N+1 until phase N passes all tests

**Scope Guardrails** (from Metis):
- No WASM sandbox in v1 — Docker only
- No JetBrains plugin in v1
- No hash-anchored edits (Hashline) in v1
- No hook system in v1
- No skill-embedded MCPs in v1
- No ChatGPT OAuth in v1 — Claude OAuth + Bedrock covers critical path
- Single SQLite database with project_id foreign keys
- Templates are static markdown — no template engine
- WebUI = chat + build logs + diffs — no code editor, no file tree, no terminal
- Self-update = binary pull + restart — no self-modification of prompts
- Reverse proxy = simple TCP proxy — no SSL termination, no custom domains

---

## Work Objectives

### Core Objective
Build a working multi-agent coding platform that receives project descriptions via Discord/WebUI, orchestrates AI agents backed by different models, executes code in Docker sandboxes, and manages full GitHub workflows — with self-update capabilities.

### Concrete Deliverables
- `cuttlefish-rs` Cargo workspace with 10 crates
- `cuttlefish-core`: Error types, config (TOML), tracing, shared traits
- `cuttlefish-db`: SQLite schema + migrations via sqlx
- `cuttlefish-providers`: ModelProvider trait + Bedrock impl + Claude OAuth impl
- `cuttlefish-sandbox`: Sandbox trait + Docker impl via bollard
- `cuttlefish-vcs`: VCS trait + git2 impl + GitHub API client
- `cuttlefish-agents`: Agent trait + Orchestrator/Coder/Critic
- `cuttlefish-discord`: Discord bot via serenity
- `cuttlefish-api`: axum HTTP/WS server
- `cuttlefish-tui`: ratatui TUI client binary
- `cuttlefish-web/`: Nuxt frontend (separate, not a Rust crate)
- Working end-to-end: Discord message → Agent orchestration → Code in Docker → Push to GitHub
- Self-update mechanism via GitHub Actions

### Definition of Done
- [ ] `cargo test --workspace` passes with 0 failures
- [ ] `cargo clippy --workspace -- -D warnings` passes
- [ ] `cargo doc --workspace --no-deps` generates without errors
- [ ] End-to-end: describe project in Discord → agent creates GitHub repo with code
- [ ] End-to-end: describe project in WebUI → same result
- [ ] TUI connects to server, displays chat, shows diffs
- [ ] Self-update: push to Cuttlefish repo → Actions build → new binary deployed → process restarts

### Must Have
- Multi-model provider abstraction (Bedrock + Claude Code OAuth)
- Category-based model routing (configurable per agent role)
- Docker sandbox per project with resource limits
- SQLite persistence with WAL mode
- Discord bot with channel-per-project
- WebUI with real-time chat and build log streaming
- GitHub clone/branch/commit/push workflow
- TDD test suite for all crates
- Structured logging via tracing
- TOML configuration

### Must NOT Have (Guardrails)
- No `unsafe` code — `#![deny(unsafe_code)]` workspace-wide
- No `unwrap()` in library crates — `#![deny(clippy::unwrap_used)]`
- No `println!` debugging — use `tracing` macros only
- No global mutable state — no `lazy_static!`, no `once_cell` globals
- No WASM sandbox — Docker only in v1
- No JetBrains plugin
- No ChatGPT OAuth — Bedrock + Claude OAuth only in v1
- No template engine — templates are static markdown read by agents
- No code editor in WebUI — chat + logs + diffs only
- No SSL termination in reverse proxy
- No hook system or MCP protocol in v1
- No self-modification of agent prompts
- No secrets stored in SQLite — env vars or restricted-permission files only

---

## Verification Strategy

> **ZERO HUMAN INTERVENTION** — ALL verification is agent-executed. No exceptions.

### Test Decision
- **Infrastructure exists**: NO (greenfield — creating from scratch)
- **Automated tests**: TDD from start
- **Framework**: `cargo test` (built-in Rust test framework)
- **TDD Flow**: Each task follows RED (failing test) → GREEN (minimal impl) → REFACTOR

### QA Policy
Every task MUST include agent-executed QA scenarios.
Evidence saved to `.sisyphus/evidence/task-{N}-{scenario-slug}.{ext}`.

- **Library crates**: `cargo test -p <crate>` — all tests pass
- **Binary crates**: Run the binary, verify output via Bash
- **API endpoints**: `curl` requests with expected responses
- **Discord bot**: Start bot, verify connection, test slash commands
- **WebUI**: Playwright — navigate, interact, assert DOM
- **TUI**: tmux — launch, send keystrokes, verify output
- **Docker integration**: bollard — create container, exec command, verify output, cleanup

### Workspace-Level Verification
```bash
cargo test --workspace                          # All tests pass
cargo clippy --workspace -- -D warnings         # Zero warnings
cargo doc --workspace --no-deps                 # Docs generate cleanly
cargo build --release                           # Release build succeeds
```

---

## Execution Strategy

### Parallel Execution Waves

```
Wave 1 (Foundation — start immediately):
├── Task 1:  Cargo workspace scaffolding + workspace lints [quick]
├── Task 2:  cuttlefish-core: error types + config (TOML) + tracing setup [quick]
├── Task 3:  Core trait definitions (ModelProvider, Agent, Sandbox, VCS, MessageBus) [deep]
├── Task 4:  cuttlefish-db: SQLite schema + migrations [quick]
├── Task 5:  CI: GitHub Actions workflow (cargo test + clippy + fmt) [quick]
└── Task 6:  Project README.md + CLAUDE.md + .gitignore [quick]

Wave 2 (Providers + Storage — after Wave 1):
├── Task 7:  cuttlefish-providers: Bedrock provider impl + streaming [deep]
├── Task 8:  cuttlefish-providers: Claude Code OAuth PKCE flow [deep]
├── Task 9:  cuttlefish-db: conversation storage + sliding window queries [unspecified-high]
├── Task 10: cuttlefish-db: project metadata + template storage [quick]
└── Task 11: cuttlefish-core: context manager (sliding window + summaries) [unspecified-high]

Wave 3 (Sandbox + VCS — after Wave 2):
├── Task 12: cuttlefish-sandbox: Docker container lifecycle (bollard) [deep]
├── Task 13: cuttlefish-sandbox: resource limits + output capture [unspecified-high]
├── Task 14: cuttlefish-vcs: git2 operations (clone, branch, commit, push) [deep]
├── Task 15: cuttlefish-vcs: GitHub API client (PAT auth) [unspecified-high]
└── Task 16: cuttlefish-sandbox: template-specific Docker images (Dockerfile gen) [quick]

Wave 4 (Agent System — after Waves 2+3):
├── Task 17: cuttlefish-agents: Agent trait + message bus (tokio channels) [deep]
├── Task 18: cuttlefish-agents: Orchestrator agent (routes tasks, manages lifecycle) [deep]
├── Task 19: cuttlefish-agents: Coder agent (file ops, code gen, build execution) [deep]
├── Task 20: cuttlefish-agents: Critic agent (code review, test execution) [deep]
└── Task 21: cuttlefish-agents: Planner→Coder→Critic loop with max iterations [deep]

Wave 5 (Discord Interface — after Wave 4):
├── Task 22: cuttlefish-discord: bot setup + connection + slash commands [unspecified-high]
├── Task 23: cuttlefish-discord: channel-per-project management [unspecified-high]
├── Task 24: cuttlefish-discord: message routing to agents + output formatting [unspecified-high]
└── Task 25: cuttlefish-discord: multi-server support [quick]

Wave 6 (API Server — after Wave 4):
├── Task 26: cuttlefish-api: axum server + WebSocket handler [deep]
├── Task 27: cuttlefish-api: REST endpoints (projects, conversations, builds) [unspecified-high]
├── Task 28: cuttlefish-api: API key authentication middleware [quick]
└── Task 29: cuttlefish-api: reverse proxy for dev servers [unspecified-high]

Wave 7 (WebUI — after Wave 6):
├── Task 30: cuttlefish-web: Nuxt project setup + WebSocket client [unspecified-high]
├── Task 31: cuttlefish-web: chat interface with code blocks [visual-engineering]
├── Task 32: cuttlefish-web: build log viewer (streaming) [visual-engineering]
├── Task 33: cuttlefish-web: file diff viewer [visual-engineering]
└── Task 34: cuttlefish-web: project dashboard + management [visual-engineering]

Wave 8 (TUI Client — after Wave 6):
├── Task 35: cuttlefish-tui: ratatui app + WebSocket connection [deep]
├── Task 36: cuttlefish-tui: chat view + input handling [unspecified-high]
└── Task 37: cuttlefish-tui: diff view + build log view [unspecified-high]

Wave 9 (Advanced Features — after Waves 5-8):
├── Task 38: Self-update: GitHub Actions build + binary pull + restart [deep]
├── Task 39: GitHub Actions monitoring (poll workflow status) [unspecified-high]
├── Task 40: Template system (.MD reading + project scaffolding) [unspecified-high]
├── Task 41: Context summarization (automatic old-context summaries) [unspecified-high]
├── Task 42: cuttlefish-vcs: GitHub App authentication [unspecified-high]
└── Task 43: Category-based model routing config (TOML) [quick]

Wave 10 (Integration + E2E — after Wave 9):
├── Task 44: End-to-end: Discord → Agent → Docker → GitHub flow [deep]
├── Task 45: End-to-end: WebUI → Agent → Docker → GitHub flow [deep]
└── Task 46: End-to-end: TUI → Server → Agent flow [deep]

Wave FINAL (Verification — after ALL tasks):
├── Task F1: Plan compliance audit [oracle]
├── Task F2: Code quality review [unspecified-high]
├── Task F3: Real manual QA [unspecified-high]
└── Task F4: Scope fidelity check [deep]
→ Present results → Get explicit user okay
```

### Dependency Matrix

| Task | Depends On | Blocks |
|------|-----------|--------|
| 1 | — | 2-6 |
| 2 | 1 | 7-11 |
| 3 | 1 | 7-8, 12-15, 17-21 |
| 4 | 1, 2 | 9-10 |
| 5 | 1 | — |
| 6 | 1 | — |
| 7 | 2, 3 | 17-21 |
| 8 | 2, 3 | 17-21 |
| 9 | 4, 3 | 11, 17 |
| 10 | 4 | 40 |
| 11 | 9, 7 | 17-21 |
| 12 | 3 | 13, 16, 17-21 |
| 13 | 12 | 17-21 |
| 14 | 3 | 15, 17-21 |
| 15 | 14 | 17-21 |
| 16 | 12 | 40 |
| 17 | 7, 11, 12-15 | 18-21 |
| 18 | 17 | 21 |
| 19 | 17 | 21 |
| 20 | 17 | 21 |
| 21 | 18, 19, 20 | 22-29, 44-46 |
| 22 | 21 | 23-25 |
| 23 | 22 | 24-25 |
| 24 | 23 | 25, 44 |
| 25 | 23 | 44 |
| 26 | 21 | 27-29, 30-37 |
| 27 | 26 | 30 |
| 28 | 26 | 30, 35 |
| 29 | 26 | — |
| 30 | 27, 28 | 31-34 |
| 31 | 30 | 45 |
| 32 | 30 | 45 |
| 33 | 30 | 45 |
| 34 | 30 | 45 |
| 35 | 26, 28 | 36-37 |
| 36 | 35 | 46 |
| 37 | 35 | 46 |
| 38 | 5 | — |
| 39 | 15 | — |
| 40 | 10, 16 | — |
| 41 | 11 | — |
| 42 | 15 | — |
| 43 | 2, 7 | — |
| 44 | 24, 25, 21 | F1-F4 |
| 45 | 31-34, 21 | F1-F4 |
| 46 | 36-37, 21 | F1-F4 |

### Agent Dispatch Summary

| Wave | Tasks | Categories |
|------|-------|------------|
| 1 | 6 | T1→`quick`, T2→`quick`, T3→`deep`, T4→`quick`, T5→`quick`, T6→`quick` |
| 2 | 5 | T7→`deep`, T8→`deep`, T9→`unspecified-high`, T10→`quick`, T11→`unspecified-high` |
| 3 | 5 | T12→`deep`, T13→`unspecified-high`, T14→`deep`, T15→`unspecified-high`, T16→`quick` |
| 4 | 5 | T17→`deep`, T18→`deep`, T19→`deep`, T20→`deep`, T21→`deep` |
| 5 | 4 | T22→`unspecified-high`, T23→`unspecified-high`, T24→`unspecified-high`, T25→`quick` |
| 6 | 4 | T26→`deep`, T27→`unspecified-high`, T28→`quick`, T29→`unspecified-high` |
| 7 | 5 | T30→`unspecified-high`, T31→`visual-engineering`, T32→`visual-engineering`, T33→`visual-engineering`, T34→`visual-engineering` |
| 8 | 3 | T35→`deep`, T36→`unspecified-high`, T37→`unspecified-high` |
| 9 | 6 | T38→`deep`, T39→`unspecified-high`, T40→`unspecified-high`, T41→`unspecified-high`, T42→`unspecified-high`, T43→`quick` |
| 10 | 3 | T44→`deep`, T45→`deep`, T46→`deep` |
| FINAL | 4 | F1→`oracle`, F2→`unspecified-high`, F3→`unspecified-high`, F4→`deep` |

---

## TODOs

### Wave 1 — Foundation

- [x] 1. Cargo Workspace Scaffolding + Workspace Lints

  **What to do**:
  - Convert root `Cargo.toml` to `[workspace]` with `members` list
  - Create all 9 crate directories under `crates/` with their own `Cargo.toml`
  - Crate list: `cuttlefish-core`, `cuttlefish-db`, `cuttlefish-providers`, `cuttlefish-sandbox`, `cuttlefish-vcs`, `cuttlefish-agents`, `cuttlefish-discord`, `cuttlefish-api`, `cuttlefish-tui`
  - Each crate gets a `lib.rs` (library crates) or `main.rs` (binary crates: tui)
  - Root binary `src/main.rs` remains as the server entry point
  - Set workspace-level lints: `deny(unsafe_code)`, `warn(missing_docs)`, `deny(clippy::unwrap_used)` on lib crates
  - Set `rust-version = "1.94.0"` in workspace
  - Add shared workspace dependencies: `tokio`, `serde`, `tracing`, `thiserror`, `anyhow`
  - Verify: `cargo check --workspace` compiles with zero errors

  **Must NOT do**:
  - Do NOT add any application logic — only scaffolding
  - Do NOT use `unsafe` anywhere
  - Do NOT add dependencies not in the shared workspace list

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: NO — this is the foundation
  - **Parallel Group**: Wave 1 (first task)
  - **Blocks**: Tasks 2-6 (all Wave 1), and transitively all subsequent tasks
  - **Blocked By**: None

  **References**:
  - `Cargo.toml` (line 1-6) — Current bare Cargo.toml to convert
  - `src/main.rs` (line 1-3) — Current Hello World to keep as server entry
  - Moltis workspace pattern: 46 crates with focused boundaries

  **Acceptance Criteria**:
  - [ ] `cargo check --workspace` exits with code 0
  - [ ] `crates/` directory contains 9 subdirectories
  - [ ] Each crate has its own `Cargo.toml` with correct name
  - [ ] Root `Cargo.toml` has `[workspace]` section with all members
  - [ ] `#![deny(unsafe_code)]` present in each lib.rs

  **QA Scenarios**:
  ```
  Scenario: Workspace compiles clean
    Tool: Bash
    Preconditions: Fresh workspace created
    Steps:
      1. Run `cargo check --workspace`
      2. Assert exit code 0
      3. Run `cargo clippy --workspace -- -D warnings`
      4. Assert exit code 0
    Expected Result: Both commands succeed with zero errors/warnings
    Evidence: .sisyphus/evidence/task-1-workspace-check.txt

  Scenario: All crates exist with correct structure
    Tool: Bash
    Preconditions: Workspace created
    Steps:
      1. Run `ls crates/` — assert 9 directories
      2. For each crate, verify `Cargo.toml` exists and contains correct `[package]` name
      3. Verify root `Cargo.toml` has `[workspace]` with all 9 members
    Expected Result: 9 crates, all named correctly, all in workspace members
    Evidence: .sisyphus/evidence/task-1-crate-structure.txt
  ```

  **Commit**: YES
  - Message: `chore(workspace): initialize cargo workspace with 9 crates`
  - Files: `Cargo.toml`, `crates/*/Cargo.toml`, `crates/*/src/lib.rs`
  - Pre-commit: `cargo check --workspace`

- [x] 2. cuttlefish-core: Error Types + Config + Tracing

  **What to do**:
  - Create error hierarchy using `thiserror`:
    - `CuttlefishError` (top-level enum): `Config`, `Provider`, `Sandbox`, `Vcs`, `Agent`, `Database`, `Discord`, `Api`, `Io`
    - Each variant wraps a domain-specific error type
  - Create TOML config structure:
    - `CuttlefishConfig` with sections: `server`, `database`, `providers`, `agents`, `discord`, `sandbox`
    - `ServerConfig`: host, port, api_key
    - `DatabaseConfig`: path to SQLite file
    - `ProviderConfig`: map of provider name → provider-specific config
    - `AgentConfig`: map of agent role → category + model override
    - `DiscordConfig`: bot_token (from env var reference), guild_ids
    - `SandboxConfig`: docker_socket, default resource limits
  - Load config from `cuttlefish.toml` with env var override support
  - Set up `tracing` + `tracing-subscriber` with JSON structured logging
  - Create `init_tracing()` function that configures subscriber with env filter
  - Write tests for: config loading, config validation, error Display impls

  **Must NOT do**:
  - No `println!` or `eprintln!` — `tracing` macros only
  - No secrets in config file — reference env vars for tokens
  - No `unwrap()` — proper error propagation

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 3-6 after Task 1 completes)
  - **Parallel Group**: Wave 1
  - **Blocks**: Tasks 7-11 (Wave 2)
  - **Blocked By**: Task 1

  **References**:
  - Moltis `moltis-config` crate (7K LoC) — config validation patterns
  - `thiserror` crate for derive-based error types
  - `tracing-subscriber` EnvFilter for log level control

  **Acceptance Criteria**:
  - [ ] `cargo test -p cuttlefish-core` passes all tests
  - [ ] Config loads from `cuttlefish.toml` correctly
  - [ ] Invalid config produces descriptive error messages
  - [ ] `tracing` outputs structured JSON logs

  **QA Scenarios**:
  ```
  Scenario: Config loads from TOML file
    Tool: Bash
    Preconditions: Create a sample cuttlefish.toml in a temp dir
    Steps:
      1. Write minimal valid TOML config to temp file
      2. Run test that loads config from that path
      3. Assert all fields parsed correctly
    Expected Result: Config struct populated with correct values
    Evidence: .sisyphus/evidence/task-2-config-load.txt

  Scenario: Missing required config field produces clear error
    Tool: Bash
    Preconditions: Create a TOML file missing `server.port`
    Steps:
      1. Attempt to load config
      2. Assert error message contains "server.port" or similar
    Expected Result: Descriptive error, not a panic or generic message
    Evidence: .sisyphus/evidence/task-2-config-error.txt
  ```

  **Commit**: YES
  - Message: `feat(core): add error types, TOML config, and tracing setup`
  - Files: `crates/cuttlefish-core/src/*`
  - Pre-commit: `cargo test -p cuttlefish-core`

- [x] 3. Core Trait Definitions

  **What to do**:
  - Define the 5 foundational traits in `cuttlefish-core`:
  - `ModelProvider` trait:
    ```rust
    #[async_trait]
    pub trait ModelProvider: Send + Sync {
        fn name(&self) -> &str;
        async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse>;
        fn stream(&self, request: CompletionRequest) -> Pin<Box<dyn Stream<Item = Result<StreamChunk>> + Send>>;
        async fn count_tokens(&self, text: &str) -> Result<usize>;
    }
    ```
  - `Agent` trait:
    ```rust
    #[async_trait]
    pub trait Agent: Send + Sync {
        fn name(&self) -> &str;
        fn role(&self) -> AgentRole;
        async fn execute(&self, ctx: &mut AgentContext, input: &str) -> Result<AgentOutput>;
    }
    ```
  - `Sandbox` trait:
    ```rust
    #[async_trait]
    pub trait Sandbox: Send + Sync {
        async fn create(&self, config: &SandboxConfig) -> Result<SandboxId>;
        async fn exec(&self, id: &SandboxId, command: &str) -> Result<ExecOutput>;
        async fn write_file(&self, id: &SandboxId, path: &Path, content: &[u8]) -> Result<()>;
        async fn read_file(&self, id: &SandboxId, path: &Path) -> Result<Vec<u8>>;
        async fn destroy(&self, id: &SandboxId) -> Result<()>;
    }
    ```
  - `VersionControl` trait:
    ```rust
    #[async_trait]
    pub trait VersionControl: Send + Sync {
        async fn clone_repo(&self, url: &str, path: &Path) -> Result<()>;
        async fn checkout_branch(&self, path: &Path, branch: &str) -> Result<()>;
        async fn commit(&self, path: &Path, message: &str, files: &[PathBuf]) -> Result<String>;
        async fn push(&self, path: &Path, remote: &str, branch: &str) -> Result<()>;
        async fn diff(&self, path: &Path) -> Result<String>;
    }
    ```
  - `MessageBus` trait:
    ```rust
    #[async_trait]
    pub trait MessageBus: Send + Sync {
        async fn publish(&self, topic: &str, message: BusMessage) -> Result<()>;
        async fn subscribe(&self, topic: &str) -> Result<Receiver<BusMessage>>;
    }
    ```
  - Define supporting types: `CompletionRequest`, `CompletionResponse`, `StreamChunk`, `AgentRole`, `AgentContext`, `AgentOutput`, `SandboxConfig`, `SandboxId`, `ExecOutput`, `BusMessage`
  - Define `AgentRole` enum: `Orchestrator`, `Planner`, `Coder`, `Critic`, `Explorer`, `Librarian`, `DevOps`
  - Define `Category` enum: `Visual`, `Deep`, `Quick`, `UltraBrain`, `UnspecifiedLow`, `UnspecifiedHigh`

  **Must NOT do**:
  - No implementations — traits and types ONLY
  - No concrete struct impls
  - No dependencies on other cuttlefish crates (core is the root)

  **Recommended Agent Profile**:
  - **Category**: `deep`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 2, 4-6)
  - **Parallel Group**: Wave 1
  - **Blocks**: Tasks 7-8, 12-15, 17-21
  - **Blocked By**: Task 1

  **References**:
  - Moltis trait-based service architecture (agents, providers, tools)
  - OmO category-based routing: `Category` enum maps to models
  - Sisyphus agent roles: Orchestrator, Planner, Coder, Critic, Explorer, Librarian

  **Acceptance Criteria**:
  - [ ] `cargo check -p cuttlefish-core` compiles with zero errors
  - [ ] All 5 traits defined with proper async signatures
  - [ ] All supporting types defined with serde Serialize/Deserialize
  - [ ] `cargo doc -p cuttlefish-core --no-deps` generates clean documentation

  **QA Scenarios**:
  ```
  Scenario: Traits compile and are object-safe where needed
    Tool: Bash
    Steps:
      1. Run `cargo check -p cuttlefish-core`
      2. Assert exit code 0
      3. Write a test that creates `Box<dyn ModelProvider>` to verify object safety
      4. Run `cargo test -p cuttlefish-core`
    Expected Result: Compilation succeeds, object safety verified
    Evidence: .sisyphus/evidence/task-3-traits-compile.txt

  Scenario: Documentation generates cleanly
    Tool: Bash
    Steps:
      1. Run `cargo doc -p cuttlefish-core --no-deps`
      2. Assert exit code 0
      3. Verify doc output contains all 5 trait names
    Expected Result: Docs generated with no warnings
    Evidence: .sisyphus/evidence/task-3-docs.txt
  ```

  **Commit**: YES
  - Message: `feat(core): define ModelProvider, Agent, Sandbox, VCS, MessageBus traits`
  - Files: `crates/cuttlefish-core/src/traits/*.rs`
  - Pre-commit: `cargo check --workspace`

- [x] 4. cuttlefish-db: SQLite Schema + Migrations

  **What to do**:
  - Set up sqlx with SQLite and compile-time checked queries
  - Design schema with these tables:
    - `projects`: id, name, description, status, template_name, github_url, discord_channel_id, docker_container_id, created_at, updated_at
    - `conversations`: id, project_id (FK), role (user/assistant/system), content, model_used, token_count, created_at
    - `agent_sessions`: id, project_id (FK), agent_role, status, started_at, completed_at
    - `templates`: id, name, description, content_md, language, created_at
    - `build_logs`: id, project_id (FK), status (running/success/failure), output, command, started_at, completed_at
    - `config_overrides`: id, project_id (FK), key, value (for per-project config)
  - Create migration files in `crates/cuttlefish-db/migrations/`
  - Implement `Database` struct with connection pool
  - Enable WAL mode on connection
  - Write CRUD functions for each table
  - Tests: insert/query/update/delete for each table

  **Must NOT do**:
  - No secrets stored in any table
  - No `unwrap()` — proper sqlx error handling

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 2, 3, 5, 6)
  - **Parallel Group**: Wave 1
  - **Blocks**: Tasks 9, 10
  - **Blocked By**: Task 1, Task 2 (for error types)

  **References**:
  - sqlx migration system: `sqlx migrate add` creates timestamped migration files
  - Moltis SQLite+FTS pattern for memory/search
  - WAL mode: `PRAGMA journal_mode=WAL;` on connection open

  **Acceptance Criteria**:
  - [ ] `cargo test -p cuttlefish-db` passes all CRUD tests
  - [ ] Migrations run successfully on fresh SQLite file
  - [ ] WAL mode is enabled (verify with PRAGMA query in test)
  - [ ] All queries are compile-time checked by sqlx

  **QA Scenarios**:
  ```
  Scenario: Database creates and migrates successfully
    Tool: Bash
    Steps:
      1. Run `cargo test -p cuttlefish-db` which creates temp DB and runs migrations
      2. Assert all tests pass
      3. Verify tables exist by running a test that queries sqlite_master
    Expected Result: All 6 tables created, WAL mode enabled
    Evidence: .sisyphus/evidence/task-4-db-migrate.txt

  Scenario: CRUD operations work for all tables
    Tool: Bash
    Steps:
      1. Run tests that insert, query, update, delete for each table
      2. Assert all pass with correct data round-tripping
    Expected Result: Zero failures across all CRUD tests
    Evidence: .sisyphus/evidence/task-4-db-crud.txt
  ```

  **Commit**: YES
  - Message: `feat(db): add SQLite schema with 6 tables and migrations`
  - Files: `crates/cuttlefish-db/src/*`, `crates/cuttlefish-db/migrations/*`
  - Pre-commit: `cargo test -p cuttlefish-db`

- [x] 5. CI: GitHub Actions Workflow

  **What to do**:
  - Create `.github/workflows/ci.yml` with:
    - Triggers: push to main, pull requests
    - Matrix: stable Rust (1.94.0)
    - Steps: checkout, install Rust, `cargo fmt -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace`, `cargo doc --workspace --no-deps`
    - Cache: cargo registry + target directory
  - Create `.github/workflows/release.yml` for self-update:
    - Triggers: tag push (v*)
    - Steps: build release binary, create GitHub Release, upload binary asset
  - This enables the self-update mechanism (Task 38) to pull release binaries

  **Must NOT do**:
  - No secrets in workflow files — use GitHub Secrets references
  - No Docker-dependent tests in CI (yet) — only unit tests

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 2-4, 6)
  - **Parallel Group**: Wave 1
  - **Blocks**: Task 38 (self-update)
  - **Blocked By**: Task 1

  **References**:
  - Standard Rust CI patterns with cargo fmt/clippy/test
  - GitHub Actions release workflow with `softprops/action-gh-release`

  **Acceptance Criteria**:
  - [ ] `.github/workflows/ci.yml` exists and is valid YAML
  - [ ] `.github/workflows/release.yml` exists and is valid YAML
  - [ ] CI runs fmt, clippy, test, doc in correct order

  **QA Scenarios**:
  ```
  Scenario: CI workflow file is valid
    Tool: Bash
    Steps:
      1. Validate YAML syntax: `python3 -c "import yaml; yaml.safe_load(open('.github/workflows/ci.yml'))"`
      2. Verify all expected steps are present (fmt, clippy, test, doc)
    Expected Result: Valid YAML with all 4 check steps
    Evidence: .sisyphus/evidence/task-5-ci-valid.txt
  ```

  **Commit**: YES
  - Message: `ci: add GitHub Actions workflows for CI and release`
  - Files: `.github/workflows/ci.yml`, `.github/workflows/release.yml`

- [x] 6. Project README.md + CLAUDE.md + .gitignore

  **What to do**:
  - Create `README.md` with:
    - Project name (Cuttlefish 🐙), description, philosophy
    - Architecture overview (multi-agent, multi-model, multi-interface)
    - Crate structure diagram
    - Getting started (prerequisites, build, config, run)
    - Configuration reference (cuttlefish.toml)
    - License (TBD — ask user or default to MIT)
  - Create `CLAUDE.md` with:
    - Project conventions: Rust 2024 edition, zero unsafe, thiserror for errors
    - Architecture notes: workspace crates, trait-first design
    - Development workflow: TDD, cargo test, cargo clippy
    - Module boundaries and dependency rules
  - Update `.gitignore` with: `/target`, `*.db`, `*.sqlite`, `.env`, `cuttlefish.toml` (user config), `*.log`
  - Create sample `cuttlefish.example.toml` with documented defaults

  **Must NOT do**:
  - No real secrets in example config
  - No overblown marketing copy — clear technical documentation

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 2-5)
  - **Parallel Group**: Wave 1
  - **Blocks**: None directly
  - **Blocked By**: Task 1

  **Acceptance Criteria**:
  - [ ] README.md exists with all sections
  - [ ] CLAUDE.md exists with project conventions
  - [ ] .gitignore covers target/, *.db, .env
  - [ ] cuttlefish.example.toml is valid TOML

  **QA Scenarios**:
  ```
  Scenario: Documentation files exist and are well-formed
    Tool: Bash
    Steps:
      1. Verify README.md, CLAUDE.md, .gitignore, cuttlefish.example.toml all exist
      2. Validate TOML: `python3 -c "import tomllib; tomllib.load(open('cuttlefish.example.toml','rb'))"`
    Expected Result: All files present, TOML valid
    Evidence: .sisyphus/evidence/task-6-docs.txt
  ```

  **Commit**: YES
  - Message: `docs: add README.md, CLAUDE.md, .gitignore, and example config`
  - Files: `README.md`, `CLAUDE.md`, `.gitignore`, `cuttlefish.example.toml`

---

### Wave 2 — Providers + Storage

- [x] 7. cuttlefish-providers: AWS Bedrock Provider + Streaming

  **What to do**:
  - Implement `ModelProvider` trait for AWS Bedrock
  - Use `aws-sdk-bedrockruntime` with `aws-config` for credential resolution
  - Support `converse_stream` API for Claude models on Bedrock
  - Implement streaming via `Pin<Box<dyn Stream<Item = Result<StreamChunk>>>>` 
  - Handle: model selection (claude-sonnet, claude-opus, claude-haiku), region config, credential chain (env vars, IAM role, profile)
  - Implement token counting estimation
  - Create `MockModelProvider` for testing (returns canned responses)
  - Tests: mock provider tests, request serialization, response parsing, stream assembly

  **Must NOT do**:
  - No hardcoded credentials — use AWS credential chain
  - No blocking I/O — all async

  **Recommended Agent Profile**:
  - **Category**: `deep`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 8-11)
  - **Parallel Group**: Wave 2
  - **Blocks**: Tasks 11, 17-21, 43
  - **Blocked By**: Tasks 2, 3

  **References**:
  - `aws-sdk-bedrockruntime` converse_stream API
  - Bedrock model IDs: `anthropic.claude-3-5-sonnet-20241022-v2:0`, etc.
  - AWS credential chain: env vars → profile → IAM role

  **Acceptance Criteria**:
  - [ ] `cargo test -p cuttlefish-providers` passes
  - [ ] MockModelProvider returns canned responses correctly
  - [ ] Streaming assembles chunks into complete response
  - [ ] Integration test (behind feature flag) calls real Bedrock

  **QA Scenarios**:
  ```
  Scenario: Mock provider returns streaming response
    Tool: Bash
    Steps:
      1. Run unit test that sends CompletionRequest to MockModelProvider
      2. Collect all StreamChunks from the stream
      3. Assert assembled text matches expected output
    Expected Result: Stream produces correct chunks, assembles to full response
    Evidence: .sisyphus/evidence/task-7-bedrock-mock.txt

  Scenario: Bedrock request serialization is correct
    Tool: Bash
    Steps:
      1. Create a CompletionRequest with system prompt + user message
      2. Serialize to Bedrock converse format
      3. Assert JSON structure matches Bedrock API spec
    Expected Result: Correctly formatted Bedrock API request
    Evidence: .sisyphus/evidence/task-7-bedrock-serialize.txt
  ```

  **Commit**: YES
  - Message: `feat(providers): add AWS Bedrock provider with streaming support`
  - Files: `crates/cuttlefish-providers/src/*`
  - Pre-commit: `cargo test -p cuttlefish-providers`

- [x] 8. cuttlefish-providers: Claude Code OAuth PKCE Flow

  **What to do**:
  - Implement full Claude Code OAuth emulation:
    - PKCE code verifier/challenge generation (SHA256 + base64url)
    - Authorization URL builder: `https://platform.claude.com/oauth/authorize` with client_id `9d1c250a-e61b-44d9-88ed-5944d1962f5e`, scopes, PKCE challenge
    - Local callback server (ephemeral port) to receive auth code
    - Token exchange: POST to `https://platform.claude.com/v1/oauth/token` with auth code + code verifier
    - Token refresh flow
    - Token storage (file with restricted permissions, NOT SQLite)
  - Implement CCH (Claude Code Hash) body signing:
    - xxHash64 with seed `0x6e52736ac806831e`
    - Hash full request body with `cch=00000` placeholder
    - Lower 20 bits → 5-char lowercase hex
  - Implement request headers:
    - `anthropic-beta: claude-code-20250219,oauth-2025-04-20,interleaved-thinking-2025-05-14`
    - `x-stainless-*` headers (spoofing Node.js runtime)
    - `User-Agent: claude-cli/2.1.87 (external, cli)`
    - `x-anthropic-billing-header` with fingerprint + CCH
  - Implement `ModelProvider` trait using these OAuth tokens
  - SSE streaming support for responses
  - Tests: PKCE generation, URL building, CCH computation, header construction, token refresh (mocked HTTP)

  **Must NOT do**:
  - No storing OAuth tokens in SQLite — file with 0600 permissions
  - No hardcoded tokens — dynamic PKCE flow

  **Recommended Agent Profile**:
  - **Category**: `deep`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 7, 9-11)
  - **Parallel Group**: Wave 2
  - **Blocks**: Tasks 17-21
  - **Blocked By**: Tasks 2, 3

  **References**:
  - Not-Claude-Code-Emulator: full OAuth specification documented in research
  - OAuth endpoints: authorize at `platform.claude.com`, token at `platform.claude.com/v1/oauth/token`, API at `api.anthropic.com`
  - Client ID: `9d1c250a-e61b-44d9-88ed-5944d1962f5e`
  - Scopes: `user:inference user:profile user:sessions:claude_code user:mcp_servers user:file_upload org:create_api_key`
  - CCH: xxHash64 seed `0x6e52736ac806831e`, lower 20 bits, 5-char hex

  **Acceptance Criteria**:
  - [ ] `cargo test -p cuttlefish-providers` passes Claude OAuth tests
  - [ ] PKCE challenge/verifier generated correctly (SHA256 verified)
  - [ ] CCH computed correctly for known test input
  - [ ] Authorization URL contains all required parameters
  - [ ] Token refresh flow works with mocked HTTP

  **QA Scenarios**:
  ```
  Scenario: PKCE code verifier and challenge are correctly generated
    Tool: Bash
    Steps:
      1. Generate PKCE pair
      2. Compute SHA256 of verifier, base64url encode
      3. Assert it matches the challenge
    Expected Result: Challenge = base64url(SHA256(verifier))
    Evidence: .sisyphus/evidence/task-8-pkce.txt

  Scenario: CCH body signing produces correct hash
    Tool: Bash
    Steps:
      1. Create a known request body with `cch=00000` placeholder
      2. Compute xxHash64 with seed 0x6e52736ac806831e
      3. Take lower 20 bits, format as 5-char hex
      4. Assert matches expected value
    Expected Result: CCH hash matches known-good output
    Evidence: .sisyphus/evidence/task-8-cch.txt
  ```

  **Commit**: YES
  - Message: `feat(providers): add Claude Code OAuth PKCE flow with CCH signing`
  - Files: `crates/cuttlefish-providers/src/claude_oauth/*`
  - Pre-commit: `cargo test -p cuttlefish-providers`

- [x] 9. cuttlefish-db: Conversation Storage + Sliding Window Queries

  **What to do**:
  - Implement conversation CRUD with efficient windowed queries:
    - `insert_message(project_id, role, content, model, tokens)` 
    - `get_recent_messages(project_id, limit)` — last N messages for context window
    - `get_messages_since(project_id, timestamp)` — for sync
    - `get_total_token_count(project_id)` — for context management
    - `get_message_count(project_id)` — for window sizing
  - Add indexes: `(project_id, created_at DESC)` for windowed queries
  - Implement `summarize_and_archive(project_id, before_timestamp, summary)`:
    - Marks old messages as archived
    - Inserts summary as a system message
  - Tests: insert 100 messages, query window, verify ordering, test archive flow

  **Must NOT do**:
  - No full-text search (that's future RAG work)
  - No vector embeddings

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 7, 8, 10, 11)
  - **Parallel Group**: Wave 2
  - **Blocks**: Task 11
  - **Blocked By**: Tasks 3, 4

  **Acceptance Criteria**:
  - [ ] Windowed query returns correct N most recent messages
  - [ ] Archive flow marks messages and inserts summary
  - [ ] Token count query returns accurate total
  - [ ] All tests pass with `cargo test -p cuttlefish-db`

  **QA Scenarios**:
  ```
  Scenario: Sliding window returns correct recent messages
    Tool: Bash
    Steps:
      1. Insert 50 messages for a project
      2. Query with limit=10
      3. Assert returns the 10 most recent, ordered by created_at DESC
    Expected Result: Exactly 10 messages, most recent first
    Evidence: .sisyphus/evidence/task-9-window.txt
  ```

  **Commit**: YES
  - Message: `feat(db): add conversation storage with sliding window queries`
  - Pre-commit: `cargo test -p cuttlefish-db`

- [x] 10. cuttlefish-db: Project Metadata + Template Storage

  **What to do**:
  - Implement project CRUD: create, get, list, update status, delete
  - Implement template CRUD: create, get, list by language
  - Project statuses: `Active`, `Paused`, `Completed`, `Failed`
  - Query: `get_active_projects()`, `get_project_by_discord_channel(channel_id)`
  - Seed initial templates (read from `templates/` directory on startup)
  - Tests for all CRUD operations

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 7-9, 11)
  - **Blocks**: Task 40
  - **Blocked By**: Task 4

  **Acceptance Criteria**:
  - [ ] Project CRUD works end-to-end
  - [ ] Template loading from directory works
  - [ ] Lookup by Discord channel ID returns correct project

  **QA Scenarios**:
  ```
  Scenario: Project lifecycle CRUD
    Tool: Bash
    Steps:
      1. Create project → assert ID returned
      2. Get project by ID → assert fields match
      3. Update status to Completed → assert updated
      4. List active projects → assert original not in list
    Expected Result: Full CRUD lifecycle works correctly
    Evidence: .sisyphus/evidence/task-10-project-crud.txt
  ```

  **Commit**: YES
  - Message: `feat(db): add project metadata and template storage`
  - Pre-commit: `cargo test -p cuttlefish-db`

- [x] 11. cuttlefish-core: Context Manager (Sliding Window + Summaries)

  **What to do**:
  - Implement `ContextManager` struct that:
    - Takes a `ModelProvider` (for generating summaries) and DB connection
    - `build_context(project_id, max_tokens)` → returns messages fitting within token budget
    - Loads recent messages first (sliding window)
    - If oldest loaded message has a summary before it, includes that summary as first message
    - `trigger_summarization(project_id)` → summarizes messages older than window, stores summary in DB
    - Configurable window size and summarization threshold
  - Tests with MockModelProvider: verify window building, summarization trigger

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 7-10)
  - **Blocks**: Tasks 17-21
  - **Blocked By**: Tasks 7 (needs ModelProvider), 9 (needs conversation queries)

  **Acceptance Criteria**:
  - [ ] Context builds within token budget
  - [ ] Summary included when older messages exist
  - [ ] Summarization produces reasonable output via mock

  **QA Scenarios**:
  ```
  Scenario: Context stays within token budget
    Tool: Bash
    Steps:
      1. Insert 100 messages with known token counts
      2. Build context with max_tokens=4000
      3. Assert total tokens of returned messages ≤ 4000
    Expected Result: Context fits within budget, most recent messages prioritized
    Evidence: .sisyphus/evidence/task-11-context-budget.txt
  ```

  **Commit**: YES
  - Message: `feat(core): add context manager with sliding window and summarization`
  - Pre-commit: `cargo test -p cuttlefish-core`

---

### Wave 3 — Sandbox + VCS

- [x] 12. cuttlefish-sandbox: Docker Container Lifecycle

  **What to do**:
  - Implement `Sandbox` trait using `bollard` crate for Docker API:
    - `create()`: Pull base image if needed, create container with resource limits, mount project volume
    - `exec()`: Execute command inside container, capture stdout/stderr
    - `write_file()` / `read_file()`: Transfer files to/from container
    - `destroy()`: Stop and remove container, cleanup volumes
  - Docker container config:
    - CPU limit (configurable, default 2 cores)
    - Memory limit (configurable, default 2GB)
    - Disk limit (configurable, default 10GB)
    - Network: bridge mode (can access internet for npm/pip/cargo installs)
    - Timeout: configurable per-exec (default 5 min)
  - Container naming: `cuttlefish-{project_id}-{short_uuid}`
  - Handle: container already exists, Docker daemon unavailable, image pull failure
  - Tests: mock Docker API responses, test lifecycle states

  **Recommended Agent Profile**:
  - **Category**: `deep`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 13-16)
  - **Blocks**: Tasks 13, 16, 17-21
  - **Blocked By**: Task 3

  **Acceptance Criteria**:
  - [ ] Container create/exec/destroy lifecycle works
  - [ ] Resource limits are applied correctly
  - [ ] File read/write works across container boundary
  - [ ] Integration test (feature-flagged) runs real Docker

  **QA Scenarios**:
  ```
  Scenario: Container lifecycle with real Docker
    Tool: Bash
    Preconditions: Docker daemon running
    Steps:
      1. Create container from alpine:latest
      2. Exec `echo hello` — assert stdout = "hello\n"
      3. Write file `/tmp/test.txt` with content "world"
      4. Read file `/tmp/test.txt` — assert content = "world"
      5. Destroy container — assert removed
    Expected Result: Full lifecycle completes without error
    Evidence: .sisyphus/evidence/task-12-docker-lifecycle.txt
  ```

  **Commit**: YES
  - Message: `feat(sandbox): add Docker container lifecycle management`
  - Pre-commit: `cargo test -p cuttlefish-sandbox`

- [x] 13. cuttlefish-sandbox: Resource Limits + Output Capture

  **What to do**:
  - Implement streaming output capture from container exec:
    - Real-time stdout/stderr streaming via Docker attach API
    - Buffer management for large outputs (cap at 1MB per exec)
    - Timeout enforcement: kill exec after configurable duration
  - Implement resource monitoring:
    - Check container stats (CPU%, memory usage) via bollard stats API
    - Kill container if OOM or runaway CPU
  - Implement cleanup:
    - `cleanup_stale_containers()`: find and remove containers older than threshold
    - Graceful shutdown: SIGTERM → wait 10s → SIGKILL
  - Tests: timeout behavior, large output truncation, cleanup

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 12, 14-16)
  - **Blocks**: Tasks 17-21
  - **Blocked By**: Task 12

  **Acceptance Criteria**:
  - [ ] Output streaming captures real-time stdout/stderr
  - [ ] Timeout kills long-running exec
  - [ ] Stale container cleanup works
  - [ ] Large output is truncated to 1MB cap

  **QA Scenarios**:
  ```
  Scenario: Exec timeout kills long-running command
    Tool: Bash
    Steps:
      1. Create container, exec `sleep 300` with 5s timeout
      2. Assert exec returns timeout error after ~5 seconds
      3. Assert container is still accessible (not destroyed)
    Expected Result: Timeout error within 5-7 seconds
    Evidence: .sisyphus/evidence/task-13-timeout.txt
  ```

  **Commit**: YES
  - Message: `feat(sandbox): add resource limits, output streaming, and cleanup`
  - Pre-commit: `cargo test -p cuttlefish-sandbox`

- [x] 14. cuttlefish-vcs: Git Operations via git2

  **What to do**:
  - Implement `VersionControl` trait using `git2`:
    - `clone_repo(url, path)`: Clone with progress callback
    - `checkout_branch(path, branch)`: Create or switch to branch
    - `commit(path, message, files)`: Stage files and commit
    - `push(path, remote, branch)`: Push with credential callback
    - `diff(path)`: Get unified diff of working directory changes
    - `create_branch(path, name)`: Create new branch from HEAD
    - `get_log(path, limit)`: Recent commit log
  - Credential handling: PAT via callback, SSH key support
  - Handle: empty repo, detached HEAD, merge conflicts (report, don't auto-resolve)
  - Tests: create temp repo, clone, commit, diff — all using local operations (no network)

  **Recommended Agent Profile**:
  - **Category**: `deep`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 12, 13, 15, 16)
  - **Blocks**: Tasks 15, 17-21
  - **Blocked By**: Task 3

  **Acceptance Criteria**:
  - [ ] Clone, branch, commit, diff all work on local repos
  - [ ] PAT credential callback works for push
  - [ ] Diff output is correctly formatted
  - [ ] All tests pass without network access

  **QA Scenarios**:
  ```
  Scenario: Full git workflow on local repo
    Tool: Bash
    Steps:
      1. Create temp bare repo via git2
      2. Clone it to working dir
      3. Create file, stage, commit
      4. Create branch, switch to it
      5. Modify file, get diff — assert diff shows changes
      6. Commit and push to bare repo
    Expected Result: All operations succeed, push lands in bare repo
    Evidence: .sisyphus/evidence/task-14-git-workflow.txt
  ```

  **Commit**: YES
  - Message: `feat(vcs): add git operations via git2 (clone, branch, commit, push, diff)`
  - Pre-commit: `cargo test -p cuttlefish-vcs`

- [x] 15. cuttlefish-vcs: GitHub API Client (PAT Auth)

  **What to do**:
  - Implement GitHub API client using `reqwest`:
    - `create_repo(name, description, private)` → creates repo, returns URL
    - `create_pull_request(owner, repo, title, body, head, base)` → creates PR
    - `get_workflow_runs(owner, repo, limit)` → lists recent Actions runs
    - `get_workflow_run_status(owner, repo, run_id)` → status + conclusion
    - `get_workflow_run_logs(owner, repo, run_id)` → download log zip
  - Authentication: PAT via `Authorization: Bearer {token}` header
  - Rate limit handling: parse `X-RateLimit-Remaining`, backoff when low
  - Tests: mock HTTP responses for each endpoint

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 12-14, 16)
  - **Blocks**: Tasks 17-21, 39, 42
  - **Blocked By**: Task 14

  **Acceptance Criteria**:
  - [ ] All GitHub API methods work with mocked responses
  - [ ] Rate limit handling backs off correctly
  - [ ] Error responses produce descriptive errors

  **QA Scenarios**:
  ```
  Scenario: GitHub API client handles rate limiting
    Tool: Bash
    Steps:
      1. Mock response with X-RateLimit-Remaining: 0 and Retry-After: 1
      2. Call create_repo — assert it waits and retries
      3. Second mock returns success
    Expected Result: Client retries after backoff, succeeds on second attempt
    Evidence: .sisyphus/evidence/task-15-ratelimit.txt
  ```

  **Commit**: YES
  - Message: `feat(vcs): add GitHub API client with PAT auth and rate limiting`
  - Pre-commit: `cargo test -p cuttlefish-vcs`

- [x] 16. cuttlefish-sandbox: Template-Specific Docker Images

  **What to do**:
  - Create Dockerfile templates for common stacks:
    - `node-base`: Node.js 22 + npm + common global tools
    - `python-base`: Python 3.12 + pip + venv
    - `rust-base`: Rust stable + cargo
    - `go-base`: Go 1.22+
    - `generic`: Ubuntu with build-essential + curl + git
  - Store Dockerfiles in `docker/` directory
  - Implement `build_template_image(template_name)` using bollard
  - Implement image caching: check if image exists before building
  - Map templates to base images in config

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 12-15)
  - **Blocks**: Task 40
  - **Blocked By**: Task 12

  **Acceptance Criteria**:
  - [ ] 5 Dockerfiles created and valid
  - [ ] Image build works via bollard
  - [ ] Image caching prevents unnecessary rebuilds

  **QA Scenarios**:
  ```
  Scenario: Node base image builds and runs
    Tool: Bash
    Steps:
      1. Build node-base image via bollard
      2. Create container from it
      3. Exec `node --version` — assert output starts with "v22"
      4. Cleanup
    Expected Result: Node.js available in container
    Evidence: .sisyphus/evidence/task-16-node-image.txt
  ```

  **Commit**: YES
  - Message: `feat(sandbox): add template-specific Docker images`
  - Pre-commit: `cargo test -p cuttlefish-sandbox`

---

### Wave 4 — Agent System

- [ ] 17. cuttlefish-agents: Agent Trait + Message Bus

  **What to do**:
  - Implement `MessageBus` using tokio broadcast/mpsc channels:
    - Topics: `agent.{role}.input`, `agent.{role}.output`, `project.{id}.events`
    - `publish(topic, BusMessage)` — sends to all subscribers on topic
    - `subscribe(topic)` — returns async Receiver
  - Implement `AgentContext` struct:
    - Access to: ModelProvider, Sandbox, VersionControl, Database, MessageBus, ContextManager
    - Project-scoped: each context knows its project_id
    - Tool registry: list of available tools for this agent
  - Implement `AgentRunner`:
    - Accepts an `Agent` impl + `AgentContext`
    - Runs the agent loop: receive input → build context → call model → parse tool calls → execute tools → respond
    - Max iterations per invocation (configurable, default 25)
    - Hard timeout (configurable, default 5 minutes)
  - Implement tool calling protocol:
    - Model returns tool_use blocks → parse → dispatch to tool executor → return results → continue loop
    - Tools: `read_file`, `write_file`, `execute_command`, `search_files`, `git_commit`, `git_push`
  - Tests: agent loop with mock provider, tool execution, timeout enforcement

  **Must NOT do**:
  - No global state — pass everything through AgentContext
  - No spawning unbounded tasks — use JoinSet with limits

  **Recommended Agent Profile**:
  - **Category**: `deep`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: NO — foundation for Tasks 18-21
  - **Parallel Group**: Wave 4 (first)
  - **Blocks**: Tasks 18, 19, 20
  - **Blocked By**: Tasks 7, 11, 12-15

  **Acceptance Criteria**:
  - [ ] MessageBus publish/subscribe works across tasks
  - [ ] AgentRunner executes mock agent loop correctly
  - [ ] Tool calls are dispatched and results returned to model
  - [ ] Timeout kills long-running agent invocations

  **QA Scenarios**:
  ```
  Scenario: Agent loop processes tool calls correctly
    Tool: Bash
    Steps:
      1. Create MockModelProvider that returns a tool_use for "read_file"
      2. Run AgentRunner with this mock
      3. Assert read_file tool was called with correct args
      4. Assert model received tool result in follow-up message
      5. Assert loop terminates when model returns final text response
    Expected Result: Tool call → execution → result fed back → final response
    Evidence: .sisyphus/evidence/task-17-agent-loop.txt
  ```

  **Commit**: YES
  - Message: `feat(agents): add agent runner, message bus, and tool calling protocol`
  - Pre-commit: `cargo test -p cuttlefish-agents`

- [ ] 18. cuttlefish-agents: Orchestrator Agent

  **What to do**:
  - Implement `OrchestratorAgent` that implements `Agent` trait:
    - Receives user project descriptions
    - Analyzes requirements and creates a task plan
    - Dispatches sub-tasks to Coder and Critic agents via MessageBus
    - Tracks task progress and reports status
    - Handles agent failures (retry, escalate, report)
  - System prompt: "You are the Orchestrator agent for Cuttlefish. You receive project descriptions, break them into tasks, and delegate to specialized agents..."
  - Task tracking: maintains in-memory task list with status (pending/running/completed/failed)
  - Max delegation depth: 1 (Orchestrator → Coder/Critic, no sub-sub-agents in v1)
  - Tests: task creation, delegation, completion tracking

  **Recommended Agent Profile**:
  - **Category**: `deep`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 19, 20)
  - **Blocks**: Task 21
  - **Blocked By**: Task 17

  **Acceptance Criteria**:
  - [ ] Orchestrator creates task plan from description
  - [ ] Tasks dispatched to correct agent roles
  - [ ] Progress tracking works
  - [ ] Agent failure triggers retry

  **QA Scenarios**:
  ```
  Scenario: Orchestrator delegates coding task to Coder agent
    Tool: Bash
    Steps:
      1. Send "Create a hello world Node.js project" to Orchestrator
      2. Assert Orchestrator creates task plan
      3. Assert task published to coder agent topic on MessageBus
      4. Simulate Coder completing → assert Orchestrator marks task done
    Expected Result: Full delegation lifecycle works
    Evidence: .sisyphus/evidence/task-18-orchestrator.txt
  ```

  **Commit**: YES
  - Message: `feat(agents): add Orchestrator agent with task delegation`
  - Pre-commit: `cargo test -p cuttlefish-agents`

- [ ] 19. cuttlefish-agents: Coder Agent

  **What to do**:
  - Implement `CoderAgent` that implements `Agent` trait:
    - Receives coding tasks from Orchestrator
    - Tools available: `read_file`, `write_file`, `execute_command`, `search_files`, `list_directory`
    - Executes in project's Docker sandbox
    - Can: create files, edit files, run build commands, run tests, install dependencies
    - Reports results back via MessageBus
  - System prompt: "You are the Coder agent for Cuttlefish. You write code, run builds, and execute tests inside a Docker sandbox..."
  - Output: structured result with files_changed, commands_run, build_status, test_status
  - Tests: mock sandbox, verify file operations, command execution

  **Recommended Agent Profile**:
  - **Category**: `deep`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 18, 20)
  - **Blocks**: Task 21
  - **Blocked By**: Task 17

  **Acceptance Criteria**:
  - [ ] Coder creates and modifies files in sandbox
  - [ ] Build commands execute and output captured
  - [ ] Results reported back with structured format

  **QA Scenarios**:
  ```
  Scenario: Coder creates a file and runs a build
    Tool: Bash
    Steps:
      1. Give Coder task: "Create index.js with console.log('hello')"
      2. Assert write_file tool called with correct path and content
      3. Assert execute_command called with "node index.js"
      4. Assert output contains "hello"
    Expected Result: File created, command executed, output captured
    Evidence: .sisyphus/evidence/task-19-coder.txt
  ```

  **Commit**: YES
  - Message: `feat(agents): add Coder agent with file ops and build execution`
  - Pre-commit: `cargo test -p cuttlefish-agents`

- [ ] 20. cuttlefish-agents: Critic Agent

  **What to do**:
  - Implement `CriticAgent` that implements `Agent` trait:
    - Receives code review tasks from Orchestrator
    - Tools: `read_file`, `execute_command` (for running tests/linters), `git_diff`
    - Reviews code changes, identifies issues
    - Runs tests if test infrastructure exists
    - Reports: approval or rejection with specific feedback
  - System prompt: "You are the Critic agent for Cuttlefish. You review code changes for quality, correctness, and adherence to project standards..."
  - Output: structured review with verdict (approve/reject), issues list, test results
  - Max back-and-forth with Coder: 5 iterations (Metis guardrail)
  - Tests: review flow with mock model

  **Recommended Agent Profile**:
  - **Category**: `deep`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 18, 19)
  - **Blocks**: Task 21
  - **Blocked By**: Task 17

  **Acceptance Criteria**:
  - [ ] Critic reads diffs and provides structured feedback
  - [ ] Test execution works via sandbox
  - [ ] Approval/rejection verdict is clear
  - [ ] Max iteration limit enforced

  **QA Scenarios**:
  ```
  Scenario: Critic reviews and rejects bad code
    Tool: Bash
    Steps:
      1. Mock model returns review with 2 issues found
      2. Assert Critic produces rejection verdict
      3. Assert issues list has 2 entries with file:line references
    Expected Result: Structured rejection with actionable feedback
    Evidence: .sisyphus/evidence/task-20-critic.txt
  ```

  **Commit**: YES
  - Message: `feat(agents): add Critic agent with code review and test execution`
  - Pre-commit: `cargo test -p cuttlefish-agents`

- [ ] 21. cuttlefish-agents: Planner→Coder→Critic Loop

  **What to do**:
  - Implement the full agent workflow loop:
    1. Orchestrator receives project description
    2. Orchestrator creates task plan (using model to analyze requirements)
    3. For each task: Orchestrator dispatches to Coder
    4. Coder completes work, reports result
    5. Orchestrator dispatches result to Critic
    6. Critic reviews: APPROVE → move to next task, REJECT → send feedback to Coder
    7. Coder→Critic loop max 5 iterations per task
    8. After all tasks: Orchestrator reports final status
  - Implement `WorkflowEngine` that orchestrates this loop
  - Handle: Coder failure (retry once, then escalate), Critic infinite loop (max 5), all-tasks-complete
  - Integrate with ContextManager for conversation history
  - Integration test: full loop with mock providers

  **Recommended Agent Profile**:
  - **Category**: `deep`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: NO — integrates Tasks 18-20
  - **Blocks**: Tasks 22-29, 44-46
  - **Blocked By**: Tasks 18, 19, 20

  **Acceptance Criteria**:
  - [ ] Full loop executes: plan → code → review → approve
  - [ ] Rejection triggers Coder retry with feedback
  - [ ] Max iteration limit stops infinite loops
  - [ ] Final status reports all tasks and their outcomes

  **QA Scenarios**:
  ```
  Scenario: Complete workflow with one rejection and retry
    Tool: Bash
    Steps:
      1. Setup: Orchestrator plans 1 task, Coder implements, Critic rejects first time, approves second
      2. Run workflow engine
      3. Assert Coder called twice (original + retry)
      4. Assert Critic called twice
      5. Assert final status is "completed"
    Expected Result: Workflow completes after 1 rejection + 1 retry
    Evidence: .sisyphus/evidence/task-21-workflow-loop.txt
  ```

  **Commit**: YES
  - Message: `feat(agents): add Planner→Coder→Critic workflow loop`
  - Pre-commit: `cargo test -p cuttlefish-agents`

---

### Wave 5 — Discord Interface

- [ ] 22. cuttlefish-discord: Bot Setup + Slash Commands

  **What to do**:
  - Set up serenity Discord bot:
    - Gateway intents: GUILDS, GUILD_MESSAGES, MESSAGE_CONTENT
    - Event handler: Ready, InteractionCreate, Message
  - Implement slash commands:
    - `/project create <name> <description>` — creates new project
    - `/project list` — lists active projects
    - `/project status <name>` — shows project status
    - `/project cancel <name>` — cancels a project
    - `/help` — shows available commands
  - Bot presence: "🐙 Cuttlefish | /help"
  - Error handling: graceful responses for invalid commands
  - Tests: command parsing, response formatting (without real Discord connection)

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: NO — foundation for Wave 5
  - **Blocks**: Tasks 23-25
  - **Blocked By**: Task 21

  **Acceptance Criteria**:
  - [ ] Bot connects to Discord (integration test)
  - [ ] Slash commands registered and respond
  - [ ] Error handling produces user-friendly messages

  **QA Scenarios**:
  ```
  Scenario: Slash command parsing works correctly
    Tool: Bash
    Steps:
      1. Simulate `/project create my-app A cool web app` interaction
      2. Assert parsed: name="my-app", description="A cool web app"
      3. Simulate `/project list` — assert returns formatted project list
    Expected Result: Commands parsed correctly, responses formatted
    Evidence: .sisyphus/evidence/task-22-slash-commands.txt
  ```

  **Commit**: YES
  - Message: `feat(discord): add bot setup with slash commands`
  - Pre-commit: `cargo test -p cuttlefish-discord`

- [ ] 23. cuttlefish-discord: Channel-Per-Project Management

  **What to do**:
  - On `/project create`:
    1. Create Discord category "🐙 Cuttlefish Projects" if not exists
    2. Create text channel `#project-{name}` under category
    3. Set channel topic to project description
    4. Pin initial message with project details
    5. Store channel_id ↔ project_id mapping in DB
  - On project completion/cancellation:
    - Archive channel (move to "Archived" category) or mark read-only
  - Message routing: messages in project channels are routed to that project's Orchestrator
  - Handle: duplicate names, channel limit (500 per server), permission issues

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: NO — depends on Task 22
  - **Blocks**: Tasks 24, 25
  - **Blocked By**: Task 22

  **Acceptance Criteria**:
  - [ ] Channel created under Cuttlefish category
  - [ ] Channel ↔ project mapping stored in DB
  - [ ] Messages in project channel reach project's Orchestrator
  - [ ] Archival works on completion

  **QA Scenarios**:
  ```
  Scenario: Project creation creates Discord channel
    Tool: Bash
    Steps:
      1. Call create_project_channel("my-app", "A web app", guild_id)
      2. Assert channel created with name "#project-my-app"
      3. Assert DB has mapping for this channel → project
      4. Assert pinned message contains project details
    Expected Result: Channel created, mapped, and initialized
    Evidence: .sisyphus/evidence/task-23-channel-create.txt
  ```

  **Commit**: YES
  - Message: `feat(discord): add channel-per-project management`
  - Pre-commit: `cargo test -p cuttlefish-discord`

- [ ] 24. cuttlefish-discord: Message Routing + Output Formatting

  **What to do**:
  - Route messages from project channels to project's agent system:
    - On message in `#project-X` → lookup project by channel_id → forward to Orchestrator
    - Orchestrator responses → format as Discord messages → send to channel
  - Output formatting:
    - Code blocks with language syntax highlighting
    - Embed messages for status updates (green=success, red=failure, yellow=in-progress)
    - File attachments for diffs > 2000 chars
    - Split messages > 2000 chars into multiple messages
  - Handle: rate limiting (5 msg/5s per channel), message too long, bot mentioned vs direct message

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: NO — depends on Task 23
  - **Blocks**: Tasks 25, 44
  - **Blocked By**: Task 23

  **Acceptance Criteria**:
  - [ ] Messages route from channel to correct project agent
  - [ ] Agent responses formatted with code blocks and embeds
  - [ ] Long messages split correctly at 2000 char boundary
  - [ ] Rate limiting respected

  **QA Scenarios**:
  ```
  Scenario: Long code output splits into multiple messages
    Tool: Bash
    Steps:
      1. Generate 5000-char code output
      2. Format for Discord
      3. Assert split into 3 messages (2000+2000+1000)
      4. Assert each maintains valid code block formatting
    Expected Result: Clean split without breaking code blocks
    Evidence: .sisyphus/evidence/task-24-message-split.txt
  ```

  **Commit**: YES
  - Message: `feat(discord): add message routing and output formatting`
  - Pre-commit: `cargo test -p cuttlefish-discord`

- [ ] 25. cuttlefish-discord: Multi-Server Support

  **What to do**:
  - Support bot running in multiple Discord servers simultaneously:
    - Per-guild configuration (which features enabled, resource limits)
    - Guild-scoped project isolation (projects in server A invisible to server B)
    - Shard support for > 2500 guilds (serenity auto-sharding)
  - On guild join: create welcome message, setup categories
  - On guild leave: archive/cleanup (configurable)
  - Per-guild settings stored in DB

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: NO — depends on Task 23
  - **Blocks**: Task 44
  - **Blocked By**: Task 23

  **Acceptance Criteria**:
  - [ ] Bot functions in multiple guilds independently
  - [ ] Projects are guild-scoped
  - [ ] Guild settings stored and applied

  **QA Scenarios**:
  ```
  Scenario: Projects isolated between guilds
    Tool: Bash
    Steps:
      1. Create project "app-a" in guild_1
      2. List projects in guild_2 — assert empty
      3. Assert project "app-a" only appears in guild_1 listing
    Expected Result: Complete guild isolation
    Evidence: .sisyphus/evidence/task-25-multi-server.txt
  ```

  **Commit**: YES
  - Message: `feat(discord): add multi-server support with guild isolation`
  - Pre-commit: `cargo test -p cuttlefish-discord`

---

### Wave 6 — API Server

- [ ] 26. cuttlefish-api: Axum Server + WebSocket Handler

  **What to do**:
  - Create axum HTTP/WebSocket server:
    - Bind to configurable host:port from `cuttlefish.toml`
    - WebSocket endpoint: `/ws` — bidirectional communication
    - Health check: `GET /health` → 200 OK
    - Graceful shutdown on SIGTERM/SIGINT
  - WebSocket protocol (JSON messages):
    - Client → Server: `{ "type": "chat", "project_id": "...", "content": "..." }`
    - Server → Client: `{ "type": "response", "project_id": "...", "content": "...", "agent": "coder" }`
    - Server → Client: `{ "type": "build_log", "project_id": "...", "line": "..." }` (streaming)
    - Server → Client: `{ "type": "diff", "project_id": "...", "patch": "..." }`
  - Connection management: track active WebSocket connections, broadcast project events to subscribers
  - Tests: WebSocket connection, message serialization, health check

  **Recommended Agent Profile**:
  - **Category**: `deep`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (Wave 5 and 6 can run in parallel)
  - **Blocks**: Tasks 27-29, 30-37
  - **Blocked By**: Task 21

  **Acceptance Criteria**:
  - [ ] Server starts and responds to health check
  - [ ] WebSocket connects and exchanges messages
  - [ ] Multiple concurrent WebSocket connections work
  - [ ] Graceful shutdown works

  **QA Scenarios**:
  ```
  Scenario: WebSocket chat message round-trip
    Tool: Bash (curl + websocat)
    Steps:
      1. Start server on port 8080
      2. Connect WebSocket to ws://localhost:8080/ws
      3. Send chat message JSON
      4. Assert response received with agent field populated
      5. Disconnect — assert clean close
    Expected Result: Message sent and response received via WebSocket
    Evidence: .sisyphus/evidence/task-26-websocket.txt
  ```

  **Commit**: YES
  - Message: `feat(api): add axum server with WebSocket handler`
  - Pre-commit: `cargo test -p cuttlefish-api`

- [ ] 27. cuttlefish-api: REST Endpoints

  **What to do**:
  - Implement REST API endpoints:
    - `POST /api/projects` — create project
    - `GET /api/projects` — list projects
    - `GET /api/projects/:id` — get project details
    - `DELETE /api/projects/:id` — cancel project
    - `GET /api/projects/:id/conversations` — get conversation history
    - `GET /api/projects/:id/builds` — get build logs
    - `GET /api/projects/:id/diff` — get current diff
    - `POST /api/projects/:id/message` — send message (non-WS fallback)
  - JSON request/response with proper error codes (400, 401, 404, 500)
  - Request validation with descriptive errors
  - Tests: each endpoint with mock data

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 28, 29)
  - **Blocks**: Task 30
  - **Blocked By**: Task 26

  **Acceptance Criteria**:
  - [ ] All 8 endpoints respond with correct status codes
  - [ ] Invalid requests return 400 with descriptive error
  - [ ] Unauthenticated requests return 401

  **QA Scenarios**:
  ```
  Scenario: Create project via REST API
    Tool: Bash (curl)
    Steps:
      1. POST /api/projects with {"name":"test","description":"Test project"}
      2. Assert 201 Created with project ID in response
      3. GET /api/projects/:id — assert 200 with correct data
    Expected Result: Project created and retrievable via API
    Evidence: .sisyphus/evidence/task-27-rest-create.txt
  ```

  **Commit**: YES
  - Message: `feat(api): add REST endpoints for projects, conversations, builds`
  - Pre-commit: `cargo test -p cuttlefish-api`

- [ ] 28. cuttlefish-api: API Key Authentication Middleware

  **What to do**:
  - Implement tower middleware for API key auth:
    - Check `Authorization: Bearer {api_key}` header
    - Validate against configured API key(s) in `cuttlefish.toml`
    - Skip auth for health check endpoint
    - Return 401 with `{"error": "Invalid or missing API key"}` on failure
  - API key generation: `cuttlefish generate-key` CLI command that produces a random 32-byte hex key
  - Support multiple API keys (for key rotation)
  - Tests: valid key passes, invalid key rejected, missing key rejected

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 27, 29)
  - **Blocks**: Tasks 30, 35
  - **Blocked By**: Task 26

  **Acceptance Criteria**:
  - [ ] Valid API key passes middleware
  - [ ] Invalid/missing key returns 401
  - [ ] Health check bypasses auth
  - [ ] Key generation produces valid random key

  **QA Scenarios**:
  ```
  Scenario: Auth middleware rejects invalid key
    Tool: Bash (curl)
    Steps:
      1. GET /api/projects with no auth header → assert 401
      2. GET /api/projects with wrong key → assert 401
      3. GET /api/projects with valid key → assert 200
      4. GET /health with no auth → assert 200 (bypassed)
    Expected Result: Auth enforced on API, bypassed on health
    Evidence: .sisyphus/evidence/task-28-auth.txt
  ```

  **Commit**: YES
  - Message: `feat(api): add API key authentication middleware`
  - Pre-commit: `cargo test -p cuttlefish-api`

- [ ] 29. cuttlefish-api: Reverse Proxy for Dev Servers

  **What to do**:
  - Implement simple TCP/HTTP reverse proxy:
    - When a project's Docker container runs a dev server (e.g., `npm run dev` on port 3000)
    - Cuttlefish exposes it at `http://{host}:{proxy_port}/project/{name}/`
    - Proxy forwards HTTP requests to the container's exposed port
    - Supports WebSocket upgrade (for HMR/live reload)
  - Route management: add/remove proxy routes when containers start/stop
  - Handle: container not running, port not exposed, timeout
  - Tests: proxy forwards request and returns response

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 27, 28)
  - **Blocks**: None
  - **Blocked By**: Task 26

  **Acceptance Criteria**:
  - [ ] HTTP request proxied to container port
  - [ ] WebSocket upgrade forwarded
  - [ ] 502 returned when container not running

  **QA Scenarios**:
  ```
  Scenario: Reverse proxy forwards to container
    Tool: Bash (curl)
    Steps:
      1. Start container with `python3 -m http.server 8000`
      2. Register proxy route: project "test" → container:8000
      3. curl http://localhost:{proxy_port}/project/test/ → assert 200
      4. Stop container → curl same URL → assert 502
    Expected Result: Proxy works when container up, 502 when down
    Evidence: .sisyphus/evidence/task-29-proxy.txt
  ```

  **Commit**: YES
  - Message: `feat(api): add reverse proxy for project dev servers`
  - Pre-commit: `cargo test -p cuttlefish-api`

---

### Wave 7 — WebUI (Nuxt)

- [ ] 30. cuttlefish-web: Nuxt Project Setup + WebSocket Client

  **What to do**:
  - Initialize Nuxt 3 project in `cuttlefish-web/` directory
  - Configure: TypeScript, Tailwind CSS, auto-imports
  - Implement WebSocket client composable:
    - `useWebSocket(url, apiKey)` — connects, handles reconnect, auth
    - Message types: chat, build_log, diff, status
    - Auto-reconnect on disconnect with exponential backoff
  - Layout: sidebar (project list) + main (chat/content area) + header
  - Pages: `/` (dashboard), `/project/:id` (project view)
  - API client: REST calls to cuttlefish-api for CRUD operations
  - Tests: WebSocket composable unit tests

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: [`frontend-ui-ux`]

  **Parallelization**:
  - **Can Run In Parallel**: NO — foundation for Wave 7
  - **Blocks**: Tasks 31-34
  - **Blocked By**: Tasks 27, 28

  **Acceptance Criteria**:
  - [ ] Nuxt dev server starts without errors
  - [ ] WebSocket connects to cuttlefish-api
  - [ ] Dashboard page renders with project list
  - [ ] Navigation between pages works

  **QA Scenarios**:
  ```
  Scenario: Nuxt app starts and WebSocket connects
    Tool: Playwright
    Steps:
      1. Start Nuxt dev server
      2. Navigate to http://localhost:3000
      3. Assert page title contains "Cuttlefish"
      4. Assert WebSocket connection established (check browser console)
      5. Assert project list renders (even if empty)
    Expected Result: App loads, WS connects, UI renders
    Evidence: .sisyphus/evidence/task-30-nuxt-setup.png
  ```

  **Commit**: YES
  - Message: `feat(web): initialize Nuxt project with WebSocket client`
  - Pre-commit: `cd cuttlefish-web && npm test`

- [ ] 31. cuttlefish-web: Chat Interface with Code Blocks

  **What to do**:
  - Build chat UI component:
    - Message list with user/assistant differentiation
    - Markdown rendering with syntax-highlighted code blocks
    - Input textarea with send button and Enter-to-send
    - Auto-scroll to newest message
    - Typing indicator when agent is responding
    - Streaming: display agent response as it arrives via WebSocket
  - Message actions: copy code block, expand/collapse long messages
  - Mobile-responsive layout

  **Recommended Agent Profile**:
  - **Category**: `visual-engineering`
  - **Skills**: [`frontend-ui-ux`]

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 32-34)
  - **Blocks**: Task 45
  - **Blocked By**: Task 30

  **Acceptance Criteria**:
  - [ ] Messages render with correct formatting
  - [ ] Code blocks have syntax highlighting
  - [ ] Streaming response displays incrementally
  - [ ] Auto-scroll works

  **QA Scenarios**:
  ```
  Scenario: Chat message with code block renders correctly
    Tool: Playwright
    Steps:
      1. Navigate to project chat
      2. Send message "Create hello.py"
      3. Wait for response with code block
      4. Assert code block has Python syntax highlighting (class="language-python")
      5. Assert copy button visible on hover
    Expected Result: Code block rendered with syntax highlighting
    Evidence: .sisyphus/evidence/task-31-chat-codeblock.png
  ```

  **Commit**: YES
  - Message: `feat(web): add chat interface with streaming and code blocks`

- [ ] 32. cuttlefish-web: Build Log Viewer (Streaming)

  **What to do**:
  - Build log viewer component:
    - Real-time streaming of build output via WebSocket build_log messages
    - ANSI color code support (convert to HTML spans)
    - Auto-scroll with "scroll lock" toggle
    - Status indicator: running (yellow), success (green), failure (red)
    - Timestamp per line
    - Filter: show/hide stdout vs stderr
  - Tab in project view alongside chat

  **Recommended Agent Profile**:
  - **Category**: `visual-engineering`
  - **Skills**: [`frontend-ui-ux`]

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 31, 33, 34)
  - **Blocks**: Task 45
  - **Blocked By**: Task 30

  **Acceptance Criteria**:
  - [ ] Build logs stream in real-time
  - [ ] ANSI colors rendered correctly
  - [ ] Status indicator updates
  - [ ] Scroll lock works

  **QA Scenarios**:
  ```
  Scenario: Build log streams in real-time
    Tool: Playwright
    Steps:
      1. Navigate to project build logs tab
      2. Trigger a build (or simulate via WS)
      3. Assert log lines appear one-by-one (not all at once)
      4. Assert ANSI green text renders as green span
      5. Assert status shows "running" then "success"
    Expected Result: Real-time log streaming with colors
    Evidence: .sisyphus/evidence/task-32-build-log.png
  ```

  **Commit**: YES
  - Message: `feat(web): add streaming build log viewer`

- [ ] 33. cuttlefish-web: File Diff Viewer

  **What to do**:
  - Build diff viewer component:
    - Render unified diff format with added/removed/context lines
    - Side-by-side or unified view toggle
    - File grouping (multiple files in one diff)
    - Syntax highlighting within diff
    - Line numbers for both old and new files
  - Triggered when agent completes file changes
  - Diff data comes from git diff via WebSocket

  **Recommended Agent Profile**:
  - **Category**: `visual-engineering`
  - **Skills**: [`frontend-ui-ux`]

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 31, 32, 34)
  - **Blocks**: Task 45
  - **Blocked By**: Task 30

  **Acceptance Criteria**:
  - [ ] Unified diff renders with color coding (green=added, red=removed)
  - [ ] Multiple files grouped correctly
  - [ ] Side-by-side toggle works
  - [ ] Line numbers displayed

  **QA Scenarios**:
  ```
  Scenario: Diff viewer shows file changes
    Tool: Playwright
    Steps:
      1. Navigate to project diff tab
      2. Provide sample unified diff via WS
      3. Assert added lines shown in green
      4. Assert removed lines shown in red
      5. Assert file headers visible
      6. Toggle side-by-side — assert layout changes
    Expected Result: Diff rendered with correct styling
    Evidence: .sisyphus/evidence/task-33-diff-viewer.png
  ```

  **Commit**: YES
  - Message: `feat(web): add file diff viewer with side-by-side toggle`

- [ ] 34. cuttlefish-web: Project Dashboard + Management

  **What to do**:
  - Dashboard page (`/`):
    - List all projects with: name, status, last activity, GitHub link
    - Create new project button → modal with name + description + template selection
    - Search/filter projects
  - Project management:
    - Project settings page: change name, update description, view Docker status
    - Cancel/archive project action
    - View GitHub repo link, Actions status
  - Template selector: dropdown of available templates loaded from API

  **Recommended Agent Profile**:
  - **Category**: `visual-engineering`
  - **Skills**: [`frontend-ui-ux`]

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 31-33)
  - **Blocks**: Task 45
  - **Blocked By**: Task 30

  **Acceptance Criteria**:
  - [ ] Dashboard lists all projects
  - [ ] Create project modal works
  - [ ] Project settings page renders
  - [ ] Cancel project action works

  **QA Scenarios**:
  ```
  Scenario: Create project from dashboard
    Tool: Playwright
    Steps:
      1. Navigate to dashboard
      2. Click "New Project" button
      3. Fill in name="test-app", description="A test app"
      4. Select template "node-base"
      5. Click Create
      6. Assert project appears in list with status "Active"
    Expected Result: Project created and visible on dashboard
    Evidence: .sisyphus/evidence/task-34-dashboard.png
  ```

  **Commit**: YES
  - Message: `feat(web): add project dashboard and management`

---

### Wave 8 — TUI Client

- [ ] 35. cuttlefish-tui: Ratatui App + WebSocket Connection

  **What to do**:
  - Create TUI binary using ratatui + crossterm:
    - Full-screen terminal UI
    - WebSocket connection to remote server
    - Configuration: server URL, API key (from `~/.cuttlefish/config.toml`)
    - Connection status indicator
    - Auto-reconnect on disconnect
  - Layout: left panel (project list), right panel (chat/content), bottom (input)
  - Keyboard shortcuts: Tab (switch panels), Ctrl-C (quit), Enter (send)
  - CLI args: `--server ws://host:port`, `--api-key KEY`

  **Recommended Agent Profile**:
  - **Category**: `deep`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Wave 7)
  - **Blocks**: Tasks 36, 37
  - **Blocked By**: Tasks 26, 28

  **Acceptance Criteria**:
  - [ ] TUI renders full-screen terminal UI
  - [ ] WebSocket connects to server
  - [ ] Project list populates
  - [ ] Keyboard shortcuts work

  **QA Scenarios**:
  ```
  Scenario: TUI starts and connects to server
    Tool: interactive_bash (tmux)
    Steps:
      1. Start cuttlefish server
      2. Launch TUI: `./target/release/cuttlefish-tui --server ws://localhost:8080`
      3. Assert connection status shows "Connected"
      4. Assert project list panel renders
      5. Press Ctrl-C — assert clean exit
    Expected Result: TUI renders, connects, and exits cleanly
    Evidence: .sisyphus/evidence/task-35-tui-connect.txt
  ```

  **Commit**: YES
  - Message: `feat(tui): add ratatui TUI client with WebSocket connection`
  - Pre-commit: `cargo build -p cuttlefish-tui`

- [ ] 36. cuttlefish-tui: Chat View + Input Handling

  **What to do**:
  - Chat view in right panel:
    - Message history with user/assistant labels
    - Syntax-highlighted code blocks (using syntect or similar)
    - Auto-scroll with scroll-back support (arrow keys)
    - Streaming response display
  - Input handling:
    - Multi-line input textarea at bottom
    - Enter sends, Shift+Enter for newline
    - Input history (up/down arrows)
    - Command mode: `:project create ...`, `:help`, `:quit`

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Task 37)
  - **Blocks**: Task 46
  - **Blocked By**: Task 35

  **Acceptance Criteria**:
  - [ ] Messages display with formatting
  - [ ] Input sends via WebSocket
  - [ ] Scroll-back works
  - [ ] Command mode parses commands

  **QA Scenarios**:
  ```
  Scenario: Send message and receive response in TUI
    Tool: interactive_bash (tmux)
    Steps:
      1. Launch TUI connected to server
      2. Type "Create a hello world app" and press Enter
      3. Assert message appears in chat
      4. Wait for agent response
      5. Assert response appears with agent label
    Expected Result: Message sent, response received and displayed
    Evidence: .sisyphus/evidence/task-36-tui-chat.txt
  ```

  **Commit**: YES
  - Message: `feat(tui): add chat view with input handling and command mode`
  - Pre-commit: `cargo build -p cuttlefish-tui`

- [ ] 37. cuttlefish-tui: Diff View + Build Log View

  **What to do**:
  - Diff view (Tab to switch):
    - Render unified diff with color coding (green=added, red=removed)
    - Scrollable
    - File headers as separators
  - Build log view (Tab to switch):
    - Streaming log output
    - ANSI color rendering in terminal
    - Status indicator
  - View switching: Tab cycles through Chat → Diff → Build Log
  - Panel indicator showing current view name

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Task 36)
  - **Blocks**: Task 46
  - **Blocked By**: Task 35

  **Acceptance Criteria**:
  - [ ] Diff view renders colored diff
  - [ ] Build log streams in real-time
  - [ ] Tab switching works between all 3 views

  **QA Scenarios**:
  ```
  Scenario: Tab switching between views
    Tool: interactive_bash (tmux)
    Steps:
      1. Launch TUI with active project
      2. Assert chat view is default
      3. Press Tab — assert diff view active
      4. Press Tab — assert build log view active
      5. Press Tab — assert back to chat view
    Expected Result: Clean view cycling with correct content
    Evidence: .sisyphus/evidence/task-37-tui-views.txt
  ```

  **Commit**: YES
  - Message: `feat(tui): add diff view and build log view`
  - Pre-commit: `cargo build -p cuttlefish-tui`

---

### Wave 9 — Advanced Features

- [ ] 38. Self-Update: GitHub Actions Build + Binary Pull + Restart

  **What to do**:
  - Implement self-update mechanism:
    - On startup and periodically (configurable, default 5 min): check GitHub Releases API for latest release
    - Compare current version (from `Cargo.toml`) with latest release tag
    - If newer: download binary asset, verify checksum, replace current binary
    - Gracefully shutdown: stop accepting new tasks, wait for active tasks to complete (max 60s), restart
  - Use the release workflow from Task 5 to build and publish releases
  - Rollback: keep previous binary as `cuttlefish-rs.bak`, restore if new binary fails to start
  - Tests: version comparison, download mock, rollback flow

  **Recommended Agent Profile**:
  - **Category**: `deep`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 39-43)
  - **Blocks**: None
  - **Blocked By**: Task 5

  **Acceptance Criteria**:
  - [ ] Version comparison detects newer release
  - [ ] Binary download and replacement works
  - [ ] Rollback restores previous binary on failure
  - [ ] Graceful shutdown waits for active tasks

  **QA Scenarios**:
  ```
  Scenario: Self-update detects and applies new version
    Tool: Bash
    Steps:
      1. Mock GitHub API to return version "0.2.0" (newer than current "0.1.0")
      2. Trigger update check
      3. Assert download initiated
      4. Assert binary replaced
      5. Assert old binary saved as .bak
    Expected Result: Update detected, downloaded, applied
    Evidence: .sisyphus/evidence/task-38-self-update.txt
  ```

  **Commit**: YES
  - Message: `feat(core): add self-update mechanism via GitHub Releases`
  - Pre-commit: `cargo test --workspace`

- [ ] 39. GitHub Actions Monitoring

  **What to do**:
  - Implement Actions monitor:
    - For each project with a GitHub repo: periodically poll workflow runs
    - Report status changes (queued → in_progress → completed)
    - On failure: fetch logs, extract error summary, report to project channel
    - Configurable poll interval (default 30s)
  - Integration with Discord: post status embeds in project channels
  - Integration with WebUI: push status via WebSocket
  - Tests: status polling mock, error extraction

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 38, 40-43)
  - **Blocks**: None
  - **Blocked By**: Task 15

  **Acceptance Criteria**:
  - [ ] Workflow runs detected and status tracked
  - [ ] Failure logs extracted and summarized
  - [ ] Status reported to Discord and WebUI

  **QA Scenarios**:
  ```
  Scenario: Build failure detected and reported
    Tool: Bash
    Steps:
      1. Mock GitHub API: workflow run with conclusion "failure"
      2. Mock logs with a build error
      3. Assert monitor detects failure
      4. Assert error summary extracted
      5. Assert notification sent to project topic on MessageBus
    Expected Result: Failure detected, error extracted, notification sent
    Evidence: .sisyphus/evidence/task-39-actions-monitor.txt
  ```

  **Commit**: YES
  - Message: `feat(vcs): add GitHub Actions workflow monitoring`
  - Pre-commit: `cargo test -p cuttlefish-vcs`

- [ ] 40. Template System (.MD Reading + Project Scaffolding)

  **What to do**:
  - Implement template system:
    - Templates stored in `templates/` directory as `.md` files
    - Each template describes: project structure, dependencies, build commands, deployment target
    - Template format: markdown with structured sections (## Dependencies, ## Structure, ## Build, ## Deploy)
    - On project creation: Orchestrator reads template, instructs Coder to follow it
  - Create initial templates:
    - `nuxt-cloudflare.md`: Nuxt 3 + Cloudflare Pages deployment
    - `node-express.md`: Express.js API
    - `rust-axum.md`: Rust + axum web service
    - `python-fastapi.md`: Python FastAPI
    - `static-site.md`: HTML/CSS/JS static site
  - Template selection: via Discord command, WebUI dropdown, or TUI command
  - Generate CLAUDE.md and README.md from template content + project description

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 38, 39, 41-43)
  - **Blocks**: None
  - **Blocked By**: Tasks 10, 16

  **Acceptance Criteria**:
  - [ ] Templates load from `templates/` directory
  - [ ] 5 initial templates created
  - [ ] Orchestrator reads and follows template instructions
  - [ ] CLAUDE.md and README.md generated per template

  **QA Scenarios**:
  ```
  Scenario: Nuxt template scaffolds correct project
    Tool: Bash
    Steps:
      1. Select "nuxt-cloudflare" template for new project
      2. Assert template content read successfully
      3. Assert CLAUDE.md contains Nuxt conventions
      4. Assert README.md contains project description + Nuxt setup instructions
    Expected Result: Template applied, docs generated correctly
    Evidence: .sisyphus/evidence/task-40-template.txt
  ```

  **Commit**: YES
  - Message: `feat(core): add template system with 5 initial templates`
  - Pre-commit: `cargo test --workspace`

- [ ] 41. Context Summarization (Automatic)

  **What to do**:
  - Implement automatic context summarization:
    - Trigger: when conversation exceeds configurable threshold (default: 50 messages or 30K tokens)
    - Use a cheap/fast model (via category "quick") to summarize older messages
    - Summary stored as system message in DB, old messages marked archived
    - Summary prompt: "Summarize the following conversation, preserving: key decisions, code changes made, current state, outstanding tasks"
  - Configurable: enable/disable, threshold, model to use for summarization
  - Tests: trigger detection, summary quality (via mock)

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 38-40, 42-43)
  - **Blocks**: None
  - **Blocked By**: Task 11

  **Acceptance Criteria**:
  - [ ] Summarization triggers at threshold
  - [ ] Summary captures key information
  - [ ] Old messages archived, summary inserted
  - [ ] Subsequent context builds include summary

  **QA Scenarios**:
  ```
  Scenario: Auto-summarization triggers at threshold
    Tool: Bash
    Steps:
      1. Insert 60 messages for a project (above 50 threshold)
      2. Trigger context build
      3. Assert summarization was called
      4. Assert archived messages count > 0
      5. Assert summary message exists in DB
    Expected Result: Summarization triggered, old messages archived
    Evidence: .sisyphus/evidence/task-41-summarize.txt
  ```

  **Commit**: YES
  - Message: `feat(core): add automatic context summarization`
  - Pre-commit: `cargo test -p cuttlefish-core`

- [ ] 42. cuttlefish-vcs: GitHub App Authentication

  **What to do**:
  - Implement GitHub App auth as alternative to PAT:
    - JWT generation from App private key
    - Installation token exchange
    - Token caching and refresh (expires after 1 hour)
    - Automatic token selection: if GitHub App configured, use it; else fall back to PAT
  - Configuration: app_id, private_key_path, installation_id in `cuttlefish.toml`
  - Tests: JWT generation, token exchange (mocked)

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 38-41, 43)
  - **Blocks**: None
  - **Blocked By**: Task 15

  **Acceptance Criteria**:
  - [ ] JWT generated correctly from private key
  - [ ] Installation token obtained (mocked)
  - [ ] Token caching prevents unnecessary requests
  - [ ] Fallback to PAT when App not configured

  **QA Scenarios**:
  ```
  Scenario: GitHub App JWT generation
    Tool: Bash
    Steps:
      1. Generate test RSA private key
      2. Create JWT with app_id and key
      3. Decode JWT — assert payload contains correct iss (app_id) and exp
    Expected Result: Valid JWT with correct claims
    Evidence: .sisyphus/evidence/task-42-github-app-jwt.txt
  ```

  **Commit**: YES
  - Message: `feat(vcs): add GitHub App authentication with JWT`
  - Pre-commit: `cargo test -p cuttlefish-vcs`

- [ ] 43. Category-Based Model Routing Config

  **What to do**:
  - Implement category-to-model routing from TOML config:
    - `[categories]` section in `cuttlefish.toml` maps category names to model configs
    - Each category: model ID, provider, temperature, max_tokens, fallback chain
    - `[agents]` section maps agent roles to categories (with optional model override)
  - Implement `ModelRouter`:
    - `route(category: Category) → ModelConfig` — resolves category to model
    - `route_agent(role: AgentRole) → ModelConfig` — resolves agent role → category → model
    - Fallback: if primary model fails, try fallback chain
  - Tests: routing resolution, fallback behavior, config validation

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 38-42)
  - **Blocks**: None
  - **Blocked By**: Tasks 2, 7

  **Acceptance Criteria**:
  - [ ] Category routing resolves from TOML config
  - [ ] Agent role → category → model chain works
  - [ ] Fallback chain activates when primary fails
  - [ ] Invalid config produces descriptive error

  **QA Scenarios**:
  ```
  Scenario: Model routing from config
    Tool: Bash
    Steps:
      1. Create config with deep=claude-opus, quick=claude-haiku
      2. Route Category::Deep — assert resolves to claude-opus
      3. Route AgentRole::Coder (mapped to "deep") — assert resolves to claude-opus
      4. Route Category::Quick — assert resolves to claude-haiku
    Expected Result: Correct model resolved for each category and role
    Evidence: .sisyphus/evidence/task-43-routing.txt
  ```

  **Commit**: YES
  - Message: `feat(core): add category-based model routing from TOML config`
  - Pre-commit: `cargo test -p cuttlefish-core`

---

### Wave 10 — End-to-End Integration

- [ ] 44. End-to-End: Discord → Agent → Docker → GitHub Flow

  **What to do**:
  - Integration test (feature-flagged) that exercises the FULL flow:
    1. Discord bot receives `/project create test-app "A hello world Node.js app"` (simulated)
    2. Channel `#project-test-app` created
    3. Orchestrator plans task: "Create Node.js hello world"
    4. Coder creates files in Docker sandbox (package.json, index.js)
    5. Coder runs `npm install` and `node index.js` — verifies "Hello World" output
    6. Critic reviews code — approves
    7. VCS commits and pushes to GitHub (test repo)
    8. Status reported back in Discord channel
  - This test requires: real Docker, mock or test Discord (serenity test utilities), test GitHub repo
  - Document the full flow in test file for clarity

  **Recommended Agent Profile**:
  - **Category**: `deep`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 45, 46)
  - **Blocks**: F1-F4
  - **Blocked By**: Tasks 24, 25, 21

  **Acceptance Criteria**:
  - [ ] Full flow executes without manual intervention
  - [ ] Docker container created and destroyed correctly
  - [ ] Code committed to GitHub
  - [ ] Status reported in correct format

  **QA Scenarios**:
  ```
  Scenario: Full Discord→GitHub flow
    Tool: Bash
    Steps:
      1. Run integration test: `cargo test -p cuttlefish-rs --features integration e2e_discord_flow`
      2. Assert test passes
      3. Verify test GitHub repo has new commit with Node.js files
    Expected Result: End-to-end flow completes successfully
    Evidence: .sisyphus/evidence/task-44-e2e-discord.txt
  ```

  **Commit**: YES
  - Message: `test: add end-to-end Discord→Agent→Docker→GitHub integration test`
  - Pre-commit: `cargo test --workspace`

- [ ] 45. End-to-End: WebUI → Agent → Docker → GitHub Flow

  **What to do**:
  - Integration test for WebUI flow:
    1. HTTP POST to create project via REST API
    2. WebSocket connect, send chat message describing project
    3. Agent processes, creates files in Docker sandbox
    4. Build and test within sandbox
    5. Commit and push to GitHub
    6. Verify: WebSocket received streaming responses, build logs, final diff
  - Use reqwest/tokio-tungstenite for WebSocket client in test

  **Recommended Agent Profile**:
  - **Category**: `deep`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 44, 46)
  - **Blocks**: F1-F4
  - **Blocked By**: Tasks 31-34, 21

  **Acceptance Criteria**:
  - [ ] REST API creates project
  - [ ] WebSocket receives streaming responses
  - [ ] Build logs stream correctly
  - [ ] Diff shows correct changes

  **QA Scenarios**:
  ```
  Scenario: Full WebUI→GitHub flow via API
    Tool: Bash (curl + websocat)
    Steps:
      1. POST /api/projects — create project
      2. Connect WS, send chat message
      3. Collect all WS messages until completion
      4. Assert received: chat responses, build logs, diff, status
    Expected Result: Full flow via API/WebSocket
    Evidence: .sisyphus/evidence/task-45-e2e-webui.txt
  ```

  **Commit**: YES
  - Message: `test: add end-to-end WebUI→Agent→Docker→GitHub integration test`
  - Pre-commit: `cargo test --workspace`

- [ ] 46. End-to-End: TUI → Server → Agent Flow

  **What to do**:
  - Integration test for TUI flow:
    1. Start cuttlefish server
    2. Launch TUI binary, connect to server
    3. Create project via TUI command `:project create test-app`
    4. Send chat message in TUI
    5. Verify response appears in TUI output
    6. Switch to diff view — verify diff content
    7. Switch to build log view — verify log content
  - Use tmux for automated TUI interaction in tests

  **Recommended Agent Profile**:
  - **Category**: `deep`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 44, 45)
  - **Blocks**: F1-F4
  - **Blocked By**: Tasks 36, 37, 21

  **Acceptance Criteria**:
  - [ ] TUI connects and creates project
  - [ ] Chat message sent and response received
  - [ ] View switching works in automated test

  **QA Scenarios**:
  ```
  Scenario: TUI end-to-end interaction
    Tool: interactive_bash (tmux)
    Steps:
      1. Start server in background
      2. Launch TUI in tmux session
      3. Type `:project create test-app Test project` + Enter
      4. Assert project created message appears
      5. Type "Hello" + Enter
      6. Wait 10s — assert response appears in chat
      7. Press Tab — assert diff view renders
      8. Press Ctrl-C — assert clean exit
    Expected Result: Full TUI interaction works
    Evidence: .sisyphus/evidence/task-46-e2e-tui.txt
  ```

  **Commit**: YES
  - Message: `test: add end-to-end TUI→Server→Agent integration test`
  - Pre-commit: `cargo test --workspace`

---

## Final Verification Wave

> 4 review agents run in PARALLEL. ALL must APPROVE. Present consolidated results to user and get explicit "okay" before completing.

- [ ] F1. **Plan Compliance Audit** — `oracle`
  Read the plan end-to-end. For each "Must Have": verify implementation exists (read file, run command). For each "Must NOT Have": search codebase for forbidden patterns — reject with file:line if found. Check evidence files exist in .sisyphus/evidence/. Compare deliverables against plan.
  Output: `Must Have [N/N] | Must NOT Have [N/N] | Tasks [N/N] | VERDICT: APPROVE/REJECT`

- [ ] F2. **Code Quality Review** — `unspecified-high`
  Run `cargo clippy --workspace -- -D warnings` + `cargo test --workspace` + `cargo doc --workspace --no-deps`. Review all crates for: `as any` patterns, empty catches, dead code, commented-out code, unused imports. Check for AI slop: excessive comments, over-abstraction, generic names.
  Output: `Build [PASS/FAIL] | Clippy [PASS/FAIL] | Tests [N pass/N fail] | Docs [PASS/FAIL] | VERDICT`

- [ ] F3. **Real Manual QA** — `unspecified-high`
  Start from clean state. Execute EVERY QA scenario from EVERY task. Test cross-feature integration: Discord→Agent→Docker→GitHub full flow. WebUI→Agent→Docker→GitHub flow. TUI→Server connection. Edge cases: empty project, invalid input, Docker timeout, GitHub auth failure.
  Output: `Scenarios [N/N pass] | Integration [N/N] | Edge Cases [N tested] | VERDICT`

- [ ] F4. **Scope Fidelity Check** — `deep`
  For each task: read "What to do", read actual diff. Verify 1:1 — everything in spec was built, nothing beyond spec was built. Check "Must NOT do" compliance per task. Check "Must NOT Have" guardrails. Flag unaccounted changes.
  Output: `Tasks [N/N compliant] | Guardrails [N/N respected] | Unaccounted [CLEAN/N files] | VERDICT`

---

## Commit Strategy

| Wave | Commit | Command |
|------|--------|---------|
| 1 | `chore(workspace): initialize cargo workspace with crate structure` | `cargo check --workspace` |
| 1 | `feat(core): add error types, TOML config, and tracing setup` | `cargo test -p cuttlefish-core` |
| 1 | `feat(core): define ModelProvider, Agent, Sandbox, VCS, MessageBus traits` | `cargo check --workspace` |
| 1 | `feat(db): add SQLite schema and migrations` | `cargo test -p cuttlefish-db` |
| 1 | `ci: add GitHub Actions workflow for test + clippy + fmt` | — |
| 1 | `docs: add README.md, CLAUDE.md, .gitignore` | — |
| 2-3 | One commit per task | `cargo test -p <crate>` |
| 4 | `feat(agents): add agent system with Orchestrator/Coder/Critic loop` | `cargo test -p cuttlefish-agents` |
| 5-10 | One commit per task | `cargo test --workspace` |

---

## Success Criteria

### Verification Commands
```bash
cargo test --workspace                    # Expected: all tests pass, 0 failures
cargo clippy --workspace -- -D warnings   # Expected: 0 warnings, 0 errors
cargo doc --workspace --no-deps           # Expected: generates cleanly
cargo build --release                     # Expected: successful binary
./target/release/cuttlefish-rs --help     # Expected: shows usage info
```

### Final Checklist
- [ ] All "Must Have" features present and working
- [ ] All "Must NOT Have" guardrails respected
- [ ] All 46 tasks pass their QA scenarios
- [ ] All 4 final verification agents approve
- [ ] End-to-end Discord flow works
- [ ] End-to-end WebUI flow works
- [ ] TUI connects and displays
- [ ] Self-update mechanism works
- [ ] Zero `unsafe` code in workspace
- [ ] Zero clippy warnings
