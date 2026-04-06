// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! `BiomeOS` System Information and Monitoring
//!
//! This crate provides comprehensive system information gathering, health monitoring,
//! and resource metrics for the `BiomeOS` ecosystem.

#![warn(missing_docs)]
#![forbid(unsafe_code)]

mod cpu;
mod disk;
mod memory;
mod network;
mod uptime;

pub use cpu::{CpuInfo, LoadAverage};
pub use disk::DiskInfo;
pub use memory::MemoryInfo;
pub use network::{NetworkInterface, NetworkInterfaceStatus, NetworkInterfaceType};

use std::collections::HashMap;
use std::fs;

use biomeos_types::{
    AvailabilityMetrics, BiomeResult, ComponentHealth, Health, HealthReport, HealthSubject,
    HealthSubjectType, ResourceMetrics, health::HealthMetrics,
};

/// System information inspector
pub struct SystemInspector;

impl SystemInspector {
    /// Get comprehensive system information
    pub async fn get_system_info() -> BiomeResult<SystemInfo> {
        let hostname = Self::get_hostname()?;
        let kernel_info = Self::get_kernel_info()?;
        let cpu_info = cpu::get_cpu_info()?;
        let memory_info = memory::get_memory_info()?;
        let disk_info = disk::get_disk_info()?;
        let network_info = network::get_network_info()?;
        let uptime = uptime::get_uptime()?;
        let load_average = cpu::get_load_average()?;

        Ok(SystemInfo {
            hostname,
            kernel_info,
            cpu_info,
            memory_info,
            disk_info,
            network_info,
            uptime,
            load_average,
            timestamp: chrono::Utc::now(),
        })
    }

    /// Get current resource usage metrics
    pub async fn get_resource_usage() -> BiomeResult<ResourceMetrics> {
        let cpu_usage = cpu::get_cpu_usage().await?;
        let memory_usage = memory::get_memory_usage()?;
        let disk_usage = disk::get_disk_usage()?;
        let network_io = network::get_network_io().await?;

        Ok(ResourceMetrics {
            cpu_usage: Some(cpu_usage),
            memory_usage: Some(memory_usage),
            disk_usage: Some(disk_usage),
            network_io: Some(network_io),
        })
    }

    /// Get system health report
    pub async fn get_system_health() -> BiomeResult<HealthReport> {
        let resource_metrics = Self::get_resource_usage().await?;
        let system_info = Self::get_system_info().await?;

        // Determine overall health based on resource usage
        let health = Self::determine_health_from_metrics(&resource_metrics);

        // Create health subject
        let subject = HealthSubject {
            id: system_info.hostname.clone(),
            subject_type: HealthSubjectType::System,
            name: "biomeOS System".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        };

        // Build component health map
        let mut components = HashMap::new();

        // Add CPU component
        let cpu_health = Self::cpu_component_health(resource_metrics.cpu_usage);

        components.insert(
            "cpu".to_string(),
            ComponentHealth {
                name: "CPU".to_string(),
                health: cpu_health,
                metrics: {
                    let mut metrics = HashMap::new();
                    if let Some(usage) = resource_metrics.cpu_usage {
                        metrics.insert(
                            "usage_percent".to_string(),
                            serde_json::json!(usage * 100.0),
                        );
                    }
                    metrics
                },
                last_check: chrono::Utc::now(),
            },
        );

        // Add Memory component
        let memory_health = Self::memory_component_health(resource_metrics.memory_usage);

        components.insert(
            "memory".to_string(),
            ComponentHealth {
                name: "Memory".to_string(),
                health: memory_health,
                metrics: {
                    let mut metrics = HashMap::new();
                    if let Some(usage) = resource_metrics.memory_usage {
                        metrics.insert(
                            "usage_percent".to_string(),
                            serde_json::json!(usage * 100.0),
                        );
                    }
                    metrics
                },
                last_check: chrono::Utc::now(),
            },
        );

        // Build health metrics
        let health_metrics = HealthMetrics {
            response_time: None,
            resources: Some(resource_metrics),
            errors: None,
            availability: Some(AvailabilityMetrics {
                uptime_percentage: Self::calculate_uptime_percentage(&system_info),
                uptime_seconds: 0,
                downtime_seconds: 0,
                outage_count: 0,
                mttr_seconds: None,
            }),
            custom: HashMap::new(),
        };

        Ok(HealthReport {
            id: uuid::Uuid::new_v4(),
            subject,
            health,
            components,
            metrics: health_metrics,
            history: vec![], // Would be populated from a health history store
            generated_at: chrono::Utc::now(),
            next_check_at: Some(chrono::Utc::now() + chrono::Duration::minutes(5)),
        })
    }

