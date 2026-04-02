# V1 Agent System Completion — OmO-Style Prompts & Multi-Model Dispatch

## TL;DR

> **Quick Summary**: Expand Cuttlefish's agent prompts from ~21 lines to 700+ lines (OmO-style), implement runtime prompt loading, and wire agents to use category-based model routing for autonomous background execution.
> 
> **Deliverables**:
> - 7 detailed agent prompt files (50-150 lines each)
> - `PromptRegistry` for runtime prompt loading
> - Updated README with expanded cuttlefish analogy
> - End-to-end verification of multi-model dispatch
> 
> **Estimated Effort**: Medium (2-3 days)
> **Parallel Execution**: YES - 4 waves
> **Critical Path**: Task 1 (PromptRegistry) → Task 2-8 (Prompts) → Task 9 (Wire) → Task 10 (Verify)

---

## Context

### Original Request
Complete Cuttlefish-rs to V1 release quality with OmO-style detailed agent prompts and multi-model dispatch, ensuring agents work autonomously after the questions phase.

### Interview Summary
**Key Discussions**:
- **Autonomy model**: Planner agent asks questions upfront, then Orchestrator dispatches work autonomously in background
- **Prompt storage**: External `.md` files loaded at runtime (not hardcoded in Rust)
- **Model routing**: Already implemented in `AgentRoutingConfig`, needs agents to USE it

**Research Findings**:
- OmO has 30 agent files totaling 3,851 lines at `/home/jack/everything-claude-code/agents/`
- Cuttlefish currently has only 21 lines of prompts across 3 agents
- Model routing architecture is complete but agents don't load prompts dynamically

### Metis Review
**Identified Gaps** (addressed in plan):
- Prompt files need YAML frontmatter for tool/model metadata
- PromptRegistry needs caching for performance
- README needs cuttlefish analogy content (multi-colored = multi-agent)

---

## Work Objectives

### Core Objective
Transform Cuttlefish's minimal agent prompts into OmO-style comprehensive instructions, enable runtime prompt loading, and ensure autonomous multi-model execution.

### Concrete Deliverables
- `prompts/orchestrator.md` (100+ lines)
- `prompts/planner.md` (100+ lines)
- `prompts/coder.md` (150+ lines)
- `prompts/critic.md` (100+ lines)
- `prompts/explorer.md` (80+ lines)
- `prompts/librarian.md` (80+ lines)
- `prompts/devops.md` (100+ lines)
- `crates/cuttlefish-agents/src/prompt_registry.rs` (new file)
- Updated `README.md` with cuttlefish analogy

### Definition of Done
- [ ] `cargo test -p cuttlefish-agents` passes with new prompt loading tests
- [ ] `cargo clippy --workspace -- -D warnings` clean
- [ ] All 7 prompt files exist with YAML frontmatter
- [ ] Agent execution uses loaded prompts (not hardcoded)
- [ ] README contains expanded cuttlefish analogy section

### Must Have
- YAML frontmatter in each prompt file (name, description, tools, model category)
- Runtime prompt loading (not compile-time include_str!)
- Error handling for missing/malformed prompts
- Caching to avoid re-reading files on every invocation

### Must NOT Have (Guardrails)
- No unsafe code
- No `unwrap()` — use `?` or `expect("reason")`
- No hardcoded model IDs in prompts (use categories)
- No changes to database schema
- No changes to WebUI/Discord interfaces
- No AI slop (excessive comments, over-abstraction)

---

## Verification Strategy

> **ZERO HUMAN INTERVENTION** — ALL verification is agent-executed.

### Test Decision
- **Infrastructure exists**: YES (176 tests pass)
- **Automated tests**: YES (Tests-after)
- **Framework**: Rust's `#[test]` and `#[tokio::test]`

### QA Policy
Every task includes agent-executed QA scenarios.
Evidence saved to `.sisyphus/evidence/task-{N}-{scenario-slug}.{ext}`.

- **Rust code**: Use `cargo test`, `cargo clippy`, `cargo build`
- **File content**: Use `grep`, `cat`, line counting
- **Integration**: Use `cargo run` with test fixtures

---

## Execution Strategy

### Parallel Execution Waves

