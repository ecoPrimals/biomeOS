// SPDX-License-Identifier: APGL-3.0-or-later WITH Sovran-Exemption-1.0
//
// Copyright 2025 ecoPrimals Project
// Licensed under the Affero General Public License v3.0 or later with Sovran Exemption 1.0.
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
use super::types::default_family_id;
use super::deployment::DeploymentOps;
use super::documentation::DocumentationOps;
use super::filesystem::FilesystemOps;
use super::genetics::GeneticsOps;
use super::types::SporeConfig;

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
    /// * `config` - Spore configuration (label, node_id)
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
    pub fn from_path(mount_point: PathBuf) -> SporeResult<Self> {
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
    /// - Has a **unique node_id** (e.g., "tower2" vs "tower1")
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
            label: format!("biomeOS-{}", sibling_node_id),
            node_id: sibling_node_id.to_string(),
            family_id: self.config.family_id.clone(),
            spore_type: self.config.spore_type,
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
    pub fn root_path(&self) -> &Path {
        &self.root_path
    }

    /// Get the spore configuration
    pub fn config(&self) -> &SporeConfig {
        &self.config
    }

    /// Extract node_id from tower.toml (simple parsing)
    fn extract_node_id_from_config(config_str: &str) -> Option<String> {
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

// Note: Trait implementations are in their respective domain modules
