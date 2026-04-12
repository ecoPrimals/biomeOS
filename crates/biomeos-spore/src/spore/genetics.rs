// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Copyright 2025-2026 ecoPrimals Project
// Licensed under the Affero General Public License v3.0.
// See LICENSE file in the project root or visit https://www.gnu.org/licenses/agpl-3.0.html

//! Genetic seed generation for spores
//!
//! Handles family seed generation using `BearDog` cryptographic principles.

use tracing::{debug, info};

use super::core::Spore;
use crate::error::{SporeError, SporeResult};
use crate::seed::FamilySeed;

/// Trait for genetic seed operations on spores
pub(super) trait GeneticsOps {
    /// Generate family seed file
    fn generate_seed_file(&self) -> impl std::future::Future<Output = SporeResult<()>> + Send;
}

impl GeneticsOps for Spore {
    /// Generate family seed file
    async fn generate_seed_file(&self) -> SporeResult<()> {
        info!("Generating family seed");

        let seed_path = self.root_path.join(".family.seed");

        // Use tokio::task::spawn_blocking for sync operation
        let seed_path_clone = seed_path.clone();
        tokio::task::spawn_blocking(move || FamilySeed::generate_and_write(&seed_path_clone))
            .await
            .map_err(|e| SporeError::InvalidConfig(format!("Task join error: {e}")))??;

        debug!("Family seed generated at: {}", seed_path.display());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #![expect(clippy::unwrap_used, reason = "test assertions")]

    use super::*;
    use crate::spore_types::SporeType;
    use tempfile::TempDir;

    #[tokio::test]
    async fn generate_seed_file_creates_family_seed() {
        let temp = TempDir::new().unwrap();
        let root_path = temp.path().to_path_buf();
        let config = super::super::types::SporeConfig {
            label: "test".to_string(),
            node_id: "tower1".to_string(),
            family_id: "default".to_string(),
            spore_type: SporeType::Live,
            plasmid_bin_dir: None,
        };
        let spore = Spore {
            root_path: root_path.clone(),
            config,
        };

        spore.generate_seed_file().await.unwrap();

        let seed_path = root_path.join(".family.seed");
        assert!(seed_path.exists());
        assert_eq!(std::fs::metadata(&seed_path).unwrap().len(), 32);
    }

    #[tokio::test]
    #[cfg(unix)]
    async fn generate_seed_file_fails_on_readonly_path() {
        let temp = TempDir::new().unwrap();
        let root_path = temp.path().join("readonly_dir");
        let config = super::super::types::SporeConfig {
            label: "test".to_string(),
            node_id: "tower1".to_string(),
            family_id: "default".to_string(),
            spore_type: SporeType::Live,
            plasmid_bin_dir: None,
        };
        let spore = Spore {
            root_path: root_path.clone(),
            config,
        };

        std::fs::create_dir_all(&root_path).unwrap();
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&root_path).unwrap().permissions();
        perms.set_mode(0o444);
        std::fs::set_permissions(&root_path, perms).unwrap();

        let result = spore.generate_seed_file().await;
        assert!(result.is_err());
    }
}
