# V1 Discord Bot — Full Server Integration

## TL;DR

> **Quick Summary**: Build a comprehensive Discord bot with slash commands, smart notifications, auto-created project channels, and rich agent status embeds for seamless team interaction with Cuttlefish.
> 
> **Deliverables**:
> - Slash commands: `/new-project`, `/status`, `/logs`, `/approve`, `/reject`
> - Smart notifications with @mentions when agent needs input
> - Auto-created project channels (text + optional thread)
> - Rich embeds showing agent thinking/progress with status indicators
> - Channel archival for inactive projects
> 
> **Estimated Effort**: Large (4-5 days)
> **Parallel Execution**: YES - 5 waves
> **Critical Path**: Task 1 (Commands) → Tasks 2-4 (Features) → Task 5 (Embeds) → Task 6 (Polish)

---

## Context

### Original Request
Full Discord server management including new chat channels individualized for each project, nice looking chats formatted properly from each agent, and proper notifications for when it's waiting for an answer, completing tasks with subagents in the background, etc.

### Current State (Research Findings)
**Already exists (`crates/cuttlefish-discord/src/`):**
- `lib.rs` — Bot initialization and event handling
- `channel_manager.rs` — Basic channel operations
- `commands.rs` — Command registration structure
- `formatter.rs` — Message formatting utilities
- `guild_config.rs` — Per-guild configuration

**Missing:**
- Slash command implementations (only registration exists)
- Smart notification system with @mentions
- Project channel auto-creation
- Rich agent status embeds
- Channel archival logic
- Thread support for focused discussions

### Design Direction
- **Discord-native**: Use slash commands, embeds, threads — not just text messages
- **Project-centric**: Each project gets its own channel
- **Non-blocking**: Notifications when needed, not spam

---

## Work Objectives

### Core Objective
Create a Discord bot that makes interacting with Cuttlefish as natural as chatting with a teammate — project channels, smart notifications, and visual status updates.

### Concrete Deliverables
- `crates/cuttlefish-discord/src/commands/` — Slash command handlers
- `crates/cuttlefish-discord/src/notifications.rs` — Smart notification system
- `crates/cuttlefish-discord/src/embeds.rs` — Rich embed builders
- `crates/cuttlefish-discord/src/channels.rs` — Project channel lifecycle
- `crates/cuttlefish-discord/src/threads.rs` — Thread management

### Definition of Done
- [ ] All 5 slash commands work in Discord
- [ ] Project channels auto-created on `/new-project`
- [ ] @mention notifications when agent needs input
- [ ] Agent status shows as rich embeds
- [ ] `cargo test -p cuttlefish-discord` passes
- [ ] `cargo clippy -p cuttlefish-discord -- -D warnings` clean

### Must Have
- `/new-project <name> [template]` — Create project + channel
- `/status [project]` — Show project/agent status
- `/logs [project] [lines]` — Show recent activity
- `/approve` — Approve pending agent action
- `/reject [reason]` — Reject with feedback
- Auto-channel creation in designated category
- @user mention when agent asks a question
- Embed with agent name, status, current action

### Must NOT Have (Guardrails)
- No unsafe code
- No `unwrap()` — use `?` or `expect("reason")`
- No bot token in code (env var only)
- No DM support (guild-only for V1)
- No voice channel integration (text only)
- No custom emoji requirements (use Unicode)

---

## Verification Strategy

> **ZERO HUMAN INTERVENTION** — ALL verification is agent-executed.

### Test Decision
- **Infrastructure exists**: YES (serenity framework)
- **Automated tests**: YES (Tests-after)
- **Framework**: `#[tokio::test]` for async tests

### QA Policy
Every task includes agent-executed QA scenarios.
Evidence saved to `.sisyphus/evidence/task-{N}-{scenario-slug}.{ext}`.

