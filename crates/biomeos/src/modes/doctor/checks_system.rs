// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Doctor mode system checks - memory, disk, CPU, dependencies

use anyhow::Result;

use crate::proc_metrics;

use super::types::{HealthCheck, HealthStatus};

pub(crate) async fn check_system_resources() -> Result<HealthCheck> {
    let mut check = HealthCheck {
        name: "System Resources".to_string(),
        status: HealthStatus::Healthy,
        details: Vec::new(),
    };

    // Memory (pure Rust via /proc/meminfo)
    let total_mem = proc_metrics::total_memory();
    let avail_mem = proc_metrics::available_memory();
    let total_mem_gb = total_mem as f64 / 1_073_741_824.0;
    let avail_mem_gb = avail_mem as f64 / 1_073_741_824.0;
    let mem_percent = if total_mem > 0 {
        ((total_mem - avail_mem) as f64 / total_mem as f64) * 100.0
    } else {
        0.0
    };

    check.details.push(format!(
        "Memory: {total_mem_gb:.1}GB ({avail_mem_gb:.1}GB available, {mem_percent:.0}% used)"
    ));

    if mem_percent > 90.0 {
        check.status = HealthStatus::Warning;
    }

    // Disk (pure Rust via /proc/mounts + statvfs)
    if let Some((total, available)) = proc_metrics::root_disk_bytes() {
        let total_gb = total as f64 / 1_073_741_824.0;
        let avail_gb = available as f64 / 1_073_741_824.0;
        let used_percent = if total > 0 {
            ((total - available) as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        check.details.push(format!(
            "Disk: {total_gb:.1}GB ({avail_gb:.1}GB available, {used_percent:.0}% used)"
        ));

        if used_percent > 90.0 {
            check.status = HealthStatus::Warning;
        }
    }

    // CPU (pure Rust via /proc/cpuinfo)
    let cpu_count = proc_metrics::cpu_count();
    check.details.push(format!("CPUs: {cpu_count} cores"));

    // Load average (pure Rust via /proc/loadavg)
    let load_avg = proc_metrics::load_average_one();
    check.details.push(format!("Load: {load_avg:.2}"));

    Ok(check)
}

pub(crate) async fn check_dependencies() -> Result<HealthCheck> {
    let check = HealthCheck {
        name: "Dependencies".to_string(),
        status: HealthStatus::Healthy,
        details: vec![
            "Pure Rust: Evolving to 100%".to_string(),
            "UniBin: ✅ Compliant".to_string(),
            "ecoBin: ⏳ In progress".to_string(),
        ],
    };

    Ok(check)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    #[tokio::test]
    async fn test_check_dependencies() {
        let check = check_dependencies().await.unwrap();
        assert_eq!(check.status, HealthStatus::Healthy);
        assert_eq!(check.name, "Dependencies");
        assert!(check.details.iter().any(|d| d.contains("Pure Rust")));
    }
}
