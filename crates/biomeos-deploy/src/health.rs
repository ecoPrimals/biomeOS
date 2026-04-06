// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Health checking for VMs

use crate::error::{DeployError, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::SystemTime;
use tracing::{info, warn};

/// Health status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    /// VM is healthy
    Healthy,
    /// VM is degraded
    Degraded,
    /// VM is unhealthy
    Unhealthy,
    /// VM status is unknown
    Unknown,
}

/// VM health information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VmHealth {
    /// VM name
    pub vm_name: String,

    /// Health status
    pub status: HealthStatus,

    /// Last boot message timestamp
    pub last_message_time: Option<SystemTime>,

    /// Boot completed successfully
    pub boot_completed: bool,

    /// Error message (if any)
    pub error: Option<String>,
}

/// Health check implementation
pub struct HealthCheck;

impl HealthCheck {
    /// Check VM health by analyzing serial log
    pub async fn check_vm<P: AsRef<Path>>(vm_name: &str, serial_log: P) -> Result<VmHealth> {
        let log_path = serial_log.as_ref();

        if !log_path.exists() {
            return Ok(VmHealth {
                vm_name: vm_name.to_string(),
                status: HealthStatus::Unknown,
                last_message_time: None,
                boot_completed: false,
                error: Some("Serial log not found".to_string()),
            });
        }

        let contents =
            tokio::fs::read_to_string(log_path)
                .await
                .map_err(|e| DeployError::HealthCheck {
                    vm_name: vm_name.to_string(),
                    message: format!("Failed to read serial log: {e}"),
                })?;

        // Check for boot completion indicators
        let boot_completed = contents.contains("BiomeOS Init Complete")
            || contents.contains("Spawning shell")
            || contents.contains("BusyBox");

        // Check for error indicators
        let has_errors = contents.contains("Kernel panic")
            || contents.contains("FATAL")
            || contents.contains("Failed to mount");

        // Check for recent activity (last message timestamp)
        let last_message_time = if let Ok(metadata) = tokio::fs::metadata(log_path).await {
            metadata.modified().ok()
        } else {
            None
        };

        // Determine status
        let status = if has_errors {
            HealthStatus::Unhealthy
        } else if boot_completed {
            HealthStatus::Healthy
        } else if last_message_time.is_some() {
            HealthStatus::Degraded
        } else {
            HealthStatus::Unknown
        };

        let error = if has_errors {
            Some("Errors detected in serial log".to_string())
        } else {
            None
        };

        Ok(VmHealth {
            vm_name: vm_name.to_string(),
            status,
            last_message_time,
            boot_completed,
            error,
        })
    }

    /// Default poll interval between health checks (1s).
    pub const DEFAULT_POLL_INTERVAL: std::time::Duration = std::time::Duration::from_secs(1);

    /// Wait for VM to become healthy (with timeout)
    pub async fn wait_for_healthy<P: AsRef<Path>>(
        vm_name: &str,
        serial_log: P,
        timeout: std::time::Duration,
    ) -> Result<VmHealth> {
        Self::wait_for_healthy_with_poll_interval(
            vm_name,
            serial_log,
            timeout,
            Self::DEFAULT_POLL_INTERVAL,
        )
        .await
    }

    /// Wait for VM to become healthy (with timeout and configurable poll interval)
    pub async fn wait_for_healthy_with_poll_interval<P: AsRef<Path>>(
        vm_name: &str,
        serial_log: P,
        timeout: std::time::Duration,
        poll_interval: std::time::Duration,
    ) -> Result<VmHealth> {
        let start = std::time::Instant::now();

        loop {
            let health = Self::check_vm(vm_name, serial_log.as_ref()).await?;

            match health.status {
                HealthStatus::Healthy => {
                    info!("✅ VM {} is healthy", vm_name);
                    return Ok(health);
                }
                HealthStatus::Unhealthy => {
                    return Err(DeployError::HealthCheck {
                        vm_name: vm_name.to_string(),
                        message: health.error.unwrap_or_else(|| "Unknown error".to_string()),
                    });
                }
                _ => {
                    if start.elapsed() > timeout {
                        warn!("⚠️  VM {} health check timed out", vm_name);
                        return Err(DeployError::HealthCheck {
                            vm_name: vm_name.to_string(),
                            message: "Health check timed out".to_string(),
                        });
                    }
                    tokio::time::sleep(poll_interval).await;
                }
            }
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
mod tests {
    use super::*;
    use std::time::UNIX_EPOCH;

    #[test]
    fn test_health_status_variants() {
        assert_eq!(HealthStatus::Healthy as i32, HealthStatus::Healthy as i32);
        assert_ne!(HealthStatus::Healthy, HealthStatus::Unhealthy);
        assert_ne!(HealthStatus::Degraded, HealthStatus::Unknown);
    }

    #[test]
    fn test_health_status_serialization() {
        let status = HealthStatus::Healthy;
        let json = serde_json::to_string(&status).expect("serialization should succeed");
        assert!(json.contains("Healthy"));
        let deserialized: HealthStatus =
            serde_json::from_str(&json).expect("deserialization should succeed");
        assert_eq!(deserialized, status);
    }

    #[test]
    fn test_vm_health_construction() {
        let health = VmHealth {
            vm_name: "test-vm".to_string(),
            status: HealthStatus::Healthy,
            last_message_time: Some(UNIX_EPOCH),
            boot_completed: true,
            error: None,
        };
        assert_eq!(health.vm_name, "test-vm");
        assert_eq!(health.status, HealthStatus::Healthy);
        assert!(health.boot_completed);
    }

    #[test]
    fn test_vm_health_serialization() {
        let health = VmHealth {
            vm_name: "vm1".to_string(),
            status: HealthStatus::Unhealthy,
            last_message_time: None,
            boot_completed: false,
            error: Some("Kernel panic".to_string()),
        };
        let json = serde_json::to_string(&health).expect("serialization should succeed");
        assert!(json.contains("\"vm_name\":\"vm1\""));
        assert!(json.contains("Unhealthy"));
        assert!(json.contains("Kernel panic"));
    }

    #[tokio::test]
    async fn test_check_vm_nonexistent_log() {
        let result = HealthCheck::check_vm("test-vm", "/nonexistent/path/serial.log").await;
        let health = result.expect("check_vm returns Ok even for missing log");
        assert_eq!(health.vm_name, "test-vm");
        assert_eq!(health.status, HealthStatus::Unknown);
        assert!(!health.boot_completed);
        assert!(health.error.is_some());
        assert!(
            health
                .error
                .as_ref()
                .expect("error present")
                .contains("Serial log not found")
        );
    }

    #[tokio::test]
    async fn test_check_vm_boot_completed() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let log_path = temp_dir.path().join("serial.log");
        std::fs::write(&log_path, "BiomeOS Init Complete\n").expect("write log");
        let result = HealthCheck::check_vm("boot-vm", &log_path).await;
        let health = result.expect("check_vm should succeed");
        assert_eq!(health.status, HealthStatus::Healthy);
        assert!(health.boot_completed);
    }

    #[tokio::test]
    async fn test_check_vm_has_errors() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let log_path = temp_dir.path().join("serial.log");
        std::fs::write(&log_path, "Kernel panic - not syncing\n").expect("write log");
        let result = HealthCheck::check_vm("panic-vm", &log_path).await;
        let health = result.expect("check_vm should succeed");
        assert_eq!(health.status, HealthStatus::Unhealthy);
        assert!(health.error.is_some());
    }

    #[tokio::test]
    async fn test_check_vm_degraded() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let log_path = temp_dir.path().join("serial.log");
        std::fs::write(&log_path, "Booting...\n").expect("write log");
        let result = HealthCheck::check_vm("degraded-vm", &log_path).await;
        let health = result.expect("check_vm should succeed");
        assert_eq!(health.status, HealthStatus::Degraded);
        assert!(!health.boot_completed);
    }

