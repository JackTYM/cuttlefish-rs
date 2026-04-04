//! Container lifecycle management for Docker sandboxes.
//!
//! This module provides a granular container lifecycle API via the [`SandboxLifecycle`] trait,
//! separating create/start/stop/remove operations for finer control over container state.

use async_trait::async_trait;
use bollard::container::{
    Config, CreateContainerOptions, InspectContainerOptions, RemoveContainerOptions,
    StartContainerOptions, StopContainerOptions,
};
use bollard::exec::{CreateExecOptions, StartExecResults};
use bollard::Docker;
use chrono::Utc;
use cuttlefish_core::error::SandboxError;
use cuttlefish_core::traits::sandbox::{
    ContainerConfig, ContainerStatus, ExecutionResult, SandboxHandle, SandboxLifecycle,
    SandboxResult,
};
use futures::StreamExt;
use tracing::{debug, info, warn};

const MAX_OUTPUT_BYTES: usize = 1_024 * 1_024;

/// Docker-backed container lifecycle manager.
///
/// Provides granular control over container lifecycle operations:
/// - `create`: Create a container without starting it
/// - `start`: Start a created container
/// - `stop`: Stop a running container with a timeout
/// - `remove`: Remove a container (force removes if running)
/// - `execute`: Run a command inside a running container
/// - `status`: Query the current container state
pub struct DockerSandboxLifecycle {
    docker: Docker,
}

impl DockerSandboxLifecycle {
    /// Connect to the Docker daemon using default socket settings.
    ///
    /// # Errors
    ///
    /// Returns [`SandboxError::Other`] if the Docker daemon is unreachable.
    pub fn new() -> Result<Self, SandboxError> {
        let docker = Docker::connect_with_socket_defaults()
            .map_err(|e| SandboxError::Other(format!("Docker connect failed: {e}")))?;
        Ok(Self { docker })
    }

    /// Connect to the Docker daemon using a specific socket path.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the Docker socket (e.g., `/var/run/docker.sock`)
    ///
    /// # Errors
    ///
    /// Returns [`SandboxError::Other`] if the socket is unreachable.
    pub fn with_socket(path: &str) -> Result<Self, SandboxError> {
        let docker = Docker::connect_with_socket(path, 30, bollard::API_DEFAULT_VERSION)
            .map_err(|e| SandboxError::Other(format!("Docker socket {path} failed: {e}")))?;
        Ok(Self { docker })
    }

    fn build_docker_config(&self, config: &ContainerConfig) -> Config<String> {
        let binds: Vec<String> = config
            .volume_mounts
            .iter()
            .map(|m| {
                let mode = if m.read_only { "ro" } else { "rw" };
                format!(
                    "{}:{}:{}",
                    m.host_path.display(),
                    m.container_path.display(),
                    mode
                )
            })
            .collect();

        let env: Vec<String> = config
            .environment
            .iter()
            .map(|(k, v)| format!("{k}={v}"))
            .collect();

        let host_config = bollard::service::HostConfig {
            memory: config.resource_limits.memory_bytes.map(|m| m as i64),
            cpu_quota: config.resource_limits.cpu_quota,
            cpu_period: config.resource_limits.cpu_period,
            pids_limit: config.resource_limits.pids_limit,
            binds: if binds.is_empty() { None } else { Some(binds) },
            network_mode: if config.network_disabled {
                Some("none".into())
            } else {
                Some("bridge".into())
            },
            auto_remove: Some(false),
            ..Default::default()
        };

        let image_ref = format!("{}:{}", config.image.name, config.image.tag);

        Config {
            image: Some(image_ref),
            env: if env.is_empty() {
                None
            } else {
                Some(env)
            },
            host_config: Some(host_config),
            cmd: Some(vec!["tail".to_string(), "-f".to_string(), "/dev/null".to_string()]),
            working_dir: Some(config.working_dir.to_string_lossy().to_string()),
            ..Default::default()
        }
    }
}

#[async_trait]
impl SandboxLifecycle for DockerSandboxLifecycle {
    async fn create(&self, config: ContainerConfig) -> SandboxResult<SandboxHandle> {
        let container_name = config
            .name
            .clone()
            .unwrap_or_else(|| format!("cuttlefish-{}", uuid::Uuid::new_v4()));

        debug!(
            "Creating container {} with image {}:{}",
            container_name, config.image.name, config.image.tag
        );

        let docker_config = self.build_docker_config(&config);

        let options = CreateContainerOptions {
            name: container_name.as_str(),
            platform: None,
        };

        let response = self
            .docker
            .create_container(Some(options), docker_config)
            .await
            .map_err(|e| SandboxError::Other(format!("Failed to create container: {e}")))?;

        info!("Created container: {} ({})", container_name, response.id);

        Ok(SandboxHandle {
            id: response.id,
            name: container_name,
            image: config.image,
            created_at: Utc::now(),
        })
    }

