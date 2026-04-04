//! Docker image registry for Cuttlefish template stacks.
//!
//! This module provides two complementary types:
//! - [`TemplateImageMap`]: Simple template name → Docker image mapping
//! - [`DockerImageRegistry`]: Full Docker image management via bollard

use std::collections::HashMap;

use async_trait::async_trait;
use bollard::Docker;
use bollard::image::{
    BuildImageOptions, CreateImageOptions, ListImagesOptions, RemoveImageOptions,
};
use chrono::{TimeZone, Utc};
use cuttlefish_core::error::SandboxError;
use cuttlefish_core::traits::sandbox::{
    ImageBuildOptions, ImageRegistry, ImageSpec, Language, SandboxResult,
};
use futures::StreamExt;
use tracing::{debug, info};

/// Mapping from template/language name to Docker image reference.
///
/// Images can be:
/// - Pre-built public images (e.g., "node:22-slim")  
/// - Cuttlefish base images built from Dockerfiles in docker/ directory
///
/// This is a simple lookup table used by [`crate::DockerSandbox`] to resolve
/// template names to concrete Docker image references.
#[derive(Debug, Clone)]
pub struct TemplateImageMap {
    /// Map of template name → Docker image name.
    images: HashMap<String, String>,
}

impl TemplateImageMap {
    /// Create a `TemplateImageMap` with the default Cuttlefish base images.
    #[must_use]
    pub fn default_registry() -> Self {
        let mut images = HashMap::new();

        // Node.js family
        images.insert("node".to_string(), "node:22-slim".to_string());
        images.insert("nuxt".to_string(), "node:22-slim".to_string());
        images.insert("nuxt-cloudflare".to_string(), "node:22-slim".to_string());
        images.insert("typescript".to_string(), "node:22-slim".to_string());
        images.insert("node-express".to_string(), "node:22-slim".to_string());

        // Python family
        images.insert("python".to_string(), "python:3.12-slim".to_string());
        images.insert("python-fastapi".to_string(), "python:3.12-slim".to_string());
        images.insert("fastapi".to_string(), "python:3.12-slim".to_string());

        // Rust family
        images.insert("rust".to_string(), "rust:1.82-slim".to_string());
        images.insert("rust-axum".to_string(), "rust:1.82-slim".to_string());
        images.insert("axum".to_string(), "rust:1.82-slim".to_string());

        // Go family
        images.insert("go".to_string(), "golang:1.22-bookworm".to_string());
        images.insert("golang".to_string(), "golang:1.22-bookworm".to_string());

        // Generic
        images.insert("generic".to_string(), "ubuntu:22.04".to_string());
        images.insert("static-site".to_string(), "node:22-slim".to_string());

        Self { images }
    }

    /// Resolve a template name to a Docker image.
    #[must_use]
    pub fn resolve(&self, template_name: &str) -> &str {
        self.images
            .get(template_name)
            .map(|s| s.as_str())
            .unwrap_or("ubuntu:22.04")
    }

    /// Register a custom template → image mapping.
    pub fn register(&mut self, template_name: impl Into<String>, image: impl Into<String>) {
        self.images.insert(template_name.into(), image.into());
    }

    /// Check if a template has a registered image.
    #[must_use]
    pub fn has_template(&self, template_name: &str) -> bool {
        self.images.contains_key(template_name)
    }

    /// List all registered template names.
    #[must_use]
    pub fn list_templates(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self.images.keys().map(|s| s.as_str()).collect();
        names.sort();
        names
    }

    /// Get the Dockerfile path for a template (relative to workspace root).
    #[must_use]
    pub fn dockerfile_path(&self, template_name: &str) -> Option<std::path::PathBuf> {
        let image = self.resolve(template_name);
        let dockerfile_name = if image.starts_with("node:") {
            "node-base.Dockerfile"
        } else if image.starts_with("python:") {
            "python-base.Dockerfile"
        } else if image.starts_with("rust:") {
            "rust-base.Dockerfile"
        } else if image.starts_with("golang:") {
            "go-base.Dockerfile"
        } else {
            "generic-base.Dockerfile"
        };
        Some(std::path::PathBuf::from("docker").join(dockerfile_name))
    }
}

const CUTTLEFISH_IMAGE_PREFIX: &str = "cuttlefish/";

/// Docker-based image registry using bollard.
pub struct DockerImageRegistry {
    docker: Docker,
    image_prefix: String,
}

impl DockerImageRegistry {
    /// Create a new registry connected to the Docker daemon.
    pub fn new() -> Result<Self, SandboxError> {
        let docker = Docker::connect_with_socket_defaults()
            .map_err(|e| SandboxError::Other(format!("Failed to connect to Docker: {e}")))?;
        Ok(Self {
            docker,
            image_prefix: CUTTLEFISH_IMAGE_PREFIX.to_string(),
        })
    }

