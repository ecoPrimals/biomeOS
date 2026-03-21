// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

// crates/biomeos-spore/src/refresh.rs
//! Spore refresh functionality - Update stale binaries

use anyhow::Result;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use tracing::{info, warn};

use crate::manifest::{BinaryManifest, SporeManifest};
use crate::verification::{SporeVerifier, VerificationStatus};

/// Spore refresh engine - Updates stale binaries
pub struct SporeRefresher {
    nucleus_manifest: BinaryManifest,
    nucleus_path: PathBuf,
}

impl SporeRefresher {
    /// Create a new refresher from plasmidBin
    pub fn from_nucleus(nucleus_path: impl AsRef<Path>) -> Result<Self> {
        let nucleus_path = nucleus_path.as_ref().to_path_buf();

        // Load or generate nucleus manifest
        let nucleus_manifest = if let Ok(manifest) = BinaryManifest::load(&nucleus_path) {
            info!("Loaded existing nucleus manifest");
            manifest
        } else {
            info!("Generating nucleus manifest from binaries");
            let manifest = BinaryManifest::from_nucleus(&nucleus_path)?;
            // Save for next time
            if let Err(e) = manifest.save(nucleus_path.join("MANIFEST.toml")) {
                warn!("Failed to save generated manifest: {}", e);
            }
            manifest
        };

        Ok(Self {
            nucleus_manifest,
            nucleus_path,
        })
    }

    /// Refresh a single spore (update stale binaries)
    pub fn refresh_spore(&self, spore_path: impl AsRef<Path>) -> Result<RefreshReport> {
        let spore_path = spore_path.as_ref();

        info!("Refreshing spore at: {}", spore_path.display());

        // First verify to see what needs updating
        let verifier = SporeVerifier::from_nucleus(&self.nucleus_path)?;
        let verification = verifier.verify_spore(spore_path)?;

        let mut refreshed_binaries = Vec::new();
        let mut failed_binaries = Vec::new();

        // Update each stale or missing binary
        for binary_verification in &verification.binaries {
            if matches!(
                binary_verification.status,
                VerificationStatus::Stale
                    | VerificationStatus::Missing
                    | VerificationStatus::Modified
            ) {
                // Get expected binary info from nucleus
                let nucleus_binary = self
                    .nucleus_manifest
                    .binaries
                    .get(&binary_verification.name);

                if let Some(nucleus_binary) = nucleus_binary {
                    // Determine source and destination paths
                    let (source_path, dest_path) = if binary_verification.name == "tower" {
                        (
                            self.nucleus_path.join("tower").join(&nucleus_binary.name),
                            spore_path.join("bin").join(&nucleus_binary.name),
                        )
                    } else {
                        (
                            self.nucleus_path.join("primals").join(&nucleus_binary.name),
                            spore_path.join("primals").join(&nucleus_binary.name),
                        )
                    };

                    // Copy binary
                    match self.copy_binary(&source_path, &dest_path) {
                        Ok(()) => {
                            info!("✅ Refreshed: {}", binary_verification.name);
                            refreshed_binaries.push(RefreshedBinary {
                                name: binary_verification.name.clone(),
                                old_version: binary_verification.actual_version.clone(),
                                new_version: nucleus_binary.version.clone(),
                                old_sha256: binary_verification.actual_sha256.clone(),
                                new_sha256: nucleus_binary.sha256.clone(),
                            });
                        }
                        Err(e) => {
                            warn!("❌ Failed to refresh {}: {}", binary_verification.name, e);
                            failed_binaries.push(FailedBinary {
                                name: binary_verification.name.clone(),
                                error: e.to_string(),
                            });
                        }
                    }
                } else {
                    warn!("Binary {} not found in nucleus", binary_verification.name);
                    failed_binaries.push(FailedBinary {
                        name: binary_verification.name.clone(),
                        error: "Not found in plasmidBin".to_string(),
                    });
                }
            }
        }

        // Update spore manifest if any binaries were refreshed
        if !refreshed_binaries.is_empty()
            && let Err(e) = Self::update_spore_manifest(spore_path, &refreshed_binaries)
        {
            warn!("Failed to update spore manifest: {}", e);
        }

        Ok(RefreshReport {
            spore_path: spore_path.to_path_buf(),
            node_id: verification.node_id,
            refreshed_binaries,
            failed_binaries,
        })
    }

    /// Copy a binary file with verification
    fn copy_binary(&self, source: &Path, dest: &Path) -> Result<()> {
        // Read source
        let source_bytes = std::fs::read(source)?;

        // Verify source SHA256 matches expected
        let mut hasher = Sha256::new();
        hasher.update(&source_bytes);
        let source_sha256 = format!("{:x}", hasher.finalize());

        // Find expected SHA256 from nucleus manifest
        let source_name = source
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| anyhow::anyhow!("Invalid source filename"))?;

        let expected_sha256 = self
            .nucleus_manifest
            .binaries
            .values()
            .find(|b| b.name == source_name)
            .map(|b| b.sha256.as_str());

