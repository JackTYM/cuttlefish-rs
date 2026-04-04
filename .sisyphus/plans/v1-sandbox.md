# V1 Docker Sandbox — Full Container Lifecycle Management

## TL;DR

> **Quick Summary**: Expand Cuttlefish's Docker sandbox system with language-specific images, volume mounting, resource limits, and snapshot/restore capabilities for complete project isolation.
> 
> **Deliverables**:
> - 5+ language-specific Docker images (Node, Python, Rust, Go, Ruby)
> - Volume mounting for host directory access
> - CPU/memory/disk resource limits with monitoring
> - Snapshot and restore functionality (save/load container state)
> - Sandbox lifecycle API endpoints
> 
> **Estimated Effort**: Large (4-5 days)
> **Parallel Execution**: YES - 4 waves
> **Critical Path**: Task 1 (Images) → Tasks 2-4 (Features) → Task 5 (API) → Task 6 (Monitor)

---

## Context

### Original Request
Automatic Docker sandboxing for creating and managing projects. Each project gets its own container with configurable resources.

### Current State (Research Findings)
**Already exists (`crates/cuttlefish-sandbox/src/`):**
- `lib.rs` — Sandbox trait and types
- `docker.rs` — Basic Docker operations via bollard
- `images.rs` — Image mapping for templates

**Missing:**
- Language-specific optimized images
- Volume mounting implementation
- Resource limit enforcement and monitoring
- Snapshot/restore functionality
- Container health monitoring
- Cleanup for stale containers

### From Project Spec
- Containers are configured per project
- Config files are user-editable
- Configs define: base Docker image, frameworks to install, auto-install packages
- Example: "when I ask for a webapp, scaffold Nuxt 3 + Tailwind + Cloudflare Pages deployment"

---

## Work Objectives

### Core Objective
Provide secure, isolated, configurable Docker sandboxes for each project with resource management and state persistence capabilities.

### Concrete Deliverables
- `docker/images/` — Dockerfiles for language-specific images
- `crates/cuttlefish-sandbox/src/images.rs` — Enhanced image management
- `crates/cuttlefish-sandbox/src/volumes.rs` — Volume mounting
- `crates/cuttlefish-sandbox/src/resources.rs` — Resource limits and monitoring
- `crates/cuttlefish-sandbox/src/snapshots.rs` — Snapshot/restore
- `crates/cuttlefish-sandbox/src/lifecycle.rs` — Container lifecycle management

### Definition of Done
- [ ] 5+ language images build and run
- [ ] Volume mounting works for host directories
- [ ] Resource limits enforced (CPU, memory, disk)
- [ ] Snapshot creates saveable container state
- [ ] Restore recreates container from snapshot
- [ ] `cargo test -p cuttlefish-sandbox` passes
- [ ] `cargo clippy -p cuttlefish-sandbox -- -D warnings` clean

### Must Have
- Node.js image with npm/yarn/pnpm
- Python image with pip/poetry/uv
- Rust image with cargo, rustup
- Go image with go mod
- Ruby image with bundler
- Volume mount API: `mount_volume(container, host_path, container_path)`
- Resource limits: CPU cores, memory MB, disk GB
- `snapshot(container_id) -> SnapshotId`
- `restore(snapshot_id) -> ContainerId`

### Must NOT Have (Guardrails)
- No unsafe code
- No `unwrap()` — use `?` or `expect("reason")`
- No privileged containers (security)
- No host network mode (isolation)
- No root user in containers (non-root by default)
- No persistent storage outside designated volumes

---

## Verification Strategy

> **ZERO HUMAN INTERVENTION** — ALL verification is agent-executed.

### Test Decision
- **Infrastructure exists**: YES (bollard crate)
- **Automated tests**: YES (TDD for core features)
- **Framework**: `#[tokio::test]` for async Docker operations

### QA Policy
Every task includes agent-executed QA scenarios.
Evidence saved to `.sisyphus/evidence/task-{N}-{scenario-slug}.{ext}`.

- **Docker tests**: Require Docker daemon (integration tests)
- **Image tests**: Build and run basic commands
- **Snapshot tests**: Create, list, restore, delete

---

## Execution Strategy

### Parallel Execution Waves

