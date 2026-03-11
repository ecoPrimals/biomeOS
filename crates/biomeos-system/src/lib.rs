//! BiomeOS System Information and Monitoring
//!
//! This crate provides comprehensive system information gathering, health monitoring,
//! and resource metrics for the BiomeOS ecosystem.

#![warn(missing_docs)]
#![deny(unsafe_code)]

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
    health::HealthMetrics, AvailabilityMetrics, BiomeResult, ComponentHealth, Health, HealthReport,
    HealthSubject, HealthSubjectType, ResourceMetrics,
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
        let disk_info = disk::get_disk_info().await?;
        let network_info = network::get_network_info().await?;
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
        let disk_usage = disk::get_disk_usage().await?;
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
        let cpu_health = if resource_metrics.cpu_usage.unwrap_or(0.0) > 0.9 {
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
        } else if resource_metrics.cpu_usage.unwrap_or(0.0) > 0.7 {
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
        };

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
        let memory_health = if resource_metrics.memory_usage.unwrap_or(0.0) > 0.95 {
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
        } else if resource_metrics.memory_usage.unwrap_or(0.0) > 0.8 {
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
        };

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
        if let Ok(hostname) = std::env::var("HOSTNAME") {
            Ok(hostname)
        } else if let Ok(hostname) = hostname::get() {
            Ok(hostname.to_string_lossy().to_string())
        } else {
            Ok("unknown".to_string())
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
    fn determine_health_from_metrics(metrics: &ResourceMetrics) -> Health {
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

    /// Calculate uptime percentage
    fn calculate_uptime_percentage(system_info: &SystemInfo) -> f64 {
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
    /// Machine architecture (e.g. "x86_64")
    pub architecture: String,
}

/// System performance monitor
pub struct SystemMonitor {
    monitoring_interval: std::time::Duration,
}

impl SystemMonitor {
    /// Create a new system monitor
    pub fn new(monitoring_interval: std::time::Duration) -> Self {
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
                    eprintln!("Failed to get system health: {}", e);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use biomeos_types::HealthSubjectType;
    use std::sync::atomic::{AtomicUsize, Ordering};

    // ========== SystemInspector - Public API ==========

    #[tokio::test]
    async fn test_system_info_collection() {
        let system_info = SystemInspector::get_system_info()
            .await
            .expect("get_system_info should succeed");

        assert!(
            !system_info.hostname.is_empty(),
            "hostname should not be empty"
        );
        assert!(
            !system_info.kernel_info.name.is_empty(),
            "kernel name should not be empty"
        );
        assert!(
            !system_info.kernel_info.architecture.is_empty(),
            "architecture should not be empty"
        );
        assert!(
            system_info.kernel_info.architecture == std::env::consts::ARCH,
            "kernel architecture should match target architecture"
        );
        assert!(
            system_info.cpu_info.cores >= 1,
            "should have at least 1 core"
        );
        assert!(
            system_info.memory_info.total_gb >= 0.0,
            "total memory should be non-negative"
        );
        assert!(
            system_info.memory_info.usage_percent >= 0.0
                && system_info.memory_info.usage_percent <= 1.0,
            "memory usage_percent should be in 0-1 range"
        );
        assert!(
            !system_info.disk_info.is_empty(),
            "should have at least one disk"
        );
        assert!(
            !system_info.network_info.is_empty(),
            "should have at least one network interface"
        );
        assert!(
            system_info.uptime.as_secs() > 0,
            "uptime should be positive"
        );
        assert!(
            system_info.load_average.load_1m >= 0.0
                && system_info.load_average.load_5m >= 0.0
                && system_info.load_average.load_15m >= 0.0,
            "load averages should be non-negative"
        );
    }

    #[tokio::test]
    async fn test_resource_usage() {
        let resource_usage = SystemInspector::get_resource_usage()
            .await
            .expect("get_resource_usage should succeed");

        assert!(
            resource_usage.cpu_usage.is_some(),
            "cpu_usage should be present"
        );
        assert!(
            resource_usage.memory_usage.is_some(),
            "memory_usage should be present"
        );
        assert!(
            resource_usage.disk_usage.is_some(),
            "disk_usage should be present"
        );
        assert!(
            resource_usage.network_io.is_some(),
            "network_io should be present"
        );

        let cpu = resource_usage.cpu_usage.unwrap();
        assert!(
            (0.0..=1.0).contains(&cpu),
            "cpu_usage should be in 0-1 range, got {}",
            cpu
        );

        let memory = resource_usage.memory_usage.unwrap();
        assert!(
            (0.0..=1.0).contains(&memory),
            "memory_usage should be in 0-1 range, got {}",
            memory
        );

        let disk = resource_usage.disk_usage.unwrap();
        assert!(
            (0.0..=1.0).contains(&disk),
            "disk_usage should be in 0-1 range, got {}",
            disk
        );

        let network = resource_usage.network_io.as_ref().unwrap();
        assert!(
            network.bytes_in_per_sec >= 0.0 && network.bytes_out_per_sec >= 0.0,
            "network I/O should be non-negative"
        );
    }

    #[tokio::test]
    async fn test_system_health() {
        let health_report = SystemInspector::get_system_health()
            .await
            .expect("get_system_health should succeed");

        assert_eq!(
            health_report.subject.subject_type,
            HealthSubjectType::System,
            "subject type should be System"
        );
        assert_eq!(
            health_report.subject.name, "biomeOS System",
            "subject name should match"
        );
        assert!(
            !health_report.subject.id.is_empty(),
            "subject id should not be empty"
        );
        assert!(
            !health_report.components.is_empty(),
            "should have components"
        );

        assert!(
            health_report.components.contains_key("cpu"),
            "should have CPU component"
        );
        assert!(
            health_report.components.contains_key("memory"),
            "should have memory component"
        );

        assert!(
            health_report.metrics.resources.is_some(),
            "metrics should include resources"
        );
        assert!(
            health_report.metrics.availability.is_some(),
            "metrics should include availability"
        );

        let availability = health_report.metrics.availability.as_ref().unwrap();
        assert!(
            availability.uptime_percentage >= 0.0 && availability.uptime_percentage <= 1.0,
            "uptime_percentage should be in 0-1 range"
        );
    }

    #[tokio::test]
    async fn test_system_info_and_health_consistency() {
        let system_info = SystemInspector::get_system_info()
            .await
            .expect("get_system_info should succeed");
        let health_report = SystemInspector::get_system_health()
            .await
            .expect("get_system_health should succeed");

        assert_eq!(
            system_info.hostname, health_report.subject.id,
            "health report subject id should match system hostname"
        );
    }

    // ========== SystemInspector - Private helpers (crate-visible) ==========

    #[test]
    fn test_hostname_retrieval() {
        let hostname = SystemInspector::get_hostname().expect("get_hostname should succeed");
        assert!(!hostname.is_empty(), "hostname should not be empty");
    }

    #[test]
    fn test_kernel_info() {
        let kernel_info =
            SystemInspector::get_kernel_info().expect("get_kernel_info should succeed");

        assert!(
            !kernel_info.name.is_empty(),
            "kernel name should not be empty"
        );
        assert!(
            !kernel_info.architecture.is_empty(),
            "architecture should not be empty"
        );
        assert_eq!(
            kernel_info.architecture,
            std::env::consts::ARCH,
            "architecture should match target"
        );
    }

    // ========== SystemMonitor ==========

    #[test]
    fn test_system_monitor_new() {
        let interval = std::time::Duration::from_secs(30);
        let _monitor = SystemMonitor::new(interval);
        // Verify constructor succeeds; interval is used by start_monitoring
    }

    #[tokio::test]
    #[ignore = "Slow: get_system_health takes ~1.2s; run with --ignored for full coverage"]
    async fn test_system_monitor_start_monitoring_receives_reports() {
        let monitor = SystemMonitor::new(std::time::Duration::from_millis(100));
        let report_count = std::sync::Arc::new(AtomicUsize::new(0));
        let count_for_spawn = report_count.clone();

        let monitor_handle = tokio::spawn(async move {
            let count = count_for_spawn;
            monitor
                .start_monitoring(move |report| {
                    count.fetch_add(1, Ordering::SeqCst);
                    assert_eq!(report.subject.subject_type, HealthSubjectType::System);
                })
                .await
        });

        // get_system_health sleeps ~1.2s (CPU 200ms + network I/O 1s)
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        monitor_handle.abort();

        let received = report_count.load(Ordering::SeqCst);
        assert!(
            received >= 1,
            "should receive at least 1 report within 3s, got {}",
            received
        );
    }

    #[tokio::test]
    async fn test_system_monitor_start_monitoring_spawns_and_aborts() {
        let monitor = SystemMonitor::new(std::time::Duration::from_secs(60));
        let monitor_handle = tokio::spawn(async move {
            monitor
                .start_monitoring(|report| {
                    assert_eq!(report.subject.subject_type, HealthSubjectType::System);
                })
                .await
        });

        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        monitor_handle.abort();
        let _ = monitor_handle.await;
        // Verify we can spawn and abort without panicking
    }

    // ========== Serialization / Deserialization ==========

    #[test]
    fn test_system_info_serialization_roundtrip() {
        let info = SystemInfo {
            hostname: "test-host".to_string(),
            kernel_info: KernelInfo {
                name: "Linux".to_string(),
                version: "5.15.0".to_string(),
                architecture: "x86_64".to_string(),
            },
            cpu_info: CpuInfo {
                model: "Test CPU".to_string(),
                cores: 4,
                architecture: "x86_64".to_string(),
            },
            memory_info: MemoryInfo {
                total_gb: 16.0,
                used_gb: 8.0,
                available_gb: 8.0,
                usage_percent: 0.5,
            },
            disk_info: vec![DiskInfo {
                device: "/dev/sda1".to_string(),
                mount_point: "/".to_string(),
                filesystem: "ext4".to_string(),
                total_gb: 100.0,
                used_gb: 50.0,
                available_gb: 50.0,
                usage_percent: 0.5,
            }],
            network_info: vec![NetworkInterface {
                name: "eth0".to_string(),
                interface_type: NetworkInterfaceType::Ethernet,
                status: NetworkInterfaceStatus::Up,
                addresses: vec!["192.168.1.1".to_string()],
                mac_address: Some("00:11:22:33:44:55".to_string()),
                mtu: 1500,
                bytes_sent: 1000,
                bytes_received: 2000,
                packets_sent: 10,
                packets_received: 20,
            }],
            uptime: std::time::Duration::from_secs(86400),
            load_average: LoadAverage {
                load_1m: 1.5,
                load_5m: 1.2,
                load_15m: 1.0,
            },
            timestamp: chrono::Utc::now(),
        };

        let json = serde_json::to_string(&info).expect("serialization should succeed");
        let deserialized: SystemInfo =
            serde_json::from_str(&json).expect("deserialization should succeed");

        assert_eq!(info.hostname, deserialized.hostname);
        assert_eq!(info.kernel_info.name, deserialized.kernel_info.name);
        assert_eq!(info.cpu_info.cores, deserialized.cpu_info.cores);
        assert!((info.memory_info.total_gb - deserialized.memory_info.total_gb).abs() < 0.001);
        assert_eq!(info.disk_info.len(), deserialized.disk_info.len());
        assert_eq!(info.network_info.len(), deserialized.network_info.len());
        assert_eq!(info.uptime, deserialized.uptime);
    }

    #[test]
    fn test_kernel_info_serialization_roundtrip() {
        let info = KernelInfo {
            name: "Linux".to_string(),
            version: "5.15.0-generic".to_string(),
            architecture: "aarch64".to_string(),
        };
        let json = serde_json::to_string(&info).expect("serialization should succeed");
        let deserialized: KernelInfo =
            serde_json::from_str(&json).expect("deserialization should succeed");
        assert_eq!(info.name, deserialized.name);
        assert_eq!(info.version, deserialized.version);
        assert_eq!(info.architecture, deserialized.architecture);
    }

    // ========== Edge cases ==========

    #[test]
    fn test_serialization_empty_strings() {
        let info = KernelInfo {
            name: String::new(),
            version: String::new(),
            architecture: "x86_64".to_string(),
        };
        let json = serde_json::to_string(&info).expect("serialization should succeed");
        let deserialized: KernelInfo =
            serde_json::from_str(&json).expect("deserialization should succeed");
        assert!(deserialized.name.is_empty());
        assert!(deserialized.version.is_empty());
    }

    #[test]
    fn test_serialization_empty_disk_info_list() {
        let info = SystemInfo {
            hostname: "edge-test".to_string(),
            kernel_info: KernelInfo {
                name: "Linux".to_string(),
                version: "unknown".to_string(),
                architecture: "x86_64".to_string(),
            },
            cpu_info: CpuInfo {
                model: "Unknown".to_string(),
                cores: 1,
                architecture: "x86_64".to_string(),
            },
            memory_info: MemoryInfo {
                total_gb: 1.0,
                used_gb: 0.5,
                available_gb: 0.5,
                usage_percent: 0.5,
            },
            disk_info: vec![],
            network_info: vec![],
            uptime: std::time::Duration::from_secs(1),
            load_average: LoadAverage {
                load_1m: 0.0,
                load_5m: 0.0,
                load_15m: 0.0,
            },
            timestamp: chrono::Utc::now(),
        };
        let json = serde_json::to_string(&info).expect("serialization should succeed");
        let deserialized: SystemInfo =
            serde_json::from_str(&json).expect("deserialization should succeed");
        assert!(deserialized.disk_info.is_empty());
        assert!(deserialized.network_info.is_empty());
    }
}
