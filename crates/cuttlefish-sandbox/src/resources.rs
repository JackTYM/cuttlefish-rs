//! Resource usage monitoring and enforcement for sandboxes.

use bollard::container::Stats;
use bollard::Docker;
use cuttlefish_core::error::SandboxError;
use cuttlefish_core::traits::sandbox::{ResourceLimits, SandboxResult};
use futures::StreamExt;

/// Current resource usage of a container.
#[derive(Debug, Clone, Default)]
pub struct ResourceUsage {
    /// Memory usage in bytes.
    pub memory_bytes: u64,
    /// Memory limit in bytes.
    pub memory_limit: u64,
    /// CPU usage percentage (0-100 per core).
    pub cpu_percent: f64,
    /// Number of running processes.
    pub pids_current: u64,
    /// Network bytes received.
    pub network_rx_bytes: u64,
    /// Network bytes transmitted.
    pub network_tx_bytes: u64,
}

/// Monitors and enforces resource limits on containers.
pub struct ResourceEnforcer {
    docker: Docker,
}

impl ResourceEnforcer {
    /// Create a new `ResourceEnforcer` with default Docker connection.
    pub fn new() -> Result<Self, SandboxError> {
        let docker = Docker::connect_with_socket_defaults()
            .map_err(|e| SandboxError::Other(format!("Docker connect failed: {e}")))?;
        Ok(Self { docker })
    }

    /// Create a `ResourceEnforcer` with an existing Docker client.
    pub fn with_docker(docker: Docker) -> Self {
        Self { docker }
    }

    /// Get current resource usage for a container.
    pub async fn get_usage(&self, container_id: &str) -> SandboxResult<ResourceUsage> {
        use bollard::container::StatsOptions;

        let options = StatsOptions {
            stream: false,
            one_shot: true,
        };

        let mut stats_stream = self.docker.stats(container_id, Some(options));

        if let Some(result) = stats_stream.next().await {
            let stats = result.map_err(|e| {
                SandboxError::Other(format!("Failed to get stats for {container_id}: {e}"))
            })?;

            Ok(self.parse_stats(&stats))
        } else {
            Err(SandboxError::ContainerNotFound {
                id: container_id.to_string(),
            })
        }
    }

    /// Check if container is within its resource limits.
    ///
    /// Returns a list of violation descriptions, empty if within limits.
    pub fn check_limits(&self, usage: &ResourceUsage, limits: &ResourceLimits) -> Vec<String> {
        let mut violations = Vec::new();

        if let Some(mem_limit) = limits.memory_bytes
            && usage.memory_bytes > mem_limit
        {
            violations.push(format!(
                "Memory: {}MB / {}MB",
                usage.memory_bytes / (1024 * 1024),
                mem_limit / (1024 * 1024)
            ));
        }

        if let Some(pids_limit) = limits.pids_limit {
            #[allow(clippy::cast_sign_loss)]
            if usage.pids_current > pids_limit as u64 {
                violations.push(format!("PIDs: {} / {}", usage.pids_current, pids_limit));
            }
        }

        violations
    }

    fn parse_stats(&self, stats: &Stats) -> ResourceUsage {
        let memory_bytes = stats.memory_stats.usage.unwrap_or(0);
        let memory_limit = stats.memory_stats.limit.unwrap_or(0);

        let cpu_delta = stats
            .cpu_stats
            .cpu_usage
            .total_usage
            .saturating_sub(stats.precpu_stats.cpu_usage.total_usage);
        let system_delta = stats
            .cpu_stats
            .system_cpu_usage
            .unwrap_or(0)
            .saturating_sub(stats.precpu_stats.system_cpu_usage.unwrap_or(0));
        let num_cpus = stats.cpu_stats.online_cpus.unwrap_or(1) as f64;

        let cpu_percent = if system_delta > 0 {
            (cpu_delta as f64 / system_delta as f64) * num_cpus * 100.0
        } else {
            0.0
        };

        let pids_current = stats.pids_stats.current.unwrap_or(0);

        let (rx, tx) = stats
            .networks
            .as_ref()
            .map(|nets| {
                nets.values()
                    .fold((0u64, 0u64), |(rx, tx), net| (rx + net.rx_bytes, tx + net.tx_bytes))
            })
            .unwrap_or((0, 0));

        ResourceUsage {
            memory_bytes,
            memory_limit,
            cpu_percent,
            pids_current,
            network_rx_bytes: rx,
            network_tx_bytes: tx,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_resource_limits_light() {
        let limits = ResourceLimits::light();
        assert_eq!(limits.memory_bytes, Some(256 * 1024 * 1024));
        assert_eq!(limits.timeout, Some(Duration::from_secs(30)));
    }

    #[test]
    fn test_resource_limits_standard() {
        let limits = ResourceLimits::standard();
        assert_eq!(limits.memory_bytes, Some(512 * 1024 * 1024));
        assert_eq!(limits.timeout, Some(Duration::from_secs(60)));
    }

    #[test]
    fn test_resource_limits_heavy() {
        let limits = ResourceLimits::heavy();
        assert_eq!(limits.memory_bytes, Some(2 * 1024 * 1024 * 1024));
        assert_eq!(limits.timeout, Some(Duration::from_secs(300)));
    }

    #[test]
    fn test_resource_limits_builder() {
        let limits = ResourceLimits::builder()
            .memory_mb(1024)
            .cpu(1.5)
            .timeout_secs(120)
            .no_network()
            .build();

        assert_eq!(limits.memory_bytes, Some(1024 * 1024 * 1024));
        assert_eq!(limits.cpu_quota, Some(150_000));
        assert_eq!(limits.timeout, Some(Duration::from_secs(120)));
        assert!(limits.network_disabled);
    }

    #[test]
    fn test_check_limits_no_violations() {
        let usage = ResourceUsage {
            memory_bytes: 100 * 1024 * 1024, // 100MB
            pids_current: 10,
            ..Default::default()
        };
        let limits = ResourceLimits::standard();

        let docker =
            Docker::connect_with_socket_defaults().expect("Docker connection for test");
        let enforcer = ResourceEnforcer { docker };
        let violations = enforcer.check_limits(&usage, &limits);
        assert!(violations.is_empty());
    }

    #[test]
    fn test_check_limits_memory_exceeded() {
        let usage = ResourceUsage {
            memory_bytes: 600 * 1024 * 1024, // 600MB
            pids_current: 10,
            ..Default::default()
        };
        let limits = ResourceLimits::standard(); // 512MB limit

        let docker =
            Docker::connect_with_socket_defaults().expect("Docker connection for test");
        let enforcer = ResourceEnforcer { docker };
        let violations = enforcer.check_limits(&usage, &limits);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].contains("Memory"));
    }
}
