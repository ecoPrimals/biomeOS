// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Host architecture, platform, and genome metadata types.

use anyhow::{Result, bail};

/// Supported architectures
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Architecture {
    /// x86-64 (AMD64)
    X86_64,
    /// ARM 64-bit (`AArch64`)
    Aarch64,
    /// ARM 32-bit (`ARMv7`)
    Armv7,
    /// RISC-V 64-bit
    Riscv64,
}

impl Architecture {
    /// Detect host architecture
    pub fn detect() -> Result<Self> {
        match std::env::consts::ARCH {
            "x86_64" => Ok(Self::X86_64),
            "aarch64" | "arm64" => Ok(Self::Aarch64),
            "armv7" => Ok(Self::Armv7),
            "riscv64" => Ok(Self::Riscv64),
            arch => bail!("Unsupported architecture: {arch}"),
        }
    }

    /// Get architecture string for binary lookup
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::X86_64 => "x86_64",
            Self::Aarch64 => "aarch64",
            Self::Armv7 => "armv7",
            Self::Riscv64 => "riscv64",
        }
    }
}

/// Supported platforms
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    /// Linux
    Linux,
    /// Android (Linux-family)
    Android,
    /// macOS
    MacOS,
    /// Windows
    Windows,
}

impl Platform {
    /// Detect host platform
    pub fn detect() -> Result<Self> {
        // Check for Android first (it reports as Linux)
        if std::path::Path::new("/system/build.prop").exists() {
            return Ok(Self::Android);
        }

        match std::env::consts::OS {
            "linux" => Ok(Self::Linux),
            "macos" => Ok(Self::MacOS),
            "windows" => Ok(Self::Windows),
            os => bail!("Unsupported platform: {os}"),
        }
    }

    /// Get platform name
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Linux => "Linux",
            Self::Android => "Android",
            Self::MacOS => "macOS",
            Self::Windows => "Windows",
        }
    }

    /// Check if platform supports abstract sockets
    #[must_use]
    pub const fn supports_abstract_sockets(&self) -> bool {
        matches!(self, Self::Android | Self::Linux)
    }
}

/// `GenomeBin` metadata
#[derive(Debug, Clone)]
pub struct GenomeMetadata {
    /// Genome name
    pub name: String,
    /// Genome version
    pub version: String,
    /// Human-readable description
    pub description: String,
    /// Supported architectures
    pub architectures: Vec<Architecture>,
}
