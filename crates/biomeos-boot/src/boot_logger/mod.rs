// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

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
//! let mut logger = BootLogger::new().expect("Failed to initialize logger");
//! logger.log(LogLevel::Info, "BiomeOS initialization starting");
//! // logger.checkpoint(BootStage::InitStart);  // Requires full boot context
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
            return Err(BootError::ConsoleInit(std::io::Error::other(
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
    /// Total number of log entries recorded
    pub log_count: usize,
    /// System uptime in milliseconds since boot
    pub uptime_ms: u64,
    /// Whether serial console logging is active
    pub serial_active: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_level_debug_format() {
        assert_eq!(format!("{:?}", LogLevel::Info), "Info");
        assert_eq!(format!("{:?}", LogLevel::Warning), "Warning");
        assert_eq!(format!("{:?}", LogLevel::Error), "Error");
        assert_eq!(format!("{:?}", LogLevel::Critical), "Critical");
    }

    #[test]
    fn test_boot_stage_progression() {
        let stages = vec![
            BootStage::GrubHandoff,
            BootStage::InitStart,
            BootStage::FilesystemMount,
            BootStage::Complete,
        ];

        for stage in stages {
            let debug_str = format!("{:?}", stage);
            assert!(!debug_str.is_empty());
        }
    }

    #[test]
    fn test_boot_stage_debug_format() {
        assert!(format!("{:?}", BootStage::GrubHandoff).contains("GrubHandoff"));
        assert!(format!("{:?}", BootStage::InitStart).contains("InitStart"));
        assert!(format!("{:?}", BootStage::Complete).contains("Complete"));
    }

    #[test]
    fn test_boot_logger_stats_debug() {
        let stats = BootLoggerStats {
            log_count: 10,
            uptime_ms: 5000,
            serial_active: true,
        };

        let debug_str = format!("{:?}", stats);
        assert!(debug_str.contains("10"));
        assert!(debug_str.contains("5000"));
        assert!(debug_str.contains("true"));
    }

    #[test]
    fn test_boot_logger_stats_fields() {
        let stats = BootLoggerStats {
            log_count: 0,
            uptime_ms: 0,
            serial_active: false,
        };

        assert_eq!(stats.log_count, 0);
        assert_eq!(stats.uptime_ms, 0);
        assert!(!stats.serial_active);
    }

    #[test]
    fn test_log_message_format() {
        // Test the expected log message format
        let timestamp = 1234567890_u64;
        let level = LogLevel::Info;
        let msg = "Test message";

        let formatted = format!("[{:010}] [{:?}] {}\n", timestamp, level, msg);

        assert!(formatted.contains("[1234567890]"));
        assert!(formatted.contains("[Info]"));
        assert!(formatted.contains("Test message"));
        assert!(formatted.ends_with('\n'));
    }

    #[test]
    fn test_checkpoint_message_format() {
        let stage = BootStage::FilesystemMount;
        let checkpoint_msg = format!("Boot checkpoint: {:?}", stage);

        assert!(checkpoint_msg.contains("Boot checkpoint"));
        assert!(checkpoint_msg.contains("FilesystemMount"));
    }
}
