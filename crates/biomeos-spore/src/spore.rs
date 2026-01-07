//! Spore structure and USB orchestration
//!
//! Manages the complete lifecycle of a USB spore, from creation to verification.

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tokio::fs as async_fs;
use tracing::{debug, info, warn};

use crate::error::{SporeError, SporeResult};
use crate::seed::FamilySeed;

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
    /// use biomeos_spore::{Spore, SporeConfig};
    /// use std::path::PathBuf;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = SporeConfig {
    ///     label: "biomeOS1".to_string(),
    ///     node_id: "tower1".to_string(),
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
            "Creating spore '{}' at: {}",
            config.label,
            root_path.display()
        );

        let spore = Self { root_path, config };

        // Execute creation steps
        spore.create_directory_structure().await?;
        spore.generate_seed_file().await?;
        spore.create_tower_config().await?;
        spore.copy_binaries().await?;
        spore.create_readme().await?;

        info!("Spore creation complete: {}", spore.root_path.display());
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
        };

        Ok(Self { root_path, config })
    }

    /// Clone this spore to create a sibling
    ///
    /// Siblings share the same family seed, making them cryptographically
    /// related. BearDog will recognize them as family members.
    pub async fn clone_sibling(
        &self,
        target_mount: PathBuf,
        new_config: SporeConfig,
    ) -> SporeResult<Spore> {
        info!(
            "Cloning spore to create sibling '{}' from '{}'",
            new_config.node_id, self.config.node_id
        );

        // Read the family seed from source (verify it exists)
        let source_seed = self.root_path.join(".family.seed");
        let _seed = FamilySeed::from_file(&source_seed)?;

        // Create new spore
        let sibling = Self {
            root_path: target_mount.join("biomeOS"),
            config: new_config,
        };

        // Create structure
        sibling.create_directory_structure().await?;

        // Copy the same family seed (siblings!)
        let target_seed = sibling.root_path.join(".family.seed");
        async_fs::copy(&source_seed, &target_seed).await?;
        debug!("Copied family seed to sibling");

        // Create sibling's config with new node_id
        sibling.create_tower_config().await?;

        // Copy binaries
        sibling.copy_binaries().await?;
        sibling.create_readme().await?;

        info!("Sibling spore created successfully");
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
            r#"# BiomeOS Tower Configuration
# Generated spore: {}
# Port-Free Architecture - Unix Sockets + UDP Multicast

[tower]
# Family ID will be extracted by BearDog from seed file
concurrent_startup = true

# BearDog - Security Primal (Port-Free!)
[[primals]]
binary = "./primals/beardog"
provides = ["Security", "Encryption", "Trust"]
requires = []

[primals.env]
# biomeOS provides the FILE PATH
# BearDog handles ALL crypto processing
BEARDOG_FAMILY_SEED_FILE = "${{USB_ROOT}}/.family.seed"
BEARDOG_NODE_ID = "{node_id}"
RUST_LOG = "info"

# Songbird - Discovery Orchestrator (UDP Multicast)
[[primals]]
binary = "./primals/songbird"
provides = ["Discovery"]
requires = ["Security"]

[primals.env]
# Songbird discovers its family through BearDog
# No hardcoded family ID needed!
SONGBIRD_NODE_ID = "{node_id}"
SECURITY_ENDPOINT = "unix:///tmp/beardog-${{FAMILY}}-{node_id}.sock"
RUST_LOG = "info"
"#,
            self.config.label,
            node_id = self.config.node_id,
        )
    }

    /// Copy primal binaries to spore
    async fn copy_binaries(&self) -> SporeResult<()> {
        info!("Copying primal binaries");

        // Source directory (relative to workspace root)
        let source_dir = PathBuf::from("primalBins");
        let target_dir = self.root_path.join("primals");

        // Copy each primal binary
        for binary in &["beardog", "songbird"] {
            let src = source_dir.join(binary);
            let dst = target_dir.join(binary);

            if src.exists() {
                async_fs::copy(&src, &dst).await?;
                debug!("Copied binary: {} -> {}", src.display(), dst.display());

                // Set executable permissions
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let mut perms = async_fs::metadata(&dst).await?.permissions();
                    perms.set_mode(0o755);
                    async_fs::set_permissions(&dst, perms).await?;
                }
            } else {
                warn!("Binary not found (skipping): {}", src.display());
            }
        }

        // Copy tower orchestrator
        let tower_src = PathBuf::from("target/release/tower");
        let tower_dst = self.root_path.join("bin/tower");

        if tower_src.exists() {
            async_fs::copy(&tower_src, &tower_dst).await?;
            debug!("Copied tower orchestrator");

            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = async_fs::metadata(&tower_dst).await?.permissions();
                perms.set_mode(0o755);
                async_fs::set_permissions(&tower_dst, perms).await?;
            }
        } else {
            warn!("Tower binary not found at: {}", tower_src.display());
        }

        Ok(())
    }

    /// Create README with instructions
    async fn create_readme(&self) -> SporeResult<()> {
        let readme = format!(
            r#"# biomeOS Spore: {}

Node ID: {}
Created: {}

## Usage

Boot your machine from this USB drive, or mount and run:

```bash
cd /path/to/mount/biomeOS
./bin/tower run --config tower.toml
```

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
- `primals/` - Primal binaries (beardog, songbird)
- `bin/tower` - Tower orchestrator

---

Generated by biomeOS spore system
"#,
            self.config.label,
            self.config.node_id,
            chrono::Utc::now().to_rfc3339(),
        );

        let readme_path = self.root_path.join("README.md");
        async_fs::write(&readme_path, readme).await?;
        debug!("Created README.md");

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
        };

        // Note: This will fail without actual binaries, but tests the structure
        let result = Spore::create(mount_point.clone(), config).await;

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
        };

        let spore = Spore {
            root_path: PathBuf::from("/tmp/test"),
            config,
        };

        let toml = spore.generate_tower_toml();

        // Verify key elements
        assert!(toml.contains("BEARDOG_FAMILY_SEED_FILE"));
        assert!(toml.contains("tower1"));
        assert!(toml.contains("./primals/beardog"));
        assert!(toml.contains("./primals/songbird"));
        assert!(!toml.contains("BEARDOG_FAMILY_SEED =")); // Should NOT have raw seed
    }
}

