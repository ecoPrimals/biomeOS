// SPDX-License-Identifier: AGPL-3.0-only
//
// Copyright 2025-2026 ecoPrimals Project
// Licensed under the Affero General Public License v3.0.
// See LICENSE file in the project root or visit https://www.gnu.org/licenses/agpl-3.0.html

//! Core Spore structure and lifecycle management
//!
//! Handles the main Spore struct and lifecycle operations:
//! - Creating new spores
//! - Loading existing spores
//! - Cloning sibling spores (with genetic variation!)

use std::fs;
use std::path::{Path, PathBuf};
use tracing::{debug, info};

use crate::error::{SporeError, SporeResult};
use crate::spore_types::SporeType;

use super::config::ConfigOps;
use super::deployment::DeploymentOps;
use super::documentation::DocumentationOps;
use super::filesystem::FilesystemOps;
use super::genetics::GeneticsOps;
use super::types::SporeConfig;
use super::types::default_family_id;

/// USB Spore - A self-contained biomeOS deployment
///
/// A spore contains everything needed to boot a biomeOS tower:
/// - Family seed (`.family.seed`)
/// - Primal binaries (beardog, songbird, etc.)
/// - Tower orchestrator binary
/// - Configuration (`tower.toml`)
/// - Directory structure
///
/// # Biology-Inspired Design
///
/// - **Cold Spores**: Genetic preservation (storage/archival)
/// - **Live Spores**: Self-contained, bootable, ready to germinate
/// - **Siblings**: NOT perfect clones, unique genetic variation
#[derive(Debug)]
pub struct Spore {
    pub(super) root_path: PathBuf,
    pub(super) config: SporeConfig,
}

impl Spore {
    /// Create a new spore on a USB device
    ///
    /// # Steps
    ///
    /// 1. Create directory structure
    /// 2. Generate family seed file
    /// 3. Create `tower.toml` configuration
    /// 4. Copy primal binaries
    /// 5. Copy tower orchestrator
    ///
    /// # Arguments
    ///
    /// * `mount_point` - Where the USB is mounted (e.g., `/media/usb`)
    /// * `config` - Spore configuration (label, `node_id`)
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use biomeos_spore::spore::{Spore, SporeConfig};
    /// use biomeos_spore::spore_types::SporeType;
    /// use std::path::PathBuf;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = SporeConfig {
    ///     label: "biomeOS1".to_string(),
    ///     node_id: "tower1".to_string(),
    ///     spore_type: SporeType::Live,
    ///     family_id: "1894e909e454".to_string(),
    ///     plasmid_bin_dir: None,
    /// };
    ///
    /// let spore = Spore::create(
    ///     PathBuf::from("/media/usb"),
    ///     config,
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create(mount_point: PathBuf, config: SporeConfig) -> SporeResult<Self> {
        let root_path = mount_point.join("biomeOS");
        info!(
            "Creating {} '{}' at: {}",
            config.spore_type,
            config.label,
            root_path.display()
        );

        let spore = Self { root_path, config };

        // Execute creation steps (like cell division)
        spore.create_directory_structure().await?;
        spore.generate_seed_file().await?;
        spore.create_tower_config().await?;
        spore.copy_binaries().await?;

        // Only create deployment script for LiveSpores
        if spore.config.spore_type.requires_execution_env() {
            spore.create_deployment_script().await?;
        }

        spore.create_readme().await?;
        spore.create_spore_manifest().await?;

        info!(
            "{} {} creation complete: {}",
            spore.config.spore_type.emoji(),
            spore.config.spore_type,
            spore.root_path.display()
        );
        match spore.config.spore_type {
            SporeType::Cold => {
                info!("   Genetic material preserved for storage/archival");
            }
            SporeType::Live => {
                info!("   Self-contained, bootable, genetically complete!");
            }
        }
        Ok(spore)
    }

    /// Load existing spore from USB
    pub fn from_path(mount_point: &Path) -> SporeResult<Self> {
        let root_path = mount_point.join("biomeOS");

        if !root_path.exists() {
            return Err(SporeError::DeviceNotFound(root_path));
        }

        // Read config from tower.toml to extract node_id
        let config_path = root_path.join("tower.toml");
        let config_str = fs::read_to_string(&config_path)?;

        // Parse to extract node_id (simplified)
        let node_id =
            Self::extract_node_id_from_config(&config_str).unwrap_or_else(|| "unknown".to_string());

        let config = SporeConfig {
            label: root_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string(),
            family_id: default_family_id(),
            node_id,
            spore_type: SporeType::default(), // Detect from manifest if available
            plasmid_bin_dir: None,
        };

        Ok(Self { root_path, config })
    }

