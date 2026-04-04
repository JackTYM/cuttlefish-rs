# V1 Agent Memory System — Context, Why Command, and State Branching

## TL;DR

> **Quick Summary**: Build a persistent memory system where agents maintain project context across sessions, trace decisions back to conversations, and support forking entire project states before risky changes.
> 
> **Deliverables**:
> - Memory file system (agent-maintained project context)
> - "Why did you do this?" trace command
> - Project state branching (fork environment before risky refactors)
> - Memory search and retrieval API
> 
> **Estimated Effort**: Large (4-5 days)
> **Parallel Execution**: YES - 4 waves
> **Critical Path**: Task 1 (Memory Files) → Task 2-4 (Memory Ops) → Task 5 (Why) → Task 6 (Branching)

---

## Context

### Original Request (From Project Spec)
- Each project maintains a **memory file** the agent updates itself during work — decisions made, approaches rejected, gotchas discovered, architectural choices
- Resuming a project after days or weeks feels seamless because full context is preserved
- A **"why did you do this?"** command traces any file or decision back to the conversation that created it
- **Project state branching**: fork the entire environment state before a risky refactor — like git but includes running processes, installed packages, and config state

### Problem Statement
Currently, agents have no persistent memory beyond the current conversation. When a user returns to a project days later:
- Agent doesn't remember previous decisions
- No context on why code is structured a certain way
- No way to trace changes back to discussions
- No safe way to try risky changes without losing current state

### Design Philosophy
- **Agent memory is part of the project** — memory files live inside the project, are committed to version control, and travel with the project if exported
- Memory is structured but human-readable
- Agents update memory automatically, not manually by users

---

## Work Objectives

### Core Objective
Enable seamless project resumption with full context preservation, decision traceability, and safe state forking for experimental changes.

### Concrete Deliverables
- `crates/cuttlefish-agents/src/memory/mod.rs` — Memory system module
- `crates/cuttlefish-agents/src/memory/file.rs` — Memory file management
- `crates/cuttlefish-agents/src/memory/trace.rs` — Decision tracing
- `crates/cuttlefish-agents/src/memory/branch.rs` — State branching
- Memory file format: `.cuttlefish/memory.md`
- Decision log format: `.cuttlefish/decisions.jsonl`
- API endpoints for memory operations
- CLI commands: `cuttlefish why <file>`, `cuttlefish branch`, `cuttlefish restore`

### Definition of Done
- [ ] Memory file created and updated by agents automatically
- [ ] "Why" command traces file changes to conversations
- [ ] State branching creates complete environment snapshot
- [ ] Memory persists across sessions
- [ ] `cargo test -p cuttlefish-agents memory` passes
- [ ] `cargo clippy --workspace -- -D warnings` clean

### Must Have
- Memory file format with sections: Decisions, Gotchas, Architecture, Rejected Approaches
- Automatic memory updates on significant agent actions
- Decision log with: timestamp, file, change_type, reasoning, conversation_id
- `why <file>` returns conversation context for any file
- `branch <name>` forks entire state (container + git + memory)
- `restore <branch>` returns to forked state
- Memory searchable by keyword

### Must NOT Have (Guardrails)
- No unsafe code
- No `unwrap()` — use `?` or `expect("reason")`
- No storing sensitive data in memory (API keys, passwords)
- No memory files larger than 1MB (summarize if needed)
- No breaking existing agent functionality

---

## Verification Strategy

> **ZERO HUMAN INTERVENTION** — ALL verification is agent-executed.

### Test Decision
- **Infrastructure exists**: YES (agent system)
- **Automated tests**: YES (TDD)
- **Framework**: `#[tokio::test]` for async

### QA Policy
Every task includes agent-executed QA scenarios.
Evidence saved to `.sisyphus/evidence/task-{N}-{scenario-slug}.{ext}`.

---

## Execution Strategy

### Parallel Execution Waves

