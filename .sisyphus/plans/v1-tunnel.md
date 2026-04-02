# V1 Tunnel System — Self-Host ↔ cuttlefish.ai Connectivity

## TL;DR

> **Quick Summary**: Build a WebSocket-based tunnel system that allows self-hosted Cuttlefish instances to be accessible via `{username}.cuttlefish.ai` subdomains, enabling remote access to local dashboards.
> 
> **Deliverables**:
> - Tunnel daemon (server-side, runs on cuttlefish.ai infrastructure)
> - Tunnel client (built into Cuttlefish binary)
> - Link code generation and authentication
> - Reverse proxy integration (Caddy config)
> - CLI commands: `cuttlefish tunnel connect`, `cuttlefish tunnel status`
> 
> **Estimated Effort**: Large (4-5 days)
> **Parallel Execution**: YES - 4 waves
> **Critical Path**: Task 1 (Protocol) → Tasks 2-3 (Client+Server) → Task 4 (Auth) → Tasks 5-6 (Integration)

---

## Context

### Original Request
Enable self-hosted Cuttlefish instances to be accessible through `{username}.cuttlefish.ai` by establishing a secure tunnel from the user's server to the cuttlefish.ai infrastructure.

### Problem Statement
Users self-hosting Cuttlefish on their own infrastructure (home server, VPS, etc.) often:
- Don't have a static IP or domain
- Are behind NAT/firewalls
- Want easy remote access without VPN setup

**Solution**: A tunnel system where the self-hosted Cuttlefish connects *outbound* to cuttlefish.ai, and the proxy routes incoming requests back through that connection.

### How It Works

```
User's Browser                    cuttlefish.ai                   Self-Hosted Server
     │                                 │                                │
     │ GET jacktym.cuttlefish.ai       │                                │
     │────────────────────────────────▶│                                │
     │                                 │                                │
     │                                 │  [Lookup tunnel for jacktym]   │
     │                                 │                                │
     │                                 │  Forward via WebSocket         │
     │                                 │───────────────────────────────▶│
     │                                 │                                │
     │                                 │◀───────────────────────────────│
     │                                 │  Response                      │
     │                                 │                                │
     │◀────────────────────────────────│                                │
     │ Response                        │                                │
```

### Research Findings
Similar systems: Cloudflare Tunnel (cloudflared), ngrok, localtunnel, bore.
- All use outbound connections from client to avoid firewall issues
- WebSocket or HTTP/2 for multiplexing
- Authentication via tokens or link codes

---

## Work Objectives

### Core Objective
Enable self-hosted Cuttlefish instances to be accessible via personalized subdomains at cuttlefish.ai without requiring users to configure DNS, firewalls, or VPNs.

### Concrete Deliverables
- `crates/cuttlefish-tunnel/` — New crate for tunnel logic
- `crates/cuttlefish-tunnel/src/protocol.rs` — Message protocol definitions
- `crates/cuttlefish-tunnel/src/client.rs` — Tunnel client (embedded in main binary)
- `crates/cuttlefish-tunnel/src/server.rs` — Tunnel daemon (separate binary)
- `crates/cuttlefish-tunnel/src/auth.rs` — Link code and JWT auth
- CLI integration in main binary
- Caddy configuration for wildcard subdomain routing
- Database tables for link codes and active tunnels

### Definition of Done
- [ ] `cuttlefish tunnel connect <link-code>` establishes tunnel
- [ ] Requests to `{user}.cuttlefish.ai` reach self-hosted instance
- [ ] Tunnel survives network interruptions (auto-reconnect)
- [ ] `cargo test -p cuttlefish-tunnel` passes
- [ ] `cargo clippy --workspace -- -D warnings` clean

### Must Have
- Outbound WebSocket connection (works behind NAT)
- Link code authentication (one-time codes for initial pairing)
- JWT for ongoing authentication after pairing
- Auto-reconnect with exponential backoff
- Heartbeat/keepalive mechanism
- Request/response multiplexing over single connection

### Must NOT Have (Guardrails)
- No unsafe code
- No `unwrap()` — use `?` or `expect("reason")`
- No hardcoded secrets (all via env vars or config)
- No UDP-based protocols (WebSocket only for simplicity)
- No custom TLS implementation (use rustls via tokio-tungstenite)
- No changes to core Cuttlefish functionality (tunnel is additive)

---

## Verification Strategy

> **ZERO HUMAN INTERVENTION** — ALL verification is agent-executed.

