# V1 Collaboration System — Multi-User, Roles, Async Handoffs & Org Configs

## TL;DR

> **Quick Summary**: Build a collaboration system enabling multiple users to work on projects together with role-based permissions, async handoffs between sessions, and organization-level shared configurations.
> 
> **Deliverables**:
> - Project sharing with role-based access (owner, admin, member, viewer)
> - Async handoff system for passing work between users
> - Activity feed showing who did what
> - Organization management with shared configs
> - Real-time presence (who's currently working)
> - Conflict resolution for concurrent edits
> 
> **Estimated Effort**: Large (4-5 days)
> **Parallel Execution**: YES - 4 waves
> **Critical Path**: Task 1 (Sharing) → Task 2-3 (Activity) → Task 4-5 (Handoffs) → Task 6-7 (Orgs)

---

## Context

### Original Request (From Product Spec)
- **Multi-user**: Share projects with team members, each with appropriate roles
- **Async Handoffs**: "Continue this tomorrow" → teammate picks up with full context
- **Org-level configs**: Shared API keys, model preferences, templates across organization
- **Activity transparency**: See who did what, when

### Problem Statement
Current Cuttlefish is single-user. For teams, we need:
- Shared project access with permissions
- Context preservation when switching users
- Organization-wide settings and resource sharing
- Visibility into team activity

### Design Philosophy
- **Roles are simple**: 4 levels (owner, admin, member, viewer) cover 99% of cases
- **Handoffs preserve context**: Full conversation history, memory, and state
- **Orgs are optional**: Small teams can share directly; orgs for larger teams
- **Real-time optional**: Presence is nice-to-have, not blocking

### Prerequisites
- **v1-auth.md must be implemented first** — Users, sessions, roles tables

---

## Work Objectives

### Core Objective
Enable seamless team collaboration on Cuttlefish projects with clear permissions, async work handoffs, and organization-level resource sharing.

### Concrete Deliverables
- `crates/cuttlefish-db/src/collaboration.rs` — Sharing, invites, handoffs
- `crates/cuttlefish-db/src/organization.rs` — Org management
- `crates/cuttlefish-api/src/collab_routes.rs` — Collaboration API
- `crates/cuttlefish-api/src/org_routes.rs` — Organization API
- Database tables for sharing, invites, handoffs, orgs
- WebUI collaboration components
- Activity feed in project dashboard

### Definition of Done
- [ ] Projects can be shared with other users
- [ ] Roles control access correctly
- [ ] Handoffs create context-rich work items
- [ ] Organizations manage shared resources
- [ ] Activity feed shows recent actions
- [ ] `cargo test --workspace collab` passes
- [ ] `cargo clippy --workspace -- -D warnings` clean

### Must Have
- Project sharing via email invite
- Role-based access: owner (full), admin (manage), member (work), viewer (read)
- Invite system with link tokens
- Async handoff creation with context snapshot
- Activity log for all significant actions
- Organization creation and member management
- Org-level API key pools
- Org-level model configuration

### Must NOT Have (Guardrails)
- No unsafe code
- No `unwrap()` — use `?` or `expect("reason")`
- No exposing other users' email addresses (privacy)
- No role escalation vulnerabilities
- No cross-org data leakage
- No real-time collaboration in V1 (cursor sharing, etc.)

---

## Verification Strategy

> **ZERO HUMAN INTERVENTION** — ALL verification is agent-executed.

### Test Decision
- **Infrastructure exists**: YES (database, auth)
- **Automated tests**: YES (TDD)
- **Framework**: `#[tokio::test]` for async

### QA Policy
Every task includes agent-executed QA scenarios.
Evidence saved to `.sisyphus/evidence/task-{N}-{scenario-slug}.{ext}`.

---

## Execution Strategy

### Parallel Execution Waves

```
Wave 1 (Foundation — sharing + invites):
├── Task 1: Project sharing model [deep]
├── Task 2: Invite system [unspecified-high]
└── Task 3: Activity logging [unspecified-high]

Wave 2 (Handoffs — async work passing):
├── Task 4: Handoff creation [deep]
├── Task 5: Handoff acceptance + context restore [deep]
└── Task 6: Handoff notifications [quick]

Wave 3 (Organizations):
├── Task 7: Organization model [deep]
├── Task 8: Org member management [unspecified-high]
├── Task 9: Org-level configs [unspecified-high]
└── Task 10: Org API key pools [unspecified-high]

Wave 4 (Integration — API + UI):
├── Task 11: Collaboration API endpoints [unspecified-high]
├── Task 12: Organization API endpoints [unspecified-high]
├── Task 13: WebUI collaboration components [visual-engineering]
└── Task 14: Real-time presence (optional) [quick]

Wave FINAL (Verification):
├── Task F1: Plan compliance audit (oracle)
├── Task F2: Code quality review (unspecified-high)
├── Task F3: Collaboration E2E QA (unspecified-high)
└── Task F4: Scope fidelity check (deep)
-> Present results -> Get explicit user okay

Critical Path: Task 1 → Task 4 → Task 5 → Task 7 → Task 11 → Task 13 → F1-F4 → user okay
Parallel Speedup: ~55% faster than sequential
Max Concurrent: 4 (Waves 3 & 4)
```

### Dependency Matrix

| Task | Depends On | Blocks | Wave |
|------|------------|--------|------|
| 1 | v1-auth | 2, 4, 11 | 1 |
| 2 | 1 | 11, 13 | 1 |
| 3 | 1 | 11, 13 | 1 |
| 4 | 1 | 5, 6, 11 | 2 |
| 5 | 4 | 11, 13 | 2 |
| 6 | 4 | 11 | 2 |
| 7 | v1-auth | 8, 9, 10, 12 | 3 |
| 8 | 7 | 12, 13 | 3 |
| 9 | 7 | 12 | 3 |
| 10 | 7, v1-auth | 12 | 3 |
| 11 | 1-6 | 13 | 4 |
| 12 | 7-10 | 13 | 4 |
| 13 | 11, 12 | F1-F4 | 4 |
| 14 | 1 | F1-F4 | 4 |

### Agent Dispatch Summary

- **Wave 1**: 3 tasks — T1 → `deep`, T2-T3 → `unspecified-high`
- **Wave 2**: 3 tasks — T4-T5 → `deep`, T6 → `quick`
- **Wave 3**: 4 tasks — T7 → `deep`, T8-T10 → `unspecified-high`
- **Wave 4**: 4 tasks — T11-T12 → `unspecified-high`, T13 → `visual-engineering`, T14 → `quick`
- **FINAL**: 4 tasks — F1 → `oracle`, F2-F3 → `unspecified-high`, F4 → `deep`

---

## TODOs

- [ ] 1. Project Sharing Model

  **What to do**:
  - Build on `project_members` table from v1-auth:
    ```sql
    -- Already exists from v1-auth, ensure these columns:
    -- project_id, user_id, role, invited_by, created_at
    
    -- Add sharing metadata
    ALTER TABLE project_members ADD COLUMN accepted_at TEXT;
    ALTER TABLE project_members ADD COLUMN last_accessed_at TEXT;
    ```
  - Implement sharing operations:
    ```rust
    pub async fn share_project(
        project_id: &str,
        owner_id: &str,
        target_email: &str,
        role: ProjectRole,
    ) -> Result<ShareResult, CollabError>
    
    pub async fn get_project_members(project_id: &str) -> Result<Vec<ProjectMember>, CollabError>
    
    pub async fn update_member_role(
        project_id: &str,
        actor_id: &str,  // Who's making the change
        target_user_id: &str,
        new_role: ProjectRole,
    ) -> Result<(), CollabError>
    
    pub async fn remove_member(
        project_id: &str,
        actor_id: &str,
        target_user_id: &str,
    ) -> Result<(), CollabError>
    ```
  - Permission checks on all operations
  - Prevent removing last owner

  **Must NOT do**:
  - Don't expose email in member list (use display_name)
  - Don't allow self-role changes to owner

  **Recommended Agent Profile**:
  - **Category**: `deep`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (Wave 1)
  - **Blocks**: Tasks 2, 4, 11
  - **Blocked By**: v1-auth completion

  **References**:
  - v1-auth.md Task 8 — Role system foundation

  **Acceptance Criteria**:
  - [ ] Projects can be shared
  - [ ] Roles enforced
  - [ ] Owner protections work

  **QA Scenarios**:
  ```
  Scenario: Share project and verify access
    Tool: Bash (cargo test)
    Steps:
      1. User A creates project
      2. User A shares with User B as member
      3. User B accesses project
      4. Verify User B has member permissions
    Expected Result: Sharing works
    Evidence: .sisyphus/evidence/task-1-share.txt
  ```

  **Commit**: YES
  - Message: `feat(collab): add project sharing model`
  - Files: `db/collaboration.rs`

- [ ] 2. Invite System

  **What to do**:
  - Create invites table:
    ```sql
    CREATE TABLE project_invites (
        id TEXT PRIMARY KEY,
        project_id TEXT NOT NULL REFERENCES projects(id),
        inviter_id TEXT NOT NULL REFERENCES users(id),
        invitee_email TEXT NOT NULL,
        role TEXT NOT NULL,
        token TEXT NOT NULL UNIQUE,
        created_at TEXT NOT NULL,
        expires_at TEXT NOT NULL,
        accepted_at TEXT,
        declined_at TEXT
    );
    CREATE INDEX idx_invites_token ON project_invites(token);
    CREATE INDEX idx_invites_email ON project_invites(invitee_email);
    ```
  - Implement invite operations:
    ```rust
    pub async fn create_invite(
        project_id: &str,
        inviter_id: &str,
        invitee_email: &str,
        role: ProjectRole,
    ) -> Result<Invite, CollabError>
    
    pub async fn accept_invite(token: &str, user_id: &str) -> Result<ProjectMember, CollabError>
    pub async fn decline_invite(token: &str, user_id: &str) -> Result<(), CollabError>
    pub async fn get_pending_invites(email: &str) -> Result<Vec<Invite>, CollabError>
    pub async fn revoke_invite(invite_id: &str, inviter_id: &str) -> Result<(), CollabError>
    ```
  - Invite token: URL-safe, 32 bytes
  - Invite expiry: 7 days
  - Can accept with different email (if logged in)

  **Must NOT do**:
  - Don't allow duplicate invites (same project + email)
  - Don't reveal project details in invite without auth

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 1, 3)
  - **Blocks**: Tasks 11, 13
  - **Blocked By**: Task 1

  **Acceptance Criteria**:
  - [ ] Invites created with tokens
  - [ ] Acceptance adds member
  - [ ] Expired invites rejected

  **QA Scenarios**:
  ```
  Scenario: Invite acceptance flow
    Tool: Bash (cargo test)
    Steps:
      1. Create invite for email
      2. User with that email accepts
      3. Verify user is now project member
    Expected Result: Invite converts to membership
    Evidence: .sisyphus/evidence/task-2-invite.txt
  ```

  **Commit**: NO (groups with Wave 1)

