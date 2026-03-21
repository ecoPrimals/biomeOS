// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Device and primal discovery for device.management capability.
//!
//! Pure Rust discovery via /proc, /sys, and Unix socket queries.
//! NO shell-outs or external commands. TRUE PRIMAL: queries primals
//! for their identity and capabilities at runtime.

use anyhow::Result;
use biomeos_types::JsonRpcRequest;
use biomeos_types::primal_names;
use tracing::info;
use tracing::warn;

use super::types::{Device, DeviceStatus, DeviceType, ManagedPrimal, PrimalStatus};

/// Default interval between /proc/stat reads for CPU usage (100ms).
pub const DEFAULT_CPU_SAMPLE_INTERVAL: std::time::Duration = std::time::Duration::from_millis(100);

/// Discover all devices from the system (GPU, CPU, storage, network).
pub async fn discover_devices() -> Result<Vec<Device>> {
    let mut devices = Vec::new();

    if let Ok(gpus) = discover_gpus().await {
        devices.extend(gpus);
    }

    if let Ok(cpus) = discover_cpus().await {
        devices.extend(cpus);
    }

    if let Ok(storage) = discover_storage().await {
        devices.extend(storage);
    }

    if let Ok(network) = discover_network().await {
        devices.extend(network);
    }

    info!("📱 Discovered {} devices", devices.len());
    Ok(devices)
}

/// Discover GPU devices (pure Rust via /proc/driver/nvidia/)
pub async fn discover_gpus() -> Result<Vec<Device>> {
    let mut gpus = Vec::new();

    if let Ok(mut entries) = tokio::fs::read_dir("/proc/driver/nvidia/gpus").await {
        let mut idx = 0;
        while let Ok(Some(entry)) = entries.next_entry().await {
            let gpu_dir = entry.path();
            let info_path = gpu_dir.join("information");

            if let Ok(info) = tokio::fs::read_to_string(&info_path).await {
                let mut name = format!("NVIDIA GPU {idx}");
                let mut memory_total_mb: u64 = 0;

                for line in info.lines() {
                    if let Some(val) = line.strip_prefix("Model:") {
                        name = val.trim().to_string();
                    }
                }

                let pci_id = entry.file_name().to_string_lossy().to_string();
                let mem_path = format!("/sys/bus/pci/devices/{pci_id}/mem_info_vram_total");
                if let Ok(mem_str) = tokio::fs::read_to_string(&mem_path).await
                    && let Ok(bytes) = mem_str.trim().parse::<u64>()
                {
                    memory_total_mb = bytes / (1024 * 1024);
                }

                gpus.push(Device {
                    id: format!("gpu-{idx}"),
                    name,
                    device_type: DeviceType::Gpu,
                    status: DeviceStatus::Available,
                    resource_usage: 0.0,
                    assigned_to: None,
                    metadata: serde_json::json!({
                        "vendor": "nvidia",
                        "pci_id": pci_id,
                        "memory_total_mb": memory_total_mb,
                    }),
                });
                idx += 1;
            }
        }
    }

    Ok(gpus)
}

/// Discover CPU devices
pub async fn discover_cpus() -> Result<Vec<Device>> {
    let mut cpus = Vec::new();

    if let Ok(cpuinfo) = tokio::fs::read_to_string("/proc/cpuinfo").await {
        let cpu_count = cpuinfo.matches("processor").count();
        let cpu_usage = get_cpu_usage_with_interval(DEFAULT_CPU_SAMPLE_INTERVAL)
            .await
            .unwrap_or(0.0);

        cpus.push(Device {
            id: "cpu-0".to_string(),
            name: format!("CPU ({cpu_count} cores)"),
            device_type: DeviceType::Cpu,
            status: if cpu_usage > 0.9 {
                DeviceStatus::InUse
            } else {
                DeviceStatus::Available
            },
            resource_usage: cpu_usage,
            assigned_to: None,
            metadata: serde_json::json!({
                "cores": cpu_count,
                "usage_percent": (cpu_usage * 100.0) as u32
            }),
        });
    }

    Ok(cpus)
}

