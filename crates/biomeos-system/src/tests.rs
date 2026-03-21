// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

#![expect(
    clippy::unwrap_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
#![expect(
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]

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
        "cpu_usage should be in 0-1 range, got {cpu}"
    );

    let memory = resource_usage.memory_usage.unwrap();
    assert!(
        (0.0..=1.0).contains(&memory),
        "memory_usage should be in 0-1 range, got {memory}"
    );

    let disk = resource_usage.disk_usage.unwrap();
    assert!(
        (0.0..=1.0).contains(&disk),
        "disk_usage should be in 0-1 range, got {disk}"
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
    let kernel_info = SystemInspector::get_kernel_info().expect("get_kernel_info should succeed");

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

#[tokio::test(start_paused = true)]
#[ignore = "Slow: get_system_health takes ~1.2s real time; run with --ignored for full coverage"]
async fn test_system_monitor_start_monitoring_receives_reports() {
    let monitor = SystemMonitor::new(std::time::Duration::from_millis(100));
    let report_count = std::sync::Arc::new(AtomicUsize::new(0));
    let count_for_spawn = report_count.clone();
    let notify = std::sync::Arc::new(tokio::sync::Notify::new());
    let notify_for_wait = notify.clone();

    let monitor_handle = tokio::spawn(async move {
        let count = count_for_spawn;
        let notify_clone = notify.clone();
        monitor
            .start_monitoring(move |report| {
                count.fetch_add(1, Ordering::SeqCst);
                notify_clone.notify_one();
                assert_eq!(report.subject.subject_type, HealthSubjectType::System);
            })
            .await
    });

    // Advance time so interval ticks; first get_system_health runs (real time ~1.2s)
    tokio::time::advance(std::time::Duration::from_secs(3)).await;
    // Wait for first report (callback runs after get_system_health completes)
    tokio::time::timeout(
        std::time::Duration::from_secs(3),
        notify_for_wait.notified(),
    )
    .await
    .expect("timeout waiting for report");
    monitor_handle.abort();

    let received = report_count.load(Ordering::SeqCst);
    assert!(
        received >= 1,
        "should receive at least 1 report within 3s, got {received}"
    );
}

#[tokio::test(start_paused = true)]
async fn test_system_monitor_start_monitoring_spawns_and_aborts() {
    let monitor = SystemMonitor::new(std::time::Duration::from_secs(60));
    let monitor_handle = tokio::spawn(async move {
        monitor
            .start_monitoring(|report| {
                assert_eq!(report.subject.subject_type, HealthSubjectType::System);
            })
            .await
    });

    tokio::time::advance(std::time::Duration::from_millis(50)).await;
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

#[test]
fn test_determine_health_from_metrics_healthy() {
    let metrics = ResourceMetrics {
        cpu_usage: Some(0.5),
        memory_usage: Some(0.5),
        disk_usage: Some(0.5),
        network_io: None,
    };
    let health = SystemInspector::determine_health_from_metrics(&metrics);
    assert!(matches!(health, Health::Healthy));
}

#[test]
fn test_determine_health_from_metrics_critical() {
    let metrics = ResourceMetrics {
        cpu_usage: Some(0.96),
        memory_usage: Some(0.5),
        disk_usage: Some(0.5),
        network_io: None,
    };
    let health = SystemInspector::determine_health_from_metrics(&metrics);
    assert!(matches!(health, Health::Critical { .. }));
}

#[test]
fn test_determine_health_from_metrics_degraded() {
    let metrics = ResourceMetrics {
        cpu_usage: Some(0.85),
        memory_usage: Some(0.5),
        disk_usage: Some(0.5),
        network_io: None,
    };
    let health = SystemInspector::determine_health_from_metrics(&metrics);
    assert!(matches!(health, Health::Degraded { .. }));
}

#[test]
fn test_calculate_uptime_percentage_short() {
    let info = SystemInfo {
        hostname: "test".to_string(),
        kernel_info: KernelInfo {
            name: "Linux".to_string(),
            version: "5.0".to_string(),
            architecture: "x86_64".to_string(),
        },
        cpu_info: CpuInfo {
            model: "Test".to_string(),
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
        uptime: std::time::Duration::from_secs(3600),
        load_average: LoadAverage {
            load_1m: 0.0,
            load_5m: 0.0,
            load_15m: 0.0,
        },
        timestamp: chrono::Utc::now(),
    };
    let pct = SystemInspector::calculate_uptime_percentage(&info);
    assert!(pct > 0.0 && pct < 1.0);
}

#[test]
fn test_determine_health_from_metrics_critical_memory() {
    let metrics = ResourceMetrics {
        cpu_usage: Some(0.5),
        memory_usage: Some(0.96),
        disk_usage: Some(0.5),
        network_io: None,
    };
    let health = SystemInspector::determine_health_from_metrics(&metrics);
    assert!(matches!(health, Health::Critical { .. }));
}

#[test]
fn test_determine_health_from_metrics_critical_disk() {
    let metrics = ResourceMetrics {
        cpu_usage: Some(0.5),
        memory_usage: Some(0.5),
        disk_usage: Some(0.96),
        network_io: None,
    };
    let health = SystemInspector::determine_health_from_metrics(&metrics);
    assert!(matches!(health, Health::Critical { .. }));
}

#[test]
fn test_determine_health_from_metrics_degraded_disk() {
    let metrics = ResourceMetrics {
        cpu_usage: Some(0.5),
        memory_usage: Some(0.5),
        disk_usage: Some(0.85),
        network_io: None,
    };
    let health = SystemInspector::determine_health_from_metrics(&metrics);
    assert!(matches!(health, Health::Degraded { .. }));
}

#[test]
fn test_determine_health_from_metrics_none_fields_treated_as_zero() {
    let metrics = ResourceMetrics {
        cpu_usage: None,
        memory_usage: None,
        disk_usage: None,
        network_io: None,
    };
    let health = SystemInspector::determine_health_from_metrics(&metrics);
    assert!(matches!(health, Health::Healthy));
}

#[test]
fn test_determine_health_from_metrics_degraded_memory_only() {
    let metrics = ResourceMetrics {
        cpu_usage: Some(0.1),
        memory_usage: Some(0.85),
        disk_usage: Some(0.1),
        network_io: None,
    };
    let health = SystemInspector::determine_health_from_metrics(&metrics);
    assert!(matches!(health, Health::Degraded { .. }));
}

#[test]
fn test_determine_health_from_metrics_degraded_cpu_and_memory() {
    let metrics = ResourceMetrics {
        cpu_usage: Some(0.75),
        memory_usage: Some(0.85),
        disk_usage: Some(0.1),
        network_io: None,
    };
    let health = SystemInspector::determine_health_from_metrics(&metrics);
    assert!(matches!(health, Health::Degraded { .. }));
}

#[test]
fn test_cpu_component_health_at_upper_noncritical_before_degraded_band() {
    // > 0.9 is Critical; exactly 0.9 falls through to the > 0.7 Degraded branch
    assert!(matches!(
        SystemInspector::cpu_component_health(Some(0.9)),
        Health::Degraded { .. }
    ));
}

#[test]
fn test_memory_component_health_at_critical_threshold() {
    assert!(matches!(
        SystemInspector::memory_component_health(Some(0.96)),
        Health::Critical { .. }
    ));
}

#[test]
fn test_cpu_component_health_branches() {
    assert!(matches!(
        SystemInspector::cpu_component_health(Some(0.91)),
        Health::Critical { .. }
    ));
    assert!(matches!(
        SystemInspector::cpu_component_health(Some(0.75)),
        Health::Degraded { .. }
    ));
    assert!(matches!(
        SystemInspector::cpu_component_health(Some(0.5)),
        Health::Healthy
    ));
    assert!(matches!(
        SystemInspector::cpu_component_health(None),
        Health::Healthy
    ));
}

#[test]
fn test_memory_component_health_branches() {
    assert!(matches!(
        SystemInspector::memory_component_health(Some(0.96)),
        Health::Critical { .. }
    ));
    assert!(matches!(
        SystemInspector::memory_component_health(Some(0.81)),
        Health::Degraded { .. }
    ));
    assert!(matches!(
        SystemInspector::memory_component_health(Some(0.5)),
        Health::Healthy
    ));
    assert!(matches!(
        SystemInspector::memory_component_health(None),
        Health::Healthy
    ));
}

#[test]
fn test_get_hostname_uses_hostname_env_when_set() {
    use biomeos_test_utils::TestEnvGuard;
    let _g = TestEnvGuard::set("HOSTNAME", "env-host-test");
    let h = SystemInspector::get_hostname().expect("hostname");
    assert_eq!(h, "env-host-test");
}

#[test]
fn test_determine_health_not_critical_when_usage_exactly_at_95_percent() {
    let metrics = ResourceMetrics {
        cpu_usage: Some(0.95),
        memory_usage: Some(0.5),
        disk_usage: Some(0.5),
        network_io: None,
    };
    let health = SystemInspector::determine_health_from_metrics(&metrics);
    assert!(
        matches!(health, Health::Degraded { .. }),
        "strictly above 0.95 is required for Critical"
    );
}

#[test]
fn test_memory_component_health_at_95_percent_is_degraded() {
    assert!(matches!(
        SystemInspector::memory_component_health(Some(0.95)),
        Health::Degraded { .. }
    ));
}

#[test]
fn test_determine_health_critical_requires_strictly_above_95() {
    let metrics = ResourceMetrics {
        cpu_usage: Some(0.950_000_000_1),
        memory_usage: Some(0.1),
        disk_usage: Some(0.1),
        network_io: None,
    };
    let health = SystemInspector::determine_health_from_metrics(&metrics);
    assert!(matches!(health, Health::Critical { .. }));
}

#[test]
fn test_calculate_uptime_percentage_long() {
    let info = SystemInfo {
        hostname: "test".to_string(),
        kernel_info: KernelInfo {
            name: "Linux".to_string(),
            version: "5.0".to_string(),
            architecture: "x86_64".to_string(),
        },
        cpu_info: CpuInfo {
            model: "Test".to_string(),
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
        uptime: std::time::Duration::from_secs(86400 * 2),
        load_average: LoadAverage {
            load_1m: 0.0,
            load_5m: 0.0,
            load_15m: 0.0,
        },
        timestamp: chrono::Utc::now(),
    };
    let pct = SystemInspector::calculate_uptime_percentage(&info);
    assert!((pct - 0.999).abs() < 0.01);
}