- [ ] 3. Activity Logging

  **What to do**:
  - Create activity_log table:
    ```sql
    CREATE TABLE activity_log (
        id TEXT PRIMARY KEY,
        project_id TEXT NOT NULL REFERENCES projects(id),
        user_id TEXT NOT NULL REFERENCES users(id),
        action_type TEXT NOT NULL,
        action_details TEXT,  -- JSON
        created_at TEXT NOT NULL
    );
    CREATE INDEX idx_activity_project ON activity_log(project_id, created_at DESC);
    ```
  - Implement activity logging:
    ```rust
    pub enum ActivityAction {
        ProjectCreated,
        MemberAdded { member_id: String, role: ProjectRole },
        MemberRemoved { member_id: String },
        RoleChanged { member_id: String, old_role: ProjectRole, new_role: ProjectRole },
        AgentTaskStarted { task_description: String },
        AgentTaskCompleted { task_description: String, outcome: TaskOutcome },
        FileChanged { path: String, change_type: ChangeType },
        HandoffCreated { to_user_id: Option<String> },
        HandoffAccepted { by_user_id: String },
        CheckpointCreated { checkpoint_id: String },
        RollbackPerformed { checkpoint_id: String },
    }
    
    pub async fn log_activity(
        project_id: &str,
        user_id: &str,
        action: ActivityAction,
    ) -> Result<Activity, CollabError>
    
    pub async fn get_activity_feed(
        project_id: &str,
        limit: usize,
        before: Option<DateTime<Utc>>,
    ) -> Result<Vec<Activity>, CollabError>
    ```
  - Include in all relevant operations (auto-log)
  - Pagination for large feeds

  **Must NOT do**:
  - Don't log sensitive data (API keys, passwords)
  - Don't log every file read (only writes)

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 1, 2)
  - **Blocks**: Tasks 11, 13
  - **Blocked By**: Task 1

  **Acceptance Criteria**:
  - [ ] Activities logged automatically
  - [ ] Feed retrieval works
  - [ ] Pagination works

  **QA Scenarios**:
  ```
  Scenario: Activity feed shows recent actions
    Tool: Bash (cargo test)
    Steps:
      1. Perform several actions on project
      2. Get activity feed
      3. Verify actions appear in order
    Expected Result: Feed accurate and ordered
    Evidence: .sisyphus/evidence/task-3-activity.txt
  ```

  **Commit**: YES (Wave 1)
  - Message: `feat(collab): add invite system and activity logging`
  - Files: `db/collaboration.rs`, `db/activity.rs`

