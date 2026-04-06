// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! genomeBin v3.0 - TRUE Binary Isomorphic Format
//!
//! A pure Rust implementation of a self-extracting, multi-architecture
//! binary format for distributing ecoPrimals primals.
//!
#![warn(missing_docs)]
#![forbid(unsafe_code)]
//!
//! # Deep Debt Principles
//!
//! - **Runtime Discovery**: Architecture detection at execution time
//! - **Fractal Composition**: genomeBins can embed other genomeBins
//! - **Capability-Based**: Bootstrap selector based on capabilities
//! - **Platform-Agnostic**: Works on any supported architecture
//! - **100% Pure Rust**: No C dependencies
//!
//! # Format Versions
//!
//! - v3.0: Base format with manifest and compressed binaries
//! - v4.1: Multi-architecture fat binary with embedded extractors
//!
//! # AGPL-3.0-only License
//!
//! This program is free software: you can redistribute it and/or modify
//! it under the terms of the GNU Affero General Public License as published
//! by the Free Software Foundation, version 3.
//!
//! This program is distributed in the hope that it will be useful,
//! but WITHOUT ANY WARRANTY; without even the implied warranty of
//! MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.

mod builder;
mod composer;
mod manifest;
mod runtime;
mod v4_1;
mod verify;

pub use builder::GenomeBinBuilder;
pub use composer::GenomeBinComposer;
pub use manifest::GenomeManifest;

use bytes::Bytes;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Supported target architectures
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Arch {
    /// x86-64 (Intel/AMD 64-bit)
    X86_64,
    /// ARM 64-bit (Apple Silicon, Android, etc.)
    Aarch64,
    /// ARM 32-bit (older devices)
    Arm,
    /// RISC-V 64-bit (emerging architecture)
    Riscv64,
}

impl Arch {
    /// Detect current architecture at runtime
    #[must_use]
    pub const fn detect() -> Self {
        #[cfg(target_arch = "x86_64")]
        {
            Self::X86_64
        }
        #[cfg(target_arch = "aarch64")]
        {
            Arch::Aarch64
        }
        #[cfg(target_arch = "arm")]
        {
            Arch::Arm
        }
        #[cfg(target_arch = "riscv64")]
        {
            Arch::Riscv64
        }
        #[cfg(not(any(
            target_arch = "x86_64",
            target_arch = "aarch64",
            target_arch = "arm",
            target_arch = "riscv64"
        )))]
        {
            compile_error!("Unsupported architecture");
        }
    }

    /// Get target triple string for this architecture
    #[must_use]
    pub const fn target_triple(&self) -> &'static str {
        match self {
            Self::X86_64 => "x86_64-unknown-linux-musl",
            Self::Aarch64 => "aarch64-unknown-linux-musl",
            Self::Arm => "arm-unknown-linux-musleabi",
            Self::Riscv64 => "riscv64gc-unknown-linux-musl",
        }
    }
}

impl std::fmt::Display for Arch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::X86_64 => write!(f, "x86_64"),
            Self::Aarch64 => write!(f, "aarch64"),
            Self::Arm => write!(f, "arm"),
            Self::Riscv64 => write!(f, "riscv64"),
        }
    }
}

/// LZ4-compressed binary data with integrity verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressedBinary {
    /// Target architecture
    pub arch: Arch,
    /// Compressed binary data (LZ4, zero-copy via `bytes::Bytes`)
    #[serde(with = "biomeos_types::tarpc_types::bytes_serde")]
    pub data: Bytes,
    /// Original (uncompressed) size for allocation
    pub original_size: usize,
    /// BLAKE3 checksum of original (uncompressed) data
    pub checksum: [u8; 32],
}

impl CompressedBinary {
    /// Create from raw bytes
    #[must_use]
    pub fn new(arch: Arch, data: &[u8]) -> Self {
        let compressed = lz4_flex::compress_prepend_size(data);
        let hash = blake3::hash(data);

        Self {
            arch,
            data: Bytes::from(compressed),
            original_size: data.len(),
            checksum: *hash.as_bytes(),
        }
    }

    /// Compress binary data using LZ4 (alias for new)
    #[must_use]
    pub fn compress(arch: Arch, data: &[u8]) -> Self {
        Self::new(arch, data)
    }

    /// Create from file
    pub fn from_file(arch: Arch, path: &std::path::Path) -> anyhow::Result<Self> {
        use anyhow::Context;
        let data = std::fs::read(path)
            .with_context(|| format!("Failed to read binary: {}", path.display()))?;
        Ok(Self::new(arch, &data))
    }