/// Get current CPU usage from /proc/stat with configurable sample interval.
pub async fn get_cpu_usage_with_interval(sample_interval: std::time::Duration) -> Result<f64> {
    let stat1 = tokio::fs::read_to_string("/proc/stat").await?;
    tokio::time::sleep(sample_interval).await;
    let stat2 = tokio::fs::read_to_string("/proc/stat").await?;

    let parse_cpu_line = |line: &str| -> Option<(u64, u64)> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 5 || !parts[0].starts_with("cpu") {
            return None;
        }
        let user: u64 = parts[1].parse().ok()?;
        let nice: u64 = parts[2].parse().ok()?;
        let system: u64 = parts[3].parse().ok()?;
        let idle: u64 = parts[4].parse().ok()?;
        Some((user + nice + system, user + nice + system + idle))
    };

    let (active1, total1) = stat1
        .lines()
        .next()
        .and_then(parse_cpu_line)
        .ok_or_else(|| anyhow::anyhow!("Failed to parse /proc/stat"))?;
    let (active2, total2) = stat2
        .lines()
        .next()
        .and_then(parse_cpu_line)
        .ok_or_else(|| anyhow::anyhow!("Failed to parse /proc/stat"))?;

    let active_diff = active2.saturating_sub(active1);
    let total_diff = total2.saturating_sub(total1);

    if total_diff == 0 {
        return Ok(0.0);
    }

    Ok(active_diff as f64 / total_diff as f64)
}

/// Discover storage devices (pure Rust via /proc/mounts + statvfs)
pub async fn discover_storage() -> Result<Vec<Device>> {
    let mut storage = Vec::new();

    if let Ok(mounts) = tokio::fs::read_to_string("/proc/mounts").await {
        for (idx, line) in mounts.lines().enumerate() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let source = parts[0];
                let mount_point = parts[1];

                if source.starts_with("/dev/") {
                    let (size_str, used_str, usage) = statvfs_info(mount_point)
                        .unwrap_or_else(|| ("unknown".to_string(), "unknown".to_string(), 0.0));

                    storage.push(Device {
                        id: format!("storage-{idx}"),
                        name: mount_point.to_string(),
                        device_type: DeviceType::Storage,
                        status: DeviceStatus::Available,
                        resource_usage: usage,
                        assigned_to: None,
                        metadata: serde_json::json!({
                            "source": source,
                            "size": size_str,
                            "used": used_str,
                        }),
                    });
                }
            }
        }
    }

    Ok(storage)
}

/// Get filesystem stats via rustix::fs::statvfs (pure Rust, no libc)
fn statvfs_info(path: &str) -> Option<(String, String, f64)> {
    #[cfg(unix)]
    {
        let stat = rustix::fs::statvfs(path).ok()?;

        let block_size = stat.f_frsize;
        let total = stat.f_blocks * block_size;
        let available = stat.f_bavail * block_size;
        let used = total.saturating_sub(available);
        let usage = if total > 0 {
            used as f64 / total as f64
        } else {
            0.0
        };

        Some((human_size(total), human_size(used), usage))
    }
    #[cfg(not(unix))]
    None
}

/// Format bytes as human-readable size
fn human_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "K", "M", "G", "T"];
    let mut size = bytes as f64;
    for unit in UNITS {
        if size < 1024.0 {
            return format!("{size:.1}{unit}");
        }
        size /= 1024.0;
    }
    format!("{size:.1}P")
}

/// Discover network interfaces (pure Rust via /sys/class/net/)
pub async fn discover_network() -> Result<Vec<Device>> {
    let mut network = Vec::new();

    if let Ok(mut entries) = tokio::fs::read_dir("/sys/class/net").await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            let name = entry.file_name().to_string_lossy().to_string();

            if name == "lo" {
                continue;
            }

            let operstate_path = format!("/sys/class/net/{name}/operstate");
            let status = match tokio::fs::read_to_string(&operstate_path).await {
                Ok(state) if state.trim() == "up" => DeviceStatus::InUse,
                _ => DeviceStatus::Offline,
            };

            network.push(Device {
                id: format!("net-{name}"),
                name,
                device_type: DeviceType::Network,
                status,
                resource_usage: 0.0,
                assigned_to: None,
                metadata: serde_json::json!({}),
            });
        }
    }

    Ok(network)
}