```
Wave 1 (Foundation — images and base):
├── Task 1: Create language-specific Dockerfiles [quick]
├── Task 2: Enhanced image registry [unspecified-high]
└── Task 3: Container lifecycle management [deep]

Wave 2 (Features — volumes and resources):
├── Task 4: Volume mounting implementation [deep]
├── Task 5: Resource limits enforcement [deep]
└── Task 6: Resource monitoring [unspecified-high]

Wave 3 (State — snapshots):
├── Task 7: Snapshot creation [deep]
├── Task 8: Snapshot restore [deep]
└── Task 9: Snapshot management (list, delete) [quick]

Wave 4 (Integration):
├── Task 10: API endpoints for sandbox operations [unspecified-high]
├── Task 11: Stale container cleanup [unspecified-high]
└── Task 12: Health monitoring [quick]

Wave FINAL (Verification):
├── Task F1: Plan compliance audit (oracle)
├── Task F2: Code quality review (unspecified-high)
├── Task F3: Docker E2E QA (unspecified-high)
└── Task F4: Scope fidelity check (deep)
-> Present results -> Get explicit user okay

Critical Path: Task 1 → Task 3 → Task 4 → Task 7 → Task 10 → F1-F4 → user okay
Parallel Speedup: ~55% faster than sequential
Max Concurrent: 3 (all waves)
```

### Dependency Matrix

| Task | Depends On | Blocks | Wave |
|------|------------|--------|------|
| 1 | — | 2, 3 | 1 |
| 2 | 1 | 3, 10 | 1 |
| 3 | 1, 2 | 4-12 | 1 |
| 4 | 3 | 7, 10 | 2 |
| 5 | 3 | 6, 10 | 2 |
| 6 | 5 | 10, 12 | 2 |
| 7 | 3, 4 | 8 | 3 |
| 8 | 7 | 9 | 3 |
| 9 | 7, 8 | 10 | 3 |
| 10 | 2-9 | 11, 12 | 4 |
| 11 | 3 | F1-F4 | 4 |
| 12 | 6 | F1-F4 | 4 |

### Agent Dispatch Summary

- **Wave 1**: 3 tasks — T1 → `quick`, T2 → `unspecified-high`, T3 → `deep`
- **Wave 2**: 3 tasks — T4-T5 → `deep`, T6 → `unspecified-high`
- **Wave 3**: 3 tasks — T7-T8 → `deep`, T9 → `quick`
- **Wave 4**: 3 tasks — T10 → `unspecified-high`, T11 → `unspecified-high`, T12 → `quick`
- **FINAL**: 4 tasks — F1 → `oracle`, F2-F3 → `unspecified-high`, F4 → `deep`

---

## TODOs

- [ ] 1. Create Language-Specific Dockerfiles

  **What to do**:
  - Create `docker/images/` directory
  - Create `docker/images/node/Dockerfile`:
    - Base: `node:20-slim`
    - Add: npm, yarn, pnpm, git, curl
    - Non-root user: `cuttlefish` with UID 1000
    - Working directory: `/workspace`
  - Create `docker/images/python/Dockerfile`:
    - Base: `python:3.12-slim`
    - Add: pip, poetry, uv, git, curl
    - Non-root user: `cuttlefish`
  - Create `docker/images/rust/Dockerfile`:
    - Base: `rust:1.87-slim`
    - Add: cargo, rustfmt, clippy, git
    - Non-root user: `cuttlefish`
  - Create `docker/images/go/Dockerfile`:
    - Base: `golang:1.23-alpine`
    - Add: go mod support, git
    - Non-root user: `cuttlefish`
  - Create `docker/images/ruby/Dockerfile`:
    - Base: `ruby:3.3-slim`
    - Add: bundler, gem, git
    - Non-root user: `cuttlefish`
  - All images: labels for cuttlefish identification

  **Must NOT do**:
  - No root user as default
  - No privileged operations in Dockerfile
  - No hardcoded secrets or tokens

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (Wave 1 start)
  - **Blocks**: Tasks 2, 3
  - **Blocked By**: None

  **References**:
  - Docker best practices: multi-stage builds, non-root users
  - `crates/cuttlefish-sandbox/src/images.rs` — Existing image patterns

  **Acceptance Criteria**:
  - [ ] All 5 Dockerfiles exist
  - [ ] Each builds successfully: `docker build -t cuttlefish/{lang} .`
  - [ ] Non-root user verified: `docker run cuttlefish/node whoami` → `cuttlefish`

  **QA Scenarios**:
  ```
  Scenario: Build all language images
    Tool: Bash
    Preconditions: Docker daemon running
    Steps:
      1. For each image: docker build -t cuttlefish/{lang} docker/images/{lang}/
      2. Verify exit code 0 for all
    Expected Result: All images built
    Evidence: .sisyphus/evidence/task-1-build-images.txt

  Scenario: Non-root user default
    Tool: Bash
    Steps:
      1. docker run --rm cuttlefish/node whoami
      2. Verify output is "cuttlefish"
    Expected Result: Non-root user
    Evidence: .sisyphus/evidence/task-1-nonroot.txt
  ```

  **Commit**: YES
  - Message: `feat(sandbox): add language-specific Docker images`
  - Files: `docker/images/**/Dockerfile`

