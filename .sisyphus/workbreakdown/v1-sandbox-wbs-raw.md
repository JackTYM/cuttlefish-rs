Task continued and completed in 5m 56s.

---

# v1-sandbox-wbs.md

```markdown
# V1 Sandbox Work Breakdown Structure

## Executive Summary

This document provides execution-ready task breakdowns for implementing the Cuttlefish sandbox system. All tasks follow TDD methodology, include verification criteria, and are sized for agent delegation.

**Total Effort**: ~25 hours (critical path)
**Parallelization Potential**: 4-5 days with concurrent execution
**Task Count**: 16 main tasks → 45 atomic work units

---

## Dependency Graph

```
T1 (Docker Images)
 │
 ├──► T2 (Image Tests) ──────────────────────────────┐
 │                                                    │
 └──► T3 (ImageRegistry Trait) ──► T4 (Registry Impl)│
                                         │           │
                                         ▼           │
      T5 (Lifecycle Trait) ──► T6 (Lifecycle Impl)   │
             │                       │               │
             │                       ▼               │
             │              T7 (Volume Mounts)       │
             │                       │               │
             │                       ▼               │
             │              T8 (Resource Limits)     │
             │                       │               │
             ▼                       ▼               │
      T9 (Snapshot/Restore) ◄────────┘               │
             │                                       │
             ▼                                       │
      T10 (Cleanup/GC) ◄─────────────────────────────┘
             │
             ▼
      T11 (Health Checks)
             │
             ▼
      T12 (API Routes)
             │
             ├──► T13 (E2E Workflow Tests)
             │
             └──► T14 (Performance Tests)
             
      FS1 (Full Verification) ◄── T13, T14
```

---

## Critical Path

```
T1 → T3 → T4 → T7 → T8 → T9 → T10 → T11 → T12 → T13
│    │    │    │    │    │    │     │     │     │
2h   2h   3h   3h   3h   4h   2h    2h    3h    2h  = ~26 hours
```

---

## Parallel Execution Schedule

### Wave 1 (Hours 0-4)
| Track A | Track B |
|---------|---------|
| T1: Docker Images (2h) | — |
| T2: Image Tests (1h) | T3: ImageRegistry Trait (2h) |

### Wave 2 (Hours 4-10)
| Track A | Track B |
|---------|---------|
| T4: Registry Impl (3h) | T5: Lifecycle Trait (2h) |
| T6: Lifecycle Impl (3h) | — |

### Wave 3 (Hours 10-20)
| Track A | Track B |
|---------|---------|
| T7: Volume Mounts (3h) | — |
| T8: Resource Limits (3h) | — |
| T9: Snapshot/Restore (4h) | — |

### Wave 4 (Hours 20-26)
| Track A | Track B |
|---------|---------|
| T10: Cleanup/GC (2h) | — |
| T11: Health Checks (2h) | — |
| T12: API Routes (3h) | — |

### Wave 5 (Hours 26-30)
| Track A | Track B |
|---------|---------|
| T13: E2E Tests (2h) | T14: Perf Tests (2h) |
| FS1: Verification (1h) | — |

---

## Task Breakdowns

### T1: Docker Base Images

**Category**: `quick`
**Estimated Hours**: 2
**Dependencies**: None
**Skills**: Docker, multi-stage builds

#### Work Units

##### T1.1: Create Node.js Dockerfile
```yaml
file: docker/images/node/Dockerfile
action: create
content_requirements:
  - Multi-stage build (builder + runtime)
  - Node 20 LTS base
  - Non-root user (sandbox:sandbox, UID 1000)
  - Working directory /workspace
  - No shell access (use exec form)
  - Labels for metadata
verification:
  command: "docker build -t cuttlefish/node:test docker/images/node/"
  success: "Successfully built"
```

##### T1.2: Create Python Dockerfile
```yaml
file: docker/images/python/Dockerfile
action: create
content_requirements:
  - Multi-stage build
  - Python 3.12 slim base
  - Non-root user (sandbox:sandbox, UID 1000)
  - pip with --no-cache-dir
  - Virtual environment at /venv
  - Working directory /workspace
verification:
  command: "docker build -t cuttlefish/python:test docker/images/python/"
  success: "Successfully built"
```

##### T1.3: Create Rust Dockerfile
```yaml
file: docker/images/rust/Dockerfile
action: create
content_requirements:
  - Multi-stage build (builder + runtime)
  - Rust stable base
  - Non-root user (sandbox:sandbox, UID 1000)
  - Cargo home at /usr/local/cargo
  - Working directory /workspace
  - Include common tools (rustfmt, clippy)
verification:
  command: "docker build -t cuttlefish/rust:test docker/images/rust/"
  success: "Successfully built"
```

##### T1.4: Create Go Dockerfile
```yaml
file: docker/images/go/Dockerfile
action: create
content_requirements:
  - Multi-stage build
  - Go 1.22 base
  - Non-root user (sandbox:sandbox, UID 1000)
  - GOPATH at /go
  - Working directory /workspace
verification:
  command: "docker build -t cuttlefish/go:test docker/images/go/"
  success: "Successfully built"
```

##### T1.5: Create Ruby Dockerfile
```yaml
file: docker/images/ruby/Dockerfile
action: create
content_requirements:
  - Multi-stage build
  - Ruby 3.3 slim base
  - Non-root user (sandbox:sandbox, UID 1000)
  - Bundler pre-installed
  - Working directory /workspace
verification:
  command: "docker build -t cuttlefish/ruby:test docker/images/ruby/"
  success: "Successfully built"
```

#### Atomic Commit
```
feat(sandbox): add Docker base images for 5 languages

- Node.js 20 LTS with non-root user
- Python 3.12 with virtual environment
- Rust stable with cargo tools
- Go 1.22 with standard layout
- Ruby 3.3 with bundler

All images use multi-stage builds for minimal size.
All run as non-root sandbox user (UID 1000).
```

---

### T2: Image Build Tests

**Category**: `quick`
**Estimated Hours**: 1
**Dependencies**: T1
**Skills**: Rust testing, Docker API

#### Work Units

##### T2.1: Create image build integration tests
```yaml
file: crates/cuttlefish-sandbox/tests/image_build.rs
action: create
content_requirements:
  - "#[cfg(feature = \"integration\")]" gate
  - Test each Dockerfile builds successfully
  - Verify non-root user exists in image
  - Verify working directory is /workspace
  - Use bollard crate for Docker API
test_structure: |
  #[cfg(feature = "integration")]
  mod image_build_tests {
      use bollard::Docker;
      
      #[tokio::test]
      async fn test_node_image_builds() { ... }
      
      #[tokio::test]
      async fn test_python_image_builds() { ... }
      
      #[tokio::test]
      async fn test_rust_image_builds() { ... }
      
      #[tokio::test]
      async fn test_go_image_builds() { ... }
      
      #[tokio::test]
      async fn test_ruby_image_builds() { ... }
      
      #[tokio::test]
      async fn test_images_have_nonroot_user() { ... }
  }
verification:
  command: "cargo test -p cuttlefish-sandbox --features integration image_build"
  success: "test result: ok"
```

#### Atomic Commit
```
test(sandbox): add integration tests for Docker image builds

Tests verify all 5 language images build correctly and
run as non-root user. Gated behind integration feature.
```

---

### T3: ImageRegistry Trait Definition

**Category**: `deep`
**Estimated Hours**: 2
**Dependencies**: T1
**Skills**: Rust traits, async design

#### Work Units

##### T3.1: Define ImageRegistry trait in cuttlefish-core
```yaml
file: crates/cuttlefish-core/src/traits/sandbox.rs
action: modify (or create if not exists)
content_requirements:
  - Async trait using async-trait crate
  - Methods: list_images, pull_image, build_image, remove_image
  - ImageSpec struct for image metadata
  - ImageBuildOptions struct for build configuration
  - Proper error types using thiserror
trait_definition: |
  use async_trait::async_trait;
  use crate::error::SandboxError;
  
  /// Specification for a container image
  #[derive(Debug, Clone)]
  pub struct ImageSpec {
      pub name: String,
      pub tag: String,
      pub language: Language,
      pub size_bytes: Option<u64>,
      pub created_at: Option<chrono::DateTime<chrono::Utc>>,
  }
  
  /// Supported sandbox languages
  #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
  pub enum Language {
      Node,
      Python,
      Rust,
      Go,
      Ruby,
  }
  
  /// Options for building an image
  #[derive(Debug, Clone, Default)]
  pub struct ImageBuildOptions {
      pub dockerfile_path: Option<PathBuf>,
      pub build_args: HashMap<String, String>,
      pub no_cache: bool,
  }
  
  /// Registry for managing sandbox container images
  #[async_trait]
  pub trait ImageRegistry: Send + Sync {
      /// List all available sandbox images
      async fn list_images(&self) -> Result<Vec<ImageSpec>, SandboxError>;
      
      /// Pull an image from a remote registry
      async fn pull_image(&self, name: &str, tag: &str) -> Result<ImageSpec, SandboxError>;
      
      /// Build an image from a Dockerfile
      async fn build_image(
          &self,
          name: &str,
          tag: &str,
          options: ImageBuildOptions,
      ) -> Result<ImageSpec, SandboxError>;
      
      /// Remove an image
      async fn remove_image(&self, name: &str, tag: &str) -> Result<(), SandboxError>;
      
      /// Get image for a specific language
      async fn get_language_image(&self, lang: Language) -> Result<ImageSpec, SandboxError>;
  }
verification:
  command: "cargo check -p cuttlefish-core"
  success: "Finished"
```

##### T3.2: Add SandboxError variants
```yaml
file: crates/cuttlefish-core/src/error.rs
action: modify
content_requirements:
  - Add SandboxError enum if not exists
  - Variants: ImageNotFound, ImageBuildFailed, DockerError, etc.
  - Derive thiserror::Error
  - Include source errors where appropriate
