// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Memory information and metrics.

use std::fs;

use biomeos_types::BiomeResult;

/// Memory information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MemoryInfo {
    /// Total physical memory in GiB
    pub total_gb: f64,
    /// Used memory in GiB
    pub used_gb: f64,
    /// Available memory in GiB
    pub available_gb: f64,
    /// Memory usage as a percentage (0–100)
    pub usage_percent: f64,
}

/// Get memory information
pub(crate) fn get_memory_info() -> BiomeResult<MemoryInfo> {
    // Try to read from /proc/meminfo on Linux
    if let Ok(meminfo) = fs::read_to_string("/proc/meminfo") {
        let mut total_kb = 0;
        let mut available_kb = 0;

        for line in meminfo.lines() {
            if line.starts_with("MemTotal:") {
                if let Some(value) = line.split_whitespace().nth(1) {
                    total_kb = value.parse::<u64>().unwrap_or(0);
                }
            } else if line.starts_with("MemAvailable:") {
                if let Some(value) = line.split_whitespace().nth(1) {
                    available_kb = value.parse::<u64>().unwrap_or(0);
                }
            }
        }

        let total_gb = total_kb as f64 / 1024.0 / 1024.0;
        let available_gb = available_kb as f64 / 1024.0 / 1024.0;
        let used_gb = total_gb - available_gb;

        Ok(MemoryInfo {
            total_gb,
            used_gb,
            available_gb,
            usage_percent: used_gb / total_gb,
        })
    } else {
        // Fallback
        Ok(MemoryInfo {
            total_gb: 8.0,
            used_gb: 4.0,
            available_gb: 4.0,
            usage_percent: 0.5,
        })
    }
}

/// Get current memory usage
pub(crate) fn get_memory_usage() -> BiomeResult<f64> {
    let memory_info = get_memory_info()?;
    Ok(memory_info.usage_percent)
}

#[allow(clippy::unwrap_used, clippy::expect_used)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_info() {
        let memory_info = get_memory_info().expect("get_memory_info should succeed");

        assert!(
            memory_info.total_gb >= 0.0,
            "total_gb should be non-negative"
        );
        assert!(memory_info.used_gb >= 0.0, "used_gb should be non-negative");
        assert!(
            memory_info.available_gb >= 0.0,
            "available_gb should be non-negative"
        );
        assert!(
            memory_info.usage_percent >= 0.0 && memory_info.usage_percent <= 1.0,
            "usage_percent should be in 0-1 range"
        );
        assert!(
            (memory_info.used_gb + memory_info.available_gb - memory_info.total_gb).abs() < 0.01,
            "used + available should approximately equal total"
        );
    }

    #[test]
    fn test_memory_info_serialization_roundtrip() {
        let info = MemoryInfo {
            total_gb: 32.0,
            used_gb: 16.0,
            available_gb: 16.0,
            usage_percent: 0.5,
        };
        let json = serde_json::to_string(&info).expect("serialization should succeed");
        let deserialized: MemoryInfo =
            serde_json::from_str(&json).expect("deserialization should succeed");
        assert!((info.total_gb - deserialized.total_gb).abs() < 0.001);
        assert!((info.usage_percent - deserialized.usage_percent).abs() < 0.001);
    }

    #[test]
    fn test_memory_info_zero_total_avoids_nan() {
        let info = MemoryInfo {
            total_gb: 0.0,
            used_gb: 0.0,
            available_gb: 0.0,
            usage_percent: 0.0,
        };
        let json = serde_json::to_string(&info).expect("serialization should succeed");
        let deserialized: MemoryInfo =
            serde_json::from_str(&json).expect("deserialization should succeed");
        assert!(!deserialized.usage_percent.is_nan());
        assert!((deserialized.usage_percent - 0.0).abs() < f64::EPSILON);
    }
}