    /// Clone this spore to a new USB with genetic variation
    ///
    /// # Biology-Inspired Cloning
    ///
    /// Unlike artificial systems that create perfect clones, biomeOS follows
    /// real biology: siblings are NOT identical! Each sibling:
    /// - Has the same **family lineage** (same `.family.seed`)
    /// - Has a **unique `node_id`** (e.g., "tower2" vs "tower1")
    /// - Represents a **deployment batch** (siblings from the same source)
    ///
    /// This reflects real biology: siblings are NOT perfect clones!
    pub async fn clone_sibling(
        &self,
        target_mount: PathBuf,
        sibling_node_id: &str,
    ) -> SporeResult<Self> {
        info!(
            "Cloning sibling spore: {} -> {}",
            self.config.node_id, sibling_node_id
        );

        // Create sibling config (same family, different node_id)
        let sibling_config = SporeConfig {
            label: format!("biomeOS-{sibling_node_id}"),
            node_id: sibling_node_id.to_string(),
            family_id: self.config.family_id.clone(),
            spore_type: self.config.spore_type,
            plasmid_bin_dir: self.config.plasmid_bin_dir.clone(),
        };

        // Create new spore with same genetic seed
        let sibling = Self::create(target_mount, sibling_config).await?;

        // Copy the original family seed (shared lineage)
        let source_seed = self.root_path.join(".family.seed");
        let target_seed = sibling.root_path.join(".family.seed");

        if source_seed.exists() {
            tokio::fs::copy(&source_seed, &target_seed).await?;
            debug!(
                "Copied family seed: {} -> {}",
                source_seed.display(),
                target_seed.display()
            );
        }

        info!(
            "Sibling spore created: {} (shares genetic lineage with {})",
            sibling_node_id, self.config.node_id
        );

        Ok(sibling)
    }

    /// Get the root path of this spore
    #[must_use]
    pub fn root_path(&self) -> &Path {
        &self.root_path
    }

    /// Get the spore configuration
    #[must_use]
    pub const fn config(&self) -> &SporeConfig {
        &self.config
    }

    /// Extract `node_id` from tower.toml (simple parsing)
    pub(crate) fn extract_node_id_from_config(config_str: &str) -> Option<String> {
        for line in config_str.lines() {
            if line.trim().starts_with("node_id") {
                // Extract value after '=' and trim quotes
                if let Some(value) = line.split('=').nth(1) {
                    return Some(
                        value
                            .trim()
                            .trim_matches('"')
                            .trim_matches('\'')
                            .to_string(),
                    );
                }
            }
        }
        None
    }
}

/// Test-only constructor for [`Spore`] (crate tests for [`super::config::ConfigOps`]).
#[cfg(test)]
pub(crate) fn spore_for_tests(root_path: PathBuf, config: SporeConfig) -> Spore {
    Spore { root_path, config }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spore_types::SporeType;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_extract_node_id_from_config() {
        let config = r#"
node_id = "tower1"
other = "value"
"#;
        let node_id = Spore::extract_node_id_from_config(config);
        assert_eq!(node_id, Some("tower1".to_string()));
    }

    #[test]
    fn test_extract_node_id_with_quotes() {
        let config = r#"
node_id = "tower-2"
"#;
        let node_id = Spore::extract_node_id_from_config(config);
        assert_eq!(node_id, Some("tower-2".to_string()));
    }

    #[test]
    fn test_extract_node_id_single_quotes() {
        let config = "node_id = 'tower3'";
        let node_id = Spore::extract_node_id_from_config(config);
        assert_eq!(node_id, Some("tower3".to_string()));
    }

    #[test]
    fn test_extract_node_id_missing() {
        let config = "other_key = value";
        let node_id = Spore::extract_node_id_from_config(config);
        assert!(node_id.is_none());
    }

    #[test]
    fn test_extract_node_id_empty_config() {
        let node_id = Spore::extract_node_id_from_config("");
        assert!(node_id.is_none());
    }

    #[test]
    fn test_spore_from_path_nonexistent() {
        let result = Spore::from_path(Path::new("/nonexistent/path"));
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("not found") || err.to_string().contains("Device"));
    }

    #[test]
    fn test_spore_from_path_valid() {
        let temp = TempDir::new().unwrap();
        let biomeos_dir = temp.path().join("biomeOS");
        std::fs::create_dir_all(&biomeos_dir).unwrap();
        let mut f = std::fs::File::create(biomeos_dir.join("tower.toml")).unwrap();
        f.write_all(b"node_id = \"tower1\"\n").unwrap();

        let result = Spore::from_path(temp.path());
        assert!(result.is_ok());
        let spore = result.unwrap();
        assert_eq!(spore.config().node_id, "tower1");
    }

    #[test]
    fn test_spore_config_construction() {
        let config = SporeConfig {
            label: "test".to_string(),
            node_id: "node1".to_string(),
            family_id: "abc123".to_string(),
            spore_type: SporeType::Live,
            plasmid_bin_dir: None,
        };
        assert_eq!(config.label, "test");
        assert_eq!(config.node_id, "node1");
        assert_eq!(config.family_id, "abc123");
    }

    #[test]
    fn test_default_family_id() {
        biomeos_test_utils::remove_test_env("FAMILY_ID");
        let id = crate::spore::types::default_family_id();
        assert!(!id.is_empty());
    }
}
