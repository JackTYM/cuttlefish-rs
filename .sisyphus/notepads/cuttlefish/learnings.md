# Cuttlefish — Learnings

## Project State (Wave 1 start)
- Rust edition 2024, bare Cargo.toml (6 lines), src/main.rs with Hello World
- No commits yet on master branch
- Target: KVM Linux deployment, zero unsafe code policy
# Workspace Scaffolding Learnings

## Task 1: Cargo Workspace Scaffolding + Workspace Lints

### Completed
- ✅ Root Cargo.toml converted to workspace with 9 member crates
- ✅ All 9 crates created in `crates/` directory
- ✅ 8 library crates with `src/lib.rs` (core, db, providers, sandbox, vcs, agents, discord, api)
- ✅ 1 binary crate `cuttlefish-tui` with `src/main.rs`
- ✅ Root `src/main.rs` preserved (server binary entry point)
- ✅ Workspace-level lints: `deny(unsafe_code)`, `warn(missing_docs)`, `deny(clippy::unwrap_used)`
- ✅ `rust-version = "1.94.0"` set in workspace
- ✅ Shared workspace dependencies: tokio, serde, tracing, thiserror, anyhow
- ✅ All crates reference shared deps via `workspace = true`
- ✅ `cargo check --workspace` passes with 0 exit code
- ✅ All lib.rs files have `#![deny(unsafe_code)]` and `#![warn(missing_docs)]` attributes
- ✅ Module-level docstrings added to satisfy missing_docs lint

### Key Patterns
- Module-level `//!` docstrings are required for all crates (lib.rs and main.rs)
- Workspace lints inherited by all members via `[lints] workspace = true`
- Shared dependencies declared once in workspace root, referenced with `workspace = true`
- Edition set to "2024" across all crates
- No unsafe code anywhere in scaffolding

### Verification
- `cargo check --workspace` exits 0
- All 9 crates compile successfully
- No warnings or errors

## Task 2: cuttlefish-core — Error Types + Config (TOML) + Tracing Setup

### Completed
- ✅ `src/error.rs` — `CuttlefishError` enum with 8 domain error types (Config, Provider, Sandbox, Vcs, Agent, Database, Discord, Api)
- ✅ `src/config.rs` — `CuttlefishConfig` struct with TOML deserialization, all subsections (Server, Database, Providers, Agents, Discord, Sandbox)
- ✅ `src/tracing.rs` — `init_tracing()` function with environment-based format selection (JSON for production, pretty for dev)
- ✅ `src/lib.rs` — module declarations and re-exports
- ✅ `Cargo.toml` — added `toml = "0.8"`, `tracing-subscriber` with `env-filter` and `json` features
- ✅ All 14 tests pass (error display, config loading, defaults, invalid TOML, tracing init)
- ✅ `cargo clippy -p cuttlefish-core -- -D warnings` → ZERO warnings

### Key Patterns
- **Error types**: Use `thiserror::Error` with `#[error(...)]` derive. Domain errors are simple `String`-backed structs for now.
- **Config defaults**: Use `#[serde(default = "fn_name")]` for individual fields. For complex structs like `SandboxConfig`, implement `Default` trait manually to ensure all fields get proper defaults.
- **Tracing**: `tracing_subscriber::registry()` with `.with()` chains for composable setup. Global subscriber can only be set once (tests must be careful).
- **Unsafe code**: `std::env::set_var()` and `std::env::remove_var()` are unsafe and forbidden by workspace lint. Tests must avoid environment manipulation.
- **Docstrings**: All public items require docstrings (`///` for items, `//!` for modules) to satisfy `#![warn(missing_docs)]` lint.

### Gotchas
- `SandboxConfig` with `#[derive(Default)]` doesn't apply `#[serde(default = "...")]` functions. Must implement `Default` trait manually.
- Tracing subscriber is global and can only be initialized once per process. Multiple test calls fail with "SetGlobalDefaultError".
- Environment variable manipulation in tests is unsafe and violates workspace lint. Use temp files for config testing instead.

### Verification
- `cargo test -p cuttlefish-core` → 14 passed
- `cargo clippy -p cuttlefish-core -- -D warnings` → 0 warnings
- All error types display correctly
- Config loads from TOML with all sections and defaults
- Tracing initializes without panic

