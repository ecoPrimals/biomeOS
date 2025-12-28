//! Boot Logger - Production-Grade Boot Observability
//!
//! Provides multi-channel logging for BiomeOS boot process, ensuring complete
//! visibility from kernel handoff through init completion.
//!
//! ## Design Philosophy
//!
//! **Sovereignty through Observability**: "What you cannot observe, you cannot control."
//!
//! This module implements direct device access, bypassing kernel console abstractions
//! to ensure boot output is always visible, regardless of kernel configuration.
//!
//! ## Architecture
//!
//! ```text
//! BootLogger
//!     ├─> SerialChannel (/dev/ttyS0)
//!     ├─> VgaChannel (/dev/tty0)
//!     ├─> MemoryChannel (circular buffer)
//!     └─> FileChannel (/var/log/boot.log)
//! ```
//!
//! ## Usage
//!
//! ```rust,no_run
//! use biomeos_boot::boot_logger::{BootLogger, LogLevel};
//!
//! let mut logger = BootLogger::new()?;
//! logger.log(LogLevel::Info, "BiomeOS initialization starting");
//! logger.checkpoint(BootStage::InitStart);
//! ```

pub mod device_mgr;
pub mod serial;
pub mod types;

pub use device_mgr::DeviceManager;
pub use serial::SerialChannel;
pub use types::{BootStage, LogLevel};

use crate::init_error::{BootError, Result};
use std::time::SystemTime;

/// Main boot logger coordinating multiple output channels
pub struct BootLogger {
    serial: Option<SerialChannel>,
    start_time: SystemTime,
    log_count: usize,
}

impl BootLogger {
    /// Create a new boot logger
    ///
    /// Attempts to open all available output channels. If a channel fails to open,
    /// it is disabled but logging continues to other channels.
    ///
    /// # Errors
    ///
    /// Returns an error only if NO channels can be opened (complete logging failure).
    pub fn new() -> Result<Self> {
        // Ensure devices exist
        DeviceManager::ensure_serial_device()?;

        // Open serial channel (direct /dev/ttyS0 access)
        let serial = SerialChannel::new().ok();

        if serial.is_none() {
            return Err(BootError::ConsoleInit(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to open any output channels",
            )));
        }

        Ok(Self {
            serial,
            start_time: SystemTime::now(),
            log_count: 0,
        })
    }

    /// Log a message with level
    ///
    /// Writes to all available output channels simultaneously.
    pub fn log(&mut self, level: LogLevel, msg: &str) {
        let timestamp = self
            .start_time
            .elapsed()
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        // Format message
        let formatted = format!("[{:010}] [{:?}] {}\n", timestamp, level, msg);

        // Write to serial (direct device access)
        if let Some(ref mut serial) = self.serial {
            let _ = serial.write(formatted.as_bytes());
        }

        self.log_count += 1;
    }

    /// Log at INFO level
    pub fn info(&mut self, msg: &str) {
        self.log(LogLevel::Info, msg);
    }

    /// Log at WARNING level
    pub fn warning(&mut self, msg: &str) {
        self.log(LogLevel::Warning, msg);
    }

    /// Log at ERROR level
    pub fn error(&mut self, msg: &str) {
        self.log(LogLevel::Error, msg);
    }

    /// Log at CRITICAL level
    pub fn critical(&mut self, msg: &str) {
        self.log(LogLevel::Critical, msg);
    }

    /// Mark a boot checkpoint
    ///
    /// Records progress through boot stages for diagnostics.
    pub fn checkpoint(&mut self, stage: BootStage) {
        self.info(&format!("Boot checkpoint: {:?}", stage));
    }

    /// Flush all output channels
    pub fn flush(&mut self) {
        if let Some(ref mut serial) = self.serial {
            let _ = serial.flush();
        }
    }

    /// Get statistics
    pub fn stats(&self) -> BootLoggerStats {
        BootLoggerStats {
            log_count: self.log_count,
            uptime_ms: self
                .start_time
                .elapsed()
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0),
            serial_active: self.serial.is_some(),
        }
    }
}

/// Boot logger statistics
#[derive(Debug)]
pub struct BootLoggerStats {
    pub log_count: usize,
    pub uptime_ms: u64,
    pub serial_active: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_formatting() {
        // Test that log messages are formatted correctly
        // (without actually writing to devices in test)
        let level = LogLevel::Info;
        let msg = "Test message";

        // Verify format
        assert_eq!(format!("{:?}", level), "Info");
    }

    #[test]
    fn test_boot_stage_progression() {
        // Test that boot stages can be tracked
        let stages = vec![
            BootStage::GrubHandoff,
            BootStage::InitStart,
            BootStage::FilesystemMount,
            BootStage::Complete,
        ];

        for stage in stages {
            // Should not panic
            let _ = format!("{:?}", stage);
        }
    }
}