    /// Get hostname
    fn get_hostname() -> BiomeResult<String> {
        Self::get_hostname_with(None)
    }

    /// Hostname resolution with optional override (use in tests instead of setting `HOSTNAME`).
    pub(crate) fn get_hostname_with(hostname_override: Option<&str>) -> BiomeResult<String> {
        if let Some(h) = hostname_override {
            return Ok(h.to_string());
        }
        if let Ok(hostname) = std::env::var("HOSTNAME") {
            Ok(hostname)
        } else {
            Ok(gethostname::gethostname().to_string_lossy().to_string())
        }
    }

    /// Get kernel information
    fn get_kernel_info() -> BiomeResult<KernelInfo> {
        // Try to read from /proc/version on Linux
        if let Ok(version_info) = fs::read_to_string("/proc/version") {
            Ok(KernelInfo {
                name: "Linux".to_string(),
                version: version_info.lines().next().unwrap_or("unknown").to_string(),
                architecture: std::env::consts::ARCH.to_string(),
            })
        } else {
            // Fallback for other systems
            Ok(KernelInfo {
                name: std::env::consts::OS.to_string(),
                version: "unknown".to_string(),
                architecture: std::env::consts::ARCH.to_string(),
            })
        }
    }

    /// Determine health from resource metrics
    pub(crate) fn determine_health_from_metrics(metrics: &ResourceMetrics) -> Health {
        let cpu_usage = metrics.cpu_usage.unwrap_or(0.0);
        let memory_usage = metrics.memory_usage.unwrap_or(0.0);
        let disk_usage = metrics.disk_usage.unwrap_or(0.0);

        if cpu_usage > 0.95 || memory_usage > 0.95 || disk_usage > 0.95 {
            Health::Critical {
                issues: vec![biomeos_types::HealthIssue {
                    id: format!("system-critical-{}", chrono::Utc::now().timestamp()),
                    category: biomeos_types::HealthIssueCategory::Resource,
                    severity: biomeos_types::HealthIssueSeverity::Critical,
                    message: "System resources critically high".to_string(),
                    detected_at: chrono::Utc::now(),
                    details: HashMap::new(),
                    remediation: vec![],
                }],
                affected_capabilities: vec!["compute".to_string(), "storage".to_string()],
            }
        } else if cpu_usage > 0.8 || memory_usage > 0.8 || disk_usage > 0.8 {
            Health::Degraded {
                issues: vec![biomeos_types::HealthIssue {
                    id: format!("system-elevated-{}", chrono::Utc::now().timestamp()),
                    category: biomeos_types::HealthIssueCategory::Performance,
                    severity: biomeos_types::HealthIssueSeverity::Medium,
                    message: "System resources elevated".to_string(),
                    detected_at: chrono::Utc::now(),
                    details: HashMap::new(),
                    remediation: vec![],
                }],
                impact_score: Some(0.3),
            }
        } else {
            Health::Healthy
        }
    }

    /// CPU component [`Health`] from normalized usage (0.0–1.0), for tests and `get_system_health`.
    pub(crate) fn cpu_component_health(cpu_usage: Option<f64>) -> Health {
        if cpu_usage.unwrap_or(0.0) > 0.9 {
            Health::Critical {
                issues: vec![biomeos_types::HealthIssue {
                    id: format!("cpu-critical-{}", chrono::Utc::now().timestamp()),
                    category: biomeos_types::HealthIssueCategory::Resource,
                    severity: biomeos_types::HealthIssueSeverity::Critical,
                    message: "CPU usage critically high".to_string(),
                    detected_at: chrono::Utc::now(),
                    details: HashMap::new(),
                    remediation: vec![],
                }],
                affected_capabilities: vec!["compute".to_string()],
            }
        } else if cpu_usage.unwrap_or(0.0) > 0.7 {
            Health::Degraded {
                issues: vec![biomeos_types::HealthIssue {
                    id: format!("cpu-elevated-{}", chrono::Utc::now().timestamp()),
                    category: biomeos_types::HealthIssueCategory::Performance,
                    severity: biomeos_types::HealthIssueSeverity::Medium,
                    message: "CPU usage elevated".to_string(),
                    detected_at: chrono::Utc::now(),
                    details: HashMap::new(),
                    remediation: vec![],
                }],
                impact_score: Some(0.3),
            }
        } else {
            Health::Healthy
        }
    }

