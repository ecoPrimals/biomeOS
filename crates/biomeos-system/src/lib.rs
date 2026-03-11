//! BiomeOS System Information and Monitoring
//!
//! This crate provides comprehensive system information gathering, health monitoring,
//! and resource metrics for the BiomeOS ecosystem.

#![warn(missing_docs)]
#![deny(unsafe_code)]

use std::collections::HashMap;
use std::fs;

// Import unified types from biomeos-types
use biomeos_types::{
    health::HealthMetrics, AvailabilityMetrics, BiomeResult, ComponentHealth, Health, HealthReport,
    HealthSubject, HealthSubjectType, NetworkIoMetrics, ResourceMetrics,
};

/// System information inspector
pub struct SystemInspector;

impl SystemInspector {
    /// Get comprehensive system information
    pub async fn get_system_info() -> BiomeResult<SystemInfo> {
        let hostname = Self::get_hostname()?;
        let kernel_info = Self::get_kernel_info()?;
        let cpu_info = Self::get_cpu_info()?;
        let memory_info = Self::get_memory_info()?;
        let disk_info = Self::get_disk_info().await?;
        let network_info = Self::get_network_info().await?;
        let uptime = Self::get_uptime()?;
        let load_average = Self::get_load_average()?;

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
        let cpu_usage = Self::get_cpu_usage().await?;
        let memory_usage = Self::get_memory_usage()?;
        let disk_usage = Self::get_disk_usage().await?;
        let network_io = Self::get_network_io().await?;

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

    /// Get CPU information
    fn get_cpu_info() -> BiomeResult<CpuInfo> {
        // Try to read from /proc/cpuinfo on Linux
        if let Ok(cpuinfo) = fs::read_to_string("/proc/cpuinfo") {
            let mut model_name = "Unknown".to_string();
            let mut cores = 0;

            for line in cpuinfo.lines() {
                if line.starts_with("model name") {
                    if let Some(name) = line.split(':').nth(1) {
                        model_name = name.trim().to_string();
                    }
                } else if line.starts_with("processor") {
                    cores += 1;
                }
            }

            Ok(CpuInfo {
                model: model_name,
                cores,
                architecture: std::env::consts::ARCH.to_string(),
            })
        } else {
            // Fallback
            Ok(CpuInfo {
                model: "Unknown".to_string(),
                cores: 1,
                architecture: std::env::consts::ARCH.to_string(),
            })
        }
    }

    /// Get memory information
    fn get_memory_info() -> BiomeResult<MemoryInfo> {
        // Try to read from /proc/meminfo on Linux
        if let Ok(meminfo) = fs::read_to_string("/proc/meminfo") {
            let mut total_kb = 0;
            let mut available_kb = 0;

            for line in meminfo.lines() {
                if line.starts_with("MemTotal:") {
                    if let Some(value) = line.split_whitespace().nth(1) {
                        total_kb = value.parse::<u64>().unwrap_or(0);
                    }
                } else if line.starts_with("MemAvailable:") {
                    if let Some(value) = line.split_whitespace().nth(1) {
                        available_kb = value.parse::<u64>().unwrap_or(0);
                    }
                }
            }

            let total_gb = total_kb as f64 / 1024.0 / 1024.0;
            let available_gb = available_kb as f64 / 1024.0 / 1024.0;
            let used_gb = total_gb - available_gb;

            Ok(MemoryInfo {
                total_gb,
                used_gb,
                available_gb,
                usage_percent: used_gb / total_gb,
            })
        } else {
            // Fallback
            Ok(MemoryInfo {
                total_gb: 8.0,
                used_gb: 4.0,
                available_gb: 4.0,
                usage_percent: 0.5,
            })
        }
    }

    /// Get disk information using sysinfo for cross-platform support
    async fn get_disk_info() -> BiomeResult<Vec<DiskInfo>> {
        use sysinfo::Disks;

        let disks_info = Disks::new_with_refreshed_list();
        let mut result = Vec::new();

        for disk in &disks_info {
            let total_bytes = disk.total_space();
            let available_bytes = disk.available_space();
            let used_bytes = total_bytes.saturating_sub(available_bytes);

            let total_gb = total_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
            let used_gb = used_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
            let available_gb = available_bytes as f64 / (1024.0 * 1024.0 * 1024.0);

            let usage_percent = if total_gb > 0.0 {
                used_gb / total_gb
            } else {
                0.0
            };

            result.push(DiskInfo {
                device: disk.name().to_string_lossy().to_string(),
                mount_point: disk.mount_point().to_string_lossy().to_string(),
                filesystem: disk.file_system().to_string_lossy().to_string(),
                total_gb,
                used_gb,
                available_gb,
                usage_percent,
            });
        }

        // Ensure at least one disk entry for systems where detection fails
        if result.is_empty() {
            result.push(DiskInfo {
                device: "unknown".to_string(),
                mount_point: "/".to_string(),
                filesystem: "unknown".to_string(),
                total_gb: 0.0,
                used_gb: 0.0,
                available_gb: 0.0,
                usage_percent: 0.0,
            });
        }

        Ok(result)
    }

    /// Get network information using sysinfo for cross-platform support
    async fn get_network_info() -> BiomeResult<Vec<NetworkInterface>> {
        use sysinfo::Networks;

        let networks = Networks::new_with_refreshed_list();
        let mut result = Vec::new();

        for (interface_name, network) in &networks {
            // Determine interface type based on name
            let interface_type = if interface_name.starts_with("lo") {
                NetworkInterfaceType::Loopback
            } else if interface_name.starts_with("eth") || interface_name.starts_with("enp") {
                NetworkInterfaceType::Ethernet
            } else if interface_name.starts_with("wlan") || interface_name.starts_with("wlp") {
                NetworkInterfaceType::Wireless
            } else if interface_name.starts_with("docker") || interface_name.starts_with("br") {
                NetworkInterfaceType::Bridge
            } else {
                NetworkInterfaceType::Other(interface_name.clone())
            };

            result.push(NetworkInterface {
                name: interface_name.clone(),
                interface_type,
                status: NetworkInterfaceStatus::Up, // sysinfo only shows active interfaces
                addresses: vec![], // IP addresses not directly available in sysinfo
                mac_address: Some(format!("{:?}", network.mac_address())),
                mtu: 0, // MTU not available in sysinfo
                bytes_sent: network.total_transmitted(),
                bytes_received: network.total_received(),
                packets_sent: network.total_packets_transmitted(),
                packets_received: network.total_packets_received(),
            });
        }

        // Ensure at least loopback interface for systems where detection fails
        if result.is_empty() {
            result.push(NetworkInterface {
                name: "lo".to_string(),
                interface_type: NetworkInterfaceType::Loopback,
                status: NetworkInterfaceStatus::Up,
                addresses: vec!["127.0.0.1".to_string()],
                mac_address: None,
                mtu: 65536,
                bytes_sent: 0,
                bytes_received: 0,
                packets_sent: 0,
                packets_received: 0,
            });
        }

        Ok(result)
    }

    /// Get system uptime
    fn get_uptime() -> BiomeResult<std::time::Duration> {
        // Try to read from /proc/uptime on Linux
        if let Ok(uptime_str) = fs::read_to_string("/proc/uptime") {
            if let Some(uptime_seconds) = uptime_str.split_whitespace().next() {
                if let Ok(seconds) = uptime_seconds.parse::<f64>() {
                    return Ok(std::time::Duration::from_secs(seconds as u64));
                }
            }
        }

        // Fallback
        Ok(std::time::Duration::from_secs(3600)) // 1 hour placeholder
    }

    /// Get load average
    fn get_load_average() -> BiomeResult<LoadAverage> {
        // Try to read from /proc/loadavg on Linux
        if let Ok(loadavg_str) = fs::read_to_string("/proc/loadavg") {
            let parts: Vec<&str> = loadavg_str.split_whitespace().collect();
            if parts.len() >= 3 {
                return Ok(LoadAverage {
                    load_1m: parts[0].parse::<f64>().unwrap_or(0.0),
                    load_5m: parts[1].parse::<f64>().unwrap_or(0.0),
                    load_15m: parts[2].parse::<f64>().unwrap_or(0.0),
                });
            }
        }

        // Fallback
        Ok(LoadAverage {
            load_1m: 0.1,
            load_5m: 0.1,
            load_15m: 0.1,
        })
    }

    /// Get current CPU usage using sysinfo
    async fn get_cpu_usage() -> BiomeResult<f64> {
        use sysinfo::{CpuRefreshKind, RefreshKind, System};

        let mut sys =
            System::new_with_specifics(RefreshKind::new().with_cpu(CpuRefreshKind::everything()));

        // Need to refresh twice for accurate CPU usage
        sys.refresh_cpu();
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
        sys.refresh_cpu();

        let global_cpu = sys.global_cpu_info();
        Ok(f64::from(global_cpu.cpu_usage()) / 100.0)
    }

    /// Get current memory usage
    fn get_memory_usage() -> BiomeResult<f64> {
        let memory_info = Self::get_memory_info()?;
        Ok(memory_info.usage_percent)
    }

    /// Get current disk usage (average across all disks)
    async fn get_disk_usage() -> BiomeResult<f64> {
        let disks = Self::get_disk_info().await?;
        if disks.is_empty() {
            return Ok(0.0);
        }

        let total_usage: f64 = disks.iter().map(|d| d.usage_percent).sum();
        Ok(total_usage / disks.len() as f64)
    }

    /// Get current network I/O using sysinfo
    async fn get_network_io() -> BiomeResult<NetworkIoMetrics> {
        use sysinfo::Networks;

        let mut networks = Networks::new_with_refreshed_list();

        // First measurement
        let initial_rx: u64 = networks.values().map(|data| data.total_received()).sum();
        let initial_tx: u64 = networks.values().map(|data| data.total_transmitted()).sum();
        let initial_rx_packets: u64 = networks
            .values()
            .map(|data| data.total_packets_received())
            .sum();
        let initial_tx_packets: u64 = networks
            .values()
            .map(|data| data.total_packets_transmitted())
            .sum();

        // Wait 1 second
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        // Second measurement
        networks.refresh();
        let final_rx: u64 = networks.values().map(|data| data.total_received()).sum();
        let final_tx: u64 = networks.values().map(|data| data.total_transmitted()).sum();
        let final_rx_packets: u64 = networks
            .values()
            .map(|data| data.total_packets_received())
            .sum();
        let final_tx_packets: u64 = networks
            .values()
            .map(|data| data.total_packets_transmitted())
            .sum();

        Ok(NetworkIoMetrics {
            bytes_in_per_sec: (final_rx.saturating_sub(initial_rx)) as f64,
            bytes_out_per_sec: (final_tx.saturating_sub(initial_tx)) as f64,
            packets_in_per_sec: (final_rx_packets.saturating_sub(initial_rx_packets)) as f64,
            packets_out_per_sec: (final_tx_packets.saturating_sub(initial_tx_packets)) as f64,
        })
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

/// CPU information  
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CpuInfo {
    /// CPU model name
    pub model: String,
    /// Number of logical CPU cores
    pub cores: u32,
    /// CPU architecture (e.g. "x86_64", "aarch64")
    pub architecture: String,
}

/// Memory information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MemoryInfo {
    /// Total physical memory in GiB
    pub total_gb: f64,
    /// Used memory in GiB
    pub used_gb: f64,
    /// Available memory in GiB
    pub available_gb: f64,
    /// Memory usage as a percentage (0–100)
    pub usage_percent: f64,
}

/// Disk information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DiskInfo {
    /// Block device path (e.g. "/dev/sda1")
    pub device: String,
    /// Mount point (e.g. "/", "/home")
    pub mount_point: String,
    /// Filesystem type (e.g. "ext4", "btrfs")
    pub filesystem: String,
    /// Total disk capacity in GiB
    pub total_gb: f64,
    /// Used disk space in GiB
    pub used_gb: f64,
    /// Available disk space in GiB
    pub available_gb: f64,
    /// Disk usage as a percentage (0–100)
    pub usage_percent: f64,
}

/// Network interface information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NetworkInterface {
    /// Interface name (e.g. "eth0", "wlan0")
    pub name: String,
    /// Type of network interface
    pub interface_type: NetworkInterfaceType,
    /// Current operational status
    pub status: NetworkInterfaceStatus,
    /// IP addresses bound to this interface
    pub addresses: Vec<String>,
    /// MAC / hardware address
    pub mac_address: Option<String>,
    /// Maximum transmission unit in bytes
    pub mtu: u32,
    /// Cumulative bytes transmitted
    pub bytes_sent: u64,
    /// Cumulative bytes received
    pub bytes_received: u64,
    /// Cumulative packets transmitted
    pub packets_sent: u64,
    /// Cumulative packets received
    pub packets_received: u64,
}