- [ ] 4. Handoff Creation

  **What to do**:
  - Create handoffs table:
    ```sql
    CREATE TABLE handoffs (
        id TEXT PRIMARY KEY,
        project_id TEXT NOT NULL REFERENCES projects(id),
        creator_id TEXT NOT NULL REFERENCES users(id),
        assignee_id TEXT REFERENCES users(id),  -- NULL = anyone on project
        title TEXT NOT NULL,
        description TEXT,
        context_snapshot TEXT NOT NULL,  -- JSON with full context
        priority TEXT NOT NULL DEFAULT 'normal',
        status TEXT NOT NULL DEFAULT 'pending',
        created_at TEXT NOT NULL,
        accepted_at TEXT,
        accepted_by TEXT REFERENCES users(id),
        completed_at TEXT
    );
    CREATE INDEX idx_handoffs_project ON handoffs(project_id, status);
    CREATE INDEX idx_handoffs_assignee ON handoffs(assignee_id, status);
    ```
  - Implement handoff creation:
    ```rust
    pub struct HandoffContext {
        pub conversation_summary: String,
        pub recent_messages: Vec<Message>,  // Last 20
        pub memory_snapshot: String,        // Current memory file
        pub active_tasks: Vec<TaskSummary>,
        pub open_questions: Vec<String>,
        pub suggested_next_steps: Vec<String>,
    }
    
    pub async fn create_handoff(
        project_id: &str,
        creator_id: &str,
        request: CreateHandoffRequest,
    ) -> Result<Handoff, CollabError>
    ```
  - Auto-generate context from current state
  - AI-assisted summary of conversation

  **Must NOT do**:
  - Don't include API keys in context
  - Don't include full conversation (summarize)

  **Recommended Agent Profile**:
  - **Category**: `deep`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (Wave 2)
  - **Blocks**: Tasks 5, 6, 11
  - **Blocked By**: Task 1

  **References**:
  - v1-memory.md — Memory file format

  **Acceptance Criteria**:
  - [ ] Handoffs created with context
  - [ ] Context captures conversation state
  - [ ] Assignee can be specified or open

  **QA Scenarios**:
  ```
  Scenario: Create handoff with context
    Tool: Bash (cargo test)
    Steps:
      1. Have conversation with agent
      2. Create handoff
      3. Verify context includes recent messages
      4. Verify memory snapshot included
    Expected Result: Full context captured
    Evidence: .sisyphus/evidence/task-4-handoff-create.txt
  ```

  **Commit**: NO (groups with Wave 2)

