//! Sandbox trait and supporting types.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use uuid::Uuid;

use crate::error::SandboxError;

/// Result type for sandbox operations.
pub type SandboxResult<T> = Result<T, SandboxError>;

/// Supported programming languages for sandbox images.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Language {
    /// Node.js / JavaScript / TypeScript
    Node,
    /// Python
    Python,
    /// Rust
    Rust,
    /// Go
    Go,
    /// Ruby
    Ruby,
    /// Generic / unspecified language
    Generic,
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Language::Node => "node",
            Language::Python => "python",
            Language::Rust => "rust",
            Language::Go => "go",
            Language::Ruby => "ruby",
            Language::Generic => "generic",
        };
        write!(f, "{s}")
    }
}

impl FromStr for Language {
    type Err = SandboxError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "node" | "nodejs" | "javascript" | "js" | "typescript" | "ts" => Ok(Language::Node),
            "python" | "py" => Ok(Language::Python),
            "rust" | "rs" => Ok(Language::Rust),
            "go" | "golang" => Ok(Language::Go),
            "ruby" | "rb" => Ok(Language::Ruby),
            "generic" | "" => Ok(Language::Generic),
            other => Err(SandboxError::InvalidLanguage(other.to_string())),
        }
    }
}

/// Specification for a Docker image.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageSpec {
    /// Image name (e.g., "cuttlefish/node").
    pub name: String,
    /// Image tag (e.g., "latest", "20-alpine").
    pub tag: String,
    /// Programming language this image supports.
    pub language: Language,
    /// Image size in bytes, if known.
    pub size_bytes: Option<u64>,
    /// When the image was created, if known.
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Options for building a Docker image.
#[derive(Debug, Clone, Default)]
pub struct ImageBuildOptions {
    /// Path to the Dockerfile (relative or absolute).
    pub dockerfile_path: Option<PathBuf>,
    /// Build arguments to pass to Docker.
    pub build_args: HashMap<String, String>,
    /// Whether to disable Docker's build cache.
    pub no_cache: bool,
}

/// Registry for managing Docker images used by sandboxes.
#[async_trait]
pub trait ImageRegistry: Send + Sync {
    /// List all available images in the registry.
    async fn list_images(&self) -> SandboxResult<Vec<ImageSpec>>;

    /// Pull an image from a remote registry.
    async fn pull_image(&self, name: &str, tag: &str) -> SandboxResult<ImageSpec>;

    /// Build an image from a Dockerfile.
    async fn build_image(
        &self,
        name: &str,
        tag: &str,
        options: ImageBuildOptions,
    ) -> SandboxResult<ImageSpec>;

    /// Remove an image from the local registry.
    async fn remove_image(&self, name: &str, tag: &str) -> SandboxResult<()>;

    /// Get the default image for a specific programming language.
    async fn get_language_image(&self, lang: Language) -> SandboxResult<ImageSpec>;
}

/// Unique identifier for a sandbox instance.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SandboxId(pub String);

impl SandboxId {
    /// Create a new sandbox ID.
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }
}

impl Default for SandboxId {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration for creating a sandbox.
#[derive(Debug, Clone)]
pub struct SandboxConfig {
    /// Docker image to use.
    pub image: String,
    /// Memory limit in MB.
    pub memory_limit_mb: u64,
    /// CPU limit (fractional cores).
    pub cpu_limit: f64,
    /// Disk limit in GB.
    pub disk_limit_gb: u64,
    /// Environment variables.
    pub env_vars: Vec<(String, String)>,
    /// Network access enabled?
    pub network_enabled: bool,
    /// Execution timeout in seconds.
    pub timeout_secs: u64,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            image: "ubuntu:22.04".to_string(),
            memory_limit_mb: 2048,
            cpu_limit: 2.0,
            disk_limit_gb: 10,
            env_vars: Vec::new(),
            network_enabled: true,
            timeout_secs: 300,
        }
    }
}

