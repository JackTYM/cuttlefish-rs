//! Cleanup and garbage collection for sandbox resources.

use async_trait::async_trait;
use bollard::container::{ListContainersOptions, RemoveContainerOptions};
use bollard::image::{ListImagesOptions, RemoveImageOptions};
use bollard::volume::{ListVolumesOptions, RemoveVolumeOptions};
use bollard::Docker;
use cuttlefish_core::error::SandboxError;
use cuttlefish_core::traits::sandbox::{
    CleanupManager, CleanupPolicy, CleanupResult, SandboxResult, SandboxUsage,
};
use std::collections::HashMap;
use tracing::{debug, info, warn};

const CONTAINER_PREFIX: &str = "cuttlefish-";
const VOLUME_PREFIX: &str = "cuttlefish-";
const SNAPSHOT_REPO: &str = "cuttlefish-snapshot";

/// Docker-based cleanup manager for sandbox resources.
pub struct DockerCleanupManager {
    docker: Docker,
}

impl DockerCleanupManager {
    /// Create a new cleanup manager using default Docker socket.
    pub fn new() -> Result<Self, SandboxError> {
        let docker = Docker::connect_with_socket_defaults()
            .map_err(|e| SandboxError::Other(format!("Docker connect failed: {e}")))?;
        Ok(Self { docker })
    }

    /// Create a cleanup manager with an existing Docker client.
    pub fn with_docker(docker: Docker) -> Self {
        Self { docker }
    }

