//! Health checking for the sandbox system.

use async_trait::async_trait;
use bollard::Docker;
use cuttlefish_core::error::SandboxError;
use cuttlefish_core::traits::sandbox::{
    HealthChecker, Language, SandboxHealth, SandboxResult, SandboxUsage,
};
use std::collections::HashMap;
use tracing::{debug, warn};

/// Docker-based health checker
pub struct DockerHealthChecker {
    docker: Docker,
}

impl DockerHealthChecker {
    /// Create a new health checker using default Docker socket
    pub fn new() -> Result<Self, SandboxError> {
        let docker = Docker::connect_with_socket_defaults()
            .map_err(|e| SandboxError::Other(format!("Docker connect failed: {e}")))?;
        Ok(Self { docker })
    }

    /// Create a health checker with a specific Docker instance
    pub fn with_docker(docker: Docker) -> Self {
        Self { docker }
    }

    async fn check_image_available(&self, image: &str) -> bool {
        self.docker.inspect_image(image).await.is_ok()
    }

    fn language_to_image(lang: Language) -> &'static str {
        match lang {
            Language::Node => "node:22-slim",
            Language::Python => "python:3.12-slim",
            Language::Rust => "rust:1.82-slim",
            Language::Go => "golang:1.22-bookworm",
            Language::Ruby => "ruby:3.3-slim",
            Language::Generic => "ubuntu:22.04",
        }
    }
}

#[async_trait]
impl HealthChecker for DockerHealthChecker {
    async fn check_health(&self) -> SandboxResult<SandboxHealth> {
        let mut health = SandboxHealth::default();

        // Check Docker daemon
        debug!("Checking Docker daemon health...");
        match self.docker.ping().await {
            Ok(_) => {
                health.docker_healthy = true;
                debug!("Docker daemon is healthy");
            }
            Err(e) => {
                let msg = format!("Docker daemon not reachable: {}", e);
                warn!("{}", msg);
                health.errors.push(msg);
            }
        }

        // Check language images
        debug!("Checking language image availability...");
        let languages = [
            Language::Node,
            Language::Python,
            Language::Rust,
            Language::Go,
            Language::Ruby,
            Language::Generic,
        ];

        for lang in languages {
            let image = Self::language_to_image(lang);
            let available = self.check_image_available(image).await;
            health.images_available.insert(lang, available);

            if !available {
                debug!("Image {} for {:?} not found locally", image, lang);
            }
        }

        // Get resource usage
        if health.docker_healthy {
            health.resource_usage = self.get_usage_internal().await;
        }

        Ok(health)
    }

    async fn ping(&self) -> SandboxResult<bool> {
        match self.docker.ping().await {
            Ok(_) => Ok(true),
            Err(e) => Err(SandboxError::Other(format!("Docker ping failed: {}", e))),
        }
    }
}

impl DockerHealthChecker {
    async fn get_usage_internal(&self) -> SandboxUsage {
        use bollard::container::ListContainersOptions;
        use bollard::volume::ListVolumesOptions;

        let mut usage = SandboxUsage::default();

        // Count cuttlefish containers
        let mut filters = HashMap::new();
        filters.insert("name", vec!["cuttlefish-"]);
        let options = ListContainersOptions {
            all: true,
            filters,
            ..Default::default()
        };
        if let Ok(containers) = self.docker.list_containers(Some(options)).await {
            usage.container_count = containers.len();
        }

        // Count cuttlefish volumes
        let mut filters = HashMap::new();
        filters.insert("name", vec!["cuttlefish-"]);
        let options = ListVolumesOptions { filters };
        if let Ok(response) = self.docker.list_volumes(Some(options)).await {
            usage.volume_count = response.volumes.map(|v| v.len()).unwrap_or(0);
        }

        usage
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sandbox_health_default() {
        let health = SandboxHealth::default();
        assert!(!health.docker_healthy);
        assert!(health.images_available.is_empty());
        assert!(health.errors.is_empty());
    }

    #[test]
    fn test_sandbox_health_is_healthy() {
        let health = SandboxHealth {
            docker_healthy: true,
            images_available: HashMap::new(),
            resource_usage: SandboxUsage::default(),
            errors: Vec::new(),
        };
        assert!(health.is_healthy());
    }

    #[test]
    fn test_sandbox_health_unhealthy_with_errors() {
        let health = SandboxHealth {
            docker_healthy: true,
            images_available: HashMap::new(),
            resource_usage: SandboxUsage::default(),
            errors: vec!["some error".to_string()],
        };
        assert!(!health.is_healthy());
    }

    #[test]
    fn test_sandbox_health_unhealthy_docker() {
        let health = SandboxHealth {
            docker_healthy: false,
            images_available: HashMap::new(),
            resource_usage: SandboxUsage::default(),
            errors: Vec::new(),
        };
        assert!(!health.is_healthy());
    }

    #[test]
    fn test_all_images_available_empty() {
        let health = SandboxHealth::default();
        assert!(health.all_images_available()); // Empty = all available
    }

    #[test]
    fn test_all_images_available_true() {
        let mut images = HashMap::new();
        images.insert(Language::Node, true);
        images.insert(Language::Python, true);

        let health = SandboxHealth {
            docker_healthy: true,
            images_available: images,
            resource_usage: SandboxUsage::default(),
            errors: Vec::new(),
        };
        assert!(health.all_images_available());
    }

    #[test]
    fn test_all_images_available_false() {
        let mut images = HashMap::new();
        images.insert(Language::Node, true);
        images.insert(Language::Python, false);

        let health = SandboxHealth {
            docker_healthy: true,
            images_available: images,
            resource_usage: SandboxUsage::default(),
            errors: Vec::new(),
        };
        assert!(!health.all_images_available());
    }

    #[test]
    fn test_language_to_image() {
        assert_eq!(
            DockerHealthChecker::language_to_image(Language::Node),
            "node:22-slim"
        );
        assert_eq!(
            DockerHealthChecker::language_to_image(Language::Python),
            "python:3.12-slim"
        );
        assert_eq!(
            DockerHealthChecker::language_to_image(Language::Rust),
            "rust:1.82-slim"
        );
        assert_eq!(
            DockerHealthChecker::language_to_image(Language::Go),
            "golang:1.22-bookworm"
        );
        assert_eq!(
            DockerHealthChecker::language_to_image(Language::Ruby),
            "ruby:3.3-slim"
        );
        assert_eq!(
            DockerHealthChecker::language_to_image(Language::Generic),
            "ubuntu:22.04"
        );
    }
}
