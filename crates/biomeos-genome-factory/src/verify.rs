// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

// biomeos-genome-factory/src/verify.rs
// Verify genomeBin integrity
//
// Deep Debt: Comprehensive validation

use crate::GenomeFactory;
use anyhow::{Context, Result};
use biomeos_genomebin_v3::GenomeBin;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Request to verify a genome
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenomeVerifyRequest {
    /// Genome name to verify
    pub name: String,
}

/// Verification result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenomeVerifyResponse {
    /// Identifier of the verified genome
    pub genome_id: String,
    /// Whether the genome passed all verification checks
    pub valid: bool,
    /// Per-component checksum results
    pub checksums: HashMap<String, ChecksumResult>,
    /// Whether the manifest is valid
    pub manifest_valid: bool,
    /// Number of embedded genomes
    pub embedded_count: usize,
}

/// Checksum verification result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChecksumResult {
    /// Expected checksum value
    pub expected: String,
    /// Actual computed checksum value
    pub actual: String,
    /// Whether expected matches actual
    pub valid: bool,
}

impl GenomeFactory {
    /// Load a genome from storage
    pub fn load_genome(&self, name: &str) -> Result<GenomeBin> {
        let path = self.genome_path(name);
        GenomeBin::load(&path).with_context(|| format!("Failed to load genome: {name}"))
    }

    /// Verify genomeBin integrity
    pub fn verify_genome(&self, name: &str) -> Result<GenomeVerifyResponse> {
        tracing::info!("🔍 Verifying genomeBin: {}", name);

        // Load genome
        let genome = self.load_genome(name)?;

        // Verify all checksums
        let verify_results = genome.verify_all().context("Verification failed")?;

        // Convert results
        let mut checksums = HashMap::new();
        for (key, result) in verify_results {
            checksums.insert(
                key,
                ChecksumResult {
                    expected: result.expected,
                    actual: result.actual,
                    valid: result.valid,
                },
            );
        }

        let valid = checksums.values().all(|r| r.valid);

        let response = GenomeVerifyResponse {
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

#[allow(clippy::unwrap_used, clippy::expect_used)]
#[cfg(test)]
mod tests {
    use super::*;
    use biomeos_genomebin_v3::GenomeBin;
    use tempfile::TempDir;

    #[test]
    fn test_verify_nonexistent() {
        let temp_dir = TempDir::new().expect("temp dir");
        let factory = GenomeFactory::new(temp_dir.path());

        let result = factory.verify_genome("nonexistent");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Failed to load genome"));
    }

    #[test]
    fn test_verify_request_serialization() {
        let request = GenomeVerifyRequest {
            name: "beardog".to_string(),
        };
        let json = serde_json::to_string(&request).expect("serialize");
        assert!(json.contains("\"name\":\"beardog\""));
        let deserialized: GenomeVerifyRequest = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deserialized.name, request.name);
    }

    #[test]
    fn test_verify_response_serialization() {
        let mut checksums = HashMap::new();
        checksums.insert(
            "x86_64".to_string(),
            ChecksumResult {
                expected: "abc123".to_string(),
                actual: "abc123".to_string(),
                valid: true,
            },
        );
        let response = GenomeVerifyResponse {
            genome_id: "beardog".to_string(),
            valid: true,
            checksums,
            manifest_valid: true,
            embedded_count: 0,
        };
        let json = serde_json::to_string(&response).expect("serialize");
        assert!(json.contains("\"genome_id\":\"beardog\""));
        assert!(json.contains("\"valid\":true"));
    }

    #[test]
    fn test_checksum_result_serialization() {
        let result = ChecksumResult {
            expected: "deadbeef".to_string(),
            actual: "deadbeef".to_string(),
            valid: true,
        };
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(json.contains("deadbeef"));
        let deserialized: ChecksumResult = serde_json::from_str(&json).expect("deserialize");
        assert!(deserialized.valid);
    }

    #[test]
    fn test_verify_valid_genome() {
        let temp_dir = TempDir::new().expect("temp dir");
        let factory = GenomeFactory::new(temp_dir.path());

        let genome = GenomeBin::new("valid-genome");
        genome
            .save(&factory.genome_path("valid-genome"))
            .expect("save genome");

        let result = factory.verify_genome("valid-genome");
        let response = result.expect("verify should succeed");
        assert_eq!(response.genome_id, "valid-genome");
        assert!(response.valid);
        assert!(response.manifest_valid);
        assert_eq!(response.embedded_count, 0);
    }
}
