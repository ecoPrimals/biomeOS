// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

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
    /// Static service list
    Static,
    /// DNS-based discovery
    Dns,
    /// Consul service discovery
    Consul,
    /// Kubernetes service discovery
    Kubernetes,
    /// Registry-based discovery
    Registry,
    /// mDNS multicast discovery
    Mdns,
    /// Custom discovery method
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
    /// Prometheus exposition format
    Prometheus,
    /// JSON format
    Json,
    /// StatsD format
    StatsD,
    /// Custom metrics format
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

#[cfg(test)]
mod tests {
    use super::*;

    // ═══════════════════════════════════════════════════════════════════════
    // Default Implementations
    // ═══════════════════════════════════════════════════════════════════════

    #[test]
    fn test_resource_config_default() {
        let config = ResourceConfig::default();
        assert!(config.cpu.cores.is_none());
        assert!(config.memory.limit.is_none());
        assert!(config.disk.space_limit.is_none());
        assert!(config.network.bandwidth_limit.is_none());
    }

    #[test]
    fn test_cpu_config_default() {
        let config = CpuConfig::default();
        assert!(config.cores.is_none());
        assert!(config.shares.is_none());
        assert!(config.period.is_none());
        assert!(config.quota.is_none());
        assert!(config.affinity.is_none());
    }

    #[test]
    fn test_memory_config_default() {
        let config = MemoryConfig::default();
        assert!(config.limit.is_none());
        assert!(config.reservation.is_none());
        assert!(config.swap_limit.is_none());
        assert!(config.swappiness.is_none());
        assert!(!config.oom_kill_disable);
    }

    #[test]
    fn test_disk_config_default() {
        let config = DiskConfig::default();
        assert!(config.space_limit.is_none());
        assert!(config.read_bps_limit.is_none());
        assert!(config.write_bps_limit.is_none());
    }

    #[test]
    fn test_network_resource_config_default() {
        let config = NetworkResourceConfig::default();
        assert!(config.bandwidth_limit.is_none());
        assert!(config.burst_limit.is_none());
        assert!(config.connection_limit.is_none());
        assert!(config.rate_limit.is_none());
    }

    #[test]
    fn test_resource_limits_default() {
        let config = ResourceLimits::default();
        assert!(config.max_file_descriptors.is_none());
        assert!(config.max_processes.is_none());
        assert!(config.max_threads.is_none());
    }

    #[test]
    fn test_resource_allocation_default() {
        let config = ResourceAllocation::default();
        assert!(matches!(config.policy, AllocationPolicy::Fair));
        assert!(config.priorities.is_empty());
        assert!(config.auto_scaling.is_none());
    }

    #[test]
    fn test_discovery_config_default() {
        let config = DiscoveryConfig::default();
        assert_eq!(config.default_method, DiscoveryMethod::Static);
        assert_eq!(config.methods.len(), 1);
        assert!(config.registry.is_none());
        assert!(config.dns.is_none());
        assert!(config.consul.is_none());
        assert!(config.kubernetes.is_none());
    }

    #[test]
    fn test_health_monitoring_config_default() {
        let config = HealthMonitoringConfig::default();
        assert!(config.enabled);
        assert_eq!(config.check_interval, Duration::from_secs(30));
        assert_eq!(config.check_timeout, Duration::from_secs(5));
        assert_eq!(config.unhealthy_threshold, 3);
        assert_eq!(config.healthy_threshold, 2);
        assert!(config.checks.is_empty());
    }

