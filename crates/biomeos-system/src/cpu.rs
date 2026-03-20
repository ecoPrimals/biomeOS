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

/// Get current CPU usage via /proc/stat (pure Rust - ecoBin v3).
///
/// Reads /proc/stat twice with a short delay and computes usage from jiffies.
pub(crate) async fn get_cpu_usage() -> BiomeResult<f64> {
    #[cfg(not(target_os = "linux"))]
    {
        let _ = tokio::time::Duration::from_millis(1);
        return Ok(0.0);
    }

    #[cfg(target_os = "linux")]
    {
        let first = read_cpu_jiffies().ok_or_else(|| {
            biomeos_types::BiomeError::internal_error(
                "Cannot read /proc/stat",
                Some("CPU_READ_FAILED"),
            )
        })?;
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
        let second = read_cpu_jiffies().ok_or_else(|| {
            biomeos_types::BiomeError::internal_error(
                "Cannot read /proc/stat (second read)",
                Some("CPU_READ_FAILED"),
            )
        })?;

        let total_delta = second.total.saturating_sub(first.total);
        let idle_delta = second.idle.saturating_sub(first.idle);

        if total_delta == 0 {
            return Ok(0.0);
        }

        // u64->f64: precision loss acceptable for percentage metrics
        let usage = 1.0 - (idle_delta as f64 / total_delta as f64);
        Ok(usage.clamp(0.0, 1.0))
    }
}

/// Jiffies from /proc/stat first line (cpu ...)
#[cfg(target_os = "linux")]
struct CpuJiffies {
    total: u64,
    idle: u64,
}

#[cfg(target_os = "linux")]
fn read_cpu_jiffies() -> Option<CpuJiffies> {
    let stat = fs::read_to_string("/proc/stat").ok()?;
    let line = stat.lines().next()?;
    // cpu  user nice system idle iowait irq softirq steal guest guest_nice
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 5 || parts[0] != "cpu" {
        return None;
    }
    let user: u64 = parts.get(1)?.parse().ok()?;
    let nice: u64 = parts.get(2)?.parse().ok()?;
    let system: u64 = parts.get(3)?.parse().ok()?;
    let idle: u64 = parts.get(4)?.parse().ok()?;
    let iowait: u64 = parts.get(5).and_then(|s| s.parse().ok()).unwrap_or(0);
    let irq: u64 = parts.get(6).and_then(|s| s.parse().ok()).unwrap_or(0);
    let softirq: u64 = parts.get(7).and_then(|s| s.parse().ok()).unwrap_or(0);
    let steal: u64 = parts.get(8).and_then(|s| s.parse().ok()).unwrap_or(0);
    let guest: u64 = parts.get(9).and_then(|s| s.parse().ok()).unwrap_or(0);
    let guest_nice: u64 = parts.get(10).and_then(|s| s.parse().ok()).unwrap_or(0);

    let total = user + nice + system + idle + iowait + irq + softirq + steal + guest + guest_nice;
    Some(CpuJiffies {
        total,
        idle: idle + iowait,
    })
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

#[allow(clippy::unwrap_used, clippy::expect_used)]
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
        assert!((load.load_1m - cloned.load_1m).abs() < f64::EPSILON);
    }
}
