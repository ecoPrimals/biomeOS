// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

// biomeos-genomebin-v3/src/builder.rs
// Builder API for creating genomeBins
//
// Deep Debt Principles:
// - Modern idiomatic Rust (builder pattern)
// - Clear error messages
// - Runtime validation

use crate::{Arch, GenomeBin, GenomeManifest};
use anyhow::{Context, Result};

/// Builder for creating genomeBins
pub struct GenomeBinBuilder {
    name: String,
    version: Option<String>,
    description: Option<String>,
    nucleus_atomic: Option<String>,
    capabilities: Vec<String>,
    binaries: Vec<(Arch, std::path::PathBuf)>,
}

impl GenomeBinBuilder {
    /// Create new builder
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: None,
            description: None,
            nucleus_atomic: None,
            capabilities: Vec::new(),
            binaries: Vec::new(),
        }
    }

    /// Set version
    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }

    /// Set description
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set NUCLEUS atomic type
    pub fn nucleus_atomic(mut self, atomic_type: impl Into<String>) -> Self {
        self.nucleus_atomic = Some(atomic_type.into());
        self
    }

    /// Add capability
    pub fn capability(mut self, capability: impl Into<String>) -> Self {
        self.capabilities.push(capability.into());
        self
    }

    /// Add binary for architecture
    pub fn add_binary(mut self, arch: Arch, path: impl Into<std::path::PathBuf>) -> Self {
        self.binaries.push((arch, path.into()));
        self
    }

    /// Build genomeBin
    pub fn build(self) -> Result<GenomeBin> {
        tracing::info!("Building genomeBin: {}", self.name);

        // Validate
        if self.binaries.is_empty() {
            anyhow::bail!("Cannot build genomeBin without any binaries");
        }

        // Create manifest
        let mut manifest = GenomeManifest::new(&self.name);

        if let Some(version) = self.version {
            manifest.version = version;
        }

        if let Some(description) = self.description {
            manifest.description = description;
        }

        manifest.nucleus_atomic = self.nucleus_atomic;
        manifest.capabilities = self.capabilities;

        // Create genomeBin
        let mut genome = GenomeBin {
            manifest,
            binaries: std::collections::HashMap::new(),
            embedded_genomes: Vec::new(),
        };

        // Add all binaries
        for (arch, path) in self.binaries {
            if !path.exists() {
                anyhow::bail!("Binary not found: {}", path.display());
            }

            genome
                .add_binary(arch, &path)
                .with_context(|| format!("Failed to add {:?} binary", arch))?;
        }

        tracing::info!(
            "✅ genomeBin built: {} ({} architectures)",
            genome.manifest.name,
            genome.binaries.len()
        );

        Ok(genome)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    fn create_test_binary(dir: &TempDir, name: &str, content: &[u8]) -> std::path::PathBuf {
        let path = dir.path().join(name);
        let mut file = std::fs::File::create(&path).unwrap();
        file.write_all(content).unwrap();
        path
    }

    #[test]
    fn test_builder_basic() {
        let temp_dir = TempDir::new().unwrap();
        let binary = create_test_binary(&temp_dir, "test-binary", b"fake binary content");

        let genome = GenomeBinBuilder::new("test-primal")
            .version("1.0.0")
            .description("Test primal")
            .add_binary(Arch::X86_64, binary)
            .build()
            .unwrap();

        assert_eq!(genome.manifest.name, "test-primal");
        assert_eq!(genome.manifest.version, "1.0.0");
        assert_eq!(genome.manifest.description, "Test primal");
        assert_eq!(genome.binaries.len(), 1);
        assert!(genome.manifest.supports_arch(Arch::X86_64));
    }

    #[test]
    fn test_builder_multi_arch() {
        let temp_dir = TempDir::new().unwrap();
        let binary_x86 = create_test_binary(&temp_dir, "bin-x86", b"x86 binary");
        let binary_arm = create_test_binary(&temp_dir, "bin-arm", b"arm binary");

        let genome = GenomeBinBuilder::new("multi-arch")
            .add_binary(Arch::X86_64, binary_x86)
            .add_binary(Arch::Aarch64, binary_arm)
            .build()
            .unwrap();

        assert_eq!(genome.binaries.len(), 2);
        assert!(genome.manifest.supports_arch(Arch::X86_64));
        assert!(genome.manifest.supports_arch(Arch::Aarch64));
    }

    #[test]
    fn test_builder_no_binaries() {
        let result = GenomeBinBuilder::new("empty").build();

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("without any binaries"));
    }

    #[test]
    fn test_builder_missing_binary() {
        let result = GenomeBinBuilder::new("missing")
            .add_binary(Arch::X86_64, "/nonexistent/path")
            .build();

        assert!(result.is_err());
    }
}