```
Wave 1 (Foundation — start immediately):
├── Task 1: Create PromptRegistry with YAML parsing [deep]
└── Task 2: Create prompt template structure [quick]

Wave 2 (Prompt Files — MAX PARALLEL, after Wave 1):
├── Task 3: Create orchestrator.md prompt [unspecified-high]
├── Task 4: Create planner.md prompt [unspecified-high]
├── Task 5: Create coder.md prompt [unspecified-high]
├── Task 6: Create critic.md prompt [unspecified-high]
├── Task 7: Create explorer.md + librarian.md prompts [unspecified-high]
└── Task 8: Create devops.md prompt [unspecified-high]

Wave 3 (Integration — after Wave 2):
├── Task 9: Wire PromptRegistry into agent execution [deep]
└── Task 10: Expand README with cuttlefish analogy [writing]

Wave FINAL (Verification — after ALL tasks):
├── Task F1: Plan compliance audit [oracle]
├── Task F2: Code quality review [unspecified-high]
├── Task F3: End-to-end agent execution test [unspecified-high]
└── Task F4: Scope fidelity check [deep]
→ Present results → Get explicit user okay

Critical Path: Task 1 → Task 3-8 → Task 9 → F1-F4 → user okay
Parallel Speedup: ~60% faster than sequential
Max Concurrent: 6 (Wave 2)
```

### Dependency Matrix

| Task | Depends On | Blocks | Wave |
|------|------------|--------|------|
| 1 | — | 3-8, 9 | 1 |
| 2 | — | 3-8 | 1 |
| 3-8 | 1, 2 | 9 | 2 |
| 9 | 1, 3-8 | F1-F4 | 3 |
| 10 | — | F1-F4 | 3 |
| F1-F4 | 9, 10 | — | FINAL |

### Agent Dispatch Summary

- **Wave 1**: 2 tasks — T1 → `deep`, T2 → `quick`
- **Wave 2**: 6 tasks — T3-T8 → `unspecified-high`
- **Wave 3**: 2 tasks — T9 → `deep`, T10 → `writing`
- **FINAL**: 4 tasks — F1 → `oracle`, F2-F3 → `unspecified-high`, F4 → `deep`

---

## TODOs

- [ ] 1. Create PromptRegistry with YAML Parsing

  **What to do**:
  - Create `crates/cuttlefish-agents/src/prompt_registry.rs`
  - Define `PromptMetadata` struct with fields: `name`, `description`, `tools: Vec<String>`, `category: String`
  - Define `AgentPrompt` struct with `metadata: PromptMetadata` and `body: String`
  - Implement `PromptRegistry` with:
    - `new(prompts_dir: PathBuf)` constructor
    - `load(&self, agent_name: &str) -> Result<AgentPrompt, PromptError>`
    - Internal `HashMap<String, AgentPrompt>` cache
  - Use `serde_yaml` for frontmatter parsing (already in workspace deps)
  - Add `PromptError` to error types (missing file, invalid YAML, missing body)
  - Add `pub mod prompt_registry;` to `lib.rs`
  - Write 4 unit tests: load valid, missing file, invalid YAML, caching works

  **Must NOT do**:
  - No `unwrap()` — use `?` and `thiserror`
  - No unsafe code
  - No compile-time `include_str!` — must be runtime loading

  **Recommended Agent Profile**:
  - **Category**: `deep`
    - Reason: Core infrastructure code requiring careful error handling
  - **Skills**: []
    - No special skills needed — standard Rust patterns

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Task 2)
  - **Parallel Group**: Wave 1
  - **Blocks**: Tasks 3-9
  - **Blocked By**: None

  **References**:
  - `crates/cuttlefish-core/src/error.rs` — Error type patterns (thiserror usage)
  - `crates/cuttlefish-core/src/config.rs:149-188` — TOML loading pattern (similar file loading)
  - `crates/cuttlefish-agents/src/lib.rs` — Where to add `pub mod prompt_registry`
  - OmO example: `/home/jack/everything-claude-code/agents/code-reviewer.md` — YAML frontmatter format

  **Acceptance Criteria**:
  - [ ] File exists: `crates/cuttlefish-agents/src/prompt_registry.rs`
  - [ ] `cargo build -p cuttlefish-agents` → SUCCESS
  - [ ] `cargo test -p cuttlefish-agents prompt_registry` → 4 tests pass

  **QA Scenarios**:
  ```
  Scenario: Load valid prompt file
    Tool: Bash
    Preconditions: Test prompt file exists at test fixtures path
    Steps:
      1. cargo test -p cuttlefish-agents test_load_valid_prompt -- --nocapture
      2. Check test output contains "test test_load_valid_prompt ... ok"
    Expected Result: Test passes, AgentPrompt struct populated correctly
    Failure Indicators: Test fails, panic, or "FAILED" in output
    Evidence: .sisyphus/evidence/task-1-load-valid.txt

  Scenario: Handle missing prompt file gracefully
    Tool: Bash
    Preconditions: No prompt file at requested path
    Steps:
      1. cargo test -p cuttlefish-agents test_load_missing_prompt -- --nocapture
      2. Check test output shows PromptError::NotFound returned
    Expected Result: Returns Err(PromptError::NotFound), no panic
    Failure Indicators: Panic, unwrap failure, or unexpected error type
    Evidence: .sisyphus/evidence/task-1-missing-file.txt
  ```

  **Commit**: YES
  - Message: `feat(agents): add PromptRegistry for runtime prompt loading`
  - Files: `crates/cuttlefish-agents/src/prompt_registry.rs`, `crates/cuttlefish-agents/src/lib.rs`
  - Pre-commit: `cargo test -p cuttlefish-agents && cargo clippy -p cuttlefish-agents -- -D warnings`

