# V1 Authentication System — Multi-User Auth, Sessions & API Keys

## TL;DR

> **Quick Summary**: Build a complete authentication system with user accounts, session management, API key generation, and role-based access control to support multi-user collaboration.
> 
> **Deliverables**:
> - User registration and login (email/password)
> - Session management with JWT tokens
> - API key generation for programmatic access
> - Role system (owner, admin, member, viewer)
> - Password reset flow
> - Database tables for users, sessions, API keys
> 
> **Estimated Effort**: Large (4-5 days)
> **Parallel Execution**: YES - 4 waves
> **Critical Path**: Task 1 (Users) → Task 2 (Sessions) → Task 3 (API Keys) → Task 4 (Roles) → Task 5 (Routes)

---

## Context

### Original Request (From Product Spec)
- Multi-user collaboration with roles and permissions
- API key management for BYOK (Bring Your Own Key) model
- Async handoffs between users
- Organization-level shared configurations

### Problem Statement
Cuttlefish currently uses a simple `CUTTLEFISH_API_KEY` for authentication. For V1 multi-user support, we need:
- Individual user accounts
- Project-level access control
- API keys for external integrations
- Session management for web/TUI clients

### Design Philosophy
- **Simple but secure**: No OAuth complexity for V1 (just email/password)
- **API-first**: All auth flows available via REST API
- **Role-based**: Clear permission boundaries
- **Key management**: Users control their own API keys

---

## Work Objectives

### Core Objective
Build a complete authentication and authorization system that enables secure multi-user access to Cuttlefish with role-based permissions.

### Concrete Deliverables
- `crates/cuttlefish-db/src/auth.rs` — Auth-related database operations
- `crates/cuttlefish-api/src/auth_routes.rs` — Auth API endpoints
- `crates/cuttlefish-api/src/middleware/auth.rs` — Auth middleware
- `crates/cuttlefish-core/src/auth.rs` — Auth types and utilities
- Database migrations for users, sessions, API keys, roles
- Password hashing with Argon2
- JWT token generation and validation
- WebUI login/register pages

### Definition of Done
- [ ] Users can register with email/password
- [ ] Users can login and receive JWT token
- [ ] API keys can be created/revoked
- [ ] Roles control access to projects
- [ ] Password reset flow works
- [ ] `cargo test --workspace auth` passes
- [ ] `cargo clippy --workspace -- -D warnings` clean

### Must Have
- User registration with email validation format
- Password hashing (Argon2id)
- JWT tokens with expiration (24h default)
- Refresh token rotation
- API key generation (prefixed: `cfish_`)
- API key scopes (read, write, admin)
- Role hierarchy: owner > admin > member > viewer
- Password reset via email token
- Session invalidation (logout)

### Must NOT Have (Guardrails)
- No unsafe code
- No `unwrap()` — use `?` or `expect("reason")`
- No plaintext password storage (Argon2 required)
- No hardcoded secrets (all via env vars)
- No OAuth/social login in V1 (future feature)
- No SMS/phone auth in V1
- No breaking existing API key flow (backwards compatible)

---

## Verification Strategy

> **ZERO HUMAN INTERVENTION** — ALL verification is agent-executed.

### Test Decision
- **Infrastructure exists**: YES (database, API)
- **Automated tests**: YES (TDD)
- **Framework**: `#[tokio::test]` for async

### QA Policy
Every task includes agent-executed QA scenarios.
Evidence saved to `.sisyphus/evidence/task-{N}-{scenario-slug}.{ext}`.

---

## Execution Strategy

### Parallel Execution Waves