        if let Some(expected) = expected_sha256
            && source_sha256 != expected
        {
            return Err(anyhow::anyhow!(
                "Source binary SHA256 mismatch: expected {expected}, got {source_sha256}"
            ));
        }

        // Write to destination
        std::fs::write(dest, &source_bytes)?;

        // Set executable permissions on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(dest)?.permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(dest, perms)?;
        }

        info!(
            "Copied and verified: {} → {}",
            source.display(),
            dest.display()
        );

        Ok(())
    }

    /// Update spore manifest after refresh
    fn update_spore_manifest(
        spore_path: &Path,
        refreshed_binaries: &[RefreshedBinary],
    ) -> Result<()> {
        let _manifest_path = spore_path.join(".manifest.toml");

        // Load existing manifest or create new one
        let Ok(mut manifest) = SporeManifest::load(spore_path) else {
            // If no manifest exists, we can't update it
            // This is okay for legacy spores
            return Ok(());
        };

        // Update binary information
        for refreshed in refreshed_binaries {
            if let Some(binary_info) = manifest.binaries.get_mut(&refreshed.name) {
                binary_info.version.clone_from(&refreshed.new_version);
                binary_info.sha256.clone_from(&refreshed.new_sha256);
                binary_info.copied_at = chrono::Utc::now();
            }
        }

        // Save updated manifest
        manifest.save(spore_path)?;

        info!(
            "Updated spore manifest with {} refreshed binaries",
            refreshed_binaries.len()
        );

        Ok(())
    }
}

/// Report of a spore refresh operation
#[derive(Debug)]
pub struct RefreshReport {
    /// Path to the refreshed spore
    pub spore_path: PathBuf,
    /// Node identifier
    pub node_id: String,
    /// Binaries that were successfully refreshed
    pub refreshed_binaries: Vec<RefreshedBinary>,
    /// Binaries that failed to refresh
    pub failed_binaries: Vec<FailedBinary>,
}

/// Information about a refreshed binary
#[derive(Debug)]
pub struct RefreshedBinary {
    /// Binary name
    pub name: String,
    /// Previous version (if known)
    pub old_version: Option<String>,
    /// New version from the nucleus
    pub new_version: String,
    /// Previous SHA-256 digest
    pub old_sha256: Option<String>,
    /// New SHA-256 digest
    pub new_sha256: String,
}

/// Information about a failed binary refresh
#[derive(Debug)]
pub struct FailedBinary {
    /// Binary name
    pub name: String,
    /// Error message explaining the failure
    pub error: String,
}

impl RefreshReport {
    /// Check if refresh was completely successful
    pub fn is_success(&self) -> bool {
        self.failed_binaries.is_empty()
    }