- [ ] 2. Create Prompt Template Structure

  **What to do**:
  - Create `prompts/` directory in project root
  - Create `prompts/_template.md` as reference for prompt authors:
    ```yaml
    ---
    name: agent-name
    description: One-line description of agent purpose
    tools: ["Read", "Write", "Bash", "Grep", "Glob"]
    category: deep  # or quick, ultrabrain, visual, unspecified-high, unspecified-low
    ---
    
    You are the [Role] agent...
    
    ## Your Responsibilities
    ...
    
    ## Process
    1. ...
    
    ## Constraints
    - ...
    
    ## Output Format
    ...
    ```
  - Add `prompts/` to `.gitignore` exception if needed (should be tracked)

  **Must NOT do**:
  - No actual agent content yet — just the template
  - No Rust code changes

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: Simple file creation, no complex logic
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Task 1)
  - **Parallel Group**: Wave 1
  - **Blocks**: Tasks 3-8
  - **Blocked By**: None

  **References**:
  - OmO template pattern: `/home/jack/everything-claude-code/agents/code-reviewer.md:1-10` — YAML frontmatter example
  - Current agent categories: `crates/cuttlefish-core/src/traits/agent.rs:32-48` — Category enum values

  **Acceptance Criteria**:
  - [ ] Directory exists: `prompts/`
  - [ ] File exists: `prompts/_template.md`
  - [ ] Template has valid YAML frontmatter (parseable)

  **QA Scenarios**:
  ```
  Scenario: Template file is valid YAML
    Tool: Bash
    Preconditions: prompts/_template.md exists
    Steps:
      1. head -20 prompts/_template.md
      2. Verify output starts with "---" and contains name, description, tools, category
      3. python3 -c "import yaml; yaml.safe_load(open('prompts/_template.md').read().split('---')[1])"
    Expected Result: YAML parses without error, all required fields present
    Failure Indicators: yaml.YAMLError, missing fields, no "---" delimiter
    Evidence: .sisyphus/evidence/task-2-template-valid.txt
  ```

  **Commit**: YES (groups with Task 1)
  - Message: `feat(agents): add prompt template structure`
  - Files: `prompts/_template.md`
  - Pre-commit: None (no Rust code)

