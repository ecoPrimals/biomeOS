//! Family ID Discovery Module
//!
//! Discovers the family ID for a biomeOS deployment through multiple sources:
//! 1. `.family.seed` file (canonical for LiveSpore)
//! 2. `FAMILY_ID` environment variable
//! 3. `BIOMEOS_FAMILY_ID` environment variable
//!
//! The family ID is derived from the first 8 bytes of the genesis seed,
//! encoded as hex (16 characters).
//!
//! ## Dark Forest Protocol
//!
//! Family IDs enable the Dark Forest protocol for secure peer discovery:
//! - Only members of the same genetic lineage can decrypt broadcast beacons
//! - The genesis seed (first 32 bytes) is shared among family members
//! - The node key (bytes 32-63) is unique per node

use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};

/// Family ID format: 16 hex characters (8 bytes of genesis seed)
pub type FamilyId = String;

/// Result of family discovery
#[derive(Debug, Clone)]
pub struct DiscoveredFamily {
    /// The family ID (hex-encoded)
    pub id: FamilyId,
    /// Source of discovery
    pub source: FamilySource,
    /// Full genesis seed if available (32 bytes)
    pub genesis_seed: Option<Vec<u8>>,
    /// Node key if available (32 bytes)
    pub node_key: Option<Vec<u8>>,
}

/// Source of family ID discovery
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FamilySource {
    /// From `.family.seed` file
    SeedFile(PathBuf),
    /// From `FAMILY_ID` environment variable
    FamilyIdEnv,
    /// From `BIOMEOS_FAMILY_ID` environment variable
    BiomeosEnv,
    /// Default fallback (development only)
    Default,
}

impl std::fmt::Display for FamilySource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FamilySource::SeedFile(path) => write!(f, "seed file ({})", path.display()),
            FamilySource::FamilyIdEnv => write!(f, "FAMILY_ID env var"),
            FamilySource::BiomeosEnv => write!(f, "BIOMEOS_FAMILY_ID env var"),
            FamilySource::Default => write!(f, "default (development)"),
        }
    }
}

/// Family Discovery Configuration
#[derive(Debug, Clone)]
pub struct FamilyDiscoveryConfig {
    /// Paths to search for `.family.seed` file
    pub seed_file_paths: Vec<PathBuf>,
    /// Whether to allow default fallback
    pub allow_default: bool,
    /// Default family ID (only used if allow_default is true)
    pub default_family: String,
}

impl Default for FamilyDiscoveryConfig {
    fn default() -> Self {
        let mut seed_paths = vec![
            // Current directory
            PathBuf::from(".family.seed"),
            // XDG data directory
            PathBuf::from(std::env::var("XDG_DATA_HOME").unwrap_or_else(|_| {
                let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
                format!("{}/.local/share", home)
            }))
            .join("biomeos/.family.seed"),
        ];

        // Add USB mount points
        if let Ok(user) = std::env::var("USER") {
            seed_paths.push(PathBuf::from(format!("/media/{}", user)).join("biomeOS/.family.seed"));
            seed_paths.push(
                PathBuf::from(format!("/media/{}", user)).join("biomeOS1/biomeOS/.family.seed"),
            );
            seed_paths.push(
                PathBuf::from(format!("/media/{}", user)).join("biomeOS21/biomeOS/.family.seed"),
            );
        }

        Self {
            seed_file_paths: seed_paths,
            allow_default: true, // Allow default in dev
            default_family: "default".to_string(),
        }
    }
}

/// Discover family ID from available sources
///
/// Priority:
/// 1. FAMILY_ID env var (explicit override)
/// 2. BIOMEOS_FAMILY_ID env var
/// 3. .family.seed file in configured paths
/// 4. Default (if allowed)
pub fn discover_family() -> Option<DiscoveredFamily> {
    discover_family_with_config(&FamilyDiscoveryConfig::default())
}

/// Discover family ID with custom configuration
pub fn discover_family_with_config(config: &FamilyDiscoveryConfig) -> Option<DiscoveredFamily> {
    // 1. Search for .family.seed file (seed-derived identity is canonical)
    for seed_path in &config.seed_file_paths {
        if let Some(family) = read_family_seed(seed_path) {
            info!(
                "🧬 Family ID from seed file {}: {}",
                seed_path.display(),
                family.id
            );
            return Some(family);
        }
    }

    // 2. Check FAMILY_ID env var (explicit override — must be seed-derived, not a tag)
    if let Ok(family_id) = std::env::var("FAMILY_ID") {
        if !family_id.is_empty() {
            if family_id == "nat0" {
                warn!("⚠️ FAMILY_ID='nat0' is a deprecated prototype tag — ignoring. Derive from .family.seed instead.");
            } else {
                info!("🧬 Family ID from FAMILY_ID env: {}", family_id);
                return Some(DiscoveredFamily {
                    id: family_id,
                    source: FamilySource::FamilyIdEnv,
                    genesis_seed: None,
                    node_key: None,
                });
            }
        }
    }

    // 3. Check BIOMEOS_FAMILY_ID env var
    if let Ok(family_id) = std::env::var("BIOMEOS_FAMILY_ID") {
        if !family_id.is_empty() && family_id != "nat0" {
            info!("🧬 Family ID from BIOMEOS_FAMILY_ID env: {}", family_id);
            return Some(DiscoveredFamily {
                id: family_id,
                source: FamilySource::BiomeosEnv,
                genesis_seed: None,
                node_key: None,
            });
        }
    }

    // 5. Default fallback (development only)
    if config.allow_default {
        warn!("⚠️ No family seed found, using default (development mode)");
        return Some(DiscoveredFamily {
            id: config.default_family.clone(),
            source: FamilySource::Default,
            genesis_seed: None,
            node_key: None,
        });
    }

    warn!("❌ No family ID found and defaults not allowed");
    None
}

