// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

// crates/biomeos-spore/src/manifest.rs
//! Binary and spore manifest types for verification and tracking

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Detect package version from build metadata
fn detect_version() -> &'static str {
    option_env!("CARGO_PKG_VERSION").unwrap_or("unknown")
}

/// Detect git commit hash from build-time CI metadata
fn detect_git_commit() -> &'static str {
    option_env!("GIT_COMMIT_HASH").unwrap_or("unknown")
}

/// Manifest for plasmidBin binaries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinaryManifest {
    /// Manifest metadata (version, timestamps, pipeline)
    pub manifest: ManifestMeta,
    /// Map of binary name to binary information
    pub binaries: HashMap<String, BinaryInfo>,
    /// Minimum version requirements for dependent primals
    pub compatibility: CompatibilityInfo,
}

/// Manifest metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestMeta {
    /// Manifest format version
    pub version: String,
    /// When this manifest was generated
    pub created_at: DateTime<Utc>,
    /// CI pipeline run identifier
    pub pipeline_run: String,
}

/// Information about a single binary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinaryInfo {
    /// Binary name (e.g. "biomeos", "beardog")
    pub name: String,
    /// Semantic version of the binary
    pub version: String,
    /// Git commit hash the binary was built from
    pub git_commit: String,
    /// Build timestamp
    pub build_date: DateTime<Utc>,
    /// SHA-256 digest of the binary
    pub sha256: String,
    /// File size in bytes
    pub size_bytes: u64,
    /// Source repository URL
    pub source_repo: String,
    /// Cargo feature flags enabled at build time
    #[serde(default)]
    pub features: Vec<String>,
}

/// Compatibility requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatibilityInfo {
    /// Minimum compatible Tower version
    pub min_tower_version: String,
    /// Minimum compatible `BearDog` version
    pub min_beardog_version: String,
    /// Minimum compatible Songbird version
    pub min_songbird_version: String,
}

/// Manifest for a deployed spore
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SporeManifest {
    /// Spore identity and metadata
    pub spore: SporeInfo,
    /// Genetic lineage information
    pub lineage: LineageInfo,
    /// Binaries deployed in this spore
    pub binaries: HashMap<String, SporeBinaryInfo>,
    /// Chronological deployment records
    #[serde(default)]
    pub deployment_history: Vec<DeploymentRecord>,
}

/// Spore metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SporeInfo {
    /// Unique node identifier
    pub node_id: String,
    /// Family identifier (genetic lineage root)
    pub family_id: String,
    /// Spore creation timestamp
    pub created_at: DateTime<Utc>,
    /// Identity of the creator
    pub created_by: String,
    /// Type classification (e.g. "usb", "network")
    pub spore_type: String,
    /// Deployment batch identifier
    pub deployment_batch: String,
}

/// Genetic lineage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineageInfo {
    /// SHA-256 hash of the parent family seed
    pub parent_seed_hash: String,
    /// SHA-256 hash of the derived child seed
    pub child_seed_hash: String,
    /// Derivation method used (e.g. "`sha256_chain`")
    pub derivation_method: String,
}

/// Information about a binary copied to a spore
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SporeBinaryInfo {
    /// Binary name
    pub name: String,
    /// Binary version
    pub version: String,
    /// SHA-256 digest of the binary on disk
    pub sha256: String,
    /// Path or identifier of the source manifest
    pub source_manifest: String,
    /// When this binary was copied into the spore
    pub copied_at: DateTime<Utc>,
}

/// Record of a deployment event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentRecord {
    /// Deployment timestamp
    pub deployed_at: DateTime<Utc>,
    /// Target host or node identifier
    pub deployed_to: String,
    /// Identity of who triggered the deployment
    pub deployed_by: String,
    /// Whether the deployment succeeded
    pub success: bool,
    /// Optional free-form notes
    #[serde(default)]
    pub notes: String,
}

impl BinaryManifest {
    /// Load binary manifest from plasmidBin/MANIFEST.toml
    pub fn load(nucleus_path: impl AsRef<Path>) -> Result<Self> {
        let manifest_path = nucleus_path.as_ref().join("MANIFEST.toml");
        let manifest_str = std::fs::read_to_string(manifest_path)?;
        let manifest: Self = toml::from_str(&manifest_str)?;
        Ok(manifest)
    }

    /// Save binary manifest to file
    pub fn save(&self, path: impl AsRef<Path>) -> Result<()> {
        let manifest_str = toml::to_string_pretty(self)?;
        std::fs::write(path.as_ref(), manifest_str)?;
        Ok(())
    }