```
Wave 1 (Foundation — users + passwords):
├── Task 1: User model and database [deep]
├── Task 2: Password hashing (Argon2) [unspecified-high]
└── Task 3: JWT token generation [unspecified-high]

Wave 2 (Sessions — login/logout):
├── Task 4: Session management [deep]
├── Task 5: Refresh token rotation [unspecified-high]
└── Task 6: Auth middleware [deep]

Wave 3 (API Keys + Roles):
├── Task 7: API key generation [unspecified-high]
├── Task 8: Role system [deep]
└── Task 9: Password reset flow [unspecified-high]

Wave 4 (Integration — routes + UI):
├── Task 10: Auth API endpoints [unspecified-high]
├── Task 11: WebUI auth pages [visual-engineering]
└── Task 12: Backwards compatibility layer [quick]

Wave FINAL (Verification):
├── Task F1: Plan compliance audit (oracle)
├── Task F2: Code quality + security review (unspecified-high)
├── Task F3: Auth E2E QA (unspecified-high)
└── Task F4: Scope fidelity check (deep)
-> Present results -> Get explicit user okay

Critical Path: Task 1 → Task 2 → Task 4 → Task 6 → Task 10 → F1-F4 → user okay
Parallel Speedup: ~55% faster than sequential
Max Concurrent: 3 (all waves)
```

### Dependency Matrix

| Task | Depends On | Blocks | Wave |
|------|------------|--------|------|
| 1 | — | 2, 4, 7, 8 | 1 |
| 2 | 1 | 4, 9 | 1 |
| 3 | — | 4, 5, 6 | 1 |
| 4 | 1, 2, 3 | 5, 6, 10 | 2 |
| 5 | 3, 4 | 10 | 2 |
| 6 | 3, 4 | 10, 12 | 2 |
| 7 | 1 | 10, 12 | 3 |
| 8 | 1 | 10 | 3 |
| 9 | 1, 2 | 10 | 3 |
| 10 | 4, 5, 6, 7, 8, 9 | 11 | 4 |
| 11 | 10 | F1-F4 | 4 |
| 12 | 6, 7 | F1-F4 | 4 |

### Agent Dispatch Summary

- **Wave 1**: 3 tasks — T1 → `deep`, T2-T3 → `unspecified-high`
- **Wave 2**: 3 tasks — T4, T6 → `deep`, T5 → `unspecified-high`
- **Wave 3**: 3 tasks — T7, T9 → `unspecified-high`, T8 → `deep`
- **Wave 4**: 3 tasks — T10 → `unspecified-high`, T11 → `visual-engineering`, T12 → `quick`
- **FINAL**: 4 tasks — F1 → `oracle`, F2-F3 → `unspecified-high`, F4 → `deep`

---

## TODOs

- [ ] 1. User Model and Database

  **What to do**:
  - Create database migration for users table:
    ```sql
    CREATE TABLE users (
        id TEXT PRIMARY KEY,
        email TEXT NOT NULL UNIQUE,
        password_hash TEXT NOT NULL,
        display_name TEXT,
        created_at TEXT NOT NULL,
        updated_at TEXT NOT NULL,
        email_verified_at TEXT,
        last_login_at TEXT,
        is_active INTEGER NOT NULL DEFAULT 1
    );
    CREATE INDEX idx_users_email ON users(email);
    ```
  - Create `crates/cuttlefish-core/src/auth/user.rs`:
    ```rust
    pub struct User {
        pub id: UserId,
        pub email: String,
        pub display_name: Option<String>,
        pub created_at: DateTime<Utc>,
        pub email_verified: bool,
        pub is_active: bool,
    }
    
    pub struct CreateUserRequest {
        pub email: String,
        pub password: String,
        pub display_name: Option<String>,
    }
    ```
  - Add CRUD operations to `crates/cuttlefish-db/src/auth.rs`
  - Email validation (format check, not delivery)

  **Must NOT do**:
  - Don't store plaintext passwords
  - Don't allow duplicate emails

  **Recommended Agent Profile**:
  - **Category**: `deep`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (Wave 1 start)
  - **Blocks**: Tasks 2, 4, 7, 8
  - **Blocked By**: None

  **References**:
  - `crates/cuttlefish-db/src/lib.rs` — Existing migration pattern
  - `crates/cuttlefish-db/src/models.rs` — Model patterns

  **Acceptance Criteria**:
  - [ ] Users table created on migration
  - [ ] CRUD operations work
  - [ ] Email uniqueness enforced

  **QA Scenarios**:
  ```
  Scenario: Create user with valid email
    Tool: Bash (cargo test)
    Steps:
      1. Create user with "test@example.com"
      2. Verify user ID returned
      3. Retrieve user by email
      4. Verify fields match
    Expected Result: User created and retrievable
    Evidence: .sisyphus/evidence/task-1-create-user.txt
  ```

  **Commit**: YES
  - Message: `feat(auth): add user model and database`
  - Files: `db/auth.rs`, `core/auth/user.rs`, migrations