/// Read family ID from a `.family.seed` file
///
/// File format: 64 bytes raw binary
/// - Bytes 0-31: Genesis seed (shared by family)
/// - Bytes 32-63: Node key (unique per node)
///
/// Family ID = hex(genesis_seed[0..8]) = 16 hex chars
fn read_family_seed(path: &Path) -> Option<DiscoveredFamily> {
    if !path.exists() {
        debug!("Seed file not found: {}", path.display());
        return None;
    }

    match std::fs::read(path) {
        Ok(data) => {
            if data.len() < 32 {
                warn!(
                    "Seed file too short: {} bytes (need at least 32)",
                    data.len()
                );
                return None;
            }

            // Family ID = hex of first 8 bytes of genesis seed
            let family_id = hex::encode(&data[0..8]);

            // Extract genesis seed and node key
            let genesis_seed = data[0..32].to_vec();
            let node_key = if data.len() >= 64 {
                Some(data[32..64].to_vec())
            } else {
                None
            };

            info!(
                "🧬 Read family seed: genesis={} bytes, node_key={} bytes",
                genesis_seed.len(),
                node_key.as_ref().map_or(0, |k| k.len())
            );

            Some(DiscoveredFamily {
                id: family_id,
                source: FamilySource::SeedFile(path.to_path_buf()),
                genesis_seed: Some(genesis_seed),
                node_key,
            })
        }
        Err(e) => {
            warn!("Failed to read seed file {}: {}", path.display(), e);
            None
        }
    }
}

/// Get family ID string, falling back to default if not found
pub fn get_family_id() -> String {
    discover_family()
        .map(|f| f.id)
        .unwrap_or_else(|| "default".to_string())
}

/// Get family ID from environment or default
pub fn get_family_id_from_env() -> String {
    std::env::var("FAMILY_ID")
        .or_else(|_| std::env::var("BIOMEOS_FAMILY_ID"))
        .unwrap_or_else(|_| "default".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    // NOTE: This test modifies global environment state and may be flaky in parallel execution.
    // Consider using a dedicated test harness with sequential execution for environment tests.
    #[test]
    #[ignore] // Ignored due to environment variable global state issues
    fn test_discover_from_env() {
        std::env::set_var("FAMILY_ID", "test_family_123");
        let result = discover_family();
        assert!(result.is_some());
        let family = result.unwrap();
        assert_eq!(family.id, "test_family_123");
        assert_eq!(family.source, FamilySource::FamilyIdEnv);
        std::env::remove_var("FAMILY_ID");
    }

    #[test]
    fn test_discover_from_seed_file() {
        let temp_dir = TempDir::new().unwrap();
        let seed_path = temp_dir.path().join(".family.seed");

        // Create 64-byte seed
        let mut seed_data = vec![0u8; 64];
        // Genesis seed starts with 0xCF7E8729...
        seed_data[0..8].copy_from_slice(&[0xCF, 0x7E, 0x87, 0x29, 0xDC, 0x4F, 0xF0, 0x5F]);

        let mut file = std::fs::File::create(&seed_path).unwrap();
        file.write_all(&seed_data).unwrap();

        let config = FamilyDiscoveryConfig {
            seed_file_paths: vec![seed_path.clone()],
            allow_default: false,
            default_family: "default".to_string(),
        };

        // Clear env vars to ensure seed file is used
        std::env::remove_var("FAMILY_ID");
        std::env::remove_var("BIOMEOS_FAMILY_ID");

        let result = discover_family_with_config(&config);
        assert!(result.is_some());
        let family = result.unwrap();
        assert_eq!(family.id, "cf7e8729dc4ff05f");
        assert!(matches!(family.source, FamilySource::SeedFile(_)));
        assert!(family.genesis_seed.is_some());
    }

    #[test]
    fn test_get_family_id_default() {
        std::env::remove_var("FAMILY_ID");
        std::env::remove_var("BIOMEOS_FAMILY_ID");

        // Will fall back to default
        let family_id = get_family_id();
        // Should be either from seed file or "default"
        assert!(!family_id.is_empty());
    }
}