### Test Decision
- **Infrastructure exists**: YES (workspace test infra)
- **Automated tests**: YES (Tests-after)
- **Framework**: `#[tokio::test]` for async tests

### QA Policy
Every task includes agent-executed QA scenarios.
Evidence saved to `.sisyphus/evidence/task-{N}-{scenario-slug}.{ext}`.

- **Protocol tests**: Unit tests for message serialization
- **Integration tests**: Local server + client connection
- **E2E tests**: Full tunnel with mock HTTP requests

---

## Execution Strategy

### Parallel Execution Waves

```
Wave 1 (Foundation — protocol and types):
├── Task 1: Define tunnel protocol messages [quick]
└── Task 2: Create cuttlefish-tunnel crate structure [quick]

Wave 2 (Core — client and server):
├── Task 3: Implement tunnel client [deep]
├── Task 4: Implement tunnel server/daemon [deep]
└── Task 5: Link code auth system [unspecified-high]

Wave 3 (Integration — CLI and proxy):
├── Task 6: CLI commands integration [quick]
├── Task 7: Caddy reverse proxy config [quick]
└── Task 8: Database tables for tunnels [quick]

Wave 4 (Polish — reconnect and monitoring):
├── Task 9: Auto-reconnect with backoff [unspecified-high]
└── Task 10: Tunnel status monitoring [quick]

Wave FINAL (Verification):
├── Task F1: Plan compliance audit (oracle)
├── Task F2: Code quality review (unspecified-high)
├── Task F3: E2E tunnel QA (deep)
└── Task F4: Scope fidelity check (deep)
-> Present results -> Get explicit user okay

Critical Path: Task 1 → Task 2 → Tasks 3,4 → Task 6 → F1-F4 → user okay
Parallel Speedup: ~55% faster than sequential
Max Concurrent: 3 (Waves 2 & 3)
```

### Dependency Matrix

| Task | Depends On | Blocks | Wave |
|------|------------|--------|------|
| 1 | — | 2, 3, 4 | 1 |
| 2 | — | 3, 4, 5, 6 | 1 |
| 3 | 1, 2 | 6, 9 | 2 |
| 4 | 1, 2 | 6, 7, 9 | 2 |
| 5 | 2 | 3, 4, 8 | 2 |
| 6 | 2, 3 | 10 | 3 |
| 7 | 4 | — | 3 |
| 8 | 5 | — | 3 |
| 9 | 3, 4 | 10 | 4 |
| 10 | 6, 9 | F1-F4 | 4 |

### Agent Dispatch Summary

- **Wave 1**: 2 tasks — T1 → `quick`, T2 → `quick`
- **Wave 2**: 3 tasks — T3 → `deep`, T4 → `deep`, T5 → `unspecified-high`
- **Wave 3**: 3 tasks — T6-T8 → `quick`
- **Wave 4**: 2 tasks — T9 → `unspecified-high`, T10 → `quick`
- **FINAL**: 4 tasks — F1 → `oracle`, F2 → `unspecified-high`, F3 → `deep`, F4 → `deep`

---

## TODOs

- [ ] 1. Define Tunnel Protocol Messages

  **What to do**:
  - Create `crates/cuttlefish-tunnel/src/protocol.rs`
  - Define message types using serde for JSON serialization:
    ```rust
    enum ClientMessage {
        Auth { link_code: String },
        AuthToken { jwt: String },
        HttpResponse { request_id: u64, status: u16, headers: Vec<(String, String)>, body: Vec<u8> },
        Heartbeat,
    }
    
    enum ServerMessage {
        AuthSuccess { jwt: String, subdomain: String },
        AuthFailure { reason: String },
        HttpRequest { request_id: u64, method: String, path: String, headers: Vec<(String, String)>, body: Vec<u8> },
        HeartbeatAck,
        Disconnect { reason: String },
    }
    ```
  - Implement `TryFrom<&[u8]>` and `Into<Vec<u8>>` for wire format
  - Add request_id generation (atomic u64 counter)
  - Write unit tests for serialization round-trips

  **Must NOT do**:
  - Don't use binary protocols (JSON for debuggability)
  - Don't include large payloads inline (stream separately for files)

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: Well-defined types, straightforward serde
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Task 2)
  - **Parallel Group**: Wave 1
  - **Blocks**: Tasks 3, 4
  - **Blocked By**: None

  **References**:
  - `crates/cuttlefish-api/src/websocket.rs` — Existing WebSocket message patterns
  - cloudflared protocol: https://github.com/cloudflare/cloudflared

  **Acceptance Criteria**:
  - [ ] All message types serialize/deserialize correctly
  - [ ] Unit tests pass for round-trip serialization
  - [ ] `cargo clippy -p cuttlefish-tunnel -- -D warnings` clean

  **QA Scenarios**:
  ```
  Scenario: Message serialization round-trip
    Tool: Bash (cargo test)
    Preconditions: Protocol module exists
    Steps:
      1. Run `cargo test -p cuttlefish-tunnel protocol::tests`
      2. Verify ClientMessage::Auth round-trips correctly
      3. Verify ServerMessage::HttpRequest round-trips correctly
    Expected Result: All serialization tests pass
    Evidence: .sisyphus/evidence/task-1-protocol-tests.txt

  Scenario: Invalid message handling
    Tool: Bash (cargo test)
    Preconditions: Error handling tests exist
    Steps:
      1. Run `cargo test -p cuttlefish-tunnel protocol::tests::test_invalid_json`
      2. Verify returns Err, not panic
    Expected Result: Graceful error handling
    Evidence: .sisyphus/evidence/task-1-invalid-msg.txt
  ```

  **Commit**: YES (groups with Task 2)
  - Message: `feat(tunnel): add tunnel protocol definitions`
  - Files: `crates/cuttlefish-tunnel/src/protocol.rs`