- [ ] 2. Password Hashing (Argon2)

  **What to do**:
  - Add `argon2` crate to `crates/cuttlefish-core/Cargo.toml`
  - Create `crates/cuttlefish-core/src/auth/password.rs`:
    ```rust
    pub fn hash_password(password: &str) -> Result<String, AuthError>
    pub fn verify_password(password: &str, hash: &str) -> Result<bool, AuthError>
    ```
  - Use Argon2id variant (recommended)
  - Configure params: memory=65536KB, iterations=3, parallelism=4
  - Add password strength validation:
    - Minimum 8 characters
    - At least one uppercase, lowercase, digit
  - Return clear errors for weak passwords

  **Must NOT do**:
  - Don't use bcrypt or SHA (Argon2 required)
  - Don't store password params separately (embedded in hash)

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 1, 3)
  - **Blocks**: Tasks 4, 9
  - **Blocked By**: Task 1

  **References**:
  - `argon2` crate: https://docs.rs/argon2
  - OWASP password guidelines

  **Acceptance Criteria**:
  - [ ] Passwords hash correctly
  - [ ] Verification works
  - [ ] Weak passwords rejected

  **QA Scenarios**:
  ```
  Scenario: Password hash and verify
    Tool: Bash (cargo test)
    Steps:
      1. Hash "SecurePass123!"
      2. Verify same password matches
      3. Verify wrong password fails
    Expected Result: Correct verification behavior
    Evidence: .sisyphus/evidence/task-2-password.txt
  ```

  **Commit**: NO (groups with Wave 1)

- [ ] 3. JWT Token Generation

  **What to do**:
  - Add `jsonwebtoken` crate (already in workspace)
  - Create `crates/cuttlefish-core/src/auth/jwt.rs`:
    ```rust
    pub struct TokenClaims {
        pub sub: String,        // User ID
        pub exp: i64,           // Expiration timestamp
        pub iat: i64,           // Issued at
        pub token_type: TokenType,
    }
    
    pub enum TokenType {
        Access,   // Short-lived (24h)
        Refresh,  // Long-lived (30d)
    }
    
    pub fn generate_access_token(user_id: &str, secret: &[u8]) -> Result<String, AuthError>
    pub fn generate_refresh_token(user_id: &str, secret: &[u8]) -> Result<String, AuthError>
    pub fn validate_token(token: &str, secret: &[u8]) -> Result<TokenClaims, AuthError>
    ```
  - Use `AUTH_JWT_SECRET` env var for signing
  - Access token: 24 hour expiry
  - Refresh token: 30 day expiry

  **Must NOT do**:
  - Don't hardcode secrets
  - Don't use weak algorithms (use HS256 or RS256)

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 1, 2)
  - **Blocks**: Tasks 4, 5, 6
  - **Blocked By**: None

  **References**:
  - `jsonwebtoken` crate docs
  - Existing JWT in `crates/cuttlefish-tunnel/src/auth.rs`

  **Acceptance Criteria**:
  - [ ] Access tokens generate and validate
  - [ ] Refresh tokens generate and validate
  - [ ] Expired tokens rejected

  **QA Scenarios**:
  ```
  Scenario: JWT generation and validation
    Tool: Bash (cargo test)
    Steps:
      1. Generate access token for user "123"
      2. Validate token
      3. Verify claims.sub == "123"
    Expected Result: Token round-trips correctly
    Evidence: .sisyphus/evidence/task-3-jwt.txt
  ```

  **Commit**: YES (Wave 1)
  - Message: `feat(auth): add password hashing and JWT tokens`
  - Files: `core/auth/password.rs`, `core/auth/jwt.rs`

