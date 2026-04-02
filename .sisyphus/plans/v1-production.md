# V1 Production Readiness — Complete Plan

## TL;DR

> **Quick Summary**: Make Cuttlefish a production-ready AI coding platform by implementing all 7 agents, adding 11 model providers, updating to modern models (OmO stack), and ensuring the README accurately reflects reality.
> 
> **Deliverables**:
> - All 7 agents fully implemented with Rust structs and tests
> - 11 model providers (8 API key + 3 OAuth)
> - Modern model recommendations (Claude 4.6, GPT-5.4, Kimi K2.5, etc.)
> - Polished install experience with one-liner
> - Honest, accurate README
> 
> **Estimated Effort**: Large (5-7 days)
> **Parallel Execution**: YES - 4 waves
> **Critical Path**: Providers (Wave 1) → Agents (Wave 2) → Integration (Wave 3) → Polish (Wave 4)

---

## Context

### Gap Analysis: README vs Reality

| Feature | README Claims | Reality | Gap |
|---------|---------------|---------|-----|
| Agents | 7 agents listed | 3 implemented (Orchestrator, Coder, Critic) | Need 4 more |
| Providers | Bedrock + Claude OAuth | Only those 2 | Need 9 more |
| Models | Claude 3.x (outdated) | Should be Claude 4.6, GPT-5.4, etc. | Update tables |
| Install | install.sh exists | Not prominently featured | Add one-liner |

### Target Model Stack (OmO Current)

**Agent → Model Mapping:**
| Agent Role | Primary Models |
|------------|----------------|
| Sisyphus (Deep Work) | Claude Opus 4.6, Kimi K2.5, GLM 5 |
| Hephaestus (Code) | GPT-5.4 |
| Prometheus (Planning) | Claude Opus 4.6, Kimi K2.5, GLM 5 |
| Oracle (Analysis) | GPT-5.4 |

**Model Families:**

| Family | Behavior | Models |
|--------|----------|--------|
| Claude-like | Instruction-following, structured output | Claude Opus 4.6, Sonnet 4.6, Haiku 4.5, Kimi K2.5, GLM 5 |
| GPT | Explicit reasoning, principle-driven | GPT-5.4, GPT-5-Nano |
| Specialized | Domain-optimized | Gemini 3.1 Pro (visual), MiniMax M2.7 (fast), Grok Code Fast 1 (search) |

---

## Work Objectives

### Core Objective
Transform Cuttlefish from a partial implementation into a production-ready coding platform with full feature parity to README claims.

### Concrete Deliverables

**Agents (7 total):**
- ✅ Orchestrator (exists)
- ✅ Coder (exists)
- ✅ Critic (exists)
- 🔲 Planner
- 🔲 Explorer
- 🔲 Librarian
- 🔲 DevOps

**Providers (11 total):**
- ✅ AWS Bedrock (exists)
- ✅ Claude OAuth (exists)
- 🔲 Anthropic API (direct)
- 🔲 OpenAI API
- 🔲 ChatGPT OAuth
- 🔲 Google Vertex AI
- 🔲 Moonshot (Kimi)
- 🔲 Zhipu (GLM)
- 🔲 MiniMax
- 🔲 xAI (Grok)
- 🔲 Ollama (local)

### Must Have
- All 7 agents working in orchestrated workflows
- At least 8 providers functional
- README with accurate, current information
- One-liner install in README
- All tests passing

### Must NOT Have
- Outdated model names (Claude 3.x → 4.6)
- README claims without implementation
- Placeholder/mock code in production paths

---

## Verification Strategy

### Test Requirements
- Unit tests for each provider
- Unit tests for each agent
- Integration tests for multi-agent workflows
- E2E tests with mock providers

### QA Policy
- `cargo test --workspace` must pass
- `cargo clippy --workspace -- -D warnings` must be clean
- Manual verification of install.sh on fresh system

---

## Execution Strategy

### Parallel Execution Waves