    async fn cleanup_containers(&self, max_age: std::time::Duration) -> (usize, Vec<String>) {
        let mut removed = 0;
        let mut errors = Vec::new();

        let mut filters = HashMap::new();
        filters.insert("name", vec![CONTAINER_PREFIX]);

        let options = ListContainersOptions {
            all: true,
            filters,
            ..Default::default()
        };

        match self.docker.list_containers(Some(options)).await {
            Ok(containers) => {
                let cutoff =
                    chrono::Utc::now() - chrono::Duration::from_std(max_age).unwrap_or_default();

                for container in containers {
                    if let (Some(id), Some(created)) = (container.id.as_ref(), container.created) {
                        let created_time = chrono::DateTime::from_timestamp(created, 0)
                            .unwrap_or_else(chrono::Utc::now);

                        if created_time < cutoff {
                            debug!("Removing stale container: {}", id);
                            let opts = RemoveContainerOptions {
                                force: true,
                                v: true,
                                ..Default::default()
                            };

                            match self.docker.remove_container(id, Some(opts)).await {
                                Ok(()) => {
                                    removed += 1;
                                    info!("Removed stale container: {}", id);
                                }
                                Err(e) => {
                                    let msg = format!("Failed to remove container {id}: {e}");
                                    warn!("{}", msg);
                                    errors.push(msg);
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => {
                errors.push(format!("Failed to list containers: {e}"));
            }
        }

        (removed, errors)
    }

    async fn cleanup_volumes(&self, remove_orphans: bool) -> (usize, Vec<String>) {
        if !remove_orphans {
            return (0, Vec::new());
        }

        let mut removed = 0;
        let mut errors = Vec::new();

        let mut filters = HashMap::new();
        filters.insert("name", vec![VOLUME_PREFIX]);
        filters.insert("dangling", vec!["true"]);

        let options = ListVolumesOptions { filters };

        match self.docker.list_volumes(Some(options)).await {
            Ok(response) => {
                for volume in response.volumes.unwrap_or_default() {
                    debug!("Removing orphan volume: {}", volume.name);
                    let opts = RemoveVolumeOptions { force: true };

                    match self.docker.remove_volume(&volume.name, Some(opts)).await {
                        Ok(()) => {
                            removed += 1;
                            info!("Removed orphan volume: {}", volume.name);
                        }
                        Err(e) => {
                            let msg = format!("Failed to remove volume {}: {e}", volume.name);
                            warn!("{}", msg);
                            errors.push(msg);
                        }
                    }
                }
            }
            Err(e) => {
                errors.push(format!("Failed to list volumes: {e}"));
            }
        }

        (removed, errors)
    }

    async fn cleanup_snapshots(&self, max_age: std::time::Duration) -> (usize, u64, Vec<String>) {
        let mut removed = 0;
        let mut bytes_reclaimed = 0u64;
        let mut errors = Vec::new();

        let mut filters = HashMap::new();
        filters.insert("reference".to_string(), vec![format!("{SNAPSHOT_REPO}:*")]);

        let options = ListImagesOptions {
            filters,
            ..Default::default()
        };

        match self.docker.list_images(Some(options)).await {
            Ok(images) => {
                let cutoff =
                    chrono::Utc::now() - chrono::Duration::from_std(max_age).unwrap_or_default();

                for image in images {
                    let created_time = chrono::DateTime::from_timestamp(image.created, 0)
                        .unwrap_or_else(chrono::Utc::now);

                    if created_time < cutoff {
                        let image_id = &image.id;
                        debug!("Removing old snapshot: {}", image_id);

                        let opts = RemoveImageOptions {
                            force: true,
                            noprune: false,
                        };

                        match self.docker.remove_image(image_id, Some(opts), None).await {
                            Ok(_) => {
                                removed += 1;
                                #[allow(clippy::cast_sign_loss)]
                                {
                                    bytes_reclaimed += image.size as u64;
                                }
                                info!("Removed old snapshot: {}", image_id);
                            }
                            Err(e) => {
                                let msg = format!("Failed to remove snapshot {image_id}: {e}");
                                warn!("{}", msg);
                                errors.push(msg);
                            }
                        }
                    }
                }
            }
            Err(e) => {
                errors.push(format!("Failed to list snapshots: {e}"));
            }
        }

        (removed, bytes_reclaimed, errors)
    }
}

#[async_trait]
impl CleanupManager for DockerCleanupManager {
    async fn cleanup(&self, policy: &CleanupPolicy) -> SandboxResult<CleanupResult> {
        let mut result = CleanupResult::default();

        let (containers, container_errors) =
            self.cleanup_containers(policy.container_max_age).await;
        result.containers_removed = containers;
        result.errors.extend(container_errors);

        let (volumes, volume_errors) = self.cleanup_volumes(policy.remove_orphan_volumes).await;
        result.volumes_removed = volumes;
        result.errors.extend(volume_errors);

        let (snapshots, bytes, snapshot_errors) =
            self.cleanup_snapshots(policy.snapshot_max_age).await;
        result.snapshots_removed = snapshots;
        result.bytes_reclaimed = bytes;
        result.errors.extend(snapshot_errors);

        info!(
            containers = result.containers_removed,
            volumes = result.volumes_removed,
            snapshots = result.snapshots_removed,
            bytes = result.bytes_reclaimed,
            "Cleanup completed"
        );

        Ok(result)
    }

    async fn get_usage(&self) -> SandboxResult<SandboxUsage> {
        let mut usage = SandboxUsage::default();

        let mut filters = HashMap::new();
        filters.insert("name", vec![CONTAINER_PREFIX]);
        let options = ListContainersOptions {
            all: true,
            filters,
            ..Default::default()
        };
        if let Ok(containers) = self.docker.list_containers(Some(options)).await {
            usage.container_count = containers.len();
        }

        let mut filters = HashMap::new();
        filters.insert("name", vec![VOLUME_PREFIX]);
        let options = ListVolumesOptions { filters };
        if let Ok(response) = self.docker.list_volumes(Some(options)).await {
            usage.volume_count = response.volumes.map(|v| v.len()).unwrap_or(0);
        }

        let mut filters = HashMap::new();
        filters.insert("reference".to_string(), vec![format!("{SNAPSHOT_REPO}:*")]);
        let options = ListImagesOptions {
            filters,
            ..Default::default()
        };
        if let Ok(images) = self.docker.list_images(Some(options)).await {
            usage.snapshot_count = images.len();
            #[allow(clippy::cast_sign_loss)]
            {
                usage.total_bytes = images.iter().map(|i| i.size as u64).sum();
            }
        }

        Ok(usage)
    }

    async fn force_cleanup_all(&self) -> SandboxResult<CleanupResult> {
        warn!("Force cleanup ALL cuttlefish resources requested");

        let policy = CleanupPolicy {
            container_max_age: std::time::Duration::ZERO,
            snapshot_max_age: std::time::Duration::ZERO,
            remove_orphan_volumes: true,
            max_snapshots_per_container: Some(0),
        };

        self.cleanup(&policy).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_cleanup_policy_default() {
        let policy = CleanupPolicy::default();
        assert_eq!(policy.container_max_age, Duration::from_secs(3600));
        assert_eq!(policy.snapshot_max_age, Duration::from_secs(86400));
        assert!(policy.remove_orphan_volumes);
        assert_eq!(policy.max_snapshots_per_container, Some(5));
    }

    #[test]
    fn test_cleanup_result_default() {
        let result = CleanupResult::default();
        assert_eq!(result.containers_removed, 0);
        assert_eq!(result.volumes_removed, 0);
        assert_eq!(result.snapshots_removed, 0);
        assert_eq!(result.bytes_reclaimed, 0);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_sandbox_usage_default() {
        let usage = SandboxUsage::default();
        assert_eq!(usage.container_count, 0);
        assert_eq!(usage.volume_count, 0);
        assert_eq!(usage.snapshot_count, 0);
        assert_eq!(usage.total_bytes, 0);
    }
}
