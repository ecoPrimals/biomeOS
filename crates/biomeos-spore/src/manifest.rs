// crates/biomeos-spore/src/manifest.rs
//! Binary and spore manifest types for verification and tracking

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Manifest for plasmidBin binaries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinaryManifest {
    pub manifest: ManifestMeta,
    pub binaries: HashMap<String, BinaryInfo>,
    pub compatibility: CompatibilityInfo,
}

/// Manifest metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestMeta {
    pub version: String,
    pub created_at: DateTime<Utc>,
    pub pipeline_run: String,
}

/// Information about a single binary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinaryInfo {
    pub name: String,
    pub version: String,
    pub git_commit: String,
    pub build_date: DateTime<Utc>,
    pub sha256: String,
    pub size_bytes: u64,
    pub source_repo: String,
    #[serde(default)]
    pub features: Vec<String>,
}

/// Compatibility requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatibilityInfo {
    pub min_tower_version: String,
    pub min_beardog_version: String,
    pub min_songbird_version: String,
}

/// Manifest for a deployed spore
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SporeManifest {
    pub spore: SporeInfo,
    pub lineage: LineageInfo,
    pub binaries: HashMap<String, SporeBinaryInfo>,
    #[serde(default)]
    pub deployment_history: Vec<DeploymentRecord>,
}

/// Spore metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SporeInfo {
    pub node_id: String,
    pub family_id: String,
    pub created_at: DateTime<Utc>,
    pub created_by: String,
    pub spore_type: String,
    pub deployment_batch: String,
}

/// Genetic lineage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineageInfo {
    pub parent_seed_hash: String,
    pub child_seed_hash: String,
    pub derivation_method: String,
}

/// Information about a binary copied to a spore
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SporeBinaryInfo {
    pub name: String,
    pub version: String,
    pub sha256: String,
    pub source_manifest: String,
    pub copied_at: DateTime<Utc>,
}

/// Record of a deployment event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentRecord {
    pub deployed_at: DateTime<Utc>,
    pub deployed_to: String,
    pub deployed_by: String,
    pub success: bool,
    #[serde(default)]
    pub notes: String,
}

impl BinaryManifest {
    /// Load binary manifest from plasmidBin/MANIFEST.toml
    pub fn load(nucleus_path: impl AsRef<Path>) -> Result<Self> {
        let manifest_path = nucleus_path.as_ref().join("MANIFEST.toml");
        let manifest_str = std::fs::read_to_string(manifest_path)?;
        let manifest: BinaryManifest = toml::from_str(&manifest_str)?;
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
                    version: env!("CARGO_PKG_VERSION").to_string(),
                    git_commit: "unknown".to_string(), // TODO: Read from VERSION.txt
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

                    let (key, binary_info) = match file_name.as_str() {
                        "beardog-server" => (
                            "beardog".to_string(),
                            BinaryInfo {
                                name: "beardog-server".to_string(),
                                version: "0.15.0".to_string(), // TODO: Extract from binary
                                git_commit: "unknown".to_string(),
                                build_date: Utc::now(),
                                sha256,
                                size_bytes: bytes.len() as u64,
                                source_repo: "ecoPrimals/phase1/beardog".to_string(),
                                features: vec!["btsp-api".to_string(), "unix-socket".to_string()],
                            },
                        ),
                        "songbird" => (
                            "songbird".to_string(),
                            BinaryInfo {
                                name: "songbird".to_string(),
                                version: "3.19.0".to_string(), // TODO: Extract from binary
                                git_commit: "unknown".to_string(),
                                build_date: Utc::now(),
                                sha256,
                                size_bytes: bytes.len() as u64,
                                source_repo: "ecoPrimals/phase1/songbird".to_string(),
                                features: vec!["btsp-client".to_string(), "port-free".to_string()],
                            },
                        ),
                        _ => continue, // Skip unknown binaries
                    };

                    binaries.insert(key, binary_info);
                }
            }
        }

        Ok(BinaryManifest {
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
        let manifest: SporeManifest = toml::from_str(&manifest_str)?;
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
    pub fn new(
        node_id: String,
        family_id: String,
        spore_type: String,
        deployment_batch: String,
        parent_seed_hash: String,
        child_seed_hash: String,
    ) -> Self {
        SporeManifest {
            spore: SporeInfo {
                node_id,
                family_id,
                created_at: Utc::now(),
                created_by: format!("biomeos-cli v{}", env!("CARGO_PKG_VERSION")),
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
mod tests {
    use super::*;

    #[test]
    fn test_binary_manifest_creation() {
        let manifest = BinaryManifest {
            manifest: ManifestMeta {
                version: "1.0".to_string(),
                created_at: Utc::now(),
                pipeline_run: "test-run".to_string(),
            },
            binaries: HashMap::new(),
            compatibility: CompatibilityInfo {
                min_tower_version: "0.6.0".to_string(),
                min_beardog_version: "0.15.0".to_string(),
                min_songbird_version: "3.19.0".to_string(),
            },
        };

        assert_eq!(manifest.manifest.version, "1.0");
    }

    #[test]
    fn test_spore_manifest_creation() {
        let mut manifest = SporeManifest::new(
            "node-alpha".to_string(),
            "nat0".to_string(),
            "LiveSpore".to_string(),
            "2026-01-08".to_string(),
            "parent_hash".to_string(),
            "child_hash".to_string(),
        );

        manifest.add_binary(
            "beardog-server".to_string(),
            "0.15.0".to_string(),
            "abc123".to_string(),
        );

        assert_eq!(manifest.spore.node_id, "node-alpha");
        assert_eq!(manifest.binaries.len(), 1);
    }
}