/// Discover running primals via Unix socket scanning
pub async fn discover_primals() -> Result<Vec<ManagedPrimal>> {
    let mut primals = Vec::new();

    let uid = std::env::var("UID").unwrap_or_else(|_| "1000".to_string());
    let socket_dir = format!("/run/user/{uid}");

    if let Ok(mut entries) = tokio::fs::read_dir(&socket_dir).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            if let Some(name) = entry.file_name().to_str()
                && std::path::Path::new(name)
                    .extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("sock"))
            {
                let socket_path = format!("{socket_dir}/{name}");

                let primal_name = query_primal_identity(&socket_path).await;
                let primal_id = primal_name.to_lowercase();

                let (health, load, status) = probe_primal_health(&socket_path).await;

                let capabilities = get_primal_capabilities(&socket_path).await;

                primals.push(ManagedPrimal {
                    id: primal_id.clone(),
                    name: primal_name,
                    status,
                    health,
                    load,
                    capabilities,
                    assigned_devices: vec![],
                    metadata: serde_json::json!({
                        "socket": name,
                        "discovered_at": chrono::Utc::now().to_rfc3339()
                    }),
                });
            }
        }
    }

    primals.push(ManagedPrimal {
        id: primal_names::BIOMEOS.to_string(),
        name: primal_names::display::BIOMEOS.to_string(),
        status: PrimalStatus::Healthy,
        health: 1.0,
        load: 0.1,
        capabilities: vec!["orchestration".to_string(), "device.management".to_string()],
        assigned_devices: vec![],
        metadata: serde_json::json!({
            "version": env!("CARGO_PKG_VERSION"),
            "self": true
        }),
    });

    info!("🔍 Discovered {} primals", primals.len());
    Ok(primals)
}

/// Query a primal for its identity (TRUE PRIMAL discovery!)
pub async fn query_primal_identity(socket_path: &str) -> String {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    use tokio::net::UnixStream;

    let stream = match UnixStream::connect(socket_path).await {
        Ok(s) => s,
        Err(e) => {
            warn!("Failed to connect to {}: {}", socket_path, e);
            return "unknown".to_string();
        }
    };

    let request = JsonRpcRequest::new("identity.get", serde_json::json!({}));

    let request_str = match serde_json::to_string(&request) {
        Ok(s) => s + "\n",
        Err(e) => {
            warn!("Failed to serialize identity request: {}", e);
            return "unknown".to_string();
        }
    };
    let (read, mut write) = stream.into_split();

    if let Err(e) = write.write_all(request_str.as_bytes()).await {
        warn!("Failed to send identity query: {}", e);
        return "unknown".to_string();
    }

    let mut reader = BufReader::new(read);
    let mut response_line = String::new();

    match tokio::time::timeout(
        std::time::Duration::from_secs(2),
        reader.read_line(&mut response_line),
    )
    .await
    {
        Ok(Ok(_)) => {
            if let Ok(response) = serde_json::from_str::<serde_json::Value>(&response_line)
                && let Some(name) = response["result"]["name"].as_str()
            {
                return name.to_string();
            }
            warn!("Invalid identity response from {}", socket_path);
            "unknown".to_string()
        }
        Ok(Err(e)) => {
            warn!("Failed to read identity response: {}", e);
            "unknown".to_string()
        }
        Err(_) => {
            warn!("Identity query timeout for {}", socket_path);
            "unknown".to_string()
        }
    }
}

/// Probe a primal for health metrics
pub async fn probe_primal_health(socket_path: &str) -> (f64, f64, PrimalStatus) {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    use tokio::net::UnixStream;

    let Ok(stream) = UnixStream::connect(socket_path).await else {
        return (0.0, 1.0, PrimalStatus::Offline);
    };

    let request = JsonRpcRequest::new("health.check", serde_json::json!({}));

    let request_str = match serde_json::to_string(&request) {
        Ok(s) => s + "\n",
        Err(e) => {
            warn!("Failed to serialize health request: {}", e);
            return (0.0, 1.0, PrimalStatus::Degraded);
        }
    };
    let (read, mut write) = stream.into_split();

    if write.write_all(request_str.as_bytes()).await.is_err() {
        return (0.0, 1.0, PrimalStatus::Degraded);
    }

    let mut reader = BufReader::new(read);
    let mut response_line = String::new();

    match tokio::time::timeout(
        std::time::Duration::from_secs(2),
        reader.read_line(&mut response_line),
    )
    .await
    {
        Ok(Ok(_)) => {
            if let Ok(response) = serde_json::from_str::<serde_json::Value>(&response_line) {
                let health = response["result"]["health"].as_f64().unwrap_or(1.0);
                let load = response["result"]["load"].as_f64().unwrap_or(0.0);
                let status_str = response["result"]["status"].as_str().unwrap_or("healthy");

                let status = match status_str {
                    "degraded" | "unhealthy" => PrimalStatus::Degraded,
                    "offline" => PrimalStatus::Offline,
                    _ => PrimalStatus::Healthy,
                };

                return (health, load, status);
            }
            (0.8, 0.2, PrimalStatus::Degraded)
        }
        _ => (0.0, 1.0, PrimalStatus::Degraded),
    }
}