- [ ] 3. Create Orchestrator Prompt (orchestrator.md)

  **What to do**:
  - Create `prompts/orchestrator.md` (100+ lines)
  - YAML frontmatter: `name: orchestrator`, `category: deep`, `tools: ["Read", "Glob", "Grep"]`
  - Role: Receives user requests, decomposes into tasks, dispatches to other agents
  - Sections to include:
    - Identity & Authority (orchestration role)
    - Task Decomposition Process (how to break down requests)
    - Agent Dispatch Rules (when to use which agent)
    - Autonomy Guidelines (work independently after planning phase)
    - Error Recovery (what to do when sub-agents fail)
    - Output Format (structured task plans)
  - Reference OmO's `chief-of-staff.md` for orchestration patterns

  **Must NOT do**:
  - No hardcoded model IDs — use category
  - No tool implementations — just describe available tools

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
    - Reason: Creative writing with technical accuracy needed
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 4-8)
  - **Parallel Group**: Wave 2
  - **Blocks**: Task 9
  - **Blocked By**: Tasks 1, 2

  **References**:
  - OmO: `/home/jack/everything-claude-code/agents/chief-of-staff.md` — Orchestration patterns
  - Current minimal prompt: `crates/cuttlefish-agents/src/orchestrator.rs:20-25`
  - Available tools: `crates/cuttlefish-agents/src/tools.rs` — Tool definitions

  **Acceptance Criteria**:
  - [ ] File exists: `prompts/orchestrator.md`
  - [ ] Line count ≥ 100: `wc -l prompts/orchestrator.md`
  - [ ] Valid YAML frontmatter with category = "deep"

  **QA Scenarios**:
  ```
  Scenario: Orchestrator prompt meets quality bar
    Tool: Bash
    Preconditions: prompts/orchestrator.md exists
    Steps:
      1. wc -l prompts/orchestrator.md | awk '{print $1}'
      2. Verify line count >= 100
      3. grep -c "## " prompts/orchestrator.md
      4. Verify at least 4 major sections
      5. head -5 prompts/orchestrator.md | grep "category: deep"
    Expected Result: 100+ lines, 4+ sections, correct category
    Failure Indicators: < 100 lines, missing sections, wrong category
    Evidence: .sisyphus/evidence/task-3-orchestrator-quality.txt
  ```

  **Commit**: NO (groups with Tasks 4-8)

- [ ] 4. Create Planner Prompt (planner.md)

  **What to do**:
  - Create `prompts/planner.md` (100+ lines)
  - YAML frontmatter: `name: planner`, `category: ultrabrain`, `tools: ["Read", "Glob", "Grep"]`
  - Role: Strategic planning, asks clarifying questions, creates implementation plans
  - KEY: This is the INTERACTIVE agent — asks questions before autonomous execution
  - Sections to include:
    - Identity (strategic consultant role)
    - Question-Asking Protocol (when and what to ask)
    - Plan Structure (how to format implementation plans)
    - Scope Definition (in/out boundaries)
    - Handoff Protocol (how to transition to autonomous execution)
  - Reference OmO's `planner.md` for planning patterns

  **Must NOT do**:
  - No execution — planning only
  - No skipping the questions phase

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 3, 5-8)
  - **Parallel Group**: Wave 2
  - **Blocks**: Task 9
  - **Blocked By**: Tasks 1, 2

  **References**:
  - OmO: `/home/jack/everything-claude-code/agents/planner.md` — Planning patterns
  - Prometheus system prompt patterns (for question-asking)

  **Acceptance Criteria**:
  - [ ] File exists: `prompts/planner.md`
  - [ ] Line count ≥ 100
  - [ ] Contains "question" or "clarify" (interactive nature)
  - [ ] category = "ultrabrain"

  **QA Scenarios**:
  ```
  Scenario: Planner prompt emphasizes questions phase
    Tool: Bash
    Preconditions: prompts/planner.md exists
    Steps:
      1. grep -ci "question\|clarif\|ask" prompts/planner.md
      2. Verify count >= 5 (questions are emphasized)
      3. head -5 prompts/planner.md | grep "category: ultrabrain"
    Expected Result: 5+ mentions of question-asking, correct category
    Failure Indicators: No question emphasis, wrong category
    Evidence: .sisyphus/evidence/task-4-planner-questions.txt
  ```

  **Commit**: NO (groups with Wave 2)