```
Wave 1 (Foundation — memory file system):
├── Task 1: Memory file format and management [deep]
├── Task 2: Decision log structure [unspecified-high]
└── Task 3: Memory file auto-update hooks [deep]

Wave 2 (Trace — why command):
├── Task 4: Decision indexing [unspecified-high]
├── Task 5: Why command implementation [deep]
└── Task 6: Conversation context retrieval [unspecified-high]

Wave 3 (Branch — state forking):
├── Task 7: State branching architecture [deep]
├── Task 8: Branch creation (fork) [deep]
├── Task 9: Branch restoration [deep]
└── Task 10: Branch management (list, delete) [quick]

Wave 4 (Integration):
├── Task 11: Memory API endpoints [unspecified-high]
├── Task 12: CLI commands [quick]
└── Task 13: Agent prompt updates for memory usage [unspecified-high]

Wave FINAL (Verification):
├── Task F1: Plan compliance audit (oracle)
├── Task F2: Code quality review (unspecified-high)
├── Task F3: Memory E2E QA (unspecified-high)
└── Task F4: Scope fidelity check (deep)
-> Present results -> Get explicit user okay

Critical Path: Task 1 → Task 3 → Task 5 → Task 7 → Task 8 → Task 11 → F1-F4 → user okay
Parallel Speedup: ~55% faster than sequential
Max Concurrent: 4 (Wave 3)
```

### Dependency Matrix

| Task | Depends On | Blocks | Wave |
|------|------------|--------|------|
| 1 | — | 2, 3, 4, 11 | 1 |
| 2 | 1 | 4, 5 | 1 |
| 3 | 1 | 4, 5, 13 | 1 |
| 4 | 1, 2, 3 | 5, 6 | 2 |
| 5 | 2, 4 | 11, 12 | 2 |
| 6 | 4 | 5 | 2 |
| 7 | — | 8, 9 | 3 |
| 8 | 7 | 9, 10 | 3 |
| 9 | 7, 8 | 10 | 3 |
| 10 | 8, 9 | 11 | 3 |
| 11 | 1, 5, 10 | F1-F4 | 4 |
| 12 | 5, 10 | F1-F4 | 4 |
| 13 | 3 | F1-F4 | 4 |

### Agent Dispatch Summary

- **Wave 1**: 3 tasks — T1 → `deep`, T2 → `unspecified-high`, T3 → `deep`
- **Wave 2**: 3 tasks — T4 → `unspecified-high`, T5 → `deep`, T6 → `unspecified-high`
- **Wave 3**: 4 tasks — T7-T9 → `deep`, T10 → `quick`
- **Wave 4**: 3 tasks — T11 → `unspecified-high`, T12 → `quick`, T13 → `unspecified-high`
- **FINAL**: 4 tasks — F1 → `oracle`, F2-F3 → `unspecified-high`, F4 → `deep`

---

## TODOs

- [ ] 1. Memory File Format and Management

  **What to do**:
  - Create `crates/cuttlefish-agents/src/memory/mod.rs` and `file.rs`
  - Define memory file structure:
    ```markdown
    # Project Memory: {project_name}
    
    ## Summary
    > One-paragraph project description and current state
    
    ## Key Decisions
    - **{date}**: {decision} — {rationale}
    
    ## Architecture
    - {component}: {description}
    
    ## Gotchas & Lessons
    - {gotcha}: {context}
    
    ## Rejected Approaches
    - {approach}: {why rejected}
    
    ## Active Context
    - Currently working on: {task}
    - Blockers: {blockers}
    - Next steps: {steps}
    ```
  - Implement `ProjectMemory` struct with parse/serialize
  - File location: `{project_root}/.cuttlefish/memory.md`
  - Create if not exists on project access
  - Memory file is gitignored by default (optional tracking)

  **Must NOT do**:
  - Don't store code in memory (just references)
  - Don't exceed 1MB (auto-summarize old entries)

  **Recommended Agent Profile**:
  - **Category**: `deep`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (Wave 1 start)
  - **Blocks**: Tasks 2, 3, 4, 11
  - **Blocked By**: None

  **References**:
  - OpenClaw memory patterns
  - OmO session persistence

  **Acceptance Criteria**:
  - [ ] Memory file created on project init
  - [ ] Parse/serialize round-trips correctly
  - [ ] Sections are extractable individually

  **QA Scenarios**:
  ```
  Scenario: Memory file round-trip
    Tool: Bash (cargo test)
    Steps:
      1. Create ProjectMemory with all sections
      2. Serialize to markdown
      3. Parse back
      4. Verify all sections match
    Expected Result: Lossless round-trip
    Evidence: .sisyphus/evidence/task-1-roundtrip.txt
  ```

  **Commit**: YES
  - Message: `feat(agents): add memory file format and management`
  - Files: `memory/mod.rs`, `memory/file.rs`