- [ ] 4. Session Management

  **What to do**:
  - Create sessions table:
    ```sql
    CREATE TABLE sessions (
        id TEXT PRIMARY KEY,
        user_id TEXT NOT NULL REFERENCES users(id),
        refresh_token_hash TEXT NOT NULL,
        user_agent TEXT,
        ip_address TEXT,
        created_at TEXT NOT NULL,
        expires_at TEXT NOT NULL,
        revoked_at TEXT
    );
    CREATE INDEX idx_sessions_user ON sessions(user_id);
    ```
  - Implement session operations:
    ```rust
    pub async fn create_session(user_id: &str, refresh_token: &str, metadata: SessionMetadata) -> Result<Session, AuthError>
    pub async fn get_session(session_id: &str) -> Result<Option<Session>, AuthError>
    pub async fn revoke_session(session_id: &str) -> Result<(), AuthError>
    pub async fn revoke_all_sessions(user_id: &str) -> Result<u64, AuthError>  // Returns count
    pub async fn cleanup_expired_sessions() -> Result<u64, AuthError>
    ```
  - Store hashed refresh token (not plaintext)
  - Session metadata: user agent, IP (for security)

  **Must NOT do**:
  - Don't store plaintext refresh tokens
  - Don't allow unlimited sessions per user (cap at 10)

  **Recommended Agent Profile**:
  - **Category**: `deep`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (after Wave 1)
  - **Blocks**: Tasks 5, 6, 10
  - **Blocked By**: Tasks 1, 2, 3

  **Acceptance Criteria**:
  - [ ] Sessions created on login
  - [ ] Sessions revoked on logout
  - [ ] Expired sessions cleaned up

  **QA Scenarios**:
  ```
  Scenario: Session lifecycle
    Tool: Bash (cargo test)
    Steps:
      1. Create session for user
      2. Verify session exists
      3. Revoke session
      4. Verify session marked revoked
    Expected Result: Full lifecycle works
    Evidence: .sisyphus/evidence/task-4-session.txt
  ```

  **Commit**: NO (groups with Wave 2)

- [ ] 5. Refresh Token Rotation

  **What to do**:
  - Implement token refresh:
    ```rust
    pub async fn refresh_tokens(
        refresh_token: &str,
    ) -> Result<TokenPair, AuthError> {
        // 1. Validate refresh token
        // 2. Find session by token hash
        // 3. Verify session not revoked/expired
        // 4. Generate new access + refresh tokens
        // 5. Update session with new refresh token hash
        // 6. Return both tokens
    }
    
    pub struct TokenPair {
        pub access_token: String,
        pub refresh_token: String,
        pub expires_in: i64,
    }
    ```
  - Implement refresh token reuse detection:
    - If old refresh token used, revoke all sessions (security breach)
  - Single-use refresh tokens (rotated on each refresh)

  **Must NOT do**:
  - Don't allow refresh token reuse (security risk)
  - Don't extend refresh token expiry on rotation

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 4, 6)
  - **Blocks**: Task 10
  - **Blocked By**: Tasks 3, 4

  **Acceptance Criteria**:
  - [ ] Token rotation works
  - [ ] Old refresh tokens rejected
  - [ ] Reuse detection revokes sessions

  **QA Scenarios**:
  ```
  Scenario: Refresh token rotation
    Tool: Bash (cargo test)
    Steps:
      1. Login, get tokens
      2. Refresh with refresh_token
      3. Verify new tokens returned
      4. Try refreshing with old token
      5. Verify rejected
    Expected Result: Rotation enforced
    Evidence: .sisyphus/evidence/task-5-rotation.txt
  ```

  **Commit**: NO (groups with Wave 2)

