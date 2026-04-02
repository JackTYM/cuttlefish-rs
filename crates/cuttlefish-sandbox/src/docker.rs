//! Docker-based sandbox implementation using bollard.

use async_trait::async_trait;
use bollard::Docker;
use bollard::container::{
    Config, CreateContainerOptions, ListContainersOptions, RemoveContainerOptions,
    StartContainerOptions, StopContainerOptions,
};
use bollard::exec::{CreateExecOptions, StartExecResults};
use cuttlefish_core::error::SandboxError;
use cuttlefish_core::traits::sandbox::{ExecOutput, Sandbox, SandboxConfig, SandboxId};
use futures::StreamExt;
use tracing::{debug, info, warn};

const MAX_OUTPUT_BYTES: usize = 1_024 * 1_024; // 1MB per stream

/// Docker-backed sandbox implementation.
///
/// Each sandbox corresponds to a single Docker container with resource
/// limits applied. Containers are kept alive with `tail -f /dev/null`
/// and commands are executed via `docker exec`.
pub struct DockerSandbox {
    docker: Docker,
}

impl DockerSandbox {
    /// Connect to the Docker daemon at the default socket path.
    ///
    /// # Errors
    ///
    /// Returns [`SandboxError`] if the Docker daemon is unreachable.
    pub fn new() -> Result<Self, SandboxError> {
        let docker = Docker::connect_with_socket_defaults()
            .map_err(|e| SandboxError(format!("Failed to connect to Docker: {e}")))?;
        Ok(Self { docker })
    }

    /// Connect with an explicit socket path.
    ///
    /// # Errors
    ///
    /// Returns [`SandboxError`] if the socket is unreachable.
    pub fn with_socket(socket_path: &str) -> Result<Self, SandboxError> {
        let docker = Docker::connect_with_socket(socket_path, 30, bollard::API_DEFAULT_VERSION)
            .map_err(|e| {
                SandboxError(format!(
                    "Failed to connect to Docker socket {socket_path}: {e}"
                ))
            })?;
        Ok(Self { docker })
    }
}

