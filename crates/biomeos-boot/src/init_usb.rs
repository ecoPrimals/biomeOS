// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! USB block device discovery for BiomeOS init.
//!
//! Enumerates removable block devices via Linux sysfs and probes partitions for
//! the BiomeOS spore marker file.

use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};

/// Marker file indicating a BiomeOS live spore on removable media.
const BIOMEOS_USB_MARKER: &str = ".biomeos-spore";

/// Temporary mount point used while probing candidate partitions.
const USB_PROBE_MOUNT: &str = "/run/biomeos-usb-probe";

/// Detect a BiomeOS USB block device by scanning sysfs for removable media.
///
/// On Linux, enumerates `/sys/block/` for removable devices, probes each
/// partition for the `.biomeos-spore` marker, and returns the first matching
/// block device path (e.g. `/dev/sdb1`). Returns `None` on non-Linux hosts or
/// when sysfs is unavailable (containers, WSL, etc.).
#[must_use]
pub fn detect_biomeos_usb_device() -> Option<PathBuf> {
    #[cfg(target_os = "linux")]
    {
        detect_biomeos_usb_at(Path::new("/sys/block"))
    }

    #[cfg(not(target_os = "linux"))]
    {
        None
    }
}

#[cfg(target_os = "linux")]
fn detect_biomeos_usb_at(block_dir: &Path) -> Option<PathBuf> {
    if !block_dir.exists() {
        debug!(
            "sysfs block directory unavailable at {}",
            block_dir.display()
        );
        return None;
    }

    let entries = match std::fs::read_dir(block_dir) {
        Ok(entries) => entries,
        Err(e) => {
            warn!("Failed to read {}: {e}", block_dir.display());
            return None;
        }
    };

    for entry in entries.flatten() {
        let block_name = entry.file_name();
        let block_name = block_name.to_string_lossy();

        if !is_removable_block_device(&entry.path()) {
            continue;
        }

        debug!("Found removable block device: {block_name}");

        let candidates = block_device_candidates(&entry.path(), &block_name);
        for device in candidates {
            if partition_has_biomeos_marker(&device) {
                info!(
                    "BiomeOS marker found on removable device {}",
                    device.display()
                );
                return Some(device);
            }
        }
    }

    None
}

#[cfg(target_os = "linux")]
fn is_removable_block_device(block_sysfs: &Path) -> bool {
    let removable_path = block_sysfs.join("removable");
    match std::fs::read_to_string(&removable_path) {
        Ok(value) => value.trim() == "1",
        Err(_) => false,
    }
}

#[cfg(target_os = "linux")]
fn block_device_candidates(block_sysfs: &Path, block_name: &str) -> Vec<PathBuf> {
    let mut candidates = Vec::new();

    if let Ok(entries) = std::fs::read_dir(block_sysfs) {
        for entry in entries.flatten() {
            let partition_name = entry.file_name();
            let partition_name = partition_name.to_string_lossy();

            let partition_sysfs = entry.path();
            if partition_sysfs.join("partition").is_file() {
                candidates.push(PathBuf::from(format!("/dev/{partition_name}")));
            }
        }
    }

    if candidates.is_empty() {
        candidates.push(PathBuf::from(format!("/dev/{block_name}")));
    }

    candidates
}

#[cfg(target_os = "linux")]
fn partition_has_biomeos_marker(device: &Path) -> bool {
    if let Some(mount_point) = find_mount_point(device) {
        return mount_point.join(BIOMEOS_USB_MARKER).is_file();
    }

    probe_mount_and_check_marker(device)
}

#[cfg(target_os = "linux")]
fn find_mount_point(device: &Path) -> Option<PathBuf> {
    let device_path = device.to_string_lossy();
    let mounts = std::fs::read_to_string("/proc/mounts").ok()?;

    for line in mounts.lines() {
        let mut parts = line.split_whitespace();
        let mount_device = parts.next()?;
        let mount_point = parts.next()?;

        if mount_device == device_path {
            return Some(PathBuf::from(mount_point));
        }
    }

    None
}

