// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Copyright 2025-2026 ecoPrimals Project
// Licensed under the Affero General Public License v3.0.
// See LICENSE file in the project root or visit https://www.gnu.org/licenses/agpl-3.0.html

//! Filesystem operations for spores
//!
//! Handles directory structure creation and binary copying from plasmidBin.

use std::path::PathBuf;
use tokio::fs as async_fs;
use tracing::{debug, info};

use super::core::Spore;
use crate::error::{SporeError, SporeResult};

/// Trait for filesystem operations on spores
pub(super) trait FilesystemOps {
    /// Create directory structure on USB
    fn create_directory_structure(
        &self,
    ) -> impl std::future::Future<Output = SporeResult<()>> + Send;

    /// Copy binaries from plasmidBin
    fn copy_binaries(&self) -> impl std::future::Future<Output = SporeResult<()>> + Send;
}

impl FilesystemOps for Spore {
    async fn create_directory_structure(&self) -> SporeResult<()> {
        info!("Creating directory structure");

        let dirs = [
            "bin",
            "primals",
            "primals/certs",
            "secrets",
            "logs",
            "config",
        ];

        for dir in &dirs {
            let path = self.root_path.join(dir);
            async_fs::create_dir_all(&path).await?;
            debug!("Created directory: {}", path.display());
        }

        // Set secure permissions on secrets directory
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let secrets = self.root_path.join("secrets");
            let mut perms = async_fs::metadata(&secrets).await?.permissions();
            perms.set_mode(0o700);
            async_fs::set_permissions(&secrets, perms).await?;
            debug!("Set permissions to 0700 for secrets/");
        }

        Ok(())
    }

    /// Copy ALL binaries from plasmidBin/ (capability-based, agnostic)
    ///
    /// # Capability-Based & Agnostic Design
    ///
    /// This method:
    /// - **Does NOT hardcode primal names** (no "beardog", "songbird", etc.)
    /// - Copies ALL binaries from `plasmidBin/primals/`
    /// - Lets `tower.toml` determine which primals are used
    /// - Supports future evolution without code changes
    ///
    /// Benefits:
    /// - New primals without code changes
    /// - Chimeras (embedded primals)
    /// - Name changes (beardog → beardog-v2)
    /// - BYOB (Bring Your Own Biome) manifest system
    ///
    /// The tower.toml manifest determines which primals are actually used.
    async fn copy_binaries(&self) -> SporeResult<()> {
        info!("Copying genetic material from plasmidBin/ (capability-based, agnostic)");

        let nucleus_dir = self
            .config
            .plasmid_bin_dir
            .clone()
            .unwrap_or_else(|| PathBuf::from("plasmidBin"));

        // Verify nucleus exists
        if !nucleus_dir.exists() {
            return Err(SporeError::BinaryNotFound(
                "plasmidBin/ directory not found - run harvest from tools/harvest/ first"
                    .to_string(),
            ));
        }

        // 1. Copy tower orchestrator (always required)
        let tower_src = nucleus_dir.join("tower/tower");
        let tower_dst = self.root_path.join("bin/tower");

        if tower_src.exists() {
            async_fs::copy(&tower_src, &tower_dst).await?;
            info!("✅ Copied tower orchestrator from plasmidBin/tower/");

            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = async_fs::metadata(&tower_dst).await?.permissions();
                perms.set_mode(0o755);
                async_fs::set_permissions(&tower_dst, perms).await?;
            }
        } else {
            return Err(SporeError::BinaryNotFound(format!(
                "tower orchestrator not found at: {}",
                tower_src.display()
            )));
        }

        // 2. Copy ALL primals from plasmidBin/primals/ (capability-based, agnostic)
        let primals_src_dir = nucleus_dir.join("primals");
        let primals_dst_dir = self.root_path.join("primals");

        if !primals_src_dir.exists() {
            return Err(SporeError::BinaryNotFound(format!(
                "primals/ directory not found at: {}",
                primals_src_dir.display()
            )));
        }

        let mut primal_count = 0;
        let mut entries = async_fs::read_dir(&primals_src_dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();

            // Skip .gitkeep and other dotfiles
            if let Some(name) = path.file_name() {
                let name_str = name.to_string_lossy();
                if name_str.starts_with('.') {
                    continue;
                }
            }

            // Only copy files (not directories)
            if path.is_file() {
                #[expect(
                    clippy::expect_used,
                    reason = "path confirmed as file must have filename"
                )]
                let file_name = path
                    .file_name()
                    .expect("path confirmed as file must have filename");
                let dst_path = primals_dst_dir.join(file_name);

                async_fs::copy(&path, &dst_path).await?;
                primal_count += 1;

                info!("✅ Copied primal: {}", file_name.to_string_lossy());

                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let mut perms = async_fs::metadata(&dst_path).await?.permissions();
                    perms.set_mode(0o755);
                    async_fs::set_permissions(&dst_path, perms).await?;
                }
            }
        }

        if primal_count == 0 {
            return Err(SporeError::BinaryNotFound(
                "No primal binaries found in plasmidBin/primals/".to_string(),
            ));
        }

        info!(
            "✅ Genetic material copied from plasmidBin/ (tower + {} primals, capability-based)",
            primal_count
        );
        Ok(())
    }
}