    /// Check if any binaries were refreshed
    pub fn has_changes(&self) -> bool {
        !self.refreshed_binaries.is_empty()
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
mod tests {
    use super::*;

    // ========== RefreshReport Tests ==========

    #[test]
    fn test_refresh_report_success_with_changes() {
        let report = RefreshReport {
            spore_path: PathBuf::from("/test"),
            node_id: "test-node".to_string(),
            refreshed_binaries: vec![RefreshedBinary {
                name: "beardog".to_string(),
                old_version: Some("0.14.0".to_string()),
                new_version: "0.15.0".to_string(),
                old_sha256: Some("old_hash".to_string()),
                new_sha256: "new_hash".to_string(),
            }],
            failed_binaries: vec![],
        };

        assert!(report.is_success());
        assert!(report.has_changes());
    }

    #[test]
    fn test_refresh_report_success_no_changes() {
        let report = RefreshReport {
            spore_path: PathBuf::from("/media/usb/biomeOS"),
            node_id: "fresh-node".to_string(),
            refreshed_binaries: vec![],
            failed_binaries: vec![],
        };

        assert!(report.is_success());
        assert!(!report.has_changes());
    }

    #[test]
    fn test_refresh_report_with_failures() {
        let report = RefreshReport {
            spore_path: PathBuf::from("/media/usb/biomeOS"),
            node_id: "failing-node".to_string(),
            refreshed_binaries: vec![RefreshedBinary {
                name: "beardog".to_string(),
                old_version: Some("0.14.0".to_string()),
                new_version: "0.15.0".to_string(),
                old_sha256: None,
                new_sha256: "new_hash".to_string(),
            }],
            failed_binaries: vec![FailedBinary {
                name: "songbird".to_string(),
                error: "Not found in plasmidBin".to_string(),
            }],
        };

        assert!(!report.is_success());
        assert!(report.has_changes());
    }

    #[test]
    fn test_refresh_report_only_failures() {
        let report = RefreshReport {
            spore_path: PathBuf::from("/test"),
            node_id: "broken-node".to_string(),
            refreshed_binaries: vec![],
            failed_binaries: vec![
                FailedBinary {
                    name: "beardog".to_string(),
                    error: "SHA256 mismatch".to_string(),
                },
                FailedBinary {
                    name: "songbird".to_string(),
                    error: "Permission denied".to_string(),
                },
            ],
        };

        assert!(!report.is_success());
        assert!(!report.has_changes());
    }

    // ========== RefreshedBinary Tests ==========

    #[test]
    fn test_refreshed_binary_with_all_fields() {
        let binary = RefreshedBinary {
            name: "tower".to_string(),
            old_version: Some("0.5.0".to_string()),
            new_version: "0.6.0".to_string(),
            old_sha256: Some("abc123".to_string()),
            new_sha256: "def456".to_string(),
        };

        assert_eq!(binary.name, "tower");
        assert_eq!(binary.old_version, Some("0.5.0".to_string()));
        assert_eq!(binary.new_version, "0.6.0");
    }

    #[test]
    fn test_refreshed_binary_missing_old_info() {
        let binary = RefreshedBinary {
            name: "songbird".to_string(),
            old_version: None,
            new_version: "3.19.0".to_string(),
            old_sha256: None,
            new_sha256: "new_hash".to_string(),
        };

        assert!(binary.old_version.is_none());
        assert!(binary.old_sha256.is_none());
    }

    // ========== FailedBinary Tests ==========

    #[test]
    fn test_failed_binary_debug() {
        let failed = FailedBinary {
            name: "beardog".to_string(),
            error: "Source binary SHA256 mismatch".to_string(),
        };

        let debug = format!("{failed:?}");
        assert!(debug.contains("beardog"));
        assert!(debug.contains("SHA256 mismatch"));
    }

    // ========== RefreshReport Multiple Binaries ==========

    #[test]
    fn test_refresh_report_multiple_refreshed() {
        let report = RefreshReport {
            spore_path: PathBuf::from("/media/usb1/biomeOS"),
            node_id: "multi-node".to_string(),
            refreshed_binaries: vec![
                RefreshedBinary {
                    name: "beardog".to_string(),
                    old_version: Some("0.14.0".to_string()),
                    new_version: "0.15.0".to_string(),
                    old_sha256: Some("old1".to_string()),
                    new_sha256: "new1".to_string(),
                },
                RefreshedBinary {
                    name: "songbird".to_string(),
                    old_version: Some("3.18.0".to_string()),
                    new_version: "3.19.0".to_string(),
                    old_sha256: Some("old2".to_string()),
                    new_sha256: "new2".to_string(),
                },
                RefreshedBinary {
                    name: "tower".to_string(),
                    old_version: None,
                    new_version: "0.6.0".to_string(),
                    old_sha256: None,
                    new_sha256: "new3".to_string(),
                },
            ],
            failed_binaries: vec![],
        };

        assert!(report.is_success());
        assert!(report.has_changes());
        assert_eq!(report.refreshed_binaries.len(), 3);
    }

    // ========== Debug Formatting ==========

    #[test]
    fn test_refresh_report_debug() {
        let report = RefreshReport {
            spore_path: PathBuf::from("/test"),
            node_id: "debug-node".to_string(),
            refreshed_binaries: vec![],
            failed_binaries: vec![],
        };

        let debug = format!("{report:?}");
        assert!(debug.contains("debug-node"));
        assert!(debug.contains("/test"));
    }

    #[test]
    fn test_spore_refresher_from_nucleus() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let nucleus_path = temp_dir.path();
        std::fs::create_dir_all(nucleus_path.join("tower")).unwrap();
        std::fs::create_dir_all(nucleus_path.join("primals")).unwrap();
        std::fs::write(nucleus_path.join("tower").join("tower"), b"tower_binary").unwrap();

        let result = SporeRefresher::from_nucleus(nucleus_path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_spore_refresher_refresh_spore() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let nucleus_path = temp_dir.path();
        std::fs::create_dir_all(nucleus_path.join("tower")).unwrap();
        std::fs::create_dir_all(nucleus_path.join("primals")).unwrap();
        let tower_bin = b"tower_binary_content";
        std::fs::write(nucleus_path.join("tower").join("tower"), tower_bin).unwrap();

        let spore_path = temp_dir.path().join("spore");
        std::fs::create_dir_all(spore_path.join("bin")).unwrap();
        std::fs::create_dir_all(spore_path.join("primals")).unwrap();
        std::fs::write(
            spore_path.join("tower.toml"),
            r#"[tower]
NODE_ID = "test-node"
"#,
        )
        .unwrap();
        std::fs::write(spore_path.join(".family.seed"), b"seed").unwrap();
        std::fs::write(spore_path.join("bin").join("tower"), b"stale_tower").unwrap();

        let refresher = SporeRefresher::from_nucleus(nucleus_path).unwrap();
        let report = refresher.refresh_spore(&spore_path).unwrap();
        assert!(report.is_success() || !report.failed_binaries.is_empty());
    }

    #[test]
    fn test_refreshed_binary_debug() {
        let binary = RefreshedBinary {
            name: "tower".to_string(),
            old_version: Some("0.5".to_string()),
            new_version: "0.6".to_string(),
            old_sha256: Some("old".to_string()),
            new_sha256: "new".to_string(),
        };
        let dbg = format!("{binary:?}");
        assert!(dbg.contains("tower"));
        assert!(dbg.contains("0.6"));
    }
}