- [ ] 6. Auth Middleware

  **What to do**:
  - Create `crates/cuttlefish-api/src/middleware/auth.rs`:
    ```rust
    pub async fn auth_middleware(
        State(state): State<AppState>,
        request: Request,
        next: Next,
    ) -> Response {
        // 1. Extract token from Authorization header
        // 2. Or extract API key from X-API-Key header
        // 3. Validate and get user
        // 4. Add user to request extensions
        // 5. Call next
    }
    
    pub fn require_auth() -> impl Layer  // Rejects unauthenticated
    pub fn optional_auth() -> impl Layer  // Allows unauthenticated
    ```
  - Support both JWT and API key auth
  - Extract user info into request extensions
  - Return 401 for invalid/missing auth (on protected routes)

  **Must NOT do**:
  - Don't log tokens (security)
  - Don't cache auth results (always validate)

  **Recommended Agent Profile**:
  - **Category**: `deep`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 4, 5)
  - **Blocks**: Tasks 10, 12
  - **Blocked By**: Tasks 3, 4

  **References**:
  - Axum middleware patterns
  - `crates/cuttlefish-api/src/routes.rs` — Existing route setup

  **Acceptance Criteria**:
  - [ ] JWT auth works
  - [ ] API key auth works
  - [ ] 401 returned for invalid auth

  **QA Scenarios**:
  ```
  Scenario: Auth middleware validates JWT
    Tool: Bash (cargo test)
    Steps:
      1. Request protected route with valid JWT
      2. Verify 200 response
      3. Request with invalid JWT
      4. Verify 401 response
    Expected Result: Middleware validates correctly
    Evidence: .sisyphus/evidence/task-6-middleware.txt
  ```

  **Commit**: YES (Wave 2)
  - Message: `feat(auth): add session management and auth middleware`
  - Files: `db/sessions.rs`, `api/middleware/auth.rs`

- [ ] 7. API Key Generation

  **What to do**:
  - Create api_keys table:
    ```sql
    CREATE TABLE api_keys (
        id TEXT PRIMARY KEY,
        user_id TEXT NOT NULL REFERENCES users(id),
        name TEXT NOT NULL,
        key_hash TEXT NOT NULL,
        key_prefix TEXT NOT NULL,  -- First 8 chars for display
        scopes TEXT NOT NULL,      -- JSON array
        created_at TEXT NOT NULL,
        last_used_at TEXT,
        expires_at TEXT,
        revoked_at TEXT
    );
    CREATE INDEX idx_api_keys_user ON api_keys(user_id);
    CREATE INDEX idx_api_keys_prefix ON api_keys(key_prefix);
    ```
  - Implement API key operations:
    ```rust
    pub fn generate_api_key() -> (String, String)  // Returns (full_key, prefix)
    pub async fn create_api_key(user_id: &str, name: &str, scopes: Vec<Scope>) -> Result<ApiKeyCreated, AuthError>
    pub async fn validate_api_key(key: &str) -> Result<ApiKeyInfo, AuthError>
    pub async fn revoke_api_key(key_id: &str, user_id: &str) -> Result<(), AuthError>
    pub async fn list_api_keys(user_id: &str) -> Result<Vec<ApiKeySummary>, AuthError>
    ```
  - Key format: `cfish_` prefix + 32 random chars
  - Scopes: `read`, `write`, `admin`
  - Return full key only on creation (not stored)

  **Must NOT do**:
  - Don't store full key (only hash)
  - Don't allow scope escalation

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (Wave 3)
  - **Blocks**: Tasks 10, 12
  - **Blocked By**: Task 1

  **Acceptance Criteria**:
  - [ ] Keys generate with prefix
  - [ ] Keys validate correctly
  - [ ] Scopes enforced

  **QA Scenarios**:
  ```
  Scenario: API key creation and validation
    Tool: Bash (cargo test)
    Steps:
      1. Create API key for user
      2. Verify key starts with "cfish_"
      3. Validate key
      4. Verify scopes returned
    Expected Result: Key lifecycle works
    Evidence: .sisyphus/evidence/task-7-apikey.txt
  ```

  **Commit**: NO (groups with Wave 3)