## Task 3: Core Trait Definitions
- All 5 foundational traits defined: ModelProvider, Agent, Sandbox, VersionControl, MessageBus
- Dependencies added: async-trait, futures, uuid, serde_json
- `#![warn(missing_docs)]` requires doc comments on all public items — necessary for trait-only crate
- `BusMessage::new` uses `unwrap_or(0)` for timestamp (safe pattern, not `unwrap()`)
- `SandboxId::default()` delegates to `Self::new()` which uses `Uuid::new_v4()`
- `BoxStream` from `futures` used for streaming trait method return type
- `tokio::sync::broadcast::Receiver` used in MessageBus subscribe return — ties to tokio runtime
- All traits are `Send + Sync` via `#[async_trait]` default behavior

## Task 4: cuttlefish-db — SQLite Schema + Migrations

### Completed
- ✅ `Cargo.toml` — added sqlx (0.8) with sqlite, migrate, uuid, chrono features; uuid, chrono, tokio, tempfile dev-deps
- ✅ `migrations/001_initial_schema.sql` — all 6 tables (projects, conversations, agent_sessions, templates, build_logs, config_overrides) with indexes
- ✅ `src/models.rs` — 6 row structs (Project, Conversation, AgentSession, Template, BuildLog, ConfigOverride) with `sqlx::FromRow` derive
- ✅ `src/lib.rs` — `Database` struct with connection pool, `open()` method, CRUD methods for projects and conversations
- ✅ All 4 tests pass: create_and_get_project, update_project_status, insert_and_get_messages, token_count
- ✅ Test output saved to `.sisyphus/evidence/task-4-db-tests.txt`

### Key Patterns
- **SQLite setup**: Use `sqlite://path?mode=rwc` connection string for create-if-not-exists behavior
- **WAL mode**: Enable with `PRAGMA journal_mode=WAL` for better concurrent performance
- **Migrations**: Inline schema creation in `run_migrations()` method using individual `sqlx::query()` calls (not macro-based)
- **Row mapping**: `sqlx::query_as::<_, StructName>()` with `#[derive(sqlx::FromRow)]` on models
- **Trait imports**: Must import `sqlx::Row` trait to use `.get()` method on query results
- **Timestamps**: Store as TEXT using `datetime('now')` default in SQLite
- **Booleans**: SQLite stores as INTEGER (0/1); model fields use `i64` type
- **Foreign keys**: Use `ON DELETE CASCADE` for referential integrity
- **Indexes**: Create on frequently queried columns (project_id, created_at, status, archived)

### Gotchas
- `sqlx::migrate!()` macro requires compile-time migration discovery. For tests, inline schema creation is simpler.
- `sqlx::raw_sql()` doesn't exist; use individual `sqlx::query()` calls for each DDL statement.
- `Row::get()` requires `sqlx::Row` trait in scope; compiler error if trait not imported.
- `RETURNING *` clause works in SQLite 3.35+; fallback to INSERT then SELECT if needed.
- Missing documentation warnings on all public structs/fields — add `///` docstrings to satisfy lint.

### Verification
- `cargo test -p cuttlefish-db` → 4 passed, 0 failed
- All CRUD operations work: create project, get project, update status, insert messages, count tokens
- Database file created in temp directory during tests
- Schema created successfully with all tables and indexes

### Verification Fix: Missing Documentation
- ✅ Added `///` doc comments to ALL public structs in models.rs (Project, Conversation, AgentSession, Template, BuildLog, ConfigOverride)
- ✅ Added `///` doc comments to ALL public fields in each struct
- ✅ Added `///` doc comment to `pub mod models;` declaration in lib.rs
- ✅ All public methods in Database impl already had doc comments
- ✅ `cargo clippy -p cuttlefish-db -- -D warnings` → ZERO warnings/errors
- ✅ `cargo test -p cuttlefish-db` → 4 passed, 0 failed
- ✅ Clippy output saved to `.sisyphus/evidence/task-4-db-clippy-clean.txt`

### Documentation Pattern
- Module-level docstrings use `//!` format
- Public items use `///` format
- Each field documented with purpose and type information
- Workspace lint `#![warn(missing_docs)]` enforced across all crates

## Task 5: GitHub Actions Workflows