/// Get capabilities for a primal via capability-based discovery
pub async fn get_primal_capabilities(socket_path: &str) -> Vec<String> {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    use tokio::net::UnixStream;

    let stream = match UnixStream::connect(socket_path).await {
        Ok(s) => s,
        Err(e) => {
            warn!(
                "Failed to connect to {} for capabilities: {}",
                socket_path, e
            );
            return vec![];
        }
    };

    let request = JsonRpcRequest::new("capabilities.list", serde_json::json!({}));

    let request_str = match serde_json::to_string(&request) {
        Ok(s) => s + "\n",
        Err(e) => {
            warn!("Failed to serialize capabilities request: {}", e);
            return vec![];
        }
    };
    let (read, mut write) = stream.into_split();

    if let Err(e) = write.write_all(request_str.as_bytes()).await {
        warn!("Failed to send capabilities query: {}", e);
        return vec![];
    }

    let mut reader = BufReader::new(read);
    let mut response_line = String::new();

    match tokio::time::timeout(
        std::time::Duration::from_secs(2),
        reader.read_line(&mut response_line),
    )
    .await
    {
        Ok(Ok(_)) => {
            if let Ok(response) = serde_json::from_str::<serde_json::Value>(&response_line)
                && let Some(caps) = response["result"]["capabilities"].as_array()
            {
                return caps
                    .iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect();
            }
            warn!("Invalid capabilities response from {}", socket_path);
            vec![]
        }
        Ok(Err(e)) => {
            warn!("Failed to read capabilities response: {}", e);
            vec![]
        }
        Err(_) => {
            warn!("Capabilities query timeout for {}", socket_path);
            vec![]
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
mod discovery_tests {
    use super::*;

    #[test]
    fn test_default_cpu_sample_interval() {
        assert_eq!(DEFAULT_CPU_SAMPLE_INTERVAL.as_millis(), 100);
    }

    #[tokio::test]
    async fn test_discover_devices_returns_vec() {
        let devices = discover_devices().await.expect("discover_devices");
        // Linux CI: at least CPU from /proc/cpuinfo when readable
        let _ = devices;
    }

    #[tokio::test]
    async fn test_get_cpu_usage_with_interval_zero_total_diff() {
        // Uses real /proc/stat — should return Ok in [0,1] on Linux
        let usage = get_cpu_usage_with_interval(std::time::Duration::from_millis(50))
            .await
            .expect("cpu usage");
        assert!((0.0..=1.0).contains(&usage));
    }

    #[tokio::test]
    async fn test_discover_network_skips_loopback() {
        let nets = discover_network().await.expect("net");
        assert!(!nets.iter().any(|d| d.id == "net-lo"));
    }

    #[test]
    fn test_human_size_units_and_petabyte_fallback() {
        assert_eq!(human_size(0), "0.0B");
        assert_eq!(human_size(100), "100.0B");
        assert_eq!(human_size(1023), "1023.0B");
        assert_eq!(human_size(1024), "1.0K");
        assert_eq!(human_size(1024 * 1024), "1.0M");
        assert_eq!(human_size(1024_u64.pow(3)), "1.0G");
        assert_eq!(human_size(1024_u64.pow(4)), "1.0T");
        // Exhaust K/M/G/T loop → P branch
        let pb = 1024_u64.pow(5) * 1024;
        assert!(human_size(pb).ends_with('P'));
    }

    #[cfg(unix)]
    #[test]
    fn test_statvfs_info_directory_and_missing_path() {
        let temp = tempfile::tempdir().expect("tempdir");
        let info = statvfs_info(temp.path().to_str().expect("utf8 path"));
        assert!(info.is_some());
        let (size, used, usage) = info.expect("statvfs tempdir");
        assert!(!size.is_empty());
        assert!(!used.is_empty());
        assert!((0.0..=1.0).contains(&usage));

        assert!(statvfs_info("/nonexistent/path/for/statvfs/xxxxxxxx").is_none());
    }
}