    #[test]
    fn test_metrics_config_default() {
        let config = MetricsConfig::default();
        assert!(config.enabled);
        assert!(matches!(config.format, MetricsFormat::Prometheus));
        assert!(config.endpoint.is_none());
        assert_eq!(config.interval, Duration::from_secs(60));
        assert_eq!(config.retention, Duration::from_secs(86400));
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Enum Serialization
    // ═══════════════════════════════════════════════════════════════════════

    #[test]
    fn test_allocation_policy_serialization() {
        for policy in [
            AllocationPolicy::Fair,
            AllocationPolicy::Priority,
            AllocationPolicy::Fcfs,
            AllocationPolicy::Custom("weighted".to_string()),
        ] {
            let json = serde_json::to_string(&policy).expect("serialize");
            let _: AllocationPolicy = serde_json::from_str(&json).expect("deserialize");
        }
    }

    #[test]
    fn test_discovery_method_serialization() {
        for method in [
            DiscoveryMethod::Static,
            DiscoveryMethod::Dns,
            DiscoveryMethod::Consul,
            DiscoveryMethod::Kubernetes,
            DiscoveryMethod::Registry,
            DiscoveryMethod::Mdns,
            DiscoveryMethod::Custom("etcd".to_string()),
        ] {
            let json = serde_json::to_string(&method).expect("serialize");
            let _: DiscoveryMethod = serde_json::from_str(&json).expect("deserialize");
        }
    }

    #[test]
    fn test_metrics_format_serialization() {
        for format in [
            MetricsFormat::Prometheus,
            MetricsFormat::Json,
            MetricsFormat::StatsD,
            MetricsFormat::Custom("influx".to_string()),
        ] {
            let json = serde_json::to_string(&format).expect("serialize");
            let _: MetricsFormat = serde_json::from_str(&json).expect("deserialize");
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Complex Configuration Types
    // ═══════════════════════════════════════════════════════════════════════

    #[test]
    fn test_cpu_config_with_limits() {
        let config = CpuConfig {
            cores: Some(4),
            shares: Some(1024),
            period: Some(100000),
            quota: Some(50000),
            affinity: Some(vec![0, 1, 2, 3]),
        };
        assert_eq!(config.cores, Some(4));
        assert_eq!(config.affinity.as_ref().unwrap().len(), 4);
    }

    #[test]
    fn test_memory_config_with_limits() {
        let config = MemoryConfig {
            limit: Some(8 * 1024 * 1024 * 1024),       // 8GB
            reservation: Some(4 * 1024 * 1024 * 1024), // 4GB
            swap_limit: Some(2 * 1024 * 1024 * 1024),  // 2GB
            swappiness: Some(60),
            oom_kill_disable: true,
        };
        assert!(config.oom_kill_disable);
        assert_eq!(config.swappiness, Some(60));
    }

    #[test]
    fn test_network_rate_limit_creation() {
        let config = NetworkRateLimit {
            requests_per_second: 1000,
            burst_size: 2000,
            window: Duration::from_secs(60),
        };
        assert_eq!(config.requests_per_second, 1000);
        assert_eq!(config.window, Duration::from_secs(60));
    }

    #[test]
    fn test_auto_scaling_config_creation() {
        let config = AutoScalingConfig {
            enabled: true,
            min_instances: 1,
            max_instances: 10,
            target_cpu_utilization: 0.7,
            target_memory_utilization: 0.8,
            scale_up: ScalingPolicy {
                threshold: 0.8,
                evaluation_period: Duration::from_secs(60),
                cooldown: Duration::from_secs(300),
                step: 2,
            },
            scale_down: ScalingPolicy {
                threshold: 0.3,
                evaluation_period: Duration::from_secs(300),
                cooldown: Duration::from_secs(600),
                step: 1,
            },
        };
        assert!(config.enabled);
        assert_eq!(config.min_instances, 1);
        assert_eq!(config.max_instances, 10);
    }

    #[test]
    fn test_registry_config_creation() {
        let config = RegistryConfig {
            url: "http://registry.local:8500".to_string(),
            auth: Some(RegistryAuth {
                username: "admin".to_string(),
                password: "secret".to_string(),
            }),
            health_check_interval: Duration::from_secs(30),
        };
        assert!(config.url.starts_with("http://"));
        assert!(config.auth.is_some());
    }

    #[test]
    fn test_dns_config_creation() {
        let config = DnsConfig {
            servers: vec!["8.8.8.8".to_string(), "8.8.4.4".to_string()],
            domain: "biomeos.local".to_string(),
            timeout: Duration::from_secs(5),
        };
        assert_eq!(config.servers.len(), 2);
        assert_eq!(config.domain, "biomeos.local");
    }

    #[test]
    fn test_consul_config_creation() {
        let config = ConsulConfig {
            url: "http://consul.local:8500".to_string(),
            datacenter: Some("dc1".to_string()),
            token: Some("consul-token".to_string()),
        };
        assert!(config.datacenter.is_some());
        assert!(config.token.is_some());
    }

    #[test]
    fn test_kubernetes_config_creation() {
        let mut selector = HashMap::new();
        selector.insert("app".to_string(), "biomeos".to_string());
        selector.insert("tier".to_string(), "backend".to_string());

        let config = KubernetesConfig {
            namespace: "biomeos".to_string(),
            selector,
            kubeconfig: Some(PathBuf::from("~/.kube/config")),
        };
        assert_eq!(config.namespace, "biomeos");
        assert_eq!(config.selector.len(), 2);
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Serialization Roundtrip
    // ═══════════════════════════════════════════════════════════════════════

    #[test]
    fn test_resource_config_serialization() {
        let config = ResourceConfig::default();
        let json = serde_json::to_string(&config).expect("serialize");
        let deserialized: ResourceConfig = serde_json::from_str(&json).expect("deserialize");
        assert!(deserialized.cpu.cores.is_none());
    }

    #[test]
    fn test_discovery_config_serialization() {
        let config = DiscoveryConfig::default();
        let json = serde_json::to_string(&config).expect("serialize");
        let deserialized: DiscoveryConfig = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deserialized.default_method, DiscoveryMethod::Static);
    }

    #[test]
    fn test_health_monitoring_config_serialization() {
        let config = HealthMonitoringConfig::default();
        let json = serde_json::to_string(&config).expect("serialize");
        let deserialized: HealthMonitoringConfig =
            serde_json::from_str(&json).expect("deserialize");
        assert!(deserialized.enabled);
    }

    #[test]
    fn test_metrics_config_serialization() {
        let config = MetricsConfig::default();
        let json = serde_json::to_string(&config).expect("serialize");
        let deserialized: MetricsConfig = serde_json::from_str(&json).expect("deserialize");
        assert!(deserialized.enabled);
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Clone & Debug
    // ═══════════════════════════════════════════════════════════════════════

    #[test]
    fn test_resource_config_clone() {
        let original = ResourceConfig::default();
        let cloned = original.clone();
        assert!(cloned.cpu.cores.is_none());
    }

    #[test]
    fn test_allocation_policy_debug() {
        let policy = AllocationPolicy::Custom("weighted-fair".to_string());
        let debug = format!("{:?}", policy);
        assert!(debug.contains("weighted-fair"));
    }

    #[test]
    fn test_discovery_method_debug() {
        let method = DiscoveryMethod::Kubernetes;
        let debug = format!("{:?}", method);
        assert!(debug.contains("Kubernetes"));
    }

    #[test]
    fn test_metrics_format_debug() {
        let format = MetricsFormat::Prometheus;
        let debug = format!("{:?}", format);
        assert!(debug.contains("Prometheus"));
    }
}
