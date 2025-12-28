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
                    message: format!("Failed to read serial log: {}", e),
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

    /// Wait for VM to become healthy (with timeout)
    pub async fn wait_for_healthy<P: AsRef<Path>>(
        vm_name: &str,
        serial_log: P,
        timeout: std::time::Duration,
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
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                }
            }
        }
    }
}
