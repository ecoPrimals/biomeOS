// biomeos-genome-factory/src/lib.rs
// Genome Factory - DNA Replicase for ecoPrimals ecosystem
//
// Design Principles (Deep Debt):
// - Universal: Produce genomeBin for ANY primal
// - Fractal: Compose atomics from genomes
// - Self-aware: biomeOS can replicate itself
// - Federation: Exchange genomes P2P
// - Pure Rust: Zero external dependencies

use anyhow::{Context, Result};
use biomeos_genomebin_v3::GenomeBin;
use std::path::{Path, PathBuf};

pub mod create;
pub mod compose;
pub mod replicate;
pub mod verify;

pub use create::{GenomeCreateRequest, GenomeCreateResponse};
pub use compose::{GenomeComposeRequest, GenomeComposeResponse};
pub use replicate::SelfReplicateResponse;
pub use verify::{VerifyResponse, ChecksumResult};

/// Genome Factory - orchestrates genomeBin production
pub struct GenomeFactory {
    /// Storage directory for genomeBins (typically plasmidBin/)
    storage_dir: PathBuf,
}

impl GenomeFactory {
    /// Create new genome factory
    pub fn new(storage_dir: impl Into<PathBuf>) -> Result<Self> {
        let storage_dir = storage_dir.into();
        
        // Ensure storage directory exists
        std::fs::create_dir_all(&storage_dir)
            .with_context(|| format!("Failed to create storage directory: {}", storage_dir.display()))?;
        
        tracing::info!("🧬 Genome Factory initialized: {}", storage_dir.display());
        
        Ok(Self { storage_dir })
    }
    
    /// Get storage directory
    pub fn storage_dir(&self) -> &Path {
        &self.storage_dir
    }
    
    /// Get path for genome by name
    pub fn genome_path(&self, name: &str) -> PathBuf {
        self.storage_dir.join(format!("{}.genome", name))
    }
    
    /// Check if genome exists
    pub fn genome_exists(&self, name: &str) -> bool {
        self.genome_path(name).exists()
    }
    
    /// List all genomes in storage
    pub fn list_genomes(&self) -> Result<Vec<String>> {
        let mut genomes = Vec::new();
        
        for entry in std::fs::read_dir(&self.storage_dir)
            .context("Failed to read storage directory")?
        {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().and_then(|e| e.to_str()) == Some("genome") {
                if let Some(name) = path.file_stem().and_then(|n| n.to_str()) {
                    genomes.push(name.to_string());
                }
            }
        }
        
        genomes.sort();
        Ok(genomes)
    }
    
    /// Load genome from storage
    pub fn load_genome(&self, name: &str) -> Result<GenomeBin> {
        let path = self.genome_path(name);
        tracing::info!("Loading genome: {}", name);
        
        GenomeBin::from_file(&path)
            .with_context(|| format!("Failed to load genome: {}", name))
    }
    
    /// Create factory with default workspace storage location
    ///
    /// Uses workspace root + plasmidBin/ as storage directory.
    /// This method can fail if workspace root cannot be found.
    /// 
    /// Deep Debt: Runtime discovery, no hardcoding
    pub fn with_default_storage() -> Result<Self> {
        // Try to find workspace root
        let workspace_root = Self::find_workspace_root()?;
        let storage_dir = workspace_root.join("plasmidBin");
        
        Self::new(storage_dir)
    }
    
    /// Find workspace root (looks for Cargo.toml with [workspace])
    ///
    /// Deep Debt: Runtime discovery
    fn find_workspace_root() -> Result<PathBuf> {
        let current = std::env::current_dir()
            .context("Failed to get current directory")?;
        
        // Walk up looking for workspace Cargo.toml
        let mut path = current.as_path();
        loop {
            let cargo_toml = path.join("Cargo.toml");
            if cargo_toml.exists() {
                // Check if it's a workspace
                let contents = std::fs::read_to_string(&cargo_toml)?;
                if contents.contains("[workspace]") {
                    return Ok(path.to_path_buf());
                }
            }
            
            match path.parent() {
                Some(parent) => path = parent,
                None => break,
            }
        }
        
        // Fallback: current directory
        tracing::warn!("Workspace root not found, using current directory");
        Ok(current)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_factory_creation() {
        let temp_dir = TempDir::new().unwrap();
        let factory = GenomeFactory::new(temp_dir.path()).unwrap();
        assert_eq!(factory.storage_dir(), temp_dir.path());
    }
    
    #[test]
    fn test_genome_path() {
        let temp_dir = TempDir::new().unwrap();
        let factory = GenomeFactory::new(temp_dir.path()).unwrap();
        
        let path = factory.genome_path("beardog");
        assert_eq!(path.file_name().unwrap(), "beardog.genome");
    }
    
    #[test]
    fn test_list_empty() {
        let temp_dir = TempDir::new().unwrap();
        let factory = GenomeFactory::new(temp_dir.path()).unwrap();
        
        let genomes = factory.list_genomes().unwrap();
        assert_eq!(genomes.len(), 0);
    }
}