- **Command tests**: Mock Discord context, verify response
- **Integration tests**: Real bot in test server (if available)
- **Embed tests**: Verify embed structure matches Discord limits

---

## Execution Strategy

### Parallel Execution Waves

```
Wave 1 (Foundation — command structure):
├── Task 1: Slash command framework setup [quick]
├── Task 2: Implement /new-project command [unspecified-high]
└── Task 3: Implement /status command [quick]

Wave 2 (Core commands):
├── Task 4: Implement /logs command [quick]
├── Task 5: Implement /approve and /reject commands [unspecified-high]
└── Task 6: Project channel auto-creation [deep]

Wave 3 (Notifications):
├── Task 7: Smart notification system [deep]
├── Task 8: Pending action detection [unspecified-high]
└── Task 9: User mention routing [quick]

Wave 4 (Embeds and polish):
├── Task 10: Rich agent status embeds [visual-engineering]
├── Task 11: Progress indicator embeds [visual-engineering]
└── Task 12: Channel archival system [unspecified-high]

Wave 5 (Integration):
└── Task 13: Wire bot to Cuttlefish API [deep]

Wave FINAL (Verification):
├── Task F1: Plan compliance audit (oracle)
├── Task F2: Code quality review (unspecified-high)
├── Task F3: Discord integration QA (unspecified-high)
└── Task F4: Scope fidelity check (deep)
-> Present results -> Get explicit user okay

Critical Path: Task 1 → Task 2 → Task 6 → Task 7 → Task 13 → F1-F4 → user okay
Parallel Speedup: ~60% faster than sequential
Max Concurrent: 3 (Waves 2, 3, 4)
```

### Dependency Matrix

| Task | Depends On | Blocks | Wave |
|------|------------|--------|------|
| 1 | — | 2-5 | 1 |
| 2 | 1 | 6 | 1 |
| 3 | 1 | 13 | 1 |
| 4 | 1 | 13 | 2 |
| 5 | 1 | 7, 13 | 2 |
| 6 | 2 | 7, 12, 13 | 2 |
| 7 | 5, 6 | 9, 13 | 3 |
| 8 | 5 | 9 | 3 |
| 9 | 7, 8 | 13 | 3 |
| 10 | — | 11, 13 | 4 |
| 11 | 10 | 13 | 4 |
| 12 | 6 | 13 | 4 |
| 13 | 3-12 | F1-F4 | 5 |

### Agent Dispatch Summary

- **Wave 1**: 3 tasks — T1 → `quick`, T2 → `unspecified-high`, T3 → `quick`
- **Wave 2**: 3 tasks — T4 → `quick`, T5 → `unspecified-high`, T6 → `deep`
- **Wave 3**: 3 tasks — T7 → `deep`, T8 → `unspecified-high`, T9 → `quick`
- **Wave 4**: 3 tasks — T10-T11 → `visual-engineering`, T12 → `unspecified-high`
- **Wave 5**: 1 task — T13 → `deep`
- **FINAL**: 4 tasks — F1 → `oracle`, F2-F3 → `unspecified-high`, F4 → `deep`

---

## TODOs

- [ ] 1. Slash Command Framework Setup

  **What to do**:
  - Create `crates/cuttlefish-discord/src/commands/mod.rs` module structure
  - Define command registration using serenity's `CreateCommand` builder
  - Create `register_commands()` function that registers all slash commands with Discord API
  - Implement command router in event handler that dispatches to specific handlers
  - Add command options parsing utilities (extract project name, template, etc.)
  - Handle command acknowledgment (defer reply for long operations)

  **Must NOT do**:
  - Don't register commands globally (use guild commands for faster updates during dev)
  - Don't implement command logic yet (just structure)

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: NO (foundation)
  - **Blocks**: Tasks 2-5
  - **Blocked By**: None

  **References**:
  - `crates/cuttlefish-discord/src/commands.rs` — Existing command structure
  - serenity slash commands: https://docs.rs/serenity/latest/serenity/builder/struct.CreateCommand.html

  **Acceptance Criteria**:
  - [ ] Command module structure exists
  - [ ] Commands register without error
  - [ ] Router dispatches to correct handler

  **QA Scenarios**:
  ```
  Scenario: Commands register successfully
    Tool: Bash (cargo test)
    Preconditions: Command module exists
    Steps:
      1. Run `cargo test -p cuttlefish-discord commands::tests::test_register`
      2. Verify registration succeeds
    Expected Result: All commands registered
    Evidence: .sisyphus/evidence/task-1-register.txt
  ```

  **Commit**: YES
  - Message: `feat(discord): add slash command framework`
  - Files: `crates/cuttlefish-discord/src/commands/*`