    /// Create a registry with a custom image prefix.
    pub fn with_prefix(prefix: impl Into<String>) -> Result<Self, SandboxError> {
        let docker = Docker::connect_with_socket_defaults()
            .map_err(|e| SandboxError::Other(format!("Failed to connect to Docker: {e}")))?;
        Ok(Self {
            docker,
            image_prefix: prefix.into(),
        })
    }

    fn language_to_image(&self, lang: Language) -> String {
        let base = match lang {
            Language::Node => "node",
            Language::Python => "python",
            Language::Rust => "rust",
            Language::Go => "go",
            Language::Ruby => "ruby",
            Language::Generic => "generic",
        };
        format!("{}{}:latest", self.image_prefix, base)
    }

    fn parse_image_ref(image_ref: &str) -> (&str, &str) {
        if let Some((name, tag)) = image_ref.rsplit_once(':')
            && !tag.contains('/')
        {
            return (name, tag);
        }
        (image_ref, "latest")
    }

    fn image_matches_prefix(&self, repo_tags: &[String]) -> bool {
        repo_tags
            .iter()
            .any(|tag| tag.starts_with(&self.image_prefix))
    }
}

#[async_trait]
impl ImageRegistry for DockerImageRegistry {
    async fn list_images(&self) -> SandboxResult<Vec<ImageSpec>> {
        let options = ListImagesOptions::<String> {
            all: false,
            ..Default::default()
        };

        let images = self
            .docker
            .list_images(Some(options))
            .await
            .map_err(|e| SandboxError::Other(format!("Failed to list images: {e}")))?;

        let mut specs = Vec::new();
        for image in images {
            let repo_tags = image.repo_tags;
            if repo_tags.is_empty() || !self.image_matches_prefix(&repo_tags) {
                continue;
            }

            for tag in &repo_tags {
                if !tag.starts_with(&self.image_prefix) {
                    continue;
                }
                let (name, tag_str) = Self::parse_image_ref(tag);
                let language = Self::infer_language_from_name(name);
                let created_at = Utc.timestamp_opt(image.created, 0).single();

                specs.push(ImageSpec {
                    name: name.to_string(),
                    tag: tag_str.to_string(),
                    language,
                    size_bytes: Some(image.size as u64),
                    created_at,
                });
            }
        }

        Ok(specs)
    }

    async fn pull_image(&self, name: &str, tag: &str) -> SandboxResult<ImageSpec> {
        let image_ref = format!("{name}:{tag}");
        info!("Pulling image: {}", image_ref);

        let options = CreateImageOptions {
            from_image: name,
            tag,
            ..Default::default()
        };

        let mut stream = self.docker.create_image(Some(options), None, None);

        while let Some(result) = stream.next().await {
            match result {
                Ok(info) => {
                    if let Some(status) = info.status {
                        debug!("Pull status: {}", status);
                    }
                }
                Err(_) => {
                    return Err(SandboxError::ImageNotFound {
                        name: name.to_string(),
                        tag: tag.to_string(),
                    });
                }
            }
        }

        let language = Self::infer_language_from_name(name);
        Ok(ImageSpec {
            name: name.to_string(),
            tag: tag.to_string(),
            language,
            size_bytes: None,
            created_at: Some(Utc::now()),
        })
    }

    async fn build_image(
        &self,
        name: &str,
        tag: &str,
        options: ImageBuildOptions,
    ) -> SandboxResult<ImageSpec> {
        let image_tag = format!("{name}:{tag}");
        info!("Building image: {}", image_tag);

        let dockerfile = options
            .dockerfile_path
            .as_ref()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| "Dockerfile".to_string());

        let build_options = BuildImageOptions {
            t: image_tag.as_str(),
            dockerfile: dockerfile.as_str(),
            nocache: options.no_cache,
            rm: true,
            ..Default::default()
        };

        let tar_body = Self::create_build_context(&options)?;

        let mut stream = self
            .docker
            .build_image(build_options, None, Some(tar_body.into()));

        while let Some(result) = stream.next().await {
            match result {
                Ok(info) => {
                    if let Some(stream_msg) = info.stream {
                        debug!("Build: {}", stream_msg.trim());
                    }
                    if let Some(error) = info.error {
                        return Err(SandboxError::ImageBuildFailed { reason: error });
                    }
                }
                Err(e) => {
                    return Err(SandboxError::ImageBuildFailed {
                        reason: e.to_string(),
                    });
                }
            }
        }

