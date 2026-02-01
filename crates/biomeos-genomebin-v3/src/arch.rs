// biomeos-genomebin-v3/src/arch.rs
// Architecture enumeration - Pure Rust, platform-agnostic
//
// Deep Debt Principles:
// - Runtime detection (no hardcoding)
// - Modern idiomatic Rust (enum with methods)
// - Self-contained (no external dependencies)

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Supported architectures for genomeBin
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum Arch {
    // Linux / Android
    /// x86_64 (AMD64, Intel 64-bit) - Linux
    X86_64,
    
    /// ARM 64-bit (aarch64, arm64) - Linux/Android
    Aarch64,
    
    /// ARM 32-bit (armv7) - Linux/Android
    Armv7,
    
    /// RISC-V 64-bit - Linux
    Riscv64,
    
    /// x86 32-bit (i686) - Linux
    X86,
    
    /// PowerPC 64-bit (little-endian) - Linux
    Ppc64le,
    
    /// S390x (IBM Z) - Linux
    S390x,
    
    // Darwin (macOS)
    /// x86_64 Apple Darwin (Intel Mac)
    X86_64Darwin,
    
    /// ARM 64-bit Apple Darwin (Apple Silicon)
    Aarch64Darwin,
    
    // iOS
    /// ARM 64-bit iOS (iPhone, iPad)
    Aarch64Ios,
    
    /// x86_64 iOS Simulator
    X86_64IosSim,
    
    /// ARM 64-bit iOS Simulator (Apple Silicon Mac)
    Aarch64IosSim,
    
    // Windows
    /// x86_64 Windows (Intel/AMD 64-bit)
    X86_64Windows,
    
    /// ARM 64-bit Windows
    Aarch64Windows,
    
    /// x86 32-bit Windows
    I686Windows,
}

impl Arch {
    /// Detect current architecture at runtime
    /// 
    /// Deep Debt: Runtime discovery, no compile-time hardcoding
    pub fn detect() -> Self {
        let arch = std::env::consts::ARCH;
        let os = std::env::consts::OS;
        
        match (arch, os) {
            // Linux / Android
            ("x86_64", "linux") => Arch::X86_64,
            ("aarch64", "linux") => Arch::Aarch64,
            ("aarch64", "android") => Arch::Aarch64,
            ("arm", "linux") => Arch::Armv7,
            ("arm", "android") => Arch::Armv7,
            ("riscv64", "linux") => Arch::Riscv64,
            ("x86", "linux") => Arch::X86,
            ("powerpc64", "linux") => Arch::Ppc64le,
            ("s390x", "linux") => Arch::S390x,
            
            // Darwin (macOS)
            ("x86_64", "macos") => Arch::X86_64Darwin,
            ("aarch64", "macos") => Arch::Aarch64Darwin,
            
            // iOS
            ("aarch64", "ios") => Arch::Aarch64Ios,
            ("x86_64", "ios") => Arch::X86_64IosSim,
            
            // Windows
            ("x86_64", "windows") => Arch::X86_64Windows,
            ("aarch64", "windows") => Arch::Aarch64Windows,
            ("x86", "windows") => Arch::I686Windows,
            
            // Unknown
            (arch, os) => {
                tracing::warn!("Unknown architecture '{}' on OS '{}', defaulting to x86_64", arch, os);
                Arch::X86_64
            }
        }
    }
    
    /// Get architecture as string (for filenames, etc.)
    pub fn as_str(&self) -> &'static str {
        match self {
            // Linux / Android
            Arch::X86_64 => "x86_64",
            Arch::Aarch64 => "aarch64",
            Arch::Armv7 => "armv7",
            Arch::Riscv64 => "riscv64",
            Arch::X86 => "x86",
            Arch::Ppc64le => "ppc64le",
            Arch::S390x => "s390x",
            
            // Darwin
            Arch::X86_64Darwin => "x86_64-darwin",
            Arch::Aarch64Darwin => "aarch64-darwin",
            
            // iOS
            Arch::Aarch64Ios => "aarch64-ios",
            Arch::X86_64IosSim => "x86_64-ios-sim",
            Arch::Aarch64IosSim => "aarch64-ios-sim",
            
            // Windows
            Arch::X86_64Windows => "x86_64-windows",
            Arch::Aarch64Windows => "aarch64-windows",
            Arch::I686Windows => "i686-windows",
        }
    }
}

/// Parse error for Arch
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseArchError {
    input: String,
}

impl fmt::Display for ParseArchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Unknown architecture: '{}'", self.input)
    }
}

impl std::error::Error for ParseArchError {}