- [ ] 2. Implement /new-project Command

  **What to do**:
  - Create `crates/cuttlefish-discord/src/commands/new_project.rs`
  - Command options: `name` (required string), `template` (optional string), `description` (optional)
  - Handler flow:
    1. Defer reply (ephemeral)
    2. Call Cuttlefish API to create project
    3. Create project channel (call Task 6's function)
    4. Send success message with channel link
  - Error handling: project name taken, API error, channel creation failed
  - Autocomplete for template names

  **Must NOT do**:
  - Don't create channel here (delegate to channel manager)
  - Don't hardcode template list (fetch from API)

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Task 3)
  - **Blocks**: Task 6
  - **Blocked By**: Task 1

  **References**:
  - `crates/cuttlefish-api/src/api_routes.rs:37-51` — Project creation API

  **Acceptance Criteria**:
  - [ ] Command creates project via API
  - [ ] Returns success message with project details
  - [ ] Template autocomplete works

  **QA Scenarios**:
  ```
  Scenario: Create project via slash command
    Tool: Bash (cargo test)
    Preconditions: Mock API server
    Steps:
      1. Invoke command handler with test context
      2. Verify API called with correct params
      3. Verify response message contains project info
    Expected Result: Project created, channel linked
    Evidence: .sisyphus/evidence/task-2-new-project.txt
  ```

  **Commit**: NO (groups with Wave 1)

- [ ] 3. Implement /status Command

  **What to do**:
  - Create `crates/cuttlefish-discord/src/commands/status.rs`
  - Command options: `project` (optional string — defaults to current channel's project)
  - Handler flow:
    1. Determine project (from option or channel context)
    2. Fetch project status from API
    3. Build status embed (Task 10)
    4. Reply with embed
  - Show: project name, active agents, last activity, current task
  - If no project context, list all user's projects

  **Must NOT do**:
  - Don't show sensitive info (API keys, internal IDs)

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Task 2)
  - **Blocks**: Task 13
  - **Blocked By**: Task 1

  **References**:
  - `crates/cuttlefish-api/src/api_routes.rs` — Status endpoint

  **Acceptance Criteria**:
  - [ ] Command returns project status
  - [ ] Works with explicit project name or channel context
  - [ ] Output formatted as embed

  **QA Scenarios**:
  ```
  Scenario: Status command shows project info
    Tool: Bash (cargo test)
    Steps:
      1. Invoke /status with project name
      2. Verify embed contains agent status
    Expected Result: Status displayed correctly
    Evidence: .sisyphus/evidence/task-3-status.txt
  ```

  **Commit**: NO (groups with Wave 1)

