// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Boot Logger Types
//!
//! Common types used throughout the boot logger.

/// Log levels for boot messages
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    /// Debug information (verbose)
    Debug,
    /// Informational messages
    Info,
    /// Warnings (non-critical issues)
    Warning,
    /// Errors (failures that can be recovered)
    Error,
    /// Critical errors (system may not boot)
    Critical,
    /// Emergency (system is unusable)
    Emergency,
}

/// Boot stages for checkpoint tracking
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BootStage {
    /// GRUB has handed off control to kernel
    GrubHandoff,
    /// Kernel is loading
    KernelLoad,
    /// Initramfs is being mounted
    InitramfsMount,
    /// Init system is starting (PID 1)
    InitStart,
    /// Essential filesystems being mounted
    FilesystemMount,
    /// Hardware detection in progress
    HardwareDetection,
    /// Network initialization
    NetworkInit,
    /// `BiomeOS` core services starting
    BiomeOSCoreStart,
    /// Boot complete, system ready
    Complete,
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Debug => write!(f, "DEBUG"),
            Self::Info => write!(f, "INFO"),
            Self::Warning => write!(f, "WARN"),
            Self::Error => write!(f, "ERROR"),
            Self::Critical => write!(f, "CRIT"),
            Self::Emergency => write!(f, "EMERG"),
        }
    }
}

impl std::fmt::Display for BootStage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::GrubHandoff => write!(f, "GRUB Handoff"),
            Self::KernelLoad => write!(f, "Kernel Load"),
            Self::InitramfsMount => write!(f, "Initramfs Mount"),
            Self::InitStart => write!(f, "Init Start"),
            Self::FilesystemMount => write!(f, "Filesystem Mount"),
            Self::HardwareDetection => write!(f, "Hardware Detection"),
            Self::NetworkInit => write!(f, "Network Init"),
            Self::BiomeOSCoreStart => write!(f, "BiomeOS Core Start"),
            Self::Complete => write!(f, "Boot Complete"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_level_ordering() {
        assert!(LogLevel::Debug < LogLevel::Info);
        assert!(LogLevel::Info < LogLevel::Warning);
        assert!(LogLevel::Warning < LogLevel::Error);
        assert!(LogLevel::Error < LogLevel::Critical);
        assert!(LogLevel::Critical < LogLevel::Emergency);
    }

    #[test]
    fn test_log_level_display() {
        assert_eq!(LogLevel::Info.to_string(), "INFO");
        assert_eq!(LogLevel::Error.to_string(), "ERROR");
    }

    #[test]
    fn test_log_level_display_all_variants() {
        assert_eq!(LogLevel::Debug.to_string(), "DEBUG");
        assert_eq!(LogLevel::Info.to_string(), "INFO");
        assert_eq!(LogLevel::Warning.to_string(), "WARN");
        assert_eq!(LogLevel::Error.to_string(), "ERROR");
        assert_eq!(LogLevel::Critical.to_string(), "CRIT");
        assert_eq!(LogLevel::Emergency.to_string(), "EMERG");
    }

    #[test]
    fn test_boot_stage_display() {
        assert_eq!(BootStage::InitStart.to_string(), "Init Start");
        assert_eq!(BootStage::Complete.to_string(), "Boot Complete");
    }

    #[test]
    fn test_boot_stage_display_all_variants() {
        assert_eq!(BootStage::GrubHandoff.to_string(), "GRUB Handoff");
        assert_eq!(BootStage::KernelLoad.to_string(), "Kernel Load");
        assert_eq!(BootStage::InitramfsMount.to_string(), "Initramfs Mount");
        assert_eq!(BootStage::InitStart.to_string(), "Init Start");
        assert_eq!(BootStage::FilesystemMount.to_string(), "Filesystem Mount");
        assert_eq!(
            BootStage::HardwareDetection.to_string(),
            "Hardware Detection"
        );
        assert_eq!(BootStage::NetworkInit.to_string(), "Network Init");
        assert_eq!(
            BootStage::BiomeOSCoreStart.to_string(),
            "BiomeOS Core Start"
        );
        assert_eq!(BootStage::Complete.to_string(), "Boot Complete");
    }

    #[test]
    fn test_log_level_debug() {
        assert!(format!("{:?}", LogLevel::Debug).contains("Debug"));
        assert!(format!("{:?}", LogLevel::Emergency).contains("Emergency"));
    }

    #[test]
    fn test_log_level_clone_copy_eq() {
        let level = LogLevel::Warning;
        let cloned = level;
        assert_eq!(level, cloned);
        assert_eq!(LogLevel::Info, LogLevel::Info);
        assert_ne!(LogLevel::Info, LogLevel::Error);
    }

    #[test]
    fn test_boot_stage_debug() {
        assert!(format!("{:?}", BootStage::GrubHandoff).contains("GrubHandoff"));
        assert!(format!("{:?}", BootStage::Complete).contains("Complete"));
    }

    #[test]
    fn test_boot_stage_clone_copy_eq() {
        let stage = BootStage::InitStart;
        let cloned = stage;
        assert_eq!(stage, cloned);
        assert_eq!(BootStage::Complete, BootStage::Complete);
        assert_ne!(BootStage::GrubHandoff, BootStage::Complete);
    }

    #[test]
    fn test_log_level_partial_ord() {
        assert!(LogLevel::Debug < LogLevel::Emergency);
        assert!(LogLevel::Warning >= LogLevel::Info);
    }
}