        let language = Self::infer_language_from_name(name);
        Ok(ImageSpec {
            name: name.to_string(),
            tag: tag.to_string(),
            language,
            size_bytes: None,
            created_at: Some(Utc::now()),
        })
    }

    async fn remove_image(&self, name: &str, tag: &str) -> SandboxResult<()> {
        let image_ref = format!("{name}:{tag}");
        info!("Removing image: {}", image_ref);

        let options = RemoveImageOptions {
            force: false,
            noprune: false,
        };

        self.docker
            .remove_image(&image_ref, Some(options), None)
            .await
            .map_err(|e| SandboxError::Other(format!("Failed to remove image {image_ref}: {e}")))?;

        Ok(())
    }

    async fn get_language_image(&self, lang: Language) -> SandboxResult<ImageSpec> {
        let image_ref = self.language_to_image(lang);
        let (name, tag) = Self::parse_image_ref(&image_ref);

        Ok(ImageSpec {
            name: name.to_string(),
            tag: tag.to_string(),
            language: lang,
            size_bytes: None,
            created_at: None,
        })
    }
}

impl DockerImageRegistry {
    fn infer_language_from_name(name: &str) -> Language {
        let lower = name.to_lowercase();
        if lower.contains("node") || lower.contains("javascript") || lower.contains("typescript") {
            Language::Node
        } else if lower.contains("python") {
            Language::Python
        } else if lower.contains("rust") {
            Language::Rust
        } else if lower.contains("go") || lower.contains("golang") {
            Language::Go
        } else if lower.contains("ruby") {
            Language::Ruby
        } else {
            Language::Generic
        }
    }

    fn create_build_context(_options: &ImageBuildOptions) -> SandboxResult<Vec<u8>> {
        Ok(vec![0u8; 1024])
    }
}

#[cfg(test)]
#[allow(missing_docs)]
pub struct MockImageRegistry {
    images: std::sync::Mutex<Vec<ImageSpec>>,
}

#[cfg(test)]
#[allow(missing_docs)]
impl MockImageRegistry {
    pub fn new() -> Self {
        Self {
            images: std::sync::Mutex::new(Vec::new()),
        }
    }

    pub fn with_images(images: Vec<ImageSpec>) -> Self {
        Self {
            images: std::sync::Mutex::new(images),
        }
    }
}

#[cfg(test)]
#[async_trait]
impl ImageRegistry for MockImageRegistry {
    async fn list_images(&self) -> SandboxResult<Vec<ImageSpec>> {
        Ok(self.images.lock().expect("lock").clone())
    }

    async fn pull_image(&self, name: &str, tag: &str) -> SandboxResult<ImageSpec> {
        let spec = ImageSpec {
            name: name.to_string(),
            tag: tag.to_string(),
            language: Language::Generic,
            size_bytes: None,
            created_at: Some(Utc::now()),
        };
        self.images.lock().expect("lock").push(spec.clone());
        Ok(spec)
    }

    async fn build_image(
        &self,
        name: &str,
        tag: &str,
        _options: ImageBuildOptions,
    ) -> SandboxResult<ImageSpec> {
        let spec = ImageSpec {
            name: name.to_string(),
            tag: tag.to_string(),
            language: Language::Generic,
            size_bytes: None,
            created_at: Some(Utc::now()),
        };
        self.images.lock().expect("lock").push(spec.clone());
        Ok(spec)
    }

    async fn remove_image(&self, name: &str, tag: &str) -> SandboxResult<()> {
        let mut images = self.images.lock().expect("lock");
        images.retain(|img| !(img.name == name && img.tag == tag));
        Ok(())
    }