    #[tokio::test]
    async fn test_check_vm_spawning_shell() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let log_path = temp_dir.path().join("serial.log");
        std::fs::write(&log_path, "Spawning shell\n").expect("write log");
        let result = HealthCheck::check_vm("shell-vm", &log_path).await;
        let health = result.expect("check_vm should succeed");
        assert_eq!(health.status, HealthStatus::Healthy);
        assert!(health.boot_completed);
    }

    #[tokio::test]
    async fn test_check_vm_busybox() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let log_path = temp_dir.path().join("serial.log");
        std::fs::write(&log_path, "BusyBox v1.0\n").expect("write log");
        let result = HealthCheck::check_vm("busybox-vm", &log_path).await;
        let health = result.expect("check_vm should succeed");
        assert_eq!(health.status, HealthStatus::Healthy);
        assert!(health.boot_completed);
    }

    #[tokio::test]
    async fn test_check_vm_fatal_error() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let log_path = temp_dir.path().join("serial.log");
        std::fs::write(&log_path, "FATAL: Something went wrong\n").expect("write log");
        let result = HealthCheck::check_vm("fatal-vm", &log_path).await;
        let health = result.expect("check_vm should succeed");
        assert_eq!(health.status, HealthStatus::Unhealthy);
    }

    #[tokio::test]
    async fn test_wait_for_healthy_success() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let log_path = temp_dir.path().join("serial.log");
        std::fs::write(&log_path, "BiomeOS Init Complete\n").expect("write log");
        let result = HealthCheck::wait_for_healthy(
            "healthy-vm",
            &log_path,
            std::time::Duration::from_secs(5),
        )
        .await;
        let health = result.expect("wait_for_healthy should succeed");
        assert_eq!(health.status, HealthStatus::Healthy);
    }

    #[tokio::test]
    async fn test_wait_for_healthy_unhealthy_fails() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let log_path = temp_dir.path().join("serial.log");
        std::fs::write(&log_path, "Kernel panic\n").expect("write log");
        let result =
            HealthCheck::wait_for_healthy("panic-vm", &log_path, std::time::Duration::from_secs(2))
                .await;
        let err = result.expect_err("unhealthy VM should fail");
        assert!(err.to_string().contains("panic"));
    }

    #[tokio::test]
    async fn test_wait_for_healthy_timeout() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let log_path = temp_dir.path().join("serial.log");
        std::fs::write(&log_path, "Booting...\n").expect("write log");
        let result = HealthCheck::wait_for_healthy_with_poll_interval(
            "slow-vm",
            &log_path,
            std::time::Duration::from_millis(100),
            std::time::Duration::ZERO,
        )
        .await;
        let err = result.expect_err("degraded VM should timeout");
        assert!(err.to_string().contains("timed out"));
    }

    #[test]
    fn test_health_status_all_variants() {
        let _ = format!("{:?}", HealthStatus::Degraded);
        let _ = format!("{:?}", HealthStatus::Unknown);
    }

    #[test]
    fn test_vm_health_with_error() {
        let health = VmHealth {
            vm_name: "err-vm".to_string(),
            status: HealthStatus::Unhealthy,
            last_message_time: None,
            boot_completed: false,
            error: Some("Kernel panic".to_string()),
        };
        let json = serde_json::to_string(&health).expect("serialize");
        assert!(json.contains("Kernel panic"));
    }
}