```
Wave 1 (Providers — all parallel, 6 tasks):
├── T1: OpenAI API provider [deep]
├── T2: Anthropic direct API provider [deep]
├── T3: Google Vertex AI provider [deep]
├── T4: Moonshot (Kimi) provider [unspecified-high]
├── T5: ChatGPT OAuth provider [deep]
└── T6: Additional providers (Zhipu, MiniMax, xAI, Ollama) [unspecified-high]

Wave 2 (Agents — all parallel, 5 tasks):
├── T7: Planner agent [deep]
├── T8: Explorer agent [quick]
├── T9: Librarian agent [quick]
├── T10: DevOps agent [unspecified-high]
└── T11: Update workflow for all agents [deep]

Wave 3 (Integration — parallel then sequential, 3 tasks):
├── T12: Provider registry and config schema [quick]
├── T13: Category-based model routing [deep]
└── T14: Agent dispatch system [deep]

Wave 4 (Polish — parallel, 4 tasks):
├── T15: Update README completely [writing]
├── T16: Improve install.sh [unspecified-high]
├── T17: Update cuttlefish.example.toml [quick]
└── T18: Provider setup documentation [writing]

Wave FINAL (Verification — 4 parallel reviews):
├── F1: Plan compliance audit [oracle]
├── F2: Code quality review [unspecified-high]
├── F3: E2E testing [deep]
└── F4: README accuracy check [deep]
```

---

## TODOs

### Wave 1: Model Providers

- [ ] 1. OpenAI API Provider

  **What to do**:
  - Create `crates/cuttlefish-providers/src/openai.rs`
  - Implement `ModelProvider` trait
  - Models: `gpt-5.4`, `gpt-5-nano`, `gpt-4o` (fallback)
  - Auth: `OPENAI_API_KEY` env var
  - Features: streaming, function calling, JSON mode
  - Error handling: rate limits, token limits, auth failures

  **References**:
  - `crates/cuttlefish-providers/src/bedrock.rs` — existing provider pattern
  - `crates/cuttlefish-core/src/traits.rs` — ModelProvider trait
  - OpenAI API docs: https://platform.openai.com/docs/api-reference

  **Acceptance Criteria**:
  - [ ] `cargo test -p cuttlefish-providers openai` passes
  - [ ] Streaming responses work
  - [ ] Rate limit errors handled gracefully

  **Recommended Agent Profile**: `deep`
  **Parallelization**: Wave 1, parallel with T2-T6

- [ ] 2. Anthropic Direct API Provider

  **What to do**:
  - Create `crates/cuttlefish-providers/src/anthropic.rs`
  - Implement `ModelProvider` trait
  - Models: `claude-opus-4-6`, `claude-sonnet-4-6`, `claude-haiku-4-5`
  - Auth: `ANTHROPIC_API_KEY` env var
  - Reuse message types from existing Claude code where possible

  **References**:
  - `crates/cuttlefish-providers/src/claude_oauth.rs` — message format
  - Anthropic API docs: https://docs.anthropic.com/en/api

  **Acceptance Criteria**:
  - [ ] `cargo test -p cuttlefish-providers anthropic` passes
  - [ ] All 3 model tiers work
  - [ ] Compatible with existing Claude message format

  **Recommended Agent Profile**: `deep`
  **Parallelization**: Wave 1, parallel with T1, T3-T6

- [ ] 3. Google Vertex AI Provider

  **What to do**:
  - Create `crates/cuttlefish-providers/src/google.rs`
  - Implement `ModelProvider` trait
  - Models: `gemini-3.1-pro`
  - Auth: `GOOGLE_API_KEY` or service account JSON
  - Handle Gemini's different message format

  **References**:
  - Google AI docs: https://ai.google.dev/docs

  **Acceptance Criteria**:
  - [ ] `cargo test -p cuttlefish-providers google` passes
  - [ ] API key auth works
  - [ ] Multi-modal support (images) if time permits

  **Recommended Agent Profile**: `deep`
  **Parallelization**: Wave 1, parallel with T1-T2, T4-T6