- [ ] 2. Create cuttlefish-tunnel Crate Structure

  **What to do**:
  - Create `crates/cuttlefish-tunnel/` directory
  - Create `Cargo.toml` with dependencies:
    - `tokio` (runtime)
    - `tokio-tungstenite` (WebSocket)
    - `serde`, `serde_json` (serialization)
    - `tracing` (logging)
    - `thiserror` (errors)
    - `jsonwebtoken` (JWT)
  - Create `src/lib.rs` with module declarations
  - Create `src/error.rs` with `TunnelError` enum
  - Add crate to workspace `Cargo.toml`
  - Ensure `#![deny(unsafe_code)]` and `#![deny(clippy::unwrap_used)]`

  **Must NOT do**:
  - Don't add unnecessary dependencies
  - Don't create binaries yet (library first)

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: Scaffolding task, follows existing patterns
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Task 1)
  - **Parallel Group**: Wave 1
  - **Blocks**: Tasks 3, 4, 5, 6
  - **Blocked By**: None

  **References**:
  - `crates/cuttlefish-api/Cargo.toml` — Dependency patterns
  - `crates/cuttlefish-core/src/error.rs` — Error enum pattern

  **Acceptance Criteria**:
  - [ ] `cargo build -p cuttlefish-tunnel` succeeds
  - [ ] `cargo clippy -p cuttlefish-tunnel -- -D warnings` clean
  - [ ] Crate appears in workspace

  **QA Scenarios**:
  ```
  Scenario: Crate builds successfully
    Tool: Bash
    Preconditions: Crate created with Cargo.toml
    Steps:
      1. Run `cargo build -p cuttlefish-tunnel`
      2. Verify no compilation errors
    Expected Result: Build succeeds
    Evidence: .sisyphus/evidence/task-2-build.txt

  Scenario: Workspace integration
    Tool: Bash
    Preconditions: Crate added to workspace
    Steps:
      1. Run `cargo metadata --format-version=1 | jq '.packages[] | select(.name=="cuttlefish-tunnel")'`
      2. Verify crate is listed
    Expected Result: Crate in workspace metadata
    Evidence: .sisyphus/evidence/task-2-workspace.txt
  ```

  **Commit**: YES (groups with Task 1)
  - Message: `feat(tunnel): add tunnel protocol definitions`
  - Files: `crates/cuttlefish-tunnel/*`, `Cargo.toml`

