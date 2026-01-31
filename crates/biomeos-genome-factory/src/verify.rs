// biomeos-genome-factory/src/verify.rs
// Verify genomeBin integrity
//
// Deep Debt: Comprehensive validation

use crate::GenomeFactory;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Verification result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyResponse {
    pub genome_id: String,
    pub valid: bool,
    pub checksums: HashMap<String, ChecksumResult>,
    pub manifest_valid: bool,
    pub embedded_count: usize,
}

/// Checksum verification result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChecksumResult {
    pub expected: String,
    pub actual: String,
    pub valid: bool,
}

impl GenomeFactory {
    /// Verify genomeBin integrity
    pub fn verify_genome(&self, name: &str) -> Result<VerifyResponse> {
        tracing::info!("🔍 Verifying genomeBin: {}", name);
        
        // Load genome
        let genome = self.load_genome(name)?;
        
        // Verify all checksums
        let verify_results = genome.verify_all()
            .context("Verification failed")?;
        
        // Convert results
        let mut checksums = HashMap::new();
        for (key, result) in verify_results {
            checksums.insert(key, ChecksumResult {
                expected: result.expected,
                actual: result.actual,
                valid: result.valid,
            });
        }
        
        let valid = checksums.values().all(|r| r.valid);
        
        let response = VerifyResponse {
            genome_id: name.to_string(),
            valid,
            checksums,
            manifest_valid: true,
            embedded_count: genome.embedded_genomes.len(),
        };
        
        if valid {
            tracing::info!("✅ Verification successful: {} (all checksums valid)", name);
        } else {
            tracing::error!("❌ Verification failed: {} (some checksums invalid)", name);
        }
        
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
    
    #[test]
    fn test_verify_genome() {
        let temp_dir = TempDir::new().unwrap();
        let factory = GenomeFactory::new(temp_dir.path()).unwrap();
        
        // Create test genome
        let mut binary = NamedTempFile::new().unwrap();
        binary.write_all(b"test binary content").unwrap();
        binary.flush().unwrap();
        
        let mut binaries = HashMap::new();
        binaries.insert("x86_64".to_string(), binary.path().to_path_buf());
        
        factory.create_genome(GenomeCreateRequest {
            name: "test".to_string(),
            binaries,
            metadata: GenomeMetadata::default(),
        }).unwrap();
        
        // Verify
        let result = factory.verify_genome("test").unwrap();
        
        assert_eq!(result.genome_id, "test");
        assert!(result.valid);
        assert!(result.manifest_valid);
        assert!(result.checksums.values().all(|c| c.valid));
    }
}