/// Output from executing a command in a sandbox.
#[derive(Debug, Clone)]
pub struct ExecOutput {
    /// Standard output.
    pub stdout: String,
    /// Standard error.
    pub stderr: String,
    /// Exit code (0 = success).
    pub exit_code: i64,
    /// Whether the command timed out.
    pub timed_out: bool,
}

impl ExecOutput {
    /// Returns true if the command exited successfully.
    pub fn success(&self) -> bool {
        self.exit_code == 0 && !self.timed_out
    }
}

/// An isolated execution environment (Docker container, etc.).
#[async_trait]
pub trait Sandbox: Send + Sync {
    /// Create a new sandbox with the given configuration.
    async fn create(&self, config: &SandboxConfig) -> SandboxResult<SandboxId>;

    /// Execute a command inside the sandbox.
    async fn exec(&self, id: &SandboxId, command: &str) -> SandboxResult<ExecOutput>;

    /// Write a file into the sandbox.
    async fn write_file(&self, id: &SandboxId, path: &Path, content: &[u8]) -> SandboxResult<()>;

    /// Read a file from the sandbox.
    async fn read_file(&self, id: &SandboxId, path: &Path) -> SandboxResult<Vec<u8>>;

    /// List files in a directory inside the sandbox.
    async fn list_files(&self, id: &SandboxId, path: &Path) -> SandboxResult<Vec<String>>;

    /// Destroy the sandbox and clean up resources.
    async fn destroy(&self, id: &SandboxId) -> SandboxResult<()>;
}

// =============================================================================
// Container Lifecycle Types
// =============================================================================

/// Resource limits for a sandbox container.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Memory limit in bytes (e.g., 512MB = 512 * 1024 * 1024).
    pub memory_bytes: Option<u64>,
    /// CPU quota (microseconds per `cpu_period`).
    pub cpu_quota: Option<i64>,
    /// CPU period in microseconds (default 100000 = 100ms).
    pub cpu_period: Option<i64>,
    /// Maximum number of processes/threads.
    pub pids_limit: Option<i64>,
    /// Disk quota in bytes.
    pub disk_bytes: Option<u64>,
    /// Execution timeout.
    pub timeout: Option<std::time::Duration>,
    /// Read-only root filesystem.
    pub read_only_rootfs: bool,
    /// Disable network access.
    pub network_disabled: bool,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            memory_bytes: None,
            cpu_quota: None,
            cpu_period: Some(100_000), // 100ms
            pids_limit: None,
            disk_bytes: None,
            timeout: None,
            read_only_rootfs: false,
            network_disabled: false,
        }
    }
}

impl ResourceLimits {
    /// Preset for lightweight tasks (256MB, 0.5 CPU, 30s timeout).
    pub fn light() -> Self {
        Self {
            memory_bytes: Some(256 * 1024 * 1024), // 256MB
            cpu_quota: Some(50_000),               // 0.5 CPU
            cpu_period: Some(100_000),
            pids_limit: Some(50),
            disk_bytes: Some(1024 * 1024 * 1024), // 1GB
            timeout: Some(std::time::Duration::from_secs(30)),
            read_only_rootfs: false,
            network_disabled: false,
        }
    }

    /// Preset for standard tasks (512MB, 1 CPU, 60s timeout).
    pub fn standard() -> Self {
        Self {
            memory_bytes: Some(512 * 1024 * 1024), // 512MB
            cpu_quota: Some(100_000),              // 1 CPU
            cpu_period: Some(100_000),
            pids_limit: Some(100),
            disk_bytes: Some(5 * 1024 * 1024 * 1024), // 5GB
            timeout: Some(std::time::Duration::from_secs(60)),
            read_only_rootfs: false,
            network_disabled: false,
        }
    }