- [ ] 5. Handoff Acceptance + Context Restore

  **What to do**:
  - Implement handoff acceptance:
    ```rust
    pub async fn accept_handoff(
        handoff_id: &str,
        user_id: &str,
    ) -> Result<HandoffContext, CollabError>
    
    pub async fn restore_handoff_context(
        project_id: &str,
        handoff_id: &str,
        user_id: &str,
    ) -> Result<(), CollabError>
    ```
  - On acceptance:
    1. Load context snapshot
    2. Restore memory file (merge or replace, configurable)
    3. Create new conversation with context injected
    4. Mark handoff as accepted
  - Show handoff context to new user:
    - Summary of what was done
    - Open questions to address
    - Suggested next steps

  **Must NOT do**:
  - Don't overwrite existing work without confirmation
  - Don't allow accepting completed handoffs

  **Recommended Agent Profile**:
  - **Category**: `deep`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 4, 6)
  - **Blocks**: Tasks 11, 13
  - **Blocked By**: Task 4

  **Acceptance Criteria**:
  - [ ] Handoffs accepted correctly
  - [ ] Context restored to new session
  - [ ] Previous work preserved

  **QA Scenarios**:
  ```
  Scenario: Accept handoff and continue work
    Tool: Bash (cargo test)
    Steps:
      1. User A creates handoff
      2. User B accepts handoff
      3. Verify User B sees context
      4. User B continues work
    Expected Result: Seamless handoff
    Evidence: .sisyphus/evidence/task-5-handoff-accept.txt
  ```

  **Commit**: NO (groups with Wave 2)

