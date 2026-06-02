#![expect(
    clippy::unwrap_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
#![expect(
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]

use super::*;

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

#[test]
fn test_cpu_config_with_limits() {
    let config = CpuConfig {
        cores: Some(4),
        shares: Some(1024),
        period: Some(100_000),
        quota: Some(50000),
        affinity: Some(vec![0, 1, 2, 3]),
    };
    assert_eq!(config.cores, Some(4));
    assert_eq!(config.affinity.as_ref().unwrap().len(), 4);
}

#[test]
fn test_memory_config_with_limits() {
    let config = MemoryConfig {
        limit: Some(8 * 1024 * 1024 * 1024),
        reservation: Some(4 * 1024 * 1024 * 1024),
        swap_limit: Some(2 * 1024 * 1024 * 1024),
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
        servers: vec!["192.0.2.53".to_string(), "192.0.2.54".to_string()],
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

#[test]
fn test_resource_config_clone() {
    let original = ResourceConfig::default();
    let cloned = original;
    assert!(cloned.cpu.cores.is_none());
}

#[test]
fn test_allocation_policy_debug() {
    let policy = AllocationPolicy::Custom("weighted-fair".to_string());
    let debug = format!("{policy:?}");
    assert!(debug.contains("weighted-fair"));
}

#[test]
fn test_discovery_method_debug() {
    let method = DiscoveryMethod::Kubernetes;
    let debug = format!("{method:?}");
    assert!(debug.contains("Kubernetes"));
}

#[test]
fn test_metrics_format_debug() {
    let format = MetricsFormat::Prometheus;
    let debug = format!("{format:?}");
    assert!(debug.contains("Prometheus"));
}
