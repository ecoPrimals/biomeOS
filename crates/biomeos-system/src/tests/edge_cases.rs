// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use crate::*;

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