- [ ] 3. Implement Tunnel Client

  **What to do**:
  - Create `crates/cuttlefish-tunnel/src/client.rs`
  - Implement `TunnelClient` struct:
    ```rust
    pub struct TunnelClient {
        server_url: String,
        jwt: Option<String>,
        local_addr: SocketAddr,
    }
    
    impl TunnelClient {
        pub async fn connect_with_link_code(&mut self, code: &str) -> Result<(), TunnelError>
        pub async fn connect_with_jwt(&mut self, jwt: &str) -> Result<(), TunnelError>
        pub async fn run(&mut self) -> Result<(), TunnelError>  // Main loop
    }
    ```
  - Handle incoming `HttpRequest` messages:
    - Forward to local HTTP server
    - Capture response
    - Send back as `HttpResponse`
  - Implement heartbeat sending (every 30 seconds)
  - Store JWT after successful auth for reconnection
  - Use `tokio::select!` for concurrent message handling

  **Must NOT do**:
  - Don't implement reconnection logic here (Task 9)
  - Don't block on single requests (handle concurrently)

  **Recommended Agent Profile**:
  - **Category**: `deep`
    - Reason: Complex async networking code
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 4, 5)
  - **Parallel Group**: Wave 2
  - **Blocks**: Tasks 6, 9
  - **Blocked By**: Tasks 1, 2

  **References**:
  - `crates/cuttlefish-api/src/websocket.rs` — WebSocket patterns
  - tokio-tungstenite examples: https://github.com/snapview/tokio-tungstenite/tree/master/examples

  **Acceptance Criteria**:
  - [ ] Client connects to test server
  - [ ] Auth flow completes with link code
  - [ ] HTTP requests are forwarded and responses returned
  - [ ] Heartbeats sent every 30 seconds

  **QA Scenarios**:
  ```
  Scenario: Client connects with link code
    Tool: Bash (cargo test)
    Preconditions: Mock server running
    Steps:
      1. Run `cargo test -p cuttlefish-tunnel client::tests::test_connect_with_link_code`
      2. Verify client receives JWT after auth
    Expected Result: Auth completes, JWT stored
    Evidence: .sisyphus/evidence/task-3-auth.txt

  Scenario: HTTP request forwarding
    Tool: Bash (cargo test)
    Preconditions: Client connected, local HTTP server on :8080
    Steps:
      1. Run integration test that sends HttpRequest through tunnel
      2. Verify response matches local server response
    Expected Result: Request forwarded, response received
    Evidence: .sisyphus/evidence/task-3-forward.txt
  ```

  **Commit**: YES
  - Message: `feat(tunnel): implement tunnel client`
  - Files: `crates/cuttlefish-tunnel/src/client.rs`

- [ ] 4. Implement Tunnel Server/Daemon

  **What to do**:
  - Create `crates/cuttlefish-tunnel/src/server.rs`
  - Implement `TunnelServer` struct:
    ```rust
    pub struct TunnelServer {
        connections: Arc<RwLock<HashMap<String, TunnelConnection>>>,  // subdomain -> connection
        db: Arc<Database>,  // For link code validation
    }
    
    impl TunnelServer {
        pub async fn handle_connection(&self, ws: WebSocket) -> Result<(), TunnelError>
        pub async fn route_request(&self, subdomain: &str, request: HttpRequest) -> Result<HttpResponse, TunnelError>
    }
    ```
  - Handle client auth (validate link code or JWT)
  - Register connection by subdomain
  - Route incoming HTTP requests to correct tunnel
  - Handle disconnections (remove from map)
  - Create separate binary: `src/bin/tunnel-daemon.rs`

  **Must NOT do**:
  - Don't implement the reverse proxy (Caddy does that)
  - Don't handle TLS termination (Caddy does that)

  **Recommended Agent Profile**:
  - **Category**: `deep`
    - Reason: Complex async server with connection management
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 3, 5)
  - **Parallel Group**: Wave 2
  - **Blocks**: Tasks 6, 7, 9
  - **Blocked By**: Tasks 1, 2

  **References**:
  - `crates/cuttlefish-api/src/routes.rs` — Axum route patterns
  - `crates/cuttlefish-api/src/websocket.rs` — WebSocket handling

  **Acceptance Criteria**:
  - [ ] Server accepts WebSocket connections
  - [ ] Auth with link code registers subdomain
  - [ ] Requests routed to correct client
  - [ ] Disconnection cleans up connection map

  **QA Scenarios**:
  ```
  Scenario: Server accepts client connection
    Tool: Bash (cargo test)
    Preconditions: Server running on test port
    Steps:
      1. Run `cargo test -p cuttlefish-tunnel server::tests::test_accept_connection`
      2. Verify connection established and authenticated
    Expected Result: Connection in server's map
    Evidence: .sisyphus/evidence/task-4-accept.txt

  Scenario: Request routing
    Tool: Bash (cargo test)
    Preconditions: Two clients connected with different subdomains
    Steps:
      1. Route request to subdomain A
      2. Verify only client A receives it
      3. Route request to subdomain B
      4. Verify only client B receives it
    Expected Result: Correct routing
    Evidence: .sisyphus/evidence/task-4-routing.txt
  ```

  **Commit**: YES
  - Message: `feat(tunnel): implement tunnel server daemon`
  - Files: `crates/cuttlefish-tunnel/src/server.rs`, `src/bin/tunnel-daemon.rs`