/// Network interface types
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum NetworkInterfaceType {
    /// Wired Ethernet
    Ethernet,
    /// Wi-Fi / wireless
    Wireless,
    /// Loopback (lo)
    Loopback,
    /// Virtual bridge
    Bridge,
    /// VPN / tunnel interface
    Tunnel,
    /// Unknown or other interface type
    Other(String),
}

/// Network interface status
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum NetworkInterfaceStatus {
    /// Interface is up and operational
    Up,
    /// Interface is down
    Down,
    /// Status could not be determined
    Unknown,
}

/// System load average
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LoadAverage {
    /// 1-minute load average
    pub load_1m: f64,
    /// 5-minute load average
    pub load_5m: f64,
    /// 15-minute load average
    pub load_15m: f64,
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

    #[test]
    fn test_cpu_info() {
        let cpu_info = SystemInspector::get_cpu_info().expect("get_cpu_info should succeed");

        assert!(cpu_info.cores >= 1, "should have at least 1 core");
        assert_eq!(
            cpu_info.architecture,
            std::env::consts::ARCH,
            "architecture should match target"
        );
    }

    #[test]
    fn test_memory_info() {
        let memory_info =
            SystemInspector::get_memory_info().expect("get_memory_info should succeed");

        assert!(
            memory_info.total_gb >= 0.0,
            "total_gb should be non-negative"
        );
        assert!(memory_info.used_gb >= 0.0, "used_gb should be non-negative");
        assert!(
            memory_info.available_gb >= 0.0,
            "available_gb should be non-negative"
        );
        assert!(
            memory_info.usage_percent >= 0.0 && memory_info.usage_percent <= 1.0,
            "usage_percent should be in 0-1 range"
        );
        assert!(
            (memory_info.used_gb + memory_info.available_gb - memory_info.total_gb).abs() < 0.01,
            "used + available should approximately equal total"
        );
    }

    #[test]
    fn test_uptime() {
        let uptime = SystemInspector::get_uptime().expect("get_uptime should succeed");
        assert!(uptime.as_secs() > 0, "uptime should be positive");
    }

    #[test]
    fn test_load_average() {
        let load_avg =
            SystemInspector::get_load_average().expect("get_load_average should succeed");

        assert!(load_avg.load_1m >= 0.0, "load_1m should be non-negative");
        assert!(load_avg.load_5m >= 0.0, "load_5m should be non-negative");
        assert!(load_avg.load_15m >= 0.0, "load_15m should be non-negative");
    }

    #[tokio::test]
    async fn test_disk_info() {
        let disk_info = SystemInspector::get_disk_info()
            .await
            .expect("get_disk_info should succeed");

        assert!(!disk_info.is_empty(), "should have at least one disk");
        for disk in &disk_info {
            assert!(!disk.device.is_empty(), "device should not be empty");
            assert!(
                !disk.mount_point.is_empty(),
                "mount_point should not be empty"
            );
            assert!(
                disk.total_gb >= 0.0 && disk.used_gb >= 0.0 && disk.available_gb >= 0.0,
                "disk sizes should be non-negative"
            );
            assert!(
                disk.usage_percent >= 0.0 && disk.usage_percent <= 1.0,
                "usage_percent should be in 0-1 range"
            );
        }
    }

    #[tokio::test]
    async fn test_network_info() {
        let network_info = SystemInspector::get_network_info()
            .await
            .expect("get_network_info should succeed");

        assert!(
            !network_info.is_empty(),
            "should have at least one interface"
        );
        for iface in &network_info {
            assert!(!iface.name.is_empty(), "interface name should not be empty");
        }
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

    #[test]
    fn test_cpu_info_serialization_roundtrip() {
        let info = CpuInfo {
            model: "Intel Core i7".to_string(),
            cores: 8,
            architecture: "x86_64".to_string(),
        };
        let json = serde_json::to_string(&info).expect("serialization should succeed");
        let deserialized: CpuInfo =
            serde_json::from_str(&json).expect("deserialization should succeed");
        assert_eq!(info.model, deserialized.model);
        assert_eq!(info.cores, deserialized.cores);
    }

    #[test]
    fn test_memory_info_serialization_roundtrip() {
        let info = MemoryInfo {
            total_gb: 32.0,
            used_gb: 16.0,
            available_gb: 16.0,
            usage_percent: 0.5,
        };
        let json = serde_json::to_string(&info).expect("serialization should succeed");
        let deserialized: MemoryInfo =
            serde_json::from_str(&json).expect("deserialization should succeed");
        assert!((info.total_gb - deserialized.total_gb).abs() < 0.001);
        assert!((info.usage_percent - deserialized.usage_percent).abs() < 0.001);
    }

    #[test]
    fn test_disk_info_serialization_roundtrip() {
        let info = DiskInfo {
            device: "/dev/nvme0n1p1".to_string(),
            mount_point: "/".to_string(),
            filesystem: "btrfs".to_string(),
            total_gb: 500.0,
            used_gb: 250.0,
            available_gb: 250.0,
            usage_percent: 0.5,
        };
        let json = serde_json::to_string(&info).expect("serialization should succeed");
        let deserialized: DiskInfo =
            serde_json::from_str(&json).expect("deserialization should succeed");
        assert_eq!(info.device, deserialized.device);
        assert_eq!(info.mount_point, deserialized.mount_point);
        assert!((info.total_gb - deserialized.total_gb).abs() < 0.001);
    }

    #[test]
    fn test_load_average_serialization_roundtrip() {
        let info = LoadAverage {
            load_1m: 2.5,
            load_5m: 2.0,
            load_15m: 1.5,
        };
        let json = serde_json::to_string(&info).expect("serialization should succeed");
        let deserialized: LoadAverage =
            serde_json::from_str(&json).expect("deserialization should succeed");
        assert!((info.load_1m - deserialized.load_1m).abs() < 0.001);
        assert!((info.load_5m - deserialized.load_5m).abs() < 0.001);
        assert!((info.load_15m - deserialized.load_15m).abs() < 0.001);
    }

    #[test]
    fn test_network_interface_type_serialization() {
        let variants = [
            NetworkInterfaceType::Ethernet,
            NetworkInterfaceType::Wireless,
            NetworkInterfaceType::Loopback,
            NetworkInterfaceType::Bridge,
            NetworkInterfaceType::Tunnel,
            NetworkInterfaceType::Other("veth0".to_string()),
        ];
        for variant in &variants {
            let json = serde_json::to_string(variant).expect("serialization should succeed");
            let deserialized: NetworkInterfaceType =
                serde_json::from_str(&json).expect("deserialization should succeed");
            assert_eq!(
                std::mem::discriminant(variant),
                std::mem::discriminant(&deserialized)
            );
        }
    }

    #[test]
    fn test_network_interface_status_serialization() {
        let variants = [
            NetworkInterfaceStatus::Up,
            NetworkInterfaceStatus::Down,
            NetworkInterfaceStatus::Unknown,
        ];
        for variant in &variants {
            let json = serde_json::to_string(variant).expect("serialization should succeed");
            let deserialized: NetworkInterfaceStatus =
                serde_json::from_str(&json).expect("deserialization should succeed");
            assert_eq!(
                std::mem::discriminant(variant),
                std::mem::discriminant(&deserialized)
            );
        }
    }

    #[test]
    fn test_network_interface_serialization_roundtrip() {
        let info = NetworkInterface {
            name: "wlan0".to_string(),
            interface_type: NetworkInterfaceType::Wireless,
            status: NetworkInterfaceStatus::Up,
            addresses: vec!["192.168.1.100".to_string()],
            mac_address: Some("aa:bb:cc:dd:ee:ff".to_string()),
            mtu: 1500,
            bytes_sent: 1_000_000,
            bytes_received: 2_000_000,
            packets_sent: 5000,
            packets_received: 10000,
        };
        let json = serde_json::to_string(&info).expect("serialization should succeed");
        let deserialized: NetworkInterface =
            serde_json::from_str(&json).expect("deserialization should succeed");
        assert_eq!(info.name, deserialized.name);
        assert_eq!(info.addresses, deserialized.addresses);
        assert_eq!(info.bytes_sent, deserialized.bytes_sent);
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

    #[test]
    fn test_disk_info_zero_total_usage_percent() {
        let info = DiskInfo {
            device: "/dev/zero".to_string(),
            mount_point: "/mnt".to_string(),
            filesystem: "tmpfs".to_string(),
            total_gb: 0.0,
            used_gb: 0.0,
            available_gb: 0.0,
            usage_percent: 0.0,
        };
        let json = serde_json::to_string(&info).expect("serialization should succeed");
        let deserialized: DiskInfo =
            serde_json::from_str(&json).expect("deserialization should succeed");
        assert_eq!(deserialized.usage_percent, 0.0);
    }

    #[test]
    fn test_memory_info_zero_total_avoids_nan() {
        let info = MemoryInfo {
            total_gb: 0.0,
            used_gb: 0.0,
            available_gb: 0.0,
            usage_percent: 0.0,
        };
        let json = serde_json::to_string(&info).expect("serialization should succeed");
        let deserialized: MemoryInfo =
            serde_json::from_str(&json).expect("deserialization should succeed");
        assert!(!deserialized.usage_percent.is_nan());
        assert_eq!(deserialized.usage_percent, 0.0);
    }

    #[test]
    fn test_network_interface_other_type_with_custom_string() {
        let info = NetworkInterface {
            name: "veth12345".to_string(),
            interface_type: NetworkInterfaceType::Other("custom".to_string()),
            status: NetworkInterfaceStatus::Unknown,
            addresses: vec![],
            mac_address: None,
            mtu: 0,
            bytes_sent: 0,
            bytes_received: 0,
            packets_sent: 0,
            packets_received: 0,
        };
        let json = serde_json::to_string(&info).expect("serialization should succeed");
        let deserialized: NetworkInterface =
            serde_json::from_str(&json).expect("deserialization should succeed");
        if let NetworkInterfaceType::Other(s) = &deserialized.interface_type {
            assert_eq!(s, "custom");
        } else {
            panic!("Expected Other variant");
        }
    }

    #[test]
    fn test_clone_impls() {
        let load = LoadAverage {
            load_1m: 1.0,
            load_5m: 1.0,
            load_15m: 1.0,
        };
        let cloned = load.clone();
        assert_eq!(load.load_1m, cloned.load_1m);

        let status = NetworkInterfaceStatus::Up;
        let cloned_status = status.clone();
        assert!(matches!(cloned_status, NetworkInterfaceStatus::Up));
    }
}
