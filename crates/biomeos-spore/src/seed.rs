// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Family seed file management
//!
//! **IMPORTANT**: This module only handles FILE operations and seed DERIVATION.
//! `BearDog` performs cryptographic family ID extraction and key derivation.
//!
//! ## Responsibility Boundary
//!
//! ### biomeOS (This Module)
//! - Generate genesis entropy (random bytes)
//! - Derive child seeds (SHA256-based genetic mixing)
//! - Write seed files to disk
//! - Set file permissions
//! - Verify file exists and has correct size
//! - Provide file path to `BearDog`
//!
//! ### `BearDog` (Security Primal)
//! - Read seed file contents
//! - HKDF-SHA256 for family ID extraction
//! - Generate operational keys
//! - Zeroize sensitive data
//! - Trust evaluation
//!
//! ## Genetic Model
//!
//! Siblings are NOT perfect clones - they are genetically related but unique:
//! - Genesis: Creates parent seed (32 random bytes)
//! - Sibling: Derives child seed from parent + `node_id` + batch
//! - Formula: child = SHA256(parent || `node_id` || batch)

use std::fs;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use crate::error::{SporeError, SporeResult};
use tracing::{debug, info};

/// Family seed file manager
///
/// This struct represents a family seed FILE (not the cryptographic content).
/// biomeOS manages the file, `BearDog` processes the crypto.
#[derive(Debug, Clone)]
pub struct FamilySeed {
    /// Path to the seed file
    file_path: PathBuf,
}

impl FamilySeed {
    /// Generate genesis seed (parent DNA for a new family)
    ///
    /// Creates 256 bits of cryptographically secure random bytes to serve as
    /// the "parent DNA" for a new genetic family. All siblings will derive
    /// their unique seeds from this parent.
    ///
    /// # Security
    ///
    /// - Uses OS-level cryptographic RNG (`rand::rng()`)
    /// - Sets file permissions to 0600 (owner read/write only) on Unix
    /// - Creates new genetic lineage (not derived from anything)
    ///
    /// # Arguments
    ///
    /// * `path` - Where to write the seed file (typically `.family.seed`)
    pub fn generate_genesis<P: AsRef<Path>>(path: P) -> SporeResult<Self> {
        use rand::RngCore;

        let path = path.as_ref().to_path_buf();
        info!(
            "🧬 Generating genesis seed (parent DNA) at: {}",
            path.display()
        );

        // Generate 256 bits of entropy (32 bytes) - the "parent DNA"
        let mut bytes = [0u8; 32];
        rand::rng().fill_bytes(&mut bytes);

        debug!("Generated 32 bytes of genesis entropy");

        // Write to file
        fs::write(&path, bytes)?;
        info!("Wrote genesis seed to file: {}", path.display());

        // Set secure permissions on Unix
        #[cfg(unix)]
        Self::set_secure_permissions(&path)?;

        Ok(Self { file_path: path })
    }

    /// Derive sibling seed from parent (genetic mixing)
    ///
    /// Creates a UNIQUE seed for a sibling by mixing:
    /// - Parent seed (shared family DNA)
    /// - Node ID (individual identity)
    /// - Deployment batch (birth cohort)
    ///
    /// This makes siblings genetically RELATED but individually UNIQUE,
    /// just like real biology!
    ///
    /// # Formula
    ///
    /// ```text
    /// child_seed = SHA256(parent_seed || node_id || deployment_batch)
    /// ```
    ///
    /// # Security
    ///
    /// - SHA256 ensures unique, unpredictable output
    /// - Same inputs always produce same output (deterministic)
    /// - Different `node_id` produces different seed
    /// - Cannot reverse to find parent seed
    ///
    /// # Arguments
    ///
    /// * `parent_path` - Path to parent's .family.seed file
    /// * `target_path` - Where to write child's seed
    /// * `node_id` - Individual identity (e.g., "node-alpha")
    /// * `deployment_batch` - Birth cohort (e.g., "20260107")
    pub fn derive_sibling<P: AsRef<Path>>(
        parent_path: P,
        target_path: P,
        node_id: &str,
        deployment_batch: Option<&str>,
    ) -> SporeResult<Self> {
        let parent_path = parent_path.as_ref();
        let target_path = target_path.as_ref().to_path_buf();

        info!("🧬 Deriving sibling seed for '{}' from parent", node_id);

        // Read parent seed (the "parent DNA")
        let parent_seed = fs::read(parent_path)?;
        if parent_seed.len() != 32 {
            return Err(SporeError::InvalidSeedLength {
                expected: 32,
                found: parent_seed.len() as u64,
            });
        }

        // Derive child seed using genetic mixing
        let child_seed = Self::genetic_mix(&parent_seed, node_id, deployment_batch);

        debug!(
            "Derived unique child seed for '{}' (batch: {:?})",
            node_id, deployment_batch
        );

        // Write child's unique genetic identity
        fs::write(&target_path, child_seed)?;
        info!("Wrote sibling seed to file: {}", target_path.display());

        // Set secure permissions on Unix
        #[cfg(unix)]
        Self::set_secure_permissions(&target_path)?;

        Ok(Self {
            file_path: target_path,
        })
    }

