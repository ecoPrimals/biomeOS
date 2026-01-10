// SPDX-License-Identifier: APGL-3.0-or-later WITH Sovran-Exemption-1.0
//
// Copyright 2025 ecoPrimals Project
// Licensed under the Affero General Public License v3.0 or later with Sovran Exemption 1.0.
// See LICENSE file in the project root or visit https://www.gnu.org/licenses/agpl-3.0.html

//! Configuration generation for spores
//!
//! Handles tower.toml configuration file creation.

use tokio::fs as async_fs;
use tracing::{debug, info};

use crate::error::SporeResult;
use super::core::Spore;

/// Trait for configuration operations on spores
pub(super) trait ConfigOps {
    /// Create tower.toml configuration
    fn create_tower_config(&self) -> impl std::future::Future<Output = SporeResult<()>> + Send;
    
    /// Generate tower.toml content
    fn generate_tower_toml(&self) -> String;
}

impl ConfigOps for Spore {
    /// Create tower.toml configuration
    ///
    /// **Note**: Uses `BEARDOG_FAMILY_SEED_FILE` to reference the seed file.
    /// BearDog will read and process the seed at runtime.
    async fn create_tower_config(&self) -> SporeResult<()> {
        info!("Creating tower.toml configuration");

        let config = self.generate_tower_toml();
        let config_path = self.root_path.join("tower.toml");

        async_fs::write(&config_path, config).await?;
        debug!("Wrote tower.toml to: {}", config_path.display());

        Ok(())
    }

    /// Generate tower.toml content
    fn generate_tower_toml(&self) -> String {
        format!(
            r#"# BiomeOS Tower Configuration v0.4.0
# Generated spore: {}
# Port-Free Architecture - Unix Sockets + UDP Multicast
# Secure Genetic Lineage - File-based seed (not exposed in config)

[tower]
family = "nat0"
concurrent_startup = true

# BearDog v0.15.0 - Security Primal (Port-Free!)
[[primals]]
binary = "./primals/beardog-server"
provides = ["Security", "Encryption", "Trust"]
requires = []

[primals.env]
# ✅ SECURE: File-based seed (BearDog v0.15.0 reads the file)
BEARDOG_FAMILY_SEED_FILE = "./.family.seed"
BEARDOG_FAMILY_ID = "nat0"
BEARDOG_NODE_ID = "{node_id}"
RUST_LOG = "info"

# Songbird v3.19.0 - Discovery Orchestrator (UDP Multicast + BTSP)
[[primals]]
binary = "./primals/songbird"
provides = ["Discovery"]
requires = ["Security"]

[primals.env]
SONGBIRD_FAMILY_ID = "nat0"
SONGBIRD_NODE_ID = "{node_id}"
SONGBIRD_TAGS = "btsp_enabled"
# Protocol-aware endpoint URLs:
#   - "unix://..." = Auto-detect (server determines protocol)
SECURITY_ENDPOINT = "unix:///tmp/beardog-nat0-{node_id}.sock"
SONGBIRD_SECURITY_PROVIDER = "unix:///tmp/beardog-nat0-{node_id}.sock"
RUST_LOG = "info"
"#,
            self.config.label,
            node_id = self.config.node_id,
        )
    }
}


