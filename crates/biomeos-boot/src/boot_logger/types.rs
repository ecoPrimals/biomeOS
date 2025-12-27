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
    /// BiomeOS core services starting
    BiomeOSCoreStart,
    /// Boot complete, system ready
    Complete,
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevel::Debug => write!(f, "DEBUG"),
            LogLevel::Info => write!(f, "INFO"),
            LogLevel::Warning => write!(f, "WARN"),
            LogLevel::Error => write!(f, "ERROR"),
            LogLevel::Critical => write!(f, "CRIT"),
            LogLevel::Emergency => write!(f, "EMERG"),
        }
    }
}

impl std::fmt::Display for BootStage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BootStage::GrubHandoff => write!(f, "GRUB Handoff"),
            BootStage::KernelLoad => write!(f, "Kernel Load"),
            BootStage::InitramfsMount => write!(f, "Initramfs Mount"),
            BootStage::InitStart => write!(f, "Init Start"),
            BootStage::FilesystemMount => write!(f, "Filesystem Mount"),
            BootStage::HardwareDetection => write!(f, "Hardware Detection"),
            BootStage::NetworkInit => write!(f, "Network Init"),
            BootStage::BiomeOSCoreStart => write!(f, "BiomeOS Core Start"),
            BootStage::Complete => write!(f, "Boot Complete"),
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
    fn test_boot_stage_display() {
        assert_eq!(BootStage::InitStart.to_string(), "Init Start");
        assert_eq!(BootStage::Complete.to_string(), "Boot Complete");
    }
}