- [ ] 4. Moonshot (Kimi) Provider

  **What to do**:
  - Create `crates/cuttlefish-providers/src/moonshot.rs`
  - Implement `ModelProvider` trait
  - Models: `kimi-k2.5`
  - Auth: `MOONSHOT_API_KEY` env var
  - API is OpenAI-compatible, so can reuse request format

  **References**:
  - Moonshot API (OpenAI-compatible format)

  **Acceptance Criteria**:
  - [ ] `cargo test -p cuttlefish-providers moonshot` passes
  - [ ] Works with kimi-k2.5 model

  **Recommended Agent Profile**: `unspecified-high`
  **Parallelization**: Wave 1, parallel with T1-T3, T5-T6

- [ ] 5. ChatGPT OAuth Provider

  **What to do**:
  - Create `crates/cuttlefish-providers/src/chatgpt_oauth.rs`
  - Implement OAuth PKCE flow (copy pattern from `claude_oauth.rs`)
  - Support ChatGPT Plus/Pro accounts
  - Handle token storage and refresh
  - Integrate with `oauth_flow.rs` patterns

  **References**:
  - `crates/cuttlefish-providers/src/claude_oauth.rs` — OAuth pattern
  - `crates/cuttlefish-providers/src/oauth_flow.rs` — PKCE helpers

  **Acceptance Criteria**:
  - [ ] OAuth flow opens browser and completes
  - [ ] Token refresh works
  - [ ] Can send messages via authenticated session

  **Recommended Agent Profile**: `deep`
  **Parallelization**: Wave 1, parallel with T1-T4, T6

- [ ] 6. Additional Providers (Zhipu, MiniMax, xAI, Ollama)

  **What to do**:
  - Create `crates/cuttlefish-providers/src/zhipu.rs` — GLM 5
  - Create `crates/cuttlefish-providers/src/minimax.rs` — M2.7, M2.7-highspeed
  - Create `crates/cuttlefish-providers/src/xai.rs` — Grok Code Fast 1
  - Create `crates/cuttlefish-providers/src/ollama.rs` — local models via Ollama API
  - All implement `ModelProvider` trait
  - Most are OpenAI-compatible, so can share request code

  **References**:
  - Each provider's API documentation
  - Ollama API: http://localhost:11434/api

  **Acceptance Criteria**:
  - [ ] Each provider has working tests
  - [ ] Ollama works with local models
  - [ ] All providers handle errors gracefully

  **Recommended Agent Profile**: `unspecified-high`
  **Parallelization**: Wave 1 (late), parallel with T1-T5

---

### Wave 2: Remaining Agents

- [ ] 7. Planner Agent

  **What to do**:
  - Create `crates/cuttlefish-agents/src/planner.rs`
  - Implement same traits as existing agents
  - Load system prompt from `prompts/planner.md`
  - Tools: `read_file`, `search_codebase`, `list_files`, `create_plan`
  - Role: Creates strategic implementation plans before coding
  - Optimal category: `ultrabrain`

  **References**:
  - `crates/cuttlefish-agents/src/orchestrator.rs` — agent pattern
  - `prompts/planner.md` — system prompt (already exists)
  - `crates/cuttlefish-agents/src/tools.rs` — tool definitions

  **Acceptance Criteria**:
  - [ ] Agent struct compiles and loads prompt
  - [ ] Tools are registered correctly
  - [ ] Unit tests pass
  - [ ] Can generate a plan from a task description

  **Recommended Agent Profile**: `deep`
  **Parallelization**: Wave 2, parallel with T8-T10

- [ ] 8. Explorer Agent

  **What to do**:
  - Create `crates/cuttlefish-agents/src/explorer.rs`
  - Specialized for codebase discovery
  - Tools: `grep`, `find_files`, `read_file`, `lsp_references`, `lsp_symbols`
  - Role: Searches codebase to find relevant code for tasks
  - Optimal category: `quick` (fast searches)

  **References**:
  - `prompts/explorer.md` — system prompt (already exists)

  **Acceptance Criteria**:
  - [ ] Agent can search codebase
  - [ ] Returns relevant file locations
  - [ ] Unit tests pass

  **Recommended Agent Profile**: `quick`
  **Parallelization**: Wave 2, parallel with T7, T9-T10