- [ ] 2. Enhanced Image Registry

  **What to do**:
  - Expand `crates/cuttlefish-sandbox/src/images.rs`
  - Add `ImageSpec` struct:
    ```rust
    pub struct ImageSpec {
        pub name: String,
        pub tag: String,
        pub languages: Vec<String>,
        pub tools: Vec<String>,
        pub default_packages: Vec<String>,
    }
    ```
  - Implement `ImageRegistry`:
    - `get_image_for_language(lang: &str) -> Option<ImageSpec>`
    - `get_image_for_template(template: &str) -> Option<ImageSpec>`
    - `list_available_images() -> Vec<ImageSpec>`
    - `ensure_image_pulled(spec: &ImageSpec) -> Result<()>`
  - Map templates to images (nuxt → node, fastapi → python, etc.)
  - Auto-pull images if not present locally

  **Must NOT do**:
  - Don't pull images synchronously on startup (lazy pull)

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Task 3)
  - **Blocks**: Tasks 3, 10
  - **Blocked By**: Task 1

  **References**:
  - `crates/cuttlefish-sandbox/src/images.rs` — Current implementation
  - bollard image API

  **Acceptance Criteria**:
  - [ ] Registry maps languages to images
  - [ ] Auto-pull works
  - [ ] Template mapping correct

  **QA Scenarios**:
  ```
  Scenario: Get image for language
    Tool: Bash (cargo test)
    Steps:
      1. cargo test -p cuttlefish-sandbox images::tests::test_get_image
      2. Verify returns correct ImageSpec for "python"
    Expected Result: Python image spec returned
    Evidence: .sisyphus/evidence/task-2-registry.txt
  ```

  **Commit**: YES
  - Message: `feat(sandbox): enhance image registry with language mapping`
  - Files: `images.rs`

- [ ] 3. Container Lifecycle Management

  **What to do**:
  - Create `crates/cuttlefish-sandbox/src/lifecycle.rs`
  - Implement `ContainerManager`:
    ```rust
    pub struct ContainerManager {
        docker: Docker,
        containers: HashMap<ProjectId, ContainerInfo>,
    }
    
    impl ContainerManager {
        pub async fn create(&self, project_id: &str, spec: ContainerSpec) -> Result<ContainerId>
        pub async fn start(&self, container_id: &str) -> Result<()>
        pub async fn stop(&self, container_id: &str) -> Result<()>
        pub async fn destroy(&self, container_id: &str) -> Result<()>
        pub async fn exec(&self, container_id: &str, cmd: &[&str]) -> Result<ExecOutput>
        pub async fn get_status(&self, container_id: &str) -> Result<ContainerStatus>
    }
    ```
  - `ContainerSpec`: image, env vars, volumes, resource limits
  - Handle container naming: `cuttlefish-{project_id}`
  - Store container metadata in database
  - Handle already-exists and not-found errors gracefully

  **Must NOT do**:
  - Don't use privileged mode
  - Don't expose Docker socket inside container

  **Recommended Agent Profile**:
  - **Category**: `deep`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Task 2)
  - **Blocks**: Tasks 4-12
  - **Blocked By**: Tasks 1, 2

  **References**:
  - `crates/cuttlefish-sandbox/src/docker.rs` — Existing Docker utilities
  - bollard container API

  **Acceptance Criteria**:
  - [ ] Create/start/stop/destroy work
  - [ ] Exec runs commands in container
  - [ ] Status returns running/stopped/etc

  **QA Scenarios**:
  ```
  Scenario: Full container lifecycle
    Tool: Bash (cargo test)
    Steps:
      1. Create container
      2. Start container
      3. Exec "echo hello"
      4. Stop container
      5. Destroy container
    Expected Result: All operations succeed
    Evidence: .sisyphus/evidence/task-3-lifecycle.txt
  ```

  **Commit**: YES
  - Message: `feat(sandbox): add container lifecycle management`
  - Files: `lifecycle.rs`