error_definition: |
  #[derive(Debug, thiserror::Error)]
  pub enum SandboxError {
      #[error("Image not found: {name}:{tag}")]
      ImageNotFound { name: String, tag: String },
      
      #[error("Image build failed: {reason}")]
      ImageBuildFailed { reason: String },
      
      #[error("Docker error: {0}")]
      Docker(#[from] bollard::errors::Error),
      
      #[error("Container not found: {id}")]
      ContainerNotFound { id: String },
      
      #[error("Execution timeout after {seconds}s")]
      Timeout { seconds: u64 },
      
      #[error("Resource limit exceeded: {resource}")]
      ResourceLimitExceeded { resource: String },
      
      #[error("Volume mount error: {reason}")]
      VolumeMountError { reason: String },
      
      #[error("Snapshot error: {reason}")]
      SnapshotError { reason: String },
      
      #[error("IO error: {0}")]
      Io(#[from] std::io::Error),
  }
verification:
  command: "cargo check -p cuttlefish-core"
  success: "Finished"
```

##### T3.3: Export trait from cuttlefish-core
```yaml
file: crates/cuttlefish-core/src/lib.rs
action: modify
content_requirements:
  - Export ImageRegistry trait
  - Export ImageSpec, Language, ImageBuildOptions
  - Export SandboxError
verification:
  command: "cargo doc -p cuttlefish-core --no-deps"
  success: "Finished"
```

#### Atomic Commit
```
feat(core): define ImageRegistry trait for sandbox image management

- Add ImageRegistry async trait with list/pull/build/remove methods
- Add ImageSpec, Language, ImageBuildOptions types
- Add comprehensive SandboxError enum with thiserror
- Export all types from cuttlefish-core
```

---

### T4: ImageRegistry Implementation

**Category**: `deep`
**Estimated Hours**: 3
**Dependencies**: T3
**Skills**: Rust, bollard crate, Docker API

#### Work Units

##### T4.1: Implement DockerImageRegistry
```yaml
file: crates/cuttlefish-sandbox/src/images.rs
action: modify
content_requirements:
  - Implement ImageRegistry trait for DockerImageRegistry
  - Use bollard crate for Docker API
  - Cache image list with TTL
  - Map Language enum to image names
  - Handle build streaming output
implementation_skeleton: |
  use bollard::Docker;
  use cuttlefish_core::traits::{ImageRegistry, ImageSpec, Language, ImageBuildOptions};
  use cuttlefish_core::error::SandboxError;
  
  pub struct DockerImageRegistry {
      docker: Docker,
      image_prefix: String,
      cache: tokio::sync::RwLock<ImageCache>,
  }
  
  struct ImageCache {
      images: Vec<ImageSpec>,
      updated_at: std::time::Instant,
  }
  
  impl DockerImageRegistry {
      pub fn new() -> Result<Self, SandboxError> { ... }
      
      fn language_to_image_name(&self, lang: Language) -> String { ... }
  }
  
  #[async_trait]
  impl ImageRegistry for DockerImageRegistry {
      async fn list_images(&self) -> Result<Vec<ImageSpec>, SandboxError> { ... }
      async fn pull_image(&self, name: &str, tag: &str) -> Result<ImageSpec, SandboxError> { ... }
      async fn build_image(&self, name: &str, tag: &str, opts: ImageBuildOptions) -> Result<ImageSpec, SandboxError> { ... }
      async fn remove_image(&self, name: &str, tag: &str) -> Result<(), SandboxError> { ... }
      async fn get_language_image(&self, lang: Language) -> Result<ImageSpec, SandboxError> { ... }
  }
verification:
  command: "cargo check -p cuttlefish-sandbox"
  success: "Finished"
```

##### T4.2: Add MockImageRegistry for testing
```yaml
file: crates/cuttlefish-sandbox/src/images.rs
action: modify (append)
content_requirements:
  - MockImageRegistry struct
  - Configurable responses for testing
  - Track method calls for assertions
mock_implementation: |
  #[cfg(test)]
  pub struct MockImageRegistry {
      images: std::sync::Arc<tokio::sync::RwLock<Vec<ImageSpec>>>,
      calls: std::sync::Arc<tokio::sync::RwLock<Vec<String>>>,
  }
  
  #[cfg(test)]
  impl MockImageRegistry {
      pub fn new() -> Self { ... }
      pub fn with_images(images: Vec<ImageSpec>) -> Self { ... }
      pub async fn get_calls(&self) -> Vec<String> { ... }
  }
  
  #[cfg(test)]
  #[async_trait]
  impl ImageRegistry for MockImageRegistry { ... }
verification:
  command: "cargo test -p cuttlefish-sandbox"
  success: "test result: ok"
```

##### T4.3: Write unit tests for DockerImageRegistry
```yaml
file: crates/cuttlefish-sandbox/src/images.rs
action: modify (append tests module)
content_requirements:
  - Test language_to_image_name mapping
  - Test cache invalidation
  - Test error handling
  - Use MockImageRegistry for unit tests
tests: |
  #[cfg(test)]
  mod tests {
      use super::*;
      
      #[test]
      fn test_language_to_image_name() {
          let registry = DockerImageRegistry::new().unwrap();
          assert_eq!(registry.language_to_image_name(Language::Node), "cuttlefish/node");
          assert_eq!(registry.language_to_image_name(Language::Python), "cuttlefish/python");
      }
      
      #[tokio::test]
      async fn test_mock_registry_tracks_calls() {
          let mock = MockImageRegistry::new();
          let _ = mock.list_images().await;
          let calls = mock.get_calls().await;
          assert!(calls.contains(&"list_images".to_string()));
      }
  }
verification:
  command: "cargo test -p cuttlefish-sandbox images"
  success: "test result: ok"
```

#### Atomic Commit
```
feat(sandbox): implement DockerImageRegistry with caching

- Implement ImageRegistry trait using bollard Docker API
- Add image cache with TTL for performance
- Map Language enum to cuttlefish/* image names
- Add MockImageRegistry for unit testing
- Include comprehensive unit tests
```

---

### T5: SandboxLifecycle Trait Definition

**Category**: `deep`
**Estimated Hours**: 2
**Dependencies**: T3
**Skills**: Rust traits, async design

#### Work Units

##### T5.1: Define SandboxLifecycle trait
```yaml
file: crates/cuttlefish-core/src/traits/sandbox.rs
action: modify (append)
content_requirements:
  - Async trait for container lifecycle
  - Methods: create, start, stop, remove, execute
  - ContainerConfig struct for creation options
  - ExecutionResult struct for command output
  - Support for stdin/stdout/stderr streaming
trait_definition: |
  /// Configuration for creating a sandbox container
  #[derive(Debug, Clone)]
  pub struct ContainerConfig {
      pub image: ImageSpec,
      pub name: Option<String>,
      pub working_dir: PathBuf,
      pub environment: HashMap<String, String>,
      pub resource_limits: ResourceLimits,
      pub volume_mounts: Vec<VolumeMount>,
      pub network_disabled: bool,
  }
  
  /// Resource limits for a container
  #[derive(Debug, Clone, Default)]
  pub struct ResourceLimits {
      pub memory_bytes: Option<u64>,
      pub cpu_quota: Option<i64>,
      pub cpu_period: Option<i64>,
      pub pids_limit: Option<i64>,
  }
  
  /// Volume mount specification
  #[derive(Debug, Clone)]
  pub struct VolumeMount {
      pub host_path: PathBuf,
      pub container_path: PathBuf,
      pub read_only: bool,
  }
  
  /// Result of executing a command in a container
  #[derive(Debug, Clone)]
  pub struct ExecutionResult {
      pub exit_code: i64,
      pub stdout: String,
      pub stderr: String,
      pub duration: std::time::Duration,
  }
  
  /// Handle to a running sandbox container
  #[derive(Debug, Clone)]
  pub struct SandboxHandle {
      pub id: String,
      pub name: String,
      pub image: ImageSpec,
      pub created_at: chrono::DateTime<chrono::Utc>,
  }
  
  /// Lifecycle management for sandbox containers
  #[async_trait]
  pub trait SandboxLifecycle: Send + Sync {
      /// Create a new sandbox container (does not start it)
      async fn create(&self, config: ContainerConfig) -> Result<SandboxHandle, SandboxError>;
      
      /// Start a created container
      async fn start(&self, handle: &SandboxHandle) -> Result<(), SandboxError>;
      
      /// Stop a running container
      async fn stop(&self, handle: &SandboxHandle, timeout_secs: u64) -> Result<(), SandboxError>;
      
      /// Remove a container (must be stopped first)
      async fn remove(&self, handle: &SandboxHandle) -> Result<(), SandboxError>;
      
      /// Execute a command in a running container
      async fn execute(
          &self,
          handle: &SandboxHandle,
          command: &[String],
          timeout: std::time::Duration,
      ) -> Result<ExecutionResult, SandboxError>;
      
      /// Get current status of a container
      async fn status(&self, handle: &SandboxHandle) -> Result<ContainerStatus, SandboxError>;
  }
  
  /// Status of a container
  #[derive(Debug, Clone, Copy, PartialEq, Eq)]
  pub enum ContainerStatus {
      Created,
      Running,
      Paused,
      Stopped,
      Removed,
  }
verification:
  command: "cargo check -p cuttlefish-core"
  success: "Finished"
```

##### T5.2: Export lifecycle types
```yaml
file: crates/cuttlefish-core/src/lib.rs
action: modify
content_requirements:
  - Export SandboxLifecycle trait
  - Export ContainerConfig, ResourceLimits, VolumeMount
  - Export ExecutionResult, SandboxHandle, ContainerStatus
verification:
  command: "cargo doc -p cuttlefish-core --no-deps"
  success: "Finished"
```

#### Atomic Commit
```
feat(core): define SandboxLifecycle trait for container management

- Add SandboxLifecycle async trait with create/start/stop/remove/execute
- Add ContainerConfig with resource limits and volume mounts
- Add ExecutionResult for capturing command output
- Add SandboxHandle for referencing containers
- Add ContainerStatus enum for state tracking
```

---

### T6: SandboxLifecycle Implementation

**Category**: `deep`
**Estimated Hours**: 3
**Dependencies**: T4, T5
**Skills**: Rust, bollard, Docker containers

#### Work Units

##### T6.1: Implement DockerSandboxLifecycle
```yaml
file: crates/cuttlefish-sandbox/src/lifecycle.rs
action: create
content_requirements:
  - Implement SandboxLifecycle for Docker
  - Use bollard for container operations
  - Map ContainerConfig to Docker API format
  - Handle execution with timeout
  - Stream stdout/stderr
implementation_skeleton: |
  use bollard::Docker;
  use bollard::container::{Config, CreateContainerOptions, StartContainerOptions};
  use cuttlefish_core::traits::*;
  use cuttlefish_core::error::SandboxError;
  
  pub struct DockerSandboxLifecycle {
      docker: Docker,
  }
  
  impl DockerSandboxLifecycle {
      pub fn new() -> Result<Self, SandboxError> {
          let docker = Docker::connect_with_local_defaults()?;
          Ok(Self { docker })
      }
      
      fn build_container_config(&self, config: &ContainerConfig) -> Config<String> { ... }
  }
  
  #[async_trait]
  impl SandboxLifecycle for DockerSandboxLifecycle {
      async fn create(&self, config: ContainerConfig) -> Result<SandboxHandle, SandboxError> { ... }
      async fn start(&self, handle: &SandboxHandle) -> Result<(), SandboxError> { ... }
      async fn stop(&self, handle: &SandboxHandle, timeout_secs: u64) -> Result<(), SandboxError> { ... }
      async fn remove(&self, handle: &SandboxHandle) -> Result<(), SandboxError> { ... }
      async fn execute(&self, handle: &SandboxHandle, command: &[String], timeout: Duration) -> Result<ExecutionResult, SandboxError> { ... }
      async fn status(&self, handle: &SandboxHandle) -> Result<ContainerStatus, SandboxError> { ... }
  }
verification:
  command: "cargo check -p cuttlefish-sandbox"
  success: "Finished"
```

##### T6.2: Add MockSandboxLifecycle
```yaml
file: crates/cuttlefish-sandbox/src/lifecycle.rs
action: modify (append)
content_requirements:
  - MockSandboxLifecycle for testing
  - Configurable execution results
  - Track lifecycle events
mock_implementation: |
  #[cfg(test)]
  pub struct MockSandboxLifecycle {
      containers: Arc<RwLock<HashMap<String, MockContainer>>>,
      execution_results: Arc<RwLock<VecDeque<ExecutionResult>>>,
  }
  
  #[cfg(test)]
  struct MockContainer {
      handle: SandboxHandle,
      status: ContainerStatus,
  }
  
  #[cfg(test)]
  impl MockSandboxLifecycle {
      pub fn new() -> Self { ... }
      pub fn with_execution_result(result: ExecutionResult) -> Self { ... }
  }
verification:
  command: "cargo test -p cuttlefish-sandbox lifecycle"
  success: "test result: ok"
```

##### T6.3: Write lifecycle unit tests
```yaml
file: crates/cuttlefish-sandbox/src/lifecycle.rs
action: modify (append tests)
content_requirements:
  - Test container state transitions
  - Test execution timeout handling
  - Test error conditions
tests: |
  #[cfg(test)]
  mod tests {
      use super::*;
      
      #[tokio::test]
      async fn test_container_lifecycle() {
          let mock = MockSandboxLifecycle::new();
          let config = ContainerConfig::default();
          
          let handle = mock.create(config).await.unwrap();
          assert_eq!(mock.status(&handle).await.unwrap(), ContainerStatus::Created);
          
          mock.start(&handle).await.unwrap();
          assert_eq!(mock.status(&handle).await.unwrap(), ContainerStatus::Running);
          
          mock.stop(&handle, 10).await.unwrap();
          assert_eq!(mock.status(&handle).await.unwrap(), ContainerStatus::Stopped);
      }
      
      #[tokio::test]
      async fn test_execution_returns_result() {
          let result = ExecutionResult {
              exit_code: 0,
              stdout: "hello".to_string(),
              stderr: "".to_string(),
              duration: Duration::from_millis(100),
          };
          let mock = MockSandboxLifecycle::with_execution_result(result.clone());
          // ... test execution
      }
  }
verification:
  command: "cargo test -p cuttlefish-sandbox lifecycle"
  success: "test result: ok"
```

##### T6.4: Add lifecycle module to lib.rs
```yaml
file: crates/cuttlefish-sandbox/src/lib.rs
action: modify
content_requirements:
  - Add mod lifecycle
  - Export DockerSandboxLifecycle
  - Export MockSandboxLifecycle under #[cfg(test)]
verification:
  command: "cargo check -p cuttlefish-sandbox"
  success: "Finished"
```

#### Atomic Commit
```
feat(sandbox): implement DockerSandboxLifecycle for container management

- Implement SandboxLifecycle trait using bollard
- Handle container create/start/stop/remove operations
- Execute commands with timeout and output capture
- Add MockSandboxLifecycle for unit testing
- Include lifecycle state transition tests
```

---

### T7: Volume Mount System

**Category**: `deep`
**Estimated Hours**: 3
**Dependencies**: T6
**Skills**: Rust, filesystem, Docker volumes

#### Work Units

##### T7.1: Create VolumeManager trait
```yaml
file: crates/cuttlefish-core/src/traits/sandbox.rs
action: modify (append)
content_requirements:
  - Trait for managing volume mounts
  - Create temporary volumes
  - Copy files into volumes
  - Clean up volumes
trait_definition: |
  /// Manager for sandbox volume mounts
  #[async_trait]
  pub trait VolumeManager: Send + Sync {
      /// Create a temporary volume for a sandbox
      async fn create_volume(&self, name: &str) -> Result<VolumeHandle, SandboxError>;
      
      /// Copy files from host to volume
      async fn copy_to_volume(
          &self,
          volume: &VolumeHandle,
          host_path: &Path,
          container_path: &Path,
      ) -> Result<(), SandboxError>;
      
      /// Copy files from volume to host
      async fn copy_from_volume(
          &self,
          volume: &VolumeHandle,
          container_path: &Path,
          host_path: &Path,
      ) -> Result<(), SandboxError>;
      
      /// Remove a volume
      async fn remove_volume(&self, volume: &VolumeHandle) -> Result<(), SandboxError>;
      
      /// List all sandbox volumes
      async fn list_volumes(&self) -> Result<Vec<VolumeHandle>, SandboxError>;
  }
  
  /// Handle to a volume
  #[derive(Debug, Clone)]
  pub struct VolumeHandle {
      pub name: String,
      pub mount_point: PathBuf,
      pub created_at: chrono::DateTime<chrono::Utc>,
  }
verification:
  command: "cargo check -p cuttlefish-core"
  success: "Finished"
```

##### T7.2: Implement DockerVolumeManager
```yaml
file: crates/cuttlefish-sandbox/src/volumes.rs
action: create
content_requirements:
  - Implement VolumeManager for Docker
  - Use Docker volumes API
  - Handle file copying via tar archives
  - Prefix volumes with cuttlefish-
implementation_skeleton: |
  pub struct DockerVolumeManager {
      docker: Docker,
      volume_prefix: String,
  }
  
  impl DockerVolumeManager {
      pub fn new() -> Result<Self, SandboxError> { ... }
  }
  
  #[async_trait]
  impl VolumeManager for DockerVolumeManager {
      async fn create_volume(&self, name: &str) -> Result<VolumeHandle, SandboxError> { ... }
      async fn copy_to_volume(&self, ...) -> Result<(), SandboxError> { ... }
      async fn copy_from_volume(&self, ...) -> Result<(), SandboxError> { ... }
      async fn remove_volume(&self, volume: &VolumeHandle) -> Result<(), SandboxError> { ... }
      async fn list_volumes(&self) -> Result<Vec<VolumeHandle>, SandboxError> { ... }
  }
verification:
  command: "cargo check -p cuttlefish-sandbox"
  success: "Finished"
```

##### T7.3: Write volume integration tests
```yaml
file: crates/cuttlefish-sandbox/tests/volumes.rs
action: create
content_requirements:
  - Test volume creation and cleanup
  - Test file copy round-trip
  - Verify volume isolation
tests: |
  #[cfg(feature = "integration")]
  mod volume_tests {
      use cuttlefish_sandbox::volumes::DockerVolumeManager;
      use cuttlefish_core::traits::VolumeManager;
      use tempfile::TempDir;
      
      #[tokio::test]
      async fn test_volume_lifecycle() {
          let manager = DockerVolumeManager::new().unwrap();
          let volume = manager.create_volume("test-vol").await.unwrap();
          
          // Volume should exist
          let volumes = manager.list_volumes().await.unwrap();
          assert!(volumes.iter().any(|v| v.name == volume.name));
          
          // Cleanup
          manager.remove_volume(&volume).await.unwrap();
      }
      
      #[tokio::test]
      async fn test_file_copy_roundtrip() {
          let manager = DockerVolumeManager::new().unwrap();
          let volume = manager.create_volume("copy-test").await.unwrap();
          let temp_dir = TempDir::new().unwrap();
          
          // Create test file
          let src_file = temp_dir.path().join("test.txt");
          std::fs::write(&src_file, "hello world").unwrap();
          
          // Copy to volume
          manager.copy_to_volume(&volume, &src_file, Path::new("/data/test.txt")).await.unwrap();
          
          // Copy back
          let dst_file = temp_dir.path().join("test_copy.txt");
          manager.copy_from_volume(&volume, Path::new("/data/test.txt"), &dst_file).await.unwrap();
          
          assert_eq!(std::fs::read_to_string(dst_file).unwrap(), "hello world");
          
          manager.remove_volume(&volume).await.unwrap();
      }
  }
verification:
  command: "cargo test -p cuttlefish-sandbox --features integration volumes"
  success: "test result: ok"
```

##### T7.4: Export volume module
```yaml
file: crates/cuttlefish-sandbox/src/lib.rs
action: modify
content_requirements:
  - Add mod volumes
  - Export DockerVolumeManager
verification:
  command: "cargo check -p cuttlefish-sandbox"
  success: "Finished"
```

#### Atomic Commit
```
feat(sandbox): add volume mount system for file sharing

- Define VolumeManager trait in cuttlefish-core
- Implement DockerVolumeManager using Docker volumes API
- Support file copy to/from volumes via tar archives
- Add integration tests for volume lifecycle
- Prefix all volumes with cuttlefish- for identification
```

---

### T8: Resource Limits

**Category**: `deep`
**Estimated Hours**: 3
**Dependencies**: T6
**Skills**: Rust, Docker cgroups, resource management

#### Work Units

##### T8.1: Extend ResourceLimits struct
```yaml
file: crates/cuttlefish-core/src/traits/sandbox.rs
action: modify
content_requirements:
  - Add disk quota limit
  - Add network bandwidth limit
  - Add process limit
  - Add timeout limit
  - Builder pattern for ResourceLimits
extended_struct: |
  #[derive(Debug, Clone)]
  pub struct ResourceLimits {
      /// Memory limit in bytes (e.g., 512MB = 512 * 1024 * 1024)
      pub memory_bytes: Option<u64>,
      /// CPU quota (microseconds per cpu_period)
      pub cpu_quota: Option<i64>,
      /// CPU period (default 100000 = 100ms)
      pub cpu_period: Option<i64>,
      /// Maximum number of processes
      pub pids_limit: Option<i64>,
      /// Disk quota in bytes
      pub disk_bytes: Option<u64>,
      /// Execution timeout
      pub timeout: Option<std::time::Duration>,
      /// Read-only root filesystem
      pub read_only_rootfs: bool,
      /// Disable network access
      pub network_disabled: bool,
  }
  
  impl ResourceLimits {
      pub fn builder() -> ResourceLimitsBuilder { ... }
      
      /// Preset for lightweight tasks (256MB, 0.5 CPU, 30s)
      pub fn light() -> Self { ... }
      
      /// Preset for standard tasks (512MB, 1 CPU, 60s)
      pub fn standard() -> Self { ... }
      
      /// Preset for heavy tasks (2GB, 2 CPU, 300s)
      pub fn heavy() -> Self { ... }
  }
  
  pub struct ResourceLimitsBuilder { ... }
verification:
  command: "cargo check -p cuttlefish-core"
  success: "Finished"
```

##### T8.2: Implement resource limit application
```yaml
file: crates/cuttlefish-sandbox/src/resources.rs
action: create
content_requirements:
  - ResourceEnforcer struct
  - Apply limits to Docker container config
  - Monitor resource usage during execution
  - Kill containers exceeding limits
implementation: |
  use bollard::container::Config;
  use cuttlefish_core::traits::ResourceLimits;
  
  pub struct ResourceEnforcer {
      docker: Docker,
  }
  
  impl ResourceEnforcer {
      pub fn new(docker: Docker) -> Self { ... }
      
      /// Apply resource limits to container config
      pub fn apply_limits(&self, config: &mut Config<String>, limits: &ResourceLimits) {
          if let Some(mem) = limits.memory_bytes {
              config.host_config.as_mut().map(|h| h.memory = Some(mem as i64));
          }
          if let Some(cpu_quota) = limits.cpu_quota {
              config.host_config.as_mut().map(|h| h.cpu_quota = Some(cpu_quota));
          }
          // ... apply other limits
      }
      
      /// Check if container is within limits
      pub async fn check_limits(&self, container_id: &str, limits: &ResourceLimits) -> Result<bool, SandboxError> { ... }
      
      /// Get current resource usage
      pub async fn get_usage(&self, container_id: &str) -> Result<ResourceUsage, SandboxError> { ... }
  }
  
  #[derive(Debug, Clone)]
  pub struct ResourceUsage {
      pub memory_bytes: u64,
      pub cpu_percent: f64,
      pub pids: u64,
  }
verification:
  command: "cargo check -p cuttlefish-sandbox"
  success: "Finished"
```

##### T8.3: Write resource limit tests
```yaml
file: crates/cuttlefish-sandbox/tests/resources.rs
action: create
content_requirements:
  - Test memory limit enforcement
  - Test CPU limit enforcement
  - Test timeout enforcement
  - Test OOM kill behavior
tests: |
  #[cfg(feature = "integration")]
  mod resource_tests {
      use cuttlefish_sandbox::*;
      
      #[tokio::test]
      async fn test_memory_limit_kills_container() {
          let sandbox = DockerSandboxLifecycle::new().unwrap();
          let config = ContainerConfig {
              image: get_test_image().await,
              resource_limits: ResourceLimits {
                  memory_bytes: Some(64 * 1024 * 1024), // 64MB
                  ..Default::default()
              },
              ..Default::default()
          };
          
          let handle = sandbox.create(config).await.unwrap();
          sandbox.start(&handle).await.unwrap();
          
          // Try to allocate more memory than allowed
          let result = sandbox.execute(
              &handle,
              &["python3", "-c", "x = 'a' * (100 * 1024 * 1024)"],
              Duration::from_secs(10),
          ).await;
          
          // Should fail due to OOM
          assert!(result.is_err() || result.unwrap().exit_code != 0);
          
          sandbox.remove(&handle).await.unwrap();
      }
      
      #[tokio::test]
      async fn test_timeout_kills_execution() {
          let sandbox = DockerSandboxLifecycle::new().unwrap();
          let config = ContainerConfig::default();
          
          let handle = sandbox.create(config).await.unwrap();
          sandbox.start(&handle).await.unwrap();
          
          let result = sandbox.execute(
              &handle,
              &["sleep", "60"],
              Duration::from_secs(2), // 2 second timeout
          ).await;
          
          assert!(matches!(result, Err(SandboxError::Timeout { .. })));
          
          sandbox.remove(&handle).await.unwrap();
      }
  }
verification:
  command: "cargo test -p cuttlefish-sandbox --features integration resources"
  success: "test result: ok"
```

##### T8.4: Export resources module
```yaml
file: crates/cuttlefish-sandbox/src/lib.rs
action: modify
content_requirements:
  - Add mod resources
  - Export ResourceEnforcer, ResourceUsage
verification:
  command: "cargo check -p cuttlefish-sandbox"
  success: "Finished"
```

#### Atomic Commit
```
feat(sandbox): implement resource limits with enforcement

- Extend ResourceLimits with disk, network, timeout options
- Add preset configurations (light, standard, heavy)
- Implement ResourceEnforcer for applying Docker limits
- Monitor resource usage and kill exceeding containers
- Add integration tests for memory/CPU/timeout limits
```

---

### T9: Container Snapshots

**Category**: `ultrabrain`
**Estimated Hours**: 4
**Dependencies**: T7, T8
**Skills**: Rust, Docker commits, container state

#### Work Units

##### T9.1: Define Snapshot trait
```yaml
file: crates/cuttlefish-core/src/traits/sandbox.rs
action: modify (append)
content_requirements:
  - SnapshotManager trait
  - Create snapshots from running containers
  - Restore containers from snapshots
  - List and delete snapshots
trait_definition: |
  /// Snapshot of a container state
  #[derive(Debug, Clone)]
  pub struct Snapshot {
      pub id: String,
      pub name: String,
      pub container_id: String,
      pub created_at: chrono::DateTime<chrono::Utc>,
      pub size_bytes: u64,
      pub labels: HashMap<String, String>,
  }
  
  /// Options for creating a snapshot
  #[derive(Debug, Clone, Default)]
  pub struct SnapshotOptions {
      pub name: Option<String>,
      pub labels: HashMap<String, String>,
      pub pause_container: bool,
  }
  
  /// Manager for container snapshots
  #[async_trait]
  pub trait SnapshotManager: Send + Sync {
      /// Create a snapshot of a container's current state
      async fn create_snapshot(
          &self,
          handle: &SandboxHandle,
          options: SnapshotOptions,
      ) -> Result<Snapshot, SandboxError>;
      
      /// Restore a container from a snapshot
      async fn restore_snapshot(
          &self,
          snapshot: &Snapshot,
          config: ContainerConfig,
      ) -> Result<SandboxHandle, SandboxError>;
      
      /// List all snapshots
      async fn list_snapshots(&self) -> Result<Vec<Snapshot>, SandboxError>;
      
      /// List snapshots for a specific container
      async fn list_container_snapshots(
          &self,
          container_id: &str,
      ) -> Result<Vec<Snapshot>, SandboxError>;
      
      /// Delete a snapshot
      async fn delete_snapshot(&self, snapshot: &Snapshot) -> Result<(), SandboxError>;
  }
verification:
  command: "cargo check -p cuttlefish-core"
  success: "Finished"
```

##### T9.2: Implement DockerSnapshotManager
```yaml
file: crates/cuttlefish-sandbox/src/snapshots.rs
action: create
content_requirements:
  - Implement SnapshotManager using docker commit
  - Store snapshots as images with labels
  - Handle snapshot naming conventions
  - Support incremental snapshots
implementation: |
  use bollard::Docker;
  use bollard::image::CommitContainerOptions;
  
  pub struct DockerSnapshotManager {
      docker: Docker,
      snapshot_prefix: String,
  }
  
  impl DockerSnapshotManager {
      pub fn new() -> Result<Self, SandboxError> {
          Ok(Self {
              docker: Docker::connect_with_local_defaults()?,
              snapshot_prefix: "cuttlefish-snapshot".to_string(),
          })
      }
      
      fn snapshot_image_name(&self, name: &str) -> String {
          format!("{}:{}", self.snapshot_prefix, name)
      }
  }
  
  #[async_trait]
  impl SnapshotManager for DockerSnapshotManager {
      async fn create_snapshot(&self, handle: &SandboxHandle, options: SnapshotOptions) -> Result<Snapshot, SandboxError> {
          // Optionally pause container
          if options.pause_container {
              self.docker.pause_container(&handle.id).await?;
          }
          
          // Commit container to image
          let name = options.name.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
          let commit_options = CommitContainerOptions {
              container: handle.id.clone(),
              repo: self.snapshot_prefix.clone(),
              tag: name.clone(),
              ..Default::default()
          };
          
          let result = self.docker.commit_container(commit_options, Default::default()).await?;
          
          // Resume if paused
          if options.pause_container {
              self.docker.unpause_container(&handle.id).await?;
          }
          
          Ok(Snapshot {
              id: result.id,
              name,
              container_id: handle.id.clone(),
              created_at: chrono::Utc::now(),
              size_bytes: 0, // TODO: get actual size
              labels: options.labels,
          })
      }
      
      async fn restore_snapshot(&self, snapshot: &Snapshot, config: ContainerConfig) -> Result<SandboxHandle, SandboxError> {
          // Create new container from snapshot image
          // ... implementation
      }
      
      // ... other methods
  }
verification:
  command: "cargo check -p cuttlefish-sandbox"
  success: "Finished"
```

##### T9.3: Write snapshot integration tests
```yaml
file: crates/cuttlefish-sandbox/tests/snapshots.rs
action: create
content_requirements:
  - Test snapshot creation
  - Test restore from snapshot
  - Test snapshot listing
  - Test snapshot cleanup
tests: |
  #[cfg(feature = "integration")]
  mod snapshot_tests {
      use cuttlefish_sandbox::*;
      
      #[tokio::test]
      async fn test_snapshot_and_restore() {
          let lifecycle = DockerSandboxLifecycle::new().unwrap();
          let snapshots = DockerSnapshotManager::new().unwrap();
          
          // Create container and make changes
          let config = ContainerConfig::default();
          let handle = lifecycle.create(config.clone()).await.unwrap();
          lifecycle.start(&handle).await.unwrap();
          
          // Create a file in the container
          lifecycle.execute(&handle, &["touch", "/workspace/test.txt"], Duration::from_secs(5)).await.unwrap();
          
          // Take snapshot
          let snapshot = snapshots.create_snapshot(&handle, SnapshotOptions::default()).await.unwrap();
          
          // Stop and remove original
          lifecycle.stop(&handle, 10).await.unwrap();
          lifecycle.remove(&handle).await.unwrap();
          
          // Restore from snapshot
          let restored = snapshots.restore_snapshot(&snapshot, config).await.unwrap();
          lifecycle.start(&restored).await.unwrap();
          
          // Verify file exists
          let result = lifecycle.execute(&restored, &["test", "-f", "/workspace/test.txt"], Duration::from_secs(5)).await.unwrap();
          assert_eq!(result.exit_code, 0);
          
          // Cleanup
          lifecycle.stop(&restored, 10).await.unwrap();
          lifecycle.remove(&restored).await.unwrap();
          snapshots.delete_snapshot(&snapshot).await.unwrap();
      }
  }
verification:
  command: "cargo test -p cuttlefish-sandbox --features integration snapshots"
  success: "test result: ok"
```

##### T9.4: Export snapshots module
```yaml
file: crates/cuttlefish-sandbox/src/lib.rs
action: modify
content_requirements:
  - Add mod snapshots
  - Export DockerSnapshotManager
  - Export Snapshot, SnapshotOptions
verification:
  command: "cargo check -p cuttlefish-sandbox"
  success: "Finished"
```

#### Atomic Commit
```
feat(sandbox): implement container snapshot and restore

- Define SnapshotManager trait for container state management
- Implement DockerSnapshotManager using docker commit
- Support pausing containers during snapshot for consistency
- Store snapshots as labeled Docker images
- Add integration tests for snapshot/restore workflow
```

---

### T10: Cleanup and Garbage Collection

**Category**: `deep`
**Estimated Hours**: 2
**Dependencies**: T9
**Skills**: Rust, async scheduling, Docker cleanup

#### Work Units

##### T10.1: Define Cleanup trait
```yaml
file: crates/cuttlefish-core/src/traits/sandbox.rs
action: modify (append)
content_requirements:
  - CleanupManager trait
  - Remove stale containers
  - Remove unused volumes
  - Remove old snapshots
  - Configurable retention policies
trait_definition: |
  /// Policy for cleaning up sandbox resources
  #[derive(Debug, Clone)]
  pub struct CleanupPolicy {
      /// Remove containers older than this duration
      pub container_max_age: std::time::Duration,
      /// Remove snapshots older than this duration
      pub snapshot_max_age: std::time::Duration,
      /// Remove volumes not attached to any container
      pub remove_orphan_volumes: bool,
      /// Maximum number of snapshots to keep per container
      pub max_snapshots_per_container: Option<usize>,
  }
  
  impl Default for CleanupPolicy {
      fn default() -> Self {
          Self {
              container_max_age: std::time::Duration::from_secs(3600), // 1 hour
              snapshot_max_age: std::time::Duration::from_secs(86400), // 24 hours
              remove_orphan_volumes: true,
              max_snapshots_per_container: Some(5),
          }
      }
  }
  
  /// Result of a cleanup operation
  #[derive(Debug, Clone, Default)]
  pub struct CleanupResult {
      pub containers_removed: usize,
      pub volumes_removed: usize,
      pub snapshots_removed: usize,
      pub bytes_reclaimed: u64,
      pub errors: Vec<String>,
  }
  
  /// Manager for cleaning up sandbox resources
  #[async_trait]
  pub trait CleanupManager: Send + Sync {
      /// Run cleanup with the given policy
      async fn cleanup(&self, policy: &CleanupPolicy) -> Result<CleanupResult, SandboxError>;
      
      /// Get current resource usage
      async fn get_usage(&self) -> Result<SandboxUsage, SandboxError>;
      
      /// Force remove all sandbox resources (dangerous!)
      async fn force_cleanup_all(&self) -> Result<CleanupResult, SandboxError>;
  }
  
  /// Current sandbox resource usage
  #[derive(Debug, Clone)]
  pub struct SandboxUsage {
      pub container_count: usize,
      pub volume_count: usize,
      pub snapshot_count: usize,
      pub total_bytes: u64,
  }
verification:
  command: "cargo check -p cuttlefish-core"
  success: "Finished"
```

##### T10.2: Implement DockerCleanupManager
```yaml
file: crates/cuttlefish-sandbox/src/cleanup.rs
action: create
content_requirements:
  - Implement CleanupManager for Docker
  - Filter resources by cuttlefish- prefix
  - Handle cleanup errors gracefully
  - Log cleanup actions
implementation: |
  use tracing::{info, warn};
  
  pub struct DockerCleanupManager {
      docker: Docker,
      resource_prefix: String,
  }
  
  impl DockerCleanupManager {
      pub fn new() -> Result<Self, SandboxError> {
          Ok(Self {
              docker: Docker::connect_with_local_defaults()?,
              resource_prefix: "cuttlefish".to_string(),
          })
      }
      
      async fn cleanup_containers(&self, max_age: Duration) -> Result<(usize, Vec<String>), SandboxError> {
          let containers = self.docker.list_containers(Some(ListContainersOptions {
              all: true,
              filters: HashMap::from([("label", vec!["cuttlefish=sandbox"])]),
              ..Default::default()
          })).await?;
          
          let mut removed = 0;
          let mut errors = vec![];
          let cutoff = chrono::Utc::now() - chrono::Duration::from_std(max_age).unwrap();
          
          for container in containers {
              if let Some(created) = container.created {
                  let created_time = chrono::DateTime::from_timestamp(created, 0).unwrap();
                  if created_time < cutoff {
                      match self.docker.remove_container(&container.id.unwrap(), None).await {
                          Ok(_) => {
                              removed += 1;
                              info!(container_id = ?container.id, "Removed stale container");
                          }
                          Err(e) => {
                              errors.push(format!("Failed to remove container: {e}"));
                              warn!(error = %e, "Failed to remove container");
                          }
                      }
                  }
              }
          }
          
          Ok((removed, errors))
      }
      
      // ... similar methods for volumes and snapshots
  }
  
  #[async_trait]
  impl CleanupManager for DockerCleanupManager {
      async fn cleanup(&self, policy: &CleanupPolicy) -> Result<CleanupResult, SandboxError> {
          let mut result = CleanupResult::default();
          
          let (containers, container_errors) = self.cleanup_containers(policy.container_max_age).await?;
          result.containers_removed = containers;
          result.errors.extend(container_errors);
          
          // ... cleanup volumes and snapshots
          
          info!(
              containers = result.containers_removed,
              volumes = result.volumes_removed,
              snapshots = result.snapshots_removed,
              "Cleanup completed"
          );
          
          Ok(result)
      }
      
      // ... other methods
  }
verification:
  command: "cargo check -p cuttlefish-sandbox"
  success: "Finished"
```

##### T10.3: Write cleanup tests
```yaml
file: crates/cuttlefish-sandbox/src/cleanup.rs
action: modify (append tests)
content_requirements:
  - Test stale container cleanup
  - Test orphan volume cleanup
  - Test snapshot retention
tests: |
  #[cfg(test)]
  mod tests {
      use super::*;
      
      #[tokio::test]
      async fn test_cleanup_policy_defaults() {
          let policy = CleanupPolicy::default();
          assert_eq!(policy.container_max_age, Duration::from_secs(3600));
          assert!(policy.remove_orphan_volumes);
          assert_eq!(policy.max_snapshots_per_container, Some(5));
      }
  }
  
  #[cfg(all(test, feature = "integration"))]
  mod integration_tests {
      use super::*;
      
      #[tokio::test]
      async fn test_cleanup_removes_old_containers() {
          // This test would need to mock time or use very short durations
          let cleanup = DockerCleanupManager::new().unwrap();
          let usage_before = cleanup.get_usage().await.unwrap();
          
          // ... create some resources ...
          
          let policy = CleanupPolicy {
              container_max_age: Duration::from_secs(0), // Immediate cleanup
              ..Default::default()
          };
          
          let result = cleanup.cleanup(&policy).await.unwrap();
          let usage_after = cleanup.get_usage().await.unwrap();
          
          assert!(usage_after.container_count <= usage_before.container_count);
      }
  }
verification:
  command: "cargo test -p cuttlefish-sandbox cleanup"
  success: "test result: ok"
```

##### T10.4: Export cleanup module
```yaml
file: crates/cuttlefish-sandbox/src/lib.rs
action: modify
content_requirements:
  - Add mod cleanup
  - Export DockerCleanupManager
  - Export CleanupPolicy, CleanupResult, SandboxUsage
verification:
  command: "cargo check -p cuttlefish-sandbox"
  success: "Finished"
```

#### Atomic Commit
```
feat(sandbox): implement cleanup and garbage collection

- Define CleanupManager trait with configurable retention policies
- Implement DockerCleanupManager for resource cleanup
- Remove stale containers, orphan volumes, old snapshots
- Log all cleanup actions via tracing
- Add unit and integration tests for cleanup
```

---

### T11: Health Checks

**Category**: `quick`
**Estimated Hours**: 2
**Dependencies**: T10
**Skills**: Rust, health monitoring

#### Work Units

##### T11.1: Define HealthCheck trait
```yaml
file: crates/cuttlefish-core/src/traits/sandbox.rs
action: modify (append)
content_requirements:
  - HealthChecker trait
  - Check Docker daemon connectivity
  - Check image availability
  - Check resource availability
trait_definition: |
  /// Health status of the sandbox system
  #[derive(Debug, Clone)]
  pub struct SandboxHealth {
      pub docker_healthy: bool,
      pub images_available: HashMap<Language, bool>,
      pub resource_usage: SandboxUsage,
      pub errors: Vec<String>,
  }
  
  impl SandboxHealth {
      pub fn is_healthy(&self) -> bool {
          self.docker_healthy && self.images_available.values().all(|&v| v) && self.errors.is_empty()
      }
  }
  
  /// Health checker for sandbox system
  #[async_trait]
  pub trait HealthChecker: Send + Sync {
      /// Perform comprehensive health check
      async fn check_health(&self) -> Result<SandboxHealth, SandboxError>;
      
      /// Quick ping to verify basic connectivity
      async fn ping(&self) -> Result<bool, SandboxError>;
  }
verification:
  command: "cargo check -p cuttlefish-core"
  success: "Finished"
```

##### T11.2: Implement DockerHealthChecker
```yaml
file: crates/cuttlefish-sandbox/src/health.rs
action: create
content_requirements:
  - Implement HealthChecker for Docker
  - Check Docker daemon via ping
  - Verify all language images exist
  - Report resource usage
implementation: |
  pub struct DockerHealthChecker {
      docker: Docker,
      image_registry: Arc<dyn ImageRegistry>,
  }
  
  impl DockerHealthChecker {
      pub fn new(image_registry: Arc<dyn ImageRegistry>) -> Result<Self, SandboxError> {
          Ok(Self {
              docker: Docker::connect_with_local_defaults()?,
              image_registry,
          })
      }
  }
  
  #[async_trait]
  impl HealthChecker for DockerHealthChecker {
      async fn check_health(&self) -> Result<SandboxHealth, SandboxError> {
          let mut health = SandboxHealth {
              docker_healthy: false,
              images_available: HashMap::new(),
              resource_usage: SandboxUsage::default(),
              errors: vec![],
          };
          
          // Check Docker connectivity
          match self.docker.ping().await {
              Ok(_) => health.docker_healthy = true,
              Err(e) => health.errors.push(format!("Docker ping failed: {e}")),
          }
          
          // Check each language image
          for lang in [Language::Node, Language::Python, Language::Rust, Language::Go, Language::Ruby] {
              match self.image_registry.get_language_image(lang).await {
                  Ok(_) => { health.images_available.insert(lang, true); }
                  Err(_) => { health.images_available.insert(lang, false); }
              }
          }
          
          Ok(health)
      }
      
      async fn ping(&self) -> Result<bool, SandboxError> {
          self.docker.ping().await.map(|_| true).map_err(SandboxError::from)
      }
  }
verification:
  command: "cargo check -p cuttlefish-sandbox"
  success: "Finished"
```

##### T11.3: Write health check tests
```yaml
file: crates/cuttlefish-sandbox/src/health.rs
action: modify (append tests)
tests: |
  #[cfg(test)]
  mod tests {
      use super::*;
      
      #[test]
      fn test_sandbox_health_is_healthy() {
          let mut health = SandboxHealth {
              docker_healthy: true,
              images_available: HashMap::from([
                  (Language::Node, true),
                  (Language::Python, true),
              ]),
              resource_usage: SandboxUsage::default(),
              errors: vec![],
          };
          assert!(health.is_healthy());
          
          health.docker_healthy = false;
          assert!(!health.is_healthy());
      }
  }
verification:
  command: "cargo test -p cuttlefish-sandbox health"
  success: "test result: ok"
```

##### T11.4: Export health module
```yaml
file: crates/cuttlefish-sandbox/src/lib.rs
action: modify
content_requirements:
  - Add mod health
  - Export DockerHealthChecker
  - Export SandboxHealth
verification:
  command: "cargo check -p cuttlefish-sandbox"
  success: "Finished"
```

#### Atomic Commit
```
feat(sandbox): add health checking system

- Define HealthChecker trait for system monitoring
- Implement DockerHealthChecker for Docker-based checks
- Check daemon connectivity, image availability, resource usage
- Add is_healthy() convenience method
- Include unit tests for health status logic
```

---

### T12: API Routes

**Category**: `deep`
**Estimated Hours**: 3
**Dependencies**: T11
**Skills**: Rust, Axum, REST API

#### Work Units

##### T12.1: Create sandbox API routes
```yaml
file: crates/cuttlefish-api/src/routes/sandbox.rs
action: create
content_requirements:
  - POST /sandbox/create - create new sandbox
  - POST /sandbox/:id/execute - execute command
  - POST /sandbox/:id/snapshot - create snapshot
  - GET /sandbox/:id/status - get status
  - DELETE /sandbox/:id - remove sandbox
  - GET /sandbox/health - health check
routes: |
  use axum::{
      routing::{get, post, delete},
      Router, Json, extract::{Path, State},
  };
  use cuttlefish_sandbox::*;
  
  pub fn sandbox_routes() -> Router<AppState> {
      Router::new()
          .route("/sandbox/create", post(create_sandbox))
          .route("/sandbox/:id/execute", post(execute_command))
          .route("/sandbox/:id/snapshot", post(create_snapshot))
          .route("/sandbox/:id/status", get(get_status))
          .route("/sandbox/:id", delete(remove_sandbox))
          .route("/sandbox/health", get(health_check))
  }
  
  #[derive(Debug, Deserialize)]
  pub struct CreateSandboxRequest {
      pub language: String,
      pub resource_preset: Option<String>, // "light", "standard", "heavy"
  }
  
  #[derive(Debug, Serialize)]
  pub struct CreateSandboxResponse {
      pub id: String,
      pub name: String,
      pub status: String,
  }
  
  async fn create_sandbox(
      State(state): State<AppState>,
      Json(req): Json<CreateSandboxRequest>,
  ) -> Result<Json<CreateSandboxResponse>, ApiError> {
      // Parse language
      let language = match req.language.as_str() {
          "node" | "nodejs" => Language::Node,
          "python" => Language::Python,
          "rust" => Language::Rust,
          "go" => Language::Go,
          "ruby" => Language::Ruby,
          _ => return Err(ApiError::BadRequest("Unknown language".into())),
      };
      
      // Get image and create config
      let image = state.image_registry.get_language_image(language).await?;
      let limits = match req.resource_preset.as_deref() {
          Some("light") => ResourceLimits::light(),
          Some("heavy") => ResourceLimits::heavy(),
          _ => ResourceLimits::standard(),
      };
      
      let config = ContainerConfig {
          image,
          resource_limits: limits,
          ..Default::default()
      };
      
      let handle = state.sandbox_lifecycle.create(config).await?;
      state.sandbox_lifecycle.start(&handle).await?;
      
      Ok(Json(CreateSandboxResponse {
          id: handle.id,
          name: handle.name,
          status: "running".to_string(),
      }))
  }
  
  // ... other handlers
verification:
  command: "cargo check -p cuttlefish-api"
  success: "Finished"
```

##### T12.2: Add request/response types
```yaml
file: crates/cuttlefish-api/src/routes/sandbox.rs
action: modify (append types)
content_requirements:
  - ExecuteRequest/Response
  - SnapshotRequest/Response
  - StatusResponse
  - HealthResponse
types: |
  #[derive(Debug, Deserialize)]
  pub struct ExecuteRequest {
      pub command: Vec<String>,
      pub timeout_secs: Option<u64>,
  }
  
  #[derive(Debug, Serialize)]
  pub struct ExecuteResponse {
      pub exit_code: i64,
      pub stdout: String,
      pub stderr: String,
      pub duration_ms: u64,
  }
  
  #[derive(Debug, Deserialize)]
  pub struct SnapshotRequest {
      pub name: Option<String>,
  }
  
  #[derive(Debug, Serialize)]
  pub struct SnapshotResponse {
      pub id: String,
      pub name: String,
  }
  
  #[derive(Debug, Serialize)]
  pub struct StatusResponse {
      pub id: String,
      pub status: String,
      pub created_at: String,
  }
  
  #[derive(Debug, Serialize)]
  pub struct HealthResponse {
      pub healthy: bool,
      pub docker_status: String,
      pub images: HashMap<String, bool>,
  }
verification:
  command: "cargo check -p cuttlefish-api"
  success: "Finished"
```

##### T12.3: Add API error handling
```yaml
file: crates/cuttlefish-api/src/error.rs
action: modify
content_requirements:
  - Add SandboxError conversion to ApiError
  - Proper HTTP status codes
  - Error response body
error_handling: |
  impl From<SandboxError> for ApiError {
      fn from(err: SandboxError) -> Self {
          match err {
              SandboxError::ContainerNotFound { .. } => ApiError::NotFound(err.to_string()),
              SandboxError::Timeout { .. } => ApiError::Timeout(err.to_string()),
              SandboxError::ResourceLimitExceeded { .. } => ApiError::BadRequest(err.to_string()),
              _ => ApiError::Internal(err.to_string()),
          }
      }
  }
verification:
  command: "cargo check -p cuttlefish-api"
  success: "Finished"
```

##### T12.4: Write API route tests
```yaml
file: crates/cuttlefish-api/tests/sandbox_routes.rs
action: create
content_requirements:
  - Test create sandbox endpoint
  - Test execute endpoint
  - Test health endpoint
  - Use mock sandbox implementations
tests: |
  use axum_test::TestServer;
  use cuttlefish_api::routes::sandbox_routes;
  
  #[tokio::test]
  async fn test_create_sandbox() {
      let app = create_test_app();
      let server = TestServer::new(app).unwrap();
      
      let response = server
          .post("/sandbox/create")
          .json(&serde_json::json!({
              "language": "python",
              "resource_preset": "light"
          }))
          .await;
      
      response.assert_status_ok();
      let body: CreateSandboxResponse = response.json();
      assert!(!body.id.is_empty());
  }
  
  #[tokio::test]
  async fn test_health_endpoint() {
      let app = create_test_app();
      let server = TestServer::new(app).unwrap();
      
      let response = server.get("/sandbox/health").await;
      response.assert_status_ok();
      
      let body: HealthResponse = response.json();
      assert!(body.healthy);
  }
verification:
  command: "cargo test -p cuttlefish-api sandbox"
  success: "test result: ok"
```

##### T12.5: Register sandbox routes in main router
```yaml
file: crates/cuttlefish-api/src/lib.rs
action: modify
content_requirements:
  - Import sandbox routes module
  - Merge sandbox routes into main router
verification:
  command: "cargo check -p cuttlefish-api"
  success: "Finished"
```

#### Atomic Commit
```
feat(api): add REST API routes for sandbox management

- POST /sandbox/create - create and start sandbox
- POST /sandbox/:id/execute - execute command
- POST /sandbox/:id/snapshot - create snapshot
- GET /sandbox/:id/status - get container status
- DELETE /sandbox/:id - stop and remove
- GET /sandbox/health - system health check

Includes request/response types and error handling.
```

---

### T13: End-to-End Workflow Tests

**Category**: `deep`
**Estimated Hours**: 2
**Dependencies**: T12
**Skills**: Rust, integration testing

#### Work Units

##### T13.1: Create E2E test suite
```yaml
file: crates/cuttlefish-sandbox/tests/e2e.rs
action: create
content_requirements:
  - Full workflow: create → execute → snapshot → restore → cleanup
  - Test each language runtime
  - Test resource limit enforcement
  - Test error recovery
tests: |
  #[cfg(feature = "integration")]
  mod e2e_tests {
      use cuttlefish_sandbox::*;
      use cuttlefish_core::traits::*;
      
      async fn setup() -> (DockerImageRegistry, DockerSandboxLifecycle, DockerSnapshotManager, DockerCleanupManager) {
          (
              DockerImageRegistry::new().unwrap(),
              DockerSandboxLifecycle::new().unwrap(),
              DockerSnapshotManager::new().unwrap(),
              DockerCleanupManager::new().unwrap(),
          )
      }
      
      #[tokio::test]
      async fn test_full_python_workflow() {
          let (images, lifecycle, snapshots, _) = setup().await;
          
          // 1. Get Python image
          let image = images.get_language_image(Language::Python).await.unwrap();
          
          // 2. Create and start container
          let config = ContainerConfig {
              image,
              resource_limits: ResourceLimits::light(),
              ..Default::default()
          };
          let handle = lifecycle.create(config.clone()).await.unwrap();
          lifecycle.start(&handle).await.unwrap();
          
          // 3. Execute Python code
          let result = lifecycle.execute(
              &handle,
              &["python3", "-c", "print('Hello from sandbox!')"],
              Duration::from_secs(30),
          ).await.unwrap();
          assert_eq!(result.exit_code, 0);
          assert!(result.stdout.contains("Hello from sandbox!"));
          
          // 4. Create snapshot
          let snapshot = snapshots.create_snapshot(&handle, SnapshotOptions::default()).await.unwrap();
          
          // 5. Stop and remove original
          lifecycle.stop(&handle, 10).await.unwrap();
          lifecycle.remove(&handle).await.unwrap();
          
          // 6. Restore from snapshot
          let restored = snapshots.restore_snapshot(&snapshot, config).await.unwrap();
          lifecycle.start(&restored).await.unwrap();
          
          // 7. Verify restored container works
          let result2 = lifecycle.execute(
              &restored,
              &["python3", "--version"],
              Duration::from_secs(10),
          ).await.unwrap();
          assert_eq!(result2.exit_code, 0);
          
          // 8. Cleanup
          lifecycle.stop(&restored, 10).await.unwrap();
          lifecycle.remove(&restored).await.unwrap();
          snapshots.delete_snapshot(&snapshot).await.unwrap();
      }
      
      #[tokio::test]
      async fn test_all_languages_execute() {
          let (images, lifecycle, _, _) = setup().await;
          
          let test_cases = [
              (Language::Node, vec!["node", "-e", "console.log('node ok')"]),
              (Language::Python, vec!["python3", "-c", "print('python ok')"]),
              (Language::Rust, vec!["rustc", "--version"]),
              (Language::Go, vec!["go", "version"]),
              (Language::Ruby, vec!["ruby", "-e", "puts 'ruby ok'"]),
          ];
          
          for (lang, command) in test_cases {
              let image = images.get_language_image(lang).await.unwrap();
              let config = ContainerConfig { image, ..Default::default() };
              let handle = lifecycle.create(config).await.unwrap();
              lifecycle.start(&handle).await.unwrap();
              
              let result = lifecycle.execute(
                  &handle,
                  &command.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
                  Duration::from_secs(30),
              ).await.unwrap();
              
              assert_eq!(result.exit_code, 0, "Failed for {:?}: {}", lang, result.stderr);
              
              lifecycle.stop(&handle, 10).await.unwrap();
              lifecycle.remove(&handle).await.unwrap();
          }
      }
      
      #[tokio::test]
      async fn test_timeout_enforcement() {
          let (images, lifecycle, _, _) = setup().await;
          
          let image = images.get_language_image(Language::Python).await.unwrap();
          let config = ContainerConfig { image, ..Default::default() };
          let handle = lifecycle.create(config).await.unwrap();
          lifecycle.start(&handle).await.unwrap();
          
          // Should timeout
          let result = lifecycle.execute(
              &handle,
              &["sleep", "60"],
              Duration::from_secs(2),
          ).await;
          
          assert!(matches!(result, Err(SandboxError::Timeout { .. })));
          
          lifecycle.stop(&handle, 10).await.unwrap();
          lifecycle.remove(&handle).await.unwrap();
      }
  }
verification:
  command: "cargo test -p cuttlefish-sandbox --features integration e2e"
  success: "test result: ok"
```

#### Atomic Commit
```
test(sandbox): add end-to-end workflow integration tests

- Full workflow: create → execute → snapshot → restore → cleanup
- Test all 5 language runtimes
- Test timeout enforcement
- Gated behind integration feature flag
```

---

### T14: Performance Tests

**Category**: `deep`
**Estimated Hours**: 2
**Dependencies**: T12
**Skills**: Rust, benchmarking, criterion

#### Work Units

##### T14.1: Create performance benchmarks
```yaml
file: crates/cuttlefish-sandbox/benches/sandbox.rs
action: create
content_requirements:
  - Benchmark container creation time
  - Benchmark execution latency
  - Benchmark snapshot/restore time
  - Use criterion crate
benchmarks: |
  use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
  use cuttlefish_sandbox::*;
  use cuttlefish_core::traits::*;
  use tokio::runtime::Runtime;
  
  fn bench_container_creation(c: &mut Criterion) {
      let rt = Runtime::new().unwrap();
      let lifecycle = rt.block_on(async { DockerSandboxLifecycle::new().unwrap() });
      let images = rt.block_on(async { DockerImageRegistry::new().unwrap() });
      
      c.bench_function("container_create", |b| {
          b.iter(|| {
              rt.block_on(async {
                  let image = images.get_language_image(Language::Python).await.unwrap();
                  let config = ContainerConfig { image, ..Default::default() };
                  let handle = lifecycle.create(config).await.unwrap();
                  lifecycle.remove(&handle).await.unwrap();
              })
          })
      });
  }
  
  fn bench_execution_latency(c: &mut Criterion) {
      let rt = Runtime::new().unwrap();
      let lifecycle = rt.block_on(async { DockerSandboxLifecycle::new().unwrap() });
      let images = rt.block_on(async { DockerImageRegistry::new().unwrap() });
      
      // Create container once
      let handle = rt.block_on(async {
          let image = images.get_language_image(Language::Python).await.unwrap();
          let config = ContainerConfig { image, ..Default::default() };
          let handle = lifecycle.create(config).await.unwrap();
          lifecycle.start(&handle).await.unwrap();
          handle
      });
      
      c.bench_function("execute_simple_command", |b| {
          b.iter(|| {
              rt.block_on(async {
                  lifecycle.execute(
                      &handle,
                      &["echo", "hello"],
                      Duration::from_secs(10),
                  ).await.unwrap()
              })
          })
      });
      
      rt.block_on(async {
          lifecycle.stop(&handle, 10).await.unwrap();
          lifecycle.remove(&handle).await.unwrap();
      });
  }
  
  criterion_group!(benches, bench_container_creation, bench_execution_latency);
  criterion_main!(benches);
verification:
  command: "cargo bench -p cuttlefish-sandbox --features integration"
  success: "criterion"
```

##### T14.2: Add Cargo.toml bench configuration
```yaml
file: crates/cuttlefish-sandbox/Cargo.toml
action: modify
content_requirements:
  - Add criterion dev-dependency
  - Configure bench target
config: |
  [dev-dependencies]
  criterion = { version = "0.5", features = ["async_tokio"] }
  
  [[bench]]
  name = "sandbox"
  harness = false
verification:
  command: "cargo check -p cuttlefish-sandbox"
  success: "Finished"
```

##### T14.3: Document performance baselines
```yaml
file: crates/cuttlefish-sandbox/PERFORMANCE.md
action: create
content_requirements:
  - Document expected performance baselines
  - Include benchmark running instructions
  - Note factors affecting performance
content: |
  # Sandbox Performance Baselines
  
  ## Expected Performance
  
  | Operation | Target | Notes |
  |-----------|--------|-------|
  | Container creation | < 500ms | Without image pull |
  | Simple execution | < 100ms | echo command |
  | Python execution | < 500ms | Simple print |
  | Snapshot creation | < 2s | Depends on container state |
  | Snapshot restore | < 1s | From local image |
  
  ## Running Benchmarks
  
  ```bash
  # Run all benchmarks
  cargo bench -p cuttlefish-sandbox --features integration
  
  # Run specific benchmark
  cargo bench -p cuttlefish-sandbox --features integration -- container_create
  ```
  
  ## Factors Affecting Performance
  
  - Docker daemon configuration
  - Host system resources
  - Image caching
  - Volume mount type (bind vs volume)
verification:
  command: "test -f crates/cuttlefish-sandbox/PERFORMANCE.md"
  success: "exists"
```

#### Atomic Commit
```
perf(sandbox): add performance benchmarks with criterion

- Benchmark container creation time
- Benchmark command execution latency
- Document expected performance baselines
- Include benchmark running instructions
```

---

### FS1: Full System Verification

**Category**: `quick`
**Estimated Hours**: 1
**Dependencies**: T13, T14

#### Work Units

##### FS1.1: Run all verification checks
```yaml
verification_script: |
  #!/bin/bash
  set -e
  
  echo "=== Cuttlefish Sandbox Verification ==="
  
  echo "1. Checking code compiles..."
  cargo check -p cuttlefish-core
  cargo check -p cuttlefish-sandbox
  cargo check -p cuttlefish-api
  
  echo "2. Running clippy..."
  cargo clippy -p cuttlefish-sandbox -- -D warnings
  
  echo "3. Running unit tests..."
  cargo test -p cuttlefish-sandbox
  
  echo "4. Running integration tests..."
  cargo test -p cuttlefish-sandbox --features integration
  
  echo "5. Building Docker images..."
  for lang in node python rust go ruby; do
      docker build -t cuttlefish/$lang:latest docker/images/$lang/
  done
  
  echo "6. Verifying images exist..."
  for lang in node python rust go ruby; do
      docker image inspect cuttlefish/$lang:latest > /dev/null
  done
  
  echo "7. Running E2E tests..."
  cargo test -p cuttlefish-sandbox --features integration e2e
  
  echo "=== All verifications passed! ==="
```

---

## Commit Strategy

### Commit Sequence

1. **Commit 1: Docker Images + Tests**
   - T1 (all Dockerfiles) + T2 (image tests)
   - Message: `feat(sandbox): add Docker base images for 5 languages`

2. **Commit 2: Image Registry**
   - T3 (trait) + T4 (implementation)
   - Message: `feat(sandbox): implement ImageRegistry trait with Docker backend`

3. **Commit 3: Sandbox Lifecycle**
   - T5 (trait) + T6 (implementation)
   - Message: `feat(sandbox): implement SandboxLifecycle for container management`

4. **Commit 4: Volume and Resources**
   - T7 (volumes) + T8 (resource limits)
   - Message: `feat(sandbox): add volume mounts and resource limit enforcement`

5. **Commit 5: Snapshots and Cleanup**
   - T9 (snapshots) + T10 (cleanup) + T11 (health)
   - Message: `feat(sandbox): add snapshot/restore, cleanup, and health checks`

6. **Commit 6: API and Tests**
   - T12 (API routes) + T13 (E2E tests) + T14 (perf tests)
   - Message: `feat(api): add sandbox REST API with E2E and performance tests`

---

## Agent Dispatch Instructions

### Category Assignments

| Task | Category | Skills | Est. Hours |
|------|----------|--------|------------|
| T1 | quick | Docker, multi-stage builds | 2 |
| T2 | quick | Rust testing, Docker API | 1 |
| T3 | deep | Rust traits, async design | 2 |
| T4 | deep | Rust, bollard, Docker API | 3 |
| T5 | deep | Rust traits, async design | 2 |
| T6 | deep | Rust, bollard, containers | 3 |
| T7 | deep | Rust, filesystem, Docker | 3 |
| T8 | deep | Rust, cgroups, resources | 3 |
| T9 | ultrabrain | Rust, Docker commits | 4 |
| T10 | deep | Rust, scheduling, cleanup | 2 |
| T11 | quick | Rust, health monitoring | 2 |
| T12 | deep | Rust, Axum, REST API | 3 |
| T13 | deep | Rust, integration testing | 2 |
| T14 | deep | Rust, criterion, benchmarks | 2 |

### Dispatch Order

```yaml
# Phase 1: Foundation (can start immediately)
- dispatch:
    task: T1
    category: quick
    blocking: false
    
# Phase 2: After T1 completes (parallel)
- dispatch:
    task: T2
    category: quick
    depends_on: [T1]
    blocking: false
    
- dispatch:
    task: T3
    category: deep
    depends_on: [T1]
    blocking: false

# Phase 3: After T3 completes
- dispatch:
    task: T4
    category: deep
    depends_on: [T3]
    blocking: false
    
- dispatch:
    task: T5
    category: deep
    depends_on: [T3]
    blocking: false

# Phase 4: After T4, T5 complete
- dispatch:
    task: T6
    category: deep
    depends_on: [T4, T5]
    blocking: false

# Phase 5: After T6 completes (parallel)
- dispatch:
    task: T7
    category: deep
    depends_on: [T6]
    blocking: false
    
- dispatch:
    task: T8
    category: deep
    depends_on: [T6]
    blocking: false

# Phase 6: After T7, T8 complete
- dispatch:
    task: T9
    category: ultrabrain
    depends_on: [T7, T8]
    blocking: false

# Phase 7: After T9 completes
- dispatch:
    task: T10
    category: deep
    depends_on: [T9]
    blocking: false

# Phase 8: After T10 completes
- dispatch:
    task: T11
    category: quick
    depends_on: [T10]
    blocking: false

# Phase 9: After T11 completes
- dispatch:
    task: T12
    category: deep
    depends_on: [T11]
    blocking: false

# Phase 10: After T12 completes (parallel)
- dispatch:
    task: T13
    category: deep
    depends_on: [T12]
    blocking: false
    
- dispatch:
    task: T14
    category: deep
    depends_on: [T12]
    blocking: false

# Final: Verification
- dispatch:
    task: FS1
    category: quick
    depends_on: [T13, T14]
    blocking: true
```

---

## TDD Verification Matrix

| Task | Test First | Test File | Verification Command |
|------|------------|-----------|---------------------|
| T1 | N/A | N/A | `docker build` |
| T2 | Yes | `tests/image_build.rs` | `cargo test --features integration` |
| T3 | Yes | inline `#[cfg(test)]` | `cargo test -p cuttlefish-core` |
| T4 | Yes | `src/images.rs` tests | `cargo test -p cuttlefish-sandbox` |
| T5 | Yes | inline `#[cfg(test)]` | `cargo test -p cuttlefish-core` |
| T6 | Yes | `src/lifecycle.rs` tests | `cargo test -p cuttlefish-sandbox` |
| T7 | Yes | `tests/volumes.rs` | `cargo test --features integration` |
| T8 | Yes | `tests/resources.rs` | `cargo test --features integration` |
| T9 | Yes | `tests/snapshots.rs` | `cargo test --features integration` |
| T10 | Yes | `src/cleanup.rs` tests | `cargo test -p cuttlefish-sandbox` |
| T11 | Yes | `src/health.rs` tests | `cargo test -p cuttlefish-sandbox` |
| T12 | Yes | `tests/sandbox_routes.rs` | `cargo test -p cuttlefish-api` |
| T13 | N/A | `tests/e2e.rs` | `cargo test --features integration e2e` |
| T14 | N/A | `benches/sandbox.rs` | `cargo bench` |
```

---

# v1-production-polish-wbs.md

```markdown
# V1 Production Polish Work Breakdown Structure

## Executive Summary

This document provides execution-ready task breakdowns for polishing the Cuttlefish production deployment. These tasks focus on documentation, installation experience, and configuration clarity.

**Total Effort**: ~8 hours
**Parallelization**: All tasks can run concurrently
**Task Count**: 4 main tasks → 15 atomic work units

---

## Dependency Graph

```
T15 (README Updates) ──────────────────┐
                                       │
T16 (install.sh Improvements) ─────────┼──► FP1 (Documentation Completeness)
                                       │         │
T17 (Config Documentation) ────────────┤         ▼
                                       │    FP2 (Documentation Quality)
T18 (Provider Documentation) ──────────┘
```

All tasks are independent and can execute in parallel.

---

## Parallel Execution Schedule

### Wave P1 (Hours 0-8) - All Parallel

| Track A | Track B | Track C | Track D |
|---------|---------|---------|---------|
| T15: README (3h) | T16: install.sh (2h) | T17: Config (2h) | T18: Providers (4h) |

### Wave P2 (Hours 8-9)

| Verification |
|--------------|
| FP1 + FP2: Documentation verification (1h) |

---

## Task Breakdowns

### T15: README Updates

**Category**: `writing`
**Estimated Hours**: 3
**Dependencies**: None
**Skills**: Technical writing, Markdown

#### Work Units

##### T15.1: Add Quick Start section
```yaml
file: README.md
action: modify
section: Quick Start
content_requirements:
  - 3-command setup (install, configure, run)
  - Copy-pasteable commands
  - Expected output examples
  - Link to detailed docs
template: |
  ## Quick Start
  
  ```bash
  # Install Cuttlefish
  curl -fsSL https://cuttlefish.dev/install.sh | sh
  
  # Configure your API key
  export CUTTLEFISH_API_KEY="your-api-key"
  
  # Start the server
  cuttlefish serve
  ```
  
  Visit http://localhost:8080 to access the web interface.
verification:
  command: "grep -q 'Quick Start' README.md"
  success: "found"
```

##### T15.2: Document all agent types
```yaml
file: README.md
action: modify
section: Agents
content_requirements:
  - List all 7 agent types
  - Brief description of each
  - Example use case for each
  - Configuration snippet
template: |
  ## Agents
  
  Cuttlefish includes 7 specialized agents:
  
  | Agent | Description | Use Case |
  |-------|-------------|----------|
  | CodeGen | Generates code from specifications | Building new features |
  | CodeReview | Reviews code for issues | PR reviews |
  | Debug | Diagnoses and fixes bugs | Production issues |
  | Refactor | Improves code structure | Technical debt |
  | Explain | Explains code functionality | Onboarding |
  | Test | Generates test cases | Test coverage |
  | Docs | Generates documentation | API docs |
  
  ### Example: Using the CodeGen Agent
  
  ```bash
  cuttlefish agent codegen --prompt "Create a REST API for user management"
  ```
verification:
  command: "grep -c 'Agent' README.md | grep -q '[7-9]'"
  success: "found"
```

##### T15.3: Add troubleshooting section
```yaml
file: README.md
action: modify
section: Troubleshooting
content_requirements:
  - Common error messages
  - Solutions for each
  - Debug logging instructions
  - Link to issues page
template: |
  ## Troubleshooting
  
  ### Common Issues
  
  **Error: "API key not found"**
  ```
  Solution: Set the CUTTLEFISH_API_KEY environment variable
  export CUTTLEFISH_API_KEY="your-key"
  ```
  
  **Error: "Docker daemon not running"**
  ```
  Solution: Start Docker Desktop or the Docker daemon
  sudo systemctl start docker
  ```
  
  **Error: "Connection refused"**
  ```
  Solution: Check if the server is running on the expected port
  cuttlefish serve --port 8080
  ```
  
  ### Debug Logging
  
  Enable debug logs for troubleshooting:
  ```bash
  RUST_LOG=debug cuttlefish serve
  ```
  
  ### Getting Help
  
  - [GitHub Issues](https://github.com/org/cuttlefish/issues)
  - [Discord Community](https://discord.gg/cuttlefish)
verification:
  command: "grep -q 'Troubleshooting' README.md"
  success: "found"
```

##### T15.4: Add badges and metadata
```yaml
file: README.md
action: modify
section: Header
content_requirements:
  - CI status badge
  - Version badge
  - License badge
  - Crates.io badge (if published)
template: |
  # Cuttlefish
  
  [![CI](https://github.com/org/cuttlefish/actions/workflows/ci.yml/badge.svg)](https://github.com/org/cuttlefish/actions)
  [![Version](https://img.shields.io/badge/version-0.1.0-blue)](https://github.com/org/cuttlefish/releases)
  [![License](https://img.shields.io/badge/license-MIT-green)](LICENSE)
  
  AI-powered software development assistant with multi-agent architecture.
verification:
  command: "grep -q 'badge' README.md"
  success: "found"
```

#### Atomic Commit
```
docs: update README with quick start, agents, and troubleshooting

- Add 3-command Quick Start section
- Document all 7 agent types with examples
- Add troubleshooting section for common issues
- Add CI, version, and license badges
```

---

### T16: install.sh Improvements

**Category**: `quick`
**Estimated Hours**: 2
**Dependencies**: None
**Skills**: Shell scripting, POSIX compatibility

#### Work Units

##### T16.1: Add OS detection
```yaml
file: install.sh
action: modify
content_requirements:
  - Detect Linux, macOS, Windows (WSL)
  - Set appropriate download URLs
  - Handle architecture (x86_64, arm64)
implementation: |
  #!/bin/sh
  set -e
  
  # Detect OS
  detect_os() {
      case "$(uname -s)" in
          Linux*)     OS=linux;;
          Darwin*)    OS=darwin;;
          CYGWIN*|MINGW*|MSYS*) OS=windows;;
          *)          OS=unknown;;
      esac
      echo "$OS"
  }
  
  # Detect architecture
  detect_arch() {
      case "$(uname -m)" in
          x86_64|amd64)  ARCH=x86_64;;
          aarch64|arm64) ARCH=aarch64;;
          *)             ARCH=unknown;;
      esac
      echo "$ARCH"
  }
  
  OS=$(detect_os)
  ARCH=$(detect_arch)
  
  if [ "$OS" = "unknown" ] || [ "$ARCH" = "unknown" ]; then
      echo "Error: Unsupported OS ($OS) or architecture ($ARCH)"
      exit 1
  fi
  
  echo "Detected: $OS-$ARCH"
verification:
  command: "sh -n install.sh && grep -q 'detect_os' install.sh"
  success: "valid syntax and function exists"
```

##### T16.2: Add dependency checking
```yaml
file: install.sh
action: modify
content_requirements:
  - Check for curl or wget
  - Check for Docker
  - Check for Rust (optional, for building)
  - Provide installation hints for missing deps
implementation: |
  check_dependencies() {
      MISSING=""
      
      # Check for curl or wget
      if ! command -v curl >/dev/null 2>&1 && ! command -v wget >/dev/null 2>&1; then
          MISSING="$MISSING curl/wget"
      fi
      
      # Check for Docker (optional but recommended)
      if ! command -v docker >/dev/null 2>&1; then
          echo "Warning: Docker not found. Sandbox features will be unavailable."
          echo "Install Docker: https://docs.docker.com/get-docker/"
      fi
      
      if [ -n "$MISSING" ]; then
          echo "Error: Missing required dependencies:$MISSING"
          echo ""
          echo "Please install the missing dependencies and try again."
          exit 1
      fi
  }
  
  check_dependencies
verification:
  command: "grep -q 'check_dependencies' install.sh"
  success: "found"
```

##### T16.3: Add error handling
```yaml
file: install.sh
action: modify
content_requirements:
  - Trap errors and cleanup
  - Provide actionable error messages
  - Verify download integrity (checksum)
implementation: |
  # Error handling
  cleanup() {
      rm -f "$TEMP_FILE" 2>/dev/null || true
  }
  
  trap cleanup EXIT
  
  error() {
      echo "Error: $1" >&2
      exit 1
  }
  
  # Download with verification
  download() {
      URL="$1"
      DEST="$2"
      
      echo "Downloading from $URL..."
      
      if command -v curl >/dev/null 2>&1; then
          curl -fsSL "$URL" -o "$DEST" || error "Download failed. Check your network connection."
      elif command -v wget >/dev/null 2>&1; then
          wget -q "$URL" -O "$DEST" || error "Download failed. Check your network connection."
      fi
      
      if [ ! -f "$DEST" ]; then
          error "Download verification failed. File not found."
      fi
  }
verification:
  command: "grep -q 'trap cleanup' install.sh"
  success: "found"
```

##### T16.4: Add --help flag
```yaml
file: install.sh
action: modify
content_requirements:
  - --help shows usage information
  - --version selects specific version
  - --prefix sets installation directory
implementation: |
  # Parse arguments
  VERSION="latest"
  PREFIX="/usr/local"
  
  while [ $# -gt 0 ]; do
      case "$1" in
          --help|-h)
              echo "Cuttlefish Installer"
              echo ""
              echo "Usage: install.sh [OPTIONS]"
              echo ""
              echo "Options:"
              echo "  --help, -h      Show this help message"
              echo "  --version VER   Install specific version (default: latest)"
              echo "  --prefix DIR    Installation directory (default: /usr/local)"
              echo ""
              echo "Examples:"
              echo "  curl -fsSL https://cuttlefish.dev/install.sh | sh"
              echo "  curl -fsSL https://cuttlefish.dev/install.sh | sh -s -- --version 0.1.0"
              exit 0
              ;;
          --version)
              VERSION="$2"
              shift 2
              ;;
          --prefix)
              PREFIX="$2"
              shift 2
              ;;
          *)
              error "Unknown option: $1. Use --help for usage."
              ;;
      esac
  done
verification:
  command: "sh install.sh --help 2>&1 | grep -q 'Usage'"
  success: "found"
```

#### Atomic Commit
```
chore: improve install.sh with OS detection and error handling

- Add OS and architecture detection (Linux, macOS, WSL)
- Add dependency checking (curl/wget, Docker)
- Add error handling with cleanup trap
- Add --help, --version, --prefix flags
- Verify download integrity
```

---

### T17: Configuration Documentation

**Category**: `quick`
**Estimated Hours**: 2
**Dependencies**: None
**Skills**: TOML, documentation

#### Work Units

##### T17.1: Add comprehensive inline comments
```yaml
file: cuttlefish.example.toml
action: modify
content_requirements:
  - Comment every configuration option
  - Explain default values
  - Show valid value ranges
  - Group related settings
template: |
  # Cuttlefish Configuration
  # Copy this file to cuttlefish.toml and customize
  
  # =============================================================================
  # Server Configuration
  # =============================================================================
  [server]
  # Host address to bind to
  # Default: "127.0.0.1" (localhost only)
  # Use "0.0.0.0" to allow external connections
  host = "127.0.0.1"
  
  # Port number for the HTTP server
  # Default: 8080
  # Range: 1-65535
  port = 8080
  
  # =============================================================================
  # Agent Configuration
  # =============================================================================
  [agents]
  # Default model provider for all agents
  # Options: "anthropic", "openai", "google", "ollama"
  default_provider = "anthropic"
  
  # Maximum concurrent agent executions
  # Default: 4
  # Range: 1-32
  max_concurrent = 4
verification:
  command: "grep -c '#' cuttlefish.example.toml | awk '{print $1 > 20}'"
  success: "has many comments"
```

##### T17.2: Add provider configuration examples
```yaml
file: cuttlefish.example.toml
action: modify
section: Providers
content_requirements:
  - Example for each provider
  - Environment variable references for secrets
  - Default model settings
template: |
  # =============================================================================
  # Provider Configuration
  # =============================================================================
  
  # Anthropic (Claude)
  [providers.anthropic]
  # API key from https://console.anthropic.com/
  # Use environment variable for security
  api_key = "${ANTHROPIC_API_KEY}"
  # Default model (claude-3-opus-20240229, claude-3-sonnet-20240229)
  model = "claude-3-sonnet-20240229"
  
  # OpenAI (GPT)
  [providers.openai]
  api_key = "${OPENAI_API_KEY}"
  model = "gpt-4-turbo-preview"
  # Organization ID (optional)
  # organization = "org-xxx"
  
  # Google (Gemini)
  [providers.google]
  api_key = "${GOOGLE_API_KEY}"
  model = "gemini-pro"
  
  # Ollama (Local)
  [providers.ollama]
  # No API key needed for local deployment
  base_url = "http://localhost:11434"
  model = "llama2"
verification:
  command: "grep -q 'providers.anthropic' cuttlefish.example.toml"
  success: "found"
```

##### T17.3: Add sandbox configuration
```yaml
file: cuttlefish.example.toml
action: modify
section: Sandbox
content_requirements:
  - Resource limit defaults
  - Cleanup policy settings
  - Docker configuration
template: |
  # =============================================================================
  # Sandbox Configuration
  # =============================================================================
  [sandbox]
  # Enable sandbox features (requires Docker)
  enabled = true
  
  # Docker socket path
  # Default: /var/run/docker.sock (Linux/Mac)
  # docker_socket = "/var/run/docker.sock"
  
  # Resource Limits
  [sandbox.limits]
  # Default memory limit per container
  # Format: "512MB", "1GB", etc.
  memory = "512MB"
  
  # CPU limit (number of cores)
  cpu = 1.0
  
  # Execution timeout in seconds
  timeout = 60
  
  # Cleanup Policy
  [sandbox.cleanup]
  # Remove containers older than this duration
  container_max_age = "1h"
  
  # Remove snapshots older than this duration
  snapshot_max_age = "24h"
  
  # Automatically remove orphaned volumes
  remove_orphan_volumes = true
verification:
  command: "grep -q 'sandbox.limits' cuttlefish.example.toml"
  success: "found"
```

#### Atomic Commit
```
docs: enhance cuttlefish.example.toml with comprehensive documentation

- Add inline comments for all configuration options
- Document default values and valid ranges
- Add provider configuration examples
- Add sandbox configuration section
- Use environment variables for secrets
```

---

### T18: Provider Documentation

**Category**: `writing`
**Estimated Hours**: 4
**Dependencies**: None
**Skills**: Technical writing, API documentation

#### Work Units

##### T18.1: Create docs/providers directory structure
```yaml
action: create directories
paths:
  - docs/providers/
verification:
  command: "test -d docs/providers"
  success: "exists"
```

##### T18.2: Create Anthropic provider doc
```yaml
file: docs/providers/anthropic.md
action: create
content: |
  # Anthropic Provider
  
  ## Overview
  
  The Anthropic provider enables integration with Claude, Anthropic's AI assistant.
  
  ## Configuration
  
  ```toml
  [providers.anthropic]
  api_key = "${ANTHROPIC_API_KEY}"
  model = "claude-3-sonnet-20240229"
  ```
  
  ## Supported Models
  
  | Model | Context | Best For |
  |-------|---------|----------|
  | claude-3-opus-20240229 | 200K | Complex reasoning, analysis |
  | claude-3-sonnet-20240229 | 200K | Balanced performance |
  | claude-3-haiku-20240307 | 200K | Fast, simple tasks |
  
  ## Environment Variables
  
  - `ANTHROPIC_API_KEY` (required): Your API key from [Anthropic Console](https://console.anthropic.com/)
  
  ## Usage Example
  
  ```rust
  use cuttlefish_providers::anthropic::AnthropicProvider;
  
  let provider = AnthropicProvider::from_env()?;
  let response = provider.complete("Write a haiku about Rust").await?;
  ```
  
  ## Rate Limits
  
  - Tier 1: 60 requests/minute
  - Tier 2: 1000 requests/minute
  - See [Anthropic Rate Limits](https://docs.anthropic.com/claude/reference/rate-limits)
  
  ## Notes
  
  - Supports streaming responses
  - Includes built-in retry with exponential backoff
  - Vision capabilities available with Claude 3 models
verification:
  command: "test -f docs/providers/anthropic.md"
  success: "exists"
```

##### T18.3: Create OpenAI provider doc
```yaml
file: docs/providers/openai.md
action: create
content: |
  # OpenAI Provider
  
  ## Overview
  
  The OpenAI provider enables integration with GPT models.
  
  ## Configuration
  
  ```toml
  [providers.openai]
  api_key = "${OPENAI_API_KEY}"
  model = "gpt-4-turbo-preview"
  # organization = "org-xxx"  # Optional
  ```
  
  ## Supported Models
  
  | Model | Context | Best For |
  |-------|---------|----------|
  | gpt-4-turbo-preview | 128K | Complex tasks, latest features |
  | gpt-4 | 8K | High quality, stable |
  | gpt-3.5-turbo | 16K | Fast, cost-effective |
  
  ## Environment Variables
  
  - `OPENAI_API_KEY` (required): Your API key from [OpenAI Platform](https://platform.openai.com/)
  - `OPENAI_ORG_ID` (optional): Organization ID for billing
  
  ## Usage Example
  
  ```rust
  use cuttlefish_providers::openai::OpenAIProvider;
  
  let provider = OpenAIProvider::from_env()?;
  let response = provider.complete("Explain async/await in Rust").await?;
  ```
  
  ## Rate Limits
  
  - Varies by tier and model
  - See [OpenAI Rate Limits](https://platform.openai.com/docs/guides/rate-limits)
  
  ## Notes
  
  - Supports function calling
  - JSON mode available
  - Vision capabilities with gpt-4-vision-preview
verification:
  command: "test -f docs/providers/openai.md"
  success: "exists"
```

##### T18.4: Create Google provider doc
```yaml
file: docs/providers/google.md
action: create
content: |
  # Google Provider
  
  ## Overview
  
  The Google provider enables integration with Gemini models.
  
  ## Configuration
  
  ```toml
  [providers.google]
  api_key = "${GOOGLE_API_KEY}"
  model = "gemini-pro"
  ```
  
  ## Supported Models
  
  | Model | Context | Best For |
  |-------|---------|----------|
  | gemini-pro | 32K | General purpose |
  | gemini-pro-vision | 32K | Multimodal tasks |
  | gemini-ultra | 32K | Most capable |
  
  ## Environment Variables
  
  - `GOOGLE_API_KEY` (required): Your API key from [Google AI Studio](https://makersuite.google.com/)
  
  ## Usage Example
  
  ```rust
  use cuttlefish_providers::google::GoogleProvider;
  
  let provider = GoogleProvider::from_env()?;
  let response = provider.complete("What is WebAssembly?").await?;
  ```
  
  ## Rate Limits
  
  - 60 requests/minute (free tier)
  - See [Gemini Quotas](https://ai.google.dev/pricing)
  
  ## Notes
  
  - Free tier available
  - Supports multimodal input
  - Safety filters enabled by default
verification:
  command: "test -f docs/providers/google.md"
  success: "exists"
```

##### T18.5: Create AWS Bedrock provider doc
```yaml
file: docs/providers/aws-bedrock.md
action: create
content: |
  # AWS Bedrock Provider
  
  ## Overview
  
  The AWS Bedrock provider enables integration with models hosted on Amazon Bedrock.
  
  ## Configuration
  
  ```toml
  [providers.aws-bedrock]
  region = "us-east-1"
  model = "anthropic.claude-3-sonnet-20240229-v1:0"
  # Credentials from environment or IAM role
  ```
  
  ## Supported Models
  
  | Model | Provider | Best For |
  |-------|----------|----------|
  | anthropic.claude-3-* | Anthropic | General purpose |
  | amazon.titan-text-* | Amazon | Cost-effective |
  | meta.llama2-* | Meta | Open source |
  
  ## Environment Variables
  
  - `AWS_ACCESS_KEY_ID` (required if not using IAM)
  - `AWS_SECRET_ACCESS_KEY` (required if not using IAM)
  - `AWS_REGION` (optional, defaults to us-east-1)
  
  ## Usage Example
  
  ```rust
  use cuttlefish_providers::bedrock::BedrockProvider;
  
  let provider = BedrockProvider::from_env()?;
  let response = provider.complete("Summarize this code").await?;
  ```
  
  ## Notes
  
  - Supports IAM role authentication
  - VPC endpoint available
  - Enterprise compliance features
verification:
  command: "test -f docs/providers/aws-bedrock.md"
  success: "exists"
```

##### T18.6: Create Azure provider doc
```yaml
file: docs/providers/azure.md
action: create
content: |
  # Azure OpenAI Provider
  
  ## Overview
  
  The Azure provider enables integration with OpenAI models deployed on Azure.
  
  ## Configuration
  
  ```toml
  [providers.azure]
  endpoint = "https://your-resource.openai.azure.com"
  api_key = "${AZURE_OPENAI_API_KEY}"
  deployment = "gpt-4"
  api_version = "2024-02-15-preview"
  ```
  
  ## Supported Models
  
  Depends on your Azure deployment. Common options:
  - GPT-4 and GPT-4 Turbo
  - GPT-3.5 Turbo
  - Embeddings models
  
  ## Environment Variables
  
  - `AZURE_OPENAI_API_KEY` (required): Your Azure OpenAI key
  - `AZURE_OPENAI_ENDPOINT` (required): Your resource endpoint
  
  ## Usage Example
  
  ```rust
  use cuttlefish_providers::azure::AzureProvider;
  
  let provider = AzureProvider::from_env()?;
  let response = provider.complete("Generate a README template").await?;
  ```
  
  ## Notes
  
  - Enterprise compliance (SOC 2, HIPAA)
  - Private endpoints available
  - Content filtering enabled by default
verification:
  command: "test -f docs/providers/azure.md"
  success: "exists"
```

##### T18.7: Create Ollama provider doc
```yaml
file: docs/providers/ollama.md
action: create
content: |
  # Ollama Provider
  
  ## Overview
  
  The Ollama provider enables integration with locally-hosted open source models.
  
  ## Configuration
  
  ```toml
  [providers.ollama]
  base_url = "http://localhost:11434"
  model = "llama2"
  ```
  
  ## Supported Models
  
  Any model available via `ollama pull`:
  - llama2 (7B, 13B, 70B)
  - codellama
  - mistral
  - mixtral
  - phi-2
  - And many more...
  
  ## Environment Variables
  
  - `OLLAMA_HOST` (optional): Override default URL
  
  ## Usage Example
  
  ```rust
  use cuttlefish_providers::ollama::OllamaProvider;
  
  let provider = OllamaProvider::new("http://localhost:11434", "codellama")?;
  let response = provider.complete("Write a binary search in Rust").await?;
  ```
  
  ## Installation
  
  ```bash
  # macOS
  brew install ollama
  
  # Linux
  curl -fsSL https://ollama.ai/install.sh | sh
  
  # Pull a model
  ollama pull llama2
  ```
  
  ## Notes
  
  - No API key required
  - Runs completely locally
  - GPU acceleration supported
  - Great for development and testing
verification:
  command: "test -f docs/providers/ollama.md"
  success: "exists"
```

##### T18.8: Create remaining provider docs
```yaml
files:
  - docs/providers/mistral.md
  - docs/providers/groq.md
  - docs/providers/together.md
  - docs/providers/openrouter.md
  - docs/providers/deepseek.md
action: create
template: |
  # {Provider} Provider
  
  ## Overview
  
  {Brief description}
  
  ## Configuration
  
  ```toml
  [providers.{provider}]
  api_key = "${API_KEY}"
  model = "{default_model}"
  ```
  
  ## Supported Models
  
  | Model | Context | Best For |
  |-------|---------|----------|
  | {model1} | {context} | {use_case} |
  
  ## Environment Variables
  
  - `{API_KEY_VAR}` (required): Your API key
  
  ## Notes
  
  {Provider-specific notes}
verification:
  command: "ls docs/providers/*.md | wc -l | grep -q '11'"
  success: "all 11 docs exist"
```

##### T18.9: Create provider index
```yaml
file: docs/providers/README.md
action: create
content: |
  # Provider Documentation
  
  Cuttlefish supports 11 LLM providers. Choose based on your needs:
  
  ## Cloud Providers
  
  | Provider | Best For | Docs |
  |----------|----------|------|
  | [Anthropic](./anthropic.md) | Claude models, complex reasoning | [Link](./anthropic.md) |
  | [OpenAI](./openai.md) | GPT models, function calling | [Link](./openai.md) |
  | [Google](./google.md) | Gemini models, free tier | [Link](./google.md) |
  | [AWS Bedrock](./aws-bedrock.md) | Enterprise, compliance | [Link](./aws-bedrock.md) |
  | [Azure](./azure.md) | Enterprise Azure integration | [Link](./azure.md) |
  
  ## Specialized Providers
  
  | Provider | Best For | Docs |
  |----------|----------|------|
  | [Mistral](./mistral.md) | European hosting, efficiency | [Link](./mistral.md) |
  | [Groq](./groq.md) | Ultra-fast inference | [Link](./groq.md) |
  | [Together](./together.md) | Open source models | [Link](./together.md) |
  | [OpenRouter](./openrouter.md) | Multi-provider routing | [Link](./openrouter.md) |
  | [DeepSeek](./deepseek.md) | Code-specialized models | [Link](./deepseek.md) |
  
  ## Local Providers
  
  | Provider | Best For | Docs |
  |----------|----------|------|
  | [Ollama](./ollama.md) | Local development, privacy | [Link](./ollama.md) |
  
  ## Choosing a Provider
  
  - **Starting out?** Use Anthropic or OpenAI
  - **On a budget?** Use Ollama (free) or Google (free tier)
  - **Enterprise?** Use AWS Bedrock or Azure
  - **Need speed?** Use Groq
  - **Want flexibility?** Use OpenRouter
verification:
  command: "test -f docs/providers/README.md"
  success: "exists"
```

#### Atomic Commit
```
docs: add comprehensive provider documentation

- Create docs for all 11 LLM providers
- Include configuration, models, and examples for each
- Add provider selection guide
- Document environment variables and rate limits
```

---

### FP1: Documentation Completeness Verification

**Category**: `quick`
**Estimated Hours**: 0.5
**Dependencies**: T15, T16, T17, T18

#### Verification Commands
```bash
#!/bin/bash
set -e

echo "Verifying documentation completeness..."

# README checks
echo "Checking README.md..."
grep -q "Quick Start" README.md || { echo "Missing: Quick Start section"; exit 1; }
grep -q "Troubleshooting" README.md || { echo "Missing: Troubleshooting section"; exit 1; }
grep -q "badge" README.md || { echo "Missing: Badges"; exit 1; }

# install.sh checks
echo "Checking install.sh..."
sh -n install.sh || { echo "install.sh has syntax errors"; exit 1; }
grep -q "detect_os" install.sh || { echo "Missing: OS detection"; exit 1; }
grep -q "check_dependencies" install.sh || { echo "Missing: Dependency checking"; exit 1; }
sh install.sh --help | grep -q "Usage" || { echo "Missing: --help flag"; exit 1; }

# Config checks
echo "Checking cuttlefish.example.toml..."
toml_comment_count=$(grep -c "^#" cuttlefish.example.toml)
if [ "$toml_comment_count" -lt 20 ]; then
    echo "Insufficient comments in config (found $toml_comment_count, need 20+)"
    exit 1
fi

# Provider docs checks
echo "Checking provider documentation..."
provider_count=$(ls docs/providers/*.md 2>/dev/null | wc -l)
if [ "$provider_count" -lt 11 ]; then
    echo "Missing provider docs (found $provider_count, need 11)"
    exit 1
fi

echo "All documentation checks passed!"
```

---

### FP2: Documentation Quality Verification

**Category**: `quick`
**Estimated Hours**: 0.5
**Dependencies**: FP1

#### Manual Verification Checklist
```yaml
checks:
  - name: "README renders on GitHub"
    type: manual
    steps:
      - Push to a branch
      - View README.md on GitHub
      - Verify all sections render correctly
      - Verify badges display
      
  - name: "install.sh works on clean system"
    type: manual
    steps:
      - Use a fresh Docker container or VM
      - Run: curl -fsSL ./install.sh | sh --help
      - Verify help output
      - Verify OS detection works
      
  - name: "Example config is valid TOML"
    type: automated
    command: "python3 -c \"import tomllib; tomllib.load(open('cuttlefish.example.toml', 'rb'))\""
    
  - name: "Provider docs consistent"
    type: manual
    steps:
      - Review all provider docs
      - Verify consistent structure
      - Verify all have configuration, models, examples
```

---

## Commit Strategy

### Commit Sequence

All commits can be made in parallel and merged in any order.

1. **Commit P1: README**
   - T15 (all README updates)
   - Message: `docs: update README with quick start, agents, and troubleshooting`

2. **Commit P2: install.sh**
   - T16 (all install.sh improvements)
   - Message: `chore: improve install.sh with OS detection and error handling`

3. **Commit P3: Config**
   - T17 (all config documentation)
   - Message: `docs: enhance cuttlefish.example.toml with comprehensive documentation`

4. **Commit P4: Provider Docs**
   - T18 (all provider documentation)
   - Message: `docs: add comprehensive provider documentation`

---

## Agent Dispatch Instructions

### Parallel Dispatch (All at Once)

```yaml
parallel_dispatch:
  - task: T15
    category: writing
    prompt: |
      Update README.md for Cuttlefish with production-ready documentation.
      
      Requirements:
      1. Add Quick Start section with 3-command setup:
         - Install command (curl | sh)
         - Configure API key
         - Start server
      
      2. Document all 7 agent types:
         - CodeGen, CodeReview, Debug, Refactor, Explain, Test, Docs
         - Include table with description and use case
         - Add example command for one agent
      
      3. Add Troubleshooting section:
         - "API key not found" error and solution
         - "Docker daemon not running" error and solution  
         - "Connection refused" error and solution
         - Debug logging instructions (RUST_LOG=debug)
         - Links to GitHub issues and Discord
      
      4. Add badges at top:
         - CI status badge

<task_metadata>
session_id: ses_2aa1b467fffeqcHg8mt24a6wbD
subagent: plan
</task_metadata>