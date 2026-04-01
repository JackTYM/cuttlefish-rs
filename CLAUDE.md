# Cuttlefish — Developer Guide for AI Assistants

## Project Identity
- **Language**: Rust 2024 edition
- **Architecture**: Cargo workspace with 9 focused crates
- **Philosophy**: Traits before implementations, zero unsafe code

## Non-Negotiable Rules

### Code Quality
- `#![deny(unsafe_code)]` is enforced workspace-wide — never add unsafe
- `#![deny(clippy::unwrap_used)]` on all lib crates — use `?` or `expect("reason")`
- `#![warn(missing_docs)]` — every public item needs `///` documentation
- No `println!` debugging — use `tracing::debug!`, `tracing::info!`, etc.
- No global mutable state — no `lazy_static!`, no `once_cell::sync::Lazy`

### Error Handling
- Library crates use `thiserror` — define specific error types
- Binary crates (main.rs) may use `anyhow` for top-level error handling
- Error types live in `cuttlefish-core::error`

### Architecture
- Traits are defined in `cuttlefish-core::traits` — no trait definitions in impl crates
- Concrete implementations depend on trait interfaces, not other concrete types
- All I/O is behind traits to enable testing without real services

## Crate Dependency Rules
```
cuttlefish-core (no deps on other crates)
    ↑
cuttlefish-db, cuttlefish-providers, cuttlefish-sandbox, cuttlefish-vcs
    ↑
cuttlefish-agents (depends on all above)
    ↑
cuttlefish-discord, cuttlefish-api, cuttlefish-tui (interface crates)
    ↑
cuttlefish-rs (root binary — wires everything)
```

## Development Workflow
1. Write the test first (TDD)
2. Implement minimally to make the test pass
3. Run: `cargo clippy -p <crate> -- -D warnings` — must be clean
4. Run: `cargo test -p <crate>` — all pass
5. Commit: `<type>(<crate>): <description>`

## Commit Convention
Format: `<type>(<crate>): <description>`
Types: `feat`, `fix`, `refactor`, `test`, `docs`, `chore`
Example: `feat(sandbox): add Docker container lifecycle management`

## Testing
- Unit tests: in `#[cfg(test)]` blocks at bottom of each module
- Integration tests requiring real services: behind `#[cfg(feature = "integration")]`
- Mock implementations live alongside the trait (e.g., `MockModelProvider`)
- All async tests use `#[tokio::test]`

## Configuration
- User config: `cuttlefish.toml` (not committed)
- Example: `cuttlefish.example.toml` (committed)
- Secrets: environment variables only (never in TOML)
- Key env vars: `CUTTLEFISH_API_KEY`, `DISCORD_BOT_TOKEN`, `RUST_LOG`