    async fn start(&self, handle: &SandboxHandle) -> SandboxResult<()> {
        debug!("Starting container: {}", handle.id);

        self.docker
            .start_container(&handle.id, None::<StartContainerOptions<String>>)
            .await
            .map_err(|e| SandboxError::Other(format!("Failed to start container {}: {e}", handle.id)))?;

        info!("Started container: {}", handle.id);
        Ok(())
    }

    async fn stop(&self, handle: &SandboxHandle, timeout_secs: u64) -> SandboxResult<()> {
        debug!(
            "Stopping container {} with timeout {}s",
            handle.id, timeout_secs
        );

        let stop_opts = StopContainerOptions {
            t: timeout_secs as i64,
        };

        self.docker
            .stop_container(&handle.id, Some(stop_opts))
            .await
            .map_err(|e| {
                if e.to_string().contains("is not running") {
                    debug!("Container {} was already stopped", handle.id);
                    return SandboxError::Other(String::new());
                }
                SandboxError::Other(format!("Failed to stop container {}: {e}", handle.id))
            })?;

        info!("Stopped container: {}", handle.id);
        Ok(())
    }

    async fn remove(&self, handle: &SandboxHandle) -> SandboxResult<()> {
        debug!("Removing container: {}", handle.id);

        let remove_opts = RemoveContainerOptions {
            force: true,
            v: true,
            ..Default::default()
        };

        self.docker
            .remove_container(&handle.id, Some(remove_opts))
            .await
            .map_err(|e| {
                SandboxError::Other(format!("Failed to remove container {}: {e}", handle.id))
            })?;

        info!("Removed container: {}", handle.id);
        Ok(())
    }

    async fn execute(
        &self,
        handle: &SandboxHandle,
        command: &[String],
        timeout: std::time::Duration,
    ) -> SandboxResult<ExecutionResult> {
        let start_time = std::time::Instant::now();

        debug!(
            "Executing in container {}: {:?}",
            handle.id,
            command.join(" ")
        );

        let exec_options = CreateExecOptions {
            cmd: Some(command.iter().map(|s| s.as_str()).collect()),
            attach_stdout: Some(true),
            attach_stderr: Some(true),
            ..Default::default()
        };

        let exec = self
            .docker
            .create_exec(&handle.id, exec_options)
            .await
            .map_err(|e| {
            SandboxError::Other(format!("Failed to create exec in {}: {e}", handle.id))
        })?;

        let mut stdout_buf = String::new();
        let mut stderr_buf = String::new();

        let exec_future = async {
            if let StartExecResults::Attached { mut output, .. } = self
                .docker
                .start_exec(&exec.id, None)
                .await
                .map_err(|e| SandboxError::Other(format!("Failed to start exec: {e}")))?
            {
                let mut truncated = false;
                while let Some(chunk) = output.next().await {
                    match chunk {
                        Ok(bollard::container::LogOutput::StdOut { message }) => {
                            stdout_buf.push_str(&String::from_utf8_lossy(&message));
                            if stdout_buf.len() > MAX_OUTPUT_BYTES {
                                stdout_buf.truncate(MAX_OUTPUT_BYTES);
                                stdout_buf.push_str("\n[OUTPUT TRUNCATED - EXCEEDED 1MB LIMIT]");
                                truncated = true;
                            }
                        }
                        Ok(bollard::container::LogOutput::StdErr { message }) => {
                            stderr_buf.push_str(&String::from_utf8_lossy(&message));
                            if stderr_buf.len() > MAX_OUTPUT_BYTES {
                                stderr_buf.truncate(MAX_OUTPUT_BYTES);
                                stderr_buf.push_str("\n[OUTPUT TRUNCATED - EXCEEDED 1MB LIMIT]");
                                truncated = true;
                            }
                        }
                        Ok(_) => {}
                        Err(e) => warn!("Error reading exec output: {e}"),
                    }
                    if truncated {
                        warn!("Output truncated for exec in container {}", handle.id);
                        break;
                    }
                }
            }
            Ok::<(), SandboxError>(())
        };

        match tokio::time::timeout(timeout, exec_future).await {
            Ok(result) => result?,
            Err(_) => {
                warn!(
                    "Exec timed out after {:?} in container {}",
                    timeout, handle.id
                );
                return Ok(ExecutionResult {
                    exit_code: -1,
                    stdout: stdout_buf,
                    stderr: format!("Command timed out after {}s", timeout.as_secs()),
                    duration: start_time.elapsed(),
                });
            }
        }

        let inspect = self
            .docker
            .inspect_exec(&exec.id)
            .await
            .map_err(|e| SandboxError::Other(format!("Failed to inspect exec: {e}")))?;

        let exit_code = inspect.exit_code.unwrap_or(-1);
        let duration = start_time.elapsed();

        Ok(ExecutionResult {
            exit_code,
            stdout: stdout_buf,
            stderr: stderr_buf,
            duration,
        })
    }

