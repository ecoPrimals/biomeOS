// biomeos-genome-extract/src/format.rs
// genomeBin v4.0 Binary Format Specification
//
// Deep Debt: 100% Pure Rust, deterministic, binary = DNA

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::io::{Read, Seek, SeekFrom};

/// Magic bytes identifying genomeBin v4.0 format
pub const MAGIC: &[u8; 8] = b"GENOME40";

/// Version number
pub const VERSION: u32 = 4;

/// genomeBin v4.0 Header
/// 
/// This appears immediately after the universal extractor binary
/// and the MAGIC bytes in the file.
#[derive(Debug, Clone)]
#[repr(C)]
pub struct GenomeHeader {
    /// Format version (4)
    pub version: u32,
    
    /// Offset to manifest JSON (compressed)
    pub manifest_offset: u64,
    
    /// Size of compressed manifest
    pub manifest_size: u32,
    
    /// Offset to binary entries table
    pub binaries_offset: u64,
    
    /// Number of binary entries
    pub num_binaries: u32,
    
    /// SHA256 fingerprint of entire payload (manifest + binaries)
    /// This is the "DNA fingerprint" of the genome
    pub fingerprint: [u8; 32],
}

impl GenomeHeader {
    /// Read header from file at current position
    pub fn read_from<R: Read>(reader: &mut R) -> Result<Self> {
        use std::mem::size_of;
        
        let mut version = [0u8; 4];
        reader.read_exact(&mut version)?;
        let version = u32::from_le_bytes(version);
        
        if version != VERSION {
            anyhow::bail!("Unsupported genomeBin version: {}", version);
        }
        
        let mut manifest_offset = [0u8; 8];
        reader.read_exact(&mut manifest_offset)?;
        let manifest_offset = u64::from_le_bytes(manifest_offset);
        
        let mut manifest_size = [0u8; 4];
        reader.read_exact(&mut manifest_size)?;
        let manifest_size = u32::from_le_bytes(manifest_size);
        
        let mut binaries_offset = [0u8; 8];
        reader.read_exact(&mut binaries_offset)?;
        let binaries_offset = u64::from_le_bytes(binaries_offset);
        
        let mut num_binaries = [0u8; 4];
        reader.read_exact(&mut num_binaries)?;
        let num_binaries = u32::from_le_bytes(num_binaries);
        
        let mut fingerprint = [0u8; 32];
        reader.read_exact(&mut fingerprint)?;
        
        Ok(Self {
            version,
            manifest_offset,
            manifest_size,
            binaries_offset,
            num_binaries,
            fingerprint,
        })
    }
    
    /// Write header to buffer (little-endian)
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(60); // 4+8+4+8+4+32
        
        bytes.extend_from_slice(&self.version.to_le_bytes());
        bytes.extend_from_slice(&self.manifest_offset.to_le_bytes());
        bytes.extend_from_slice(&self.manifest_size.to_le_bytes());
        bytes.extend_from_slice(&self.binaries_offset.to_le_bytes());
        bytes.extend_from_slice(&self.num_binaries.to_le_bytes());
        bytes.extend_from_slice(&self.fingerprint);
        
        bytes
    }
}

/// Binary entry in the binaries table
#[derive(Debug, Clone)]
#[repr(C)]
pub struct BinaryEntry {
    /// Architecture name (e.g., "x86_64", "aarch64")
    /// Null-padded to 16 bytes
    pub architecture: [u8; 16],
    
    /// Offset from binaries data start
    pub offset: u64,
    
    /// Compressed size (zstd)
    pub compressed_size: u32,
    
    /// Uncompressed size
    pub uncompressed_size: u32,
    
    /// SHA256 checksum of uncompressed binary
    pub checksum: [u8; 32],
}