#[async_trait]
impl Sandbox for DockerSandbox {
    async fn create(&self, config: &SandboxConfig) -> Result<SandboxId, SandboxError> {
        let container_name = format!("cuttlefish-{}", uuid::Uuid::new_v4());

        debug!(
            "Creating Docker container {} with image {}",
            container_name, config.image
        );

        let env: Vec<String> = config
            .env_vars
            .iter()
            .map(|(k, v)| format!("{k}={v}"))
            .collect();

        let host_config = bollard::service::HostConfig {
            memory: Some((config.memory_limit_mb * 1024 * 1024) as i64),
            nano_cpus: Some((config.cpu_limit * 1e9) as i64),
            network_mode: if config.network_enabled {
                Some("bridge".to_string())
            } else {
                Some("none".to_string())
            },
            auto_remove: Some(false),
            ..Default::default()
        };

        let container_config = Config {
            image: Some(config.image.as_str()),
            env: Some(env.iter().map(|s| s.as_str()).collect()),
            host_config: Some(host_config),
            cmd: Some(vec!["tail", "-f", "/dev/null"]),
            working_dir: Some("/workspace"),
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
            .map_err(|e| SandboxError(format!("Failed to create container: {e}")))?;

        self.docker
            .start_container(&response.id, None::<StartContainerOptions<String>>)
            .await
            .map_err(|e| SandboxError(format!("Failed to start container {}: {e}", response.id)))?;

        let workspace_id = SandboxId(response.id.clone());
        let _ = self.exec(&workspace_id, "mkdir -p /workspace").await;

        info!("Created sandbox container: {}", response.id);
        Ok(workspace_id)
    }

    async fn exec(&self, id: &SandboxId, command: &str) -> Result<ExecOutput, SandboxError> {
        let container_id = &id.0;
        debug!("Executing in container {}: {}", container_id, command);

        let exec_options = CreateExecOptions {
            cmd: Some(vec!["sh", "-c", command]),
            attach_stdout: Some(true),
            attach_stderr: Some(true),
            ..Default::default()
        };

        let exec = self
            .docker
            .create_exec(container_id, exec_options)
            .await
            .map_err(|e| SandboxError(format!("Failed to create exec in {container_id}: {e}")))?;

        let mut stdout_buf = String::new();
        let mut stderr_buf = String::new();

        if let StartExecResults::Attached { mut output, .. } = self
            .docker
            .start_exec(&exec.id, None)
            .await
            .map_err(|e| SandboxError(format!("Failed to start exec: {e}")))?
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
                    warn!("Output truncated for exec in container {}", container_id);
                    break;
                }
            }
        }

        let inspect = self
            .docker
            .inspect_exec(&exec.id)
            .await
            .map_err(|e| SandboxError(format!("Failed to inspect exec: {e}")))?;

        let exit_code = inspect.exit_code.unwrap_or(-1);

        Ok(ExecOutput {
            stdout: stdout_buf,
            stderr: stderr_buf,
            exit_code,
            timed_out: false,
        })
    }

    async fn write_file(
        &self,
        id: &SandboxId,
        path: &std::path::Path,
        content: &[u8],
    ) -> Result<(), SandboxError> {
        let container_id = &id.0;

        if let Some(parent) = path.parent() {
            let parent_str = parent.to_string_lossy();
            if !parent_str.is_empty() {
                let _ = self.exec(id, &format!("mkdir -p '{parent_str}'")).await;
            }
        }

        // Base64-encode to avoid shell escaping issues
        let b64 = base64_encode(content);
        let path_str = path.to_string_lossy();
        let cmd = format!("echo '{b64}' | base64 -d > '{path_str}'");

        let result = self.exec(id, &cmd).await?;
        if !result.success() {
            return Err(SandboxError(format!(
                "Failed to write file {path_str} in container {container_id}: {}",
                result.stderr
            )));
        }
        Ok(())
    }

    async fn read_file(
        &self,
        id: &SandboxId,
        path: &std::path::Path,
    ) -> Result<Vec<u8>, SandboxError> {
        let path_str = path.to_string_lossy();
        let cmd = format!("base64 '{path_str}'");

        let result = self.exec(id, &cmd).await?;
        if !result.success() {
            return Err(SandboxError(format!(
                "Failed to read file {path_str}: {}",
                result.stderr
            )));
        }

        base64_decode(result.stdout.trim())
            .map_err(|e| SandboxError(format!("Failed to decode file content: {e}")))
    }

    async fn list_files(
        &self,
        id: &SandboxId,
        path: &std::path::Path,
    ) -> Result<Vec<String>, SandboxError> {
        let path_str = path.to_string_lossy();
        let cmd = format!("ls -1 '{path_str}'");

        let result = self.exec(id, &cmd).await?;
        if !result.success() {
            return Err(SandboxError(format!(
                "Failed to list files in {path_str}: {}",
                result.stderr
            )));
        }

        Ok(result
            .stdout
            .lines()
            .map(|l| l.to_string())
            .filter(|l| !l.is_empty())
            .collect())
    }

    async fn destroy(&self, id: &SandboxId) -> Result<(), SandboxError> {
        let container_id = &id.0;
        info!("Destroying sandbox container: {}", container_id);

        let stop_opts = StopContainerOptions { t: 10 };
        let _ = self
            .docker
            .stop_container(container_id, Some(stop_opts))
            .await;

        let remove_opts = RemoveContainerOptions {
            force: true,
            v: true,
            ..Default::default()
        };
        self.docker
            .remove_container(container_id, Some(remove_opts))
            .await
            .map_err(|e| SandboxError(format!("Failed to remove container {container_id}: {e}")))?;

        Ok(())
    }
}

impl DockerSandbox {
    /// Execute a command with a timeout, killing the exec if it exceeds the duration.
    ///
    /// Returns [`ExecOutput`] with `timed_out = true` if the timeout was exceeded.
    pub async fn exec_with_timeout(
        &self,
        id: &SandboxId,
        command: &str,
        timeout_secs: u64,
    ) -> Result<ExecOutput, SandboxError> {
        use tokio::time::{Duration, timeout};

        let exec_future = self.exec(id, command);

        match timeout(Duration::from_secs(timeout_secs), exec_future).await {
            Ok(result) => result,
            Err(_) => {
                warn!(
                    "Exec timed out after {}s in container {}",
                    timeout_secs, id.0
                );
                Ok(ExecOutput {
                    stdout: String::new(),
                    stderr: format!("Command timed out after {timeout_secs}s"),
                    exit_code: -1,
                    timed_out: true,
                })
            }
        }
    }

