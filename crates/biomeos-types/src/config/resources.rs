//! Resources Configuration
//!
//! This module contains resource management and service discovery configuration types.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

use crate::health::HealthCheckConfig;

/// Resource management configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceConfig {
    /// CPU resource configuration
    pub cpu: CpuConfig,

    /// Memory resource configuration
    pub memory: MemoryConfig,

    /// Disk resource configuration
    pub disk: DiskConfig,

    /// Network resource configuration
    pub network: NetworkResourceConfig,

    /// Resource limits
    pub limits: ResourceLimits,

    /// Resource allocation strategy
    pub allocation: ResourceAllocation,
}

/// CPU resource configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CpuConfig {
    /// CPU cores limit
    pub cores: Option<u32>,

    /// CPU shares (relative weight)
    pub shares: Option<u32>,

    /// CPU period (microseconds)
    pub period: Option<u64>,

    /// CPU quota (microseconds)
    pub quota: Option<u64>,

    /// CPU affinity
    pub affinity: Option<Vec<u32>>,
}

/// Memory resource configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MemoryConfig {
    /// Memory limit in bytes
    pub limit: Option<u64>,

    /// Memory reservation in bytes
    pub reservation: Option<u64>,

    /// Swap limit in bytes
    pub swap_limit: Option<u64>,

    /// Enable memory swappiness
    pub swappiness: Option<u32>,

    /// OOM kill disable
    #[serde(default)]
    pub oom_kill_disable: bool,
}

/// Disk resource configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DiskConfig {
    /// Disk space limit in bytes
    pub space_limit: Option<u64>,

    /// Disk IO read limit (bytes per second)
    pub read_bps_limit: Option<u64>,

    /// Disk IO write limit (bytes per second)
    pub write_bps_limit: Option<u64>,

    /// Disk IO read IOPS limit
    pub read_iops_limit: Option<u64>,

    /// Disk IO write IOPS limit
    pub write_iops_limit: Option<u64>,
}

/// Network resource configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NetworkResourceConfig {
    /// Network bandwidth limit (bytes per second)
    pub bandwidth_limit: Option<u64>,

    /// Network burst limit (bytes)
    pub burst_limit: Option<u64>,

    /// Connection limit
    pub connection_limit: Option<u32>,

    /// Rate limiting
    pub rate_limit: Option<NetworkRateLimit>,
}

/// Network rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkRateLimit {
    /// Requests per second
    pub requests_per_second: u32,

    /// Burst size
    pub burst_size: u32,

    /// Rate limit window
    pub window: Duration,
}

/// Resource limits
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum file descriptors
    pub max_file_descriptors: Option<u64>,

    /// Maximum processes
    pub max_processes: Option<u64>,

    /// Maximum threads
    pub max_threads: Option<u64>,

    /// Maximum memory mappings
    pub max_memory_mappings: Option<u64>,

    /// Maximum stack size
    pub max_stack_size: Option<u64>,
}

/// Resource allocation strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceAllocation {
    /// Allocation policy
    pub policy: AllocationPolicy,

    /// Resource priorities
    pub priorities: HashMap<String, u32>,

    /// Auto-scaling configuration
    pub auto_scaling: Option<AutoScalingConfig>,
}

/// Resource allocation policies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AllocationPolicy {
    /// Fair allocation
    Fair,
    /// Priority-based allocation
    Priority,
    /// First-come-first-served
    Fcfs,
    /// Custom allocation policy
    Custom(String),
}

/// Auto-scaling configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoScalingConfig {
    /// Enable auto-scaling
    pub enabled: bool,

    /// Minimum instances
    pub min_instances: u32,

    /// Maximum instances
    pub max_instances: u32,

    /// Target CPU utilization
    pub target_cpu_utilization: f64,

    /// Target memory utilization
    pub target_memory_utilization: f64,

    /// Scale-up policy
    pub scale_up: ScalingPolicy,

    /// Scale-down policy
    pub scale_down: ScalingPolicy,
}

/// Scaling policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalingPolicy {
    /// Scaling threshold
    pub threshold: f64,

    /// Evaluation period
    pub evaluation_period: Duration,

    /// Cooldown period
    pub cooldown: Duration,

    /// Scaling step
    pub step: u32,
}