- [ ] 4. Volume Mounting Implementation

  **What to do**:
  - Create `crates/cuttlefish-sandbox/src/volumes.rs`
  - Implement `VolumeManager`:
    ```rust
    pub struct VolumeMount {
        pub host_path: PathBuf,
        pub container_path: String,
        pub readonly: bool,
    }
    
    pub fn validate_mount(mount: &VolumeMount) -> Result<()>
    pub fn create_bind_mount(mount: &VolumeMount) -> bollard::service::Mount
    ```
  - Security validation:
    - Host path must be under allowed directories
    - No mounting /etc, /var, /usr, etc.
    - No symlink traversal
  - Default mounts:
    - Project workspace: `{project_dir}` → `/workspace`
    - Shared cache: `~/.cache/cuttlefish` → `/home/cuttlefish/.cache`
  - Add volume config to ContainerSpec

  **Must NOT do**:
  - Don't allow mounting outside project directory without explicit config
  - Don't allow mounting Docker socket

  **Recommended Agent Profile**:
  - **Category**: `deep`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 5, 6)
  - **Blocks**: Tasks 7, 10
  - **Blocked By**: Task 3

  **References**:
  - bollard mount API
  - Docker bind mounts documentation

  **Acceptance Criteria**:
  - [ ] Mount creates host→container binding
  - [ ] Security validation rejects dangerous paths
  - [ ] Files written in container appear on host

  **QA Scenarios**:
  ```
  Scenario: Mount project directory
    Tool: Bash (cargo test)
    Steps:
      1. Create container with volume mount
      2. Exec "echo test > /workspace/test.txt"
      3. Verify file exists on host
    Expected Result: File persists to host
    Evidence: .sisyphus/evidence/task-4-mount.txt

  Scenario: Reject dangerous mount
    Tool: Bash (cargo test)
    Steps:
      1. Attempt mount of /etc
      2. Verify validation error
    Expected Result: Mount rejected
    Evidence: .sisyphus/evidence/task-4-security.txt
  ```

  **Commit**: YES
  - Message: `feat(sandbox): add volume mounting with security validation`
  - Files: `volumes.rs`

- [ ] 5. Resource Limits Enforcement

  **What to do**:
  - Create `crates/cuttlefish-sandbox/src/resources.rs`
  - Implement `ResourceLimits`:
    ```rust
    pub struct ResourceLimits {
        pub cpu_cores: f64,      // e.g., 2.0
        pub memory_mb: u64,      // e.g., 2048
        pub disk_gb: u64,        // e.g., 10
        pub pids_limit: i64,     // e.g., 100
    }
    
    impl Default for ResourceLimits {
        // Use values from cuttlefish.toml [sandbox] section
    }
    ```
  - Convert to bollard `HostConfig`:
    - `cpu_cores` → `NanoCpus` (multiply by 1e9)
    - `memory_mb` → `Memory` (multiply by 1024*1024)
    - `disk_gb` → storage driver quota (if supported)
  - Validate limits against system resources
  - Apply limits on container creation

  **Must NOT do**:
  - Don't allow unlimited resources
  - Don't exceed host capacity

  **Recommended Agent Profile**:
  - **Category**: `deep`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 4, 6)
  - **Blocks**: Tasks 6, 10
  - **Blocked By**: Task 3

  **References**:
  - Docker resource constraints: https://docs.docker.com/config/containers/resource_constraints/
  - bollard HostConfig

  **Acceptance Criteria**:
  - [ ] CPU limit enforced
  - [ ] Memory limit enforced
  - [ ] Container killed if exceeds memory

  **QA Scenarios**:
  ```
  Scenario: Memory limit enforced
    Tool: Bash (cargo test)
    Steps:
      1. Create container with 256MB limit
      2. Exec memory-consuming command
      3. Verify OOM kill or graceful limit
    Expected Result: Memory constrained
    Evidence: .sisyphus/evidence/task-5-memory.txt
  ```

  **Commit**: NO (groups with Wave 2)

