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
        let mut composer = GenomeBinComposer::new(&request.name)
            .nucleus_type(&request.nucleus_type);
        
        let mut genome_names = Vec::new();
        for genome_name in &request.genomes {
            let genome = self.load_genome(genome_name)
                .with_context(|| format!("Failed to load genome: {}", genome_name))?;
            
            genome_names.push(genome.manifest.name.clone());
            composer = composer.add_genome(genome);
        }
        
        // Build composed genome
        let composed = composer.build()
            .context("Failed to compose genomeBin")?;
        
        // Write to storage
        let output_path = self.genome_path(&request.name);
        composed.write(&output_path)
            .with_context(|| format!("Failed to write composed genome: {}", output_path.display()))?;
        
        let response = GenomeComposeResponse {
            genome_id: format!("{}-atomic", request.name),
            path: output_path,
            size: composed.total_size(),
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
    use crate::create::{GenomeCreateRequest, GenomeMetadata};
    use std::collections::HashMap;
    use std::io::Write;
    use tempfile::{TempDir, NamedTempFile};
    
    fn create_test_genome(factory: &GenomeFactory, name: &str) -> Result<()> {
        let mut binary = NamedTempFile::new()?;
        binary.write_all(b"test binary")?;
        binary.flush()?;
        
        let mut binaries = HashMap::new();
        binaries.insert("x86_64".to_string(), binary.path().to_path_buf());
        
        factory.create_genome(GenomeCreateRequest {
            name: name.to_string(),
            binaries,
            metadata: GenomeMetadata::default(),
        })?;
        
        Ok(())
    }
    
    #[test]
    fn test_compose_tower() {
        let temp_dir = TempDir::new().unwrap();
        let factory = GenomeFactory::new(temp_dir.path()).unwrap();
        
        // Create test genomes
        create_test_genome(&factory, "beardog").unwrap();
        create_test_genome(&factory, "songbird").unwrap();
        
        let request = GenomeComposeRequest {
            name: "tower".to_string(),
            nucleus_type: "TOWER".to_string(),
            genomes: vec!["beardog".to_string(), "songbird".to_string()],
        };
        
        let response = factory.compose_genome(request).unwrap();
        
        assert_eq!(response.genome_id, "tower-atomic");
        assert_eq!(response.nucleus_type, "TOWER");
        assert_eq!(response.embedded_genomes.len(), 2);
        assert!(response.path.exists());
    }
}