/// Implement standard FromStr trait for idiomatic Rust
impl FromStr for Arch {
    type Err = ParseArchError;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            // Linux / Android
            "x86_64" | "amd64" => Ok(Arch::X86_64),
            "aarch64" | "arm64" => Ok(Arch::Aarch64),
            "armv7" | "arm" => Ok(Arch::Armv7),
            "riscv64" => Ok(Arch::Riscv64),
            "x86" | "i686" => Ok(Arch::X86),
            "ppc64le" | "powerpc64le" => Ok(Arch::Ppc64le),
            "s390x" => Ok(Arch::S390x),
            
            // Darwin
            "x86_64-darwin" | "x86-64-apple-darwin" | "darwin-x86_64" => Ok(Arch::X86_64Darwin),
            "aarch64-darwin" | "arm64-apple-darwin" | "apple-silicon" | "darwin-aarch64" => Ok(Arch::Aarch64Darwin),
            
            // iOS
            "aarch64-ios" | "arm64-ios" | "ios" | "ios-aarch64" => Ok(Arch::Aarch64Ios),
            "x86_64-ios-sim" | "ios-sim" | "ios-sim-x86_64" => Ok(Arch::X86_64IosSim),
            "aarch64-ios-sim" | "arm64-ios-sim" | "ios-sim-aarch64" => Ok(Arch::Aarch64IosSim),
            
            // Windows
            "x86_64-windows" | "win64" | "windows-x86_64" => Ok(Arch::X86_64Windows),
            "aarch64-windows" | "arm64-windows" | "windows-aarch64" => Ok(Arch::Aarch64Windows),
            "i686-windows" | "win32" | "windows-i686" => Ok(Arch::I686Windows),
            
            _ => Err(ParseArchError { input: s.to_string() }),
        }
    }
}

impl Arch {
    /// Get typical triple suffix (for Rust targets)
    pub fn rust_target_suffix(&self) -> &'static str {
        match self {
            // Linux / Android
            Arch::X86_64 => "x86_64-unknown-linux-gnu",
            Arch::Aarch64 => "aarch64-unknown-linux-gnu",
            Arch::Armv7 => "armv7-unknown-linux-gnueabihf",
            Arch::Riscv64 => "riscv64gc-unknown-linux-gnu",
            Arch::X86 => "i686-unknown-linux-gnu",
            Arch::Ppc64le => "powerpc64le-unknown-linux-gnu",
            Arch::S390x => "s390x-unknown-linux-gnu",
            
            // Darwin
            Arch::X86_64Darwin => "x86_64-apple-darwin",
            Arch::Aarch64Darwin => "aarch64-apple-darwin",
            
            // iOS
            Arch::Aarch64Ios => "aarch64-apple-ios",
            Arch::X86_64IosSim => "x86_64-apple-ios",
            Arch::Aarch64IosSim => "aarch64-apple-ios-sim",
            
            // Windows
            Arch::X86_64Windows => "x86_64-pc-windows-msvc",
            Arch::Aarch64Windows => "aarch64-pc-windows-msvc",
            Arch::I686Windows => "i686-pc-windows-msvc",
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
        // Linux
        assert_eq!(Arch::X86_64.as_str(), "x86_64");
        assert_eq!(Arch::Aarch64.as_str(), "aarch64");
        
        // Darwin
        assert_eq!(Arch::X86_64Darwin.as_str(), "x86_64-darwin");
        assert_eq!(Arch::Aarch64Darwin.as_str(), "aarch64-darwin");
        
        // iOS
        assert_eq!(Arch::Aarch64Ios.as_str(), "aarch64-ios");
        
        // Windows
        assert_eq!(Arch::X86_64Windows.as_str(), "x86_64-windows");
        
        // Parse tests (using standard FromStr trait)
        assert_eq!("x86_64".parse::<Arch>(), Ok(Arch::X86_64));
        assert_eq!("amd64".parse::<Arch>(), Ok(Arch::X86_64));
        assert_eq!("aarch64".parse::<Arch>(), Ok(Arch::Aarch64));
        assert_eq!("arm64".parse::<Arch>(), Ok(Arch::Aarch64));
        
        // Darwin parse
        assert_eq!("x86_64-darwin".parse::<Arch>(), Ok(Arch::X86_64Darwin));
        assert_eq!("apple-silicon".parse::<Arch>(), Ok(Arch::Aarch64Darwin));
        
        // iOS parse
        assert_eq!("aarch64-ios".parse::<Arch>(), Ok(Arch::Aarch64Ios));
        assert_eq!("ios-sim".parse::<Arch>(), Ok(Arch::X86_64IosSim));
        
        // Windows parse
        assert_eq!("x86_64-windows".parse::<Arch>(), Ok(Arch::X86_64Windows));
        assert_eq!("win64".parse::<Arch>(), Ok(Arch::X86_64Windows));
        
        // Unknown should return error
        assert!("unknown".parse::<Arch>().is_err());
    }
    
    #[test]
    fn test_arch_display() {
        assert_eq!(format!("{}", Arch::X86_64), "x86_64");
        assert_eq!(format!("{}", Arch::Aarch64), "aarch64");
    }
}
