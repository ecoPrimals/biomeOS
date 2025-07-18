//! Universal Platform Detection and Management
//!
//! This module provides cross-platform detection and management capabilities
//! for biomeOS deployment across different operating systems and architectures.

use crate::BiomeResult;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Universal platform detection and management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalPlatform {
    /// Platform information
    pub platform: PlatformInfo,
    /// Deployment configuration
    pub deployment: DeploymentConfig,
    /// MYCORRHIZA configuration
    pub mycorrhiza: super::mycorrhiza::MycorrhizaConfig,
}

/// Platform information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformInfo {
    /// Operating system type
    pub os_type: OsType,
    /// Architecture
    pub architecture: String,
    /// Kernel version
    pub kernel_version: String,
    /// Capabilities
    pub capabilities: Vec<PlatformCapability>,
    /// Resource information
    pub resources: PlatformResources,
}

/// Operating system types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OsType {
    /// Linux-based systems
    Linux,
    /// Windows systems
    Windows,
    /// macOS systems
    MacOS,
    /// FreeBSD systems
    FreeBSD,
    /// Android systems
    Android,
    /// iOS systems
    iOS,
    /// Other/Unknown
    Other(String),
}

/// Platform resource information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformResources {
    /// CPU cores
    pub cpu_cores: u32,
    /// Memory in bytes
    pub memory_bytes: u64,
    /// Storage devices
    pub storage_devices: Vec<StorageDevice>,
    /// Network interfaces
    pub network_interfaces: Vec<NetworkInterface>,
    /// GPU information
    pub gpu_info: Option<GpuInfo>,
}

/// GPU information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuInfo {
    /// GPU vendor
    pub vendor: String,
    /// GPU model
    pub model: String,
    /// Memory in bytes
    pub memory_bytes: u64,
    /// CUDA support
    pub cuda_support: bool,
    /// OpenCL support
    pub opencl_support: bool,
}

/// Network interface information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInterface {
    /// Interface name
    pub name: String,
    /// MAC address
    pub mac_address: String,
    /// IP addresses
    pub ip_addresses: Vec<String>,
}

/// Platform capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PlatformCapability {
    /// Container support
    Containers,
    /// Virtualization support
    Virtualization,
    /// Hardware acceleration
    HardwareAcceleration,
    /// Secure boot
    SecureBoot,
    /// TPM support
    TpmSupport,
    /// Network isolation
    NetworkIsolation,
}

/// Deployment configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentConfig {
    /// Deployment mode
    pub mode: DeploymentMode,
    /// Installation path
    pub install_path: PathBuf,
    /// Data directory
    pub data_dir: PathBuf,
    /// Service management
    pub service_management: ServiceManagement,
    /// Auto-update configuration
    pub auto_update: AutoUpdateConfig,
}

/// Deployment modes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeploymentMode {
    /// Single node deployment
    SingleNode,
    /// Multi-node cluster
    Cluster,
    /// Edge deployment
    Edge(EdgeConstraints),
    /// Federated deployment
    Federated(Vec<DeploymentZone>),
}

/// Edge deployment constraints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeConstraints {
    /// Maximum memory usage
    pub max_memory_mb: u32,
    /// Maximum CPU usage
    pub max_cpu_percent: f64,
    /// Network bandwidth limit
    pub max_bandwidth_mbps: f64,
    /// Storage limit
    pub max_storage_gb: u32,
}

/// Deployment zone configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentZone {
    /// Zone name
    pub name: String,
    /// Zone location
    pub location: String,
    /// Resource limits
    pub resource_limits: EdgeConstraints,
}

/// Service management configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceManagement {
    /// Service manager type
    pub manager_type: ServiceManagerType,
    /// Auto-start services
    pub auto_start: bool,
    /// Service dependencies
    pub dependencies: Vec<String>,
}

/// Service manager types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceManagerType {
    /// systemd (Linux)
    Systemd,
    /// launchd (macOS)
    Launchd,
    /// Windows Service Manager
    WindowsService,
    /// Docker containers
    Docker,
    /// Kubernetes
    Kubernetes,
}

/// Auto-update configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoUpdateConfig {
    /// Enable auto-updates
    pub enabled: bool,
    /// Update channel
    pub channel: UpdateChannel,
    /// Update schedule (cron format)
    pub schedule: String,
    /// Backup before update
    pub backup_before_update: bool,
}

