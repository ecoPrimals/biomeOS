//! Spore structure and USB orchestration
//!
//! Manages the complete lifecycle of a USB spore, from creation to verification.

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tokio::fs as async_fs;
use tracing::{debug, info};

use crate::error::{SporeError, SporeResult};
use crate::seed::FamilySeed;
use crate::spore_types::SporeType;

/// USB Spore - A self-contained biomeOS deployment
///
/// A spore contains everything needed to boot a biomeOS tower:
/// - Family seed (`.family.seed`)
/// - Primal binaries (beardog, songbird, etc.)
/// - Tower orchestrator binary
/// - Configuration (`tower.toml`)
/// - Directory structure
#[derive(Debug)]
pub struct Spore {
    root_path: PathBuf,
    config: SporeConfig,
}

/// Configuration for spore creation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SporeConfig {
    /// Human-readable label for this spore
    pub label: String,
    
    /// Node ID for this tower (e.g., "tower1")
    pub node_id: String,

    /// Type of spore (Cold = storage, Live = deployable)
    #[serde(default)]
    pub spore_type: SporeType,
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
    /// use biomeos_spore::{Spore, SporeConfig, SporeType};
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
        let node_id = Self::extract_node_id_from_config(&config_str)
            .unwrap_or_else(|| "unknown".to_string());

        let config = SporeConfig {
            label: root_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string(),
            node_id,
            spore_type: SporeType::default(), // Detect from manifest if available
        };

        Ok(Self { root_path, config })
    }

    /// Clone this spore to create a sibling
    ///
    /// Creates a genetically RELATED but individually UNIQUE sibling.
    /// The sibling derives its seed from the parent, making them:
    /// - Related by lineage (share parent DNA)
    /// - Individually unique (each has own seed)
    /// - Batch-trackable (know deployment cohort)
    ///
    /// This reflects real biology: siblings are NOT perfect clones!
    pub async fn clone_sibling(
        &self,
        target_mount: PathBuf,
        new_config: SporeConfig,
    ) -> SporeResult<Spore> {
        info!(
            "🧬 Creating sibling '{}' from parent '{}' (genetic derivation)",
            new_config.node_id, self.config.node_id
        );

        // Read the parent seed (verify it exists)
        let parent_seed_path = self.root_path.join(".family.seed");
        let _parent = FamilySeed::from_file(&parent_seed_path)?;

        // Create new spore
        let sibling = Self {
            root_path: target_mount.join("biomeOS"),
            config: new_config.clone(),
        };

        // Create structure
        sibling.create_directory_structure().await?;

        // Derive UNIQUE child seed from parent (genetic mixing!)
        // This makes sibling related but unique - like real DNA
        let target_seed = sibling.root_path.join(".family.seed");
        let deployment_batch = chrono::Utc::now().format("%Y%m%d").to_string();
        
        FamilySeed::derive_sibling(
            &parent_seed_path,
            &target_seed,
            &new_config.node_id,
            Some(&deployment_batch),
        )?;
        
        info!(
            "🌱 Derived unique seed for sibling '{}' (batch: {})",
            new_config.node_id, deployment_batch
        );

        // Create sibling's config with new node_id
        sibling.create_tower_config().await?;

        // Copy binaries (genetic material from parent)
        sibling.copy_binaries().await?;
        sibling.create_deployment_script().await?;
        sibling.create_readme().await?;

        info!("✅ Sibling spore created (related but unique!)");
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

    /// Create directory structure on USB
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
            self.config.label,
            self.config.node_id,
            self.config.label,
            self.config.node_id,
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

    /// Copy primal binaries to spore
    ///
    /// **Capability-Based Copying** (NOT hardcoded!)
    /// 
    /// Copies ALL binaries from nucleusBin/:
    /// - `tower/` - biomeOS orchestrator (always required)
    /// - `primals/*` - ALL primal binaries (agnostic, discovered at runtime)
    ///
    /// This enables:
    /// - New primals without code changes
    /// - Chimeras (embedded primals)
    /// - Name changes (beardog → beardog-v2)
    /// - BYOB (Bring Your Own Biome) manifest system
    ///
    /// The tower.toml manifest determines which primals are actually used.
    async fn copy_binaries(&self) -> SporeResult<()> {
        info!("Copying genetic material from nucleusBin/ (capability-based, agnostic)");

        // Source: NucleusBin - Single source of truth for stable binaries
        let nucleus_dir = PathBuf::from("nucleusBin");
        
        // Verify nucleus exists
        if !nucleus_dir.exists() {
            return Err(SporeError::BinaryNotFound(
                "nucleusBin/ directory not found - run scripts/harvest-primals.sh first".to_string()
            ));
        }
        
        // 1. Copy tower orchestrator (always required)
        let tower_src = nucleus_dir.join("tower/tower");
        let tower_dst = self.root_path.join("bin/tower");

        if tower_src.exists() {
            async_fs::copy(&tower_src, &tower_dst).await?;
            info!("✅ Copied tower orchestrator from nucleusBin/tower/");

            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = async_fs::metadata(&tower_dst).await?.permissions();
                perms.set_mode(0o755);
                async_fs::set_permissions(&tower_dst, perms).await?;
            }
        } else {
            return Err(SporeError::BinaryNotFound(
                format!("tower orchestrator not found at: {}", tower_src.display()),
            ));
        }

        // 2. Copy ALL primals from nucleusBin/primals/ (capability-based, agnostic)
        let primals_src_dir = nucleus_dir.join("primals");
        let primals_dst_dir = self.root_path.join("primals");
        
        if !primals_src_dir.exists() {
            return Err(SporeError::BinaryNotFound(
                format!("primals/ directory not found at: {}", primals_src_dir.display())
            ));
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
                let file_name = path.file_name().unwrap();
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
                "No primal binaries found in nucleusBin/primals/".to_string()
            ));
        }

        info!(
            "✅ Genetic material copied from nucleusBin/ (tower + {} primals, capability-based)",
            primal_count
        );
        Ok(())
    }

    /// Create README with instructions
    async fn create_readme(&self) -> SporeResult<()> {
        let usage_section = match self.config.spore_type {
            SporeType::Cold => {
                r#"## Usage (ColdSpore - Archive)

This is a **ColdSpore** - genetic material preserved for storage.

To activate this spore:
1. Copy to ext4/ext3 filesystem (preserves permissions)
2. OR convert to LiveSpore: `biomeos spore convert --type live`
3. Then deploy as normal

ColdSpores are ideal for:
- Long-term storage
- Distribution archives
- Backup copies
- Non-FAT32 transfers
"#
            }
            SporeType::Live => {
                r#"## Usage (LiveSpore - Deployable)

This is a **LiveSpore** - ready for immediate deployment!

Boot your machine from this USB drive, or mount and run:

```bash
cd /path/to/mount/biomeOS
./deploy.sh
```

LiveSpores automatically handle:
- FAT32 filesystem limitations
- Permission setup
- Execution environment
"#
            }
        };

        let readme = format!(
            r#"# biomeOS {}: {}

Node ID: {}
Type: {} {}
Created: {}

{}

## Architecture

This spore uses a composable security architecture:

- **biomeOS**: Orchestration layer (this spore)
- **BearDog**: Security layer (reads `.family.seed`, handles all crypto)
- **Songbird**: Discovery layer (UDP multicast peer discovery)

## Genetic Lineage

The `.family.seed` file contains cryptographic material that establishes
this tower's family membership. BearDog processes this seed using HKDF-SHA256
to derive unique child keys while maintaining family relationships.

**DO NOT share or copy .family.seed publicly!**

## Files

- `.family.seed` - Family cryptographic seed (600 permissions)
- `tower.toml` - Tower configuration
- `primals/` - Primal binaries (beardog-server, songbird)
- `bin/tower` - Tower orchestrator
{}

---

Generated by biomeOS spore system
"#,
            self.config.spore_type,
            self.config.label,
            self.config.node_id,
            self.config.spore_type.emoji(),
            self.config.spore_type,
            chrono::Utc::now().to_rfc3339(),
            usage_section,
            if self.config.spore_type.requires_execution_env() {
                "\n- `deploy.sh` - Deployment script (LiveSpore)"
            } else {
                ""
            },
        );

        let readme_path = self.root_path.join("README.md");
        async_fs::write(&readme_path, readme).await?;
        debug!("Created README.md");

        Ok(())
    }

    /// Create spore manifest (metadata about this spore)
    async fn create_spore_manifest(&self) -> SporeResult<()> {
        use serde_json::json;

        let manifest = json!({
            "version": "1.0",
            "spore_type": self.config.spore_type,
            "label": self.config.label,
            "node_id": self.config.node_id,
            "created_at": chrono::Utc::now().to_rfc3339(),
            "genetic_material": {
                "tower": "bin/tower",
                "beardog_server": "primals/beardog-server",
                "songbird": "primals/songbird"
            },
            "genetic_lineage": ".family.seed",
            "configuration": "tower.toml"
        });

        let manifest_path = self.root_path.join(".spore.json");
        async_fs::write(&manifest_path, serde_json::to_string_pretty(&manifest)?).await?;
        debug!("Created .spore.json manifest");

        Ok(())
    }

    /// Extract node_id from tower.toml (simple parsing)
    fn extract_node_id_from_config(config_str: &str) -> Option<String> {
        for line in config_str.lines() {
            if line.contains("BEARDOG_NODE_ID") || line.contains("SONGBIRD_NODE_ID") {
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_create_spore() {
        let temp_dir = TempDir::new().unwrap();
        let mount_point = temp_dir.path().to_path_buf();

        let config = SporeConfig {
            label: "test_spore".to_string(),
            node_id: "tower1".to_string(),
            spore_type: SporeType::default(),
        };

        // Note: This will fail without actual binaries, but tests the structure
        let _result = Spore::create(mount_point.clone(), config).await;

        // Should create directory structure even if binary copy fails
        let root_path = mount_point.join("biomeOS");
        assert!(root_path.exists());
        assert!(root_path.join("primals").exists());
        assert!(root_path.join("bin").exists());
        assert!(root_path.join("secrets").exists());
    }

    #[tokio::test]
    async fn test_directory_structure() {
        let temp_dir = TempDir::new().unwrap();
        let mount_point = temp_dir.path().to_path_buf();

        let config = SporeConfig {
            label: "test".to_string(),
            node_id: "tower1".to_string(),
            spore_type: SporeType::default(),
        };

        let spore = Spore {
            root_path: mount_point.join("biomeOS"),
            config,
        };

        spore.create_directory_structure().await.unwrap();

        // Verify all directories were created
        assert!(spore.root_path.join("bin").exists());
        assert!(spore.root_path.join("primals").exists());
        assert!(spore.root_path.join("primals/certs").exists());
        assert!(spore.root_path.join("secrets").exists());
        assert!(spore.root_path.join("logs").exists());
    }

    #[test]
    fn test_generate_tower_toml() {
        let config = SporeConfig {
            label: "test_spore".to_string(),
            node_id: "tower1".to_string(),
            spore_type: SporeType::default(),
        };

        let spore = Spore {
            root_path: PathBuf::from("/tmp/test"),
            config,
        };

        let toml = spore.generate_tower_toml();

        // Verify key elements
        assert!(toml.contains("BEARDOG_FAMILY_SEED_FILE"));
        assert!(toml.contains("tower1"));
        assert!(toml.contains("./primals/beardog-server")); // Server binary, not CLI
        assert!(toml.contains("./primals/songbird"));
        assert!(toml.contains("btsp_enabled")); // BTSP support
        assert!(!toml.contains("BEARDOG_FAMILY_SEED =")); // Should NOT have raw seed
    }
}