### CI Workflow (ci.yml)
- Triggers on push to main/master and pull requests
- Uses dtolnay/rust-toolchain@stable for Rust 1.94.0
- Includes rustfmt and clippy components
- Implements 3-level cargo caching: registry, git, target
- Cache key includes Cargo.lock hash for invalidation
- Runs: fmt check → clippy → tests → doc generation
- All checks must pass for PR merge

### Release Workflow (release.yml)
- Triggers only on version tags (v*)
- Builds optimized release binary
- Renames binary to: cuttlefish-{version}-linux-linux-x86_64
- Uses softprops/action-gh-release@v2 for GitHub Release creation
- Auto-generates release notes from commits
- Uses GITHUB_TOKEN (no hardcoded secrets)
- Requires contents:write permission

### Key Design Decisions
1. **Toolchain pinning**: Explicit 1.94.0 ensures reproducible builds
2. **Cargo caching**: Separate cache keys for CI vs Release builds
3. **Strict linting**: -D warnings enforces no clippy warnings
4. **Linux x86_64 only**: Matches KVM deployment target
5. **No multi-platform builds**: Simplifies CI/CD pipeline

### Validation
Both workflows validated as valid YAML. Ready for GitHub Actions execution.

## Task 6: Project Documentation

### Completed
- ✅ `README.md` — Project overview with 🐙 emoji, philosophy, architecture, crate structure table, getting started guide
- ✅ `CLAUDE.md` — Developer guide with non-negotiable rules, crate dependency hierarchy, development workflow, commit convention, testing guidelines, configuration
- ✅ `.gitignore` — Comprehensive coverage: Rust artifacts, database files, user config, env files, secrets, logs, IDE files, macOS, temp files
- ✅ `cuttlefish.example.toml` — Valid TOML with documented configuration sections: server, database, sandbox, discord, providers, agents

### Key Patterns
- **README**: Technical focus, no marketing copy. Includes crate structure table for quick reference.
- **CLAUDE.md**: Enforces workspace lints, trait-first architecture, TDD workflow, commit convention format.
- **.gitignore**: Covers all Rust-specific artifacts plus project-specific files (cuttlefish.toml, *.db, oauth_tokens.json).
- **Example TOML**: Comments explain purpose, security guidance, and required environment variables. No real secrets included.

### Verification
- ✅ TOML syntax valid (Python tomllib parser)
- ✅ All files created successfully
- ✅ No syntax errors in Markdown
- ✅ All files follow project conventions
- ✅ Evidence saved to `.sisyphus/evidence/task-6-docs.txt`

## Task 7: cuttlefish-providers — AWS Bedrock Provider + Streaming

### Completed
- ✅ `Cargo.toml` — Added aws-sdk-bedrockruntime, aws-config, aws-smithy-types, async-trait, futures, serde_json, cuttlefish-core dep
- ✅ `src/bedrock.rs` — `BedrockProvider` implementing `ModelProvider` trait with `converse` API, message conversion, tool call extraction
- ✅ `src/mock.rs` — `MockModelProvider` with canned response queue, default responses, streaming support
- ✅ `src/lib.rs` — Module exports for bedrock/mock, re-export of `ModelProvider` trait
- ✅ All 10 unit tests + 1 doctest pass
- ✅ `cargo clippy -p cuttlefish-providers -- -D warnings` → ZERO warnings

### Key Patterns
- **AWS Document → serde_json**: `aws_smithy_types::Document` does NOT implement `serde::Serialize`. Must manually convert via recursive `document_to_json()` function.
- **AWS Number**: `Number` is `Copy`, so use `*n` not `n.clone()`. Method is `to_f64_lossy()` not `to_f64()`.
- **Bedrock API types**: `ToolUseBlock` fields (`tool_use_id`, `name`) are `String` not `Option<String>`. `input` is `Document` not `Option<Document>`.
- **InferenceConfiguration**: `temperature()` takes `f32` not `f64`.
- **Mock pattern**: `Arc<Mutex<Vec<String>>>` for thread-safe canned response queue. `expect("mutex poisoned")` is acceptable Rust pattern.
- **Pseudo-streaming**: `stream()` method wraps `complete()` in `futures::stream::once(fut).flatten().boxed()` — real `converse_stream` deferred.
- **StreamExt import**: `futures::StreamExt` must be imported separately from `futures::stream` for `.boxed()` method.

