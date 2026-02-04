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
    pub genome_id: String,
    pub path: PathBuf,
    pub size: u64,
    pub embedded_genomes: Vec<String>,
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
                .with_context(|| format!("Failed to load genome: {}", genome_name))?;

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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_compose_empty() {
        let temp_dir = TempDir::new().unwrap();
        let factory = GenomeFactory::new(temp_dir.path());

        let request = GenomeComposeRequest {
            name: "test".to_string(),
            nucleus_type: "TOWER".to_string(),
            genomes: vec![],
        };

        // Empty genomes should fail
        let result = factory.compose_genome(request);
        assert!(result.is_err());
    }
}
