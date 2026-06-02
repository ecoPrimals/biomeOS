// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

// crates/biomeos-spore/src/verification.rs
//! Spore verification and validation

use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use tracing::{info, warn};

use crate::manifest::{BinaryManifest, SporeManifest};

/// Verification status for a binary or spore
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerificationStatus {
    /// Binary matches plasmidBin exactly
    Fresh,
    /// Binary is older than plasmidBin
    Stale,
    /// Binary has different hash (manual edit or corruption?)
    Modified,
    /// Binary file not found
    Missing,
    /// Binary is newer than plasmidBin (unusual)
    Newer,
}

impl std::fmt::Display for VerificationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Fresh => write!(f, "Fresh"),
            Self::Stale => write!(f, "Stale"),
            Self::Modified => write!(f, "Modified"),
            Self::Missing => write!(f, "Missing"),
            Self::Newer => write!(f, "Newer"),
        }
    }
}

/// Verification report for a single binary
#[derive(Debug)]
pub struct BinaryVerification {
    /// Binary name
    pub name: String,
    /// Verification result
    pub status: VerificationStatus,
    /// Expected version from the manifest
    pub expected_version: String,
    /// Actual version on disk (if determinable)
    pub actual_version: Option<String>,
    /// Expected SHA-256 digest from the manifest
    pub expected_sha256: String,
    /// Actual SHA-256 digest on disk
    pub actual_sha256: Option<String>,
}

/// Complete verification report for a spore
#[derive(Debug)]
pub struct VerificationReport {
    /// Path to the spore directory
    pub spore_path: PathBuf,
    /// Node identifier
    pub node_id: String,
    /// Aggregate status across all binaries
    pub overall_status: VerificationStatus,
    /// Per-binary verification results
    pub binaries: Vec<BinaryVerification>,
    /// Human-readable recommendations
    pub recommendations: Vec<String>,
}

/// Spore verification engine
pub struct SporeVerifier {
    nucleus_manifest: BinaryManifest,
}

impl SporeVerifier {
    /// Create a new verifier from plasmidBin
    pub fn from_nucleus(nucleus_path: impl AsRef<Path>) -> Result<Self> {
        let nucleus_path = nucleus_path.as_ref();

        // Try to load existing manifest, generate if not found
        let nucleus_manifest = if let Ok(manifest) = BinaryManifest::load(nucleus_path) {
            info!(
                "Loaded existing nucleus manifest with {} binaries",
                manifest.binaries.len()
            );
            manifest
        } else {
            info!("Generating nucleus manifest from binaries");
            let manifest = BinaryManifest::from_nucleus(nucleus_path)?;
            // Save for next time
            if let Err(e) = manifest.save(nucleus_path.join("MANIFEST.toml")) {
                warn!("Failed to save generated manifest: {}", e);
            }
            manifest
        };

        Ok(Self { nucleus_manifest })
    }

    /// Verify a single spore against plasmidBin
    pub fn verify_spore(&self, spore_path: impl AsRef<Path>) -> Result<VerificationReport> {
        let spore_path = spore_path.as_ref();

        info!("Verifying spore at: {}", spore_path.display());

        // Load spore manifest (if exists)
        let spore_manifest = SporeManifest::load(spore_path).ok();

        // Get node_id from manifest or tower.toml
        let node_id = if let Some(ref manifest) = spore_manifest {
            manifest.spore.node_id.clone()
        } else {
            Self::extract_node_id_from_tower_toml(spore_path)?
        };

        let mut binary_verifications = Vec::new();
        let mut overall_fresh = true;

        // Verify each expected binary
        for (key, expected_binary) in &self.nucleus_manifest.binaries {
            let binary_path = if key == "tower" {
                spore_path.join("bin").join(&expected_binary.name)
            } else {
                spore_path.join("primals").join(&expected_binary.name)
            };

            let verification = if binary_path.exists() {
                let actual_sha256 = Self::compute_sha256(&binary_path)?;

                let status = if actual_sha256 == expected_binary.sha256 {
                    VerificationStatus::Fresh
                } else {
                    overall_fresh = false;
                    // Check if spore manifest has version info
                    if let Some(ref manifest) = spore_manifest {
                        if let Some(spore_binary) = manifest.binaries.get(key) {
                            match spore_binary.version.cmp(&expected_binary.version) {
                                std::cmp::Ordering::Less => VerificationStatus::Stale,
                                std::cmp::Ordering::Greater => VerificationStatus::Newer,
                                std::cmp::Ordering::Equal => VerificationStatus::Modified,
                            }
                        } else {
                            VerificationStatus::Stale
                        }
                    } else {
                        VerificationStatus::Stale // No manifest, assume stale
                    }
                };

                let actual_version = spore_manifest
                    .as_ref()
                    .and_then(|m| m.binaries.get(key))
                    .map(|b| b.version.clone());

                BinaryVerification {
                    name: key.clone(),
                    status,
                    expected_version: expected_binary.version.clone(),
                    actual_version,
                    expected_sha256: expected_binary.sha256.clone(),
                    actual_sha256: Some(actual_sha256),
                }
            } else {
                overall_fresh = false;
                BinaryVerification {
                    name: key.clone(),
                    status: VerificationStatus::Missing,
                    expected_version: expected_binary.version.clone(),
                    actual_version: None,
                    expected_sha256: expected_binary.sha256.clone(),
                    actual_sha256: None,
                }
            };

            binary_verifications.push(verification);
        }

        // Generate recommendations
        let mut recommendations = Vec::new();
        if !overall_fresh {
            recommendations.push(format!(
                "Run: biomeos spore refresh {} to update binaries",
                spore_path.display()
            ));

            let stale_count = binary_verifications
                .iter()
                .filter(|b| matches!(b.status, VerificationStatus::Stale))
                .count();

            if stale_count > 0 {
                recommendations.push(format!("{stale_count} stale binaries need update"));
            }
        }

        Ok(VerificationReport {
            spore_path: spore_path.to_path_buf(),
            node_id,
            overall_status: if overall_fresh {
                VerificationStatus::Fresh
            } else {
                VerificationStatus::Stale
            },
            binaries: binary_verifications,
            recommendations,
        })
    }

