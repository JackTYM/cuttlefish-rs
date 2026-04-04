//! Docker volume management for Cuttlefish sandboxes.

use async_trait::async_trait;
use bollard::volume::{CreateVolumeOptions, ListVolumesOptions, RemoveVolumeOptions};
use bollard::Docker;
use cuttlefish_core::error::SandboxError;
use cuttlefish_core::traits::sandbox::{SandboxResult, VolumeHandle, VolumeManager};
use std::collections::HashMap;
use std::path::Path;
use tracing::{debug, info};

const VOLUME_PREFIX: &str = "cuttlefish-";

/// Docker-based volume manager.
pub struct DockerVolumeManager {
    docker: Docker,
}

impl DockerVolumeManager {
    /// Create a new DockerVolumeManager connected to the local Docker daemon.
    pub fn new() -> Result<Self, SandboxError> {
        let docker = Docker::connect_with_socket_defaults()
            .map_err(|e| SandboxError::Other(format!("Docker connect failed: {e}")))?;
        Ok(Self { docker })
    }

    fn prefixed_name(&self, name: &str) -> String {
        if name.starts_with(VOLUME_PREFIX) {
            name.to_string()
        } else {
            format!("{VOLUME_PREFIX}{name}")
        }
    }
}

#[async_trait]
impl VolumeManager for DockerVolumeManager {
    async fn create_volume(&self, name: &str) -> SandboxResult<VolumeHandle> {
        let full_name = self.prefixed_name(name);
        debug!("Creating volume: {}", full_name);

        let options = CreateVolumeOptions {
            name: full_name.as_str(),
            driver: "local",
            ..Default::default()
        };

        let volume = self
            .docker
            .create_volume(options)
            .await
            .map_err(|e| SandboxError::VolumeMountError {
                reason: format!("Failed to create volume {full_name}: {e}"),
            })?;

        info!("Created volume: {}", full_name);

        Ok(VolumeHandle {
            name: full_name,
            mount_point: std::path::PathBuf::from(volume.mountpoint),
            created_at: chrono::Utc::now(),
        })
    }

    async fn copy_to_volume(
        &self,
        volume: &VolumeHandle,
        host_path: &Path,
        container_path: &Path,
    ) -> SandboxResult<()> {
        // TODO: Implement using a temporary container with tar streaming
        debug!(
            "Copying {} to volume {} at {}",
            host_path.display(),
            volume.name,
            container_path.display()
        );

        Err(SandboxError::VolumeMountError {
            reason: "copy_to_volume not yet implemented".to_string(),
        })
    }

    async fn copy_from_volume(
        &self,
        volume: &VolumeHandle,
        container_path: &Path,
        host_path: &Path,
    ) -> SandboxResult<()> {
        // TODO: Implement using a temporary container with tar streaming
        debug!(
            "Copying from volume {} at {} to {}",
            volume.name,
            container_path.display(),
            host_path.display()
        );

        Err(SandboxError::VolumeMountError {
            reason: "copy_from_volume not yet implemented".to_string(),
        })
    }

    async fn remove_volume(&self, volume: &VolumeHandle) -> SandboxResult<()> {
        debug!("Removing volume: {}", volume.name);

        let options = RemoveVolumeOptions { force: true };

        self.docker
            .remove_volume(&volume.name, Some(options))
            .await
            .map_err(|e| SandboxError::VolumeMountError {
                reason: format!("Failed to remove volume {}: {e}", volume.name),
            })?;

        info!("Removed volume: {}", volume.name);
        Ok(())
    }

    async fn list_volumes(&self) -> SandboxResult<Vec<VolumeHandle>> {
        let mut filters = HashMap::new();
        filters.insert("name", vec![VOLUME_PREFIX]);

        let options = ListVolumesOptions { filters };

        let response = self
            .docker
            .list_volumes(Some(options))
            .await
            .map_err(|e| SandboxError::VolumeMountError {
                reason: format!("Failed to list volumes: {e}"),
            })?;

        let volumes = response
            .volumes
            .unwrap_or_default()
            .into_iter()
            .map(|v| VolumeHandle {
                name: v.name,
                mount_point: std::path::PathBuf::from(v.mountpoint),
                created_at: chrono::Utc::now(),
            })
            .collect();

        Ok(volumes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prefixed_name_adds_prefix() {
        let manager = DockerVolumeManager {
            docker: Docker::connect_with_socket_defaults()
                .expect("Docker connection required for test"),
        };
        assert_eq!(manager.prefixed_name("test"), "cuttlefish-test");
    }

    #[test]
    fn test_prefixed_name_preserves_existing_prefix() {
        let manager = DockerVolumeManager {
            docker: Docker::connect_with_socket_defaults()
                .expect("Docker connection required for test"),
        };
        assert_eq!(
            manager.prefixed_name("cuttlefish-test"),
            "cuttlefish-test"
        );
    }
}