    /// Preset for heavy tasks (2GB, 2 CPUs, 300s timeout).
    pub fn heavy() -> Self {
        Self {
            memory_bytes: Some(2 * 1024 * 1024 * 1024), // 2GB
            cpu_quota: Some(200_000),                   // 2 CPUs
            cpu_period: Some(100_000),
            pids_limit: Some(500),
            disk_bytes: Some(20 * 1024 * 1024 * 1024), // 20GB
            timeout: Some(std::time::Duration::from_secs(300)),
            read_only_rootfs: false,
            network_disabled: false,
        }
    }

    /// Builder pattern for custom limits.
    pub fn builder() -> ResourceLimitsBuilder {
        ResourceLimitsBuilder::default()
    }
}

/// Builder for `ResourceLimits`.
#[derive(Debug, Default)]
pub struct ResourceLimitsBuilder {
    limits: ResourceLimits,
}

impl ResourceLimitsBuilder {
    /// Set memory limit in bytes.
    pub fn memory(mut self, bytes: u64) -> Self {
        self.limits.memory_bytes = Some(bytes);
        self
    }

    /// Set memory limit in megabytes.
    pub fn memory_mb(mut self, mb: u64) -> Self {
        self.limits.memory_bytes = Some(mb * 1024 * 1024);
        self
    }

    /// Set CPU limit in fractional cores.
    pub fn cpu(mut self, cores: f64) -> Self {
        #[allow(clippy::cast_possible_truncation)]
        {
            self.limits.cpu_quota = Some((cores * 100_000.0) as i64);
        }
        self
    }

    /// Set maximum number of PIDs.
    pub fn pids(mut self, limit: i64) -> Self {
        self.limits.pids_limit = Some(limit);
        self
    }

    /// Set disk quota in bytes.
    pub fn disk(mut self, bytes: u64) -> Self {
        self.limits.disk_bytes = Some(bytes);
        self
    }

    /// Set disk quota in gigabytes.
    pub fn disk_gb(mut self, gb: u64) -> Self {
        self.limits.disk_bytes = Some(gb * 1024 * 1024 * 1024);
        self
    }

    /// Set execution timeout.
    pub fn timeout(mut self, duration: std::time::Duration) -> Self {
        self.limits.timeout = Some(duration);
        self
    }

    /// Set execution timeout in seconds.
    pub fn timeout_secs(mut self, secs: u64) -> Self {
        self.limits.timeout = Some(std::time::Duration::from_secs(secs));
        self
    }

    /// Enable read-only root filesystem.
    pub fn read_only(mut self) -> Self {
        self.limits.read_only_rootfs = true;
        self
    }

    /// Disable network access.
    pub fn no_network(mut self) -> Self {
        self.limits.network_disabled = true;
        self
    }

    /// Build the `ResourceLimits`.
    pub fn build(self) -> ResourceLimits {
        self.limits
    }
}

/// A volume mount binding a host path to a container path.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeMount {
    /// Path on the host filesystem.
    pub host_path: PathBuf,
    /// Path inside the container.
    pub container_path: PathBuf,
    /// Whether the mount is read-only.
    pub read_only: bool,
}

/// Configuration for creating a container.
#[derive(Debug, Clone)]
pub struct ContainerConfig {
    /// Docker image specification.
    pub image: ImageSpec,
    /// Optional container name.
    pub name: Option<String>,
    /// Working directory inside the container.
    pub working_dir: PathBuf,
    /// Environment variables.
    pub environment: HashMap<String, String>,
    /// Resource limits for the container.
    pub resource_limits: ResourceLimits,
    /// Volume mounts.
    pub volume_mounts: Vec<VolumeMount>,
    /// Whether network access is disabled.
    pub network_disabled: bool,
}

impl Default for ContainerConfig {
    fn default() -> Self {
        Self {
            image: ImageSpec {
                name: "ubuntu".to_string(),
                tag: "22.04".to_string(),
                language: Language::Generic,
                size_bytes: None,
                created_at: None,
            },
            name: None,
            working_dir: PathBuf::from("/workspace"),
            environment: HashMap::new(),
            resource_limits: ResourceLimits::default(),
            volume_mounts: Vec::new(),
            network_disabled: false,
        }
    }
}

