// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Copyright 2025-2026 ecoPrimals Project
// Licensed under the Affero General Public License v3.0.
// See LICENSE file in the project root or visit https://www.gnu.org/licenses/agpl-3.0.html

//! Configuration generation for spores
//!
//! Handles tower.toml configuration file creation.

use tokio::fs as async_fs;
use tracing::{debug, info};

use super::core::Spore;
use crate::error::SporeResult;

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
    /// `BearDog` will read and process the seed at runtime.
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
family = "test_family"
concurrent_startup = true

# BearDog v0.15.0 - Security Primal (Port-Free!)
[[primals]]
binary = "./primals/beardog-server"
provides = ["Security", "Encryption", "Trust"]
requires = []

[primals.env]
# ✅ SECURE: File-based seed (BearDog v0.15.0 reads the file)
BEARDOG_FAMILY_SEED_FILE = "./.family.seed"
BEARDOG_FAMILY_ID = "test_family"
BEARDOG_NODE_ID = "{node_id}"
RUST_LOG = "info"

# Songbird v3.19.0 - Discovery Orchestrator (UDP Multicast + BTSP)
[[primals]]
binary = "./primals/songbird"
provides = ["Discovery"]
requires = ["Security"]

[primals.env]
SONGBIRD_FAMILY_ID = "test_family"
SONGBIRD_NODE_ID = "{node_id}"
SONGBIRD_TAGS = "btsp_enabled"
# Protocol-aware endpoint URLs:
#   - "unix://..." = Auto-detect (server determines protocol)
SECURITY_ENDPOINT = "unix:///tmp/beardog-{family_id}-{node_id}.sock"
SONGBIRD_SECURITY_PROVIDER = "unix:///tmp/beardog-{family_id}-{node_id}.sock"
RUST_LOG = "info"
"#,
            self.config.label,
            node_id = self.config.node_id,
            family_id = self.config.family_id,
        )
    }
}

#[cfg(test)]
mod config_tests {
    use super::ConfigOps;
    use crate::SporeConfig;
    use crate::spore::core::spore_for_tests;
    use crate::spore_types::SporeType;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_create_tower_config_writes_tower_toml() {
        let dir = tempfile::tempdir().expect("tempdir");
        let spore = spore_for_tests(
            dir.path().to_path_buf(),
            SporeConfig {
                label: "cfg-test".to_string(),
                node_id: "node-cfg".to_string(),
                family_id: "family-cfg".to_string(),
                spore_type: SporeType::Cold,
                plasmid_bin_dir: None,
            },
        );
        spore.create_tower_config().await.expect("write tower.toml");
        let path = dir.path().join("tower.toml");
        assert!(path.exists(), "tower.toml should exist");
        let content = std::fs::read_to_string(&path).expect("read");
        assert!(content.contains("node-cfg"));
        assert!(content.contains("family-cfg"));
    }

    #[test]
    fn test_generate_tower_toml_substitutes_label_node_family() {
        let spore = spore_for_tests(
            PathBuf::from("/tmp/spore-test-root"),
            SporeConfig {
                label: "spore-lbl".to_string(),
                node_id: "nid-99".to_string(),
                family_id: "fam-88".to_string(),
                spore_type: SporeType::Live,
                plasmid_bin_dir: None,
            },
        );
        let s = spore.generate_tower_toml();
        assert!(s.contains("spore-lbl"), "{s}");
        assert!(s.contains("nid-99"), "{s}");
        assert!(s.contains("fam-88"), "{s}");
        assert!(s.contains("BEARDOG_NODE_ID = \"nid-99\""), "{s}");
    }
}