- [ ] 8. Role System

  **What to do**:
  - Create roles tables:
    ```sql
    CREATE TABLE project_members (
        id TEXT PRIMARY KEY,
        project_id TEXT NOT NULL REFERENCES projects(id),
        user_id TEXT NOT NULL REFERENCES users(id),
        role TEXT NOT NULL,  -- owner, admin, member, viewer
        invited_by TEXT REFERENCES users(id),
        created_at TEXT NOT NULL,
        UNIQUE(project_id, user_id)
    );
    CREATE INDEX idx_project_members_project ON project_members(project_id);
    CREATE INDEX idx_project_members_user ON project_members(user_id);
    ```
  - Implement role operations:
    ```rust
    pub enum ProjectRole {
        Owner,   // Full control, can delete project, transfer ownership
        Admin,   // Manage members, configure project
        Member,  // Work on project, create branches
        Viewer,  // Read-only access
    }
    
    pub fn can_perform(role: ProjectRole, action: ProjectAction) -> bool
    pub async fn get_user_role(project_id: &str, user_id: &str) -> Result<Option<ProjectRole>, AuthError>
    pub async fn add_member(project_id: &str, user_id: &str, role: ProjectRole, invited_by: &str) -> Result<(), AuthError>
    pub async fn update_role(project_id: &str, user_id: &str, new_role: ProjectRole) -> Result<(), AuthError>
    pub async fn remove_member(project_id: &str, user_id: &str) -> Result<(), AuthError>
    ```
  - Permission matrix for actions (documented)

  **Must NOT do**:
  - Don't allow removing last owner
  - Don't allow self-promotion to owner

  **Recommended Agent Profile**:
  - **Category**: `deep`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (Wave 3)
  - **Blocks**: Task 10
  - **Blocked By**: Task 1

  **Acceptance Criteria**:
  - [ ] Roles assigned correctly
  - [ ] Permissions enforced
  - [ ] Owner protections work

  **QA Scenarios**:
  ```
  Scenario: Role-based access control
    Tool: Bash (cargo test)
    Steps:
      1. Add user as viewer
      2. Attempt write action
      3. Verify rejected
      4. Upgrade to member
      5. Attempt write action
      6. Verify allowed
    Expected Result: RBAC enforced
    Evidence: .sisyphus/evidence/task-8-roles.txt
  ```

  **Commit**: NO (groups with Wave 3)

- [ ] 9. Password Reset Flow

  **What to do**:
  - Create password_reset_tokens table:
    ```sql
    CREATE TABLE password_reset_tokens (
        id TEXT PRIMARY KEY,
        user_id TEXT NOT NULL REFERENCES users(id),
        token_hash TEXT NOT NULL,
        created_at TEXT NOT NULL,
        expires_at TEXT NOT NULL,
        used_at TEXT
    );
    ```
  - Implement reset flow:
    ```rust
    pub async fn request_password_reset(email: &str) -> Result<(), AuthError>
    // Creates token, would send email (email sending separate)
    
    pub async fn validate_reset_token(token: &str) -> Result<String, AuthError>
    // Returns user_id if valid
    
    pub async fn reset_password(token: &str, new_password: &str) -> Result<(), AuthError>
    // Validates token, updates password, invalidates all sessions
    ```
  - Token format: URL-safe base64, 32 bytes
  - Token expiry: 1 hour
  - Single-use tokens
  - Invalidate all sessions on password change

  **Must NOT do**:
  - Don't reveal if email exists (security)
  - Don't allow token reuse

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (Wave 3)
  - **Blocks**: Task 10
  - **Blocked By**: Tasks 1, 2

  **Acceptance Criteria**:
  - [ ] Reset tokens generated
  - [ ] Tokens expire correctly
  - [ ] Password updated, sessions invalidated

  **QA Scenarios**:
  ```
  Scenario: Password reset flow
    Tool: Bash (cargo test)
    Steps:
      1. Request reset for user
      2. Validate token
      3. Reset password
      4. Verify old sessions revoked
      5. Verify login with new password
    Expected Result: Full reset flow works
    Evidence: .sisyphus/evidence/task-9-reset.txt
  ```

  **Commit**: YES (Wave 3)
  - Message: `feat(auth): add API keys, roles, and password reset`
  - Files: `db/api_keys.rs`, `db/roles.rs`, `core/auth/reset.rs`