    async fn status(&self, handle: &SandboxHandle) -> SandboxResult<ContainerStatus> {
        let inspect_opts = InspectContainerOptions { size: false };

        let info = self
            .docker
            .inspect_container(&handle.id, Some(inspect_opts))
            .await
            .map_err(|e| {
                if e.to_string().contains("No such container") {
                    return SandboxError::ContainerNotFound {
                        id: handle.id.clone(),
                    };
                }
                SandboxError::Other(format!("Failed to inspect container {}: {e}", handle.id))
            })?;

        let status = match info.state {
            Some(state) => {
                match state.status {
                    Some(bollard::service::ContainerStateStatusEnum::CREATED) => {
                        ContainerStatus::Created
                    }
                    Some(bollard::service::ContainerStateStatusEnum::RUNNING) => {
                        ContainerStatus::Running
                    }
                    Some(bollard::service::ContainerStateStatusEnum::PAUSED) => {
                        ContainerStatus::Paused
                    }
                    Some(bollard::service::ContainerStateStatusEnum::EXITED) => {
                        ContainerStatus::Stopped
                    }
                    Some(bollard::service::ContainerStateStatusEnum::DEAD) => {
                        ContainerStatus::Stopped
                    }
                    Some(bollard::service::ContainerStateStatusEnum::REMOVING) => {
                        ContainerStatus::Removed
                    }
                    Some(bollard::service::ContainerStateStatusEnum::RESTARTING) => {
                        ContainerStatus::Running
                    }
                    None => {
                        if state.running == Some(true) {
                            ContainerStatus::Running
                        } else if state.paused == Some(true) {
                            ContainerStatus::Paused
                        } else {
                            ContainerStatus::Stopped
                        }
                    }
                    _ => ContainerStatus::Stopped,
                }
            }
            None => ContainerStatus::Stopped,
        };

        debug!("Container {} status: {:?}", handle.id, status);
        Ok(status)
    }
}

#[cfg(test)]
pub use mock::MockSandboxLifecycle;

#[cfg(test)]
mod mock {
    //! Mock implementation of [`SandboxLifecycle`] for testing.

    use super::*;
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    /// Internal state for a mock container.
    struct MockContainer {
        #[allow(dead_code)]
        handle: SandboxHandle,
        status: ContainerStatus,
    }

    /// Mock implementation of [`SandboxLifecycle`] for unit testing.
    ///
    /// Tracks container state in memory without requiring Docker.
    pub struct MockSandboxLifecycle {
        containers: Arc<RwLock<HashMap<String, MockContainer>>>,
    }

    impl MockSandboxLifecycle {
        /// Create a new mock lifecycle manager.
        pub fn new() -> Self {
            Self {
                containers: Arc::new(RwLock::new(HashMap::new())),
            }
        }
    }

    impl Default for MockSandboxLifecycle {
        fn default() -> Self {
            Self::new()
        }
    }

    #[async_trait]
    impl SandboxLifecycle for MockSandboxLifecycle {
        async fn create(&self, config: ContainerConfig) -> SandboxResult<SandboxHandle> {
            let id = uuid::Uuid::new_v4().to_string();
            let name = config
                .name
                .clone()
                .unwrap_or_else(|| format!("mock-{}", &id[..8]));

            let handle = SandboxHandle {
                id: id.clone(),
                name,
                image: config.image,
                created_at: Utc::now(),
            };

            let container = MockContainer {
                handle: handle.clone(),
                status: ContainerStatus::Created,
            };

            self.containers.write().await.insert(id, container);
            Ok(handle)
        }

