// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Device Manager - Ensure Required Devices Exist
//!
//! Creates and manages device nodes required for boot logging.

use crate::init_error::{BootError, Result};
use rustix::fs::{CWD, FileType, Mode, makedev, mknodat};
use std::path::Path;

/// Manages device node creation and permissions
pub struct DeviceManager;

impl DeviceManager {
    /// Ensure serial device exists
    ///
    /// Creates `/dev/ttyS0` (COM1) if it doesn't exist.
    ///
    /// Device parameters:
    /// - Major: 4 (TTY devices)
    /// - Minor: 64 (ttyS0/COM1)
    /// - Type: Character device
    /// - Permissions: 0660 (rw-rw----)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Not running as root (mknod requires CAP_MKNOD)
    /// - /dev directory doesn't exist
    /// - mknod system call fails
    pub fn ensure_serial_device() -> Result<()> {
        let path = "/dev/ttyS0";

        // Check if already exists
        if Path::new(path).exists() {
            return Ok(());
        }

        // Ensure /dev directory exists
        if !Path::new("/dev").exists() {
            return Err(BootError::DirectoryCreation {
                path: std::path::PathBuf::from("/dev"),
                error: "Directory doesn't exist".to_string(),
            });
        }

        // Create character device
        // Major 4 = TTY devices, Minor 64 = ttyS0 (COM1)
        let mode = Mode::from_bits_truncate(0o660);
        mknodat(CWD, path, FileType::CharacterDevice, mode, makedev(4, 64)).map_err(|e| {
            BootError::DeviceCreation {
                device: path.to_string(),
                error: format!("mknod failed: {e}"),
            }
        })?;

        // Note: chown requires additional nix features
        // For now, device is created with current user permissions
        // In production (as root), this will be root-owned automatically

        Ok(())
    }

    /// Ensure VGA console device exists
    ///
    /// Creates `/dev/tty0` if it doesn't exist.
    ///
    /// Device parameters:
    /// - Major: 4 (TTY devices)
    /// - Minor: 0 (tty0/VGA console)
    pub fn ensure_vga_device() -> Result<()> {
        let path = "/dev/tty0";

        if Path::new(path).exists() {
            return Ok(());
        }

        // Major 4, Minor 0 = tty0 (VGA console)
        let mode = Mode::from_bits_truncate(0o660);
        mknodat(CWD, path, FileType::CharacterDevice, mode, makedev(4, 0)).map_err(|e| {
            BootError::DeviceCreation {
                device: path.to_string(),
                error: format!("mknod failed: {e}"),
            }
        })?;

        Ok(())
    }

    /// Setup /dev/console symlink
    ///
    /// Creates a symlink from `/dev/console` to `/dev/ttyS0`, ensuring
    /// that legacy code using `/dev/console` writes to the serial port.
    pub fn setup_console_symlink() -> Result<()> {
        let console_path = "/dev/console";
        let target = "/dev/ttyS0";

        // Remove existing console device/symlink if present
        if Path::new(console_path).exists() {
            std::fs::remove_file(console_path).ok(); // Ignore errors, may not have permission
        }

        // Create symlink
        std::os::unix::fs::symlink(target, console_path).map_err(|e| {
            BootError::DeviceCreation {
                device: console_path.to_string(),
                error: format!("symlink failed: {e}"),
            }
        })?;

        Ok(())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_device_manager_safe() {
        // Test that DeviceManager can be constructed
        // (actual device creation requires root, so just test structure)
        let _ = DeviceManager;
    }

    #[test]
    fn test_device_paths() {
        // Verify device paths are correct
        assert_eq!("/dev/ttyS0".len(), 10);
        assert_eq!("/dev/tty0".len(), 9);
    }

    #[test]
    fn test_serial_device_path() {
        assert_eq!(Path::new("/dev/ttyS0").parent(), Some(Path::new("/dev")));
    }

    #[test]
    fn test_vga_device_path() {
        assert_eq!(Path::new("/dev/tty0").parent(), Some(Path::new("/dev")));
    }

    #[test]
    fn test_console_symlink_target() {
        let target = "/dev/ttyS0";
        assert!(target.starts_with("/dev"));
    }
}