### Verification
- `cargo test -p cuttlefish-providers` → 10 passed + 1 doctest
- `cargo clippy -p cuttlefish-providers -- -D warnings` → 0 warnings
- Evidence saved to `.sisyphus/evidence/task-7-providers-tests.txt`

## Task 9: cuttlefish-db — Conversation Storage + Sliding Window Queries

### Completed
- ✅ 4 new methods added to `Database` impl: `get_recent_messages_chrono`, `get_message_count`, `archive_and_summarize`, `get_nth_recent_message_timestamp`
- ✅ 3 new tests: chrono order, message count, archive+summarize
- ✅ All 7 tests pass (4 existing + 3 new)
- ✅ `cargo clippy -p cuttlefish-db -- -D warnings` → ZERO warnings

### Key Patterns
- **Chrono ordering**: Subquery pattern `SELECT * FROM (SELECT ... ORDER BY DESC LIMIT ?) ORDER BY ASC` to get N most recent in oldest-first order
- **Archive flow**: Two-step — UPDATE archived=1 on old messages, then INSERT summary as system message with cutoff timestamp
- **Nth message offset**: `LIMIT 1 OFFSET n` to get the cutoff timestamp for sliding window
- **SQLite datetime precision**: 1-second precision means rapid inserts get same timestamp. Tests verify count, not strict ordering.
- **Existing code style**: Uses runtime `query_as::<_, T>` with `.bind()`, not compile-time `query_as!` macros. Follow same pattern.

## Task 8: cuttlefish-providers — Claude Code OAuth PKCE Flow + CCH Signing

### Completed
- ✅ `src/oauth_flow.rs` — PKCE verifier/challenge generation, CCH body signing (xxHash64), auth URL builder, token types
- ✅ `src/claude_oauth.rs` — `ClaudeOAuthProvider` implementing `ModelProvider` with spoofed CLI headers and CCH signing
- ✅ `src/lib.rs` — Updated to export `claude_oauth` and `oauth_flow` modules
- ✅ `Cargo.toml` — Added reqwest, base64, sha2, xxhash-rust dependencies
- ✅ All 24 tests pass (10 existing + 14 new) + 1 doctest
- ✅ `cargo clippy -p cuttlefish-providers -- -D warnings` → ZERO warnings

### Key Patterns
- **CCH algorithm**: xxHash64 with seed `0x6e52736ac806831e`, lower 20 bits, 5-char lowercase hex
- **PKCE**: SHA256 of verifier, base64url (no padding) encoded as challenge
- **URL encoding**: Custom `urlencoded()` function — colon `:` becomes `%3A`, test must match encoded form
- **Billing header**: `cc_version=2.1.87.fingerprint; cc_entrypoint=cli; cch={hash};`
- **System message handling**: Filtered from messages array, extracted to top-level `system` field
- **No real HTTP in tests**: Only test request body construction and utility functions

### Verification
- `cargo test -p cuttlefish-providers` → 24 passed + 1 doctest
- `cargo clippy -p cuttlefish-providers -- -D warnings` → 0 warnings
- Evidence saved to `.sisyphus/evidence/task-8-claude-oauth-tests.txt`

## Task 10: cuttlefish-db — Project Metadata + Template Storage + Discord Lookup

### Completed
- ✅ 5 extended project methods: `get_project_by_discord_channel`, `get_projects_by_guild`, `set_project_discord_channel`, `set_project_container`, `set_project_github_url`
- ✅ 5 template CRUD methods: `create_template`, `get_template`, `list_templates`, `list_templates(language)`, `delete_template`
- ✅ 2 new tests: `test_discord_channel_lookup`, `test_template_crud`
- ✅ All 9 tests pass (7 existing + 2 new)
- ✅ `cargo clippy -p cuttlefish-db -- -D warnings` → ZERO warnings

### Key Patterns
- **Discord lookup**: Uses indexed column `idx_projects_discord_channel` for fast channel→project resolution
- **Guild queries**: Returns Vec sorted by `created_at DESC` for chronological listing
- **Template CRUD**: Standard SQL patterns with parameterized queries (no injection risk)
- **Optional filtering**: `list_templates(Option<&str>)` branches on language filter presence
- **Delete return**: Returns `bool` indicating whether row was actually deleted (rows_affected > 0)
- **All methods**: Return `Result<T, sqlx::Error>` for proper error propagation

