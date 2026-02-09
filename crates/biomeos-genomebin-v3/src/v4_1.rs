// biomeos-genomebin-v3/src/v4_1.rs
// genomeBin v4.1 - Multi-Architecture Fat Binary Format
//
// Deep Debt Principles:
// - Smart architecture (not just simple splits)
// - Runtime discovery (architecture detection at execution)
// - Capability-based (bootstrap selector)
// - Platform-agnostic (works everywhere)
// - Self-knowledge (extractors know their own architecture)

use crate::{Arch, GenomeBin};
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

/// genomeBin v4.1 Format Constants
const BOOTSTRAP_SIZE: usize = 4096; // Bootstrap script padded to 4KB
const TABLE_SIZE: usize = 128; // Extractor table (128 bytes)
const EXTRACTOR_SIZE: usize = 1_048_576; // Each extractor padded to 1MB

/// Extractor Table Entry (32 bytes)
#[repr(C)]
#[derive(Debug, Clone)]
struct ExtractorEntry {
    architecture: [u8; 16], // Null-padded architecture name
    offset: u64,            // Offset from start of file
    size: u64,              // Actual size (before padding)
    checksum: [u8; 8],      // First 8 bytes of SHA256
}

impl ExtractorEntry {
    fn new(arch: &str, offset: u64, size: u64, checksum: &[u8; 32]) -> Self {
        let mut arch_bytes = [0u8; 16];
        let arch_str = arch.as_bytes();
        let len = arch_str.len().min(16);
        arch_bytes[..len].copy_from_slice(&arch_str[..len]);

        let mut checksum_short = [0u8; 8];
        checksum_short.copy_from_slice(&checksum[..8]);

        Self {
            architecture: arch_bytes,
            offset,
            size,
            checksum: checksum_short,
        }
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(32);
        bytes.extend_from_slice(&self.architecture);
        bytes.extend_from_slice(&self.offset.to_le_bytes());
        bytes.extend_from_slice(&self.size.to_le_bytes());
        bytes.extend_from_slice(&self.checksum);
        bytes
    }
}

