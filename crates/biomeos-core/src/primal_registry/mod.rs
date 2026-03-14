// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Primal binary registry and deployment system
//!
//! BiomeOS as a bootable platform needs to:
//! 1. Discover available primal binaries (local USB, remote GitHub)
//! 2. Manage primal versions
//! 3. Deploy primals to target nodes
//! 4. Update primals from registries
//!
//! This is the foundation for BiomeOS as a "PopOS/Windows bootloader" for primals.

use anyhow::Result;
use biomeos_types::CapabilityTaxonomy; // Phase 1 enum (not PrimalCapability struct!)
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Primal binary metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalBinary {
    /// Primal name (e.g., "beardog", "songbird")
    pub name: String,
    /// Version (e.g., "1.0.0", "2.1.3")
    pub version: String,
    /// Binary path (local or remote)
    pub path: BinaryLocation,
    /// SHA256 checksum for verification
    pub checksum: Option<String>,
    /// Metadata
    pub metadata: PrimalMetadata,
}

/// Binary location
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BinaryLocation {
    /// Local file path (e.g., USB drive)
    Local(PathBuf),
    /// GitHub release
    GitHub {
        /// GitHub organization or user
        org: String,
        /// Repository name
        repo: String,
        /// Release tag (e.g., "v1.0.0")
        tag: String,
        /// Release asset filename
        asset: String,
    },
    /// Custom URL
    Remote(String),
}

/// Primal metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalMetadata {
    /// Description
    pub description: String,
    /// Required capabilities
    pub capabilities: Vec<String>,
    /// Default ports
    pub default_ports: HashMap<String, u16>,
    /// Configuration hints
    pub config_hints: HashMap<String, String>,
}

/// Primal registry - manages available binaries
pub struct PrimalRegistry {
    /// Local binary directory (e.g., ../phase1bins/)
    local_dir: PathBuf,
    /// Available binaries
    binaries: HashMap<String, Vec<PrimalBinary>>,
}

impl PrimalRegistry {
    /// Create a new registry
    pub fn new(local_dir: impl Into<PathBuf>) -> Self {
        Self {
            local_dir: local_dir.into(),
            binaries: HashMap::new(),
        }
    }