    /// Create a new binary manifest from plasmidBin directory
    pub fn from_nucleus(nucleus_path: impl AsRef<Path>) -> Result<Self> {
        use sha2::{Digest, Sha256};
        use std::fs;

        let nucleus_path = nucleus_path.as_ref();
        let mut binaries = HashMap::new();

        // Scan tower binary
        let tower_path = nucleus_path.join("tower").join("tower");
        if tower_path.exists() {
            let bytes = fs::read(&tower_path)?;
            let mut hasher = Sha256::new();
            hasher.update(&bytes);
            let sha256 = format!("{:x}", hasher.finalize());

            binaries.insert(
                "tower".to_string(),
                BinaryInfo {
                    name: "tower".to_string(),
                    version: detect_version().to_string(),
                    git_commit: detect_git_commit().to_string(),
                    build_date: Utc::now(),
                    sha256,
                    size_bytes: bytes.len() as u64,
                    source_repo: "biomeOS".to_string(),
                    features: vec![],
                },
            );
        }

        // Scan primals directory
        let primals_dir = nucleus_path.join("primals");
        if primals_dir.exists() {
            for entry in fs::read_dir(&primals_dir)? {
                let entry = entry?;
                let path = entry.path();

                if path.is_file() {
                    let bytes = fs::read(&path)?;
                    let mut hasher = Sha256::new();
                    hasher.update(&bytes);
                    let sha256 = format!("{:x}", hasher.finalize());

                    let file_name = path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown")
                        .to_string();

                    let key = file_name
                        .strip_suffix("-server")
                        .unwrap_or(&file_name)
                        .to_string();

                    let binary_info = BinaryInfo {
                        name: file_name,
                        version: detect_version().to_string(),
                        git_commit: detect_git_commit().to_string(),
                        build_date: Utc::now(),
                        sha256,
                        size_bytes: bytes.len() as u64,
                        source_repo: String::new(),
                        features: vec![],
                    };

                    binaries.insert(key, binary_info);
                }
            }
        }

        Ok(Self {
            manifest: ManifestMeta {
                version: "1.0".to_string(),
                created_at: Utc::now(),
                pipeline_run: format!("harvest-{}", Utc::now().format("%Y-%m-%d-%H%M")),
            },
            binaries,
            compatibility: CompatibilityInfo {
                min_tower_version: "0.6.0".to_string(),
                min_beardog_version: "0.15.0".to_string(),
                min_songbird_version: "3.19.0".to_string(),
            },
        })
    }
}

impl SporeManifest {
    /// Load spore manifest from .manifest.toml
    pub fn load(spore_path: impl AsRef<Path>) -> Result<Self> {
        let manifest_path = spore_path.as_ref().join(".manifest.toml");
        let manifest_str = std::fs::read_to_string(manifest_path)?;
        let manifest: Self = toml::from_str(&manifest_str)?;
        Ok(manifest)
    }

    /// Save spore manifest to file
    pub fn save(&self, spore_path: impl AsRef<Path>) -> Result<()> {
        let manifest_path = spore_path.as_ref().join(".manifest.toml");
        let manifest_str = toml::to_string_pretty(self)?;
        std::fs::write(manifest_path, manifest_str)?;
        Ok(())
    }

    /// Create a new spore manifest
    #[must_use]
    pub fn new(
        node_id: String,
        family_id: String,
        spore_type: String,
        deployment_batch: String,
        parent_seed_hash: String,
        child_seed_hash: String,
    ) -> Self {
        Self {
            spore: SporeInfo {
                node_id,
                family_id,
                created_at: Utc::now(),
                created_by: format!("biomeos v{}", env!("CARGO_PKG_VERSION")),
                spore_type,
                deployment_batch,
            },
            lineage: LineageInfo {
                parent_seed_hash,
                child_seed_hash,
                derivation_method: "SHA256(parent || node_id || batch)".to_string(),
            },
            binaries: HashMap::new(),
            deployment_history: vec![],
        }
    }

    /// Add a binary to the spore manifest
    pub fn add_binary(&mut self, name: String, version: String, sha256: String) {
        self.binaries.insert(
            name.clone(),
            SporeBinaryInfo {
                name,
                version,
                sha256,
                source_manifest: "plasmidBin/MANIFEST.toml".to_string(),
                copied_at: Utc::now(),
            },
        );
    }

    /// Record a deployment event
    pub fn record_deployment(&mut self, deployed_to: String, deployed_by: String, success: bool) {
        self.deployment_history.push(DeploymentRecord {
            deployed_at: Utc::now(),
            deployed_to,
            deployed_by,
            success,
            notes: String::new(),
        });
    }
}


#[cfg(test)]
#[path = "manifest_tests.rs"]
mod tests;