- [ ] 4. Implement /logs Command

  **What to do**:
  - Create `crates/cuttlefish-discord/src/commands/logs.rs`
  - Command options: `project` (optional), `lines` (optional, default 20, max 50)
  - Fetch recent agent activity from API
  - Format as code block with timestamps
  - Paginate if more than 2000 chars (Discord limit)
  - Add "Show More" button for pagination

  **Must NOT do**:
  - Don't dump raw JSON (human-readable format)
  - Don't show more than 50 lines (performance)

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 5, 6)
  - **Blocks**: Task 13
  - **Blocked By**: Task 1

  **Acceptance Criteria**:
  - [ ] Command returns formatted logs
  - [ ] Pagination works for long output
  - [ ] Timestamps are human-readable

  **QA Scenarios**:
  ```
  Scenario: Logs command with pagination
    Tool: Bash (cargo test)
    Steps:
      1. Create mock response > 2000 chars
      2. Invoke /logs
      3. Verify pagination button present
    Expected Result: Logs paginated correctly
    Evidence: .sisyphus/evidence/task-4-logs.txt
  ```

  **Commit**: YES (Wave 2)
  - Message: `feat(discord): add /status and /logs commands`
  - Files: `commands/status.rs`, `commands/logs.rs`

- [ ] 5. Implement /approve and /reject Commands

  **What to do**:
  - Create `crates/cuttlefish-discord/src/commands/approve.rs`
  - Create `crates/cuttlefish-discord/src/commands/reject.rs`
  - `/approve` options: none (approves pending action in channel context)
  - `/reject` options: `reason` (optional string)
  - Check for pending action in current channel/project
  - Call API to approve/reject
  - Update original notification message (edit to show "Approved by @user")
  - Trigger agent to continue/retry

  **Must NOT do**:
  - Don't allow approve/reject without pending action
  - Don't allow users to approve others' requests (permission check)

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 4, 6)
  - **Blocks**: Tasks 7, 13
  - **Blocked By**: Task 1

  **References**:
  - Pending action API endpoint (may need to create)

  **Acceptance Criteria**:
  - [ ] Approve triggers agent continuation
  - [ ] Reject sends feedback to agent
  - [ ] Original message updated with result

  **QA Scenarios**:
  ```
  Scenario: Approve pending action
    Tool: Bash (cargo test)
    Steps:
      1. Create pending action state
      2. Invoke /approve
      3. Verify action marked approved
      4. Verify agent notified
    Expected Result: Action approved, agent continues
    Evidence: .sisyphus/evidence/task-5-approve.txt
  ```

  **Commit**: YES
  - Message: `feat(discord): add /approve and /reject commands`
  - Files: `commands/approve.rs`, `commands/reject.rs`

- [ ] 6. Project Channel Auto-Creation

  **What to do**:
  - Enhance `crates/cuttlefish-discord/src/channel_manager.rs`
  - Implement `create_project_channel(guild_id, project_name, category_id) -> Result<ChannelId>`
  - Channel naming: `project-{sanitized_name}` (lowercase, dashes)
  - Set channel topic to project description
  - Add channel permissions (only project members can see)
  - Store channel-project mapping in database
  - Create initial message with project info embed

  **Must NOT do**:
  - Don't create duplicate channels for same project
  - Don't use threads by default (dedicated channels)

  **Recommended Agent Profile**:
  - **Category**: `deep`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 4, 5)
  - **Blocks**: Tasks 7, 12, 13
  - **Blocked By**: Task 2

  **References**:
  - `crates/cuttlefish-discord/src/channel_manager.rs` — Existing channel utilities
  - serenity channel creation

  **Acceptance Criteria**:
  - [ ] Channel created with correct name
  - [ ] Permissions set correctly
  - [ ] Mapping stored in database

  **QA Scenarios**:
  ```
  Scenario: Auto-create project channel
    Tool: Bash (cargo test)
    Steps:
      1. Call create_project_channel()
      2. Verify channel exists with correct name
      3. Verify database mapping created
    Expected Result: Channel ready for use
    Evidence: .sisyphus/evidence/task-6-channel.txt
  ```

  **Commit**: YES
  - Message: `feat(discord): add project channel auto-creation`
  - Files: `channel_manager.rs`