    /// Memory component [`Health`] from normalized usage (0.0–1.0), for tests and `get_system_health`.
    pub(crate) fn memory_component_health(memory_usage: Option<f64>) -> Health {
        if memory_usage.unwrap_or(0.0) > 0.95 {
            Health::Critical {
                issues: vec![biomeos_types::HealthIssue {
                    id: format!("memory-critical-{}", chrono::Utc::now().timestamp()),
                    category: biomeos_types::HealthIssueCategory::Resource,
                    severity: biomeos_types::HealthIssueSeverity::Critical,
                    message: "Memory usage critically high".to_string(),
                    detected_at: chrono::Utc::now(),
                    details: HashMap::new(),
                    remediation: vec![],
                }],
                affected_capabilities: vec!["memory".to_string()],
            }
        } else if memory_usage.unwrap_or(0.0) > 0.8 {
            Health::Degraded {
                issues: vec![biomeos_types::HealthIssue {
                    id: format!("memory-elevated-{}", chrono::Utc::now().timestamp()),
                    category: biomeos_types::HealthIssueCategory::Performance,
                    severity: biomeos_types::HealthIssueSeverity::Medium,
                    message: "Memory usage elevated".to_string(),
                    detected_at: chrono::Utc::now(),
                    details: HashMap::new(),
                    remediation: vec![],
                }],
                impact_score: Some(0.2),
            }
        } else {
            Health::Healthy
        }
    }

    /// Calculate uptime percentage
    pub(crate) fn calculate_uptime_percentage(system_info: &SystemInfo) -> f64 {
        // Simplified calculation - in production would track actual downtime
        let uptime_hours = system_info.uptime.as_secs() as f64 / 3600.0;
        if uptime_hours < 24.0 {
            uptime_hours / 24.0
        } else {
            0.999 // 99.9% uptime assumed for systems running over 24 hours
        }
    }
}

/// Comprehensive system information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SystemInfo {
    /// System hostname
    pub hostname: String,
    /// Kernel version and architecture
    pub kernel_info: KernelInfo,
    /// CPU model and core count
    pub cpu_info: CpuInfo,
    /// RAM usage statistics
    pub memory_info: MemoryInfo,
    /// Per-disk usage statistics
    pub disk_info: Vec<DiskInfo>,
    /// Network interface details
    pub network_info: Vec<NetworkInterface>,
    /// System uptime
    pub uptime: std::time::Duration,
    /// 1/5/15-minute load averages
    pub load_average: LoadAverage,
    /// Timestamp when this snapshot was taken
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Kernel information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct KernelInfo {
    /// Kernel name (e.g. "Linux")
    pub name: String,
    /// Kernel version string
    pub version: String,
    /// Machine architecture (e.g. "`x86_64`")
    pub architecture: String,
}

/// System performance monitor
pub struct SystemMonitor {
    monitoring_interval: std::time::Duration,
}

impl SystemMonitor {
    /// Create a new system monitor
    #[must_use]
    pub const fn new(monitoring_interval: std::time::Duration) -> Self {
        Self {
            monitoring_interval,
        }
    }

    /// Start continuous system monitoring
    pub async fn start_monitoring<F>(&self, callback: F) -> BiomeResult<()>
    where
        F: Fn(HealthReport) + Send + Sync + 'static,
    {
        let mut interval = tokio::time::interval(self.monitoring_interval);

        loop {
            interval.tick().await;

            match SystemInspector::get_system_health().await {
                Ok(health_report) => {
                    callback(health_report);
                }
                Err(e) => {
                    tracing::warn!("Failed to get system health: {e}");
                }
            }
        }
    }
}

#[cfg(test)]
mod tests;