    /// Genetic mixing: combine parent DNA with individual traits
    ///
    /// Formula: `child_seed` = `SHA256(parent_seed` || `node_id` || `batch_id`)
    ///
    /// This creates unique individuals who share family traits:
    /// - Same parent + different `node_id` = siblings (related but unique)
    /// - Same `batch_id` = from same deployment (like twins/triplets)
    fn genetic_mix(parent_seed: &[u8], node_id: &str, deployment_batch: Option<&str>) -> [u8; 32] {
        let mut hasher = Sha256::new();

        // Parent genetic material (shared by all siblings)
        hasher.update(parent_seed);

        // Individual identity (unique to this sibling)
        hasher.update(node_id.as_bytes());

        // Deployment batch (shared by siblings "born together")
        if let Some(batch) = deployment_batch {
            hasher.update(batch.as_bytes());
        }

        let result = hasher.finalize();
        let mut child_seed = [0u8; 32];
        child_seed.copy_from_slice(&result);
        child_seed
    }

    /// Legacy: Generate and write (backward compatibility)
    ///
    /// Wraps `generate_genesis` for existing code.
    pub fn generate_and_write<P: AsRef<Path>>(path: P) -> SporeResult<Self> {
        Self::generate_genesis(path)
    }

    /// Load existing seed file
    ///
    /// Verifies the file exists and is 32 bytes, but does NOT read or
    /// process the contents. `BearDog` will read the file when needed.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the existing seed file
    pub fn from_file<P: AsRef<Path>>(path: P) -> SporeResult<Self> {
        let path = path.as_ref().to_path_buf();

        if !path.exists() {
            return Err(SporeError::SeedFileNotFound(path));
        }

        // Verify it's 32 bytes (but don't read the contents!)
        let metadata = fs::metadata(&path)?;
        if metadata.len() != 32 {
            return Err(SporeError::InvalidSeedLength {
                expected: 32,
                found: metadata.len(),
            });
        }

        debug!("Verified seed file: {}", path.display());
        Ok(Self { file_path: path })
    }

    /// Get the file path
    ///
    /// This path should be passed to `BearDog` via the
    /// `BEARDOG_FAMILY_SEED_FILE` environment variable.
    #[must_use]
    pub fn file_path(&self) -> &Path {
        &self.file_path
    }

    /// Build environment variables for a crypto provider to use this seed.
    ///
    /// Returns a `(key, value)` pair suitable for `Command::env()`.
    /// The caller should pass this to the discovered crypto-capable primal
    /// via the process spawning API rather than mutating global env state.
    pub fn crypto_provider_env(&self) -> SporeResult<(String, String)> {
        let path_str = self
            .file_path
            .to_str()
            .ok_or(SporeError::InvalidPath)?
            .to_string();

        debug!("Prepared seed file env for crypto provider: {}", path_str);
        Ok(("FAMILY_SEED_FILE".to_string(), path_str))
    }

