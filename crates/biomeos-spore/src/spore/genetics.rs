// SPDX-License-Identifier: APGL-3.0-or-later WITH Sovran-Exemption-1.0
//
// Copyright 2025 ecoPrimals Project
// Licensed under the Affero General Public License v3.0 or later with Sovran Exemption 1.0.
// See LICENSE file in the project root or visit https://www.gnu.org/licenses/agpl-3.0.html

//! Genetic seed generation for spores
//!
//! Handles family seed generation using BearDog cryptographic principles.

use tracing::{debug, info};

use crate::error::{SporeError, SporeResult};
use crate::seed::FamilySeed;
use super::core::Spore;

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
        tokio::task::spawn_blocking(move || {
            FamilySeed::generate_and_write(&seed_path_clone)
        })
        .await
        .map_err(|e| SporeError::InvalidConfig(format!("Task join error: {}", e)))??;

        debug!("Family seed generated at: {}", seed_path.display());
        Ok(())
    }
}