/// Result of executing a command in a container.
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    /// Exit code of the command.
    pub exit_code: i64,
    /// Standard output.
    pub stdout: String,
    /// Standard error.
    pub stderr: String,
    /// Duration of the execution.
    pub duration: std::time::Duration,
}

impl ExecutionResult {
    /// Returns true if the command exited successfully (exit code 0).
    pub fn success(&self) -> bool {
        self.exit_code == 0
    }
}

/// Handle to a running or created sandbox container.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxHandle {
    /// Unique container ID.
    pub id: String,
    /// Container name.
    pub name: String,
    /// Image used to create the container.
    pub image: ImageSpec,
    /// When the container was created.
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Status of a container.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContainerStatus {
    /// Container has been created but not started.
    Created,
    /// Container is running.
    Running,
    /// Container is paused.
    Paused,
    /// Container has been stopped.
    Stopped,
    /// Container has been removed.
    Removed,
}

/// Trait for managing container lifecycle operations.
#[async_trait]
pub trait SandboxLifecycle: Send + Sync {
    /// Create a new container with the given configuration.
    async fn create(&self, config: ContainerConfig) -> SandboxResult<SandboxHandle>;

    /// Start a created container.
    async fn start(&self, handle: &SandboxHandle) -> SandboxResult<()>;

    /// Stop a running container with a timeout.
    async fn stop(&self, handle: &SandboxHandle, timeout_secs: u64) -> SandboxResult<()>;

    /// Remove a container.
    async fn remove(&self, handle: &SandboxHandle) -> SandboxResult<()>;

    /// Execute a command inside a running container.
    async fn execute(
        &self,
        handle: &SandboxHandle,
        command: &[String],
        timeout: std::time::Duration,
    ) -> SandboxResult<ExecutionResult>;

    /// Get the current status of a container.
    async fn status(&self, handle: &SandboxHandle) -> SandboxResult<ContainerStatus>;
}

// =============================================================================
// Volume Management Types
// =============================================================================

/// Handle to a Docker volume.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeHandle {
    /// Volume name (e.g., "cuttlefish-abc123").
    pub name: String,
    /// Mount point inside containers.
    pub mount_point: PathBuf,
    /// When the volume was created.
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Manager for sandbox volume operations.
#[async_trait]
pub trait VolumeManager: Send + Sync {
    /// Create a new named volume.
    async fn create_volume(&self, name: &str) -> SandboxResult<VolumeHandle>;

    /// Copy files from host path to volume.
    async fn copy_to_volume(
        &self,
        volume: &VolumeHandle,
        host_path: &Path,
        container_path: &Path,
    ) -> SandboxResult<()>;

    /// Copy files from volume to host path.
    async fn copy_from_volume(
        &self,
        volume: &VolumeHandle,
        container_path: &Path,
        host_path: &Path,
    ) -> SandboxResult<()>;

    /// Remove a volume.
    async fn remove_volume(&self, volume: &VolumeHandle) -> SandboxResult<()>;

    /// List all cuttlefish-managed volumes.
    async fn list_volumes(&self) -> SandboxResult<Vec<VolumeHandle>>;
}

// =============================================================================
// Snapshot Management Types
// =============================================================================

/// A snapshot of container state (stored as a Docker image).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    /// Unique snapshot ID (image ID).
    pub id: String,
    /// Human-readable name/tag.
    pub name: String,
    /// ID of the container this was created from.
    pub container_id: String,
    /// When the snapshot was created.
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Size in bytes.
    pub size_bytes: u64,
    /// Custom labels.
    pub labels: HashMap<String, String>,
}