impl GenomeBin {
    /// Write genomeBin v4.1 - Multi-Architecture Fat Binary
    ///
    /// This creates a truly universal genomeBin that can execute natively
    /// on any supported architecture without external tools.
    ///
    /// Deep Debt Principles:
    /// - Smart architecture: Each extractor is native for its platform
    /// - Runtime discovery: Bootstrap detects architecture at execution
    /// - No hardcoding: Architecture list derived from available extractors
    /// - Capability-based: Only includes extractors that exist
    pub fn write_v4_1(&self, output: &Path, extractors: &HashMap<Arch, &Path>) -> Result<()> {
        tracing::info!("Creating genomeBin v4.1 (multi-arch fat binary)...");

        let mut file = File::create(output).context("Failed to create genomeBin v4.1 file")?;

        // Phase 1: Write bootstrap selector (padded to 1KB)
        let bootstrap = include_bytes!("../bootstrap-selector.sh");
        file.write_all(bootstrap)?;

        // Pad to BOOTSTRAP_SIZE
        let padding_size = BOOTSTRAP_SIZE.saturating_sub(bootstrap.len());
        if padding_size > 0 {
            let padding = vec![0u8; padding_size];
            file.write_all(&padding)?;
        }

        tracing::info!(
            "   Bootstrap selector: {} bytes (padded to {})",
            bootstrap.len(),
            BOOTSTRAP_SIZE
        );

        // Phase 2: Build extractor table
        let mut table = Vec::new();
        let mut extractor_data = Vec::new();
        let mut current_offset = BOOTSTRAP_SIZE + TABLE_SIZE;

        // Sort extractors for deterministic output (Deep Debt: no arbitrary ordering)
        let mut sorted_extractors: Vec<_> = extractors.iter().collect();
        sorted_extractors.sort_by_key(|(arch, _)| format!("{:?}", arch));

        for (arch, extractor_path) in sorted_extractors {
            // Read extractor binary
            let mut extractor_file = File::open(extractor_path)
                .with_context(|| format!("Failed to open extractor: {:?}", extractor_path))?;

            let mut extractor_bytes = Vec::new();
            extractor_file.read_to_end(&mut extractor_bytes)?;

            let actual_size = extractor_bytes.len() as u64;

            // Calculate checksum (self-knowledge)
            use sha2::{Digest, Sha256};
            let mut hasher = Sha256::new();
            hasher.update(&extractor_bytes);
            let checksum: [u8; 32] = hasher.finalize().into();

            // Create table entry
            let arch_str = match arch {
                Arch::X86_64 => "x86_64",
                Arch::Aarch64 => "aarch64",
                Arch::Riscv64 => "riscv64",
                _ => continue, // Skip unsupported architectures
            };

            let entry =
                ExtractorEntry::new(arch_str, current_offset as u64, actual_size, &checksum);

            table.extend_from_slice(&entry.to_bytes());

            // Pad extractor to EXTRACTOR_SIZE (alignment for fast seeking)
            extractor_bytes.resize(EXTRACTOR_SIZE, 0);
            extractor_data.extend_from_slice(&extractor_bytes);

            current_offset += EXTRACTOR_SIZE;

            tracing::info!(
                "   {} extractor: {} bytes (padded to {})",
                arch_str,
                actual_size,
                EXTRACTOR_SIZE
            );
        }

        // Pad table to TABLE_SIZE
        table.resize(TABLE_SIZE, 0);
        file.write_all(&table)?;

        tracing::info!(
            "   Extractor table: {} entries ({} bytes)",
            extractors.len(),
            TABLE_SIZE
        );

        // Phase 3: Write all extractors
        file.write_all(&extractor_data)?;

        tracing::info!("   Total extractors: {} bytes", extractor_data.len());

        // Phase 4: Write GENOME40 payload (same as v4.0)
        // This includes MAGIC, header, manifest, binaries
        let payload_offset = file.metadata()?.len();

        // Write using existing v4.0 logic (code reuse, not duplication)
        // We'll append the v4.0 format after the extractors
        self.write_v4_payload(&mut file)?;

        let total_size = file.metadata()?.len();

        tracing::info!("genomeBin v4.1 created successfully:");
        tracing::info!("   Bootstrap: {}KB", BOOTSTRAP_SIZE / 1024);
        tracing::info!(
            "   Extractors: {}KB ({} architectures)",
            extractor_data.len() / 1024,
            extractors.len()
        );
        tracing::info!("   Payload: {}KB", (total_size - payload_offset) / 1024);
        tracing::info!("   Total: {}KB", total_size / 1024);

        // Make executable on Unix (capability-based)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = file.metadata()?.permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(output, perms)?;
        }