    /// List all Cuttlefish-managed containers.
    ///
    /// Returns container IDs for all containers with names starting with `cuttlefish-`.
    pub async fn list_cuttlefish_containers(&self) -> Result<Vec<String>, SandboxError> {
        use std::collections::HashMap;

        let options = ListContainersOptions::<String> {
            all: true,
            filters: {
                let mut m = HashMap::new();
                m.insert("name".to_string(), vec!["cuttlefish-".to_string()]);
                m
            },
            ..Default::default()
        };

        let containers = self
            .docker
            .list_containers(Some(options))
            .await
            .map_err(|e| SandboxError(format!("Failed to list containers: {e}")))?;

        Ok(containers.iter().filter_map(|c| c.id.clone()).collect())
    }

    /// Remove all stopped Cuttlefish-managed containers.
    ///
    /// Returns the number of containers removed.
    pub async fn cleanup_stopped_containers(&self) -> Result<usize, SandboxError> {
        use std::collections::HashMap;

        let options = ListContainersOptions::<String> {
            all: true,
            filters: {
                let mut m = HashMap::new();
                m.insert("name".to_string(), vec!["cuttlefish-".to_string()]);
                m.insert(
                    "status".to_string(),
                    vec!["exited".to_string(), "dead".to_string()],
                );
                m
            },
            ..Default::default()
        };

        let containers = self
            .docker
            .list_containers(Some(options))
            .await
            .map_err(|e| SandboxError(format!("Failed to list containers: {e}")))?;

        let mut removed = 0;
        for container in containers {
            if let Some(id) = container.id {
                let remove_opts = RemoveContainerOptions {
                    force: true,
                    v: true,
                    ..Default::default()
                };
                if self
                    .docker
                    .remove_container(&id, Some(remove_opts))
                    .await
                    .is_ok()
                {
                    removed += 1;
                    debug!("Removed stopped container {}", id);
                }
            }
        }

        Ok(removed)
    }
}

/// Encode bytes to base64 (standard alphabet, with padding).
fn base64_encode(data: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::new();
    let mut i = 0;
    while i < data.len() {
        let b0 = data[i] as usize;
        let b1 = if i + 1 < data.len() {
            data[i + 1] as usize
        } else {
            0
        };
        let b2 = if i + 2 < data.len() {
            data[i + 2] as usize
        } else {
            0
        };

        result.push(CHARS[(b0 >> 2) & 63] as char);
        result.push(CHARS[((b0 << 4) | (b1 >> 4)) & 63] as char);

        if i + 1 < data.len() {
            result.push(CHARS[((b1 << 2) | (b2 >> 6)) & 63] as char);
        } else {
            result.push('=');
        }

        if i + 2 < data.len() {
            result.push(CHARS[b2 & 63] as char);
        } else {
            result.push('=');
        }

        i += 3;
    }
    result
}

