// biomeos-genomebin-v3/src/v4.rs
// genomeBin v4.0 Creator - Pure Rust Universal Binary Format
//
// Deep Debt: Binary as DNA, deterministic fingerprint

use crate::{Arch, CompressedBinary, GenomeBin};
use anyhow::{Context, Result};
use std::fs::File;
use std::io::Write;
use std::path::Path;

/// genomeBin v4.0 format constants
const MAGIC: &[u8; 8] = b"GENOME40";
const VERSION: u32 = 4;

impl GenomeBin {
    /// Write genomeBin in v4.0 format (Pure Rust universal extractor)
    ///
    /// Format:
    /// ```text
    /// [Universal Extractor Binary] <- Pure Rust (~741KB)
    /// [MAGIC: "GENOME40"]          <- 8 bytes
    /// [Header]                     <- 60 bytes
    /// [Manifest (compressed)]      <- Variable
    /// [Binary Table]               <- 64 bytes per architecture
    /// [Compressed Binaries]        <- Variable
    /// ```
    ///
    /// This creates a TRUE universal genomeBin with deterministic fingerprint
    pub fn write_v4(&self, output: &Path, extractor_path: &Path) -> Result<()> {
        tracing::info!("🧬 Writing genomeBin v4.0 (Pure Rust Universal)");
        tracing::info!("   Output: {}", output.display());
        tracing::info!("   Extractor: {}", extractor_path.display());
        
        // Read universal extractor binary
        let extractor = std::fs::read(extractor_path)
            .with_context(|| format!("Failed to read extractor: {}", extractor_path.display()))?;
        
        tracing::info!("   Extractor size: {} bytes ({:.1} KB)", 
            extractor.len(), extractor.len() as f64 / 1024.0);
        
        // Prepare manifest (compressed JSON)
        let manifest_json = serde_json::to_vec(&self.manifest)
            .context("Failed to serialize manifest")?;
        
        let manifest_compressed = zstd::encode_all(&manifest_json[..], 3)
            .context("Failed to compress manifest")?;
        
        tracing::info!("   Manifest: {} → {} bytes", 
            manifest_json.len(), manifest_compressed.len());
        
        // Calculate offsets
        let extractor_size = extractor.len() as u64;
        let magic_offset = extractor_size;
        let header_offset = magic_offset + MAGIC.len() as u64;
        let manifest_offset = header_offset + 60; // Header is 60 bytes
        let binaries_table_offset = manifest_offset + manifest_compressed.len() as u64;
        let binaries_data_offset = binaries_table_offset + 
            (self.binaries.len() as u64 * 64); // Each entry is 64 bytes
        
        // Create binary entries
        let mut entries = Vec::new();
        let mut current_offset = 0u64;
        
        for (arch, binary) in &self.binaries {
            let arch_str = format!("{:?}", arch).to_lowercase();
            
            let entry = BinaryEntry {
                architecture: arch_str_to_bytes(&arch_str),
                offset: current_offset,
                compressed_size: binary.size_compressed as u32,
                uncompressed_size: binary.size_original as u32,
                checksum: binary.checksum,
            };
            
            entries.push(entry);
            current_offset += binary.size_compressed;
            
            tracing::info!("   Binary {:?}: {} → {} bytes ({:.1}%)",
                arch,
                binary.size_original,
                binary.size_compressed,
                (binary.size_compressed as f64 / binary.size_original as f64) * 100.0
            );
        }
        
        // Calculate DNA fingerprint (SHA256 of entire payload)
        let fingerprint = calculate_fingerprint(
            &manifest_compressed,
            &entries,
            &self.binaries,
        )?;
        
        tracing::info!("   🧬 DNA Fingerprint: {}", hex::encode(fingerprint));
        
        // Create header
        let header = GenomeHeader {
            version: VERSION,
            manifest_offset: manifest_offset - header_offset,
            manifest_size: manifest_compressed.len() as u32,
            binaries_offset: binaries_table_offset - header_offset,
            num_binaries: self.binaries.len() as u32,
            fingerprint,
        };
        
        // Write genomeBin v4.0
        let mut file = File::create(output)
            .with_context(|| format!("Failed to create output: {}", output.display()))?;
        
        // 1. Write extractor binary
        file.write_all(&extractor)?;
        
        // 2. Write MAGIC
        file.write_all(MAGIC)?;
        
        // 3. Write header
        file.write_all(&header.to_bytes())?;
        
        // 4. Write manifest (compressed)
        file.write_all(&manifest_compressed)?;
        
        // 5. Write binary entries table
        for entry in &entries {
            file.write_all(&entry.to_bytes())?;
        }
        
        // 6. Write compressed binaries
        for binary in self.binaries.values() {
            file.write_all(&binary.data)?;
        }
        
        // Make executable
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = file.metadata()?.permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(output, perms)?;
        }
        