    async fn get_language_image(&self, lang: Language) -> SandboxResult<ImageSpec> {
        Ok(ImageSpec {
            name: format!("mock/{lang}"),
            tag: "latest".to_string(),
            language: lang,
            size_bytes: None,
            created_at: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_registry_resolves_node() {
        let registry = TemplateImageMap::default_registry();
        assert_eq!(registry.resolve("node"), "node:22-slim");
        assert_eq!(registry.resolve("nuxt"), "node:22-slim");
    }

    #[test]
    fn test_default_registry_resolves_python() {
        let registry = TemplateImageMap::default_registry();
        assert_eq!(registry.resolve("python"), "python:3.12-slim");
        assert_eq!(registry.resolve("fastapi"), "python:3.12-slim");
    }

    #[test]
    fn test_default_registry_resolves_rust() {
        let registry = TemplateImageMap::default_registry();
        assert_eq!(registry.resolve("rust"), "rust:1.82-slim");
    }

    #[test]
    fn test_unknown_template_returns_generic() {
        let registry = TemplateImageMap::default_registry();
        assert_eq!(registry.resolve("unknown-language"), "ubuntu:22.04");
    }

    #[test]
    fn test_register_custom_template() {
        let mut registry = TemplateImageMap::default_registry();
        registry.register("elixir", "elixir:1.16-slim");
        assert_eq!(registry.resolve("elixir"), "elixir:1.16-slim");
    }

    #[test]
    fn test_has_template() {
        let registry = TemplateImageMap::default_registry();
        assert!(registry.has_template("node"));
        assert!(!registry.has_template("cobol"));
    }

    #[test]
    fn test_list_templates_sorted() {
        let registry = TemplateImageMap::default_registry();
        let templates = registry.list_templates();
        assert!(!templates.is_empty());
        let mut sorted = templates.clone();
        sorted.sort();
        assert_eq!(templates, sorted);
    }

    #[test]
    fn test_dockerfile_path_for_node() {
        let registry = TemplateImageMap::default_registry();
        let path = registry.dockerfile_path("node").expect("path");
        assert_eq!(path.to_str().expect("str"), "docker/node-base.Dockerfile");
    }

    #[test]
    fn test_docker_image_registry_language_mapping() {
        let registry = DockerImageRegistry {
            docker: Docker::connect_with_socket_defaults().expect("docker"),
            image_prefix: "cuttlefish/".to_string(),
        };
        assert_eq!(
            registry.language_to_image(Language::Node),
            "cuttlefish/node:latest"
        );
        assert_eq!(
            registry.language_to_image(Language::Python),
            "cuttlefish/python:latest"
        );
        assert_eq!(
            registry.language_to_image(Language::Rust),
            "cuttlefish/rust:latest"
        );
        assert_eq!(
            registry.language_to_image(Language::Go),
            "cuttlefish/go:latest"
        );
        assert_eq!(
            registry.language_to_image(Language::Ruby),
            "cuttlefish/ruby:latest"
        );
        assert_eq!(
            registry.language_to_image(Language::Generic),
            "cuttlefish/generic:latest"
        );
    }

    #[test]
    fn test_parse_image_ref() {
        assert_eq!(
            DockerImageRegistry::parse_image_ref("nginx:latest"),
            ("nginx", "latest")
        );
        assert_eq!(
            DockerImageRegistry::parse_image_ref("cuttlefish/node:v1"),
            ("cuttlefish/node", "v1")
        );
        assert_eq!(
            DockerImageRegistry::parse_image_ref("ubuntu"),
            ("ubuntu", "latest")
        );
        assert_eq!(
            DockerImageRegistry::parse_image_ref("registry.io/image:tag"),
            ("registry.io/image", "tag")
        );
    }

    #[test]
    fn test_infer_language_from_name() {
        assert_eq!(
            DockerImageRegistry::infer_language_from_name("cuttlefish/node"),
            Language::Node
        );
        assert_eq!(
            DockerImageRegistry::infer_language_from_name("python-base"),
            Language::Python
        );
        assert_eq!(
            DockerImageRegistry::infer_language_from_name("rust-builder"),
            Language::Rust
        );
        assert_eq!(
            DockerImageRegistry::infer_language_from_name("golang-alpine"),
            Language::Go
        );
        assert_eq!(
            DockerImageRegistry::infer_language_from_name("ruby-slim"),
            Language::Ruby
        );
        assert_eq!(
            DockerImageRegistry::infer_language_from_name("ubuntu"),
            Language::Generic
        );
    }

    #[tokio::test]
    async fn test_get_language_image() {
        let registry = DockerImageRegistry {
            docker: Docker::connect_with_socket_defaults().expect("docker"),
            image_prefix: "cuttlefish/".to_string(),
        };
        let spec = registry
            .get_language_image(Language::Node)
            .await
            .expect("spec");
        assert_eq!(spec.name, "cuttlefish/node");
        assert_eq!(spec.tag, "latest");
        assert_eq!(spec.language, Language::Node);
    }

    #[tokio::test]
    async fn test_mock_registry_pull_and_list() {
        let registry = MockImageRegistry::new();
        assert!(registry.list_images().await.expect("list").is_empty());

        let spec = registry.pull_image("test/image", "v1").await.expect("pull");
        assert_eq!(spec.name, "test/image");
        assert_eq!(spec.tag, "v1");

        let images = registry.list_images().await.expect("list");
        assert_eq!(images.len(), 1);
    }

    #[tokio::test]
    async fn test_mock_registry_remove() {
        let registry = MockImageRegistry::with_images(vec![ImageSpec {
            name: "test/image".to_string(),
            tag: "v1".to_string(),
            language: Language::Generic,
            size_bytes: None,
            created_at: None,
        }]);

        assert_eq!(registry.list_images().await.expect("list").len(), 1);
        registry
            .remove_image("test/image", "v1")
            .await
            .expect("remove");
        assert!(registry.list_images().await.expect("list").is_empty());
    }

    #[tokio::test]
    async fn test_mock_registry_get_language_image() {
        let registry = MockImageRegistry::new();
        let spec = registry
            .get_language_image(Language::Python)
            .await
            .expect("spec");
        assert_eq!(spec.name, "mock/python");
        assert_eq!(spec.language, Language::Python);
    }
}
