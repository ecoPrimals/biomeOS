// SPDX-License-Identifier: APGL-3.0-or-later WITH Sovran-Exemption-1.0
//
// Copyright 2025 ecoPrimals Project
// Licensed under the Affero General Public License v3.0 or later with Sovran Exemption 1.0.
// See LICENSE file in the project root or visit https://www.gnu.org/licenses/agpl-3.0.html

//! Deployment script generation for spores
//!
//! Handles creation of deploy.sh for LiveSpores (bootable spores).

use tokio::fs as async_fs;
use tracing::info;

use super::core::Spore;
use crate::error::SporeResult;

/// Trait for deployment script operations on spores
pub(super) trait DeploymentOps {
    /// Create deployment script for spore
    fn create_deployment_script(&self)
        -> impl std::future::Future<Output = SporeResult<()>> + Send;
}

impl DeploymentOps for Spore {
    /// Create deployment script for spore
    ///
    /// Makes the spore immediately bootable from USB
    /// Handles FAT32 filesystem limitations (no execute permissions)
    async fn create_deployment_script(&self) -> SporeResult<()> {
        info!("Creating deployment script");

        let script = format!(
            r#"#!/usr/bin/env bash
#
# BiomeOS USB Spore Deployment
# Generated for: {}
# Node ID: {}
#
# This spore is self-contained and ready to deploy!
#

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${{BASH_SOURCE[0]}}")" && pwd)"
cd "$SCRIPT_DIR"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🌱 biomeOS Spore Deployment"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Spore: {}"
echo "Node:  {}"
echo "Family: nat0 (genetic lineage)"
echo ""

# Fix permissions (FAT32 USB drives don't preserve execute bits)
echo "🔧 Preparing genetic material for execution..."
chmod -R +x bin/ primals/ 2>/dev/null || true
chmod 600 .family.seed 2>/dev/null || true
chmod 700 secrets/ 2>/dev/null || true
echo "✅ Permissions set"
echo ""

# Verify genetic material (3 core binaries)
if [ ! -f "bin/tower" ]; then
    echo "❌ Error: tower orchestrator not found"
    exit 1
fi

if [ ! -f "primals/beardog-server" ]; then
    echo "❌ Error: beardog-server not found"
    exit 1
fi

if [ ! -f "primals/songbird" ]; then
    echo "❌ Error: songbird not found"
    exit 1
fi

# Verify genetic lineage
if [ ! -f ".family.seed" ]; then
    echo "❌ Error: .family.seed not found"
    echo "   This spore has no genetic lineage!"
    exit 1
fi

echo "✅ Genetic material verified (3/3 binaries)"
echo "✅ Genetic lineage present (.family.seed)"
echo ""

# Display configuration
echo "📋 Configuration:"
echo "  • Config: tower.toml"
echo "  • Family: nat0"
echo "  • Concurrent: true"
echo ""

# Start tower with modern orchestration
echo "🌊 Starting tower with genetic lineage..."
echo ""

# Use bash to execute (works on FAT32 where execute bit doesn't work)
if [ -x "./bin/tower" ]; then
    exec ./bin/tower run --config tower.toml
else
    # FAT32 fallback: Copy to temp location with proper permissions
    TEMP_DIR=$(mktemp -d)
    echo "ℹ️  FAT32 detected - copying to temporary location..."
    cp -r . "$TEMP_DIR/spore"
    cd "$TEMP_DIR/spore"
    chmod -R +x bin/ primals/
    chmod 600 .family.seed
    chmod 700 secrets/
    echo "✅ Prepared in: $TEMP_DIR/spore"
    exec ./bin/tower run --config tower.toml
fi
"#,
            self.config.label, self.config.node_id, self.config.label, self.config.node_id,
        );

        let script_path = self.root_path.join("deploy.sh");
        async_fs::write(&script_path, script).await?;

        // Make executable (will work on ext4, not on FAT32, but deploy.sh handles this)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = async_fs::metadata(&script_path).await?.permissions();
            perms.set_mode(0o755);
            async_fs::set_permissions(&script_path, perms).await?;
        }

        info!("✅ Created deploy.sh (self-bootable, FAT32-aware)");
        Ok(())
    }
}