- [ ] 6. Handoff Notifications

  **What to do**:
  - Add notification preferences to users:
    ```sql
    ALTER TABLE users ADD COLUMN notification_prefs TEXT;  -- JSON
    ```
  - Implement notification system:
    ```rust
    pub enum NotificationType {
        HandoffAssigned { handoff_id: String, project_name: String },
        HandoffAccepted { handoff_id: String, accepted_by: String },
        HandoffCompleted { handoff_id: String },
        InviteReceived { project_name: String },
        MentionedInActivity { project_id: String, activity_id: String },
    }
    
    pub async fn send_notification(
        user_id: &str,
        notification: NotificationType,
    ) -> Result<(), CollabError>
    
    pub async fn get_notifications(
        user_id: &str,
        unread_only: bool,
    ) -> Result<Vec<Notification>, CollabError>
    ```
  - WebSocket push for real-time
  - Email notifications (if email configured)
  - In-app notification list

  **Must NOT do**:
  - Don't send email without user consent
  - Don't spam notifications (debounce)

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 4, 5)
  - **Blocks**: Task 11
  - **Blocked By**: Task 4

  **Acceptance Criteria**:
  - [ ] Notifications created on events
  - [ ] WebSocket push works
  - [ ] Notification list retrievable

  **QA Scenarios**:
  ```
  Scenario: Handoff notification delivered
    Tool: Bash (cargo test)
    Steps:
      1. Create handoff assigned to User B
      2. Check User B's notifications
      3. Verify handoff notification present
    Expected Result: Notification delivered
    Evidence: .sisyphus/evidence/task-6-notification.txt
  ```

  **Commit**: YES (Wave 2)
  - Message: `feat(collab): add handoff system with notifications`
  - Files: `db/handoffs.rs`, `db/notifications.rs`

- [ ] 7. Organization Model

  **What to do**:
  - Create organizations tables:
    ```sql
    CREATE TABLE organizations (
        id TEXT PRIMARY KEY,
        name TEXT NOT NULL,
        slug TEXT NOT NULL UNIQUE,
        owner_id TEXT NOT NULL REFERENCES users(id),
        created_at TEXT NOT NULL,
        settings TEXT  -- JSON
    );
    CREATE INDEX idx_orgs_slug ON organizations(slug);
    
    CREATE TABLE organization_members (
        id TEXT PRIMARY KEY,
        org_id TEXT NOT NULL REFERENCES organizations(id),
        user_id TEXT NOT NULL REFERENCES users(id),
        role TEXT NOT NULL,  -- owner, admin, member
        created_at TEXT NOT NULL,
        UNIQUE(org_id, user_id)
    );
    CREATE INDEX idx_org_members_org ON organization_members(org_id);
    CREATE INDEX idx_org_members_user ON organization_members(user_id);
    ```
  - Implement org operations:
    ```rust
    pub struct Organization {
        pub id: OrgId,
        pub name: String,
        pub slug: String,
        pub owner_id: String,
        pub settings: OrgSettings,
    }
    
    pub async fn create_organization(
        name: &str,
        owner_id: &str,
    ) -> Result<Organization, CollabError>
    
    pub async fn get_user_organizations(user_id: &str) -> Result<Vec<Organization>, CollabError>
    ```
  - Projects can belong to org OR be personal
  - Org admins can access all org projects

  **Must NOT do**:
  - Don't allow duplicate slugs
  - Don't allow org deletion with active projects

  **Recommended Agent Profile**:
  - **Category**: `deep`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (Wave 3)
  - **Blocks**: Tasks 8, 9, 10, 12
  - **Blocked By**: v1-auth completion

  **Acceptance Criteria**:
  - [ ] Orgs created with owner
  - [ ] Slug uniqueness enforced
  - [ ] User can list their orgs

  **QA Scenarios**:
  ```
  Scenario: Create and retrieve organization
    Tool: Bash (cargo test)
    Steps:
      1. Create org "Acme Corp"
      2. Verify slug "acme-corp" created
      3. List user's orgs
      4. Verify org appears
    Expected Result: Org management works
    Evidence: .sisyphus/evidence/task-7-org.txt
  ```

  **Commit**: NO (groups with Wave 3)

- [ ] 8. Org Member Management

  **What to do**:
  - Implement member operations:
    ```rust
    pub enum OrgRole {
        Owner,   // Full control, billing
        Admin,   // Manage members, create projects
        Member,  // Work on projects
    }
    
    pub async fn invite_to_org(
        org_id: &str,
        inviter_id: &str,
        invitee_email: &str,
        role: OrgRole,
    ) -> Result<OrgInvite, CollabError>
    
    pub async fn accept_org_invite(token: &str, user_id: &str) -> Result<(), CollabError>
    pub async fn get_org_members(org_id: &str) -> Result<Vec<OrgMember>, CollabError>
    pub async fn update_org_member_role(org_id: &str, user_id: &str, role: OrgRole) -> Result<(), CollabError>
    pub async fn remove_from_org(org_id: &str, user_id: &str) -> Result<(), CollabError>
    ```
  - Org membership grants access to all org projects
  - Org role can override project role (org admin > project member)

  **Must NOT do**:
  - Don't allow removing last owner
  - Don't expose member emails to non-admins

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 7, 9, 10)
  - **Blocks**: Tasks 12, 13
  - **Blocked By**: Task 7

  **Acceptance Criteria**:
  - [ ] Members invited and added
  - [ ] Roles enforced
  - [ ] Org access grants project access

  **QA Scenarios**:
  ```
  Scenario: Org admin accesses org project
    Tool: Bash (cargo test)
    Steps:
      1. Create org with admin user
      2. Create project in org
      3. Add member to org (not project)
      4. Verify org member can access project
    Expected Result: Org role grants access
    Evidence: .sisyphus/evidence/task-8-org-access.txt
  ```

  **Commit**: NO (groups with Wave 3)