- [ ] 5. Link Code Authentication System

  **What to do**:
  - Create `crates/cuttlefish-tunnel/src/auth.rs`
  - Implement link code generation:
    ```rust
    pub fn generate_link_code() -> String  // 6 alphanumeric chars, uppercase
    pub fn hash_link_code(code: &str) -> String  // For storage
    ```
  - Implement JWT generation/validation:
    ```rust
    pub struct TunnelClaims {
        pub sub: String,  // user_id
        pub subdomain: String,
        pub exp: i64,
    }
    
    pub fn generate_jwt(claims: &TunnelClaims, secret: &[u8]) -> Result<String, TunnelError>
    pub fn validate_jwt(token: &str, secret: &[u8]) -> Result<TunnelClaims, TunnelError>
    ```
  - Link codes expire in 10 minutes
  - JWTs expire in 7 days (long-lived for reconnection)
  - Add `TUNNEL_JWT_SECRET` env var requirement

  **Must NOT do**:
  - Don't store link codes in plaintext (hash them)
  - Don't use short JWT expiry (would break reconnection)

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
    - Reason: Security-sensitive auth code
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 3, 4)
  - **Parallel Group**: Wave 2
  - **Blocks**: Tasks 3, 4, 8
  - **Blocked By**: Task 2

  **References**:
  - `jsonwebtoken` crate docs: https://docs.rs/jsonwebtoken
  - `crates/cuttlefish-core/src/advanced.rs:267-289` — JWT claims pattern

  **Acceptance Criteria**:
  - [ ] Link codes are 6 chars, alphanumeric
  - [ ] Link code validation works within 10 min
  - [ ] JWT generation and validation work
  - [ ] JWT contains correct claims

  **QA Scenarios**:
  ```
  Scenario: Link code generation and validation
    Tool: Bash (cargo test)
    Preconditions: Auth module exists
    Steps:
      1. Generate link code
      2. Hash it
      3. Validate original code against hash
    Expected Result: Validation passes
    Evidence: .sisyphus/evidence/task-5-linkcode.txt

  Scenario: JWT round-trip
    Tool: Bash (cargo test)
    Preconditions: JWT functions implemented
    Steps:
      1. Generate JWT with test claims
      2. Validate JWT
      3. Verify claims match
    Expected Result: Claims preserved through round-trip
    Evidence: .sisyphus/evidence/task-5-jwt.txt
  ```

  **Commit**: YES
  - Message: `feat(tunnel): add link code and JWT authentication`
  - Files: `crates/cuttlefish-tunnel/src/auth.rs`

- [ ] 6. CLI Commands Integration

  **What to do**:
  - Add tunnel subcommands to main binary in `src/main.rs`:
    - `cuttlefish tunnel connect <link-code>` — Connect with link code
    - `cuttlefish tunnel connect --jwt <path>` — Connect with saved JWT
    - `cuttlefish tunnel disconnect` — Gracefully disconnect
    - `cuttlefish tunnel status` — Show connection status
  - Store JWT in `~/.config/cuttlefish/tunnel.jwt` after successful auth
  - Load saved JWT on startup for auto-reconnect
  - Display connection status: subdomain, connected since, bytes transferred
  - Handle Ctrl+C gracefully (send Disconnect message)

  **Must NOT do**:
  - Don't store link codes (one-time use)
  - Don't run tunnel in background by default (foreground with logs)

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: CLI wiring, straightforward
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 7, 8)
  - **Parallel Group**: Wave 3
  - **Blocks**: Task 10
  - **Blocked By**: Tasks 2, 3

  **References**:
  - `src/main.rs` — Existing CLI structure
  - clap subcommands

  **Acceptance Criteria**:
  - [ ] `cuttlefish tunnel connect ABC123` initiates connection
  - [ ] JWT saved after successful auth
  - [ ] `cuttlefish tunnel status` shows connection info
  - [ ] Ctrl+C disconnects cleanly

  **QA Scenarios**:
  ```
  Scenario: Connect with link code via CLI
    Tool: Bash
    Preconditions: Tunnel daemon running, valid link code generated
    Steps:
      1. Run `./target/release/cuttlefish tunnel connect ABC123`
      2. Verify "Connected to jacktym.cuttlefish.ai" message
      3. Check JWT file exists at ~/.config/cuttlefish/tunnel.jwt
    Expected Result: Connection established, JWT saved
    Evidence: .sisyphus/evidence/task-6-cli-connect.txt

  Scenario: Status shows connection info
    Tool: Bash
    Preconditions: Tunnel connected
    Steps:
      1. Run `./target/release/cuttlefish tunnel status`
      2. Verify shows subdomain, uptime, bytes
    Expected Result: Status displayed correctly
    Evidence: .sisyphus/evidence/task-6-cli-status.txt
  ```

  **Commit**: YES
  - Message: `feat(tunnel): add CLI commands for tunnel management`
  - Files: `src/main.rs` or `src/commands/tunnel.rs`

