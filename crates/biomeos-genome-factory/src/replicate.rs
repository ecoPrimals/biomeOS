// biomeos-genome-factory/src/replicate.rs
// Self-replication - biomeOS creates its own genomeBin
//
// Deep Debt: Autonomous reproduction

use crate::GenomeFactory;
use anyhow::{Context, Result};
use biomeos_genomebin_v3::{Arch, GenomeBinBuilder};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Response after self-replication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfReplicateResponse {
    pub genome_id: String,
    pub path: PathBuf,
    pub size: u64,
    pub architectures: Vec<String>,
}

impl GenomeFactory {
    /// Self-replicate: Create biomeOS's own genomeBin
    ///
    /// Deep Debt: Introspection + self-knowledge
    pub fn self_replicate(&self) -> Result<SelfReplicateResponse> {
        tracing::info!("🧬 Self-replication initiated: biomeOS creating its own genomeBin");
        
        // Find current executable
        let self_binary = std::env::current_exe()
            .context("Failed to get current executable path")?;
        
        tracing::debug!("Found self binary: {}", self_binary.display());
        
        // Detect current architecture
        let arch = Arch::detect();
        
        // Build self-genomeBin
        let genome = GenomeBinBuilder::new("biomeos")
            .version(env!("CARGO_PKG_VERSION"))
            .description("biomeOS System Orchestrator - Self-Replicated")
            .nucleus_atomic("ORCHESTRATOR")
            .capability("orchestration")
            .capability("genome-factory")
            .capability("federation")
            .capability("self-replication")
            .add_binary(arch, self_binary)
            .build()
            .context("Failed to build self-genomeBin")?;
        
        // Write to storage
        let output_path = self.genome_path("biomeos-self");
        genome.write(&output_path)
            .context("Failed to write self-genomeBin")?;
        
        let response = SelfReplicateResponse {
            genome_id: "biomeos-self".to_string(),
            path: output_path,
            size: genome.total_size(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_self_replicate() {
        let temp_dir = TempDir::new().unwrap();
        let factory = GenomeFactory::new(temp_dir.path()).unwrap();
        
        let response = factory.self_replicate().unwrap();
        
        assert_eq!(response.genome_id, "biomeos-self");
        assert!(response.path.exists());
        assert_eq!(response.architectures.len(), 1);
        assert!(factory.has_self_genome());
    }
}