- [ ] 9. Org-Level Configs

  **What to do**:
  - Add org settings structure:
    ```rust
    pub struct OrgSettings {
        pub default_model_config: Option<ModelConfig>,
        pub allowed_models: Option<Vec<String>>,  // Restrict to these
        pub default_sandbox_limits: Option<SandboxLimits>,
        pub default_gate_config: Option<GateConfig>,
        pub shared_templates: Vec<String>,  // Template IDs
    }
    ```
  - Implement config inheritance:
    - Project config inherits from org config
    - Project can override org defaults
    - Some settings org-enforced (can't override)
  - Org settings UI in dashboard

  **Must NOT do**:
  - Don't store secrets in org settings
  - Don't allow members to change org settings

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 7, 8, 10)
  - **Blocks**: Task 12
  - **Blocked By**: Task 7

  **Acceptance Criteria**:
  - [ ] Org settings saved and retrieved
  - [ ] Project inherits org defaults
  - [ ] Overrides work

  **QA Scenarios**:
  ```
  Scenario: Project inherits org model config
    Tool: Bash (cargo test)
    Steps:
      1. Set org default model to "claude-sonnet"
      2. Create project in org
      3. Verify project uses "claude-sonnet"
    Expected Result: Config inherited
    Evidence: .sisyphus/evidence/task-9-org-config.txt
  ```

  **Commit**: NO (groups with Wave 3)

- [ ] 10. Org API Key Pools

  **What to do**:
  - Create org_api_keys table:
    ```sql
    CREATE TABLE org_api_keys (
        id TEXT PRIMARY KEY,
        org_id TEXT NOT NULL REFERENCES organizations(id),
        provider TEXT NOT NULL,  -- anthropic, openai, etc.
        key_encrypted TEXT NOT NULL,
        name TEXT NOT NULL,
        created_by TEXT NOT NULL REFERENCES users(id),
        created_at TEXT NOT NULL,
        last_used_at TEXT,
        usage_limit_monthly REAL,  -- Optional spending cap
        usage_current_month REAL DEFAULT 0
    );
    ```
  - Implement key pool:
    ```rust
    pub async fn add_org_api_key(
        org_id: &str,
        provider: &str,
        api_key: &str,  // Encrypted before storage
        name: &str,
    ) -> Result<OrgApiKey, CollabError>
    
    pub async fn get_org_api_key(
        org_id: &str,
        provider: &str,
    ) -> Result<Option<String>, CollabError>  // Decrypted
    
    pub async fn list_org_api_keys(org_id: &str) -> Result<Vec<OrgApiKeySummary>, CollabError>
    ```
  - Keys encrypted at rest (AES-256-GCM)
  - Usage tracking per key
  - Optional monthly spending limits

  **Must NOT do**:
  - Don't store keys unencrypted
  - Don't return full keys in list (only prefix)

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 7, 8, 9)
  - **Blocks**: Task 12
  - **Blocked By**: Tasks 7, v1-auth

  **References**:
  - v1-costs.md — Usage tracking

  **Acceptance Criteria**:
  - [ ] Keys stored encrypted
  - [ ] Keys retrieved for provider
  - [ ] Usage tracked

  **QA Scenarios**:
  ```
  Scenario: Org API key usage
    Tool: Bash (cargo test)
    Steps:
      1. Add Anthropic key to org
      2. Project in org makes API call
      3. Verify org key used
      4. Verify usage incremented
    Expected Result: Org key shared
    Evidence: .sisyphus/evidence/task-10-org-key.txt
  ```

  **Commit**: YES (Wave 3)
  - Message: `feat(collab): add organization management and shared configs`
  - Files: `db/organization.rs`, `db/org_api_keys.rs`