- [ ] 6. Resource Monitoring

  **What to do**:
  - Add to `resources.rs`:
    ```rust
    pub struct ResourceUsage {
        pub cpu_percent: f64,
        pub memory_used_mb: u64,
        pub memory_limit_mb: u64,
        pub disk_used_gb: f64,
        pub network_rx_bytes: u64,
        pub network_tx_bytes: u64,
    }
    
    pub async fn get_usage(container_id: &str) -> Result<ResourceUsage>
    ```
  - Use Docker stats API (bollard)
  - Calculate percentages
  - Cache recent readings (for trends)
  - Alert thresholds: warn at 80%, critical at 95%

  **Must NOT do**:
  - Don't poll more than once per second (performance)

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: NO (depends on Task 5)
  - **Blocks**: Tasks 10, 12
  - **Blocked By**: Task 5

  **References**:
  - bollard stats API

  **Acceptance Criteria**:
  - [ ] Stats retrieved correctly
  - [ ] Percentages calculated
  - [ ] Thresholds detected

  **QA Scenarios**:
  ```
  Scenario: Get container resource usage
    Tool: Bash (cargo test)
    Steps:
      1. Start container
      2. Run some commands
      3. Get usage
      4. Verify non-zero values
    Expected Result: Usage data returned
    Evidence: .sisyphus/evidence/task-6-usage.txt
  ```

  **Commit**: YES (Wave 2)
  - Message: `feat(sandbox): add resource limits and monitoring`
  - Files: `resources.rs`

- [ ] 7. Snapshot Creation

  **What to do**:
  - Create `crates/cuttlefish-sandbox/src/snapshots.rs`
  - Implement snapshot creation:
    ```rust
    pub struct Snapshot {
        pub id: SnapshotId,
        pub project_id: String,
        pub container_id: String,
        pub created_at: DateTime<Utc>,
        pub size_bytes: u64,
        pub description: Option<String>,
    }
    
    pub async fn create_snapshot(
        container_id: &str,
        description: Option<&str>
    ) -> Result<Snapshot>
    ```
  - Use `docker commit` to create image from container
  - Tag image: `cuttlefish-snapshot:{project_id}-{timestamp}`
  - Store snapshot metadata in database
  - Include volumes in snapshot (tar + store)

  **Must NOT do**:
  - Don't snapshot running containers without pause (data consistency)
  - Don't store snapshots indefinitely (limit per project)

  **Recommended Agent Profile**:
  - **Category**: `deep`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 8, 9 partially)
  - **Blocks**: Task 8
  - **Blocked By**: Tasks 3, 4

  **References**:
  - Docker commit: https://docs.docker.com/engine/reference/commandline/commit/
  - bollard commit API

  **Acceptance Criteria**:
  - [ ] Snapshot created as image
  - [ ] Metadata stored in DB
  - [ ] Volumes included

  **QA Scenarios**:
  ```
  Scenario: Create and verify snapshot
    Tool: Bash (cargo test)
    Steps:
      1. Create container
      2. Make changes (create file)
      3. Create snapshot
      4. Verify image exists: docker images | grep cuttlefish-snapshot
    Expected Result: Snapshot image created
    Evidence: .sisyphus/evidence/task-7-snapshot.txt
  ```

  **Commit**: NO (groups with Wave 3)

- [ ] 8. Snapshot Restore

  **What to do**:
  - Add to `snapshots.rs`:
    ```rust
    pub async fn restore_snapshot(
        snapshot_id: &str,
        new_project_id: Option<&str>
    ) -> Result<ContainerId>
    ```
  - Create new container from snapshot image
  - Restore volumes from snapshot tar
  - Optionally assign to new project (fork)
  - Update database with new container mapping
  - Handle snapshot not found error

  **Must NOT do**:
  - Don't delete original snapshot on restore
  - Don't restore to running container (create new)

  **Recommended Agent Profile**:
  - **Category**: `deep`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: NO (depends on Task 7)
  - **Blocks**: Task 9
  - **Blocked By**: Task 7

  **References**:
  - Docker run from committed image

  **Acceptance Criteria**:
  - [ ] Container created from snapshot
  - [ ] State restored correctly
  - [ ] Fork to new project works

  **QA Scenarios**:
  ```
  Scenario: Restore snapshot
    Tool: Bash (cargo test)
    Steps:
      1. Create container, make changes
      2. Create snapshot
      3. Destroy original container
      4. Restore from snapshot
      5. Verify file from step 1 exists
    Expected Result: State restored
    Evidence: .sisyphus/evidence/task-8-restore.txt
  ```

  **Commit**: NO (groups with Wave 3)