- [ ] 7. Caddy Reverse Proxy Configuration

  **What to do**:
  - Create `deploy/caddy/Caddyfile` for production setup
  - Configure wildcard subdomain routing:
    ```
    *.cuttlefish.ai {
        @tunnel host *.cuttlefish.ai
        handle @tunnel {
            reverse_proxy localhost:8081  # Tunnel daemon HTTP endpoint
        }
    }
    ```
  - Document the Caddy setup in `docs/deployment/tunnel-proxy.md`:
    - DNS wildcard record requirement (*.cuttlefish.ai → server IP)
    - TLS certificate via Let's Encrypt (automatic with Caddy)
    - Caddy installation steps
  - Add environment variable for daemon HTTP port
  - Test configuration locally with modified /etc/hosts

  **Must NOT do**:
  - Don't implement proxy in Rust (use Caddy)
  - Don't hardcode domain names (use config)

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: Configuration file, documentation
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 6, 8)
  - **Parallel Group**: Wave 3
  - **Blocks**: None
  - **Blocked By**: Task 4

  **References**:
  - Caddy docs: https://caddyserver.com/docs/
  - Wildcard certificates: https://caddyserver.com/docs/automatic-https#wildcard-certificates

  **Acceptance Criteria**:
  - [ ] Caddyfile syntax validates
  - [ ] Documentation explains full setup
  - [ ] Local test with /etc/hosts works

  **QA Scenarios**:
  ```
  Scenario: Caddy config validates
    Tool: Bash
    Preconditions: Caddy installed
    Steps:
      1. Run `caddy validate --config deploy/caddy/Caddyfile`
      2. Verify no errors
    Expected Result: Configuration valid
    Evidence: .sisyphus/evidence/task-7-caddy-validate.txt

  Scenario: Local proxy test
    Tool: Bash
    Preconditions: Caddy running, tunnel daemon running, /etc/hosts modified
    Steps:
      1. Add `127.0.0.1 test.cuttlefish.ai` to /etc/hosts
      2. curl http://test.cuttlefish.ai (routed through tunnel)
      3. Verify response from self-hosted instance
    Expected Result: Request proxied correctly
    Evidence: .sisyphus/evidence/task-7-local-proxy.txt
  ```

  **Commit**: YES
  - Message: `docs: add Caddy configuration for tunnel routing`
  - Files: `deploy/caddy/Caddyfile`, `docs/deployment/tunnel-proxy.md`

- [ ] 8. Database Tables for Tunnels

  **What to do**:
  - Add migration in `crates/cuttlefish-db/src/lib.rs` `run_migrations()`:
    ```sql
    CREATE TABLE IF NOT EXISTS tunnel_link_codes (
        id TEXT PRIMARY KEY,
        user_id TEXT NOT NULL,
        code_hash TEXT NOT NULL,
        subdomain TEXT NOT NULL,
        created_at TEXT NOT NULL,
        expires_at TEXT NOT NULL,
        used_at TEXT
    );
    
    CREATE TABLE IF NOT EXISTS active_tunnels (
        id TEXT PRIMARY KEY,
        user_id TEXT NOT NULL,
        subdomain TEXT NOT NULL UNIQUE,
        connected_at TEXT NOT NULL,
        last_heartbeat TEXT NOT NULL,
        client_version TEXT,
        client_ip TEXT
    );
    ```
  - Add model structs in `crates/cuttlefish-db/src/models.rs`
  - Add CRUD methods:
    - `create_link_code()`, `validate_link_code()`, `mark_link_code_used()`
    - `register_tunnel()`, `update_heartbeat()`, `remove_tunnel()`, `get_tunnel_by_subdomain()`
  - Write tests

  **Must NOT do**:
  - Don't store plaintext link codes (hash only)
  - Don't modify existing tables

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: SQL + CRUD methods, follows existing patterns
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 6, 7)
  - **Parallel Group**: Wave 3
  - **Blocks**: None
  - **Blocked By**: Task 5

  **References**:
  - `crates/cuttlefish-db/src/lib.rs` — Existing migration and CRUD patterns
  - `crates/cuttlefish-db/src/models.rs` — Model struct patterns

  **Acceptance Criteria**:
  - [ ] Tables created on startup
  - [ ] Link code CRUD works
  - [ ] Active tunnel CRUD works
  - [ ] Tests pass

  **QA Scenarios**:
  ```
  Scenario: Link code lifecycle
    Tool: Bash (cargo test)
    Preconditions: DB module with tunnel tables
    Steps:
      1. Create link code
      2. Validate it (should succeed)
      3. Mark as used
      4. Validate again (should fail - already used)
    Expected Result: Full lifecycle works
    Evidence: .sisyphus/evidence/task-8-linkcode-crud.txt

  Scenario: Active tunnel registration
    Tool: Bash (cargo test)
    Preconditions: DB module with tunnel tables
    Steps:
      1. Register tunnel with subdomain
      2. Get tunnel by subdomain (should find)
      3. Remove tunnel
      4. Get tunnel by subdomain (should not find)
    Expected Result: Registration and cleanup work
    Evidence: .sisyphus/evidence/task-8-tunnel-crud.txt
  ```

  **Commit**: YES
  - Message: `feat(db): add tunnel-related database tables`
  - Files: `crates/cuttlefish-db/src/lib.rs`, `crates/cuttlefish-db/src/models.rs`

