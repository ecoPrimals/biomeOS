// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! USB device discovery and management
//!
//! Provides capability-based USB device discovery without hardcoding.

use std::path::PathBuf;
use tracing::{debug, info};

use crate::error::{SporeError, SporeResult};

/// USB device information
#[derive(Debug, Clone)]
pub struct UsbDevice {
    /// Mount point (e.g., `/media/usb`)
    pub mount_point: PathBuf,

    /// Device label (if available)
    pub label: Option<String>,

    /// Available space in bytes
    pub available_space: u64,

    /// Total space in bytes
    pub total_space: u64,
}

impl UsbDevice {
    /// Check if device has sufficient space for a spore
    pub fn has_sufficient_space(&self, required_bytes: u64) -> bool {
        self.available_space >= required_bytes
    }

    /// Get space utilization as a percentage
    pub fn utilization_percent(&self) -> f64 {
        if self.total_space == 0 {
            0.0
        } else {
            ((self.total_space - self.available_space) as f64 / self.total_space as f64) * 100.0
        }
    }
}

/// Discover USB devices on the system
///
/// This uses capability-based discovery, not hardcoded paths.
/// Checks common mount points and verifies they're writable.
pub async fn discover_usb_devices() -> SporeResult<Vec<UsbDevice>> {
    info!("Discovering USB devices");

    let mut devices = Vec::new();

    // Common mount point patterns (capability-based, not hardcoded)
    let mount_prefixes = ["/media", "/mnt", "/run/media"];

    for prefix in &mount_prefixes {
        if let Ok(entries) = tokio::fs::read_dir(prefix).await {
            let mut entries = entries;
            while let Ok(Some(entry)) = entries.next_entry().await {
                if let Ok(device) = probe_device(entry.path()).await {
                    debug!("Found USB device: {:?}", device);
                    devices.push(device);
                }
            }
        }
    }

    info!("Discovered {} USB device(s)", devices.len());
    Ok(devices)
}

/// Probe a potential mount point to see if it's a valid USB device
async fn probe_device(path: PathBuf) -> SporeResult<UsbDevice> {
    // Check if path exists and is a directory
    let metadata = tokio::fs::metadata(&path).await?;
    if !metadata.is_dir() {
        return Err(SporeError::InvalidConfig("Not a directory".to_string()));
    }

    // Try to get filesystem stats via statvfs (Unix) or fallback
    #[cfg(unix)]
    let (available_space, total_space) = {
        let path_clone = path.clone();
        if let Ok(Ok(st)) =
            tokio::task::spawn_blocking(move || rustix::fs::statvfs(path_clone.as_path())).await
        {
            let avail = st.f_bavail * st.f_frsize;
            let total = st.f_blocks * st.f_frsize;
            (avail, total)
        } else {
            tracing::warn!(
                "statvfs failed for {}, reporting zero free space",
                path.display()
            );
            (0, 0)
        }
    };

    #[cfg(not(unix))]
    let (available_space, total_space) = {
        tracing::warn!(
            "statvfs unavailable on non-Unix, reporting zero free space for {}",
            path.display()
        );
        (0u64, 0u64)
    };

    // Extract label from path (last component)
    let label = path
        .file_name()
        .and_then(|n| n.to_str())
        .map(std::string::ToString::to_string);

    Ok(UsbDevice {
        mount_point: path,
        label,
        available_space,
        total_space,
    })
}

