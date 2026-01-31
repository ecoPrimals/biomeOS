// biomeos-genomebin-v3/src/lib.rs
// TRUE Binary Isomorphic genomeBin v3.0
// 
// Design Principles (Deep Debt):
// - 100% Pure Rust (zero C dependencies)
// - Zero unsafe code
// - Modern idiomatic Rust
// - Platform-agnostic (runtime discovery)
// - Self-contained (no external tools)
// - Capability-based (no hardcoding)

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

pub mod arch;
pub mod builder;
pub mod composer;
pub mod manifest;
pub mod runtime;
pub mod verify;

// Re-exports for convenience
pub use arch::Arch;
pub use builder::GenomeBinBuilder;
pub use composer::GenomeBinComposer;
pub use manifest::GenomeManifest;

/// Compressed binary with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressedBinary {
    /// Target architecture
    pub arch: Arch,
    
    /// Compressed binary data (zstd)
    pub data: Vec<u8>,
    
    /// SHA256 checksum of ORIGINAL (uncompressed) binary
    pub checksum: [u8; 32],
    
    /// Original size (bytes)
    pub size_original: u64,
    
    /// Compressed size (bytes)
    pub size_compressed: u64,
}

impl CompressedBinary {
    /// Compress binary from file
    pub fn from_file(arch: Arch, path: &Path) -> Result<Self> {
        tracing::info!("Compressing binary for {:?} from: {}", arch, path.display());
        
        // Read original binary
        let data = std::fs::read(path)
            .with_context(|| format!("Failed to read binary: {}", path.display()))?;
        
        let size_original = data.len() as u64;
        
        // Calculate SHA256 of original
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(&data);
        let checksum: [u8; 32] = hasher.finalize().into();
        
        // Compress with zstd (level 3 - good balance of speed/ratio)
        let compressed = zstd::encode_all(&data[..], 3)
            .context("Failed to compress binary with zstd")?;
        
        let size_compressed = compressed.len() as u64;
        let ratio = (size_compressed as f64 / size_original as f64) * 100.0;
        
        tracing::info!(
            "Compressed {:?}: {} → {} bytes ({:.1}% of original)",
            arch, size_original, size_compressed, ratio
        );
        
        Ok(Self {
            arch,
            data: compressed,
            checksum,
            size_original,
            size_compressed,
        })
    }
    
    /// Decompress binary
    pub fn decompress(&self) -> Result<Vec<u8>> {
        tracing::debug!("Decompressing {:?} binary ({} bytes)", self.arch, self.size_compressed);
        
        let decompressed = zstd::decode_all(&self.data[..])
            .context("Failed to decompress binary with zstd")?;
        
        // Verify size
        if decompressed.len() as u64 != self.size_original {
            anyhow::bail!(
                "Size mismatch after decompression: expected {}, got {}",
                self.size_original,
                decompressed.len()
            );
        }
        
        // Verify checksum
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(&decompressed);
        let checksum: [u8; 32] = hasher.finalize().into();
        
        if checksum != self.checksum {
            anyhow::bail!(
                "Checksum mismatch: expected {}, got {}",
                hex::encode(self.checksum),
                hex::encode(checksum)
            );
        }
        
        tracing::debug!("Decompression verified: {} bytes", decompressed.len());
        Ok(decompressed)
    }
}

/// Main genomeBin structure - TRUE binary isomorphic format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenomeBin {
    /// Manifest with metadata
    pub manifest: GenomeManifest,
    
    /// Multi-arch binaries (compressed)
    pub binaries: HashMap<Arch, CompressedBinary>,
    
    /// Embedded genomeBins (for fractal composition)
    pub embedded_genomes: Vec<GenomeBin>,
}

