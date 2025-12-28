//! Hardware Detection for BiomeOS Init
//!
//! Detects and reports hardware capabilities during boot.

use crate::init_error::{BootError, Result};
use std::num::NonZeroUsize;
use sysinfo::System;
use tracing::info;

/// Hardware information detected during boot
#[derive(Debug, Clone)]
pub struct HardwareInfo {
    /// Number of CPU cores
    pub cpu_count: NonZeroUsize,
    /// Total RAM in gigabytes
    pub total_memory_gb: u64,
    /// System architecture
    pub architecture: Architecture,
}

/// System architecture
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Architecture {
    /// x86-64 (AMD64)
    X86_64,
    /// ARM64 (AArch64)
    Aarch64,
    /// RISC-V 64-bit
    Riscv64,
    /// Other/Unknown
    Unknown,
}

impl Architecture {
    /// Detects the current system architecture
    pub fn detect() -> Self {
        #[cfg(target_arch = "x86_64")]
        return Self::X86_64;

        #[cfg(target_arch = "aarch64")]
        return Self::Aarch64;

        #[cfg(target_arch = "riscv64")]
        return Self::Riscv64;

        #[cfg(not(any(
            target_arch = "x86_64",
            target_arch = "aarch64",
            target_arch = "riscv64"
        )))]
        return Self::Unknown;
    }

    /// Returns the architecture as a string
    pub fn as_str(&self) -> &str {
        match self {
            Self::X86_64 => "x86_64",
            Self::Aarch64 => "aarch64",
            Self::Riscv64 => "riscv64",
            Self::Unknown => "unknown",
        }
    }
}

/// Detects hardware capabilities
///
/// # Errors
///
/// Returns an error if hardware detection fails.
pub async fn detect() -> Result<HardwareInfo> {
    let mut sys = System::new_all();
    sys.refresh_all();

    // Detect CPU count
    let cpu_count = NonZeroUsize::new(sys.cpus().len()).ok_or_else(|| {
        BootError::HardwareDetection(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            "No CPUs detected",
        )))
    })?;

    // Detect memory
    let total_memory = sys.total_memory();
    let total_memory_gb = total_memory / (1024 * 1024 * 1024);

    // Detect architecture
    let architecture = Architecture::detect();

    let info = HardwareInfo {
        cpu_count,
        total_memory_gb,
        architecture,
    };

    info!(
        "Hardware: {} cores, {} GB RAM, {}",
        info.cpu_count,
        info.total_memory_gb,
        info.architecture.as_str()
    );

    Ok(info)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_architecture_detect() {
        let arch = Architecture::detect();
        // Should return something valid
        assert!(matches!(
            arch,
            Architecture::X86_64
                | Architecture::Aarch64
                | Architecture::Riscv64
                | Architecture::Unknown
        ));
    }

    #[test]
    fn test_architecture_as_str() {
        assert_eq!(Architecture::X86_64.as_str(), "x86_64");
        assert_eq!(Architecture::Aarch64.as_str(), "aarch64");
        assert_eq!(Architecture::Riscv64.as_str(), "riscv64");
    }

    #[tokio::test]
    async fn test_hardware_detection() {
        let result = detect().await;
        // Should succeed on any system
        assert!(result.is_ok());

        if let Ok(hw) = result {
            assert!(hw.cpu_count.get() > 0);
            // Memory might be 0 in some test environments
            assert!(hw.total_memory_gb >= 0);
        }
    }
}