- [ ] 5. Create Coder Prompt (coder.md)

  **What to do**:
  - Create `prompts/coder.md` (150+ lines — most detailed)
  - YAML frontmatter: `name: coder`, `category: deep`, `tools: ["Read", "Write", "Bash", "Glob", "Grep", "EditFile"]`
  - Role: Writes code, executes commands, modifies files, runs tests
  - Sections to include:
    - Identity (senior developer role)
    - Code Quality Standards (no unwrap, use thiserror, document APIs)
    - File Modification Protocol (read before write, use EditFile for surgical changes)
    - Testing Requirements (write tests, verify they pass)
    - Error Handling (how to handle build failures)
    - Tool Usage Examples (show how to use each tool)
    - Language-Specific Guidelines (Rust conventions from CLAUDE.md)
  - Reference OmO's `code-reviewer.md` for quality checklist patterns

  **Must NOT do**:
  - No `println!` debugging
  - No global mutable state
  - No unsafe code examples

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 3-4, 6-8)
  - **Parallel Group**: Wave 2
  - **Blocks**: Task 9
  - **Blocked By**: Tasks 1, 2

  **References**:
  - OmO: `/home/jack/everything-claude-code/agents/code-reviewer.md` — Quality patterns
  - `/home/jack/Coding/Rust/cuttlefish-rs/CLAUDE.md` — Rust conventions to encode
  - Current: `crates/cuttlefish-agents/src/coder.rs:15-21` — Current minimal prompt
  - Tools: `crates/cuttlefish-agents/src/tools.rs` — Available tools

  **Acceptance Criteria**:
  - [ ] File exists: `prompts/coder.md`
  - [ ] Line count ≥ 150 (most detailed agent)
  - [ ] Contains "thiserror" or "unwrap" (Rust conventions)
  - [ ] Lists all 6 tools in frontmatter

  **QA Scenarios**:
  ```
  Scenario: Coder prompt includes Rust conventions
    Tool: Bash
    Preconditions: prompts/coder.md exists
    Steps:
      1. wc -l prompts/coder.md | awk '{print $1}'
      2. Verify >= 150 lines
      3. grep -c "unwrap\|thiserror\|unsafe" prompts/coder.md
      4. Verify Rust conventions mentioned
      5. head -10 prompts/coder.md | grep -o '"[^"]*"' | wc -l
      6. Verify 6 tools listed
    Expected Result: 150+ lines, Rust conventions present, 6 tools
    Failure Indicators: < 150 lines, no Rust conventions, missing tools
    Evidence: .sisyphus/evidence/task-5-coder-conventions.txt
  ```

  **Commit**: NO (groups with Wave 2)

- [ ] 6. Create Critic Prompt (critic.md)

  **What to do**:
  - Create `prompts/critic.md` (100+ lines)
  - YAML frontmatter: `name: critic`, `category: unspecified-high`, `tools: ["Read", "Bash", "Glob", "Grep"]`
  - Role: Reviews code changes, runs tests, approves or rejects with feedback
  - Sections to include:
    - Identity (senior code reviewer role)
    - Review Checklist (security, quality, conventions, tests)
    - Confidence-Based Filtering (only report issues >80% confident)
    - Approval/Rejection Criteria (when to approve vs reject)
    - Feedback Format (structured, actionable feedback)
    - Iteration Protocol (how to handle rejections and re-reviews)
  - Reference OmO's `code-reviewer.md` for review patterns

  **Must NOT do**:
  - No code writing — review only
  - No nitpicking style (unless violates conventions)

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 3-5, 7-8)
  - **Parallel Group**: Wave 2
  - **Blocks**: Task 9
  - **Blocked By**: Tasks 1, 2

  **References**:
  - OmO: `/home/jack/everything-claude-code/agents/code-reviewer.md:1-30` — Review checklist pattern
  - OmO: `/home/jack/everything-claude-code/agents/security-reviewer.md` — Security review patterns

  **Acceptance Criteria**:
  - [ ] File exists: `prompts/critic.md`
  - [ ] Line count ≥ 100
  - [ ] Contains "checklist" or "review" (review nature)
  - [ ] Contains confidence threshold mention

  **QA Scenarios**:
  ```
  Scenario: Critic prompt has review structure
    Tool: Bash
    Preconditions: prompts/critic.md exists
    Steps:
      1. grep -ci "checklist\|review\|approve\|reject" prompts/critic.md
      2. Verify count >= 10 (review terminology)
      3. grep -i "confidence\|80%" prompts/critic.md
      4. Verify confidence threshold mentioned
    Expected Result: Review terminology present, confidence threshold defined
    Failure Indicators: No review structure, no confidence filtering
    Evidence: .sisyphus/evidence/task-6-critic-review.txt
  ```

  **Commit**: NO (groups with Wave 2)