- [ ] 11. Collaboration API Endpoints

  **What to do**:
  - Create `crates/cuttlefish-api/src/collab_routes.rs`:
    - `GET /api/projects/:id/members` — List members
    - `POST /api/projects/:id/members` — Add member
    - `PUT /api/projects/:id/members/:user_id` — Update role
    - `DELETE /api/projects/:id/members/:user_id` — Remove member
    - `POST /api/projects/:id/invites` — Create invite
    - `GET /api/invites/pending` — List pending invites for user
    - `POST /api/invites/:token/accept` — Accept invite
    - `POST /api/invites/:token/decline` — Decline invite
    - `GET /api/projects/:id/activity` — Activity feed
    - `GET /api/projects/:id/handoffs` — List handoffs
    - `POST /api/projects/:id/handoffs` — Create handoff
    - `POST /api/handoffs/:id/accept` — Accept handoff
    - `GET /api/notifications` — User notifications
    - `POST /api/notifications/:id/read` — Mark read

  **Must NOT do**:
  - Don't expose other users' emails
  - Don't allow unauthorized role changes

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Task 12)
  - **Blocks**: Task 13
  - **Blocked By**: Tasks 1-6

  **Acceptance Criteria**:
  - [ ] All endpoints work
  - [ ] Permissions checked
  - [ ] Responses well-formed

  **QA Scenarios**:
  ```
  Scenario: Collaboration API flow
    Tool: Bash (curl)
    Steps:
      1. POST /api/projects/test/invites
      2. POST /api/invites/{token}/accept
      3. GET /api/projects/test/members
      4. Verify new member listed
    Expected Result: API flow works
    Evidence: .sisyphus/evidence/task-11-collab-api.txt
  ```

  **Commit**: YES
  - Message: `feat(api): add collaboration endpoints`
  - Files: `api/collab_routes.rs`

- [ ] 12. Organization API Endpoints

  **What to do**:
  - Create `crates/cuttlefish-api/src/org_routes.rs`:
    - `GET /api/organizations` — List user's orgs
    - `POST /api/organizations` — Create org
    - `GET /api/organizations/:id` — Get org details
    - `PUT /api/organizations/:id` — Update org
    - `DELETE /api/organizations/:id` — Delete org
    - `GET /api/organizations/:id/members` — List members
    - `POST /api/organizations/:id/members` — Invite member
    - `PUT /api/organizations/:id/members/:user_id` — Update role
    - `DELETE /api/organizations/:id/members/:user_id` — Remove member
    - `GET /api/organizations/:id/settings` — Get settings
    - `PUT /api/organizations/:id/settings` — Update settings
    - `GET /api/organizations/:id/api-keys` — List API keys
    - `POST /api/organizations/:id/api-keys` — Add API key
    - `DELETE /api/organizations/:id/api-keys/:key_id` — Remove key

  **Must NOT do**:
  - Don't return full API keys
  - Don't allow non-owners to delete org

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Task 11)
  - **Blocks**: Task 13
  - **Blocked By**: Tasks 7-10

  **Acceptance Criteria**:
  - [ ] All endpoints work
  - [ ] Owner permissions enforced
  - [ ] Settings CRUD works

  **QA Scenarios**:
  ```
  Scenario: Organization API flow
    Tool: Bash (curl)
    Steps:
      1. POST /api/organizations
      2. GET /api/organizations/{id}
      3. PUT /api/organizations/{id}/settings
      4. Verify settings saved
    Expected Result: Org API works
    Evidence: .sisyphus/evidence/task-12-org-api.txt
  ```

  **Commit**: YES
  - Message: `feat(api): add organization endpoints`
  - Files: `api/org_routes.rs`

- [ ] 13. WebUI Collaboration Components

  **What to do**:
  - Create collaboration UI components:
    - `MembersPanel.vue` — Project members list with role badges
    - `InviteModal.vue` — Invite user by email
    - `ActivityFeed.vue` — Timeline of project activity
    - `HandoffCard.vue` — Handoff summary with accept button
    - `HandoffModal.vue` — Create handoff form
    - `NotificationBell.vue` — Notification dropdown
    - `OrgSwitcher.vue` — Organization context selector
  - Add to project dashboard:
    - Members tab showing collaborators
    - Activity tab with feed
    - Handoffs section
  - Add organization pages:
    - `pages/org/[slug]/index.vue` — Org dashboard
    - `pages/org/[slug]/members.vue` — Member management
    - `pages/org/[slug]/settings.vue` — Org settings

  **Must NOT do**:
  - Don't show email addresses (use display names)
  - Don't allow UI role escalation

  **Recommended Agent Profile**:
  - **Category**: `visual-engineering`
  - **Skills**: [`frontend-ui-ux`]

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Blocks**: F1-F4
  - **Blocked By**: Tasks 11, 12

  **Acceptance Criteria**:
  - [ ] Members panel shows collaborators
  - [ ] Invite flow works
  - [ ] Activity feed populated
  - [ ] Handoff creation works

  **QA Scenarios**:
  ```
  Scenario: Invite collaborator via UI
    Tool: Playwright
    Steps:
      1. Navigate to project
      2. Click "Share" button
      3. Enter email, select role
      4. Click "Send Invite"
      5. Verify success message
    Expected Result: Invite sent via UI
    Evidence: .sisyphus/evidence/task-13-invite-ui.png
  ```

  **Commit**: YES
  - Message: `feat(web): add collaboration UI components`
  - Files: `components/*.vue`, `pages/org/*`

