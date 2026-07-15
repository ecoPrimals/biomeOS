// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use crate::{CpuInfo, KernelInfo, LoadAverage, MemoryInfo, SystemInfo, SystemInspector};

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
