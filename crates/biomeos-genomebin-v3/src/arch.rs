// biomeos-genomebin-v3/src/arch.rs
// Architecture enumeration - Pure Rust, platform-agnostic
//
// Deep Debt Principles:
// - Runtime detection (no hardcoding)
// - Modern idiomatic Rust (enum with methods)
// - Self-contained (no external dependencies)

use serde::{Deserialize, Serialize};
use std::fmt;

/// Supported architectures for genomeBin
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum Arch {
    /// x86_64 (AMD64, Intel 64-bit)
    X86_64,
    
    /// ARM 64-bit (aarch64, arm64)
    Aarch64,
    
    /// ARM 32-bit (armv7)
    Armv7,
    
    /// RISC-V 64-bit
    Riscv64,
    
    /// x86 32-bit (i686)
    X86,
    
    /// PowerPC 64-bit (little-endian)
    Ppc64le,
    
    /// S390x (IBM Z)
    S390x,
}

impl Arch {
    /// Detect current architecture at runtime
    /// 
    /// Deep Debt: Runtime discovery, no compile-time hardcoding
    pub fn detect() -> Self {
        match std::env::consts::ARCH {
            "x86_64" => Arch::X86_64,
            "aarch64" => Arch::Aarch64,
            "arm" => Arch::Armv7,
            "riscv64" => Arch::Riscv64,
            "x86" => Arch::X86,
            "powerpc64" => Arch::Ppc64le,
            "s390x" => Arch::S390x,
            arch => {
                tracing::warn!("Unknown architecture '{}', defaulting to x86_64", arch);
                Arch::X86_64
            }
        }
    }
    
    /// Get architecture as string (for filenames, etc.)
    pub fn as_str(&self) -> &'static str {
        match self {
            Arch::X86_64 => "x86_64",
            Arch::Aarch64 => "aarch64",
            Arch::Armv7 => "armv7",
            Arch::Riscv64 => "riscv64",
            Arch::X86 => "x86",
            Arch::Ppc64le => "ppc64le",
            Arch::S390x => "s390x",
        }
    }
    
    /// Parse architecture from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "x86_64" | "amd64" => Some(Arch::X86_64),
            "aarch64" | "arm64" => Some(Arch::Aarch64),
            "armv7" | "arm" => Some(Arch::Armv7),
            "riscv64" => Some(Arch::Riscv64),
            "x86" | "i686" => Some(Arch::X86),
            "ppc64le" | "powerpc64le" => Some(Arch::Ppc64le),
            "s390x" => Some(Arch::S390x),
            _ => None,
        }
    }
    
    /// Get typical triple suffix (for Rust targets)
    pub fn rust_target_suffix(&self) -> &'static str {
        match self {
            Arch::X86_64 => "x86_64-unknown-linux-gnu",
            Arch::Aarch64 => "aarch64-unknown-linux-gnu",
            Arch::Armv7 => "armv7-unknown-linux-gnueabihf",
            Arch::Riscv64 => "riscv64gc-unknown-linux-gnu",
            Arch::X86 => "i686-unknown-linux-gnu",
            Arch::Ppc64le => "powerpc64le-unknown-linux-gnu",
            Arch::S390x => "s390x-unknown-linux-gnu",
        }
    }
}

impl fmt::Display for Arch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_arch_detect() {
        let arch = Arch::detect();
        // Should match current platform
        assert!(matches!(arch, Arch::X86_64 | Arch::Aarch64 | Arch::Armv7 | Arch::Riscv64 | Arch::X86));
    }
    
    #[test]
    fn test_arch_string_conversion() {
        assert_eq!(Arch::X86_64.as_str(), "x86_64");
        assert_eq!(Arch::Aarch64.as_str(), "aarch64");
        
        assert_eq!(Arch::from_str("x86_64"), Some(Arch::X86_64));
        assert_eq!(Arch::from_str("amd64"), Some(Arch::X86_64));
        assert_eq!(Arch::from_str("aarch64"), Some(Arch::Aarch64));
        assert_eq!(Arch::from_str("arm64"), Some(Arch::Aarch64));
        assert_eq!(Arch::from_str("unknown"), None);
    }
    
    #[test]
    fn test_arch_display() {
        assert_eq!(format!("{}", Arch::X86_64), "x86_64");
        assert_eq!(format!("{}", Arch::Aarch64), "aarch64");
    }
}