### Verification
- `cargo test -p cuttlefish-db` → 9 passed (7 existing + 2 new)
- `cargo clippy -p cuttlefish-db -- -D warnings` → 0 warnings
- Evidence saved to `.sisyphus/evidence/task-10-db-templates.txt`

## Task 11: cuttlefish-core — Context Manager (Sliding Window + Summaries)

### Completed
- ✅ `src/context.rs` — `ContextManager` struct with `build_context` and `trigger_summarization` methods
- ✅ `ContextConfig` with `max_tokens`, `summarize_threshold`, `summarization_enabled` fields
- ✅ `src/lib.rs` — Updated to export `context` module, re-exports `ContextManager` and `ContextConfig`
- ✅ `Cargo.toml` — Added `cuttlefish-db` dependency, `sqlx` dev-dependency for tests
- ✅ 4 new tests: empty context, within budget, respects budget, summarization at threshold
- ✅ All 18 tests pass (14 existing + 4 new)
- ✅ `cargo clippy -p cuttlefish-core -- -D warnings` → ZERO warnings

### Key Patterns
- **Dependency direction**: `cuttlefish-core → cuttlefish-db` is safe since `cuttlefish-db` does NOT depend on `cuttlefish-core`
- **Inline mock**: Test uses `TestProvider` implementing `ModelProvider` directly (cannot use cuttlefish-providers — would be circular)
- **Token estimation**: `content.len() / 4 + 1` — rough 4 chars/token approximation
- **Summarization flow**: Check count → get cutoff timestamp → archive old messages → insert summary
- **SQLite timestamp precision**: 1-second resolution means rapid inserts share timestamps; tests use explicit timestamps via raw SQL
- **Error mapping**: `sqlx::Error` → `DatabaseError(e.to_string())` → `CuttlefishError::Database` via thiserror `#[from]`
- **Provider errors**: `?` operator directly converts `ProviderError` to `CuttlefishError` via `From` impl

### Gotchas
- SQLite `datetime('now')` has 1-second precision. Rapid message inserts in tests share the same timestamp, causing `archive_and_summarize`'s `created_at < cutoff` to archive 0 rows. Fix: use explicit timestamps via `sqlx::query` with `db.pool()`.
- `sqlx` must be an explicit dev-dependency even though it's a transitive dep via `cuttlefish-db` (Rust 2024 edition doesn't expose transitive deps directly).
- `NamedTempFile` must be kept alive for test duration (return as `_tmp` binding) to prevent SQLite file deletion.

### Verification
- `cargo test -p cuttlefish-core` → 18 passed (14 existing + 4 new)
- `cargo clippy -p cuttlefish-core -- -D warnings` → 0 warnings
- Evidence saved to `.sisyphus/evidence/task-11-context-manager.txt`

## Task 12: cuttlefish-sandbox — Docker Container Lifecycle Management

### Completed
- ✅ `Cargo.toml` — Added bollard 0.18, cuttlefish-core, uuid, futures, async-trait, serde_json; declared `integration` feature
- ✅ `src/docker.rs` — `DockerSandbox` implementing full `Sandbox` trait: create, exec, write_file, read_file, list_files, destroy
- ✅ `src/lib.rs` — Exports `DockerSandbox`, re-exports `Sandbox` trait and supporting types
- ✅ 10 unit tests pass (base64 roundtrip, edge cases, ExecOutput, SandboxConfig defaults)
- ✅ `cargo clippy -p cuttlefish-sandbox -- -D warnings` → ZERO warnings
- ✅ Integration tests behind `#[cfg(feature = "integration")]` for Docker-dependent tests

### Key Patterns
- **bollard API**: Two-step container lifecycle: `create_container` → `start_container`. Exec is `create_exec` → `start_exec` → `inspect_exec` for exit code.
- **Container keep-alive**: `cmd: Some(vec!["tail", "-f", "/dev/null"])` keeps container running for exec commands.
- **Resource limits**: `memory` in bytes (MB × 1024 × 1024), `nano_cpus` (cores × 1e9 as i64).
- **Network isolation**: `network_mode: "none"` disables networking, `"bridge"` enables.
- **File I/O via base64**: Inline base64 encode/decode avoids shell escaping issues. No external `base64` crate needed.
- **StartExecResults**: Must match `StartExecResults::Attached` variant to get stdout/stderr streams.
- **Feature gating**: `#[cfg(feature = "integration")]` requires `[features] integration = []` in Cargo.toml to avoid `unexpected_cfgs` warning.
- **Dead code in cfg(test)**: Test-only helper functions still need `#[cfg(test)]` AND must be actually used in tests, otherwise `dead_code` warning triggers.