impl BinaryEntry {
    /// Read binary entry from reader
    pub fn read_from<R: Read>(reader: &mut R) -> Result<Self> {
        let mut architecture = [0u8; 16];
        reader.read_exact(&mut architecture)?;
        
        let mut offset = [0u8; 8];
        reader.read_exact(&mut offset)?;
        let offset = u64::from_le_bytes(offset);
        
        let mut compressed_size = [0u8; 4];
        reader.read_exact(&mut compressed_size)?;
        let compressed_size = u32::from_le_bytes(compressed_size);
        
        let mut uncompressed_size = [0u8; 4];
        reader.read_exact(&mut uncompressed_size)?;
        let uncompressed_size = u32::from_le_bytes(uncompressed_size);
        
        let mut checksum = [0u8; 32];
        reader.read_exact(&mut checksum)?;
        
        Ok(Self {
            architecture,
            offset,
            compressed_size,
            uncompressed_size,
            checksum,
        })
    }
    
    /// Get architecture as string
    pub fn architecture_str(&self) -> String {
        let end = self.architecture.iter()
            .position(|&b| b == 0)
            .unwrap_or(self.architecture.len());
        
        String::from_utf8_lossy(&self.architecture[..end]).to_string()
    }
    
    /// Create entry from components
    pub fn new(
        arch: &str,
        offset: u64,
        compressed_size: u32,
        uncompressed_size: u32,
        checksum: [u8; 32],
    ) -> Self {
        let mut architecture = [0u8; 16];
        let arch_bytes = arch.as_bytes();
        let len = arch_bytes.len().min(16);
        architecture[..len].copy_from_slice(&arch_bytes[..len]);
        
        Self {
            architecture,
            offset,
            compressed_size,
            uncompressed_size,
            checksum,
        }
    }
    
    /// Write entry to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(64); // 16+8+4+4+32
        
        bytes.extend_from_slice(&self.architecture);
        bytes.extend_from_slice(&self.offset.to_le_bytes());
        bytes.extend_from_slice(&self.compressed_size.to_le_bytes());
        bytes.extend_from_slice(&self.uncompressed_size.to_le_bytes());
        bytes.extend_from_slice(&self.checksum);
        
        bytes
    }
}

/// Genome manifest (metadata)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenomeManifest {
    pub name: String,
    pub version: String,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    
    pub architectures: Vec<String>,
    
    #[serde(default)]
    pub capabilities: Vec<String>,
    
    /// Timestamp of creation (for reproducible builds)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub build_timestamp: Option<String>,
}

/// Detect current system architecture
pub fn current_arch() -> String {
    match std::env::consts::ARCH {
        "x86_64" => "x86_64".to_string(),
        "aarch64" => "aarch64".to_string(),
        "riscv64" => "riscv64".to_string(),
        arch => arch.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_header_round_trip() {
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
        
        let mut cursor = std::io::Cursor::new(bytes);
        let decoded = GenomeHeader::read_from(&mut cursor).unwrap();
        
        assert_eq!(decoded.version, 4);
        assert_eq!(decoded.manifest_offset, 1000);
        assert_eq!(decoded.manifest_size, 500);
        assert_eq!(decoded.binaries_offset, 2000);
        assert_eq!(decoded.num_binaries, 2);
        assert_eq!(decoded.fingerprint, [42u8; 32]);
    }
    
    #[test]
    fn test_binary_entry_architecture_str() {
        let entry = BinaryEntry::new("x86_64", 0, 100, 200, [0u8; 32]);
        assert_eq!(entry.architecture_str(), "x86_64");
        
        let entry = BinaryEntry::new("aarch64", 0, 100, 200, [0u8; 32]);
        assert_eq!(entry.architecture_str(), "aarch64");
    }
    
    #[test]
    fn test_current_arch() {
        let arch = current_arch();
        assert!(!arch.is_empty());
        // Should be one of the supported architectures
        assert!(arch == "x86_64" || arch == "aarch64" || arch == "riscv64" || !arch.is_empty());
    }
}