- [ ] 7. Smart Notification System

  **What to do**:
  - Create `crates/cuttlefish-discord/src/notifications.rs`
  - Implement `NotificationManager` struct
  - `notify_user(user_id, project_id, message, urgency)` method
  - Urgency levels: `info`, `action_required`, `error`
  - For `action_required`: include @mention, add approve/reject buttons
  - Rate limiting: max 1 notification per user per minute for same project
  - Store pending notifications for offline users
  - Deliver when user comes online or after timeout

  **Must NOT do**:
  - Don't spam users (rate limit)
  - Don't notify for every agent action (only important ones)

  **Recommended Agent Profile**:
  - **Category**: `deep`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 8, 9)
  - **Blocks**: Task 9, 13
  - **Blocked By**: Tasks 5, 6

  **References**:
  - Discord notification best practices

  **Acceptance Criteria**:
  - [ ] @mention appears for action_required
  - [ ] Rate limiting prevents spam
  - [ ] Buttons work on notifications

  **QA Scenarios**:
  ```
  Scenario: Action required notification
    Tool: Bash (cargo test)
    Steps:
      1. Trigger action_required notification
      2. Verify @mention included
      3. Verify buttons present
    Expected Result: User pinged with actionable message
    Evidence: .sisyphus/evidence/task-7-notify.txt
  ```

  **Commit**: YES
  - Message: `feat(discord): add smart notification system`
  - Files: `notifications.rs`

- [ ] 8. Pending Action Detection

  **What to do**:
  - Add `PendingAction` struct: `id`, `project_id`, `user_id`, `action_type`, `context`, `created_at`
  - Actions types: `ConfirmDestructive`, `ApproveChange`, `AnswerQuestion`, `ReviewDiff`
  - Store pending actions in database
  - Detect from agent output (parse for question markers, confirmation requests)
  - Timeout after configurable period (default 24h)
  - Auto-reject or remind after timeout

  **Must NOT do**:
  - Don't create false positives (only clear action requests)

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 7, 9)
  - **Blocks**: Task 9
  - **Blocked By**: Task 5

  **Acceptance Criteria**:
  - [ ] Actions detected from agent output
  - [ ] Stored with correct type and context
  - [ ] Timeout handling works

  **QA Scenarios**:
  ```
  Scenario: Detect pending action from agent
    Tool: Bash (cargo test)
    Steps:
      1. Feed agent output with question
      2. Verify PendingAction created
      3. Verify correct type assigned
    Expected Result: Action detected and stored
    Evidence: .sisyphus/evidence/task-8-detect.txt
  ```

  **Commit**: NO (groups with Wave 3)

- [ ] 9. User Mention Routing

  **What to do**:
  - Determine which user to mention for each project
  - Primary: project owner (from project creation)
  - Secondary: last active user in project channel
  - Fallback: configured admin role
  - Store user-project associations
  - Handle user leaving server (fall back to admin)

  **Must NOT do**:
  - Don't mention @everyone or @here

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: NO (depends on 7, 8)
  - **Blocks**: Task 13
  - **Blocked By**: Tasks 7, 8

  **Acceptance Criteria**:
  - [ ] Correct user mentioned per project
  - [ ] Fallback works when owner unavailable

  **QA Scenarios**:
  ```
  Scenario: Route mention to project owner
    Tool: Bash (cargo test)
    Steps:
      1. Create project with owner
      2. Trigger notification
      3. Verify owner mentioned
    Expected Result: Owner gets @mention
    Evidence: .sisyphus/evidence/task-9-routing.txt
  ```

  **Commit**: YES (Wave 3)
  - Message: `feat(discord): add notification routing and pending actions`
  - Files: `notifications.rs`, `pending_actions.rs`

