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
pub mod v4;    // genomeBin v4.0 Pure Rust format
pub mod v4_1;  // genomeBin v4.1 Multi-architecture fat binary

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
    /// Uses Rust stub (v3.0 - DEPRECATED, platform-specific)
    pub fn write(&self, output: &Path) -> Result<()> {
        tracing::warn!("write() is DEPRECATED - uses platform-specific Rust stub");
        tracing::warn!("Use write_universal() for TRUE universal deployment");
        self.write_with_stub(output)
    }
    
    /// Write genomeBin with universal shell script wrapper (v3.5 - Transitional)
    /// This is the recommended method for universal deployment
    /// NOTE: This is a temporary solution - evolving to Pure Rust in v4.0
    pub fn write_universal(&self, output: &Path) -> Result<()> {
        tracing::info!("Writing UNIVERSAL genomeBin to: {}", output.display());
        tracing::info!("Using shell script wrapper (v3.5 - Transitional to Pure Rust v4)");
        
        use std::io::Write;
        
        // Create manifest JSON with architecture offsets
        let manifest_json = self.create_manifest_json()?;
        
        // Get universal shell script extractor
        let script_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("universal-extractor.sh");
        
        let mut script = std::fs::read_to_string(&script_path)
            .with_context(|| format!("Failed to read universal extractor: {}", script_path.display()))?;
        
        // Calculate script size (will be embedded in script itself)
        let script_size = script.len();
        script = script.replace("__SCRIPT_SIZE__", &script_size.to_string());
        
        // Open output file
        let mut file = std::fs::File::create(output)
            .with_context(|| format!("Failed to create output file: {}", output.display()))?;
        
        // Write shell script (with shebang and size embedded)
        file.write_all(script.as_bytes())?;
        
        // Write manifest JSON (padded to 8KB for easy parsing)
        let manifest_bytes = manifest_json.as_bytes();
        let manifest_padded = if manifest_bytes.len() < 8192 {
            let mut padded = manifest_bytes.to_vec();
            padded.resize(8192, b' ');
            padded
        } else {
            anyhow::bail!("Manifest too large: {} bytes (max 8KB)", manifest_bytes.len());
        };
        
        file.write_all(&manifest_padded)?;
        
        // Write compressed binaries for each architecture
        let mut current_offset = 8192u64; // After manifest
        for arch in [Arch::X86_64, Arch::Aarch64, Arch::Riscv64] {
            if let Some(binary) = self.binaries.get(&arch) {
                tracing::info!("Writing {:?} binary at offset {}", arch, current_offset);
                file.write_all(&binary.data)?;
                current_offset += binary.size_compressed;
            }
        }
        
        // Make executable on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = file.metadata()?.permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(output, perms)?;
        }
        
        let total_size = script.len() + manifest_padded.len() + 
            self.binaries.values().map(|b| b.size_compressed as usize).sum::<usize>();
        
        tracing::info!("✅ UNIVERSAL genomeBin written: {} bytes", total_size);
        tracing::info!("   Script: {} bytes, Manifest: 8KB, Binaries: {} bytes",
            script.len(),
            self.binaries.values().map(|b| b.size_compressed).sum::<u64>()
        );
        tracing::info!("   🌍 Works on: Linux x86_64, ARM64, RISC-V, macOS, BSD, Android");
        tracing::info!("   ⚠️  Temporary shell solution - evolving to Pure Rust v4");
        
        Ok(())
    }
    
    /// Create manifest JSON with architecture metadata
    fn create_manifest_json(&self) -> Result<String> {
        use serde_json::json;
        
        let mut binaries_meta = serde_json::Map::new();
        let mut current_offset = 8192u64; // After manifest
        
        for (arch, binary) in &self.binaries {
            let arch_str = match arch {
                Arch::X86_64 => "x86_64",
                Arch::Aarch64 => "aarch64",
                Arch::Riscv64 => "riscv64",
                _ => continue,
            };
            
            binaries_meta.insert(arch_str.to_string(), json!({
                "offset": current_offset,
                "compressed_size": binary.size_compressed,
                "uncompressed_size": binary.size_original,
                "checksum": hex::encode(binary.checksum),
            }));
            
            current_offset += binary.size_compressed;
        }
        
        let manifest = json!({
            "name": self.manifest.name,
            "version": self.manifest.version,
            "description": self.manifest.description,
            "format_version": "3.5-universal",
            "architectures": self.manifest.architectures.iter()
                .map(|a| format!("{:?}", a).to_lowercase())
                .collect::<Vec<_>>(),
            "binaries": binaries_meta,
        });
        
        Ok(serde_json::to_string_pretty(&manifest)?)
    }
    
    /// Internal method: Write with Rust stub (platform-specific)
    fn write_with_stub(&self, output: &Path) -> Result<()> {
        tracing::info!("Writing genomeBin to: {}", output.display());
        
        // Serialize embedded genomes as Vec<Vec<u8>> for the stub to deserialize
        let embedded_serialized: Vec<Vec<u8>> = self.embedded_genomes
            .iter()
            .map(|genome| {
                bincode::serialize(&(
                    &genome.manifest,
                    &genome.binaries,
                    &genome.embedded_genomes,
                )).expect("Failed to serialize embedded genome")
            })
            .collect();
        
        // Serialize the entire genomeBin
        let payload = bincode::serialize(&(
            &self.manifest,
            &self.binaries,
            &embedded_serialized,
        )).context("Failed to serialize genomeBin")?;
        
        tracing::info!(
            "genomeBin payload: {} bytes ({} binaries, {} embedded)",
            payload.len(),
            self.binaries.len(),
            self.embedded_genomes.len()
        );
        
        // Get self-extracting stub binary
        // First try to read from build location
        let stub_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("stub/target/x86_64-unknown-linux-musl/release/genomebin-stub");
        
        let stub_binary = if stub_path.exists() {
            tracing::debug!("Loading stub from: {}", stub_path.display());
            std::fs::read(&stub_path)
                .with_context(|| format!("Failed to read stub binary: {}", stub_path.display()))?
        } else {
            // Fallback: Try embedded stub (if available)
            #[cfg(feature = "embedded-stub")]
            {
                tracing::debug!("Using embedded stub binary");
                include_bytes!("../stub/target/x86_64-unknown-linux-musl/release/genomebin-stub").to_vec()
            }
            #[cfg(not(feature = "embedded-stub"))]
            {
                anyhow::bail!(
                    "Stub binary not found at: {}\nPlease build the stub first:\n  cd crates/biomeos-genomebin-v3/stub && cargo build --release",
                    stub_path.display()
                );
            }
        };
        
        tracing::info!("Using stub binary: {} bytes", stub_binary.len());
        
        // Payload marker
        let marker = b"__GENOME_PAYLOAD__\n";
        
        // Write: [stub] + [marker] + [payload]
        let mut file = std::fs::File::create(output)
            .with_context(|| format!("Failed to create output file: {}", output.display()))?;
        
        use std::io::Write;
        file.write_all(&stub_binary)?;
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
        
        let total_size = stub_binary.len() + marker.len() + payload.len();
        tracing::info!("✅ genomeBin written: {} bytes (stub: {}, payload: {})",
            total_size, stub_binary.len(), payload.len());
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