        async fn start(&self, handle: &SandboxHandle) -> SandboxResult<()> {
            let mut containers = self.containers.write().await;
            let container = containers.get_mut(&handle.id).ok_or_else(|| {
                SandboxError::ContainerNotFound {
                    id: handle.id.clone(),
                }
            })?;

            match container.status {
                ContainerStatus::Created | ContainerStatus::Stopped => {
                    container.status = ContainerStatus::Running;
                    Ok(())
                }
                ContainerStatus::Running => Ok(()),
                ContainerStatus::Paused => {
                    container.status = ContainerStatus::Running;
                    Ok(())
                }
                ContainerStatus::Removed => Err(SandboxError::ContainerNotFound {
                    id: handle.id.clone(),
                }),
            }
        }

        async fn stop(&self, handle: &SandboxHandle, _timeout_secs: u64) -> SandboxResult<()> {
            let mut containers = self.containers.write().await;
            let container = containers.get_mut(&handle.id).ok_or_else(|| {
                SandboxError::ContainerNotFound {
                    id: handle.id.clone(),
                }
            })?;

            match container.status {
                ContainerStatus::Running | ContainerStatus::Paused => {
                    container.status = ContainerStatus::Stopped;
                    Ok(())
                }
                ContainerStatus::Created | ContainerStatus::Stopped => Ok(()),
                ContainerStatus::Removed => Err(SandboxError::ContainerNotFound {
                    id: handle.id.clone(),
                }),
            }
        }

        async fn remove(&self, handle: &SandboxHandle) -> SandboxResult<()> {
            let mut containers = self.containers.write().await;
            if containers.remove(&handle.id).is_some() {
                Ok(())
            } else {
                Err(SandboxError::ContainerNotFound {
                    id: handle.id.clone(),
                })
            }
        }

        async fn execute(
            &self,
            handle: &SandboxHandle,
            command: &[String],
            _timeout: std::time::Duration,
        ) -> SandboxResult<ExecutionResult> {
            let containers = self.containers.read().await;
            let container = containers.get(&handle.id).ok_or_else(|| {
                SandboxError::ContainerNotFound {
                    id: handle.id.clone(),
                }
            })?;

            if container.status != ContainerStatus::Running {
                return Err(SandboxError::Other(format!(
                    "Container {} is not running",
                    handle.id
                )));
            }

            Ok(ExecutionResult {
                exit_code: 0,
                stdout: format!("mock: {}", command.join(" ")),
                stderr: String::new(),
                duration: std::time::Duration::from_millis(1),
            })
        }