- [ ] 9. Snapshot Management

  **What to do**:
  - Add to `snapshots.rs`:
    ```rust
    pub async fn list_snapshots(project_id: &str) -> Result<Vec<Snapshot>>
    pub async fn delete_snapshot(snapshot_id: &str) -> Result<()>
    pub async fn get_snapshot_size(snapshot_id: &str) -> Result<u64>
    ```
  - List shows: id, description, created_at, size
  - Delete removes: image + volume tar + DB record
  - Enforce snapshot limit per project (default 5)
  - Auto-delete oldest when limit exceeded

  **Must NOT do**:
  - Don't allow deleting in-use snapshots

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: NO (depends on Tasks 7, 8)
  - **Blocks**: Task 10
  - **Blocked By**: Tasks 7, 8

  **Acceptance Criteria**:
  - [ ] List returns all project snapshots
  - [ ] Delete cleans up completely
  - [ ] Limit enforced

  **QA Scenarios**:
  ```
  Scenario: Snapshot limit enforcement
    Tool: Bash (cargo test)
    Steps:
      1. Create 6 snapshots (limit is 5)
      2. Verify oldest deleted automatically
      3. List shows exactly 5
    Expected Result: Limit enforced
    Evidence: .sisyphus/evidence/task-9-limit.txt
  ```

  **Commit**: YES (Wave 3)
  - Message: `feat(sandbox): add snapshot create/restore/manage`
  - Files: `snapshots.rs`

- [ ] 10. API Endpoints for Sandbox Operations

  **What to do**:
  - Add to `crates/cuttlefish-api/src/api_routes.rs`:
    - `POST /api/projects/:id/sandbox/start` — Start sandbox
    - `POST /api/projects/:id/sandbox/stop` — Stop sandbox
    - `GET /api/projects/:id/sandbox/status` — Get status + resources
    - `POST /api/projects/:id/sandbox/exec` — Execute command
    - `POST /api/projects/:id/sandbox/snapshot` — Create snapshot
    - `POST /api/projects/:id/sandbox/restore` — Restore snapshot
    - `GET /api/projects/:id/sandbox/snapshots` — List snapshots
    - `DELETE /api/projects/:id/sandbox/snapshots/:snapshot_id` — Delete snapshot
  - Request/response types with proper validation
  - Error responses with helpful messages
  - Rate limiting on expensive operations (snapshot)

  **Must NOT do**:
  - Don't allow sandbox operations without project auth
  - Don't expose internal Docker IDs

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 11, 12)
  - **Blocks**: F1-F4
  - **Blocked By**: Tasks 2-9

  **References**:
  - `crates/cuttlefish-api/src/api_routes.rs` — Existing API patterns

  **Acceptance Criteria**:
  - [ ] All endpoints respond correctly
  - [ ] Auth required
  - [ ] Errors are user-friendly

  **QA Scenarios**:
  ```
  Scenario: Start and stop sandbox via API
    Tool: Bash (curl)
    Steps:
      1. POST /api/projects/test/sandbox/start
      2. GET /api/projects/test/sandbox/status (verify running)
      3. POST /api/projects/test/sandbox/stop
      4. GET /api/projects/test/sandbox/status (verify stopped)
    Expected Result: Lifecycle via API works
    Evidence: .sisyphus/evidence/task-10-api.txt
  ```

  **Commit**: YES
  - Message: `feat(api): add sandbox management endpoints`
  - Files: `api_routes.rs`

- [ ] 11. Stale Container Cleanup

  **What to do**:
  - Create `crates/cuttlefish-sandbox/src/cleanup.rs`
  - Implement cleanup logic:
    ```rust
    pub async fn cleanup_stale_containers(
        max_idle_hours: u64,
        dry_run: bool
    ) -> Result<CleanupReport>
    ```
  - Find containers with no activity for N hours
  - Stop and optionally remove them
  - Clean up orphaned images (snapshots with no container)
  - Clean up old volume tars
  - Run on schedule (configurable, default every 6 hours)
  - Log cleanup actions

  **Must NOT do**:
  - Don't delete snapshots automatically (only orphaned)
  - Don't cleanup without logging

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 10, 12)
  - **Blocks**: F1-F4
  - **Blocked By**: Task 3

  **Acceptance Criteria**:
  - [ ] Stale containers identified
  - [ ] Cleanup respects dry_run
  - [ ] Report shows what was cleaned

  **QA Scenarios**:
  ```
  Scenario: Cleanup stale container
    Tool: Bash (cargo test)
    Steps:
      1. Create container
      2. Mock last_activity to 48 hours ago
      3. Run cleanup with max_idle_hours=24
      4. Verify container stopped/removed
    Expected Result: Stale container cleaned
    Evidence: .sisyphus/evidence/task-11-cleanup.txt
  ```

  **Commit**: NO (groups with Wave 4)

