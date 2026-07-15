// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use crate::*;

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
            addresses: vec!["192.0.2.1".to_string()],
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
