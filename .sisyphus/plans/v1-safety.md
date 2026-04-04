# V1 Safety System — Confidence Gates, Diff Previews & Checkpoint Rollback

## TL;DR

> **Quick Summary**: Build a comprehensive safety system with confidence-based action gating, human-readable diff previews before file writes, and full checkpoint rollback that includes container state, git history, and memory files.
> 
> **Deliverables**:
> - Confidence scoring for all agent actions
> - Configurable approval thresholds (auto/prompt/block)
> - Rich diff preview with syntax highlighting
> - Checkpoint system (snapshot container + git + memory)
> - One-click rollback to any checkpoint
> - Undo last N operations
> 
> **Estimated Effort**: Large (4-5 days)
> **Parallel Execution**: YES - 4 waves
> **Critical Path**: Task 1 (Confidence) → Task 2-3 (Gates) → Task 4-6 (Checkpoints) → Task 7-8 (UI/CLI)

---

## Context

### Original Request (From Product Spec)
- **Confidence Gates**: Agent asks before doing something it's unsure about (model reports confidence, user sets threshold)
- **Diff Preview**: User can see exactly what will change before any file is written
- **Checkpoint Rollback**: Take a snapshot before risky operations — rollback includes container state, installed packages, config state

### Problem Statement
AI agents can make mistakes or take unexpected actions. Users need:
- Visibility into what agents are about to do
- Control over which actions require approval
- Ability to undo changes, including environmental state
- Peace of mind when delegating work to agents

### Design Philosophy
- **Default safe**: Low-confidence actions require approval by default
- **Configurable**: Power users can reduce friction; cautious users can increase scrutiny
- **Complete rollback**: Checkpoints capture everything (files + container + memory)
- **Non-blocking**: High-confidence actions proceed automatically

---

## Work Objectives

### Core Objective
Enable safe agent operation through confidence-based gating, transparent diff previews, and complete state rollback capabilities.

### Concrete Deliverables
- `crates/cuttlefish-agents/src/safety/mod.rs` — Safety module root
- `crates/cuttlefish-agents/src/safety/confidence.rs` — Confidence scoring
- `crates/cuttlefish-agents/src/safety/gates.rs` — Approval gates
- `crates/cuttlefish-agents/src/safety/diff.rs` — Diff generation
- `crates/cuttlefish-agents/src/safety/checkpoint.rs` — Checkpoint management
- `crates/cuttlefish-api/src/safety_routes.rs` — Safety API endpoints
- WebUI diff preview modal and approval workflow
- CLI commands: `cuttlefish checkpoint`, `cuttlefish rollback`, `cuttlefish undo`

### Definition of Done
- [ ] Agent actions include confidence scores
- [ ] Configurable thresholds gate actions appropriately
- [ ] Diff preview shown before file writes (when required)
- [ ] Checkpoints capture container + git + memory state
- [ ] Rollback restores complete state
- [ ] `cargo test -p cuttlefish-agents safety` passes
- [ ] `cargo clippy --workspace -- -D warnings` clean

### Must Have
- Confidence scoring (0.0-1.0) on file writes, bash commands, git operations
- Three threshold levels: auto-approve, prompt-user, block
- Diff preview with line-by-line changes (unified diff format)
- Syntax highlighting in diffs (basic, language-aware)
- Checkpoint creation (manual and automatic before risky ops)
- Checkpoint includes: git state, container snapshot, memory file
- Rollback to any checkpoint
- Undo last N file operations

### Must NOT Have (Guardrails)
- No unsafe code
- No `unwrap()` — use `?` or `expect("reason")`
- No storing passwords in checkpoint (sanitize)
- No infinite checkpoint storage (cap at 20, rotate oldest)
- No blocking on every action (respect confidence thresholds)
- No breaking existing agent execution flow

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
Wave 1 (Foundation — confidence + gates):
├── Task 1: Confidence scoring system [deep]
├── Task 2: Action gates with thresholds [deep]
└── Task 3: Diff generation engine [unspecified-high]