- [ ] 12. Health Monitoring

  **What to do**:
  - Create `crates/cuttlefish-sandbox/src/health.rs`
  - Implement health checks:
    ```rust
    pub struct HealthStatus {
        pub healthy: bool,
        pub container_running: bool,
        pub cpu_ok: bool,
        pub memory_ok: bool,
        pub disk_ok: bool,
        pub last_check: DateTime<Utc>,
    }
    
    pub async fn check_health(container_id: &str) -> Result<HealthStatus>
    ```
  - Unhealthy conditions:
    - Container stopped unexpectedly
    - Resource usage > 95%
    - No response to health command
  - Store health history (last 24 hours)
  - Trigger alerts on repeated failures

  **Must NOT do**:
  - Don't auto-restart unhealthy containers (notify only)

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 10, 11)
  - **Blocks**: F1-F4
  - **Blocked By**: Task 6

  **Acceptance Criteria**:
  - [ ] Health check runs correctly
  - [ ] Unhealthy conditions detected
  - [ ] History stored

  **QA Scenarios**:
  ```
  Scenario: Detect unhealthy container
    Tool: Bash (cargo test)
    Steps:
      1. Create container
      2. Stop it externally
      3. Run health check
      4. Verify healthy=false, container_running=false
    Expected Result: Unhealthy detected
    Evidence: .sisyphus/evidence/task-12-health.txt
  ```

  **Commit**: YES (Wave 4)
  - Message: `feat(sandbox): add cleanup and health monitoring`
  - Files: `cleanup.rs`, `health.rs`

---

## Final Verification Wave

- [ ] F1. **Plan Compliance Audit** — `oracle`
  Read the plan end-to-end. For each "Must Have": verify implementation exists. For each "Must NOT Have": search for privileged, host network, root user violations.
  Output: `Must Have [N/N] | Must NOT Have [N/N] | Tasks [N/N] | VERDICT: APPROVE/REJECT`

- [ ] F2. **Code Quality Review** — `unspecified-high`
  Run `cargo clippy -p cuttlefish-sandbox -- -D warnings` + `cargo test -p cuttlefish-sandbox`. Review for unwrap(), unsafe code.
  Output: `Build [PASS/FAIL] | Lint [PASS/FAIL] | Tests [N pass/N fail] | VERDICT`

- [ ] F3. **Docker E2E QA** — `unspecified-high`
  Full workflow: create container, mount volume, write file, create snapshot, destroy, restore, verify file exists.
  Output: `Create [PASS/FAIL] | Volume [PASS/FAIL] | Snapshot [PASS/FAIL] | Restore [PASS/FAIL] | VERDICT`

- [ ] F4. **Scope Fidelity Check** — `deep`
  Verify no privileged containers, no host network, non-root users in all images.
  Output: `Tasks [N/N compliant] | Security [CLEAN/N violations] | VERDICT`

---

## Commit Strategy

| Wave | Commit |
|------|--------|
| 1 | `feat(sandbox): add language-specific Docker images` |
| 1 | `feat(sandbox): enhance image registry and lifecycle` |
| 2 | `feat(sandbox): add volume mounting and resource limits` |
| 3 | `feat(sandbox): add snapshot create/restore/manage` |
| 4 | `feat(api): add sandbox management endpoints` |
| 4 | `feat(sandbox): add cleanup and health monitoring` |

---

## Success Criteria

### Verification Commands
```bash
docker build -t cuttlefish/node docker/images/node/  # Image builds
cargo test -p cuttlefish-sandbox  # All tests pass
cargo clippy -p cuttlefish-sandbox -- -D warnings  # Clean
curl -X POST http://localhost:8080/api/projects/test/sandbox/snapshot  # API works
```

### Final Checklist
- [ ] 5 language images build and run with non-root user
- [ ] Volume mounting works with security validation
- [ ] Resource limits enforced (CPU, memory)
- [ ] Snapshots create and restore correctly
- [ ] API endpoints all functional
- [ ] No privileged containers, no host network, no root
