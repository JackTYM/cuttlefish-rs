//! Container snapshot management using Docker image commits.

use async_trait::async_trait;
use bollard::container::{Config, CreateContainerOptions};
use bollard::image::{CommitContainerOptions, ListImagesOptions, RemoveImageOptions};
use bollard::Docker;
use cuttlefish_core::error::SandboxError;
use cuttlefish_core::traits::sandbox::{
    ContainerConfig, SandboxHandle, SandboxResult, Snapshot, SnapshotManager, SnapshotOptions,
};
use std::collections::HashMap;
use tracing::{debug, info};

const SNAPSHOT_REPO: &str = "cuttlefish-snapshot";
const SNAPSHOT_LABEL: &str = "cuttlefish.snapshot";

/// Docker-based snapshot manager using container commits.
pub struct DockerSnapshotManager {
    docker: Docker,
}

impl DockerSnapshotManager {
    /// Create a new snapshot manager connecting to the default Docker socket.
    pub fn new() -> Result<Self, SandboxError> {
        let docker = Docker::connect_with_socket_defaults()
            .map_err(|e| SandboxError::Other(format!("Docker connect failed: {e}")))?;
        Ok(Self { docker })
    }

    /// Create a snapshot manager with an existing Docker client.
    pub fn with_docker(docker: Docker) -> Self {
        Self { docker }
    }

    fn image_reference(&self, name: &str) -> String {
        format!("{SNAPSHOT_REPO}:{name}")
    }
}

#[async_trait]
impl SnapshotManager for DockerSnapshotManager {
    async fn create_snapshot(
        &self,
        handle: &SandboxHandle,
        options: SnapshotOptions,
    ) -> SandboxResult<Snapshot> {
        let name = options
            .name
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        debug!("Creating snapshot {} from container {}", name, handle.id);

        if options.pause_container {
            self.docker
                .pause_container(&handle.id)
                .await
                .map_err(|e| SandboxError::SnapshotError {
                    reason: format!("Failed to pause container: {e}"),
                })?;
        }

        let mut labels = options.labels.clone();
        labels.insert(SNAPSHOT_LABEL.to_string(), "true".to_string());
        labels.insert("cuttlefish.container_id".to_string(), handle.id.clone());

        let commit_options = CommitContainerOptions {
            container: handle.id.as_str(),
            repo: SNAPSHOT_REPO,
            tag: name.as_str(),
            pause: false,
            ..Default::default()
        };

        let container_config = bollard::container::Config::<String> {
            labels: Some(labels.clone()),
            ..Default::default()
        };

        let result = self
            .docker
            .commit_container(commit_options, container_config)
            .await
            .map_err(|e| SandboxError::SnapshotError {
                reason: format!("Failed to commit container: {e}"),
            })?;

        if options.pause_container {
            let _ = self.docker.unpause_container(&handle.id).await;
        }

        let snapshot_id = result.id.unwrap_or_default();
        info!("Created snapshot {} with ID {}", name, snapshot_id);

        Ok(Snapshot {
            id: snapshot_id,
            name,
            container_id: handle.id.clone(),
            created_at: chrono::Utc::now(),
            size_bytes: 0,
            labels,
        })
    }

    async fn restore_snapshot(
        &self,
        snapshot: &Snapshot,
        config: ContainerConfig,
    ) -> SandboxResult<SandboxHandle> {
        let image_ref = self.image_reference(&snapshot.name);
        debug!("Restoring snapshot {} to new container", image_ref);

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
            network_mode: if config.network_disabled {
                Some("none".to_string())
            } else {
                Some("bridge".to_string())
            },
            ..Default::default()
        };

        let container_name = config
            .name
            .unwrap_or_else(|| format!("cuttlefish-restored-{}", uuid::Uuid::new_v4()));

        let container_config = Config {
            image: Some(image_ref.clone()),
            env: Some(env),
            working_dir: Some(config.working_dir.to_string_lossy().to_string()),
            host_config: Some(host_config),
            cmd: Some(vec![
                "tail".to_string(),
                "-f".to_string(),
                "/dev/null".to_string(),
            ]),
            ..Default::default()
        };

        let options = CreateContainerOptions {
            name: container_name.as_str(),
            platform: None,
        };

        let response = self
            .docker
            .create_container(Some(options), container_config)
            .await
            .map_err(|e| SandboxError::SnapshotError {
                reason: format!("Failed to create container from snapshot: {e}"),
            })?;

        info!("Restored snapshot to container {}", response.id);

        Ok(SandboxHandle {
            id: response.id,
            name: container_name,
            image: config.image,
            created_at: chrono::Utc::now(),
        })
    }

    async fn list_snapshots(&self) -> SandboxResult<Vec<Snapshot>> {
        let mut filters = HashMap::new();
        filters.insert("reference".to_string(), vec![format!("{SNAPSHOT_REPO}:*")]);

        let options = ListImagesOptions {
            filters,
            ..Default::default()
        };

        let images = self
            .docker
            .list_images(Some(options))
            .await
            .map_err(|e| SandboxError::SnapshotError {
                reason: format!("Failed to list snapshots: {e}"),
            })?;

        let snapshots = images
            .into_iter()
            .map(|img| {
                let name = img
                    .repo_tags
                    .first()
                    .and_then(|t| t.strip_prefix(&format!("{SNAPSHOT_REPO}:")))
                    .unwrap_or("unknown")
                    .to_string();

                let labels = img.labels.clone();
                let container_id = labels
                    .get("cuttlefish.container_id")
                    .cloned()
                    .unwrap_or_default();

                Snapshot {
                    id: img.id,
                    name,
                    container_id,
                    created_at: chrono::DateTime::from_timestamp(img.created, 0)
                        .unwrap_or_else(chrono::Utc::now),
                    size_bytes: img.size as u64,
                    labels,
                }
            })
            .collect();

        Ok(snapshots)
    }

    async fn delete_snapshot(&self, snapshot: &Snapshot) -> SandboxResult<()> {
        let image_ref = self.image_reference(&snapshot.name);
        debug!("Deleting snapshot {}", image_ref);

        let options = RemoveImageOptions {
            force: true,
            noprune: false,
        };

        self.docker
            .remove_image(&image_ref, Some(options), None)
            .await
            .map_err(|e| SandboxError::SnapshotError {
                reason: format!("Failed to delete snapshot {image_ref}: {e}"),
            })?;

        info!("Deleted snapshot {}", image_ref);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image_reference() {
        let docker =
            Docker::connect_with_socket_defaults().expect("Docker connection should succeed");
        let manager = DockerSnapshotManager { docker };
        assert_eq!(
            manager.image_reference("test123"),
            "cuttlefish-snapshot:test123"
        );
    }

    #[test]
    fn test_snapshot_options_default() {
        let options = SnapshotOptions::default();
        assert!(options.name.is_none());
        assert!(!options.pause_container);
        assert!(options.labels.is_empty());
    }
}