- [ ] 9. Auto-Reconnect with Exponential Backoff

  **What to do**:
  - Modify `crates/cuttlefish-tunnel/src/client.rs`:
    - Add `ReconnectPolicy` struct:
      ```rust
      pub struct ReconnectPolicy {
          initial_delay: Duration,      // 1 second
          max_delay: Duration,          // 5 minutes
          multiplier: f64,              // 2.0
          max_attempts: Option<u32>,    // None = infinite
      }
      ```
    - Wrap `run()` in reconnection loop
    - On disconnect: wait, attempt reconnect with saved JWT
    - Increase delay on each failure (exponential backoff)
    - Reset delay on successful connection
    - Log reconnection attempts with timing
  - Add `--no-reconnect` flag to CLI for one-shot mode
  - Emit events for monitoring (connected, disconnected, reconnecting)

  **Must NOT do**:
  - Don't retry with link code (it's one-time use)
  - Don't reconnect on auth failure (only network issues)

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
    - Reason: Async control flow, state management
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Task 10)
  - **Parallel Group**: Wave 4
  - **Blocks**: Task 10
  - **Blocked By**: Tasks 3, 4

  **References**:
  - Exponential backoff patterns
  - `tokio::time::sleep`

  **Acceptance Criteria**:
  - [ ] Client reconnects after network drop
  - [ ] Delay increases exponentially
  - [ ] Delay resets after success
  - [ ] `--no-reconnect` disables behavior

  **QA Scenarios**:
  ```
  Scenario: Reconnect after disconnect
    Tool: Bash (cargo test)
    Preconditions: Client connected, server can be restarted
    Steps:
      1. Establish connection
      2. Kill server (simulate network drop)
      3. Verify client logs "Reconnecting in 1s"
      4. Restart server
      5. Verify client reconnects
    Expected Result: Automatic reconnection
    Evidence: .sisyphus/evidence/task-9-reconnect.txt

  Scenario: Exponential backoff
    Tool: Bash (cargo test)
    Preconditions: Client with reconnect enabled
    Steps:
      1. Fail connection 5 times
      2. Capture delay values: 1s, 2s, 4s, 8s, 16s
    Expected Result: Delays double each time
    Evidence: .sisyphus/evidence/task-9-backoff.txt
  ```

  **Commit**: YES
  - Message: `feat(tunnel): add auto-reconnect with exponential backoff`
  - Files: `crates/cuttlefish-tunnel/src/client.rs`