- [ ] 9. Librarian Agent

  **What to do**:
  - Create `crates/cuttlefish-agents/src/librarian.rs`
  - Specialized for external documentation lookup
  - Tools: `web_search`, `fetch_url`, `read_docs`, `search_docs`
  - Role: Finds documentation for libraries, APIs, frameworks
  - Optimal category: `quick`

  **References**:
  - `prompts/librarian.md` — system prompt (already exists)

  **Acceptance Criteria**:
  - [ ] Agent can search web for docs
  - [ ] Can fetch and parse documentation pages
  - [ ] Unit tests pass

  **Recommended Agent Profile**: `quick`
  **Parallelization**: Wave 2, parallel with T7-T8, T10

- [ ] 10. DevOps Agent

  **What to do**:
  - Create `crates/cuttlefish-agents/src/devops.rs`
  - Specialized for build, deploy, infrastructure
  - Tools: `run_command`, `docker_exec`, `check_ci`, `deploy`
  - Role: Handles builds, deployments, CI/CD operations
  - Optimal category: `unspecified-high`

  **References**:
  - `prompts/devops.md` — system prompt (already exists)

  **Acceptance Criteria**:
  - [ ] Agent can run build commands
  - [ ] Can check CI status
  - [ ] Unit tests pass

  **Recommended Agent Profile**: `unspecified-high`
  **Parallelization**: Wave 2, parallel with T7-T9

- [ ] 11. Update Workflow for All Agents

  **What to do**:
  - Update `crates/cuttlefish-agents/src/workflow.rs`
  - Add Planner step before Coder for complex tasks
  - Add Explorer dispatch from Orchestrator for codebase questions
  - Add Librarian dispatch for external documentation needs
  - Add DevOps dispatch for build/deploy tasks
  - Update `orchestrator.rs` to route to new agents
  - Add integration tests for multi-agent scenarios

  **References**:
  - `crates/cuttlefish-agents/src/workflow.rs` — current workflow
  - `crates/cuttlefish-agents/src/orchestrator.rs` — dispatch logic

  **Acceptance Criteria**:
  - [ ] Orchestrator can dispatch to all 7 agents
  - [ ] Planner → Coder → Critic flow works
  - [ ] Explorer/Librarian called when needed
  - [ ] Integration tests pass

  **Recommended Agent Profile**: `deep`
  **Parallelization**: Wave 2 (after T7-T10)

---

### Wave 3: Integration

- [ ] 12. Provider Registry and Config Schema

  **What to do**:
  - Create `crates/cuttlefish-providers/src/registry.rs`
  - Registry that holds all provider instances
  - Update `crates/cuttlefish-core/src/config.rs` with all provider configs
  - Each provider has: name, auth method, model list, default model

  **References**:
  - `crates/cuttlefish-core/src/config.rs` — current config
  - `cuttlefish.example.toml` — example config

  **Acceptance Criteria**:
  - [ ] All providers configurable in TOML
  - [ ] Registry can instantiate any provider
  - [ ] Config validation works

  **Recommended Agent Profile**: `quick`
  **Parallelization**: Wave 3, parallel with T13-T14

- [ ] 13. Category-Based Model Routing

  **What to do**:
  - Create routing logic that maps categories to providers/models
  - Categories: `ultrabrain`, `deep`, `code`, `visual`, `quick`, `search`, `unspecified-high`, `unspecified-low`
  - Each category has default provider + model, with fallbacks
  - User can override per-category in config

  **Config example**:
  ```toml
  [routing]
  ultrabrain = { provider = "anthropic", model = "claude-opus-4-6" }
  deep = { provider = "openai", model = "gpt-5.4" }
  quick = { provider = "anthropic", model = "claude-haiku-4-5" }
  visual = { provider = "google", model = "gemini-3.1-pro" }
  ```

  **Acceptance Criteria**:
  - [ ] Routing picks correct provider for category
  - [ ] Fallback to default if preferred unavailable
  - [ ] Config overrides work

  **Recommended Agent Profile**: `deep`
  **Parallelization**: Wave 3, parallel with T12, T14