    /// Decompress and verify integrity
    ///
    /// Returns [`Bytes`] for zero-copy read-only access to decompressed data.
    pub fn decompress(&self) -> anyhow::Result<Bytes> {
        let decompressed = lz4_flex::decompress_size_prepended(self.data.as_ref())
            .map_err(|e| anyhow::anyhow!("LZ4 decompression failed: {e}"))?;

        // Verify checksum
        let actual_hash = blake3::hash(&decompressed);
        if actual_hash.as_bytes() != &self.checksum {
            anyhow::bail!(
                "Integrity check failed: checksum mismatch (expected {}, got {})",
                hex::encode(self.checksum),
                hex::encode(actual_hash.as_bytes())
            );
        }

        // Verify size
        if decompressed.len() != self.original_size {
            anyhow::bail!(
                "Size mismatch: expected {} bytes, got {} bytes",
                self.original_size,
                decompressed.len()
            );
        }

        Ok(Bytes::from(decompressed))
    }

    /// Verify integrity without decompressing
    pub fn verify(&self) -> anyhow::Result<()> {
        let _ = self.decompress()?;
        Ok(())
    }
}

/// The core genomeBin structure
///
/// A genomeBin is a self-contained, multi-architecture binary package
/// that can be installed and run on any supported platform.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenomeBin {
    /// Manifest with metadata and capabilities
    pub manifest: GenomeManifest,
    /// Architecture-specific compressed binaries
    pub binaries: HashMap<Arch, CompressedBinary>,
    /// Embedded genomeBins (for fractal composition)
    #[serde(default)]
    pub embedded_genomes: Vec<Self>,
}