### Verification
- `cargo check -p cuttlefish-sandbox` → exits 0
- `cargo clippy -p cuttlefish-sandbox -- -D warnings` → 0 warnings
- `cargo test -p cuttlefish-sandbox` → 10 passed, 0 failed
- Evidence saved to `.sisyphus/evidence/task-12-sandbox-clippy.txt`

## Task 14: cuttlefish-vcs — Git Operations via git2

### Completed
- ✅ `Cargo.toml` — Added git2 0.19 (no default features), cuttlefish-core, async-trait; tempfile dev-dep
- ✅ `src/git.rs` — `GitRepository` implementing full `VersionControl` trait: clone, checkout, commit, push, diff, log, current_branch
- ✅ `src/lib.rs` — Exports `GitRepository`, re-exports `VersionControl` and `CommitInfo`
- ✅ All 6 unit tests pass (no network required — uses tempfile local repos)
- ✅ `cargo clippy -p cuttlefish-vcs -- -D warnings` → ZERO warnings

### Key Patterns
- **git2 is sync**: All git2 operations wrapped in `tokio::task::spawn_blocking` for async compatibility
- **spawn_blocking JoinError**: `.map_err(|e| VcsError(format!("Task join: {e}")))?` pattern for handling JoinError
- **PAT auth**: `git2::Cred::userpass_plaintext("oauth2", &token)` via `RemoteCallbacks::credentials`
- **Initial commit**: Empty `parents` slice when `repo.head()` returns Err (no HEAD yet)
- **Branch creation**: `repo.branch()` → `repo.set_head()` → `repo.checkout_head()` three-step flow
- **Diff format**: `diff.print(DiffFormat::Patch, ...)` with line origin prefix (+/-/space)
- **Short hash**: `oid.to_string()[..7]` for CommitInfo display
- **No unwrap in lib code**: All `expect()` calls have descriptive messages; `unwrap_or_else` for fallback signatures
- **Branch ref name**: Use `.ok_or_else()` instead of `.expect()` for branch ref name UTF-8 validation

### Verification
- `cargo test -p cuttlefish-vcs` → 6 passed, 0 failed
- `cargo clippy -p cuttlefish-vcs -- -D warnings` → 0 warnings
- Evidence saved to `.sisyphus/evidence/task-14-vcs-tests.txt`

## Task 13: cuttlefish-sandbox — Resource Limits + Streaming Output Capture + Cleanup

### Completed
- ✅ `exec_with_timeout` — wraps `exec()` in `tokio::time::timeout`, returns `timed_out: true` on expiry
- ✅ `list_cuttlefish_containers` — lists all containers with `cuttlefish-` name prefix via `ListContainersOptions` filter
- ✅ `cleanup_stopped_containers` — removes exited/dead cuttlefish containers, returns removal count
- ✅ Output size capping in `exec()` — truncates stdout/stderr at 1MB with `[OUTPUT TRUNCATED]` marker
- ✅ `MAX_OUTPUT_BYTES` constant (1MB) for stream cap
- ✅ 5 new tests (15 total): timeout output structure, truncation constants, truncation logic, under-limit check
- ✅ `cargo clippy -p cuttlefish-sandbox -- -D warnings` → ZERO warnings
- ✅ `cargo test -p cuttlefish-sandbox` → 15 passed

### Key Patterns
- **Timeout wrapping**: `tokio::time::timeout(Duration, future)` returns `Err(Elapsed)` on timeout — map to `ExecOutput { timed_out: true }`
- **Container filtering**: `ListContainersOptions` with `filters` HashMap: `name → ["cuttlefish-"]`, `status → ["exited", "dead"]`
- **Output capping**: Check buffer length after each chunk push; truncate + append marker + break on exceed
- **No Docker in unit tests**: All tests validate data structures and logic only; Docker-dependent tests behind `integration` feature

