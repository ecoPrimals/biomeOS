// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Tests for [`crate::Architecture`], [`crate::Platform`], and [`crate::GenomeMetadata`].

use crate::{Architecture, GenomeMetadata, Platform};

// ============================================================================
// Architecture Tests
// ============================================================================

#[test]
fn test_architecture_as_str() {
    assert_eq!(Architecture::X86_64.as_str(), "x86_64");
    assert_eq!(Architecture::Aarch64.as_str(), "aarch64");
    assert_eq!(Architecture::Armv7.as_str(), "armv7");
    assert_eq!(Architecture::Riscv64.as_str(), "riscv64");
}

#[test]
fn test_architecture_detect_returns_known_arch() {
    // detect() should succeed on any supported platform
    let result = Architecture::detect();
    // On test systems, this should succeed (we're running on a known arch)
    assert!(
        result.is_ok(),
        "Should detect architecture on supported system"
    );
    let arch = result.unwrap();
    // Verify it returns a valid architecture string
    let arch_str = arch.as_str();
    assert!(
        ["x86_64", "aarch64", "armv7", "riscv64"].contains(&arch_str),
        "Should be a known architecture"
    );
}

#[test]
fn test_architecture_equality() {
    assert_eq!(Architecture::X86_64, Architecture::X86_64);
    assert_ne!(Architecture::X86_64, Architecture::Aarch64);
}

#[test]
fn test_architecture_clone() {
    let arch = Architecture::Aarch64;
    let cloned = arch;
    assert_eq!(arch, cloned);
}

#[test]
fn test_architecture_debug() {
    let arch = Architecture::X86_64;
    let debug_str = format!("{arch:?}");
    assert!(debug_str.contains("X86_64"));
}

// ============================================================================
// Platform Tests
// ============================================================================

#[test]
fn test_platform_name() {
    assert_eq!(Platform::Linux.name(), "Linux");
    assert_eq!(Platform::Android.name(), "Android");
    assert_eq!(Platform::MacOS.name(), "macOS");
    assert_eq!(Platform::Windows.name(), "Windows");
}

#[test]
fn test_platform_supports_abstract_sockets() {
    assert!(Platform::Android.supports_abstract_sockets());
    assert!(Platform::Linux.supports_abstract_sockets());
    assert!(!Platform::MacOS.supports_abstract_sockets());
    assert!(!Platform::Windows.supports_abstract_sockets());
}

#[test]
fn test_platform_detect_returns_known_platform() {
    let result = Platform::detect();
    assert!(result.is_ok(), "Should detect platform on supported system");
    let platform = result.unwrap();
    let name = platform.name();
    assert!(
        ["Linux", "Android", "macOS", "Windows"].contains(&name),
        "Should be a known platform"
    );
}

#[test]
fn test_platform_equality() {
    assert_eq!(Platform::Linux, Platform::Linux);
    assert_ne!(Platform::Linux, Platform::Android);
}

#[test]
fn test_platform_clone() {
    let platform = Platform::Linux;
    let cloned = platform;
    assert_eq!(platform, cloned);
}

#[test]
fn test_platform_debug() {
    let platform = Platform::Linux;
    let debug_str = format!("{platform:?}");
    assert!(debug_str.contains("Linux"));
}

// ============================================================================
// GenomeMetadata Tests
// ============================================================================

#[test]
fn test_genome_metadata_creation() {
    let metadata = GenomeMetadata {
        name: "test-primal".to_string(),
        version: "1.0.0".to_string(),
        description: "Test primal for testing".to_string(),
        architectures: vec![Architecture::X86_64, Architecture::Aarch64],
    };

    assert_eq!(metadata.name, "test-primal");
    assert_eq!(metadata.version, "1.0.0");
    assert_eq!(metadata.description, "Test primal for testing");
    assert_eq!(metadata.architectures.len(), 2);
}

#[test]
fn test_genome_metadata_clone() {
    let metadata = GenomeMetadata {
        name: "test".to_string(),
        version: "1.0.0".to_string(),
        description: "desc".to_string(),
        architectures: vec![Architecture::X86_64],
    };
    let cloned = metadata.clone();
    assert_eq!(metadata.name, cloned.name);
    assert_eq!(metadata.version, cloned.version);
}

#[test]
fn test_genome_metadata_debug() {
    let metadata = GenomeMetadata {
        name: "beardog".to_string(),
        version: "0.9.0".to_string(),
        description: "Security primal".to_string(),
        architectures: vec![Architecture::X86_64],
    };
    let debug_str = format!("{metadata:?}");
    assert!(debug_str.contains("beardog"));
    assert!(debug_str.contains("0.9.0"));
}