/// Update channels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UpdateChannel {
    /// Stable releases
    Stable,
    /// Beta releases
    Beta,
    /// Alpha releases
    Alpha,
    /// Custom channel
    Custom(String),
}

/// Storage device information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageDevice {
    /// Device name
    pub name: String,
    /// Device type
    pub device_type: StorageType,
    /// Size in bytes
    pub size_bytes: u64,
    /// Available space in bytes
    pub available_bytes: u64,
    /// Mount point
    pub mount_point: Option<String>,
}

/// Storage device types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageType {
    /// Hard disk drive
    Hdd,
    /// Solid state drive
    Ssd,
    /// Network attached storage
    Nas,
    /// Cloud storage
    Cloud,
    /// Other storage type
    Other(String),
}

/// Platform diagnostics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformDiagnostics {
    /// Health status
    pub health_status: String,
    /// Resource usage
    pub resource_usage: PlatformResources,
    /// Service status
    pub service_status: HashMap<String, ServiceStatus>,
    /// Performance metrics
    pub performance_metrics: PerformanceMetrics,
    /// Security status
    pub security_status: SecurityStatus,
}

/// Service status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceStatus {
    /// Service name
    pub name: String,
    /// Status
    pub status: String,
    /// Process ID
    pub pid: Option<u32>,
    /// Memory usage
    pub memory_usage_bytes: u64,
    /// CPU usage percentage
    pub cpu_usage_percent: f64,
}

/// Performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// CPU usage percentage
    pub cpu_usage_percent: f64,
    /// Memory usage percentage
    pub memory_usage_percent: f64,
    /// Disk usage percentage
    pub disk_usage_percent: f64,
    /// Network throughput in Mbps
    pub network_throughput_mbps: f64,
    /// Response times by service
    pub response_times_ms: HashMap<String, f64>,
}

/// Security status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityStatus {
    /// MYCORRHIZA status
    pub mycorrhiza_status: String,
    /// Threat detection active
    pub threat_detection_active: bool,
    /// Encryption status
    pub encryption_status: String,
    /// Access control status
    pub access_control_status: String,
    /// Recent security events
    pub recent_events: Vec<SecurityEvent>,
}

/// Security event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityEvent {
    /// Event timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Event type
    pub event_type: String,
    /// Event description
    pub description: String,
    /// Event severity
    pub severity: SecuritySeverity,
}

/// Security event severity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecuritySeverity {
    /// Low severity
    Low,
    /// Medium severity
    Medium,
    /// High severity
    High,
    /// Critical severity
    Critical,
}

/// Service information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    /// Service name
    pub name: String,
    /// Service description
    pub description: String,
    /// Service status
    pub status: ServiceStatus,
    /// Service manager
    pub manager: ServiceManagerType,
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Universal platform operations trait
#[async_trait]
pub trait UniversalPlatformOps {
    /// Detect platform information
    async fn detect_platform(&self) -> BiomeResult<PlatformInfo>;
    
    /// Install biomeOS
    async fn install_biomeos(&self, config: &DeploymentConfig) -> BiomeResult<()>;
    
    /// Configure services
    async fn configure_services(&self, services: &[String]) -> BiomeResult<()>;
    
    /// Start services
    async fn start_services(&self) -> BiomeResult<()>;
    
    /// Stop services
    async fn stop_services(&self) -> BiomeResult<()>;
    
    /// Update biomeOS
    async fn update_biomeos(&self) -> BiomeResult<()>;
    
    /// Get diagnostics
    async fn get_diagnostics(&self) -> BiomeResult<PlatformDiagnostics>;
}

impl Default for DeploymentConfig {
    fn default() -> Self {
        Self {
            mode: DeploymentMode::SingleNode,
            install_path: PathBuf::from("/opt/biomeos"),
            data_dir: PathBuf::from("/var/lib/biomeos"),
            service_management: ServiceManagement {
                manager_type: ServiceManagerType::Systemd,
                auto_start: true,
                dependencies: Vec::new(),
            },
            auto_update: AutoUpdateConfig {
                enabled: true,
                channel: UpdateChannel::Stable,
                schedule: "0 2 * * *".to_string(),
                backup_before_update: true,
            },
        }
    }
} 