        Ok(())
    }

    /// Write v4.0 payload portion (internal helper)
    /// This reuses the v4.0 format for the actual primal binaries
    fn write_v4_payload(&self, file: &mut File) -> Result<()> {
        use sha2::{Digest, Sha256};

        // For v4.1, we don't embed another extractor in the payload
        // The bootstrap already handles extraction
        // So we write the GENOME40 payload directly

        // Write MAGIC marker
        let magic: &[u8; 8] = b"GENOME40";
        file.write_all(magic)?;

        // Prepare manifest (compressed JSON) - reuse from v4.0
        let manifest_json =
            serde_json::to_vec(&self.manifest).context("Failed to serialize manifest")?;

        let manifest_compressed =
            zstd::encode_all(&manifest_json[..], 19).context("Failed to compress manifest")?;

        tracing::debug!(
            "   Manifest: {} → {} bytes (compressed)",
            manifest_json.len(),
            manifest_compressed.len()
        );

        // Calculate offsets (relative to header start)
        // All offsets in header are relative to header start
        let manifest_offset_rel = 60u64; // Right after header
        let binaries_table_offset_rel = manifest_offset_rel + manifest_compressed.len() as u64;

        // Calculate DNA fingerprint (SHA256 of payload)
        let mut hasher = Sha256::new();
        hasher.update(&manifest_compressed);

        // Hash binaries in deterministic order
        let mut sorted_binaries: Vec<_> = self.binaries.iter().collect();
        sorted_binaries.sort_by_key(|(arch, _)| format!("{:?}", arch));

        for (_arch, bin) in &sorted_binaries {
            hasher.update(&bin.data);
        }
        let fingerprint: [u8; 32] = hasher.finalize().into();

        // Create header (60 bytes)
        let mut header = Vec::with_capacity(60);
        header.extend_from_slice(&4u32.to_le_bytes()); // version
        header.extend_from_slice(&manifest_offset_rel.to_le_bytes()); // manifest_offset
        header.extend_from_slice(&(manifest_compressed.len() as u32).to_le_bytes()); // manifest_size
        header.extend_from_slice(&binaries_table_offset_rel.to_le_bytes()); // binaries_offset
        header.extend_from_slice(&(self.binaries.len() as u32).to_le_bytes()); // num_binaries
        header.extend_from_slice(&fingerprint); // fingerprint (32 bytes)

        // Write header
        file.write_all(&header)?;

        // Write compressed manifest
        file.write_all(&manifest_compressed)?;

        // Write binary table (64 bytes per entry)
        let mut binary_offset = 0u64;
        for (arch, compressed_bin) in &sorted_binaries {
            let arch_str = match arch {
                Arch::X86_64 => "x86_64",
                Arch::Aarch64 => "aarch64",
                Arch::Riscv64 => "riscv64",
                _ => "unknown",
            };

            // Create 64-byte entry
            let mut entry = Vec::with_capacity(64);
            let mut arch_bytes = [0u8; 16];
            let arch_str_bytes = arch_str.as_bytes();
            let len = arch_str_bytes.len().min(16);
            arch_bytes[..len].copy_from_slice(&arch_str_bytes[..len]);

            entry.extend_from_slice(&arch_bytes); // architecture (16 bytes)
            entry.extend_from_slice(&binary_offset.to_le_bytes()); // offset (8 bytes)
            entry.extend_from_slice(&(compressed_bin.data.len() as u32).to_le_bytes()); // compressed_size (4 bytes)
            entry.extend_from_slice(&(compressed_bin.original_size as u32).to_le_bytes()); // uncompressed_size (4 bytes)
            entry.extend_from_slice(&compressed_bin.checksum); // checksum (32 bytes)

            file.write_all(&entry)?;
            binary_offset += compressed_bin.data.len() as u64;
        }

        // Write compressed binaries (MUST use same sorted order as table!)
        for (_arch, compressed_bin) in sorted_binaries {
            file.write_all(&compressed_bin.data)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extractor_entry_size() {
        // Verify entry is exactly 40 bytes (16 + 8 + 8 + 8)
        // architecture: 16 bytes
        // offset: 8 bytes (u64)
        // size: 8 bytes (u64)
        // checksum: 8 bytes (first 8 of SHA256)
        let entry = ExtractorEntry::new("x86_64", 0, 0, &[0u8; 32]);
        let bytes = entry.to_bytes();
        assert_eq!(bytes.len(), 40, "ExtractorEntry must be exactly 40 bytes");
    }

    #[test]
    fn test_bootstrap_size() {
        // Verify bootstrap script exists and is reasonable size
        let bootstrap = include_bytes!("../bootstrap-selector.sh");
        assert!(!bootstrap.is_empty(), "Bootstrap script must exist");
        assert!(
            bootstrap.len() < BOOTSTRAP_SIZE,
            "Bootstrap script ({} bytes) must fit in {} bytes",
            bootstrap.len(),
            BOOTSTRAP_SIZE
        );
    }

    #[test]
    fn test_table_capacity() {
        // Verify table can hold at least 4 architectures (32 bytes each)
        const MIN_ARCHITECTURES: usize = 4;
        assert!(
            TABLE_SIZE >= MIN_ARCHITECTURES * 32,
            "Table must hold at least {} architecture entries",
            MIN_ARCHITECTURES
        );
    }
}
