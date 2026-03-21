// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Serial Channel - Direct Serial Port Access
//!
//! Provides direct access to /dev/ttyS0 (COM1) for boot logging,
//! bypassing kernel console abstractions.

use crate::init_error::{BootError, Result};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;

/// Direct serial port channel
///
/// Writes directly to /dev/ttyS0 (COM1) without relying on kernel console
/// device mapping. This ensures output is visible even if the kernel's
/// console= parameter is misconfigured.
pub struct SerialChannel {
    device: File,
}

impl SerialChannel {
    /// Open the serial device for writing
    ///
    /// Opens `/dev/ttyS0` (COM1) directly. This requires the device node
    /// to exist - use `DeviceManager::ensure_serial_device()` first.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - `/dev/ttyS0` doesn't exist
    /// - Permission denied (need to be root or in dialout group)
    /// - Device is already open exclusively
    pub fn new() -> Result<Self> {
        Self::open_path(Path::new("/dev/ttyS0"))
    }

    /// Open a writable device path (used by tests with a regular file; production uses [`Self::new`].
    pub(crate) fn open_path(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Err(BootError::DeviceNotFound {
                device: path.display().to_string(),
            });
        }

        let device =
            OpenOptions::new()
                .write(true)
                .open(path)
                .map_err(|e| BootError::DeviceOpen {
                    device: path.display().to_string(),
                    error: e.to_string(),
                })?;

        Ok(Self { device })
    }

    /// Write bytes to serial port
    ///
    /// Writes raw bytes directly to the serial port and flushes immediately
    /// to ensure output is visible.
    ///
    /// # Errors
    ///
    /// Returns an error if the write or flush operation fails.
    pub fn write(&mut self, data: &[u8]) -> Result<()> {
        self.device
            .write_all(data)
            .map_err(|e| BootError::IoError {
                operation: "serial write".to_string(),
                error: e.to_string(),
            })?;

        self.flush()?;
        Ok(())
    }

    /// Flush the serial port buffer
    ///
    /// Ensures all buffered data is written to the device immediately.
    pub fn flush(&mut self) -> Result<()> {
        self.device.flush().map_err(|e| BootError::IoError {
            operation: "serial flush".to_string(),
            error: e.to_string(),
        })?;
        Ok(())
    }

    /// Check if serial device is available
    ///
    /// Returns true if `/dev/ttyS0` exists and is accessible.
    pub fn available() -> bool {
        Path::new("/dev/ttyS0").exists()
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_serial_availability() {
        // Test device availability check
        // (may be false in test environment, which is fine)
        let _ = SerialChannel::available();
    }

    #[test]
    fn test_serial_channel_creation() {
        // Only test if device exists
        if SerialChannel::available() {
            let result = SerialChannel::new();
            // Should either succeed or fail gracefully
            match result {
                Ok(_) => println!("Serial channel opened successfully"),
                Err(e) => println!("Serial channel failed (expected in test): {e:?}"),
            }
        }
    }

    #[test]
    fn test_open_path_missing_device() {
        let result = SerialChannel::open_path(Path::new("/nonexistent/ttyS0-serial-test"));
        assert!(matches!(result, Err(BootError::DeviceNotFound { .. })));
    }

    #[test]
    fn test_write_and_flush_roundtrip_on_temp_file() {
        let tmp = NamedTempFile::new().expect("temp file");
        let path = tmp.path().to_path_buf();

        let mut channel = SerialChannel::open_path(&path).expect("open temp file");
        channel.write(b"boot> ").expect("write");
        channel.flush().expect("flush");

        let buf = std::fs::read(&path).expect("read back");
        assert_eq!(buf, b"boot> ");
    }

    #[test]
    fn test_device_not_found_preserves_path_display() {
        let p = "/no/such/serial";
        let result = SerialChannel::open_path(Path::new(p));
        let err = result.err().expect("should err");
        assert!(
            format!("{err:?}").contains("no/such") || err.to_string().contains("no/such"),
            "{err:?}"
        );
    }
}