- [ ] 2. Decision Log Structure

  **What to do**:
  - Create `crates/cuttlefish-agents/src/memory/log.rs`
  - Define decision log entry:
    ```rust
    pub struct DecisionEntry {
        pub id: Uuid,
        pub timestamp: DateTime<Utc>,
        pub conversation_id: String,
        pub message_id: String,
        pub file_path: Option<String>,
        pub change_type: ChangeType,  // Create, Modify, Delete, Decide
        pub summary: String,
        pub reasoning: String,
        pub agent: String,
        pub confidence: f32,
    }
    ```
  - Store in `.cuttlefish/decisions.jsonl` (append-only)
  - Index by: file_path, conversation_id, timestamp
  - Keep last 1000 entries (rotate old ones to archive)

  **Must NOT do**:
  - Don't store full file contents (just paths)
  - Don't block on log writes (async append)

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 1, 3)
  - **Blocks**: Tasks 4, 5
  - **Blocked By**: Task 1

  **Acceptance Criteria**:
  - [ ] Log entries append correctly
  - [ ] Indexing by file works
  - [ ] Rotation at 1000 entries

  **QA Scenarios**:
  ```
  Scenario: Log file rotation
    Tool: Bash (cargo test)
    Steps:
      1. Append 1001 entries
      2. Verify log has 1000 entries
      3. Verify oldest entry archived
    Expected Result: Rotation works
    Evidence: .sisyphus/evidence/task-2-rotation.txt
  ```

  **Commit**: NO (groups with Wave 1)

- [ ] 3. Memory File Auto-Update Hooks

  **What to do**:
  - Create hooks in agent execution pipeline:
    - After file creation: log + update Architecture if new component
    - After significant decision: log + update Key Decisions
    - After error/workaround: update Gotchas
    - After rejecting approach: update Rejected Approaches
    - After task completion: update Active Context
  - Define "significant" triggers:
    - New file in src/
    - Architectural change (new module, dependency)
    - Bug fix with workaround
    - Explicit agent reasoning about trade-offs
  - Hooks run async (don't block main execution)
  - Rate limit: max 1 memory update per minute

  **Must NOT do**:
  - Don't log every small change (noise)
  - Don't block agent on memory writes

  **Recommended Agent Profile**:
  - **Category**: `deep`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 1, 2)
  - **Blocks**: Tasks 4, 5, 13
  - **Blocked By**: Task 1

  **Acceptance Criteria**:
  - [ ] File creation triggers log
  - [ ] Memory updated on significant actions
  - [ ] Rate limiting works

  **QA Scenarios**:
  ```
  Scenario: Auto-update on file creation
    Tool: Bash (cargo test)
    Steps:
      1. Simulate agent creating src/new_module.rs
      2. Check decision log has entry
      3. Check memory Architecture section updated
    Expected Result: Auto-update triggered
    Evidence: .sisyphus/evidence/task-3-autoupdate.txt
  ```

  **Commit**: YES (Wave 1)
  - Message: `feat(agents): add memory auto-update hooks`
  - Files: `memory/hooks.rs`, agent files