### Verification
- `cargo test -p cuttlefish-sandbox` → 15 passed (10 existing + 5 new)
- `cargo clippy -p cuttlefish-sandbox -- -D warnings` → 0 warnings
- Evidence saved to `.sisyphus/evidence/task-13-sandbox-limits.txt`

## Task 15: cuttlefish-vcs — GitHub API Client with PAT Authentication

### Completed
- ✅ `Cargo.toml` — Added reqwest 0.12 (json, rustls-tls), serde (workspace), serde_json 1
- ✅ `src/github.rs` — `GitHubClient` with 6 async methods: create_repo, get_repo, create_pull_request, list_workflow_runs, get_workflow_run, get_workflow_run_logs_url
- ✅ `src/lib.rs` — Exports `github` module and `GitHubClient`
- ✅ 5 new tests (11 total): client creation, auth header, workflow states, serialization
- ✅ `cargo clippy -p cuttlefish-vcs -- -D warnings` → ZERO warnings

### Key Patterns
- **Redirect handling**: `get_workflow_run_logs_url` builds a separate `reqwest::Client` with `Policy::none()` — `RequestBuilder` has no `.redirect()` method, only `ClientBuilder` does
- **Status checks**: Use `status.is_redirection()` and `status.is_success()` instead of bare integer comparison (StatusCode doesn't impl `PartialEq<u16>`)
- **Error body capture**: `response.text().await.unwrap_or_default()` is allowed — `unwrap_or_default()` is not flagged by `clippy::unwrap_used`
- **expect() in constructor**: Acceptable for `Client::builder().build()` — TLS backend missing is a programming/environment error
- **No real HTTP in tests**: All 5 tests validate struct construction, serialization, and data access patterns only

### Verification
- `cargo test -p cuttlefish-vcs` → 11 passed (6 existing + 5 new)
- `cargo clippy -p cuttlefish-vcs -- -D warnings` → 0 warnings
- Evidence saved to `.sisyphus/evidence/task-15-github-client.txt`


## Task 16: Template-specific Docker Images + ImageRegistry

**Completed**: 2026-04-01

### Deliverables
✅ Created 5 Dockerfiles in `docker/` directory:
- `node-base.Dockerfile` — Node.js 22 + npm + git + curl
- `python-base.Dockerfile` — Python 3.12 + pip + build-essential + git + curl
- `rust-base.Dockerfile` — Rust 1.82 + cargo + pkg-config + libssl-dev + git + curl
- `go-base.Dockerfile` — Go 1.22 + git + curl
- `generic-base.Dockerfile` — Ubuntu 22.04 + build-essential + git + curl + wget + unzip + ca-certificates

✅ Created `crates/cuttlefish-sandbox/src/images.rs`:
- `ImageRegistry` struct with HashMap-based template → image mapping
- `default_registry()` — pre-configured with 15+ template names
- `resolve(template_name)` — resolves template to Docker image (fallback: ubuntu:22.04)
- `register()` — custom template registration
- `has_template()` — check if template exists
- `list_templates()` — sorted list of all templates
- `dockerfile_path()` — maps template to Dockerfile path
- 8 comprehensive unit tests

✅ Updated `crates/cuttlefish-sandbox/src/lib.rs`:
- Exported `pub mod images`
- Exported `pub use images::ImageRegistry`

### Test Results
- **23 tests passed** (15 existing + 8 new)
- **0 warnings** from clippy with `-D warnings`
- `cargo check` ✅
- `cargo clippy` ✅
- `cargo test` ✅
- `cargo doc` ✅

### Key Design Decisions
1. **Public images by default** — Uses official Docker Hub images (node:22-slim, python:3.12-slim, etc.) rather than building custom images
2. **Fallback to generic** — Unknown templates resolve to ubuntu:22.04
3. **Dockerfile mapping** — `dockerfile_path()` enables future build-from-Dockerfile support
4. **HashMap-based registry** — Simple, fast O(1) lookups; supports runtime registration

### Template Coverage
- **Node.js family**: node, nuxt, nuxt-cloudflare, typescript, node-express
- **Python family**: python, python-fastapi, fastapi
- **Rust family**: rust, rust-axum, axum
- **Go family**: go, golang
- **Generic**: generic, static-site