- [ ] 10. Rich Agent Status Embeds

  **What to do**:
  - Create `crates/cuttlefish-discord/src/embeds.rs`
  - Implement `AgentStatusEmbed` builder
  - Fields:
    - Agent name with emoji (🎭 Orchestrator, 💻 Coder, 🔍 Critic)
    - Current status (Thinking, Working, Waiting, Complete)
    - Current action description
    - Time elapsed
    - Sub-agent status (if delegating)
  - Color coding: green=success, yellow=working, red=error, gray=waiting
  - Thumbnail: agent-specific icon (optional)

  **Must NOT do**:
  - Don't exceed Discord embed limits (6000 chars total)
  - Don't use external images (Unicode emojis only)

  **Recommended Agent Profile**:
  - **Category**: `visual-engineering`
  - **Skills**: [`frontend-ui-ux`]

  **Parallelization**:
  - **Can Run In Parallel**: YES (independent)
  - **Blocks**: Tasks 11, 13
  - **Blocked By**: None

  **References**:
  - Discord embed limits: https://discord.com/developers/docs/resources/channel#embed-limits

  **Acceptance Criteria**:
  - [ ] Embed renders correctly in Discord
  - [ ] All status types have distinct colors
  - [ ] Within Discord limits

  **QA Scenarios**:
  ```
  Scenario: Agent status embed renders
    Tool: Bash (cargo test)
    Steps:
      1. Build embed for each agent type
      2. Verify JSON within limits
      3. Verify color coding correct
    Expected Result: Valid Discord embed JSON
    Evidence: .sisyphus/evidence/task-10-embed.txt
  ```

  **Commit**: NO (groups with Wave 4)

- [ ] 11. Progress Indicator Embeds

  **What to do**:
  - Implement `ProgressEmbed` for long-running operations
  - Show:
    - Task description
    - Progress bar (using block characters: ▓▓▓░░░░░░░ 30%)
    - Estimated time remaining (if available)
    - Steps completed / total steps
    - Current step description
  - Update embed in place (edit message, don't create new)
  - Final state: success/failure summary

  **Must NOT do**:
  - Don't update more than once per 5 seconds (rate limit)

  **Recommended Agent Profile**:
  - **Category**: `visual-engineering`
  - **Skills**: [`frontend-ui-ux`]

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 10, 12)
  - **Blocks**: Task 13
  - **Blocked By**: Task 10

  **Acceptance Criteria**:
  - [ ] Progress bar renders correctly
  - [ ] Updates without creating new messages
  - [ ] Rate limited to 5s intervals

  **QA Scenarios**:
  ```
  Scenario: Progress updates in place
    Tool: Bash (cargo test)
    Steps:
      1. Create progress embed
      2. Update progress 3 times
      3. Verify same message ID edited
    Expected Result: Single message updated
    Evidence: .sisyphus/evidence/task-11-progress.txt
  ```

  **Commit**: NO (groups with Wave 4)

- [ ] 12. Channel Archival System

  **What to do**:
  - Implement `archive_project_channel(channel_id)`
  - Move channel to "Archived" category
  - Rename with `-archived` suffix
  - Remove write permissions (read-only)
  - Add final summary message
  - Track archival in database
  - Auto-archive after configurable inactivity (default 30 days)
  - Unarchive command: `/unarchive <project>`

  **Must NOT do**:
  - Don't delete channels (archive only)
  - Don't auto-archive active projects

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 10, 11)
  - **Blocks**: Task 13
  - **Blocked By**: Task 6

  **Acceptance Criteria**:
  - [ ] Channel moves to archive category
  - [ ] Permissions updated to read-only
  - [ ] Unarchive restores functionality

  **QA Scenarios**:
  ```
  Scenario: Archive inactive project
    Tool: Bash (cargo test)
    Steps:
      1. Create project channel
      2. Call archive function
      3. Verify moved and read-only
    Expected Result: Channel archived correctly
    Evidence: .sisyphus/evidence/task-12-archive.txt
  ```

  **Commit**: YES (Wave 4)
  - Message: `feat(discord): add embeds and channel archival`
  - Files: `embeds.rs`, `channel_manager.rs`