- [ ] 4. Decision Indexing

  **What to do**:
  - Create `crates/cuttlefish-agents/src/memory/index.rs`
  - Build in-memory index on load:
    - `file_index: HashMap<String, Vec<DecisionEntry>>`
    - `conversation_index: HashMap<String, Vec<DecisionEntry>>`
    - `time_index: BTreeMap<DateTime, DecisionEntry>`
  - Query methods:
    - `find_by_file(path: &str) -> Vec<&DecisionEntry>`
    - `find_by_conversation(id: &str) -> Vec<&DecisionEntry>`
    - `find_in_range(start: DateTime, end: DateTime) -> Vec<&DecisionEntry>`
    - `search(query: &str) -> Vec<&DecisionEntry>` (fuzzy match on summary/reasoning)
  - Rebuild index on startup, update incrementally on new entries

  **Must NOT do**:
  - Don't use external search engine (keep simple)

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (after Wave 1)
  - **Blocks**: Tasks 5, 6
  - **Blocked By**: Tasks 1, 2, 3

  **Acceptance Criteria**:
  - [ ] Index built correctly
  - [ ] All query methods work
  - [ ] Fuzzy search finds relevant entries

  **QA Scenarios**:
  ```
  Scenario: Query by file path
    Tool: Bash (cargo test)
    Steps:
      1. Add entries for files A, B, C
      2. Query for file B
      3. Verify only B's entries returned
    Expected Result: Correct entries
    Evidence: .sisyphus/evidence/task-4-query.txt
  ```

  **Commit**: NO (groups with Wave 2)

- [ ] 5. Why Command Implementation

  **What to do**:
  - Implement `why` function:
    ```rust
    pub async fn why(
        project_id: &str,
        target: WhyTarget,  // File, Decision, Line
    ) -> Result<WhyExplanation>
    
    pub struct WhyExplanation {
        pub target: String,
        pub decisions: Vec<DecisionEntry>,
        pub conversation_excerpts: Vec<ConversationExcerpt>,
        pub summary: String,
    }
    ```
  - For file: find all decisions affecting that file
  - For specific line: use git blame + decision log correlation
  - Include conversation excerpts that led to the decision
  - Generate human-readable summary

  **Must NOT do**:
  - Don't return raw JSON (format nicely)
  - Don't include full conversations (excerpts only)

  **Recommended Agent Profile**:
  - **Category**: `deep`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Task 6)
  - **Blocks**: Tasks 11, 12
  - **Blocked By**: Tasks 2, 4

  **References**:
  - Git blame integration
  - Conversation history storage

  **Acceptance Criteria**:
  - [ ] Returns relevant decisions
  - [ ] Conversation excerpts included
  - [ ] Summary is human-readable

  **QA Scenarios**:
  ```
  Scenario: Why command for file
    Tool: Bash (cargo test)
    Steps:
      1. Create file with logged decision
      2. Run why("src/file.rs")
      3. Verify decision returned with context
    Expected Result: Explanation complete
    Evidence: .sisyphus/evidence/task-5-why.txt
  ```

  **Commit**: YES (Wave 2)
  - Message: `feat(agents): add decision indexing and why command`
  - Files: `memory/index.rs`, `memory/why.rs`

- [ ] 6. Conversation Context Retrieval

  **What to do**:
  - Implement conversation excerpt retrieval:
    ```rust
    pub async fn get_conversation_excerpt(
        conversation_id: &str,
        message_id: &str,
        context_messages: usize,  // before and after
    ) -> Result<ConversationExcerpt>
    ```
  - Fetch from conversation storage (database)
  - Include N messages before and after the decision point
  - Redact any sensitive content (API keys, passwords)
  - Format for display (markdown-friendly)

  **Must NOT do**:
  - Don't return full conversation (excerpts only)
  - Don't include system prompts

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Task 5)
  - **Blocks**: Task 5 completion
  - **Blocked By**: Task 4

  **Acceptance Criteria**:
  - [ ] Excerpts retrieved correctly
  - [ ] Sensitive content redacted
  - [ ] Context messages included

  **QA Scenarios**:
  ```
  Scenario: Retrieve conversation context
    Tool: Bash (cargo test)
    Steps:
      1. Store conversation with 10 messages
      2. Request excerpt for message 5 with context 2
      3. Verify messages 3-7 returned
    Expected Result: Context included
    Evidence: .sisyphus/evidence/task-6-context.txt
  ```

  **Commit**: NO (groups with Wave 2)

