// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Spore incubation - Deploy spores on local computers with local entropy mixing
//!
//! This module enables USB spores to be deployed on multiple computers while maintaining
//! genetic lineage for federation. Each deployment mixes the spore's genetic seed with
//! local computer entropy to create a unique deployed node identity.
//!
//! # Architecture
//!
//! ```text
//! USB Spore (Genetic Seed)
//!   ├─> Computer A → deployed_seed = SHA256(spore_seed || entropy_A)
//!   └─> Computer B → deployed_seed = SHA256(spore_seed || entropy_B)
//!
//! Both nodes:
//!   - Share genetic lineage (can federate)
//!   - Have unique local identity
//!   - Recognize each other as siblings
//! ```

mod local_entropy;
mod node_config;
mod tower_metadata;

pub use local_entropy::LocalEntropy;
pub use node_config::{
    CreateLocalConfigParams, FederationInfo, IncubatedNode, LineageInfo, NodeConfig, NodeInfo,
    SporeInfo,
};
pub use tower_metadata::{extract_family_id, extract_spore_id};

use anyhow::Context;
use bytes::Bytes;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;
use tracing::{debug, info};

use chrono::Utc;

use crate::error::SporeResult;
use crate::seed::FamilySeed;
use crate::spore_log_tracker::SporeLogTracker;

/// Spore incubator - Handles deployment of spores on local computers
pub struct SporeIncubator {
    spore_path: PathBuf,
    _spore_seed: FamilySeed,
}

impl SporeIncubator {
    /// Create a new incubator for a spore
    pub fn new(spore_path: impl AsRef<Path>) -> SporeResult<Self> {
        let spore_path = spore_path.as_ref().to_path_buf();

        info!("Creating incubator for spore at: {}", spore_path.display());

        // Load spore seed
        let seed_path = spore_path.join(".family.seed");
        let spore_seed = FamilySeed::from_file(&seed_path)?;

        Ok(Self {
            spore_path,
            _spore_seed: spore_seed,
        })
    }

    /// Incubate this spore on the local computer
    ///
    /// This creates a deployed node by mixing the spore seed with local entropy.
    /// The result is stored in ~/.config/biomeos/deployed-nodes/{spore-id}/
    ///
    /// # Arguments
    ///
    /// * `computer_name` - Optional name for this computer (uses hostname if None)
    /// * `deploy_local` - If true, also create local deployment in /tmp
    pub async fn incubate(
        &self,
        computer_name: Option<&str>,
        deploy_local: bool,
    ) -> SporeResult<IncubatedNode> {
        self.incubate_with_home(computer_name, deploy_local, None)
            .await
    }

    /// Like [`Self::incubate`], but uses `home` as the user's home directory for config paths
    /// instead of `HOME` / `USERPROFILE`.
    pub async fn incubate_with_home(
        &self,
        computer_name: Option<&str>,
        _deploy_local: bool,
        home: Option<&Path>,
    ) -> SporeResult<IncubatedNode> {
        info!("Incubating spore on local computer");

        // 1. Generate local entropy
        let local_entropy = LocalEntropy::generate(computer_name)?;
        let entropy_hash = local_entropy.hash();

        debug!("Generated local entropy hash: {}", entropy_hash);

        // 2. Derive deployed node seed
        let deployed_seed = self.derive_deployed_seed(&local_entropy)?;
        let deployed_seed_hash = Self::hash_seed(&deployed_seed);

        // 3. Determine spore ID and node ID (hostname in Arc for cheap clone across async)
        let spore_id = extract_spore_id(&self.spore_path)?;
        let hostname: Arc<str> = Arc::from(local_entropy.hostname.as_str());
        let node_id = format!("node-{}-{}", spore_id, hostname.as_ref());

        info!("Creating incubated node: {}", node_id);

        // 4. Create local configuration
        let home = if let Some(h) = home {
            h.to_path_buf()
        } else {
            std::env::var(biomeos_types::env_config::vars::HOME)
                .or_else(|_| std::env::var("USERPROFILE"))
                .map(PathBuf::from)
                .context("Could not determine home directory")?
        };
        let local_config_path = Self::get_local_config_path(&spore_id, &home)?;
        self.create_local_config(CreateLocalConfigParams {
            config_path: &local_config_path,
            spore_id: &spore_id,
            node_id: &node_id,
            deployed_seed_hash: &deployed_seed_hash,
            entropy_hash: &entropy_hash,
            computer_name: hostname.as_ref(),
            local_entropy: &local_entropy,
        })
        .await?;

        // 5. Store deployed seed securely
        self.store_deployed_seed(&local_config_path, &deployed_seed)
            .await?;

        // 6. Log incubation to spore
        let log_tracker = SporeLogTracker::new(&self.spore_path)?;
        log_tracker.initialize().await?;

        let mut metadata = std::collections::HashMap::new();
        metadata.insert("computer_name".to_string(), hostname.as_ref().to_string());
        metadata.insert("entropy_hash".to_string(), entropy_hash.clone());
        metadata.insert("node_id".to_string(), node_id.clone());

        log_tracker
            .record_event(crate::spore_log_tracker::SporeLifecycleEvent {
                timestamp: Utc::now(),
                event_type: crate::spore_log_tracker::SporeEventType::Custom(
                    "incubation".to_string(),
                ),
                node_id: Some(node_id.clone()),
                deployed_to: Some(hostname.as_ref().to_string()),
                metadata,
            })
            .await?;

        info!("✅ Spore incubated successfully as node: {}", node_id);

        Ok(IncubatedNode {
            node_id,
            spore_id,
            deployed_seed_hash,
            local_config_path,
            incubated_at: Utc::now(),
            entropy_hash,
            spore_path: Some(self.spore_path.clone()),
        })
    }