- [x] 10. Auth API Endpoints

  **What to do**:
  - Create `crates/cuttlefish-api/src/auth_routes.rs`:
    - `POST /api/auth/register` — Create account
    - `POST /api/auth/login` — Login, returns tokens
    - `POST /api/auth/refresh` — Refresh tokens
    - `POST /api/auth/logout` — Revoke session
    - `POST /api/auth/logout-all` — Revoke all sessions
    - `GET /api/auth/me` — Get current user
    - `PUT /api/auth/me` — Update profile
    - `POST /api/auth/password` — Change password
    - `POST /api/auth/reset-request` — Request password reset
    - `POST /api/auth/reset` — Reset password with token
    - `GET /api/auth/api-keys` — List API keys
    - `POST /api/auth/api-keys` — Create API key
    - `DELETE /api/auth/api-keys/:id` — Revoke API key
  - Request/response types with validation
  - Rate limiting on auth endpoints (10 req/min for login)
  - Add routes to main router

  **Must NOT do**:
  - Don't return password hash in responses
  - Don't allow brute force (rate limit)

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: NO (depends on all Wave 3)
  - **Blocks**: Task 11
  - **Blocked By**: Tasks 4, 5, 6, 7, 8, 9

  **Acceptance Criteria**:
  - [ ] All endpoints work
  - [ ] Validation errors returned
  - [ ] Rate limiting active

  **QA Scenarios**:
  ```
  Scenario: Full auth flow via API
    Tool: Bash (curl)
    Steps:
      1. POST /api/auth/register
      2. POST /api/auth/login
      3. GET /api/auth/me (with token)
      4. POST /api/auth/logout
    Expected Result: Complete flow works
    Evidence: .sisyphus/evidence/task-10-api.txt
  ```

  **Commit**: YES
  - Message: `feat(api): add auth endpoints`
  - Files: `api/auth_routes.rs`, `api/routes.rs`

- [ ] 11. WebUI Auth Pages

  **What to do**:
  - Create `cuttlefish-web/pages/login.vue`:
    - Email/password form
    - "Remember me" checkbox
    - Link to register and reset
    - Error display
  - Create `cuttlefish-web/pages/register.vue`:
    - Email, password, confirm password
    - Password strength indicator
    - Terms acceptance checkbox
  - Create `cuttlefish-web/pages/reset-password.vue`:
    - Request form (email only)
    - Reset form (new password)
  - Add auth state to composables:
    - `useAuth()` — login, logout, user state
    - Store tokens in localStorage/cookies
    - Auto-refresh before expiry
  - Redirect to login when 401 received

  **Must NOT do**:
  - Don't store tokens in plaintext (use httpOnly cookies if possible)
  - Don't show password in URL

  **Recommended Agent Profile**:
  - **Category**: `visual-engineering`
  - **Skills**: [`frontend-ui-ux`]

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Blocks**: F1-F4
  - **Blocked By**: Task 10

  **Acceptance Criteria**:
  - [ ] Login form works
  - [ ] Registration works
  - [ ] Password reset works
  - [ ] Auth state persists

  **QA Scenarios**:
  ```
  Scenario: WebUI login flow
    Tool: Playwright
    Steps:
      1. Navigate to /login
      2. Fill email and password
      3. Click login
      4. Verify redirect to dashboard
      5. Verify user info shown
    Expected Result: Login works in UI
    Evidence: .sisyphus/evidence/task-11-login.png
  ```

  **Commit**: YES
  - Message: `feat(web): add auth pages (login, register, reset)`
  - Files: `pages/login.vue`, `pages/register.vue`, etc.

