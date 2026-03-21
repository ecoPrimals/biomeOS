// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Filesystem Management for BiomeOS Init
//!
//! Handles mounting and managing essential filesystems during boot.

use crate::init_error::{BootError, Result};
use rustix::mount::{MountFlags, mount};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use tracing::info;

/// Manages filesystem mounts during initialization
pub struct FilesystemManager {
    mounted: HashSet<PathBuf>,
}

impl FilesystemManager {
    /// Creates a new filesystem manager
    pub fn new() -> Self {
        Self {
            mounted: HashSet::new(),
        }
    }

    /// Mounts all essential filesystems
    ///
    /// Mounts `/proc`, `/sys`, `/dev`, and other pseudo-filesystems required
    /// for system operation.
    ///
    /// # Errors
    ///
    /// Returns an error if any critical filesystem cannot be mounted.
    pub async fn mount_essential(&mut self) -> Result<()> {
        info!("📁 Mounting essential filesystems...");

        // /proc - Process information
        self.mount_if_needed("/proc", "proc", "proc", MountFlags::empty())
            .await?;

        // /sys - Kernel and device information
        self.mount_if_needed("/sys", "sysfs", "sysfs", MountFlags::empty())
            .await?;

        // /dev - Device files (may already be mounted by kernel)
        self.mount_if_needed("/dev", "devtmpfs", "devtmpfs", MountFlags::empty())
            .await?;

        // /dev/pts - Pseudo-terminals
        self.mount_if_needed("/dev/pts", "devpts", "devpts", MountFlags::empty())
            .await?;

        // /dev/shm - Shared memory
        self.mount_if_needed("/dev/shm", "tmpfs", "tmpfs", MountFlags::empty())
            .await?;

        // /run - Runtime data
        self.mount_if_needed("/run", "tmpfs", "tmpfs", MountFlags::empty())
            .await?;

        // /tmp - Temporary files
        self.mount_if_needed("/tmp", "tmpfs", "tmpfs", MountFlags::empty())
            .await?;

        info!("✅ Essential filesystems mounted");
        Ok(())
    }

    /// Mounts a filesystem if not already mounted
    ///
    /// # Arguments
    ///
    /// * `target` - Mount point (e.g., `/proc`)
    /// * `source` - Source device or pseudo-filesystem
    /// * `fstype` - Filesystem type (e.g., `"proc"`)
    /// * `flags` - Mount flags
    ///
    /// # Errors
    ///
    /// Returns [`BootError::MountFailed`] if the mount fails for reasons other than EBUSY.
    /// Returns [`BootError::DirectoryCreation`] if the mount point cannot be created.
    async fn mount_if_needed(
        &mut self,
        target: impl AsRef<Path>,
        source: &str,
        fstype: &str,
        flags: MountFlags,
    ) -> Result<()> {
        let target_path = target.as_ref().to_path_buf();

        // Skip if already mounted by us
        if self.mounted.contains(&target_path) {
            return Ok(());
        }

        // Create mount point if it doesn't exist
        tokio::fs::create_dir_all(&target_path).await.map_err(|e| {
            BootError::DirectoryCreation {
                path: target_path.clone(),
                error: e.to_string(),
            }
        })?;

        // Try to mount (rustix: source, target, fstype, flags, data)
        match mount(source, target.as_ref(), fstype, flags, "") {
            Ok(()) => {
                info!("  ✓ {}", target.as_ref().display());
                self.mounted.insert(target_path);
                Ok(())
            }
            Err(rustix::io::Errno::BUSY) => {
                // Already mounted by kernel - this is fine
                info!("  ✓ {} (already mounted)", target.as_ref().display());
                self.mounted.insert(target_path);
                Ok(())
            }
            Err(e) => Err(BootError::mount_failed(
                target.as_ref().display().to_string(),
                source,
                e,
            )),
        }
    }

    /// Checks if a filesystem is mounted
    pub fn is_mounted(&self, path: impl AsRef<Path>) -> bool {
        self.mounted.contains(path.as_ref())
    }

    /// Gets list of mounted filesystems
    pub fn mounted_filesystems(&self) -> Vec<&Path> {
        self.mounted
            .iter()
            .map(std::path::PathBuf::as_path)
            .collect()
    }
}

impl Default for FilesystemManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
mod tests {
    use super::*;
    use crate::init_error::BootError;

    #[test]
    fn test_filesystem_manager_creation() {
        let mgr = FilesystemManager::new();
        assert_eq!(mgr.mounted_filesystems().len(), 0);
    }

    #[test]
    fn test_filesystem_manager_default() {
        let mgr = FilesystemManager::default();
        assert_eq!(mgr.mounted_filesystems().len(), 0);
    }

    #[test]
    fn test_is_mounted() {
        let mut mgr = FilesystemManager::new();
        assert!(!mgr.is_mounted("/proc"));
        mgr.mounted.insert(PathBuf::from("/proc"));
        assert!(mgr.is_mounted("/proc"));
    }

    #[test]
    fn test_is_mounted_multiple() {
        let mut mgr = FilesystemManager::new();
        mgr.mounted.insert(PathBuf::from("/proc"));
        mgr.mounted.insert(PathBuf::from("/sys"));
        assert!(mgr.is_mounted("/proc"));
        assert!(mgr.is_mounted("/sys"));
        assert!(!mgr.is_mounted("/dev"));
    }

    #[test]
    fn test_mounted_filesystems_returns_paths() {
        let mut mgr = FilesystemManager::new();
        mgr.mounted.insert(PathBuf::from("/proc"));
        let paths = mgr.mounted_filesystems();
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0], Path::new("/proc"));
    }

    #[tokio::test]
    async fn mount_if_needed_skips_when_already_tracked() {
        let mut mgr = FilesystemManager::new();
        let tmp = tempfile::tempdir().expect("tempdir");
        let target = tmp.path().join("already-tracked");
        mgr.mounted.insert(target.clone());
        let r = mgr
            .mount_if_needed(&target, "proc", "proc", MountFlags::empty())
            .await;
        assert!(r.is_ok());
        assert!(mgr.is_mounted(&target));
    }

    #[tokio::test]
    async fn mount_if_needed_directory_creation_fails_when_parent_is_file() {
        let mut mgr = FilesystemManager::new();
        let tmp = tempfile::tempdir().expect("tempdir");
        let file_path = tmp.path().join("not_a_directory");
        std::fs::write(&file_path, b"x").expect("write file");
        let nested = file_path.join("mountpoint");
        let r = mgr
            .mount_if_needed(&nested, "proc", "proc", MountFlags::empty())
            .await;
        assert!(
            matches!(r, Err(BootError::DirectoryCreation { .. })),
            "expected DirectoryCreation, got {r:?}"
        );
    }

    /// Invalid fstype should fail the mount syscall (not EBUSY), covering the `Err(e)` branch.
    #[tokio::test]
    async fn mount_if_needed_mount_failed_for_nonsense_fstype() {
        let mut mgr = FilesystemManager::new();
        let tmp = tempfile::tempdir().expect("tempdir");
        let target = tmp.path().join("mnt-invalid-fstype");
        let r = mgr
            .mount_if_needed(
                &target,
                "none",
                "biomeos_test_not_a_fstype_xyz",
                MountFlags::empty(),
            )
            .await;
        assert!(
            matches!(r, Err(BootError::MountFailed { .. })),
            "expected MountFailed, got {r:?}"
        );
    }
}