    /// Derive deployed node seed from spore seed + local entropy
    ///
    /// Formula: `deployed_seed` = `SHA256(spore_seed` || `local_entropy_hash`)
    ///
    /// This ensures:
    /// - Each deployment is unique
    /// - Same spore on different computers = different seeds
    /// - Deterministic (same computer + same spore = same seed)
    fn derive_deployed_seed(&self, local_entropy: &LocalEntropy) -> SporeResult<Bytes> {
        let spore_seed_bytes = std::fs::read(self.spore_path.join(".family.seed"))?;
        let entropy_hash = local_entropy.hash();

        let mut hasher = Sha256::new();
        hasher.update(&spore_seed_bytes);
        hasher.update(entropy_hash.as_bytes());

        Ok(Bytes::copy_from_slice(&hasher.finalize()))
    }

    /// Hash a seed for display/storage
    fn hash_seed(seed: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(seed);
        format!("{:x}", hasher.finalize())
    }

    /// Get local configuration path for this spore under the given home directory.
    fn get_local_config_path(spore_id: &str, home: &Path) -> SporeResult<PathBuf> {
        Ok(home
            .join(".config")
            .join(biomeos_types::constants::runtime_paths::BIOMEOS_SUBDIR)
            .join("deployed-nodes")
            .join(spore_id))
    }

    /// Create local configuration for incubated node
    async fn create_local_config(&self, params: CreateLocalConfigParams<'_>) -> SporeResult<()> {
        // Create directory
        fs::create_dir_all(params.config_path).await?;

        // Read parent and spore seed hashes
        let spore_seed_bytes = std::fs::read(self.spore_path.join(".family.seed"))?;
        let spore_seed_hash = Self::hash_seed(&spore_seed_bytes);

        // Parent seed lineage is tracked in the spore manifest when available;
        // falls back to self-reference for the root of the lineage chain.
        let parent_seed_hash = spore_seed_hash.clone();

        // Extract family_id from tower.toml
        let family_id =
            extract_family_id(&self.spore_path).unwrap_or_else(|_| "unknown".to_string());

        // Create node config
        let config = node_config::NodeConfig {
            node: node_config::NodeInfo {
                spore_id: params.spore_id.to_string(),
                node_id: params.node_id.to_string(),
                deployed_at: Utc::now(),
                computer_name: params.computer_name.to_string(),
                entropy_hash: params.entropy_hash.to_string(),
            },
            lineage: node_config::LineageInfo {
                parent_seed_hash,
                spore_seed_hash,
                deployed_seed_hash: params.deployed_seed_hash.to_string(),
            },
            spore: node_config::SporeInfo {
                original_path: Some(self.spore_path.clone()),
                last_seen: Utc::now(),
                deployment_count: 1,
            },
            federation: node_config::FederationInfo {
                family_id,
                sub_federations: vec![],
            },
        };

        // Write node.toml
        let config_toml =
            toml::to_string_pretty(&config).context("Failed to serialize node config")?;
        fs::write(params.config_path.join("node.toml"), config_toml).await?;

        // Write entropy.json for reference
        let entropy_json = serde_json::to_string_pretty(params.local_entropy)
            .context("Failed to serialize entropy")?;
        fs::write(params.config_path.join("entropy.json"), entropy_json).await?;

        info!("Created local config at: {}", params.config_path.display());

        Ok(())
    }

    /// Store deployed seed securely
    async fn store_deployed_seed(
        &self,
        config_path: &Path,
        deployed_seed: &[u8],
    ) -> SporeResult<()> {
        let seed_path = config_path.join(".deployed.seed");
        fs::write(&seed_path, deployed_seed).await?;

        // Set secure permissions (Unix only)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&seed_path).await?.permissions();
            perms.set_mode(0o600); // Read/write for owner only
            fs::set_permissions(&seed_path, perms).await?;
        }

        debug!("Stored deployed seed securely");
        Ok(())
    }
}

/// List all locally incubated nodes under `home` (typically `$HOME`).
pub async fn list_local_nodes_in(home: &Path) -> SporeResult<Vec<NodeConfig>> {
    let nodes_dir = home.join(".config").join(biomeos_types::constants::runtime_paths::BIOMEOS_SUBDIR).join("deployed-nodes");

    if !nodes_dir.exists() {
        return Ok(vec![]);
    }

    let mut nodes = Vec::new();
    let mut entries = fs::read_dir(&nodes_dir).await?;

    while let Some(entry) = entries.next_entry().await? {
        let node_config_path = entry.path().join("node.toml");

        if node_config_path.exists()
            && let Ok(content) = fs::read_to_string(&node_config_path).await
            && let Ok(config) = toml::from_str::<NodeConfig>(&content)
        {
            nodes.push(config);
        }
    }

    Ok(nodes)
}

/// List all locally incubated nodes (uses `HOME` / `USERPROFILE`).
pub async fn list_local_nodes() -> SporeResult<Vec<NodeConfig>> {
    let home = std::env::var(biomeos_types::env_config::vars::HOME)
        .or_else(|_| std::env::var("USERPROFILE"))
        .context("Could not determine home directory")?;
    list_local_nodes_in(Path::new(&home)).await
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