- [ ] 14. Agent Dispatch System

  **What to do**:
  - Connect agents to routing system
  - Each agent declares its preferred category
  - Orchestrator uses category to select provider
  - Support runtime model switching

  **Agent categories**:
  - Orchestrator: `deep`
  - Planner: `ultrabrain`
  - Coder: `code` or `deep`
  - Critic: `deep`
  - Explorer: `quick`
  - Librarian: `quick`
  - DevOps: `unspecified-high`

  **Acceptance Criteria**:
  - [ ] Each agent gets routed to appropriate model
  - [ ] Category overrides work
  - [ ] Tests verify routing

  **Recommended Agent Profile**: `deep`
  **Parallelization**: Wave 3, parallel with T12-T13

---

### Wave 4: Polish & Documentation

- [ ] 15. Update README Completely

  **What to do**:
  - Remove "v1 ships with 3 agents" (now 7)
  - Update model recommendations table with OmO models
  - Update provider table with all 11 providers
  - Add prominent install command at top
  - Update architecture diagram to show all 7 agents
  - Add model family explanations
  - Remove any claims about unimplemented features
  - Add comparison table vs other AI coding tools

  **New install section**:
  ```markdown
  ## Quick Start
  
  ```bash
  curl -sSL https://cuttlefish.dev/install.sh | bash
  ```
  
  Or manually:
  ```bash
  cargo install cuttlefish
  ```
  ```

  **Acceptance Criteria**:
  - [ ] All tables accurate
  - [ ] No outdated model names
  - [ ] Install command prominent
  - [ ] Architecture diagram current

  **Recommended Agent Profile**: `writing`
  **Parallelization**: Wave 4, parallel with T16-T18

- [ ] 16. Improve install.sh

  **What to do**:
  - Add provider selection menu
  - Let user choose which providers to configure
  - Validate API keys on entry
  - Add model selection by category
  - Test on fresh Ubuntu 22.04/24.04
  - Add completion verification step

  **Acceptance Criteria**:
  - [ ] Works on fresh Ubuntu
  - [ ] Provider selection is intuitive
  - [ ] API key validation helpful
  - [ ] Creates working config

  **Recommended Agent Profile**: `unspecified-high`
  **Parallelization**: Wave 4, parallel with T15, T17-T18

- [ ] 17. Update cuttlefish.example.toml

  **What to do**:
  - Add all 11 providers with example config
  - Add category routing config
  - Add agent-specific overrides
  - Document each option with comments
  - Include sensible defaults

  **Acceptance Criteria**:
  - [ ] All providers represented
  - [ ] Comments explain each option
  - [ ] Defaults are reasonable
  - [ ] Copy-paste ready

  **Recommended Agent Profile**: `quick`
  **Parallelization**: Wave 4, parallel with T15-T16, T18

- [ ] 18. Provider Setup Documentation

  **What to do**:
  - Create `docs/providers/` directory
  - One doc per provider with setup instructions
  - Include API key acquisition steps
  - Include OAuth flow instructions
  - Add troubleshooting sections

  **Files to create**:
  - `docs/providers/anthropic.md`
  - `docs/providers/openai.md`
  - `docs/providers/google.md`
  - `docs/providers/moonshot.md`
  - `docs/providers/zhipu.md`
  - `docs/providers/minimax.md`
  - `docs/providers/xai.md`
  - `docs/providers/ollama.md`
  - `docs/providers/oauth.md` (for Claude/ChatGPT OAuth)

  **Acceptance Criteria**:
  - [ ] Each provider has clear setup steps
  - [ ] Links to official docs included
  - [ ] Troubleshooting covers common issues

  **Recommended Agent Profile**: `writing`
  **Parallelization**: Wave 4, parallel with T15-T17

---

## Final Verification Wave

- [ ] F1. **Plan Compliance Audit** — `oracle`
  
  Verify all deliverables from this plan are implemented:
  - All 7 agents have Rust files with tests
  - All 11 providers implemented
  - README matches implementation
  - Config supports all providers
  
  Output: `Agents [7/7] | Providers [11/11] | README [ACCURATE/OUTDATED] | VERDICT`