- [ ] 10. Tunnel Status Monitoring

  **What to do**:
  - Add HTTP endpoint to tunnel daemon: `GET /status`:
    ```rust
    pub struct TunnelStatusResponse {
        pub active_tunnels: u32,
        pub tunnels: Vec<TunnelInfo>,
    }
    
    pub struct TunnelInfo {
        pub subdomain: String,
        pub connected_since: String,
        pub last_heartbeat: String,
        pub bytes_in: u64,
        pub bytes_out: u64,
    }
    ```
  - Track bytes transferred per tunnel
  - Add metrics to client: connection uptime, requests handled, bytes transferred
  - Enhance `cuttlefish tunnel status` to show detailed stats
  - Add `--json` flag for machine-readable output

  **Must NOT do**:
  - Don't expose sensitive info (user IDs, IPs) in public endpoint
  - Don't add heavy metrics framework (simple counters)

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: HTTP endpoint, simple stats
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Wave 4 (after Task 9)
  - **Blocks**: F1-F4
  - **Blocked By**: Tasks 6, 9

  **References**:
  - `crates/cuttlefish-api/src/api_routes.rs` — HTTP endpoint patterns

  **Acceptance Criteria**:
  - [ ] `/status` endpoint returns tunnel list
  - [ ] Bytes transferred tracked accurately
  - [ ] CLI shows detailed stats
  - [ ] `--json` outputs valid JSON

  **QA Scenarios**:
  ```
  Scenario: Status endpoint returns data
    Tool: Bash (curl)
    Preconditions: Daemon running with 1+ connected tunnel
    Steps:
      1. curl http://localhost:8081/status
      2. Verify JSON with active_tunnels > 0
    Expected Result: Status returned correctly
    Evidence: .sisyphus/evidence/task-10-status-endpoint.txt

  Scenario: CLI status with --json
    Tool: Bash
    Preconditions: Tunnel connected
    Steps:
      1. Run `./target/release/cuttlefish tunnel status --json`
      2. Parse output as JSON
      3. Verify required fields present
    Expected Result: Valid JSON output
    Evidence: .sisyphus/evidence/task-10-cli-json.txt
  ```

  **Commit**: YES
  - Message: `feat(tunnel): add tunnel status monitoring`
  - Files: `crates/cuttlefish-tunnel/src/server.rs`, `src/main.rs`

---

## Final Verification Wave

- [ ] F1. **Plan Compliance Audit** — `oracle`
  Read the plan end-to-end. For each "Must Have": verify implementation exists. For each "Must NOT Have": search codebase for forbidden patterns. Check evidence files exist. Compare deliverables against plan.
  Output: `Must Have [N/N] | Must NOT Have [N/N] | Tasks [N/N] | VERDICT: APPROVE/REJECT`

- [ ] F2. **Code Quality Review** — `unspecified-high`
  Run `cargo clippy --workspace -- -D warnings` + `cargo test -p cuttlefish-tunnel`. Review all changed files for security issues (hardcoded secrets, missing validation). Check for proper error handling.
  Output: `Build [PASS/FAIL] | Lint [PASS/FAIL] | Tests [N pass/N fail] | Security [N issues] | VERDICT`

- [ ] F3. **E2E Tunnel QA** — `deep`
  Start tunnel daemon on test port. Connect client with link code. Send HTTP request through proxy. Verify response. Test reconnection. Test multiple clients. Test invalid auth.
  Output: `Auth [PASS/FAIL] | Routing [PASS/FAIL] | Reconnect [PASS/FAIL] | Multi-client [PASS/FAIL] | VERDICT`

- [ ] F4. **Scope Fidelity Check** — `deep`
  For each task: read "What to do", read actual diff. Verify 1:1 match. Check "Must NOT do" compliance. Detect cross-task contamination. Flag unaccounted changes.
  Output: `Tasks [N/N compliant] | Contamination [CLEAN/N issues] | Unaccounted [CLEAN/N files] | VERDICT`

---

## Commit Strategy

| Wave | Commit |
|------|--------|
| 1 | `feat(tunnel): add tunnel crate and protocol definitions` |
| 2 | `feat(tunnel): implement tunnel client` |
| 2 | `feat(tunnel): implement tunnel server daemon` |
| 2 | `feat(tunnel): add link code and JWT authentication` |
| 3 | `feat(tunnel): add CLI commands for tunnel management` |
| 3 | `docs: add Caddy configuration for tunnel routing` |
| 3 | `feat(db): add tunnel-related database tables` |
| 4 | `feat(tunnel): add auto-reconnect with exponential backoff` |
| 4 | `feat(tunnel): add tunnel status monitoring` |

---

## Success Criteria

### Verification Commands
```bash
cargo test -p cuttlefish-tunnel                    # All tests pass
cargo clippy -p cuttlefish-tunnel -- -D warnings   # Clean
cargo run --bin tunnel-daemon &                    # Daemon starts
cuttlefish tunnel connect ABC123                   # Client connects
curl -H "Host: testuser.cuttlefish.ai" http://localhost:8081/health  # Routed correctly
```

### Final Checklist
- [ ] Tunnel client connects with link code
- [ ] Tunnel client reconnects with JWT
- [ ] HTTP requests routed through tunnel
- [ ] Auto-reconnect works after disconnect
- [ ] Multiple clients can connect simultaneously
- [ ] No unsafe code added
- [ ] All clippy warnings resolved
