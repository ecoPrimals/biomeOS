// biomeos-genome-factory/src/create.rs
// Create genomeBin from binaries
//
// Deep Debt: Universal production for ANY primal

use crate::GenomeFactory;
use anyhow::{Context, Result};
use biomeos_genomebin_v3::{Arch, GenomeBinBuilder};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Request to create genomeBin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenomeCreateRequest {
    /// Primal name
    pub name: String,
    
    /// Binaries per architecture
    pub binaries: HashMap<String, PathBuf>,
    
    /// Metadata
    #[serde(default)]
    pub metadata: GenomeMetadata,
}

/// Genome metadata
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GenomeMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nucleus_atomic: Option<String>,
    
    #[serde(default)]
    pub capabilities: Vec<String>,
}

/// Response after creating genome
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenomeCreateResponse {
    pub genome_id: String,
    pub path: PathBuf,
    pub size: u64,
    pub architectures: Vec<String>,
    pub checksums: HashMap<String, String>,
}

impl GenomeFactory {
    /// Create genomeBin from request
    pub fn create_genome(&self, request: GenomeCreateRequest) -> Result<GenomeCreateResponse> {
        tracing::info!("🧬 Creating genomeBin: {}", request.name);
        
        // Validate request
        if request.binaries.is_empty() {
            anyhow::bail!("No binaries provided");
        }
        
        // Build genomeBin
        let mut builder = GenomeBinBuilder::new(&request.name);
        
        if let Some(version) = request.metadata.version {
            builder = builder.version(version);
        }
        
        if let Some(description) = request.metadata.description {
            builder = builder.description(description);
        }
        
        if let Some(nucleus) = request.metadata.nucleus_atomic {
            builder = builder.nucleus_atomic(nucleus);
        }
        
        for capability in request.metadata.capabilities {
            builder = builder.capability(capability);
        }
        
        // Add binaries
        for (arch_str, binary_path) in &request.binaries {
            let arch = Arch::from_str(arch_str)
                .ok_or_else(|| anyhow::anyhow!("Unknown architecture: {}", arch_str))?;
            
            if !binary_path.exists() {
                anyhow::bail!("Binary not found: {}", binary_path.display());
            }
            
            builder = builder.add_binary(arch, binary_path.clone());
        }
        
        let genome = builder.build()
            .context("Failed to build genomeBin")?;
        
        // Write to storage
        let output_path = self.genome_path(&request.name);
        genome.write(&output_path)
            .with_context(|| format!("Failed to write genomeBin: {}", output_path.display()))?;
        
        // Collect checksums
        let mut checksums = HashMap::new();
        for (arch, compressed) in &genome.binaries {
            checksums.insert(
                format!("{:?}", arch),
                hex::encode(compressed.checksum)
            );
        }
        
        let response = GenomeCreateResponse {
            genome_id: format!("{}-{}", request.name, genome.manifest.version),
            path: output_path,
            size: genome.total_size(),
            architectures: genome.manifest.architectures.iter()
                .map(|a| format!("{:?}", a))
                .collect(),
            checksums,
        };
        
        tracing::info!(
            "✅ genomeBin created: {} ({} bytes, {} architectures)",
            response.genome_id,
            response.size,
            response.architectures.len()
        );
        
        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::{TempDir, NamedTempFile};
    
    #[test]
    fn test_create_genome_basic() {
        let temp_dir = TempDir::new().unwrap();
        let factory = GenomeFactory::new(temp_dir.path()).unwrap();
        
        // Create test binary
        let mut binary_file = NamedTempFile::new().unwrap();
        binary_file.write_all(b"test binary content").unwrap();
        binary_file.flush().unwrap();
        
        let mut binaries = HashMap::new();
        binaries.insert("x86_64".to_string(), binary_file.path().to_path_buf());
        
        let request = GenomeCreateRequest {
            name: "test-primal".to_string(),
            binaries,
            metadata: GenomeMetadata {
                version: Some("1.0.0".to_string()),
                description: Some("Test primal".to_string()),
                ..Default::default()
            },
        };
        
        let response = factory.create_genome(request).unwrap();
        
        assert_eq!(response.genome_id, "test-primal-1.0.0");
        assert!(response.path.exists());
        assert_eq!(response.architectures.len(), 1);
        assert!(response.checksums.contains_key("X86_64"));
    }
}