impl GenomeBin {
    /// Create new genomeBin
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            manifest: GenomeManifest::new(name),
            binaries: HashMap::new(),
            embedded_genomes: Vec::new(),
        }
    }
    
    /// Add binary for architecture
    pub fn add_binary(&mut self, arch: Arch, binary_path: &Path) -> Result<()> {
        tracing::info!("Adding {:?} binary from: {}", arch, binary_path.display());
        
        let compressed = CompressedBinary::from_file(arch, binary_path)?;
        
        self.binaries.insert(arch, compressed);
        
        // Update manifest architectures
        if !self.manifest.architectures.contains(&arch) {
            self.manifest.architectures.push(arch);
        }
        
        Ok(())
    }
    
    /// Embed another genomeBin (fractal composition)
    pub fn embed(&mut self, genome: GenomeBin) -> Result<()> {
        tracing::info!("Embedding genomeBin: {}", genome.manifest.name);
        self.embedded_genomes.push(genome);
        Ok(())
    }
    
    /// Get total size (all binaries + metadata)
    pub fn total_size(&self) -> u64 {
        let binaries_size: u64 = self.binaries.values()
            .map(|b| b.size_compressed)
            .sum();
        
        let embedded_size: u64 = self.embedded_genomes.iter()
            .map(|g| g.total_size())
            .sum();
        
        // Rough estimate for manifest (JSON serialized)
        let manifest_size = serde_json::to_string(&self.manifest)
            .map(|s| s.len() as u64)
            .unwrap_or(1024);
        
        binaries_size + embedded_size + manifest_size
    }
    
    /// Write genomeBin to file (self-extracting binary)
    pub fn write(&self, output: &Path) -> Result<()> {
        tracing::info!("Writing genomeBin to: {}", output.display());
        
        // Serialize the entire genomeBin
        let payload = bincode::serialize(&(
            &self.manifest,
            &self.binaries,
            &self.embedded_genomes,
        )).context("Failed to serialize genomeBin")?;
        
        tracing::info!(
            "genomeBin payload: {} bytes ({} binaries, {} embedded)",
            payload.len(),
            self.binaries.len(),
            self.embedded_genomes.len()
        );
        
        // TODO: In full implementation, prepend runtime stub binary
        // For now, just write payload with marker
        let marker = b"__GENOME_PAYLOAD__\n";
        
        let mut file = std::fs::File::create(output)
            .with_context(|| format!("Failed to create output file: {}", output.display()))?;
        
        use std::io::Write;
        file.write_all(marker)?;
        file.write_all(&payload)?;
        
        // Make executable on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = file.metadata()?.permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(output, perms)?;
        }
        
        tracing::info!("✅ genomeBin written: {} bytes", payload.len() + marker.len());
        Ok(())
    }
    
    /// Load genomeBin from file
    pub fn from_file(path: &Path) -> Result<Self> {
        tracing::info!("Loading genomeBin from: {}", path.display());
        
        let contents = std::fs::read(path)
            .with_context(|| format!("Failed to read genomeBin: {}", path.display()))?;
        
        // Find payload marker
        let marker = b"__GENOME_PAYLOAD__\n";
        let marker_pos = contents.windows(marker.len())
            .position(|window| window == marker)
            .context("Payload marker not found in genomeBin")?;
        
        // Deserialize payload
        let payload_start = marker_pos + marker.len();
        let (manifest, binaries, embedded_genomes): (GenomeManifest, HashMap<Arch, CompressedBinary>, Vec<GenomeBin>) = 
            bincode::deserialize(&contents[payload_start..])
            .context("Failed to deserialize genomeBin payload")?;
        
        tracing::info!("✅ genomeBin loaded: {}", manifest.name);
        
        Ok(Self {
            manifest,
            binaries,
            embedded_genomes,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_genomebin_creation() {
        let genome = GenomeBin::new("test-primal");
        assert_eq!(genome.manifest.name, "test-primal");
        assert_eq!(genome.binaries.len(), 0);
        assert_eq!(genome.embedded_genomes.len(), 0);
    }
    
    #[test]
    fn test_genomebin_total_size() {
        let genome = GenomeBin::new("test-primal");
        let size = genome.total_size();
        assert!(size > 0, "Total size should include manifest");
    }
}