    /// Verify all mounted spores
    pub fn verify_all_spores(&self) -> Result<Vec<VerificationReport>> {
        // Auto-detect mounted USB spores
        let mount_points = Self::discover_spores()?;

        info!("Found {} mounted spores", mount_points.len());

        let mut reports = Vec::new();
        for mount_point in mount_points {
            match self.verify_spore(&mount_point) {
                Ok(report) => {
                    info!(
                        "✅ Verified: {} ({})",
                        report.node_id, report.overall_status
                    );
                    reports.push(report);
                }
                Err(e) => {
                    warn!("Failed to verify spore at {}: {}", mount_point.display(), e);
                }
            }
        }

        Ok(reports)
    }

    /// Compute SHA256 hash of a file
    fn compute_sha256(path: &Path) -> Result<String> {
        let bytes = std::fs::read(path)
            .with_context(|| format!("Failed to read file: {}", path.display()))?;

        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        let result = hasher.finalize();

        Ok(format!("{result:x}"))
    }

    /// Discover mounted spores by scanning /media/*
    fn discover_spores() -> Result<Vec<PathBuf>> {
        let mut spores = Vec::new();

        // Check common mount points
        let media_dir = PathBuf::from("/media").join(std::env::var(biomeos_types::env_config::vars::SYS_USER)?);

        if media_dir.exists() {
            for entry in std::fs::read_dir(&media_dir)? {
                let entry = entry?;
                let spore_candidate = entry.path().join("biomeOS");

                // Check if this looks like a biomeOS spore
                if spore_candidate.exists() && Self::is_valid_spore(&spore_candidate) {
                    spores.push(spore_candidate);
                }
            }
        }

        Ok(spores)
    }

    /// Check if a directory is a valid biomeOS spore
    fn is_valid_spore(path: &Path) -> bool {
        // Check for key indicators: tower.toml, .family.seed, bin/, primals/
        path.join("tower.toml").exists()
            && path.join(".family.seed").exists()
            && path.join("bin").exists()
            && path.join("primals").exists()
    }

    /// Extract `node_id` from tower.toml
    fn extract_node_id_from_tower_toml(spore_path: &Path) -> Result<String> {
        let tower_toml_path = spore_path.join("tower.toml");
        let tower_toml_str = std::fs::read_to_string(&tower_toml_path)
            .with_context(|| format!("Failed to read tower.toml from {}", spore_path.display()))?;

        // Parse as TOML
        let tower_toml: toml::Value = toml::from_str(&tower_toml_str)?;

        // Try multiple locations for node_id (evolved config format)
        let node_id = tower_toml
            .get("primals")
            .and_then(|p| p.as_array())
            .and_then(|arr| arr.first())
            .and_then(|primal| primal.get("env"))
            .and_then(|env| env.get("NODE_ID"))
            .and_then(|v| v.as_str())
            .or_else(|| {
                tower_toml
                    .get("tower")
                    .and_then(|t| t.get("NODE_ID"))
                    .or_else(|| tower_toml.get("node_id"))
                    .and_then(|v| v.as_str())
            })
            .ok_or_else(|| anyhow::anyhow!("node_id not found in tower.toml (tried primals.env.NODE_ID, tower.NODE_ID, node_id)"))?
            .to_string();

        Ok(node_id)
    }
}

#[cfg(test)]
#[path = "verification_tests.rs"]
mod tests;
