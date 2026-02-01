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
    
    /// Use universal shell script wrapper (v3.5) instead of Rust stub (v3.0)
    /// Recommended for TRUE cross-platform deployment (USB + Pixel identical code)
    #[serde(default = "default_universal")]
    pub universal: bool,
    
    /// Use Pure Rust v4.0 format (PRODUCTION - binary = DNA)
    /// This is the TRUE genomic architecture with deterministic fingerprint
    #[serde(default)]
    pub v4: bool,
    
    /// Use Multi-Architecture Fat Binary v4.1 (UNIVERSAL - genomeBin standard)
    /// Creates single file with embedded extractors for multiple architectures
    #[serde(default)]
    pub v4_1: bool,
    
    /// Extractor architectures for v4.1 (e.g., ["x86_64", "aarch64"])
    #[serde(default)]
    pub extractor_arches: Vec<String>,
}

fn default_universal() -> bool {
    true  // Default to universal (Deep Debt compliant)
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
            let arch = arch_str.parse::<Arch>()
                .map_err(|e| anyhow::anyhow!("Invalid architecture '{}': {}", arch_str, e))?;
            
            if !binary_path.exists() {
                anyhow::bail!("Binary not found: {}", binary_path.display());
            }
            
            builder = builder.add_binary(arch, binary_path.clone());
        }
        
        let genome = builder.build()
            .context("Failed to build genomeBin")?;
        
        // Write to storage
        let output_path = self.genome_path(&request.name);
        
        // Choose format: v4.1 (fat bin) > v4.0 (Pure Rust) > v3.5 (shell) > v3.0 (stub)
        if request.v4_1 {
            tracing::info!("Using MULTI-ARCH FAT BINARY v4.1 format (UNIVERSAL - genomeBin standard)");
            
            // Build extractor paths map
            let mut extractor_paths = HashMap::new();
            
            for arch_str in &request.extractor_arches {
                let arch = arch_str.parse::<Arch>()
                    .map_err(|e| anyhow::anyhow!("Invalid extractor architecture '{}': {}", arch_str, e))?;
                
                // Runtime discovery: find extractor for this architecture
                let target_triple = match arch {
                    Arch::X86_64 => "x86_64-unknown-linux-musl",
                    Arch::Aarch64 => "aarch64-unknown-linux-musl",
                    Arch::Riscv64 => "riscv64gc-unknown-linux-musl",
                    _ => anyhow::bail!("Unsupported extractor architecture: {:?}", arch),
                };
                
                let extractor_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .parent().unwrap()
                    .parent().unwrap()
                    .join(format!("target/{}/release/genome-extract", target_triple));
                
                if !extractor_path.exists() {
                    anyhow::bail!(
                        "v4.1 extractor not found for {}: {}\nPlease build it first:\n  cargo build --release --target {} -p biomeos-genome-extract",
                        arch_str,
                        extractor_path.display(),
                        target_triple
                    );
                }
                
                tracing::info!("  Found {} extractor: {}", arch_str, extractor_path.display());
                extractor_paths.insert(arch, extractor_path);
            }
            
            if extractor_paths.is_empty() {
                anyhow::bail!("No extractor architectures specified for v4.1. Use --extractor-arches x86_64,aarch64");
            }
            
            // Convert PathBuf to &Path for the API
            let extractors: HashMap<Arch, &std::path::Path> = extractor_paths.iter()
                .map(|(arch, path)| (*arch, path.as_path()))
                .collect();
            
            genome.write_v4_1(&output_path, &extractors)
                .with_context(|| format!("Failed to write v4.1 genomeBin: {}", output_path.display()))?;
            
        } else if request.v4 {
            tracing::info!("Using PURE RUST v4.0 format (binary = DNA)");
            
            // Get extractor path
            let extractor_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .parent().unwrap()
                .parent().unwrap()
                .join("target/x86_64-unknown-linux-musl/release/genome-extract");
            
            if !extractor_path.exists() {
                anyhow::bail!(
                    "v4.0 extractor not found: {}\nPlease build it first:\n  cargo build --release --target x86_64-unknown-linux-musl -p biomeos-genome-extract",
                    extractor_path.display()
                );
            }
            
            genome.write_v4(&output_path, &extractor_path)
                .with_context(|| format!("Failed to write v4.0 genomeBin: {}", output_path.display()))?;
            
        } else if request.universal {
            tracing::info!("Using UNIVERSAL shell wrapper (v3.5 - Deep Debt compliant)");
            genome.write_universal(&output_path)
                .with_context(|| format!("Failed to write universal genomeBin: {}", output_path.display()))?;
        } else {
            tracing::warn!("Using legacy Rust stub (v3.0 - platform-specific)");
            tracing::warn!("Recommend --v4 or --universal flag for cross-platform deployment");
            genome.write(&output_path)
                .with_context(|| format!("Failed to write genomeBin: {}", output_path.display()))?;
        }
        
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