    /// Set secure permissions on the seed file (Unix only)
    ///
    /// Sets permissions to 0600 (owner read/write only) to prevent
    /// unauthorized access to the seed material.
    #[cfg(unix)]
    fn set_secure_permissions(path: &Path) -> SporeResult<()> {
        use std::os::unix::fs::PermissionsExt;

        let mut perms = fs::metadata(path)?.permissions();
        perms.set_mode(0o600);
        fs::set_permissions(path, perms)?;

        debug!("Set permissions to 0600 for: {}", path.display());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_generate_and_write() {
        let temp_dir = TempDir::new().unwrap();
        let seed_path = temp_dir.path().join(".family.seed");

        let seed = FamilySeed::generate_and_write(&seed_path).unwrap();
        assert_eq!(seed.file_path(), seed_path.as_path());

        // Verify file exists and is 32 bytes
        let metadata = fs::metadata(&seed_path).unwrap();
        assert_eq!(metadata.len(), 32);

        // Verify permissions on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = metadata.permissions();
            assert_eq!(perms.mode() & 0o777, 0o600);
        }
    }

    #[test]
    fn test_from_file() {
        let temp_dir = TempDir::new().unwrap();
        let seed_path = temp_dir.path().join(".family.seed");

        // Create a seed file
        let bytes = [42u8; 32];
        fs::write(&seed_path, bytes).unwrap();

        // Load it
        let seed = FamilySeed::from_file(&seed_path).unwrap();
        assert_eq!(seed.file_path(), seed_path.as_path());
    }

    #[test]
    fn test_from_file_not_found() {
        let result = FamilySeed::from_file("/nonexistent/.family.seed");
        assert!(result.is_err());
        assert!(matches!(result, Err(SporeError::SeedFileNotFound(_))));
    }

