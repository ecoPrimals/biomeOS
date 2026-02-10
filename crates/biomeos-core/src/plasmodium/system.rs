//! System probing — pure Rust OS-level information gathering.
//!
//! Reads `/proc` and `/sys` for GPU, RAM, CPU, and load information
//! without external dependencies (no `nvidia-smi`, no `sysinfo` crate).

use super::types::{ComputeInfo, GpuInfo};

/// Query local compute capabilities (GPU, RAM, CPU)
///
/// Pure Rust: reads `/proc/driver/nvidia/gpus/` for GPU info,
/// `/proc/meminfo` for RAM, `/proc/cpuinfo` for CPU cores.
pub(crate) async fn query_local_compute(local_gate_id: &str) -> ComputeInfo {
    let mut gpus = Vec::new();

    // Read NVIDIA GPU info from /proc/driver/nvidia/gpus/ (pure Rust, no nvidia-smi)
    if let Ok(mut entries) = tokio::fs::read_dir("/proc/driver/nvidia/gpus").await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            let info_path = entry.path().join("information");
            if let Ok(info) = tokio::fs::read_to_string(&info_path).await {
                let mut name = "NVIDIA GPU".to_string();
                for line in info.lines() {
                    if let Some(val) = line.strip_prefix("Model:") {
                        name = val.trim().to_string();
                    }
                }

                // Try PCI sysfs for VRAM
                let pci_id = entry.file_name().to_string_lossy().to_string();
                let mem_path = format!("/sys/bus/pci/devices/{pci_id}/mem_info_vram_total");
                let vram_mb = if let Ok(mem_str) = tokio::fs::read_to_string(&mem_path).await {
                    mem_str.trim().parse::<u64>().unwrap_or(0) / (1024 * 1024)
                } else {
                    0
                };

                gpus.push(GpuInfo {
                    name,
                    vram_mb,
                    gate_id: local_gate_id.to_string(),
                });
            }
        }
    }

    ComputeInfo {
        gpus,
        ram_gb: get_system_ram_gb(),
        cpu_cores: num_cpus(),
    }
}

/// Read system load from `/proc/loadavg`, normalized to [0.0, 1.0] by CPU count.
pub(crate) fn get_system_load() -> f64 {
    std::fs::read_to_string("/proc/loadavg")
        .ok()
        .and_then(|s| {
            s.split_whitespace()
                .next()
                .and_then(|load| load.parse::<f64>().ok())
        })
        .map(|load_1m| {
            let cores = num_cpus() as f64;
            if cores > 0.0 {
                (load_1m / cores).min(1.0)
            } else {
                0.0
            }
        })
        .unwrap_or(0.0)
}

/// Read total system RAM from `/proc/meminfo`, in gigabytes.
pub(crate) fn get_system_ram_gb() -> u64 {
    std::fs::read_to_string("/proc/meminfo")
        .ok()
        .and_then(|s| {
            s.lines()
                .find(|l| l.starts_with("MemTotal:"))
                .and_then(|l| {
                    l.split_whitespace()
                        .nth(1)
                        .and_then(|kb| kb.parse::<u64>().ok())
                })
        })
        .map(|kb| kb / 1_048_576) // KB to GB
        .unwrap_or(0)
}

/// CPU core count from `/proc/cpuinfo` (no external dependency).
pub(crate) fn num_cpus() -> usize {
    std::fs::read_to_string("/proc/cpuinfo")
        .ok()
        .map(|s| s.lines().filter(|l| l.starts_with("processor")).count())
        .unwrap_or(1)
}