        let total_size = extractor.len() + MAGIC.len() + 60 + 
            manifest_compressed.len() + (entries.len() * 64) + 
            self.binaries.values().map(|b| b.size_compressed as usize).sum::<usize>();
        
        tracing::info!("✅ genomeBin v4.0 written: {} bytes ({:.2} MB)",
            total_size, total_size as f64 / 1_048_576.0);
        tracing::info!("   Format: Pure Rust Universal (binary = DNA)");
        tracing::info!("   🌍 Works on ALL platforms (same file!)");
        tracing::info!("   🧬 Deterministic fingerprint: {}", hex::encode(&fingerprint[..8]));
        
        Ok(())
    }
}

/// GenomeHeader for v4.0 format
#[derive(Debug, Clone)]
struct GenomeHeader {
    version: u32,
    manifest_offset: u64,
    manifest_size: u32,
    binaries_offset: u64,
    num_binaries: u32,
    fingerprint: [u8; 32],
}

impl GenomeHeader {
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(60);
        
        bytes.extend_from_slice(&self.version.to_le_bytes());
        bytes.extend_from_slice(&self.manifest_offset.to_le_bytes());
        bytes.extend_from_slice(&self.manifest_size.to_le_bytes());
        bytes.extend_from_slice(&self.binaries_offset.to_le_bytes());
        bytes.extend_from_slice(&self.num_binaries.to_le_bytes());
        bytes.extend_from_slice(&self.fingerprint);
        
        bytes
    }
}

/// BinaryEntry for v4.0 format
#[derive(Debug, Clone)]
struct BinaryEntry {
    architecture: [u8; 16],
    offset: u64,
    compressed_size: u32,
    uncompressed_size: u32,
    checksum: [u8; 32],
}

impl BinaryEntry {
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(64);
        
        bytes.extend_from_slice(&self.architecture);
        bytes.extend_from_slice(&self.offset.to_le_bytes());
        bytes.extend_from_slice(&self.compressed_size.to_le_bytes());
        bytes.extend_from_slice(&self.uncompressed_size.to_le_bytes());
        bytes.extend_from_slice(&self.checksum);
        
        bytes
    }
}

/// Convert architecture string to fixed 16-byte array
fn arch_str_to_bytes(arch: &str) -> [u8; 16] {
    let mut bytes = [0u8; 16];
    let arch_bytes = arch.as_bytes();
    let len = arch_bytes.len().min(16);
    bytes[..len].copy_from_slice(&arch_bytes[..len]);
    bytes
}

/// Calculate DNA fingerprint (SHA256 of entire payload)
fn calculate_fingerprint(
    manifest: &[u8],
    entries: &[BinaryEntry],
    binaries: &std::collections::HashMap<Arch, CompressedBinary>,
) -> Result<[u8; 32]> {
    use sha2::{Sha256, Digest};
    
    let mut hasher = Sha256::new();
    
    // Hash manifest
    hasher.update(manifest);
    
    // Hash binary entries
    for entry in entries {
        hasher.update(&entry.to_bytes());
    }
    
    // Hash binary data
    for binary in binaries.values() {
        hasher.update(&binary.data);
    }
    
    Ok(hasher.finalize().into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::io::Write;
    
    #[test]
    fn test_arch_str_to_bytes() {
        let bytes = arch_str_to_bytes("x86_64");
        assert_eq!(&bytes[..6], b"x86_64");
        assert_eq!(&bytes[6..], &[0u8; 10]);
    }
    
    #[test]
    fn test_header_serialization() {
        let header = GenomeHeader {
            version: 4,
            manifest_offset: 1000,
            manifest_size: 500,
            binaries_offset: 2000,
            num_binaries: 2,
            fingerprint: [42u8; 32],
        };
        
        let bytes = header.to_bytes();
        assert_eq!(bytes.len(), 60);
    }
}
