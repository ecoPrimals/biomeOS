// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

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
    /// Minimum compatible BearDog version
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
    /// Derivation method used (e.g. "sha256_chain")
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
                    git_commit: "unknown".to_string(), // Populated from VERSION.txt when available
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
                        version: "unknown".to_string(),
                        git_commit: "unknown".to_string(),
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
    use tempfile::TempDir;

    // ========== Helper ==========

    fn create_test_binary_manifest() -> BinaryManifest {
        let mut binaries = HashMap::new();
        binaries.insert(
            "beardog".to_string(),
            BinaryInfo {
                name: "beardog-server".to_string(),
                version: "0.15.0".to_string(),
                git_commit: "abc123".to_string(),
                build_date: Utc::now(),
                sha256: "deadbeef".to_string(),
                size_bytes: 4096,
                source_repo: "ecoPrimals/phase1/beardog".to_string(),
                features: vec!["btsp-api".to_string()],
            },
        );

        BinaryManifest {
            manifest: ManifestMeta {
                version: "1.0".to_string(),
                created_at: Utc::now(),
                pipeline_run: "test-run".to_string(),
            },
            binaries,
            compatibility: CompatibilityInfo {
                min_tower_version: "0.6.0".to_string(),
                min_beardog_version: "0.15.0".to_string(),
                min_songbird_version: "3.19.0".to_string(),
            },
        }
    }

    // ========== BinaryManifest Tests ==========

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
        assert!(manifest.binaries.is_empty());
    }

    #[test]
    fn test_binary_manifest_serialization_roundtrip() {
        let manifest = create_test_binary_manifest();

        // Serialize to TOML
        let toml_str = toml::to_string_pretty(&manifest).expect("serialize to TOML");
        assert!(toml_str.contains("beardog"));

        // Deserialize back
        let deserialized: BinaryManifest =
            toml::from_str(&toml_str).expect("deserialize from TOML");
        assert_eq!(deserialized.manifest.version, "1.0");
        assert_eq!(deserialized.binaries.len(), 1);
        assert_eq!(deserialized.binaries["beardog"].name, "beardog-server");
        assert_eq!(deserialized.binaries["beardog"].size_bytes, 4096);
    }

    #[test]
    fn test_binary_manifest_json_roundtrip() {
        let manifest = create_test_binary_manifest();

        let json = serde_json::to_string(&manifest).expect("serialize to JSON");
        let deserialized: BinaryManifest =
            serde_json::from_str(&json).expect("deserialize from JSON");

        assert_eq!(deserialized.compatibility.min_tower_version, "0.6.0");
        assert_eq!(deserialized.binaries["beardog"].features, vec!["btsp-api"]);
    }

    #[test]
    fn test_binary_manifest_save_and_load() {
        let temp_dir = TempDir::new().expect("create temp dir");
        let manifest = create_test_binary_manifest();

        // Save
        manifest
            .save(temp_dir.path().join("MANIFEST.toml"))
            .expect("save manifest");

        // Load
        let loaded = BinaryManifest::load(temp_dir.path()).expect("load manifest");

        assert_eq!(loaded.manifest.version, manifest.manifest.version);
        assert_eq!(loaded.binaries.len(), manifest.binaries.len());
        assert_eq!(
            loaded.compatibility.min_beardog_version,
            manifest.compatibility.min_beardog_version
        );
    }

    #[test]
    fn test_binary_manifest_load_missing_file() {
        let temp_dir = TempDir::new().expect("create temp dir");
        let result = BinaryManifest::load(temp_dir.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_binary_manifest_from_nucleus_empty_dir() {
        let temp_dir = TempDir::new().expect("create temp dir");
        let manifest = BinaryManifest::from_nucleus(temp_dir.path()).expect("from_nucleus");

        assert!(manifest.binaries.is_empty());
        assert_eq!(manifest.manifest.version, "1.0");
    }

    #[test]
    fn test_binary_manifest_from_nucleus_with_tower() {
        let temp_dir = TempDir::new().expect("create temp dir");

        // Create tower binary
        let tower_dir = temp_dir.path().join("tower");
        std::fs::create_dir_all(&tower_dir).expect("create tower dir");
        std::fs::write(tower_dir.join("tower"), b"fake binary").expect("write tower");

        let manifest = BinaryManifest::from_nucleus(temp_dir.path()).expect("from_nucleus");

        assert_eq!(manifest.binaries.len(), 1);
        assert!(manifest.binaries.contains_key("tower"));
        assert_eq!(manifest.binaries["tower"].name, "tower");
        assert_eq!(manifest.binaries["tower"].size_bytes, 11); // "fake binary" = 11 bytes
        assert!(!manifest.binaries["tower"].sha256.is_empty());
    }

    #[test]
    fn test_binary_manifest_from_nucleus_with_primals() {
        let temp_dir = TempDir::new().expect("create temp dir");

        // Create primals directory with known binaries
        let primals_dir = temp_dir.path().join("primals");
        std::fs::create_dir_all(&primals_dir).expect("create primals dir");
        std::fs::write(primals_dir.join("beardog-server"), b"beardog binary")
            .expect("write beardog");
        std::fs::write(primals_dir.join("songbird"), b"songbird binary").expect("write songbird");
        // Unknown binary should be skipped
        std::fs::write(primals_dir.join("unknown-primal"), b"unknown").expect("write unknown");

        let manifest = BinaryManifest::from_nucleus(temp_dir.path()).expect("from_nucleus");

        assert_eq!(manifest.binaries.len(), 3);
        assert!(manifest.binaries.contains_key("beardog"));
        assert!(manifest.binaries.contains_key("songbird"));
        assert!(manifest.binaries.contains_key("unknown-primal"));
    }

    // ========== SporeManifest Tests ==========

    #[test]
    fn test_spore_manifest_creation() {
        let manifest = SporeManifest::new(
            "node-alpha".to_string(),
            "1894e909e454".to_string(),
            "LiveSpore".to_string(),
            "2026-01-08".to_string(),
            "parent_hash".to_string(),
            "child_hash".to_string(),
        );

        assert_eq!(manifest.spore.node_id, "node-alpha");
        assert_eq!(manifest.spore.family_id, "1894e909e454");
        assert_eq!(manifest.spore.spore_type, "LiveSpore");
        assert_eq!(manifest.spore.deployment_batch, "2026-01-08");
        assert_eq!(manifest.lineage.parent_seed_hash, "parent_hash");
        assert_eq!(manifest.lineage.child_seed_hash, "child_hash");
        assert_eq!(
            manifest.lineage.derivation_method,
            "SHA256(parent || node_id || batch)"
        );
        assert!(manifest.binaries.is_empty());
        assert!(manifest.deployment_history.is_empty());
    }

    #[test]
    fn test_spore_manifest_add_binary() {
        let mut manifest = SporeManifest::new(
            "node-alpha".to_string(),
            "fam123".to_string(),
            "LiveSpore".to_string(),
            "batch1".to_string(),
            "phash".to_string(),
            "chash".to_string(),
        );

        manifest.add_binary(
            "beardog-server".to_string(),
            "0.15.0".to_string(),
            "abc123".to_string(),
        );
        manifest.add_binary(
            "songbird".to_string(),
            "3.19.0".to_string(),
            "def456".to_string(),
        );

        assert_eq!(manifest.binaries.len(), 2);
        assert_eq!(manifest.binaries["beardog-server"].version, "0.15.0");
        assert_eq!(manifest.binaries["songbird"].sha256, "def456");
        assert_eq!(
            manifest.binaries["songbird"].source_manifest,
            "plasmidBin/MANIFEST.toml"
        );
    }

    #[test]
    fn test_spore_manifest_record_deployment() {
        let mut manifest = SporeManifest::new(
            "node-beta".to_string(),
            "fam456".to_string(),
            "ColdSpore".to_string(),
            "batch2".to_string(),
            "phash".to_string(),
            "chash".to_string(),
        );

        manifest.record_deployment("laptop-01".to_string(), "admin".to_string(), true);
        manifest.record_deployment("laptop-02".to_string(), "admin".to_string(), false);

        assert_eq!(manifest.deployment_history.len(), 2);
        assert!(manifest.deployment_history[0].success);
        assert_eq!(manifest.deployment_history[0].deployed_to, "laptop-01");
        assert!(!manifest.deployment_history[1].success);
        assert!(manifest.deployment_history[1].notes.is_empty());
    }

    #[test]
    fn test_spore_manifest_save_and_load() {
        let temp_dir = TempDir::new().expect("create temp dir");

        let mut manifest = SporeManifest::new(
            "node-gamma".to_string(),
            "fam789".to_string(),
            "LiveSpore".to_string(),
            "batch3".to_string(),
            "phash".to_string(),
            "chash".to_string(),
        );
        manifest.add_binary(
            "beardog-server".to_string(),
            "0.15.0".to_string(),
            "hash1".to_string(),
        );
        manifest.record_deployment("server-01".to_string(), "ci-bot".to_string(), true);

        // Save
        manifest.save(temp_dir.path()).expect("save manifest");

        // Load
        let loaded = SporeManifest::load(temp_dir.path()).expect("load manifest");

        assert_eq!(loaded.spore.node_id, "node-gamma");
        assert_eq!(loaded.binaries.len(), 1);
        assert_eq!(loaded.deployment_history.len(), 1);
        assert!(loaded.deployment_history[0].success);
    }

    #[test]
    fn test_spore_manifest_load_missing() {
        let temp_dir = TempDir::new().expect("create temp dir");
        let result = SporeManifest::load(temp_dir.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_spore_manifest_serialization_roundtrip() {
        let mut manifest = SporeManifest::new(
            "node-delta".to_string(),
            "fam000".to_string(),
            "LiveSpore".to_string(),
            "batch4".to_string(),
            "ph".to_string(),
            "ch".to_string(),
        );
        manifest.add_binary("tower".to_string(), "0.6.0".to_string(), "h1".to_string());

        let toml_str = toml::to_string_pretty(&manifest).expect("serialize");
        let deserialized: SporeManifest = toml::from_str(&toml_str).expect("deserialize");

        assert_eq!(deserialized.spore.node_id, "node-delta");
        assert_eq!(
            deserialized.lineage.derivation_method,
            manifest.lineage.derivation_method
        );
    }

    // ========== BinaryInfo Tests ==========

    #[test]
    fn test_binary_info_with_features() {
        let info = BinaryInfo {
            name: "beardog-server".to_string(),
            version: "0.15.0".to_string(),
            git_commit: "abc123def".to_string(),
            build_date: Utc::now(),
            sha256: "deadbeef".to_string(),
            size_bytes: 8192,
            source_repo: "ecoPrimals/phase1/beardog".to_string(),
            features: vec!["btsp-api".to_string(), "unix-socket".to_string()],
        };

        assert_eq!(info.features.len(), 2);

        // Verify serde default for features
        let json = r#"{
            "name": "test",
            "version": "1.0",
            "git_commit": "abc",
            "build_date": "2026-01-01T00:00:00Z",
            "sha256": "hash",
            "size_bytes": 100,
            "source_repo": "test"
        }"#;
        let deserialized: BinaryInfo = serde_json::from_str(json).expect("deserialize");
        assert!(deserialized.features.is_empty()); // default should be empty vec
    }

    // ========== DeploymentRecord Tests ==========

    #[test]
    fn test_deployment_record_default_notes() {
        let json = r#"{
            "deployed_at": "2026-01-01T00:00:00Z",
            "deployed_to": "host-1",
            "deployed_by": "admin",
            "success": true
        }"#;
        let record: DeploymentRecord =
            serde_json::from_str(json).expect("deserialize with default notes");
        assert!(record.notes.is_empty());
    }

    // ========== ManifestMeta Tests ==========

    #[test]
    fn test_manifest_meta_clone() {
        let meta = ManifestMeta {
            version: "1.0".to_string(),
            created_at: Utc::now(),
            pipeline_run: "harvest-2026-01-10".to_string(),
        };
        let cloned = meta.clone();
        assert_eq!(cloned.version, meta.version);
        assert_eq!(cloned.pipeline_run, meta.pipeline_run);
    }

    // ========== CompatibilityInfo Tests ==========

    #[test]
    fn test_compatibility_info_serialization() {
        let compat = CompatibilityInfo {
            min_tower_version: "0.6.0".to_string(),
            min_beardog_version: "0.15.0".to_string(),
            min_songbird_version: "3.19.0".to_string(),
        };

        let json = serde_json::to_string(&compat).expect("serialize");
        let deserialized: CompatibilityInfo = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(deserialized.min_tower_version, "0.6.0");
        assert_eq!(deserialized.min_beardog_version, "0.15.0");
        assert_eq!(deserialized.min_songbird_version, "3.19.0");
    }
}