- [ ] 7. Create Explorer and Librarian Prompts

  **What to do**:
  - Create `prompts/explorer.md` (80+ lines)
    - YAML: `name: explorer`, `category: quick`, `tools: ["Read", "Glob", "Grep", "Bash"]`
    - Role: Searches codebases, finds patterns, maps dependencies
    - Sections: Identity, Search Strategies, Pattern Matching, Output Format
  - Create `prompts/librarian.md` (80+ lines)
    - YAML: `name: librarian`, `category: quick`, `tools: ["Read", "Bash", "WebFetch"]`
    - Role: Finds documentation, retrieves external resources, looks up APIs
    - Sections: Identity, Search Sources, Documentation Lookup, Citation Format
  - Both use `quick` category — fast, focused tasks

  **Must NOT do**:
  - No code modification for explorer
  - No code writing for librarian

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 3-6, 8)
  - **Parallel Group**: Wave 2
  - **Blocks**: Task 9
  - **Blocked By**: Tasks 1, 2

  **References**:
  - OmO: `/home/jack/everything-claude-code/agents/docs-lookup.md` — Documentation lookup patterns
  - Search patterns from explore/librarian agents in OmO

  **Acceptance Criteria**:
  - [ ] Files exist: `prompts/explorer.md`, `prompts/librarian.md`
  - [ ] Each ≥ 80 lines
  - [ ] Both have `category: quick`

  **QA Scenarios**:
  ```
  Scenario: Explorer and Librarian are quick agents
    Tool: Bash
    Preconditions: Both prompt files exist
    Steps:
      1. wc -l prompts/explorer.md prompts/librarian.md
      2. Verify each >= 80 lines
      3. head -5 prompts/explorer.md | grep "category: quick"
      4. head -5 prompts/librarian.md | grep "category: quick"
    Expected Result: Both 80+ lines, both quick category
    Failure Indicators: < 80 lines, wrong category
    Evidence: .sisyphus/evidence/task-7-quick-agents.txt
  ```

  **Commit**: NO (groups with Wave 2)

- [ ] 8. Create DevOps Prompt (devops.md)

  **What to do**:
  - Create `prompts/devops.md` (100+ lines)
  - YAML frontmatter: `name: devops`, `category: unspecified-high`, `tools: ["Bash", "Read", "Write", "Glob"]`
  - Role: Handles builds, deployments, infrastructure, CI/CD
  - Sections to include:
    - Identity (infrastructure engineer role)
    - Build Process (cargo build, test, clippy)
    - Deployment Protocol (Docker, systemd)
    - CI/CD Integration (GitHub Actions)
    - Error Recovery (build failures, deployment rollbacks)
    - Security Considerations (no secrets in code, env vars only)

  **Must NOT do**:
  - No hardcoded secrets
  - No direct production access examples

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 3-7)
  - **Parallel Group**: Wave 2
  - **Blocks**: Task 9
  - **Blocked By**: Tasks 1, 2

  **References**:
  - `/home/jack/Coding/Rust/cuttlefish-rs/install.sh` — Deployment patterns
  - `/home/jack/Coding/Rust/cuttlefish-rs/.github/workflows/` — CI patterns (if exists)

  **Acceptance Criteria**:
  - [ ] File exists: `prompts/devops.md`
  - [ ] Line count ≥ 100
  - [ ] Contains "deploy" or "build" (devops nature)
  - [ ] Contains security warning about secrets

  **QA Scenarios**:
  ```
  Scenario: DevOps prompt covers deployment
    Tool: Bash
    Preconditions: prompts/devops.md exists
    Steps:
      1. wc -l prompts/devops.md | awk '{print $1}'
      2. Verify >= 100 lines
      3. grep -ci "deploy\|build\|docker\|systemd" prompts/devops.md
      4. Verify deployment terminology present
      5. grep -i "secret\|credential\|env" prompts/devops.md
      6. Verify security mention
    Expected Result: 100+ lines, deployment covered, security mentioned
    Failure Indicators: < 100 lines, no deployment, no security
    Evidence: .sisyphus/evidence/task-8-devops.txt
  ```

  **Commit**: YES (all Wave 2)
  - Message: `feat(agents): add OmO-style detailed agent prompts`
  - Files: `prompts/*.md` (7 files)
  - Pre-commit: YAML validation for all files

