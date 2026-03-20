// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

// biomeos-genome-factory/src/replicate.rs
// Self-replication - biomeOS creates its own genomeBin
//
// Deep Debt: Autonomous reproduction

use crate::GenomeFactory;
use anyhow::{Context, Result};
use biomeos_genomebin_v3::{Arch, GenomeBin, GenomeManifest};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Request for replicating a genome
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenomeReplicateRequest {
    /// Genome name to replicate
    pub name: String,
    /// Target destination
    pub destination: PathBuf,
}

/// Response after replication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenomeReplicateResponse {
    /// Identifier of the replicated genome
    pub genome_id: String,
    /// Path where the replicated genome was written
    pub path: PathBuf,
    /// Size of the replicated genome in bytes
    pub size: u64,
    /// Supported architectures in the replicated genome
    pub architectures: Vec<String>,
}

impl GenomeFactory {
    /// Self-replicate: Create biomeOS's own genomeBin
    ///
    /// Deep Debt: Introspection + self-knowledge
    pub fn self_replicate(&self) -> Result<GenomeReplicateResponse> {
        tracing::info!("🧬 Self-replication initiated: biomeOS creating its own genomeBin");

        // Find current executable
        let self_binary =
            std::env::current_exe().context("Failed to get current executable path")?;

        tracing::debug!("Found self binary: {}", self_binary.display());

        // Detect current architecture
        let arch = Arch::detect();

        // Read binary data
        let binary_data = std::fs::read(&self_binary).context("Failed to read self binary")?;

        // Build manifest
        let manifest = GenomeManifest::new("biomeos")
            .version(env!("CARGO_PKG_VERSION"))
            .description("biomeOS System Orchestrator - Self-Replicated")
            .nucleus_atomic("ORCHESTRATOR")
            .add_capability("orchestration")
            .add_capability("genome-factory")
            .add_capability("federation")
            .add_capability("self-replication");

        // Build genomeBin
        let mut genome = GenomeBin::with_manifest(manifest);
        genome.add_binary_bytes(arch, &binary_data);

        // Write to storage
        let output_path = self.genome_path("biomeos-self");
        genome
            .save(&output_path)
            .context("Failed to write self-genomeBin")?;

        let size = std::fs::metadata(&output_path)
            .map(|m| m.len())
            .unwrap_or(0);

        let response = GenomeReplicateResponse {
            genome_id: "biomeos-self".to_string(),
            path: output_path,
            size,
            architectures: vec![format!("{:?}", arch)],
        };

        tracing::info!(
            "✅ Self-replication complete: {} bytes for {:?}",
            response.size,
            arch
        );

        Ok(response)
    }

    /// Check if self-genomeBin exists
    pub fn has_self_genome(&self) -> bool {
        self.genome_exists("biomeos-self")
    }
}

#[allow(clippy::unwrap_used, clippy::expect_used)]
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_self_replicate() {
        let temp_dir = TempDir::new().unwrap();
        let factory = GenomeFactory::new(temp_dir.path());

        let response = factory.self_replicate().unwrap();

        assert_eq!(response.genome_id, "biomeos-self");
        assert!(response.path.exists());
        assert_eq!(response.architectures.len(), 1);
        assert!(factory.has_self_genome());
    }
}