impl GenomeBin {
    /// Create a new empty genomeBin with a name
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            manifest: GenomeManifest::new(name),
            binaries: HashMap::new(),
            embedded_genomes: Vec::new(),
        }
    }

    /// Create a new empty genomeBin with the given manifest
    #[must_use]
    pub fn with_manifest(manifest: GenomeManifest) -> Self {
        Self {
            manifest,
            binaries: HashMap::new(),
            embedded_genomes: Vec::new(),
        }
    }

    /// Add binary for a specific architecture (from raw bytes)
    pub fn add_binary_bytes(&mut self, arch: Arch, data: &[u8]) {
        self.binaries
            .insert(arch, CompressedBinary::compress(arch, data));
        if !self.manifest.architectures.contains(&arch) {
            self.manifest.architectures.push(arch);
        }
    }

    /// Add binary for a specific architecture (from file path)
    pub fn add_binary(&mut self, arch: Arch, path: &std::path::Path) -> anyhow::Result<()> {
        use anyhow::Context;
        let data = std::fs::read(path)
            .with_context(|| format!("Failed to read binary: {}", path.display()))?;
        self.add_binary_bytes(arch, &data);
        Ok(())
    }

    /// Embed another genomeBin (fractal composition)
    pub fn embed(&mut self, genome: Self) -> anyhow::Result<()> {
        self.embedded_genomes.push(genome);
        Ok(())
    }

    /// Check if binary exists for current architecture
    #[must_use]
    pub fn has_current_arch(&self) -> bool {
        self.binaries.contains_key(&Arch::detect())
    }

    /// Get compressed binary for architecture (if available)
    #[must_use]
    pub fn get_binary(&self, arch: Arch) -> Option<&CompressedBinary> {
        self.binaries.get(&arch)
    }

    /// Serialize to JSON
    pub fn to_json(&self) -> anyhow::Result<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    /// Deserialize from JSON
    pub fn from_json(json: &str) -> anyhow::Result<Self> {
        Ok(serde_json::from_str(json)?)
    }

    /// Save to file (JSON format)
    pub fn save(&self, path: &std::path::Path) -> anyhow::Result<()> {
        use anyhow::Context;
        let json = self.to_json()?;
        std::fs::write(path, json).with_context(|| format!("Failed to write: {}", path.display()))
    }

    /// Load from file (JSON format)
    pub fn load(path: &std::path::Path) -> anyhow::Result<Self> {
        use anyhow::Context;
        let json = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read: {}", path.display()))?;
        Self::from_json(&json)
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
mod tests {
    use super::*;

    #[test]
    fn test_arch_detect() {
        let arch = Arch::detect();
        // Should be one of the supported architectures
        assert!(matches!(
            arch,
            Arch::X86_64 | Arch::Aarch64 | Arch::Arm | Arch::Riscv64
        ));
    }

    #[test]
    fn test_arch_target_triple() {
        assert_eq!(Arch::X86_64.target_triple(), "x86_64-unknown-linux-musl");
        assert_eq!(Arch::Aarch64.target_triple(), "aarch64-unknown-linux-musl");
        assert_eq!(Arch::Arm.target_triple(), "arm-unknown-linux-musleabi");
        assert_eq!(
            Arch::Riscv64.target_triple(),
            "riscv64gc-unknown-linux-musl"
        );
    }

    #[test]
    fn test_arch_display() {
        assert_eq!(format!("{}", Arch::X86_64), "x86_64");
        assert_eq!(format!("{}", Arch::Aarch64), "aarch64");
    }

    #[test]
    fn test_compression_roundtrip() {
        let original = b"Hello, genomeBin! This is test data for compression.";
        let compressed = CompressedBinary::compress(Arch::X86_64, original);

        assert_eq!(compressed.original_size, original.len());

        let decompressed = compressed.decompress().expect("Decompression failed");
        assert_eq!(decompressed.as_ref(), original);
    }

    #[test]
    fn test_compressed_binary_verify() {
        let original = b"verify me";
        let compressed = CompressedBinary::compress(Arch::X86_64, original);
        compressed.verify().expect("verify should succeed");

        // Tampered data should fail
        let mut tampered = compressed;
        tampered.checksum[0] ^= 0xff;
        tampered
            .verify()
            .expect_err("verify should fail on tampered checksum");
    }

    #[test]
    fn test_compressed_binary_decompress_wrong_size() {
        let mut bad = CompressedBinary::compress(Arch::X86_64, b"x");
        bad.original_size = 999;
        let err = bad.decompress().expect_err("wrong size should fail");
        assert!(err.to_string().contains("Size mismatch"));
    }

    #[test]
    fn test_compressed_binary_new() {
        let c = CompressedBinary::new(Arch::Aarch64, b"data");
        assert_eq!(c.arch, Arch::Aarch64);
        assert_eq!(c.original_size, 4);
    }

    #[test]
    fn test_genomebin_creation() {
        let genome = GenomeBin::new("test-primal");

        assert_eq!(genome.manifest.name, "test-primal");
        assert!(genome.binaries.is_empty());
        assert!(genome.embedded_genomes.is_empty());
    }

    #[test]
    fn test_genomebin_with_manifest() {
        let manifest = GenomeManifest::new("test-primal")
            .version("1.0.0")
            .description("Test primal for unit tests")
            .add_capability("test");

        let genome = GenomeBin::with_manifest(manifest);

        assert_eq!(genome.manifest.name, "test-primal");
        assert_eq!(genome.manifest.version, "1.0.0");
        assert!(genome.binaries.is_empty());
        assert!(genome.embedded_genomes.is_empty());
    }

    #[test]
    fn test_add_binary() {
        let mut genome = GenomeBin::new("test");

        genome.add_binary_bytes(Arch::X86_64, b"fake binary data");

        assert!(genome.binaries.contains_key(&Arch::X86_64));
        assert!(genome.manifest.architectures.contains(&Arch::X86_64));
    }

    #[test]
    fn test_get_binary() {
        let mut genome = GenomeBin::new("test");
        genome.add_binary_bytes(Arch::X86_64, b"data");
        assert!(genome.get_binary(Arch::X86_64).is_some());
        assert!(genome.get_binary(Arch::Aarch64).is_none());
    }

    #[test]
    fn test_embed() {
        let mut parent = GenomeBin::new("parent");
        let child = GenomeBin::new("child");
        parent.embed(child).expect("embed should succeed");
        assert_eq!(parent.embedded_genomes.len(), 1);
        assert_eq!(parent.embedded_genomes[0].manifest.name, "child");
    }

    #[test]
    fn test_json_roundtrip() {
        let mut genome = GenomeBin::new("roundtrip-test");
        genome.add_binary_bytes(Arch::X86_64, b"test binary");

        let json = genome.to_json().expect("Serialization failed");
        let loaded = GenomeBin::from_json(&json).expect("Deserialization failed");

        assert_eq!(loaded.manifest.name, "roundtrip-test");
        assert!(loaded.binaries.contains_key(&Arch::X86_64));
    }

    #[test]
    fn test_save_load_json() {
        let mut genome = GenomeBin::new("save-load-test");
        genome.add_binary_bytes(Arch::X86_64, b"binary");
        let temp = tempfile::tempdir().expect("temp dir");
        let path = temp.path().join("genome.json");
        genome.save(&path).expect("save");
        let loaded = GenomeBin::load(&path).expect("load");
        assert_eq!(loaded.manifest.name, "save-load-test");
    }

    #[test]
    fn test_from_json_invalid() {
        let err = GenomeBin::from_json("not json").expect_err("invalid json");
        assert!(!err.to_string().is_empty());
    }
}