- [ ] 14. Real-Time Presence (Optional)

  **What to do**:
  - Add presence tracking:
    ```rust
    pub struct UserPresence {
        pub user_id: String,
        pub project_id: String,
        pub status: PresenceStatus,
        pub last_active: DateTime<Utc>,
    }
    
    pub enum PresenceStatus {
        Active,      // Currently interacting
        Idle,        // Connected but inactive
        Away,        // Disconnected recently
    }
    ```
  - WebSocket presence updates:
    - Send heartbeat every 30 seconds
    - Broadcast presence changes to project members
  - UI indicator: show avatars of active users
  - "Currently editing" indicator (file-level, not real-time cursor)

  **Must NOT do**:
  - Don't implement cursor sharing (V2)
  - Don't track when user is not in project

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 11-13)
  - **Blocks**: F1-F4
  - **Blocked By**: Task 1

  **Acceptance Criteria**:
  - [ ] Presence tracked
  - [ ] UI shows active users
  - [ ] Status transitions correctly

  **QA Scenarios**:
  ```
  Scenario: Presence indicator shows collaborator
    Tool: Playwright
    Steps:
      1. User A opens project
      2. User B opens same project
      3. Verify User A sees User B's presence
    Expected Result: Presence visible
    Evidence: .sisyphus/evidence/task-14-presence.png
  ```

  **Commit**: YES (Wave 4)
  - Message: `feat(collab): add real-time presence indicators`
  - Files: `api/websocket.rs`, `web/components/PresenceIndicator.vue`

---

## Final Verification Wave

- [ ] F1. **Plan Compliance Audit** — `oracle`
  Verify sharing, invites, handoffs, orgs, activity all implemented. Check no email exposure.
  Output: `Must Have [N/N] | Must NOT Have [N/N] | Tasks [N/N] | VERDICT: APPROVE/REJECT`

- [ ] F2. **Code Quality Review** — `unspecified-high`
  Run `cargo clippy --workspace -- -D warnings` + tests. Review for permission checks, data isolation.
  Output: `Build [PASS/FAIL] | Lint [PASS/FAIL] | Tests [N pass/N fail] | VERDICT`

- [ ] F3. **Collaboration E2E QA** — `unspecified-high`
  Full workflow: User A creates project, invites User B, User B accepts, creates handoff, User A accepts handoff. Test org flow: create org, add member, create org project, verify shared access.
  Output: `Sharing [PASS/FAIL] | Handoffs [PASS/FAIL] | Orgs [PASS/FAIL] | VERDICT`

- [ ] F4. **Scope Fidelity Check** — `deep`
  Verify no real-time cursor sharing (V1), no cross-org data leakage, org keys encrypted.
  Output: `Tasks [N/N compliant] | Scope [CLEAN/N violations] | VERDICT`

---

## Commit Strategy

| Wave | Commit |
|------|--------|
| 1 | `feat(collab): add project sharing model` |
| 1 | `feat(collab): add invite system and activity logging` |
| 2 | `feat(collab): add handoff system with notifications` |
| 3 | `feat(collab): add organization management and shared configs` |
| 4 | `feat(api): add collaboration endpoints` |
| 4 | `feat(api): add organization endpoints` |
| 4 | `feat(web): add collaboration UI components` |
| 4 | `feat(collab): add real-time presence indicators` |

---

## Success Criteria

### Verification Commands
```bash
cargo test --workspace collab  # All tests pass
cargo clippy --workspace -- -D warnings  # Clean
curl localhost:8080/api/projects/test/members  # Returns member list
curl localhost:8080/api/organizations  # Returns user's orgs
```

### Final Checklist
- [ ] Projects shareable with role-based access
- [ ] Invites work via email
- [ ] Handoffs preserve full context
- [ ] Organizations manage shared resources
- [ ] Activity feed shows who did what
- [ ] No email addresses exposed
- [ ] No cross-org data leakage
- [ ] Org API keys encrypted
