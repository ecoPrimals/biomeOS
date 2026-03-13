// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Disk information and metrics.

use biomeos_types::BiomeResult;

/// Disk information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DiskInfo {
    /// Block device path (e.g. "/dev/sda1")
    pub device: String,
    /// Mount point (e.g. "/", "/home")
    pub mount_point: String,
    /// Filesystem type (e.g. "ext4", "btrfs")
    pub filesystem: String,
    /// Total disk capacity in GiB
    pub total_gb: f64,
    /// Used disk space in GiB
    pub used_gb: f64,
    /// Available disk space in GiB
    pub available_gb: f64,
    /// Disk usage as a percentage (0–100)
    pub usage_percent: f64,
}

/// Get disk information using sysinfo for cross-platform support
pub(crate) async fn get_disk_info() -> BiomeResult<Vec<DiskInfo>> {
    use sysinfo::Disks;

    let disks_info = Disks::new_with_refreshed_list();
    let mut result = Vec::new();

    for disk in &disks_info {
        let total_bytes = disk.total_space();
        let available_bytes = disk.available_space();
        let used_bytes = total_bytes.saturating_sub(available_bytes);

        let total_gb = total_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
        let used_gb = used_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
        let available_gb = available_bytes as f64 / (1024.0 * 1024.0 * 1024.0);

        let usage_percent = if total_gb > 0.0 {
            used_gb / total_gb
        } else {
            0.0
        };

        result.push(DiskInfo {
            device: disk.name().to_string_lossy().to_string(),
            mount_point: disk.mount_point().to_string_lossy().to_string(),
            filesystem: disk.file_system().to_string_lossy().to_string(),
            total_gb,
            used_gb,
            available_gb,
            usage_percent,
        });
    }

    // Ensure at least one disk entry for systems where detection fails
    if result.is_empty() {
        result.push(DiskInfo {
            device: "unknown".to_string(),
            mount_point: "/".to_string(),
            filesystem: "unknown".to_string(),
            total_gb: 0.0,
            used_gb: 0.0,
            available_gb: 0.0,
            usage_percent: 0.0,
        });
    }

    Ok(result)
}

/// Get current disk usage (average across all disks)
pub(crate) async fn get_disk_usage() -> BiomeResult<f64> {
    let disks = get_disk_info().await?;
    if disks.is_empty() {
        return Ok(0.0);
    }

    let total_usage: f64 = disks.iter().map(|d| d.usage_percent).sum();
    Ok(total_usage / disks.len() as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_disk_info() {
        let disk_info = get_disk_info().await.expect("get_disk_info should succeed");

        assert!(!disk_info.is_empty(), "should have at least one disk");
        for disk in &disk_info {
            assert!(!disk.device.is_empty(), "device should not be empty");
            assert!(
                !disk.mount_point.is_empty(),
                "mount_point should not be empty"
            );
            assert!(
                disk.total_gb >= 0.0 && disk.used_gb >= 0.0 && disk.available_gb >= 0.0,
                "disk sizes should be non-negative"
            );
            assert!(
                disk.usage_percent >= 0.0 && disk.usage_percent <= 1.0,
                "usage_percent should be in 0-1 range"
            );
        }
    }

    #[test]
    fn test_disk_info_serialization_roundtrip() {
        let info = DiskInfo {
            device: "/dev/nvme0n1p1".to_string(),
            mount_point: "/".to_string(),
            filesystem: "btrfs".to_string(),
            total_gb: 500.0,
            used_gb: 250.0,
            available_gb: 250.0,
            usage_percent: 0.5,
        };
        let json = serde_json::to_string(&info).expect("serialization should succeed");
        let deserialized: DiskInfo =
            serde_json::from_str(&json).expect("deserialization should succeed");
        assert_eq!(info.device, deserialized.device);
        assert_eq!(info.mount_point, deserialized.mount_point);
        assert!((info.total_gb - deserialized.total_gb).abs() < 0.001);
    }

    #[test]
    fn test_disk_info_zero_total_usage_percent() {
        let info = DiskInfo {
            device: "/dev/zero".to_string(),
            mount_point: "/mnt".to_string(),
            filesystem: "tmpfs".to_string(),
            total_gb: 0.0,
            used_gb: 0.0,
            available_gb: 0.0,
            usage_percent: 0.0,
        };
        let json = serde_json::to_string(&info).expect("serialization should succeed");
        let deserialized: DiskInfo =
            serde_json::from_str(&json).expect("deserialization should succeed");
        assert_eq!(deserialized.usage_percent, 0.0);
    }
}