#[cfg(target_os = "linux")]
fn probe_mount_and_check_marker(device: &Path) -> bool {
    use rustix::mount::{MountFlags, UnmountFlags, mount, unmount};
    use std::ffi::CStr;

    if std::fs::create_dir_all(USB_PROBE_MOUNT).is_err() {
        return false;
    }

    let Some(device_str) = device.to_str() else {
        return false;
    };

    let mounted = match mount(
        device_str,
        USB_PROBE_MOUNT,
        "auto",
        MountFlags::RDONLY,
        None::<&CStr>,
    ) {
        Ok(()) => true,
        Err(rustix::io::Errno::BUSY) => {
            debug!(
                "Probe mount busy for {}, checking existing mount",
                device.display()
            );
            return find_mount_point(device)
                .is_some_and(|mp| mp.join(BIOMEOS_USB_MARKER).is_file());
        }
        Err(e) => {
            debug!("Probe mount failed for {}: {e}", device.display());
            false
        }
    };

    if !mounted {
        return false;
    }

    let marker_path = Path::new(USB_PROBE_MOUNT).join(BIOMEOS_USB_MARKER);
    let has_marker = marker_path.is_file();

    if unmount(USB_PROBE_MOUNT, UnmountFlags::empty()).is_err() {
        warn!(
            "Failed to unmount probe mount at {USB_PROBE_MOUNT} after scanning {}",
            device.display()
        );
    }

    has_marker
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
mod tests {
    use super::*;

    #[test]
    fn detect_returns_none_on_non_linux_or_missing_sysfs() {
        #[cfg(not(target_os = "linux"))]
        assert!(detect_biomeos_usb_device().is_none());

        #[cfg(target_os = "linux")]
        {
            let missing = tempfile::tempdir().expect("tempdir");
            let missing = missing.path().join("nonexistent-block");
            assert!(detect_biomeos_usb_at(&missing).is_none());
        }
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn skips_non_removable_block_devices() {
        let root = tempfile::tempdir().expect("tempdir");
        let block_dir = root.path().join("block");
        let block_path = block_dir.join("sda");
        std::fs::create_dir_all(&block_path).expect("mkdir block");
        std::fs::write(block_path.join("removable"), "0").expect("write removable");

        assert!(detect_biomeos_usb_at(&block_dir).is_none());
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn enumerates_partitions_from_sysfs() {
        let root = tempfile::tempdir().expect("tempdir");
        let block_dir = root.path().join("block");
        let block_path = block_dir.join("sdb");
        std::fs::create_dir_all(&block_path).expect("mkdir block");
        std::fs::write(block_path.join("removable"), "1").expect("write removable");

        let part1 = block_path.join("sdb1");
        std::fs::create_dir_all(&part1).expect("mkdir partition");
        std::fs::write(part1.join("partition"), "1").expect("write partition");

        let part2 = block_path.join("sdb2");
        std::fs::create_dir_all(&part2).expect("mkdir partition");
        std::fs::write(part2.join("partition"), "2").expect("write partition");

        let candidates = block_device_candidates(&block_path, "sdb");
        assert_eq!(
            candidates,
            vec![PathBuf::from("/dev/sdb1"), PathBuf::from("/dev/sdb2")]
        );
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn falls_back_to_whole_disk_when_no_partitions() {
        let root = tempfile::tempdir().expect("tempdir");
        let block_path = root.path().join("sdc");
        std::fs::create_dir_all(&block_path).expect("mkdir block");

        let candidates = block_device_candidates(&block_path, "sdc");
        assert_eq!(candidates, vec![PathBuf::from("/dev/sdc")]);
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn removable_flag_requires_exact_one() {
        let root = tempfile::tempdir().expect("tempdir");
        let block_path = root.path().join("sdd");
        std::fs::create_dir_all(&block_path).expect("mkdir block");

        std::fs::write(block_path.join("removable"), "1\n").expect("write removable");
        assert!(is_removable_block_device(&block_path));

        std::fs::write(block_path.join("removable"), "0").expect("write removable");
        assert!(!is_removable_block_device(&block_path));

        assert!(!is_removable_block_device(&root.path().join("missing")));
    }

    #[test]
    fn usb_marker_constant_matches_spore_layout() {
        assert_eq!(BIOMEOS_USB_MARKER, ".biomeos-spore");
    }
}