- [ ] 7. State Branching Architecture

  **What to do**:
  - Create `crates/cuttlefish-agents/src/memory/branch.rs`
  - Define state branch:
    ```rust
    pub struct StateBranch {
        pub id: BranchId,
        pub name: String,
        pub project_id: String,
        pub created_at: DateTime<Utc>,
        pub description: Option<String>,
        pub git_ref: String,           // Git branch/commit
        pub container_snapshot: String, // Docker snapshot ID
        pub memory_snapshot: String,    // Memory file backup
    }
    ```
  - State branch includes:
    - Git state (branch or stash)
    - Container snapshot (from sandbox)
    - Memory file copy
    - Any running process state (if possible)
  - Store branch metadata in database

  **Must NOT do**:
  - Don't branch without user confirmation (risky)
  - Don't auto-delete branches

  **Recommended Agent Profile**:
  - **Category**: `deep`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (independent foundation)
  - **Blocks**: Tasks 8, 9
  - **Blocked By**: None

  **References**:
  - Docker snapshot system (v1-sandbox.md)
  - Git stash/branch

  **Acceptance Criteria**:
  - [ ] Branch struct captures all state
  - [ ] Database storage works
  - [ ] Metadata queryable

  **QA Scenarios**:
  ```
  Scenario: Branch metadata storage
    Tool: Bash (cargo test)
    Steps:
      1. Create StateBranch
      2. Store in database
      3. Retrieve and verify all fields
    Expected Result: Metadata persists
    Evidence: .sisyphus/evidence/task-7-branch-store.txt
  ```

  **Commit**: NO (groups with Wave 3)

- [ ] 8. Branch Creation (Fork)

  **What to do**:
  - Implement branch creation:
    ```rust
    pub async fn create_branch(
        project_id: &str,
        name: &str,
        description: Option<&str>,
    ) -> Result<StateBranch>
    ```
  - Steps:
    1. Create git branch from current HEAD
    2. Create container snapshot (use sandbox snapshot)
    3. Copy memory file to `.cuttlefish/branches/{name}/memory.md`
    4. Copy decision log
    5. Store branch metadata
    6. Return to original state (don't switch)
  - Naming: `cuttlefish-branch-{name}`
  - Limit: 10 branches per project

  **Must NOT do**:
  - Don't switch to branch after creation
  - Don't modify working state

  **Recommended Agent Profile**:
  - **Category**: `deep`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Task 9 partially)
  - **Blocks**: Tasks 9, 10
  - **Blocked By**: Task 7

  **References**:
  - Sandbox snapshot (v1-sandbox.md Task 7)
  - Git branch commands

  **Acceptance Criteria**:
  - [ ] Git branch created
  - [ ] Container snapshot created
  - [ ] Memory backed up
  - [ ] Can list branches

  **QA Scenarios**:
  ```
  Scenario: Create state branch
    Tool: Bash (cargo test)
    Steps:
      1. Make changes to project
      2. Create branch "pre-refactor"
      3. Verify git branch exists
      4. Verify snapshot exists
      5. Verify memory copied
    Expected Result: Full state captured
    Evidence: .sisyphus/evidence/task-8-create.txt
  ```

  **Commit**: NO (groups with Wave 3)

- [ ] 9. Branch Restoration

  **What to do**:
  - Implement branch restoration:
    ```rust
    pub async fn restore_branch(
        project_id: &str,
        branch_name: &str,
        create_backup: bool,  // Auto-branch current state first
    ) -> Result<()>
    ```
  - Steps:
    1. If create_backup, branch current state first
    2. Stop current container
    3. Restore container from snapshot
    4. Git checkout the branch (or reset to commit)
    5. Restore memory file
    6. Start container
  - Handle conflicts (uncommitted changes)

  **Must NOT do**:
  - Don't lose uncommitted work (warn or auto-backup)
  - Don't delete the restored branch

  **Recommended Agent Profile**:
  - **Category**: `deep`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: NO (depends on Task 8)
  - **Blocks**: Task 10
  - **Blocked By**: Tasks 7, 8

  **References**:
  - Sandbox restore (v1-sandbox.md Task 8)

  **Acceptance Criteria**:
  - [ ] Container restored
  - [ ] Git state restored
  - [ ] Memory restored
  - [ ] Uncommitted work warning

  **QA Scenarios**:
  ```
  Scenario: Restore to previous state
    Tool: Bash (cargo test)
    Steps:
      1. Create branch
      2. Make more changes
      3. Restore branch
      4. Verify original state restored
    Expected Result: State matches branch
    Evidence: .sisyphus/evidence/task-9-restore.txt
  ```

  **Commit**: NO (groups with Wave 3)