    /// Scan local directory for primal binaries
    pub async fn scan_local(&mut self) -> Result<()> {
        if !self.local_dir.exists() {
            tracing::warn!(
                "Local binary directory does not exist: {:?}",
                self.local_dir
            );
            return Ok(());
        }

        // Scan for binaries
        let entries = std::fs::read_dir(&self.local_dir)?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            // Check if it's an executable
            if path.is_file() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    // Try to detect primal type from name
                    let primal_name = self.detect_primal_name(name);
                    let version = self
                        .detect_version(&path)
                        .await
                        .unwrap_or_else(|| "unknown".to_string());

                    let binary = PrimalBinary {
                        name: primal_name.clone(),
                        version,
                        path: BinaryLocation::Local(path.clone()),
                        checksum: self.compute_checksum(&path).await.ok(),
                        metadata: self.default_metadata(&primal_name),
                    };

                    self.binaries.entry(primal_name).or_default().push(binary);
                }
            }
        }

        tracing::info!(
            "Scanned local directory, found {} primal types",
            self.binaries.len()
        );
        Ok(())
    }

    /// Fetch available binaries from GitHub releases
    pub async fn fetch_from_github(&mut self, org: &str, repos: &[&str]) -> Result<()> {
        tracing::info!("Fetching primal binaries from GitHub: {}", org);

        for repo in repos {
            // Future: Implement GitHub API integration using `octocrab` crate
            // Would fetch releases, check latest versions, download binaries
            tracing::info!("Would fetch from: {}/{}", org, repo);
        }

        Ok(())
    }

    /// Get all available versions of a primal
    pub fn get_primal_versions(&self, name: &str) -> Vec<&PrimalBinary> {
        self.binaries
            .get(name)
            .map(|v| v.iter().collect())
            .unwrap_or_default()
    }

    /// Get latest version of a primal
    pub fn get_latest(&self, name: &str) -> Option<&PrimalBinary> {
        self.binaries.get(name).and_then(|versions| {
            versions.iter().max_by(|a, b| {
                // Simple lexicographic version comparison
                // Future: use `semver` crate for proper semantic versioning
                a.version.cmp(&b.version)
            })
        })
    }

    /// Get all available primals
    pub fn list_primals(&self) -> Vec<String> {
        self.binaries.keys().cloned().collect()
    }

    /// Find primals by capability (capability-based discovery!)
    ///
    /// **Deep Debt Principle**: No hardcoded primal names!
    /// Discovers primals based on what they can do, not what they're called.
    pub fn find_by_capability(&self, capability: CapabilityTaxonomy) -> Vec<&PrimalBinary> {
        let capability_str = capability.to_string();
        let mut results = Vec::new();

        for binaries in self.binaries.values() {
            for binary in binaries {
                // Check if this primal provides the requested capability
                if binary
                    .metadata
                    .capabilities
                    .iter()
                    .any(|c| c == &capability_str || self.capability_matches(c, &capability_str))
                {
                    results.push(binary);
                }
            }
        }

        tracing::debug!(
            "Found {} primals providing capability: {:?}",
            results.len(),
            capability
        );

        results
    }

    /// Find primals by multiple capabilities (all must match)
    ///
    /// Useful for finding primals that provide multiple capabilities, e.g.,
    /// "I need both encryption AND federation"
    pub fn find_by_capabilities(&self, capabilities: &[CapabilityTaxonomy]) -> Vec<&PrimalBinary> {
        let mut results = Vec::new();

        for binaries in self.binaries.values() {
            for binary in binaries {
                // Check if this primal provides ALL requested capabilities
                let provides_all = capabilities.iter().all(|cap| {
                    let cap_str = cap.to_string();
                    binary
                        .metadata
                        .capabilities
                        .iter()
                        .any(|c| c == &cap_str || self.capability_matches(c, &cap_str))
                });

                if provides_all {
                    results.push(binary);
                }
            }
        }

        tracing::debug!(
            "Found {} primals providing all capabilities: {:?}",
            results.len(),
            capabilities
        );

        results
    }

    /// Get the best primal for a capability (latest version)
    ///
    /// Returns the primal with the highest version number that provides
    /// the requested capability.
    pub fn get_best_for_capability(&self, capability: CapabilityTaxonomy) -> Option<&PrimalBinary> {
        let mut candidates = self.find_by_capability(capability);

        // Sort by version (descending)
        candidates.sort_by(|a, b| b.version.cmp(&a.version));

        candidates.first().copied()
    }

    /// Helper: Check if a capability string matches (fuzzy match)
    ///
    /// Handles legacy string capabilities that might not exactly match
    /// the CapabilityTaxonomy enum strings.
    fn capability_matches(&self, legacy_cap: &str, new_cap: &str) -> bool {
        let legacy_lower = legacy_cap.to_lowercase();
        let new_lower = new_cap.to_lowercase();

        // Exact match
        if legacy_lower == new_lower {
            return true;
        }

        // Fuzzy matches for common synonyms
        match new_cap {
            "encryption" => legacy_lower.contains("crypto") || legacy_lower.contains("security"),
            "discovery" => legacy_lower.contains("mesh") || legacy_lower.contains("federation"),
            "compute" => legacy_lower.contains("orchestration"),
            "storage" => legacy_lower.contains("data"),
            "ai" => legacy_lower.contains("ml") || legacy_lower.contains("intelligence"),
            _ => false,
        }
    }

    /// Deploy a primal binary to a target
    pub async fn deploy_to_target(
        &self,
        primal_name: &str,
        version: Option<&str>,
        target: &str,
    ) -> Result<PathBuf> {
        let binary = if let Some(v) = version {
            self.binaries
                .get(primal_name)
                .and_then(|versions| versions.iter().find(|b| b.version == v))
                .ok_or_else(|| anyhow::anyhow!("Primal {primal_name} version {v} not found"))?
        } else {
            self.get_latest(primal_name)
                .ok_or_else(|| anyhow::anyhow!("Primal {primal_name} not found"))?
        };

        tracing::info!(
            "Deploying {} v{} to {}",
            primal_name,
            binary.version,
            target
        );

        match &binary.path {
            BinaryLocation::Local(path) => {
                // Copy local binary to target
                tracing::info!("Copying from local: {:?}", path);
                Ok(path.clone())
            }
            BinaryLocation::GitHub {
                org,
                repo,
                tag,
                asset,
            } => {
                // Download from GitHub
                tracing::info!(
                    "Would download from GitHub: {}/{} @ {} ({})",
                    org,
                    repo,
                    tag,
                    asset
                );
                // Future: Implement actual GitHub release asset download using octocrab/reqwest
                Err(anyhow::anyhow!("GitHub download not yet implemented"))
            }
            BinaryLocation::Remote(url) => {
                // Download from URL
                tracing::info!("Would download from: {}", url);
                // Future: Implement HTTP download with progress tracking using reqwest
                Err(anyhow::anyhow!("Remote download not yet implemented"))
            }
        }
    }

    /// Detect primal name from filename
    /// TRUE PRIMAL: No hardcoded list - accept any binary
    fn detect_primal_name(&self, filename: &str) -> String {
        // Remove common suffixes
        let name = filename
            .trim_end_matches(".exe")
            .trim_end_matches("-linux")
            .trim_end_matches("-macos")
            .trim_end_matches("-windows")
            .trim_start_matches("biomeos-")
            .trim_end_matches("-bin")
            .trim_end_matches("-server")
            .trim_end_matches("-cli");

        // TRUE PRIMAL: No hardcoded list of known primals
        // Accept any binary and query it for capabilities at runtime
        name.to_string()
    }

    /// Detect version from binary
    ///
    /// Future: Execute binary with --version flag and parse output
    async fn detect_version(&self, _path: &Path) -> Option<String> {
        // Would execute: `Command::new(path).arg("--version").output()`
        // and parse version string from output
        None
    }

    /// Compute SHA256 checksum
    async fn compute_checksum(&self, path: &Path) -> Result<String> {
        use sha2::{Digest, Sha256};

        let contents = tokio::fs::read(path).await?;
        let hash = Sha256::digest(&contents);
        Ok(format!("{hash:x}"))
    }

    /// Get default metadata for a primal
    /// TRUE PRIMAL: No hardcoded metadata - primals announce their own capabilities
    fn default_metadata(&self, name: &str) -> PrimalMetadata {
        // Return minimal metadata - primal should announce its own capabilities
        // This is only used as a fallback for legacy primals that don't support
        // capability announcement
        PrimalMetadata {
            description: format!("{name} primal (query for capabilities)"),
            capabilities: vec![], // Will be discovered at runtime via JSON-RPC
            default_ports: HashMap::new(), // Will be discovered or configured
            config_hints: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_registry_creation() {
        let registry = PrimalRegistry::new("/tmp/test-bins");
        assert_eq!(registry.list_primals().len(), 0);
    }

    #[test]
    fn test_primal_name_detection() {
        let registry = PrimalRegistry::new("/tmp");
        assert_eq!(registry.detect_primal_name("beardog"), "beardog");
        assert_eq!(registry.detect_primal_name("beardog-linux"), "beardog");
        assert_eq!(registry.detect_primal_name("songbird.exe"), "songbird");
    }

    #[test]
    fn test_find_by_capability() {
        let mut registry = PrimalRegistry::new("/tmp");

        // Add test binaries
        let beardog = PrimalBinary {
            name: "beardog".to_string(),
            version: "1.0.0".to_string(),
            path: BinaryLocation::Local(PathBuf::from("/tmp/beardog")),
            checksum: None,
            metadata: PrimalMetadata {
                description: "Security primal".to_string(),
                capabilities: vec!["encryption".to_string(), "crypto".to_string()],
                default_ports: HashMap::new(),
                config_hints: HashMap::new(),
            },
        };

        let songbird = PrimalBinary {
            name: "songbird".to_string(),
            version: "1.0.0".to_string(),
            path: BinaryLocation::Local(PathBuf::from("/tmp/songbird")),
            checksum: None,
            metadata: PrimalMetadata {
                description: "Discovery primal".to_string(),
                capabilities: vec!["discovery".to_string(), "federation".to_string()],
                default_ports: HashMap::new(),
                config_hints: HashMap::new(),
            },
        };

        registry
            .binaries
            .insert("beardog".to_string(), vec![beardog]);
        registry
            .binaries
            .insert("songbird".to_string(), vec![songbird]);

        // Test finding by capability
        let encryption_primals = registry.find_by_capability(CapabilityTaxonomy::Encryption);
        assert_eq!(encryption_primals.len(), 1);
        assert_eq!(encryption_primals[0].name, "beardog");

        let discovery_primals = registry.find_by_capability(CapabilityTaxonomy::Discovery);
        assert_eq!(discovery_primals.len(), 1);
        assert_eq!(discovery_primals[0].name, "songbird");

        // Test finding non-existent capability
        let compute_primals = registry.find_by_capability(CapabilityTaxonomy::WorkloadExecution);
        assert_eq!(compute_primals.len(), 0);
    }

    #[test]
    fn test_find_by_multiple_capabilities() {
        let mut registry = PrimalRegistry::new("/tmp");

        // Add a primal that provides multiple capabilities
        let multi_cap = PrimalBinary {
            name: "multi".to_string(),
            version: "1.0.0".to_string(),
            path: BinaryLocation::Local(PathBuf::from("/tmp/multi")),
            checksum: None,
            metadata: PrimalMetadata {
                description: "Multi-capability primal".to_string(),
                capabilities: vec![
                    "encryption".to_string(),
                    "discovery".to_string(),
                    "compute".to_string(),
                ],
                default_ports: HashMap::new(),
                config_hints: HashMap::new(),
            },
        };

        registry
            .binaries
            .insert("multi".to_string(), vec![multi_cap]);

        // Test finding by multiple capabilities (all match)
        let primals = registry.find_by_capabilities(&[
            CapabilityTaxonomy::Encryption,
            CapabilityTaxonomy::Discovery,
        ]);
        assert_eq!(primals.len(), 1);
        assert_eq!(primals[0].name, "multi");

        // Test finding by multiple capabilities (not all match)
        let primals = registry.find_by_capabilities(&[
            CapabilityTaxonomy::Encryption,
            CapabilityTaxonomy::DataStorage, // multi doesn't have storage
        ]);
        assert_eq!(primals.len(), 0);
    }

    #[test]
    fn test_get_best_for_capability() {
        let mut registry = PrimalRegistry::new("/tmp");

        // Add multiple versions of the same primal
        let v1 = PrimalBinary {
            name: "beardog".to_string(),
            version: "1.0.0".to_string(),
            path: BinaryLocation::Local(PathBuf::from("/tmp/beardog-1.0.0")),
            checksum: None,
            metadata: PrimalMetadata {
                description: "Security primal".to_string(),
                capabilities: vec!["encryption".to_string()],
                default_ports: HashMap::new(),
                config_hints: HashMap::new(),
            },
        };

        let v2 = PrimalBinary {
            name: "beardog".to_string(),
            version: "2.0.0".to_string(),
            path: BinaryLocation::Local(PathBuf::from("/tmp/beardog-2.0.0")),
            checksum: None,
            metadata: PrimalMetadata {
                description: "Security primal".to_string(),
                capabilities: vec!["encryption".to_string()],
                default_ports: HashMap::new(),
                config_hints: HashMap::new(),
            },
        };

        registry
            .binaries
            .insert("beardog".to_string(), vec![v1, v2]);

        // Get best (should return v2.0.0)
        let best = registry.get_best_for_capability(CapabilityTaxonomy::Encryption);
        assert!(best.is_some());
        assert_eq!(best.unwrap().version, "2.0.0");
    }

    #[test]
    fn test_capability_fuzzy_matching() {
        let registry = PrimalRegistry::new("/tmp");

        // Test encryption/crypto match
        assert!(registry.capability_matches("crypto", "encryption"));
        assert!(registry.capability_matches("security", "encryption"));

        // Test discovery/federation match
        assert!(registry.capability_matches("mesh", "discovery"));
        assert!(registry.capability_matches("federation", "discovery"));

        // Test compute/orchestration match
        assert!(registry.capability_matches("orchestration", "compute"));

        // Test non-matches
        assert!(!registry.capability_matches("storage", "encryption"));
        assert!(!registry.capability_matches("random", "discovery"));
    }
}