/// Service discovery configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryConfig {
    /// Default discovery method
    pub default_method: DiscoveryMethod,

    /// Available discovery methods
    pub methods: Vec<DiscoveryMethod>,

    /// Registry configuration
    pub registry: Option<RegistryConfig>,

    /// DNS discovery configuration
    pub dns: Option<DnsConfig>,

    /// Consul configuration
    pub consul: Option<ConsulConfig>,

    /// Kubernetes configuration
    pub kubernetes: Option<KubernetesConfig>,
}

/// Discovery methods
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DiscoveryMethod {
    Static,
    Dns,
    Consul,
    Kubernetes,
    Registry,
    Mdns,
    Custom(String),
}

/// Registry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryConfig {
    /// Registry URL
    pub url: String,

    /// Registry authentication
    pub auth: Option<RegistryAuth>,

    /// Health check interval
    pub health_check_interval: Duration,
}

/// Registry authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryAuth {
    /// Username
    pub username: String,

    /// Password
    pub password: String,
}

/// DNS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsConfig {
    /// DNS servers
    pub servers: Vec<String>,

    /// DNS domain
    pub domain: String,

    /// DNS timeout
    pub timeout: Duration,
}

/// Consul configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsulConfig {
    /// Consul URL
    pub url: String,

    /// Consul datacenter
    pub datacenter: Option<String>,

    /// Consul token
    pub token: Option<String>,
}

/// Kubernetes configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KubernetesConfig {
    /// Kubernetes namespace
    pub namespace: String,

    /// Service selector labels
    pub selector: HashMap<String, String>,

    /// Kubeconfig path
    pub kubeconfig: Option<PathBuf>,
}

/// Health monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthMonitoringConfig {
    /// Enable health monitoring
    pub enabled: bool,

    /// Health check interval
    pub check_interval: Duration,

    /// Health check timeout
    pub check_timeout: Duration,

    /// Unhealthy threshold
    pub unhealthy_threshold: u32,

    /// Healthy threshold
    pub healthy_threshold: u32,

    /// Health check configurations by component
    pub checks: HashMap<String, HealthCheckConfig>,

    /// Metrics collection configuration
    pub metrics: MetricsConfig,
}

/// Metrics configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    /// Enable metrics collection
    pub enabled: bool,

    /// Metrics export format
    pub format: MetricsFormat,

    /// Metrics export endpoint
    pub endpoint: Option<String>,

    /// Metrics collection interval
    pub interval: Duration,

    /// Metrics retention period
    pub retention: Duration,
}

/// Metrics formats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetricsFormat {
    Prometheus,
    Json,
    StatsD,
    Custom(String),
}

/// Default implementations
impl Default for ResourceConfig {
    fn default() -> Self {
        Self {
            cpu: CpuConfig::default(),
            memory: MemoryConfig::default(),
            disk: DiskConfig::default(),
            network: NetworkResourceConfig::default(),
            limits: ResourceLimits::default(),
            allocation: ResourceAllocation::default(),
        }
    }
}

// Default impls now derived via #[derive(Default)] on structs above

impl Default for ResourceAllocation {
    fn default() -> Self {
        Self {
            policy: AllocationPolicy::Fair,
            priorities: HashMap::new(),
            auto_scaling: None,
        }
    }
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        Self {
            default_method: DiscoveryMethod::Static,
            methods: vec![DiscoveryMethod::Static],
            registry: None,
            dns: None,
            consul: None,
            kubernetes: None,
        }
    }
}

impl Default for HealthMonitoringConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            check_interval: Duration::from_secs(30),
            check_timeout: Duration::from_secs(5),
            unhealthy_threshold: 3,
            healthy_threshold: 2,
            checks: HashMap::new(),
            metrics: MetricsConfig::default(),
        }
    }
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            format: MetricsFormat::Prometheus,
            endpoint: None,
            interval: Duration::from_secs(60),
            retention: Duration::from_secs(24 * 60 * 60), // 24 hours
        }
    }
}