    #[test]
    fn test_from_file_wrong_size() {
        let temp_dir = TempDir::new().unwrap();
        let seed_path = temp_dir.path().join(".family.seed");

        // Create file with wrong size
        let bytes = [42u8; 16]; // Only 16 bytes, not 32
        fs::write(&seed_path, bytes).unwrap();

        let result = FamilySeed::from_file(&seed_path);
        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(SporeError::InvalidSeedLength {
                expected: 32,
                found: 16
            })
        ));
    }

    #[test]
    fn test_crypto_provider_env() {
        let temp_dir = TempDir::new().unwrap();
        let seed_path = temp_dir.path().join(".family.seed");

        let seed = FamilySeed::generate_and_write(&seed_path).unwrap();
        let (key, value) = seed.crypto_provider_env().unwrap();

        assert_eq!(key, "FAMILY_SEED_FILE");
        assert_eq!(value, seed_path.to_str().unwrap());
    }

    // ========== Sibling Derivation Tests ==========

    #[test]
    fn test_derive_sibling() {
        let temp_dir = TempDir::new().unwrap();
        let parent_path = temp_dir.path().join("parent.seed");
        let child_path = temp_dir.path().join("child.seed");

        // Generate parent
        FamilySeed::generate_genesis(&parent_path).expect("generate parent");

        // Derive sibling
        let child =
            FamilySeed::derive_sibling(&parent_path, &child_path, "node-alpha", Some("2026-01-08"))
                .expect("derive sibling");

        assert_eq!(child.file_path(), child_path.as_path());

        // Verify child is 32 bytes
        let metadata = fs::metadata(&child_path).expect("child metadata");
        assert_eq!(metadata.len(), 32);

        // Verify child is different from parent
        let parent_bytes = fs::read(&parent_path).expect("read parent");
        let child_bytes = fs::read(&child_path).expect("read child");
        assert_ne!(parent_bytes, child_bytes);
    }

    #[test]
    fn test_derive_sibling_deterministic() {
        let temp_dir = TempDir::new().unwrap();
        let parent_path = temp_dir.path().join("parent.seed");
        let child1_path = temp_dir.path().join("child1.seed");
        let child2_path = temp_dir.path().join("child2.seed");

        // Write known parent seed
        let parent_seed = [42u8; 32];
        fs::write(&parent_path, parent_seed).expect("write parent");

        // Derive same sibling twice
        FamilySeed::derive_sibling(&parent_path, &child1_path, "node-alpha", Some("batch1"))
            .expect("derive child1");
        FamilySeed::derive_sibling(&parent_path, &child2_path, "node-alpha", Some("batch1"))
            .expect("derive child2");

        let child1 = fs::read(&child1_path).expect("read child1");
        let child2 = fs::read(&child2_path).expect("read child2");

        // Same inputs = same output
        assert_eq!(child1, child2);
    }

    #[test]
    fn test_derive_sibling_different_node_ids() {
        let temp_dir = TempDir::new().unwrap();
        let parent_path = temp_dir.path().join("parent.seed");
        let child1_path = temp_dir.path().join("child1.seed");
        let child2_path = temp_dir.path().join("child2.seed");

        let parent_seed = [42u8; 32];
        fs::write(&parent_path, parent_seed).expect("write parent");

        FamilySeed::derive_sibling(&parent_path, &child1_path, "node-alpha", Some("batch1"))
            .expect("derive child1");
        FamilySeed::derive_sibling(&parent_path, &child2_path, "node-beta", Some("batch1"))
            .expect("derive child2");

        let child1 = fs::read(&child1_path).expect("read child1");
        let child2 = fs::read(&child2_path).expect("read child2");

        // Different node_id = different seed (siblings are unique)
        assert_ne!(child1, child2);
    }

    #[test]
    fn test_derive_sibling_different_batches() {
        let temp_dir = TempDir::new().unwrap();
        let parent_path = temp_dir.path().join("parent.seed");
        let child1_path = temp_dir.path().join("child1.seed");
        let child2_path = temp_dir.path().join("child2.seed");

        let parent_seed = [42u8; 32];
        fs::write(&parent_path, parent_seed).expect("write parent");

        FamilySeed::derive_sibling(&parent_path, &child1_path, "node-alpha", Some("batch1"))
            .expect("derive child1");
        FamilySeed::derive_sibling(&parent_path, &child2_path, "node-alpha", Some("batch2"))
            .expect("derive child2");

        let child1 = fs::read(&child1_path).expect("read child1");
        let child2 = fs::read(&child2_path).expect("read child2");

        // Different batch = different seed
        assert_ne!(child1, child2);
    }

    #[test]
    fn test_derive_sibling_no_batch() {
        let temp_dir = TempDir::new().unwrap();
        let parent_path = temp_dir.path().join("parent.seed");
        let child_path = temp_dir.path().join("child.seed");

        let parent_seed = [42u8; 32];
        fs::write(&parent_path, parent_seed).expect("write parent");

        let child = FamilySeed::derive_sibling(&parent_path, &child_path, "node-alpha", None)
            .expect("derive child");

        assert_eq!(child.file_path(), child_path.as_path());
        let metadata = fs::metadata(&child_path).expect("child metadata");
        assert_eq!(metadata.len(), 32);
    }

    #[test]
    fn test_derive_sibling_wrong_parent_size() {
        let temp_dir = TempDir::new().unwrap();
        let parent_path = temp_dir.path().join("parent.seed");
        let child_path = temp_dir.path().join("child.seed");

        // Write wrong-size parent
        fs::write(&parent_path, [0u8; 16]).expect("write parent");

        let result = FamilySeed::derive_sibling(&parent_path, &child_path, "node", Some("batch"));
        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(SporeError::InvalidSeedLength {
                expected: 32,
                found: 16
            })
        ));
    }

    #[test]
    fn test_derive_sibling_parent_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let parent_path = temp_dir.path().join("nonexistent.seed");
        let child_path = temp_dir.path().join("child.seed");

        let result = FamilySeed::derive_sibling(&parent_path, &child_path, "node", Some("batch"));
        assert!(result.is_err());
    }

    // ========== Genesis Tests ==========

    #[test]
    fn test_generate_genesis_unique() {
        let temp_dir = TempDir::new().unwrap();
        let seed1_path = temp_dir.path().join("seed1");
        let seed2_path = temp_dir.path().join("seed2");

        FamilySeed::generate_genesis(&seed1_path).expect("genesis 1");
        FamilySeed::generate_genesis(&seed2_path).expect("genesis 2");

        let seed1 = fs::read(&seed1_path).expect("read seed1");
        let seed2 = fs::read(&seed2_path).expect("read seed2");

        // Two genesis seeds should be different (random)
        assert_ne!(seed1, seed2);
    }
}