- [ ] 9. Wire PromptRegistry into Agent Execution

  **What to do**:
  - Modify `crates/cuttlefish-agents/src/orchestrator.rs`:
    - Add `PromptRegistry` field to `Orchestrator` struct
    - Load prompt in `new()` or lazily on first use
    - Replace hardcoded prompt string with `registry.load("orchestrator")?.body`
  - Modify `crates/cuttlefish-agents/src/coder.rs`:
    - Same pattern: add registry, load "coder" prompt
  - Modify `crates/cuttlefish-agents/src/critic.rs`:
    - Same pattern: add registry, load "critic" prompt
  - Update `AgentRunner` to pass `PromptRegistry` to agents
  - Ensure prompt loading errors are properly propagated (not panics)
  - Add integration test that verifies prompts load from files

  **Must NOT do**:
  - No `unwrap()` on prompt loading
  - No fallback to hardcoded prompts (fail loudly if file missing)
  - No changes to model routing (already works)

  **Recommended Agent Profile**:
  - **Category**: `deep`
    - Reason: Core integration work touching multiple files
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Task 10)
  - **Parallel Group**: Wave 3
  - **Blocks**: F1-F4
  - **Blocked By**: Tasks 1, 3-8

  **References**:
  - `crates/cuttlefish-agents/src/prompt_registry.rs` — Registry API (from Task 1)
  - `crates/cuttlefish-agents/src/orchestrator.rs` — Current orchestrator structure
  - `crates/cuttlefish-agents/src/runner.rs` — Agent execution flow
  - `crates/cuttlefish-core/src/advanced.rs:157-217` — Model routing (don't change, just verify it still works)

  **Acceptance Criteria**:
  - [ ] `cargo build -p cuttlefish-agents` → SUCCESS
  - [ ] `cargo test -p cuttlefish-agents` → All tests pass
  - [ ] No `include_str!` or hardcoded prompts remain in agent files
  - [ ] Grep for old prompts returns nothing: `grep -r "You are the Orchestrator" crates/`

  **QA Scenarios**:
  ```
  Scenario: Agents load prompts from files
    Tool: Bash
    Preconditions: All prompt files exist, PromptRegistry implemented
    Steps:
      1. cargo build -p cuttlefish-agents
      2. Verify build succeeds
      3. grep -r "You are the Orchestrator agent" crates/cuttlefish-agents/src/
      4. Verify NO matches (hardcoded prompts removed)
      5. cargo test -p cuttlefish-agents integration
      6. Verify integration tests pass
    Expected Result: Build succeeds, no hardcoded prompts, tests pass
    Failure Indicators: Build fails, hardcoded prompts found, tests fail
    Evidence: .sisyphus/evidence/task-9-wiring.txt

  Scenario: Missing prompt file causes clear error
    Tool: Bash
    Preconditions: Temporarily rename a prompt file
    Steps:
      1. mv prompts/orchestrator.md prompts/orchestrator.md.bak
      2. cargo test -p cuttlefish-agents test_missing_prompt_error 2>&1
      3. Verify error message mentions "orchestrator" and "not found"
      4. mv prompts/orchestrator.md.bak prompts/orchestrator.md
    Expected Result: Clear error about missing file, no panic
    Failure Indicators: Panic, generic error, or silent fallback
    Evidence: .sisyphus/evidence/task-9-missing-error.txt
  ```

  **Commit**: YES
  - Message: `feat(agents): wire PromptRegistry into agent execution`
  - Files: `orchestrator.rs`, `coder.rs`, `critic.rs`, `runner.rs`, `lib.rs`
  - Pre-commit: `cargo test -p cuttlefish-agents && cargo clippy -p cuttlefish-agents -- -D warnings`

- [ ] 10. Expand README with Cuttlefish Analogy

  **What to do**:
  - Add new section to `README.md` after "## Philosophy" called "## The Cuttlefish Analogy"
  - Expand the cuttlefish metaphor per user's direction:
    - **Multi-colored**: Like a cuttlefish's chromatophores displaying many colors, Cuttlefish uses multiple specialized agents working in concert
    - **Adaptation**: Cuttlefish adapt to their environment instantly. This platform will eventually rewrite itself, fix its own bugs, and reboot with new versions — true self-evolution
    - **Camouflage**: Cuttlefish hide in plain sight. Cuttlefish (the platform) integrates seamlessly into existing workflows (Discord, CLI, WebUI)
    - **Intelligence**: Cuttlefish have the largest brain-to-body ratio of any invertebrate. This platform brings intelligence to coding workflows
  - Add visual ASCII art or diagram showing multi-agent flow
  - Update "## Philosophy" to reference the analogy section
  - Aim for 50+ new lines in this section

  **Must NOT do**:
  - No changes to technical documentation sections
  - No removal of existing content
  - No emojis (unless already present in README style)

  **Recommended Agent Profile**:
  - **Category**: `writing`
    - Reason: Creative documentation writing
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Task 9)
  - **Parallel Group**: Wave 3
  - **Blocks**: F1-F4
  - **Blocked By**: None (independent of code tasks)

  **References**:
  - Current README: `/home/jack/Coding/Rust/cuttlefish-rs/README.md` — Existing structure
  - User's direction: "multi-colored = multi-agent design, adaptation = self-rewriting, fixing bugs, rebooting with new versions"

  **Acceptance Criteria**:
  - [ ] New section "## The Cuttlefish Analogy" exists
  - [ ] Section contains "multi-colored", "adapt", "self" keywords
  - [ ] Section is 50+ lines
  - [ ] README still renders correctly (valid markdown)

  **QA Scenarios**:
  ```
  Scenario: README has cuttlefish analogy section
    Tool: Bash
    Preconditions: README.md updated
    Steps:
      1. grep -n "## The Cuttlefish Analogy" README.md
      2. Verify section header exists
      3. grep -ci "multi-colored\|chromatophore\|adapt\|self-rewriting\|evolv" README.md
      4. Verify >= 5 analogy keywords present
      5. sed -n '/## The Cuttlefish Analogy/,/^## /p' README.md | wc -l
      6. Verify section is 50+ lines
    Expected Result: Section exists, keywords present, 50+ lines
    Failure Indicators: Section missing, no analogy content, too short
    Evidence: .sisyphus/evidence/task-10-readme-analogy.txt
  ```

  **Commit**: YES
  - Message: `docs: expand README with cuttlefish analogy`
  - Files: `README.md`
  - Pre-commit: None (markdown only)