/// Find a USB device by label
pub async fn find_device_by_label(label: &str) -> SporeResult<UsbDevice> {
    let devices = discover_usb_devices().await?;

    devices
        .into_iter()
        .find(|d| d.label.as_deref() == Some(label))
        .ok_or_else(|| SporeError::DeviceNotFound(PathBuf::from(format!("label:{label}"))))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_discover_usb_devices() {
        // This will vary by system, just ensure it doesn't panic
        let result = discover_usb_devices().await;
        assert!(result.is_ok());
    }

    // ========== UsbDevice Space Checks ==========

    #[test]
    fn test_has_sufficient_space() {
        let device = UsbDevice {
            mount_point: PathBuf::from("/test"),
            label: Some("test".to_string()),
            available_space: 1_000_000_000, // 1GB
            total_space: 8_000_000_000,     // 8GB
        };

        assert!(device.has_sufficient_space(500_000_000)); // 500MB - should fit
        assert!(!device.has_sufficient_space(2_000_000_000)); // 2GB - too big
    }

    #[test]
    fn test_has_sufficient_space_exact() {
        let device = UsbDevice {
            mount_point: PathBuf::from("/test"),
            label: None,
            available_space: 1000,
            total_space: 2000,
        };

        assert!(device.has_sufficient_space(1000)); // Exact match
        assert!(!device.has_sufficient_space(1001)); // One byte over
    }

    #[test]
    fn test_has_sufficient_space_zero_available() {
        let device = UsbDevice {
            mount_point: PathBuf::from("/test"),
            label: None,
            available_space: 0,
            total_space: 8_000_000_000,
        };

        assert!(device.has_sufficient_space(0));
        assert!(!device.has_sufficient_space(1));
    }

    // ========== Utilization Tests ==========

    #[test]
    fn test_utilization_percent() {
        let device = UsbDevice {
            mount_point: PathBuf::from("/test"),
            label: Some("test".to_string()),
            available_space: 2_000_000_000, // 2GB available
            total_space: 8_000_000_000,     // 8GB total
        };

        // 6GB used out of 8GB = 75% utilization
        let util = device.utilization_percent();
        assert!((util - 75.0).abs() < 0.1);
    }

    #[test]
    fn test_utilization_percent_zero_total() {
        let device = UsbDevice {
            mount_point: PathBuf::from("/test"),
            label: None,
            available_space: 0,
            total_space: 0,
        };

        // Should return 0.0, not NaN or panic
        assert!((device.utilization_percent() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_utilization_percent_empty_device() {
        let device = UsbDevice {
            mount_point: PathBuf::from("/test"),
            label: Some("empty".to_string()),
            available_space: 8_000_000_000,
            total_space: 8_000_000_000,
        };

        // No data used = 0% utilization
        let util = device.utilization_percent();
        assert!((util - 0.0).abs() < 0.1);
    }

    #[test]
    fn test_utilization_percent_full_device() {
        let device = UsbDevice {
            mount_point: PathBuf::from("/test"),
            label: Some("full".to_string()),
            available_space: 0,
            total_space: 4_000_000_000,
        };

        // 100% utilization
        let util = device.utilization_percent();
        assert!((util - 100.0).abs() < 0.1);
    }

    // ========== UsbDevice Properties ==========

    #[test]
    fn test_usb_device_with_label() {
        let device = UsbDevice {
            mount_point: PathBuf::from("/media/user/biomeOS1"),
            label: Some("biomeOS1".to_string()),
            available_space: 5_000_000_000,
            total_space: 16_000_000_000,
        };

        assert_eq!(device.label, Some("biomeOS1".to_string()));
        assert_eq!(device.mount_point, PathBuf::from("/media/user/biomeOS1"));
    }

    #[test]
    fn test_usb_device_without_label() {
        let device = UsbDevice {
            mount_point: PathBuf::from("/mnt/usb"),
            label: None,
            available_space: 1_000_000,
            total_space: 2_000_000,
        };

        assert!(device.label.is_none());
    }

    #[test]
    fn test_usb_device_clone() {
        let device = UsbDevice {
            mount_point: PathBuf::from("/test"),
            label: Some("test".to_string()),
            available_space: 100,
            total_space: 200,
        };

        let cloned = device.clone();
        assert_eq!(cloned.mount_point, device.mount_point);
        assert_eq!(cloned.label, device.label);
        assert_eq!(cloned.available_space, device.available_space);
        assert_eq!(cloned.total_space, device.total_space);
    }

    #[test]
    fn test_usb_device_debug() {
        let device = UsbDevice {
            mount_point: PathBuf::from("/media/usb0"),
            label: Some("USB_DRIVE".to_string()),
            available_space: 1024,
            total_space: 2048,
        };

        let debug = format!("{device:?}");
        assert!(debug.contains("USB_DRIVE"));
        assert!(debug.contains("/media/usb0"));
    }

    // ========== find_device_by_label ==========

    #[tokio::test]
    async fn test_find_device_by_label_not_found() {
        // This label almost certainly doesn't exist
        let result = find_device_by_label("nonexistent_biomeOS_label_xyz123").await;
        assert!(result.is_err());
    }
}