Wave 2 (Checkpoints — state capture):
├── Task 4: Checkpoint architecture [deep]
├── Task 5: Checkpoint creation [deep]
└── Task 6: Checkpoint restoration (rollback) [deep]

Wave 3 (Integration — API + CLI):
├── Task 7: Safety API endpoints [unspecified-high]
├── Task 8: CLI safety commands [quick]
└── Task 9: Wire gates into agent execution [deep]

Wave 4 (UI — diff preview modal):
└── Task 10: WebUI diff preview and approval [visual-engineering]

Wave FINAL (Verification):
├── Task F1: Plan compliance audit (oracle)
├── Task F2: Code quality review (unspecified-high)
├── Task F3: Safety E2E QA (unspecified-high)
└── Task F4: Scope fidelity check (deep)
-> Present results -> Get explicit user okay

Critical Path: Task 1 → Task 2 → Task 9 → Task 4 → Task 5 → Task 6 → F1-F4 → user okay
Parallel Speedup: ~55% faster than sequential
Max Concurrent: 3 (Waves 1 & 2)
```

### Dependency Matrix

| Task | Depends On | Blocks | Wave |
|------|------------|--------|------|
| 1 | — | 2, 9 | 1 |
| 2 | 1 | 9, 7 | 1 |
| 3 | — | 7, 10 | 1 |
| 4 | — | 5, 6 | 2 |
| 5 | 4 | 6, 7 | 2 |
| 6 | 4, 5 | 7, 8 | 2 |
| 7 | 2, 3, 5, 6 | 10 | 3 |
| 8 | 6 | F1-F4 | 3 |
| 9 | 1, 2 | F1-F4 | 3 |
| 10 | 3, 7 | F1-F4 | 4 |

### Agent Dispatch Summary

- **Wave 1**: 3 tasks — T1-T2 → `deep`, T3 → `unspecified-high`
- **Wave 2**: 3 tasks — T4-T6 → `deep`
- **Wave 3**: 3 tasks — T7 → `unspecified-high`, T8 → `quick`, T9 → `deep`
- **Wave 4**: 1 task — T10 → `visual-engineering`
- **FINAL**: 4 tasks — F1 → `oracle`, F2-F3 → `unspecified-high`, F4 → `deep`

---

## TODOs

- [ ] 1. Confidence Scoring System

  **What to do**:
  - Create `crates/cuttlefish-agents/src/safety/mod.rs` and `confidence.rs`
  - Define confidence score structure:
    ```rust
    pub struct ConfidenceScore {
        pub value: f32,           // 0.0 to 1.0
        pub reasoning: String,    // Why this score
        pub action_type: ActionType,
        pub risk_factors: Vec<RiskFactor>,
    }
    
    pub enum ActionType {
        FileWrite,
        FileDelete,
        BashCommand,
        GitOperation,
        PackageInstall,
        ConfigChange,
    }
    
    pub enum RiskFactor {
        ModifiesExisting,     // vs creating new
        AffectsMultipleFiles,
        SystemCommand,        // sudo, rm -rf, etc.
        IrreversibleGit,      // force push, reset --hard
        HighImpactPath,       // src/, config, etc.
    }
    ```
  - Implement `calculate_confidence(action: &AgentAction) -> ConfidenceScore`
  - Risk factor detection via pattern matching
  - Export from mod.rs

  **Must NOT do**:
  - Don't use ML models (simple heuristics only)
  - Don't block on confidence calculation (must be fast)

  **Recommended Agent Profile**:
  - **Category**: `deep`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (Wave 1 start)
  - **Blocks**: Tasks 2, 9
  - **Blocked By**: None

  **References**:
  - Agent action types in `crates/cuttlefish-agents/src/tools.rs`

  **Acceptance Criteria**:
  - [ ] Confidence scores calculated for all action types
  - [ ] Risk factors detected correctly
  - [ ] Scores in valid range (0.0-1.0)

  **QA Scenarios**:
  ```
  Scenario: File write confidence scoring
    Tool: Bash (cargo test)
    Steps:
      1. Calculate confidence for new file creation
      2. Calculate confidence for modifying src/main.rs
      3. Verify new file > existing file (higher confidence)
    Expected Result: New files have higher confidence
    Evidence: .sisyphus/evidence/task-1-confidence.txt
  ```

  **Commit**: YES
  - Message: `feat(agents): add confidence scoring system`
  - Files: `safety/mod.rs`, `safety/confidence.rs`

- [ ] 2. Action Gates with Thresholds

  **What to do**:
  - Create `crates/cuttlefish-agents/src/safety/gates.rs`
  - Define gate configuration:
    ```rust
    pub struct GateConfig {
        pub auto_approve_threshold: f32,   // Default: 0.9
        pub prompt_threshold: f32,         // Default: 0.5
        // Below prompt_threshold = block
    }
    
    pub enum GateDecision {
        AutoApprove,
        PromptUser { action: AgentAction, preview: ActionPreview },
        Block { reason: String },
    }
    ```
  - Implement `evaluate_action(action: &AgentAction, config: &GateConfig) -> GateDecision`
  - Per-action-type threshold overrides (e.g., always prompt for git push)
  - Config loaded from project settings

  **Must NOT do**:
  - Don't hardcode thresholds (configurable)
  - Don't block without reason

  **Recommended Agent Profile**:
  - **Category**: `deep`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 1, 3)
  - **Blocks**: Tasks 7, 9
  - **Blocked By**: Task 1

  **Acceptance Criteria**:
  - [ ] Gates evaluate correctly based on thresholds
  - [ ] Per-action overrides work
  - [ ] Decisions include context

  **QA Scenarios**:
  ```
  Scenario: Gate evaluation with thresholds
    Tool: Bash (cargo test)
    Steps:
      1. Action with confidence 0.95 → AutoApprove
      2. Action with confidence 0.7 → PromptUser
      3. Action with confidence 0.3 → Block
    Expected Result: Correct decisions at each threshold
    Evidence: .sisyphus/evidence/task-2-gates.txt
  ```

  **Commit**: NO (groups with Wave 1)

- [ ] 3. Diff Generation Engine

  **What to do**:
  - Create `crates/cuttlefish-agents/src/safety/diff.rs`
  - Implement diff generation:
    ```rust
    pub struct FileDiff {
        pub path: String,
        pub old_content: Option<String>,
        pub new_content: String,
        pub hunks: Vec<DiffHunk>,
        pub stats: DiffStats,
    }
    
    pub struct DiffHunk {
        pub old_start: usize,
        pub old_lines: usize,
        pub new_start: usize,
        pub new_lines: usize,
        pub lines: Vec<DiffLine>,
    }
    
    pub struct DiffLine {
        pub change_type: ChangeType,  // Add, Remove, Context
        pub content: String,
        pub line_number: Option<usize>,
    }
    ```
  - Use `similar` crate for diff algorithm
  - Generate unified diff format
  - Add `DiffStats` (lines added, removed, changed)
  - Support binary file detection (show "Binary file changed")

  **Must NOT do**:
  - Don't load huge files into memory (stream for large files)
  - Don't diff if file > 1MB (warn instead)

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 1, 2)
  - **Blocks**: Tasks 7, 10
  - **Blocked By**: None

  **References**:
  - `similar` crate: https://docs.rs/similar

  **Acceptance Criteria**:
  - [ ] Diffs generate correctly for text files
  - [ ] Binary files handled gracefully
  - [ ] Stats accurate

  **QA Scenarios**:
  ```
  Scenario: Generate diff for file modification
    Tool: Bash (cargo test)
    Steps:
      1. Create diff between "hello\nworld" and "hello\nrust\nworld"
      2. Verify hunk shows +rust insertion
      3. Verify stats: 1 added, 0 removed
    Expected Result: Correct diff output
    Evidence: .sisyphus/evidence/task-3-diff.txt
  ```

  **Commit**: YES (Wave 1)
  - Message: `feat(agents): add confidence gates and diff generation`
  - Files: `safety/gates.rs`, `safety/diff.rs`

- [ ] 4. Checkpoint Architecture

  **What to do**:
  - Create `crates/cuttlefish-agents/src/safety/checkpoint.rs`
  - Define checkpoint structure:
    ```rust
    pub struct Checkpoint {
        pub id: CheckpointId,
        pub project_id: String,
        pub created_at: DateTime<Utc>,
        pub description: String,
        pub trigger: CheckpointTrigger,
        pub components: CheckpointComponents,
    }
    
    pub enum CheckpointTrigger {
        Manual { user_id: String },
        AutoPreRiskyOp { operation: String },
        Scheduled,
    }
    
    pub struct CheckpointComponents {
        pub git_ref: String,              // Branch or commit SHA
        pub container_snapshot_id: String,
        pub memory_backup_path: PathBuf,
        pub env_snapshot: HashMap<String, String>,  // Non-secret env vars
    }
    ```
  - Database table for checkpoints
  - Checkpoint storage location: `.cuttlefish/checkpoints/`
  - Enforce limit: max 20 checkpoints per project

  **Must NOT do**:
  - Don't store secrets in env snapshot (filter known secret keys)
  - Don't allow unlimited checkpoints

  **Recommended Agent Profile**:
  - **Category**: `deep`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (Wave 2 start)
  - **Blocks**: Tasks 5, 6
  - **Blocked By**: None

  **References**:
  - v1-memory.md Task 7 (StateBranch) — Similar architecture
  - v1-sandbox.md Task 7 (Snapshots) — Container snapshots

  **Acceptance Criteria**:
  - [ ] Checkpoint struct captures all state
  - [ ] Database storage works
  - [ ] Limit enforced

  **QA Scenarios**:
  ```
  Scenario: Checkpoint limit enforcement
    Tool: Bash (cargo test)
    Steps:
      1. Create 21 checkpoints
      2. Verify oldest deleted
      3. Verify 20 remain
    Expected Result: Rotation works
    Evidence: .sisyphus/evidence/task-4-limit.txt
  ```

  **Commit**: NO (groups with Wave 2)

- [ ] 5. Checkpoint Creation

  **What to do**:
  - Implement checkpoint creation:
    ```rust
    pub async fn create_checkpoint(
        project_id: &str,
        description: &str,
        trigger: CheckpointTrigger,
    ) -> Result<Checkpoint, SafetyError>
    ```
  - Steps:
    1. Create git stash or commit (configurable)
    2. Create container snapshot via sandbox
    3. Copy memory file to backup location
    4. Capture non-secret environment variables
    5. Store checkpoint metadata in database
    6. Enforce limit (delete oldest if at cap)
  - Auto-checkpoint triggers:
    - Before git reset/rebase
    - Before rm -rf or large deletions
    - Before package major upgrades

  **Must NOT do**:
  - Don't checkpoint on every action (only risky ones)
  - Don't block agent execution waiting for snapshot

  **Recommended Agent Profile**:
  - **Category**: `deep`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Task 4)
  - **Blocks**: Tasks 6, 7
  - **Blocked By**: Task 4

  **References**:
  - v1-sandbox.md Task 7 — Container snapshots
  - v1-memory.md Task 8 — Branch creation

  **Acceptance Criteria**:
  - [ ] All components captured
  - [ ] Auto-triggers work
  - [ ] Checkpoint created in < 30 seconds

  **QA Scenarios**:
  ```
  Scenario: Create checkpoint captures all state
    Tool: Bash (cargo test)
    Steps:
      1. Create checkpoint
      2. Verify git ref captured
      3. Verify container snapshot exists
      4. Verify memory backup exists
    Expected Result: All components present
    Evidence: .sisyphus/evidence/task-5-create.txt
  ```

  **Commit**: NO (groups with Wave 2)

- [ ] 6. Checkpoint Restoration (Rollback)

  **What to do**:
  - Implement rollback:
    ```rust
    pub async fn rollback_to_checkpoint(
        project_id: &str,
        checkpoint_id: CheckpointId,
        create_safety_checkpoint: bool,  // Checkpoint current state first
    ) -> Result<(), SafetyError>
    ```
  - Steps:
    1. Optionally create checkpoint of current state
    2. Stop running container
    3. Restore container from snapshot
    4. Git checkout/reset to checkpoint ref
    5. Restore memory file from backup
    6. Start container
  - Implement undo functionality:
    ```rust
    pub async fn undo_last_operations(
        project_id: &str,
        count: usize,  // Number of operations to undo
    ) -> Result<UndoResult, SafetyError>
    ```
  - Track operations in a journal for undo

  **Must NOT do**:
  - Don't rollback without option to save current state
  - Don't delete the checkpoint after rollback

  **Recommended Agent Profile**:
  - **Category**: `deep`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: NO (depends on Tasks 4, 5)
  - **Blocks**: Tasks 7, 8
  - **Blocked By**: Tasks 4, 5

  **References**:
  - v1-memory.md Task 9 — Branch restoration
  - v1-sandbox.md Task 8 — Container restore

  **Acceptance Criteria**:
  - [ ] Full state restored
  - [ ] Container running after rollback
  - [ ] Undo works for file operations

  **QA Scenarios**:
  ```
  Scenario: Rollback restores complete state
    Tool: Bash (cargo test)
    Steps:
      1. Create checkpoint
      2. Make changes (files, packages)
      3. Rollback
      4. Verify original state restored
    Expected Result: State matches checkpoint
    Evidence: .sisyphus/evidence/task-6-rollback.txt
  ```

  **Commit**: YES (Wave 2)
  - Message: `feat(agents): add checkpoint creation and rollback`
  - Files: `safety/checkpoint.rs`

- [ ] 7. Safety API Endpoints

  **What to do**:
  - Add to `crates/cuttlefish-api/src/`:
    - `GET /api/projects/:id/safety/config` — Get gate configuration
    - `PUT /api/projects/:id/safety/config` — Update gate configuration
    - `GET /api/projects/:id/safety/pending` — Get pending approvals
    - `POST /api/projects/:id/safety/approve/:action_id` — Approve action
    - `POST /api/projects/:id/safety/reject/:action_id` — Reject action
    - `GET /api/projects/:id/checkpoints` — List checkpoints
    - `POST /api/projects/:id/checkpoints` — Create manual checkpoint
    - `POST /api/projects/:id/checkpoints/:id/rollback` — Rollback
    - `POST /api/projects/:id/undo` — Undo last N operations
    - `GET /api/projects/:id/diff/:action_id` — Get diff preview
  - WebSocket events for pending approvals
  - Timeout for pending approvals (configurable, default 5 min)

  **Must NOT do**:
  - Don't auto-reject on timeout (pause execution instead)
  - Don't expose internal checkpoint paths

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 8, 9)
  - **Blocks**: Task 10
  - **Blocked By**: Tasks 2, 3, 5, 6

  **Acceptance Criteria**:
  - [ ] All endpoints work
  - [ ] WebSocket events fire
  - [ ] Timeout behavior correct

  **QA Scenarios**:
  ```
  Scenario: Approve pending action via API
    Tool: Bash (curl)
    Steps:
      1. Trigger action that requires approval
      2. GET /api/projects/test/safety/pending
      3. POST /api/projects/test/safety/approve/{id}
      4. Verify action proceeds
    Expected Result: Approval workflow works
    Evidence: .sisyphus/evidence/task-7-approve-api.txt
  ```

  **Commit**: YES
  - Message: `feat(api): add safety endpoints for gates and checkpoints`
  - Files: `safety_routes.rs`, `routes.rs`

- [ ] 8. CLI Safety Commands

  **What to do**:
  - Add commands to main binary:
    - `cuttlefish checkpoint [description]` — Create manual checkpoint
    - `cuttlefish checkpoint list` — List checkpoints
    - `cuttlefish rollback <checkpoint-id>` — Rollback to checkpoint
    - `cuttlefish rollback --latest` — Rollback to most recent
    - `cuttlefish undo [count]` — Undo last N operations (default 1)
    - `cuttlefish safety config` — Show current gate config
    - `cuttlefish safety config --auto-approve 0.8` — Update thresholds
  - Human-readable output with colors
  - `--json` flag for machine output
  - Confirmation prompt for rollback (unless `--yes`)

  **Must NOT do**:
  - Don't rollback without confirmation

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 7, 9)
  - **Blocks**: F1-F4
  - **Blocked By**: Task 6

  **Acceptance Criteria**:
  - [ ] All commands work
  - [ ] Confirmation prompts work
  - [ ] JSON output valid

  **QA Scenarios**:
  ```
  Scenario: CLI checkpoint and rollback
    Tool: Bash
    Steps:
      1. cuttlefish checkpoint "Before test"
      2. Make changes
      3. cuttlefish rollback --latest --yes
      4. Verify restored
    Expected Result: CLI workflow works
    Evidence: .sisyphus/evidence/task-8-cli.txt
  ```

  **Commit**: NO (groups with Wave 3)

- [ ] 9. Wire Gates into Agent Execution

  **What to do**:
  - Modify agent execution pipeline:
    - Before each tool execution, calculate confidence
    - Evaluate against gates
    - If `PromptUser`: pause execution, emit WebSocket event, wait for approval
    - If `Block`: return error to agent with reason
    - If `AutoApprove`: proceed
  - Add approval timeout handling
  - Add bypass flag for trusted operations (internal use only)
  - Ensure gates don't break existing agent tests

  **Must NOT do**:
  - Don't block internal agent operations (only user-facing)
  - Don't add gates to read operations

  **Recommended Agent Profile**:
  - **Category**: `deep`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 7, 8)
  - **Blocks**: F1-F4
  - **Blocked By**: Tasks 1, 2

  **References**:
  - `crates/cuttlefish-agents/src/runner.rs` — Agent execution
  - `crates/cuttlefish-agents/src/tools.rs` — Tool dispatch

  **Acceptance Criteria**:
  - [ ] Gates integrated into execution
  - [ ] Approval flow works end-to-end
  - [ ] Existing tests still pass

  **QA Scenarios**:
  ```
  Scenario: Low confidence action pauses for approval
    Tool: Bash (integration test)
    Steps:
      1. Configure low auto-approve threshold
      2. Trigger action that normally auto-approves
      3. Verify execution pauses
      4. Approve via API
      5. Verify execution continues
    Expected Result: Gate pauses and resumes correctly
    Evidence: .sisyphus/evidence/task-9-gate-flow.txt
  ```

  **Commit**: YES (Wave 3)
  - Message: `feat(agents): wire safety gates into agent execution`
  - Files: `runner.rs`, `tools.rs`

- [ ] 10. WebUI Diff Preview and Approval

  **What to do**:
  - Create `cuttlefish-web/components/DiffPreview.vue`:
    - Display unified diff with syntax highlighting
    - Line numbers for old/new content
    - Color coding: green for additions, red for removals
    - Collapsible hunks for large diffs
    - Stats summary (X added, Y removed)
  - Create `cuttlefish-web/components/ApprovalModal.vue`:
    - Shows when action requires approval
    - Displays: action description, confidence score, risk factors
    - Includes diff preview for file operations
    - Approve/Reject buttons
    - Countdown timer for timeout
  - Add toast notifications for pending approvals
  - Real-time updates via WebSocket

  **Must NOT do**:
  - Don't block entire UI while waiting for approval
  - Don't auto-dismiss approval modal

  **Recommended Agent Profile**:
  - **Category**: `visual-engineering`
  - **Skills**: [`frontend-ui-ux`]

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Blocks**: F1-F4
  - **Blocked By**: Tasks 3, 7

  **References**:
  - `cuttlefish-web/pages/project/[id].vue` — Existing diff styling
  - GitHub diff view for inspiration

  **Acceptance Criteria**:
  - [ ] Diff preview shows correctly
  - [ ] Approval modal functional
  - [ ] WebSocket updates work
  - [ ] Mobile responsive

  **QA Scenarios**:
  ```
  Scenario: Approval modal workflow
    Tool: Playwright
    Steps:
      1. Trigger action requiring approval
      2. Verify modal appears with diff
      3. Click Approve
      4. Verify modal closes, action proceeds
    Expected Result: Full approval flow works
    Evidence: .sisyphus/evidence/task-10-approval-modal.png
  ```

  **Commit**: YES (Wave 4)
  - Message: `feat(web): add diff preview and approval modal`
  - Files: `DiffPreview.vue`, `ApprovalModal.vue`

---

## Final Verification Wave

- [ ] F1. **Plan Compliance Audit** — `oracle`
  Verify confidence scoring, gates, diff preview, checkpoints, and rollback all implemented. Check no secrets stored.
  Output: `Must Have [N/N] | Must NOT Have [N/N] | Tasks [N/N] | VERDICT: APPROVE/REJECT`

- [ ] F2. **Code Quality Review** — `unspecified-high`
  Run `cargo clippy --workspace -- -D warnings` + tests. Review for unwrap(), security issues.
  Output: `Build [PASS/FAIL] | Lint [PASS/FAIL] | Tests [N pass/N fail] | VERDICT`

- [ ] F3. **Safety E2E QA** — `unspecified-high`
  Full workflow: trigger low-confidence action, see approval modal, approve, verify action proceeds. Create checkpoint, make changes, rollback, verify restoration.
  Output: `Gates [PASS/FAIL] | Checkpoints [PASS/FAIL] | Rollback [PASS/FAIL] | VERDICT`

- [ ] F4. **Scope Fidelity Check** — `deep`
  Verify no secrets in checkpoints, checkpoint limit enforced, gates don't break existing tests.
  Output: `Tasks [N/N compliant] | Scope [CLEAN/N violations] | VERDICT`

---

## Commit Strategy

| Wave | Commit |
|------|--------|
| 1 | `feat(agents): add confidence scoring system` |
| 1 | `feat(agents): add confidence gates and diff generation` |
| 2 | `feat(agents): add checkpoint creation and rollback` |
| 3 | `feat(api): add safety endpoints for gates and checkpoints` |
| 3 | `feat(agents): wire safety gates into agent execution` |
| 4 | `feat(web): add diff preview and approval modal` |

---

## Success Criteria

### Verification Commands
```bash
cargo test -p cuttlefish-agents safety  # All tests pass
cargo clippy --workspace -- -D warnings  # Clean
cuttlefish checkpoint "test"  # Creates checkpoint
cuttlefish rollback --latest --yes  # Restores state
```

### Final Checklist
- [ ] Confidence scores calculated for all action types
- [ ] Gates pause execution for low-confidence actions
- [ ] Diff preview shows file changes clearly
- [ ] Checkpoints capture container + git + memory
- [ ] Rollback restores complete state
- [ ] No secrets stored in checkpoints
- [ ] Checkpoint limit enforced (max 20)