---

## Final Verification Wave

- [ ] F1. **Plan Compliance Audit** — `oracle`
  Read the plan end-to-end. For each "Must Have": verify implementation exists. For each "Must NOT Have": search codebase for forbidden patterns. Check evidence files exist in .sisyphus/evidence/. Compare deliverables against plan.
  Output: `Must Have [N/N] | Must NOT Have [N/N] | Tasks [N/N] | VERDICT: APPROVE/REJECT`

- [ ] F2. **Code Quality Review** — `unspecified-high`
  Run `cargo clippy --workspace -- -D warnings` + `cargo test --workspace`. Review all changed files for: `unwrap()`, empty catches, unsafe code. Check for AI slop: excessive comments, over-abstraction.
  Output: `Build [PASS/FAIL] | Clippy [PASS/FAIL] | Tests [N pass/N fail] | VERDICT`

- [ ] F3. **End-to-End Agent Test** — `unspecified-high`
  Start Cuttlefish server, submit a test request via API, verify: (1) prompts load from files, (2) different agents use different model categories, (3) execution completes without hardcoded prompts.
  Output: `Prompt Loading [PASS/FAIL] | Model Routing [PASS/FAIL] | Execution [PASS/FAIL] | VERDICT`

- [ ] F4. **Scope Fidelity Check** — `deep`
  For each task: read "What to do", verify actual changes match. Check no database schema changes, no WebUI changes, no Discord changes beyond what's specified.
  Output: `Tasks [N/N compliant] | Scope [CLEAN/N violations] | VERDICT`

---

## Commit Strategy

| Wave | Commit Message | Files |
|------|----------------|-------|
| 1 | `feat(agents): add PromptRegistry for runtime prompt loading` | prompt_registry.rs, lib.rs |
| 2 | `feat(agents): add OmO-style detailed agent prompts` | prompts/*.md |
| 3 | `feat(agents): wire PromptRegistry into agent execution` | orchestrator.rs, coder.rs, critic.rs |
| 3 | `docs: expand README with cuttlefish analogy` | README.md |

---

## Success Criteria

### Verification Commands
```bash
cargo test -p cuttlefish-agents  # Expected: all tests pass including new prompt tests
cargo clippy --workspace -- -D warnings  # Expected: no warnings
ls -la prompts/  # Expected: 7 .md files
wc -l prompts/*.md  # Expected: 700+ total lines
grep -l "^---" prompts/*.md | wc -l  # Expected: 7 (all have frontmatter)
```

### Final Checklist
- [ ] All 7 prompt files created with YAML frontmatter
- [ ] PromptRegistry loads and caches prompts
- [ ] Agents use loaded prompts (not hardcoded)
- [ ] Model categories used correctly per agent
- [ ] README contains cuttlefish analogy section
- [ ] All tests pass, clippy clean
- [ ] No unsafe code, no unwrap()