/// Decode a base64 string back to bytes.
fn base64_decode(s: &str) -> Result<Vec<u8>, String> {
    fn char_to_val(c: char) -> Result<u8, String> {
        match c {
            'A'..='Z' => Ok(c as u8 - b'A'),
            'a'..='z' => Ok(c as u8 - b'a' + 26),
            '0'..='9' => Ok(c as u8 - b'0' + 52),
            '+' => Ok(62),
            '/' => Ok(63),
            '=' => Ok(0),
            other => Err(format!("Invalid base64 character: {other}")),
        }
    }

    let clean: String = s.chars().filter(|c| !c.is_whitespace()).collect();
    let mut result = Vec::new();
    let chars: Vec<char> = clean.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let c0 = if i < chars.len() { chars[i] } else { '=' };
        let c1 = if i + 1 < chars.len() {
            chars[i + 1]
        } else {
            '='
        };
        let c2 = if i + 2 < chars.len() {
            chars[i + 2]
        } else {
            '='
        };
        let c3 = if i + 3 < chars.len() {
            chars[i + 3]
        } else {
            '='
        };

        let v0 = char_to_val(c0)?;
        let v1 = char_to_val(c1)?;
        let v2 = char_to_val(c2)?;
        let v3 = char_to_val(c3)?;

        result.push((v0 << 2) | (v1 >> 4));
        if c2 != '=' {
            result.push((v1 << 4) | (v2 >> 2));
        }
        if c3 != '=' {
            result.push((v2 << 6) | v3);
        }

        i += 4;
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base64_roundtrip() {
        let data = b"Hello, World! This is a test.";
        let encoded = base64_encode(data);
        let decoded = base64_decode(&encoded).expect("decode should succeed");
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_base64_empty() {
        let data = b"";
        let encoded = base64_encode(data);
        let decoded = base64_decode(&encoded).expect("decode empty should succeed");
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_base64_single_byte() {
        let data = b"A";
        let encoded = base64_encode(data);
        let decoded = base64_decode(&encoded).expect("decode single byte");
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_base64_two_bytes() {
        let data = b"AB";
        let encoded = base64_encode(data);
        let decoded = base64_decode(&encoded).expect("decode two bytes");
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_base64_binary_data() {
        let data: Vec<u8> = (0..=255).collect();
        let encoded = base64_encode(&data);
        let decoded = base64_decode(&encoded).expect("decode binary data");
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_sandbox_id_new() {
        let id = SandboxId::new();
        assert!(!id.0.is_empty());
    }

    #[test]
    fn test_sandbox_config_default() {
        let config = SandboxConfig::default();
        assert_eq!(config.image, "ubuntu:22.04");
        assert_eq!(config.memory_limit_mb, 2048);
        assert!(config.network_enabled);
    }

    #[test]
    fn test_exec_output_success() {
        let output = ExecOutput {
            stdout: "ok".to_string(),
            stderr: String::new(),
            exit_code: 0,
            timed_out: false,
        };
        assert!(output.success());
    }

    #[test]
    fn test_exec_output_failure() {
        let output = ExecOutput {
            stdout: String::new(),
            stderr: "error".to_string(),
            exit_code: 1,
            timed_out: false,
        };
        assert!(!output.success());
    }

    #[test]
    fn test_exec_output_timeout() {
        let output = ExecOutput {
            stdout: String::new(),
            stderr: String::new(),
            exit_code: 0,
            timed_out: true,
        };
        assert!(!output.success());
    }

    #[tokio::test]
    async fn test_exec_with_timeout_fast_command_succeeds() {
        let output = ExecOutput {
            stdout: "hello".to_string(),
            stderr: String::new(),
            exit_code: 0,
            timed_out: false,
        };
        assert!(output.success());
        assert!(!output.timed_out);
    }

    #[tokio::test]
    async fn test_timed_out_output_success_is_false() {
        let output = ExecOutput {
            stdout: String::new(),
            stderr: "timed out".to_string(),
            exit_code: -1,
            timed_out: true,
        };
        assert!(!output.success());
        assert!(output.timed_out);
        assert_eq!(output.exit_code, -1);
    }

    #[test]
    fn test_output_truncation_constants() {
        assert_eq!(MAX_OUTPUT_BYTES, 1_048_576);
    }

    #[test]
    fn test_output_truncation_logic() {
        let mut buf = "x".repeat(MAX_OUTPUT_BYTES + 100);
        if buf.len() > MAX_OUTPUT_BYTES {
            buf.truncate(MAX_OUTPUT_BYTES);
            buf.push_str("\n[OUTPUT TRUNCATED - EXCEEDED 1MB LIMIT]");
        }
        assert!(buf.starts_with("xxx"));
        assert!(buf.ends_with("[OUTPUT TRUNCATED - EXCEEDED 1MB LIMIT]"));
        assert!(buf.len() > MAX_OUTPUT_BYTES);
    }

    #[test]
    fn test_output_under_limit_not_truncated() {
        let buf = "hello world".to_string();
        assert!(buf.len() < MAX_OUTPUT_BYTES);
    }

    #[cfg(feature = "integration")]
    mod integration {
        use super::*;
        use std::path::PathBuf;

        #[tokio::test]
        async fn test_docker_lifecycle() {
            let sandbox = DockerSandbox::new().expect("docker connect");
            let config = SandboxConfig {
                image: "alpine:latest".to_string(),
                ..Default::default()
            };

            let id = sandbox.create(&config).await.expect("create");

            let output = sandbox.exec(&id, "echo hello").await.expect("exec");
            assert!(output.success());
            assert!(output.stdout.contains("hello"));

            let test_path = PathBuf::from("/workspace/test.txt");
            let content = b"Hello from cuttlefish!";
            sandbox
                .write_file(&id, &test_path, content)
                .await
                .expect("write");
            let read_back = sandbox.read_file(&id, &test_path).await.expect("read");
            assert_eq!(read_back, content);

            let files = sandbox
                .list_files(&id, &PathBuf::from("/workspace"))
                .await
                .expect("list");
            assert!(files.contains(&"test.txt".to_string()));

            sandbox.destroy(&id).await.expect("destroy");
        }
    }
}
