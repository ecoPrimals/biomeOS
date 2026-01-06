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
        org: String,
        repo: String,
        tag: String,
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
                .ok_or_else(|| anyhow::anyhow!("Primal {} version {} not found", primal_name, v))?
        } else {
            self.get_latest(primal_name)
                .ok_or_else(|| anyhow::anyhow!("Primal {} not found", primal_name))?
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
    fn detect_primal_name(&self, filename: &str) -> String {
        // Remove common suffixes
        let name = filename
            .trim_end_matches(".exe")
            .trim_end_matches("-linux")
            .trim_end_matches("-macos")
            .trim_end_matches("-windows");

        // Known primal names
        let known = ["beardog", "songbird", "toadstool", "nestgate", "squirrel"];
        for primal in known {
            if name.to_lowercase().contains(primal) {
                return primal.to_string();
            }
        }

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
        Ok(format!("{:x}", hash))
    }

    /// Get default metadata for a primal
    fn default_metadata(&self, name: &str) -> PrimalMetadata {
        match name {
            "beardog" => PrimalMetadata {
                description: "Cryptography & Security primal".to_string(),
                capabilities: vec![
                    "crypto".to_string(),
                    "security".to_string(),
                    "btsp".to_string(),
                ],
                default_ports: [("api".to_string(), 9000)].into(),
                config_hints: HashMap::new(),
            },
            "songbird" => PrimalMetadata {
                description: "Service Mesh & Federation primal".to_string(),
                capabilities: vec![
                    "discovery".to_string(),
                    "federation".to_string(),
                    "mesh".to_string(),
                ],
                default_ports: [("api".to_string(), 8000)].into(),
                config_hints: HashMap::new(),
            },
            "toadstool" => PrimalMetadata {
                description: "Compute & Orchestration primal".to_string(),
                capabilities: vec!["compute".to_string(), "orchestration".to_string()],
                default_ports: [("api".to_string(), 7000)].into(),
                config_hints: HashMap::new(),
            },
            "nestgate" => PrimalMetadata {
                description: "Storage & Data primal".to_string(),
                capabilities: vec!["storage".to_string(), "data".to_string()],
                default_ports: [("api".to_string(), 6000)].into(),
                config_hints: HashMap::new(),
            },
            "squirrel" => PrimalMetadata {
                description: "AI & Intelligence primal".to_string(),
                capabilities: vec!["ai".to_string(), "ml".to_string()],
                default_ports: [("api".to_string(), 5000)].into(),
                config_hints: HashMap::new(),
            },
            _ => PrimalMetadata {
                description: format!("Custom primal: {}", name),
                capabilities: vec![],
                default_ports: HashMap::new(),
                config_hints: HashMap::new(),
            },
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
}
