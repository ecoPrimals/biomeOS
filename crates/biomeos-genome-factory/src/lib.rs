//! Genome Factory - Universal genomeBin production for any primal
//!
//! This crate provides a factory for building, composing, and replicating
//! genomeBins - the self-extracting binary format for ecoPrimals primals.
//!
#![warn(missing_docs)]
#![deny(unsafe_code)]
//!
//! # AGPL-3.0-only License
//!
//! This program is free software: you can redistribute it and/or modify
//! it under the terms of the GNU Affero General Public License as published
//! by the Free Software Foundation, version 3.
//!
//! # Deep Debt Principles
//!
//! - **Fractal Composition**: Atomics are composed from individual genomes
//! - **Self-Replication**: Genomes can replicate across the federation
//! - **Runtime Discovery**: No hardcoded paths, discover storage at runtime
//! - **100% Pure Rust**: No C dependencies

/// Atomic genomeBin composition from individual genomes
pub mod compose;
/// Self-replication and genome distribution
pub mod replicate;
/// GenomeBin integrity verification
pub mod verify;

pub use compose::{GenomeComposeRequest, GenomeComposeResponse};
pub use replicate::{GenomeReplicateRequest, GenomeReplicateResponse};
pub use verify::{GenomeVerifyRequest, GenomeVerifyResponse};

use anyhow::{Context, Result};
use std::path::PathBuf;

/// Genome Factory for building and managing genomeBins
pub struct GenomeFactory {
    /// Storage directory for genomes
    storage_dir: PathBuf,
}

impl GenomeFactory {
    /// Create new factory with storage directory
    pub fn new(storage_dir: impl Into<PathBuf>) -> Self {
        Self {
            storage_dir: storage_dir.into(),
        }
    }

    /// Create factory with XDG-compliant storage
    pub fn with_xdg_storage() -> Result<Self> {
        use etcetera::base_strategy::{choose_base_strategy, BaseStrategy};

        let strategy = choose_base_strategy().context("Failed to determine base strategy")?;

        let storage = strategy.data_dir().join("biomeos/genomes");
        std::fs::create_dir_all(&storage).context("Failed to create genome storage directory")?;

        Ok(Self::new(storage))
    }

    /// Get storage directory
    pub fn storage_dir(&self) -> &PathBuf {
        &self.storage_dir
    }

    /// Get path for a specific genome
    pub fn genome_path(&self, name: &str) -> PathBuf {
        self.storage_dir.join(format!("{}.genome", name))
    }

    /// Check if genome exists
    pub fn genome_exists(&self, name: &str) -> bool {
        self.genome_path(name).exists()
    }

    /// List all available genomes
    pub fn list_genomes(&self) -> Result<Vec<String>> {
        let mut genomes = Vec::new();

        for entry in std::fs::read_dir(&self.storage_dir)
            .context("Failed to read genome storage directory")?
        {
            let entry = entry?;
            let path = entry.path();

            if path.extension().map(|e| e == "genome").unwrap_or(false) {
                if let Some(stem) = path.file_stem() {
                    genomes.push(stem.to_string_lossy().to_string());
                }
            }
        }

        Ok(genomes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_factory_creation() {
        let temp = TempDir::new().unwrap();
        let factory = GenomeFactory::new(temp.path());
        assert_eq!(factory.storage_dir(), temp.path());
    }

    #[test]
    fn test_genome_path() {
        let temp = TempDir::new().unwrap();
        let factory = GenomeFactory::new(temp.path());

        let path = factory.genome_path("beardog");
        assert!(path.ends_with("beardog.genome"));
    }

    #[test]
    fn test_genome_exists() {
        let temp = TempDir::new().unwrap();
        let factory = GenomeFactory::new(temp.path());

        assert!(!factory.genome_exists("nonexistent"));

        // Create a genome file
        std::fs::write(factory.genome_path("test"), b"test").unwrap();
        assert!(factory.genome_exists("test"));
    }
}