        async fn status(&self, handle: &SandboxHandle) -> SandboxResult<ContainerStatus> {
            let containers = self.containers.read().await;
            let container = containers.get(&handle.id).ok_or_else(|| {
                SandboxError::ContainerNotFound {
                    id: handle.id.clone(),
                }
            })?;
            Ok(container.status)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cuttlefish_core::traits::sandbox::{ImageSpec, Language, ResourceLimits, VolumeMount};
    use std::path::PathBuf;

    fn default_container_config() -> ContainerConfig {
        ContainerConfig::default()
    }

    #[tokio::test]
    async fn test_mock_lifecycle_create() {
        let lifecycle = MockSandboxLifecycle::new();
        let config = default_container_config();

        let handle = lifecycle.create(config).await.expect("create should succeed");
        assert!(!handle.id.is_empty());
        assert!(!handle.name.is_empty());
    }

    #[tokio::test]
    async fn test_mock_lifecycle_state_transitions() {
        let lifecycle = MockSandboxLifecycle::new();
        let config = default_container_config();

        let handle = lifecycle.create(config).await.expect("create");
        assert_eq!(
            lifecycle.status(&handle).await.expect("status"),
            ContainerStatus::Created
        );

        lifecycle.start(&handle).await.expect("start");
        assert_eq!(
            lifecycle.status(&handle).await.expect("status"),
            ContainerStatus::Running
        );

        lifecycle.stop(&handle, 10).await.expect("stop");
        assert_eq!(
            lifecycle.status(&handle).await.expect("status"),
            ContainerStatus::Stopped
        );

        lifecycle.remove(&handle).await.expect("remove");
        assert!(lifecycle.status(&handle).await.is_err());
    }

    #[tokio::test]
    async fn test_mock_lifecycle_execute_requires_running() {
        let lifecycle = MockSandboxLifecycle::new();
        let config = default_container_config();

        let handle = lifecycle.create(config).await.expect("create");

        let result = lifecycle
            .execute(
                &handle,
                &["echo".to_string(), "hello".to_string()],
                std::time::Duration::from_secs(10),
            )
            .await;
        assert!(result.is_err());

        lifecycle.start(&handle).await.expect("start");
        let result = lifecycle
            .execute(
                &handle,
                &["echo".to_string(), "hello".to_string()],
                std::time::Duration::from_secs(10),
            )
            .await
            .expect("execute");
        assert!(result.success());
        assert!(result.stdout.contains("echo hello"));
    }

    #[tokio::test]
    async fn test_mock_lifecycle_remove_nonexistent() {
        let lifecycle = MockSandboxLifecycle::new();
        let fake_handle = SandboxHandle {
            id: "nonexistent".to_string(),
            name: "fake".to_string(),
            image: ImageSpec {
                name: "ubuntu".to_string(),
                tag: "22.04".to_string(),
                language: Language::Generic,
                size_bytes: None,
                created_at: None,
            },
            created_at: Utc::now(),
        };

        let result = lifecycle.remove(&fake_handle).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_execution_result_success() {
        let result = ExecutionResult {
            exit_code: 0,
            stdout: "ok".to_string(),
            stderr: String::new(),
            duration: std::time::Duration::from_millis(100),
        };
        assert!(result.success());
    }

    #[test]
    fn test_execution_result_failure() {
        let result = ExecutionResult {
            exit_code: 1,
            stdout: String::new(),
            stderr: "error".to_string(),
            duration: std::time::Duration::from_millis(100),
        };
        assert!(!result.success());
    }

    #[test]
    fn test_container_config_default() {
        let config = ContainerConfig::default();
        assert_eq!(config.image.name, "ubuntu");
        assert_eq!(config.image.tag, "22.04");
        assert_eq!(config.working_dir, PathBuf::from("/workspace"));
        assert!(!config.network_disabled);
        assert!(config.volume_mounts.is_empty());
    }

    #[test]
    fn test_resource_limits_default() {
        let limits = ResourceLimits::default();
        assert!(limits.memory_bytes.is_none());
        assert!(limits.cpu_quota.is_none());
        assert_eq!(limits.cpu_period, Some(100_000));
        assert!(limits.pids_limit.is_none());
        assert!(limits.disk_bytes.is_none());
        assert!(limits.timeout.is_none());
        assert!(!limits.read_only_rootfs);
        assert!(!limits.network_disabled);
    }

    #[test]
    fn test_volume_mount_format() {
        let mount = VolumeMount {
            host_path: PathBuf::from("/host/data"),
            container_path: PathBuf::from("/container/data"),
            read_only: true,
        };

        let mode = if mount.read_only { "ro" } else { "rw" };
        let bind = format!(
            "{}:{}:{}",
            mount.host_path.display(),
            mount.container_path.display(),
            mode
        );
        assert_eq!(bind, "/host/data:/container/data:ro");
    }

    #[test]
    fn test_container_status_variants() {
        let statuses = [
            ContainerStatus::Created,
            ContainerStatus::Running,
            ContainerStatus::Paused,
            ContainerStatus::Stopped,
            ContainerStatus::Removed,
        ];
        assert_eq!(statuses.len(), 5);
    }

    #[cfg(feature = "integration")]
    mod integration {
        use super::*;

        #[tokio::test]
        async fn test_docker_lifecycle_full() {
            let lifecycle =
                DockerSandboxLifecycle::new().expect("should connect to Docker");

            let config = ContainerConfig {
                image: ImageSpec {
                    name: "alpine".to_string(),
                    tag: "latest".to_string(),
                    language: Language::Generic,
                    size_bytes: None,
                    created_at: None,
                },
                name: Some(format!("cuttlefish-test-{}", uuid::Uuid::new_v4())),
                ..Default::default()
            };

            let handle = lifecycle.create(config).await.expect("create");
            assert_eq!(
                lifecycle.status(&handle).await.expect("status"),
                ContainerStatus::Created
            );

            lifecycle.start(&handle).await.expect("start");
            assert_eq!(
                lifecycle.status(&handle).await.expect("status"),
                ContainerStatus::Running
            );

            let result = lifecycle
                .execute(
                    &handle,
                    &["echo".to_string(), "hello".to_string()],
                    std::time::Duration::from_secs(30),
                )
                .await
                .expect("execute");
            assert!(result.success());
            assert!(result.stdout.contains("hello"));

            lifecycle.stop(&handle, 10).await.expect("stop");

            lifecycle.remove(&handle).await.expect("remove");
        }
    }
}