- [ ] F2. **Code Quality Review** — `unspecified-high`
  
  Run full quality checks:
  - `cargo clippy --workspace -- -D warnings`
  - `cargo test --workspace`
  - `cargo fmt --all -- --check`
  - Check for unwrap/expect in library code
  
  Output: `Clippy [PASS/FAIL] | Tests [N pass] | Fmt [PASS/FAIL] | VERDICT`

- [ ] F3. **E2E Testing** — `deep`
  
  Test complete workflows:
  - Provider initialization for each
  - Agent dispatch to correct providers
  - Multi-agent workflow execution
  - Install.sh on fresh system
  
  Output: `Providers [N/11 working] | Agents [N/7 working] | Install [PASS/FAIL] | VERDICT`

- [ ] F4. **README Accuracy Check** — `deep`
  
  Line-by-line README verification:
  - Every claimed feature exists
  - Every model name is current
  - Every provider listed works
  - Install instructions work
  
  Output: `Claims [N/N verified] | Models [CURRENT/OUTDATED] | VERDICT`

---

## Success Criteria

### Quantitative
- [ ] 7/7 agents implemented with tests
- [ ] 11/11 providers working
- [ ] 100% test pass rate
- [ ] 0 clippy warnings
- [ ] install.sh works on fresh Ubuntu

### Qualitative
- [ ] README is honest and accurate
- [ ] Model recommendations match OmO stack
- [ ] User can get started in < 5 minutes
- [ ] Config is well-documented

---

## Appendix: Updated Tables for README

### Model Providers Table

| Provider | Auth Method | Models | Best For |
|----------|-------------|--------|----------|
| **Anthropic** | API Key | Claude Opus 4.6, Sonnet 4.6, Haiku 4.5 | General, planning |
| **OpenAI** | API Key | GPT-5.4, GPT-5-Nano | Coding, reasoning |
| **Google Vertex** | API Key | Gemini 3.1 Pro | Visual, frontend |
| **Moonshot** | API Key | Kimi K2.5 | Claude-like tasks |
| **Zhipu** | API Key | GLM 5 | Broad tasks |
| **MiniMax** | API Key | M2.7, M2.7-highspeed | Fast utility |
| **xAI** | API Key | Grok Code Fast 1 | Code search |
| **AWS Bedrock** | IAM | Claude family, others | Enterprise |
| **Ollama** | Local | Any GGUF | Privacy, offline |
| **Claude OAuth** | PKCE | Claude (via claude.ai) | Personal accounts |
| **ChatGPT OAuth** | PKCE | GPT (via ChatGPT Plus/Pro) | Personal accounts |

### Category Routing Table

| Category | Use Case | Default Model | Alternatives |
|----------|----------|---------------|--------------|
| `ultrabrain` | Hard logic, architecture | Claude Opus 4.6 | Kimi K2.5, GLM 5 |
| `deep` | Complex autonomous work | GPT-5.4 | Claude Opus 4.6 |
| `code` | Code generation | GPT-5.4 | Claude Sonnet 4.6 |
| `visual` | Frontend, UI/UX | Gemini 3.1 Pro | Claude Sonnet 4.6 |
| `quick` | Simple, fast tasks | Claude Haiku 4.5 | GPT-5-Nano, MiniMax M2.7-highspeed |
| `search` | Code grep, pattern finding | Grok Code Fast 1 | MiniMax M2.7 |
| `unspecified-high` | General, higher effort | Claude Sonnet 4.6 | GPT-5.4 |
| `unspecified-low` | General, lower effort | Claude Haiku 4.5 | GPT-5-Nano |

### Agent → Category Mapping

| Agent | Role | Category | Why |
|-------|------|----------|-----|
| Orchestrator | Coordination | `deep` | Needs good reasoning |
| Planner | Strategy | `ultrabrain` | Complex planning |
| Coder | Implementation | `code` | Code generation |
| Critic | Review | `deep` | Analytical |
| Explorer | Search | `quick` | Fast lookups |
| Librarian | Docs | `quick` | Fast retrieval |
| DevOps | Infrastructure | `unspecified-high` | Varied tasks |