- [ ] 13. Wire Bot to Cuttlefish API

  **What to do**:
  - Create `crates/cuttlefish-discord/src/api_client.rs`
  - Implement HTTP client for Cuttlefish API
  - Endpoints needed:
    - `POST /api/projects` — Create project
    - `GET /api/projects/:id` — Get project status
    - `GET /api/projects/:id/logs` — Get activity logs
    - `POST /api/projects/:id/approve` — Approve action
    - `POST /api/projects/:id/reject` — Reject action
  - Handle authentication (API key from config)
  - Wire all commands to use API client
  - Handle API errors gracefully (show user-friendly messages)

  **Must NOT do**:
  - Don't hardcode API URL (config)
  - Don't expose internal errors to users

  **Recommended Agent Profile**:
  - **Category**: `deep`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: NO (integration task)
  - **Blocks**: F1-F4
  - **Blocked By**: Tasks 3-12

  **References**:
  - `crates/cuttlefish-api/src/api_routes.rs` — API endpoints

  **Acceptance Criteria**:
  - [ ] All commands call correct API endpoints
  - [ ] Auth works correctly
  - [ ] Error handling is user-friendly

  **QA Scenarios**:
  ```
  Scenario: End-to-end command flow
    Tool: Bash (cargo test)
    Steps:
      1. Start mock API server
      2. Run /new-project command
      3. Verify API called, channel created
    Expected Result: Full flow works
    Evidence: .sisyphus/evidence/task-13-e2e.txt
  ```

  **Commit**: YES
  - Message: `feat(discord): wire bot to Cuttlefish API`
  - Files: `api_client.rs`, all command files

---

## Final Verification Wave

- [ ] F1. **Plan Compliance Audit** — `oracle`
  Read the plan end-to-end. For each "Must Have": verify implementation exists. For each "Must NOT Have": search codebase for forbidden patterns. Check evidence files exist.
  Output: `Must Have [N/N] | Must NOT Have [N/N] | Tasks [N/N] | VERDICT: APPROVE/REJECT`

- [ ] F2. **Code Quality Review** — `unspecified-high`
  Run `cargo clippy -p cuttlefish-discord -- -D warnings` + `cargo test -p cuttlefish-discord`. Review for unwrap(), hardcoded values.
  Output: `Build [PASS/FAIL] | Lint [PASS/FAIL] | Tests [N pass/N fail] | VERDICT`

- [ ] F3. **Discord Integration QA** — `unspecified-high`
  If test Discord server available: create project via /new-project, verify channel created. Trigger notification, verify @mention. Test approve/reject flow.
  Output: `Commands [N/N pass] | Notifications [PASS/FAIL] | Embeds [PASS/FAIL] | VERDICT`

- [ ] F4. **Scope Fidelity Check** — `deep`
  Verify no DM support added, no voice channels, no custom emoji requirements. Check all files match task specifications.
  Output: `Tasks [N/N compliant] | Scope [CLEAN/N violations] | VERDICT`

---

## Commit Strategy

| Wave | Commit |
|------|--------|
| 1 | `feat(discord): add slash command framework` |
| 2 | `feat(discord): add core slash commands` |
| 2 | `feat(discord): add project channel auto-creation` |
| 3 | `feat(discord): add smart notification system` |
| 4 | `feat(discord): add embeds and channel archival` |
| 5 | `feat(discord): wire bot to Cuttlefish API` |

---

## Success Criteria

### Verification Commands
```bash
cargo test -p cuttlefish-discord  # All tests pass
cargo clippy -p cuttlefish-discord -- -D warnings  # Clean
grep -r "unwrap()" crates/cuttlefish-discord/src/  # No matches
```

### Final Checklist
- [ ] All 5 slash commands implemented and working
- [ ] Project channels auto-created
- [ ] @mention notifications for action_required
- [ ] Rich embeds for agent status
- [ ] Channel archival working
- [ ] No unsafe code, no unwrap()
