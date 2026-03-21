// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

// biomeos-genomebin-v3/src/verify.rs
// Verification and integrity checking
//
// Deep Debt Principles:
// - Comprehensive validation
// - Clear error messages
// - Pure Rust (sha2 for checksums)

use crate::GenomeBin;
use anyhow::{Context, Result};
use std::collections::HashMap;

/// Verification result for a single binary
#[derive(Debug, Clone)]
pub struct VerifyResult {
    pub expected: String,
    pub actual: String,
    pub valid: bool,
}

impl GenomeBin {
    /// Verify all checksums
    pub fn verify_all(&self) -> Result<HashMap<String, VerifyResult>> {
        tracing::info!("Verifying genomeBin: {}", self.manifest.name);

        let mut results = HashMap::new();

        // Verify each binary
        for (arch, compressed) in &self.binaries {
            let arch_name = format!("{arch:?}");

            tracing::debug!("Verifying {} binary...", arch_name);

            // Decompress and verify
            match compressed.decompress() {
                Ok(_) => {
                    // decompress() already verifies the checksum internally
                    results.insert(
                        arch_name.clone(),
                        VerifyResult {
                            expected: hex::encode(compressed.checksum),
                            actual: hex::encode(compressed.checksum),
                            valid: true,
                        },
                    );
                    tracing::debug!("✅ {} binary verified", arch_name);
                }
                Err(e) => {
                    tracing::error!("❌ {} binary verification failed: {}", arch_name, e);
                    results.insert(
                        arch_name.clone(),
                        VerifyResult {
                            expected: hex::encode(compressed.checksum),
                            actual: "FAILED".to_string(),
                            valid: false,
                        },
                    );
                }
            }
        }

        // Verify embedded genomes recursively
        for (i, embedded) in self.embedded_genomes.iter().enumerate() {
            tracing::debug!("Verifying embedded genome: {}", embedded.manifest.name);
            let embedded_results = embedded.verify_all().with_context(|| {
                format!(
                    "Failed to verify embedded genome: {}",
                    embedded.manifest.name
                )
            })?;

            for (key, result) in embedded_results {
                results.insert(format!("embedded[{i}]/{key}"), result);
            }
        }

        let valid_count = results.values().filter(|r| r.valid).count();
        let total_count = results.len();

        if valid_count == total_count {
            tracing::info!("✅ All checksums valid ({}/{})", valid_count, total_count);
        } else {
            tracing::error!(
                "❌ Verification failed: {}/{} valid",
                valid_count,
                total_count
            );
        }

        Ok(results)
    }

    /// Quick check if all checksums are valid
    pub fn is_valid(&self) -> Result<bool> {
        let results = self.verify_all()?;
        Ok(results.values().all(|r| r.valid))
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;
    use crate::{Arch, CompressedBinary};
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_verify_valid_binary() {
        // Create test binary
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"test binary content").unwrap();
        temp_file.flush().unwrap();

        // Create compressed binary
        let compressed = CompressedBinary::from_file(Arch::X86_64, temp_file.path()).unwrap();

        // Create genomeBin
        let mut genome = GenomeBin::new("test");
        genome.binaries.insert(Arch::X86_64, compressed);

        // Verify
        let results = genome.verify_all().unwrap();
        assert_eq!(results.len(), 1);
        assert!(results.values().all(|r| r.valid));
        assert!(genome.is_valid().unwrap());
    }

    #[test]
    fn test_verify_invalid_checksum_returns_false_result() {
        let mut genome = GenomeBin::new("invalid-test");
        let mut compressed = CompressedBinary::compress(Arch::X86_64, b"data");
        compressed.checksum[0] ^= 0xff;
        genome.binaries.insert(Arch::X86_64, compressed);

        let results = genome.verify_all().unwrap();
        assert_eq!(results.len(), 1);
        let result = results.values().next().unwrap();
        assert!(!result.valid);
        assert_eq!(result.actual, "FAILED");
        assert!(!genome.is_valid().unwrap());
    }

    #[test]
    fn test_verify_embedded_genome() {
        let mut child = GenomeBin::new("child");
        child.add_binary_bytes(Arch::Aarch64, b"child binary");

        let mut parent = GenomeBin::new("parent");
        parent.add_binary_bytes(Arch::X86_64, b"parent binary");
        parent.embed(child).unwrap();

        let results = parent.verify_all().unwrap();
        assert!(results.len() >= 2);
        assert!(results.values().all(|r| r.valid));
        assert!(parent.is_valid().unwrap());
    }

    #[test]
    fn test_verify_embedded_genome_invalid_propagates() {
        let mut child = GenomeBin::new("child");
        let mut bad_compressed = CompressedBinary::compress(Arch::Aarch64, b"child");
        bad_compressed.checksum[0] ^= 0xff;
        child.binaries.insert(Arch::Aarch64, bad_compressed);

        let mut parent = GenomeBin::new("parent");
        parent.add_binary_bytes(Arch::X86_64, b"parent");
        parent.embed(child).unwrap();

        let results = parent.verify_all().unwrap();
        assert!(!results.values().all(|r| r.valid));
        assert!(!parent.is_valid().unwrap());
    }

    #[test]
    fn test_verify_result_fields() {
        let mut genome = GenomeBin::new("fields-test");
        genome.add_binary_bytes(Arch::X86_64, b"x");

        let results = genome.verify_all().unwrap();
        let result = results.values().next().unwrap();
        assert!(result.valid);
        assert_eq!(result.expected, result.actual);
    }
}
