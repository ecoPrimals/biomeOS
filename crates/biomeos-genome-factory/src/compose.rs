// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

// biomeos-genome-factory/src/compose.rs
// Compose atomic genomeBins from individual genomes
//
// Deep Debt: Fractal composition with validation

use crate::GenomeFactory;
use anyhow::{Context, Result};
use biomeos_genomebin_v3::GenomeBinComposer;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Request to compose atomic genomeBin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenomeComposeRequest {
    /// Atomic name (e.g., "tower", "node", "nest")
    pub name: String,

    /// NUCLEUS type ("TOWER", "NODE", "NEST", "NUCLEUS")
    pub nucleus_type: String,

    /// Genome names to compose (will load from storage)
    pub genomes: Vec<String>,
}

/// Response after composing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenomeComposeResponse {
    /// Unique identifier for the composed genome
    pub genome_id: String,
    /// Path where the composed genome was written
    pub path: PathBuf,
    /// Size of the composed genome in bytes
    pub size: u64,
    /// Names of genomes embedded in the composition
    pub embedded_genomes: Vec<String>,
    /// NUCLEUS type of the composed genome
    pub nucleus_type: String,
}

impl GenomeFactory {
    /// Compose atomic genomeBin
    pub fn compose_genome(&self, request: GenomeComposeRequest) -> Result<GenomeComposeResponse> {
        tracing::info!(
            "🧬 Composing {} atomic: {} (from {} genomes)",
            request.nucleus_type,
            request.name,
            request.genomes.len()
        );

        // Validate request
        if request.genomes.is_empty() {
            anyhow::bail!("No genomes provided for composition");
        }

        // Load all genomes
        let mut composer =
            GenomeBinComposer::new(&request.name).nucleus_type(&request.nucleus_type);

        let mut genome_names = Vec::new();
        for genome_name in &request.genomes {
            let genome = self
                .load_genome(genome_name)
                .with_context(|| format!("Failed to load genome: {genome_name}"))?;

            genome_names.push(genome.manifest.name.clone());
            composer = composer.add_genome(genome);
        }

        // Build composed genome
        let composed = composer.build().context("Failed to compose genomeBin")?;

        // Write to storage
        let output_path = self.genome_path(&request.name);
        composed.save(&output_path).with_context(|| {
            format!("Failed to write composed genome: {}", output_path.display())
        })?;

        let size = std::fs::metadata(&output_path)
            .map(|m| m.len())
            .unwrap_or(0);

        let response = GenomeComposeResponse {
            genome_id: format!("{}-atomic", request.name),
            path: output_path,
            size,
            embedded_genomes: genome_names,
            nucleus_type: request.nucleus_type,
        };

        tracing::info!(
            "✅ Composed genomeBin: {} ({} bytes, {} embedded)",
            response.genome_id,
            response.size,
            response.embedded_genomes.len()
        );

        Ok(response)
    }
}

#[allow(clippy::unwrap_used, clippy::expect_used)]
#[cfg(test)]
mod tests {
    use super::*;
    use biomeos_genomebin_v3::GenomeBin;
    use tempfile::TempDir;

    #[test]
    fn test_compose_empty() {
        let temp_dir = TempDir::new().expect("temp dir");
        let factory = GenomeFactory::new(temp_dir.path());

        let request = GenomeComposeRequest {
            name: "test".to_string(),
            nucleus_type: "TOWER".to_string(),
            genomes: vec![],
        };

        let result = factory.compose_genome(request);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("No genomes provided"));
    }

    #[test]
    fn test_compose_request_serialization() {
        let request = GenomeComposeRequest {
            name: "tower".to_string(),
            nucleus_type: "TOWER".to_string(),
            genomes: vec!["beardog".to_string(), "nest".to_string()],
        };
        let json = serde_json::to_string(&request).expect("serialize");
        assert!(json.contains("\"name\":\"tower\""));
        assert!(json.contains("TOWER"));
        let deserialized: GenomeComposeRequest = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deserialized.name, request.name);
        assert_eq!(deserialized.genomes.len(), 2);
    }

    #[test]
    fn test_compose_response_serialization() {
        let response = GenomeComposeResponse {
            genome_id: "tower-atomic".to_string(),
            path: PathBuf::from("/tmp/tower.genome"),
            size: 1024,
            embedded_genomes: vec!["beardog".to_string()],
            nucleus_type: "TOWER".to_string(),
        };
        let json = serde_json::to_string(&response).expect("serialize");
        assert!(json.contains("\"genome_id\":\"tower-atomic\""));
        assert!(json.contains("\"size\":1024"));
    }

    #[test]
    fn test_compose_nonexistent_genome() {
        let temp_dir = TempDir::new().expect("temp dir");
        let factory = GenomeFactory::new(temp_dir.path());

        let request = GenomeComposeRequest {
            name: "tower".to_string(),
            nucleus_type: "TOWER".to_string(),
            genomes: vec!["nonexistent-genome".to_string()],
        };

        let result = factory.compose_genome(request);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Failed to load genome"));
    }

    #[test]
    fn test_compose_success() {
        let temp_dir = TempDir::new().expect("temp dir");
        let factory = GenomeFactory::new(temp_dir.path());

        let genome = GenomeBin::new("beardog");
        genome
            .save(&factory.genome_path("beardog"))
            .expect("save genome");

        let genome2 = GenomeBin::new("songbird");
        genome2
            .save(&factory.genome_path("songbird"))
            .expect("save genome");

        let request = GenomeComposeRequest {
            name: "tower".to_string(),
            nucleus_type: "TOWER".to_string(),
            genomes: vec!["beardog".to_string(), "songbird".to_string()],
        };

        let result = factory.compose_genome(request);
        let response = result.expect("compose should succeed");
        assert_eq!(response.genome_id, "tower-atomic");
        assert_eq!(response.embedded_genomes, vec!["beardog", "songbird"]);
        assert_eq!(response.nucleus_type, "TOWER");
        assert!(factory.genome_exists("tower"));
    }

    #[test]
    fn test_compose_success_standalone_type() {
        let temp_dir = TempDir::new().expect("temp dir");
        let factory = GenomeFactory::new(temp_dir.path());

        let genome = GenomeBin::new("custom-primal");
        genome
            .save(&factory.genome_path("custom-primal"))
            .expect("save genome");

        let request = GenomeComposeRequest {
            name: "custom-atomic".to_string(),
            nucleus_type: "STANDALONE".to_string(),
            genomes: vec!["custom-primal".to_string()],
        };

        let result = factory.compose_genome(request);
        let response = result.expect("compose should succeed");
        assert_eq!(response.genome_id, "custom-atomic-atomic");
        assert_eq!(response.embedded_genomes, vec!["custom-primal"]);
    }
}