/// Options for creating a snapshot.
#[derive(Debug, Clone, Default)]
pub struct SnapshotOptions {
    /// Custom name (auto-generated UUID if None).
    pub name: Option<String>,
    /// Custom labels to attach.
    pub labels: HashMap<String, String>,
    /// Pause container during snapshot for consistency.
    pub pause_container: bool,
}

/// Manager for container snapshots.
#[async_trait]
pub trait SnapshotManager: Send + Sync {
    /// Create a snapshot of a container's current state.
    async fn create_snapshot(
        &self,
        handle: &SandboxHandle,
        options: SnapshotOptions,
    ) -> SandboxResult<Snapshot>;

    /// Restore a container from a snapshot (creates new container).
    async fn restore_snapshot(
        &self,
        snapshot: &Snapshot,
        config: ContainerConfig,
    ) -> SandboxResult<SandboxHandle>;

    /// List all cuttlefish snapshots.
    async fn list_snapshots(&self) -> SandboxResult<Vec<Snapshot>>;

    /// Delete a snapshot.
    async fn delete_snapshot(&self, snapshot: &Snapshot) -> SandboxResult<()>;
}

// =============================================================================
// Cleanup Management Types
// =============================================================================

/// Policy for cleaning up sandbox resources.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupPolicy {
    /// Remove containers older than this duration.
    pub container_max_age: std::time::Duration,
    /// Remove snapshots older than this duration.
    pub snapshot_max_age: std::time::Duration,
    /// Remove volumes not attached to any container.
    pub remove_orphan_volumes: bool,
    /// Maximum snapshots to keep per container (oldest removed first).
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

/// Result of a cleanup operation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CleanupResult {
    /// Number of containers removed.
    pub containers_removed: usize,
    /// Number of volumes removed.
    pub volumes_removed: usize,
    /// Number of snapshots removed.
    pub snapshots_removed: usize,
    /// Bytes of disk space reclaimed.
    pub bytes_reclaimed: u64,
    /// Non-fatal errors encountered.
    pub errors: Vec<String>,
}

/// Current sandbox resource usage.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SandboxUsage {
    /// Number of active containers.
    pub container_count: usize,
    /// Number of volumes.
    pub volume_count: usize,
    /// Number of snapshots.
    pub snapshot_count: usize,
    /// Total bytes used.
    pub total_bytes: u64,
}

/// Manager for cleaning up sandbox resources.
#[async_trait]
pub trait CleanupManager: Send + Sync {
    /// Run cleanup with the given policy.
    async fn cleanup(&self, policy: &CleanupPolicy) -> SandboxResult<CleanupResult>;

    /// Get current resource usage.
    async fn get_usage(&self) -> SandboxResult<SandboxUsage>;

    /// Force remove ALL cuttlefish sandbox resources (dangerous!).
    async fn force_cleanup_all(&self) -> SandboxResult<CleanupResult>;
}

// =============================================================================
// Health Checking Types
// =============================================================================

/// Health status of the sandbox system
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SandboxHealth {
    /// Docker daemon is reachable
    pub docker_healthy: bool,
    /// Which language images are available (true = image exists)
    pub images_available: std::collections::HashMap<Language, bool>,
    /// Current resource usage
    pub resource_usage: SandboxUsage,
    /// Errors encountered during health check
    pub errors: Vec<String>,
}

impl SandboxHealth {
    /// Returns true if all systems are healthy
    pub fn is_healthy(&self) -> bool {
        self.docker_healthy && self.errors.is_empty()
    }

    /// Returns true if all required images are available
    pub fn all_images_available(&self) -> bool {
        self.images_available.values().all(|&available| available)
    }
}

/// Health checker for the sandbox system
#[async_trait]
pub trait HealthChecker: Send + Sync {
    /// Perform comprehensive health check
    async fn check_health(&self) -> SandboxResult<SandboxHealth>;

    /// Quick ping to verify Docker connectivity
    async fn ping(&self) -> SandboxResult<bool>;
}
