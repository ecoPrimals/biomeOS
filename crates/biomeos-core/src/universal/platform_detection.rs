//! Platform Detection and Information
//!
//! This module handles OS-agnostic platform detection, resource discovery,
//! and capability assessment for biomeOS deployment.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Platform information detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformInfo {
    /// Operating system type
    pub os_type: OsType,
    /// Architecture
    pub architecture: String,
    /// Available resources
    pub resources: PlatformResources,
    /// Capabilities detected
    pub capabilities: Vec<PlatformCapability>,
}

/// Operating system types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OsType {
    /// Bare metal deployment
    BareMetal,
    /// Linux distributions
    Linux {
        distribution: String,
        version: String,
    },
    /// Windows versions
    Windows { version: String },
    /// macOS versions
    MacOS { version: String },
    /// Container environment
    Container { runtime: String },
    /// Cloud environment
    Cloud {
        provider: String,
        instance_type: String,
    },
    /// Unknown/custom platform
    Unknown,
}

/// Platform resources detected
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformResources {
    /// CPU cores available
    pub cpu_cores: u32,
    /// Memory in MB
    pub memory_mb: u64,
    /// Storage in MB
    pub storage_mb: u64,
    /// GPU information
    pub gpu_info: Option<GpuInfo>,
    /// Network interfaces
    pub network_interfaces: Vec<NetworkInterface>,
}

/// GPU information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuInfo {
    /// GPU vendor
    pub vendor: String,
    /// GPU model
    pub model: String,
    /// GPU memory in MB
    pub memory_mb: u64,
    /// CUDA/OpenCL support
    pub compute_capability: Vec<String>,
}

/// Network interface information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInterface {
    /// Interface name
    pub name: String,
    /// IP addresses
    pub addresses: Vec<String>,
    /// Interface speed in Mbps
    pub speed_mbps: Option<u64>,
}

/// Platform capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PlatformCapability {
    /// Containerization support
    Containers { runtime: String },
    /// Virtualization support
    Virtualization { technology: String },
    /// GPU compute support
    GpuCompute { technology: String },
    /// High-speed storage
    HighSpeedStorage { technology: String },
    /// High-bandwidth networking
    HighBandwidthNet { speed_mbps: u64 },
    /// Specialized hardware
    SpecializedHardware {
        name: String,
        capabilities: Vec<String>,
    },
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
    /// Single-node deployment
    SingleNode,
    /// Multi-node cluster
    Cluster { nodes: Vec<String> },
    /// Edge deployment
    Edge { constraints: EdgeConstraints },
    /// Cloud deployment
    Cloud { provider: String, region: String },
    /// Hybrid deployment
    Hybrid { zones: Vec<DeploymentZone> },
}

/// Edge deployment constraints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeConstraints {
    /// Maximum memory usage
    pub max_memory_mb: u64,
    /// Maximum storage usage
    pub max_storage_mb: u64,
    /// Network bandwidth limits
    pub max_bandwidth_mbps: u64,
    /// Power consumption limits
    pub max_power_watts: Option<u64>,
}

/// Deployment zone configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentZone {
    /// Zone name
    pub name: String,
    /// Zone type
    pub zone_type: String,
    /// Nodes in this zone
    pub nodes: Vec<String>,
    /// Zone-specific configuration
    pub config: HashMap<String, serde_json::Value>,
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
    /// Windows Service Manager
    WindowsService,
    /// macOS launchd
    Launchd,
    /// Docker Compose
    DockerCompose,
    /// Kubernetes
    Kubernetes,
    /// Custom service manager
    Custom { name: String },
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
    /// Stable releases only
    Stable,
    /// Beta releases
    Beta,
    /// Nightly builds
    Nightly,
    /// Custom channel
    Custom { name: String },
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
                schedule: "0 2 * * *".to_string(), // Daily at 2 AM
                backup_before_update: true,
            },
        }
    }
}
