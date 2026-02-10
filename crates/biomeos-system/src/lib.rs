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

    #[tokio::test]
    async fn test_system_info_collection() {
        let system_info = SystemInspector::get_system_info().await;
        assert!(system_info.is_ok());

        let info = system_info.unwrap();
        assert!(!info.hostname.is_empty());
        assert!(!info.kernel_info.name.is_empty());
    }

    #[tokio::test]
    async fn test_resource_usage() {
        let resource_usage = SystemInspector::get_resource_usage().await;
        assert!(resource_usage.is_ok());

        let usage = resource_usage.unwrap();
        assert!(usage.cpu_usage.is_some());
        assert!(usage.memory_usage.is_some());
    }

    #[tokio::test]
    async fn test_system_health() {
        let health_report = SystemInspector::get_system_health().await;
        assert!(health_report.is_ok());

        let report = health_report.unwrap();
        assert_eq!(report.subject.subject_type, HealthSubjectType::System);
        assert!(!report.components.is_empty());
    }

    #[test]
    fn test_hostname_retrieval() {
        let hostname = SystemInspector::get_hostname();
        assert!(hostname.is_ok());
        assert!(!hostname.unwrap().is_empty());
    }

    #[test]
    fn test_kernel_info() {
        let kernel_info = SystemInspector::get_kernel_info();
        assert!(kernel_info.is_ok());

        let info = kernel_info.unwrap();
        assert!(!info.name.is_empty());
        assert!(!info.architecture.is_empty());
    }

    #[test]
    fn test_load_average() {
        let load_avg = SystemInspector::get_load_average();
        assert!(load_avg.is_ok());

        let avg = load_avg.unwrap();
        assert!(avg.load_1m >= 0.0);
        assert!(avg.load_5m >= 0.0);
        assert!(avg.load_15m >= 0.0);
    }
}