- [ ] 10. Branch Management

  **What to do**:
  - Implement branch management:
    ```rust
    pub async fn list_branches(project_id: &str) -> Result<Vec<StateBranch>>
    pub async fn delete_branch(project_id: &str, branch_name: &str) -> Result<()>
    pub async fn get_branch(project_id: &str, branch_name: &str) -> Result<StateBranch>
    pub async fn compare_branches(branch_a: &str, branch_b: &str) -> Result<BranchDiff>
    ```
  - Delete removes: git branch, snapshot, memory backup, metadata
  - Compare shows: git diff, file changes, memory diff
  - Enforce branch limit (delete oldest on overflow, with warning)

  **Must NOT do**:
  - Don't delete without confirmation
  - Don't compare running state vs branch (only branch vs branch)

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: NO (depends on Tasks 8, 9)
  - **Blocks**: Task 11
  - **Blocked By**: Tasks 8, 9

  **Acceptance Criteria**:
  - [ ] List shows all branches
  - [ ] Delete cleans up everything
  - [ ] Compare shows meaningful diff

  **QA Scenarios**:
  ```
  Scenario: Delete branch cleans up
    Tool: Bash (cargo test)
    Steps:
      1. Create branch
      2. Delete branch
      3. Verify git branch gone
      4. Verify snapshot gone
      5. Verify metadata gone
    Expected Result: Complete cleanup
    Evidence: .sisyphus/evidence/task-10-delete.txt
  ```

  **Commit**: YES (Wave 3)
  - Message: `feat(agents): add state branching (fork/restore)`
  - Files: `memory/branch.rs`

- [x] 11. Memory API Endpoints

  **What to do**:
  - Add to `crates/cuttlefish-api/src/api_routes.rs`:
    - `GET /api/projects/:id/memory` — Get memory file content
    - `GET /api/projects/:id/memory/search?q=` — Search memory
    - `GET /api/projects/:id/why/:path` — Why command for file
    - `GET /api/projects/:id/branches` — List state branches
    - `POST /api/projects/:id/branches` — Create branch
    - `POST /api/projects/:id/branches/:name/restore` — Restore branch
    - `DELETE /api/projects/:id/branches/:name` — Delete branch
  - Proper request/response types
  - Error handling with clear messages

  **Must NOT do**:
  - Don't expose internal IDs
  - Don't allow memory write via API (agent-only)

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 12, 13)
  - **Blocks**: F1-F4
  - **Blocked By**: Tasks 1, 5, 10

  **Acceptance Criteria**:
  - [ ] All endpoints work
  - [ ] Auth required
  - [ ] Errors user-friendly

  **QA Scenarios**:
  ```
  Scenario: Memory search via API
    Tool: Bash (curl)
    Steps:
      1. Add memory entries
      2. GET /api/projects/test/memory/search?q=architecture
      3. Verify matching entries returned
    Expected Result: Search works
    Evidence: .sisyphus/evidence/task-11-api-search.txt
  ```

  **Commit**: YES
  - Message: `feat(api): add memory and branching endpoints`
  - Files: `api_routes.rs`

- [x] 12. CLI Commands

  **What to do**:
  - Add CLI commands to main binary:
    - `cuttlefish why <file>` — Explain why file exists/changed
    - `cuttlefish why <file>:<line>` — Explain specific line
    - `cuttlefish memory` — Show current memory summary
    - `cuttlefish memory search <query>` — Search memory
    - `cuttlefish branch <name>` — Create state branch
    - `cuttlefish branch list` — List branches
    - `cuttlefish branch restore <name>` — Restore branch
    - `cuttlefish branch delete <name>` — Delete branch
  - Human-readable output with colors
  - `--json` flag for machine output

  **Must NOT do**:
  - Don't require server running for read operations (local file access)

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 11, 13)
  - **Blocks**: F1-F4
  - **Blocked By**: Tasks 5, 10

  **Acceptance Criteria**:
  - [ ] All commands work
  - [ ] Output is readable
  - [ ] JSON flag works

  **QA Scenarios**:
  ```
  Scenario: Why command via CLI
    Tool: Bash
    Steps:
      1. cuttlefish why src/main.rs
      2. Verify output shows decisions
    Expected Result: Explanation displayed
    Evidence: .sisyphus/evidence/task-12-cli-why.txt
  ```

  **Commit**: NO (groups with Wave 4)