- [x] 12. Backwards Compatibility Layer

  **What to do**:
  - Support existing `CUTTLEFISH_API_KEY` env var:
    - If set, treat as a "system" API key
    - Map to a system user with admin permissions
    - Log deprecation warning on startup
  - Migration path:
    - On first run with new auth, create admin user from env
    - Generate API key matching `CUTTLEFISH_API_KEY` for that user
    - Document migration in release notes
  - Feature flag: `auth.enabled` in config
    - If false, use old single-key auth
    - If true (default), use new system

  **Must NOT do**:
  - Don't break existing deployments
  - Don't force migration immediately

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 10, 11)
  - **Blocks**: F1-F4
  - **Blocked By**: Tasks 6, 7

  **Acceptance Criteria**:
  - [ ] Old API key still works
  - [ ] Migration creates admin user
  - [ ] Feature flag toggles behavior

  **QA Scenarios**:
  ```
  Scenario: Backwards compatible with old API key
    Tool: Bash
    Steps:
      1. Set CUTTLEFISH_API_KEY=old-key
      2. Start server
      3. curl with X-API-Key: old-key
      4. Verify 200 response
    Expected Result: Old key works
    Evidence: .sisyphus/evidence/task-12-compat.txt
  ```

  **Commit**: YES (Wave 4)
  - Message: `feat(auth): add backwards compatibility for old API key`
  - Files: `api/middleware/auth.rs`, config

---

## Final Verification Wave

- [ ] F1. **Plan Compliance Audit** — `oracle`
  Verify all auth components: users, sessions, API keys, roles, password reset. Check no plaintext passwords.
  Output: `Must Have [N/N] | Must NOT Have [N/N] | Tasks [N/N] | VERDICT: APPROVE/REJECT`

- [ ] F2. **Code Quality + Security Review** — `unspecified-high`
  Run `cargo clippy --workspace -- -D warnings` + tests. Review for security: password hashing, token handling, SQL injection, rate limiting.
  Output: `Build [PASS/FAIL] | Lint [PASS/FAIL] | Security [N issues] | VERDICT`

- [ ] F3. **Auth E2E QA** — `unspecified-high`
  Full workflow: register, login, create project, add member, API key, logout. Test invalid cases: wrong password, expired token, insufficient role.
  Output: `Registration [PASS/FAIL] | Login [PASS/FAIL] | Roles [PASS/FAIL] | API Keys [PASS/FAIL] | VERDICT`

- [ ] F4. **Scope Fidelity Check** — `deep`
  Verify no OAuth (V1 scope), backwards compatibility works, no breaking changes to existing API.
  Output: `Tasks [N/N compliant] | Scope [CLEAN/N violations] | VERDICT`

---

## Commit Strategy

| Wave | Commit |
|------|--------|
| 1 | `feat(auth): add user model and database` |
| 1 | `feat(auth): add password hashing and JWT tokens` |
| 2 | `feat(auth): add session management and auth middleware` |
| 3 | `feat(auth): add API keys, roles, and password reset` |
| 4 | `feat(api): add auth endpoints` |
| 4 | `feat(web): add auth pages (login, register, reset)` |
| 4 | `feat(auth): add backwards compatibility for old API key` |

---

## Success Criteria

### Verification Commands
```bash
cargo test --workspace auth  # All tests pass
cargo clippy --workspace -- -D warnings  # Clean
curl -X POST localhost:8080/api/auth/register -d '{"email":"test@example.com","password":"SecurePass123!"}' # 201
curl -X POST localhost:8080/api/auth/login -d '{"email":"test@example.com","password":"SecurePass123!"}' # Returns tokens
```

### Final Checklist
- [ ] User registration and login work
- [ ] JWT tokens generated with proper expiry
- [ ] Refresh token rotation enforced
- [ ] API keys created with scopes
- [ ] Roles control project access
- [ ] Password reset flow complete
- [ ] Backwards compatible with old API key
- [ ] No plaintext passwords stored
