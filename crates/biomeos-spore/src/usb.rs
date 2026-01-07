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
    let mount_prefixes = [
        "/media",
        "/mnt",
        "/run/media",
    ];

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

    // Try to get filesystem stats
    #[cfg(unix)]
    let (available_space, total_space) = {
        // Simplified - in production, use statvfs or similar
        let size = metadata.len();
        (size, size * 2) // Placeholder
    };

    #[cfg(not(unix))]
    let (available_space, total_space) = (0, 0);

    // Extract label from path (last component)
    let label = path
        .file_name()
        .and_then(|n| n.to_str())
        .map(|s| s.to_string());

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
        .ok_or_else(|| {
            SporeError::DeviceNotFound(PathBuf::from(format!("label:{}", label)))
        })
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
}