- [x] 13. Agent Prompt Updates for Memory Usage

  **What to do**:
  - Update all agent prompts to use memory:
    - On session start: read memory file for context
    - Include "Active Context" in system prompt
    - Reference "Rejected Approaches" before suggesting solutions
    - Update memory after significant decisions
  - Add memory instructions to prompts:
    - When to update memory
    - What to put in each section
    - How to reference previous decisions
  - Agents should proactively mention remembered context

  **Must NOT do**:
  - Don't override user requests with memory (user > memory)
  - Don't spam memory updates

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 11, 12)
  - **Blocks**: F1-F4
  - **Blocked By**: Task 3

  **References**:
  - `prompts/*.md` — Existing prompts

  **Acceptance Criteria**:
  - [ ] Prompts reference memory
  - [ ] Agents read memory on start
  - [ ] Memory updated on decisions

  **QA Scenarios**:
  ```
  Scenario: Agent uses memory context
    Tool: Bash (integration test)
    Steps:
      1. Add "Rejected: MySQL for performance" to memory
      2. Ask agent about database choice
      3. Verify agent mentions the rejection
    Expected Result: Memory informs response
    Evidence: .sisyphus/evidence/task-13-agent-memory.txt
  ```

  **Commit**: YES (Wave 4)
  - Message: `feat(agents): integrate memory into agent prompts`
  - Files: `prompts/*.md`, agent execution code

---

## Final Verification Wave

- [ ] F1. **Plan Compliance Audit** — `oracle`
  Verify memory file format, why command, branching all implemented. Check no sensitive data storage.
  Output: `Must Have [N/N] | Must NOT Have [N/N] | Tasks [N/N] | VERDICT: APPROVE/REJECT`

- [ ] F2. **Code Quality Review** — `unspecified-high`
  Run `cargo clippy --workspace -- -D warnings` + tests. Review for unwrap(), sensitive data handling.
  Output: `Build [PASS/FAIL] | Lint [PASS/FAIL] | Tests [N pass/N fail] | VERDICT`

- [ ] F3. **Memory E2E QA** — `unspecified-high`
  Full workflow: create project, agent makes decisions, memory updated, why command works, branch and restore.
  Output: `Memory [PASS/FAIL] | Why [PASS/FAIL] | Branch [PASS/FAIL] | Restore [PASS/FAIL] | VERDICT`

- [ ] F4. **Scope Fidelity Check** — `deep`
  Verify no API key storage, memory under 1MB, branches capped at 10.
  Output: `Tasks [N/N compliant] | Scope [CLEAN/N violations] | VERDICT`

---

## Commit Strategy

| Wave | Commit |
|------|--------|
| 1 | `feat(agents): add memory file format and management` |
| 1 | `feat(agents): add memory auto-update hooks` |
| 2 | `feat(agents): add decision indexing and why command` |
| 3 | `feat(agents): add state branching (fork/restore)` |
| 4 | `feat(api): add memory and branching endpoints` |
| 4 | `feat(agents): integrate memory into agent prompts` |

---

## Success Criteria

### Verification Commands
```bash
cargo test -p cuttlefish-agents memory  # All tests pass
cargo clippy --workspace -- -D warnings  # Clean
cuttlefish why src/main.rs  # Returns explanation
cuttlefish branch test && cuttlefish branch restore test  # Works
```

### Final Checklist
- [ ] Memory file created and maintained by agents
- [ ] Why command traces decisions to conversations
- [ ] State branching captures container + git + memory
- [ ] Branch restore works completely
- [ ] No sensitive data in memory
- [ ] Memory under 1MB with auto-summarization
