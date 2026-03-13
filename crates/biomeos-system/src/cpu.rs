// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! CPU information and metrics.

use std::fs;

use biomeos_types::BiomeResult;

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

/// Get CPU information
pub(crate) fn get_cpu_info() -> BiomeResult<CpuInfo> {
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

/// Get current CPU usage using sysinfo
pub(crate) async fn get_cpu_usage() -> BiomeResult<f64> {
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

/// Get load average
pub(crate) fn get_load_average() -> BiomeResult<LoadAverage> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_info() {
        let cpu_info = get_cpu_info().expect("get_cpu_info should succeed");

        assert!(cpu_info.cores >= 1, "should have at least 1 core");
        assert_eq!(
            cpu_info.architecture,
            std::env::consts::ARCH,
            "architecture should match target"
        );
    }

    #[test]
    fn test_load_average() {
        let load_avg = get_load_average().expect("get_load_average should succeed");

        assert!(load_avg.load_1m >= 0.0, "load_1m should be non-negative");
        assert!(load_avg.load_5m >= 0.0, "load_5m should be non-negative");
        assert!(load_avg.load_15m >= 0.0, "load_15m should be non-negative");
    }

    #[test]
    fn test_cpu_info_serialization_roundtrip() {
        let info = CpuInfo {
            model: "Intel Core i7".to_string(),
            cores: 8,
            architecture: "x86_64".to_string(),
        };
        let json = serde_json::to_string(&info).expect("serialization should succeed");
        let deserialized: CpuInfo =
            serde_json::from_str(&json).expect("deserialization should succeed");
        assert_eq!(info.model, deserialized.model);
        assert_eq!(info.cores, deserialized.cores);
    }

    #[test]
    fn test_load_average_serialization_roundtrip() {
        let info = LoadAverage {
            load_1m: 2.5,
            load_5m: 2.0,
            load_15m: 1.5,
        };
        let json = serde_json::to_string(&info).expect("serialization should succeed");
        let deserialized: LoadAverage =
            serde_json::from_str(&json).expect("deserialization should succeed");
        assert!((info.load_1m - deserialized.load_1m).abs() < 0.001);
        assert!((info.load_5m - deserialized.load_5m).abs() < 0.001);
        assert!((info.load_15m - deserialized.load_15m).abs() < 0.001);
    }

    #[test]
    fn test_clone_load_average() {
        let load = LoadAverage {
            load_1m: 1.0,
            load_5m: 1.0,
            load_15m: 1.0,
        };
        let cloned = load.clone();
        assert_eq!(load.load_1m, cloned.load_1m);
    }
}
