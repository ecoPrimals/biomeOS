//! Family seed file management
//!
//! **IMPORTANT**: This module only handles FILE operations.
//! All cryptographic processing is delegated to BearDog.
//!
//! ## Responsibility Boundary
//!
//! ### biomeOS (This Module)
//! - Generate entropy (random bytes)
//! - Write seed file to disk
//! - Set file permissions
//! - Verify file exists and has correct size
//! - Provide file path to BearDog
//!
//! ### BearDog (Security Primal)
//! - Read seed file contents
//! - HKDF-SHA256 key derivation
//! - Extract family ID
//! - Generate child keys
//! - Zeroize sensitive data
//! - All cryptographic operations

use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{SporeError, SporeResult};
use tracing::{debug, info};

/// Family seed file manager
///
/// This struct represents a family seed FILE (not the cryptographic content).
/// biomeOS manages the file, BearDog processes the crypto.
#[derive(Debug, Clone)]
pub struct FamilySeed {
    /// Path to the seed file
    file_path: PathBuf,
}

impl FamilySeed {
    /// Generate random entropy and write to file
    ///
    /// **Note**: This generates 256 bits of cryptographically secure random
    /// bytes and writes them to disk. It does NOT perform any cryptographic
    /// processing - that's BearDog's job.
    ///
    /// # Security
    ///
    /// - Uses OS-level cryptographic RNG (`rand::thread_rng()`)
    /// - Sets file permissions to 0600 (owner read/write only) on Unix
    /// - Does NOT process or derive keys from the seed
    ///
    /// # Arguments
    ///
    /// * `path` - Where to write the seed file (typically `.family.seed`)
    pub fn generate_and_write<P: AsRef<Path>>(path: P) -> SporeResult<Self> {
        use rand::RngCore;

        let path = path.as_ref().to_path_buf();
        info!("Generating family seed file at: {}", path.display());

        // Generate 256 bits of entropy (32 bytes)
        let mut bytes = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut bytes);

        debug!("Generated 32 bytes of entropy");

        // Write to file
        fs::write(&path, bytes)?;
        info!("Wrote seed to file: {}", path.display());

        // Set secure permissions on Unix
        #[cfg(unix)]
        Self::set_secure_permissions(&path)?;

        Ok(Self { file_path: path })
    }

    /// Load existing seed file
    ///
    /// Verifies the file exists and is 32 bytes, but does NOT read or
    /// process the contents. BearDog will read the file when needed.
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
    /// This path should be passed to BearDog via the
    /// `BEARDOG_FAMILY_SEED_FILE` environment variable.
    pub fn file_path(&self) -> &Path {
        &self.file_path
    }

    /// Configure environment for BearDog to use this seed
    ///
    /// Sets the `BEARDOG_FAMILY_SEED_FILE` environment variable so BearDog
    /// knows where to find the seed file. BearDog will then read and process
    /// the file using its cryptographic functions.
    pub fn configure_beardog_env(&self) -> SporeResult<()> {
        let path_str = self
            .file_path
            .to_str()
            .ok_or(SporeError::InvalidPath)?;

        std::env::set_var("BEARDOG_FAMILY_SEED_FILE", path_str);
        debug!("Set BEARDOG_FAMILY_SEED_FILE={}", path_str);

        Ok(())
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
        fs::write(&seed_path, &bytes).unwrap();

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
        fs::write(&seed_path, &bytes).unwrap();

        let result = FamilySeed::from_file(&seed_path);
        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(SporeError::InvalidSeedLength { expected: 32, found: 16 })
        ));
    }

    #[test]
    fn test_configure_beardog_env() {
        let temp_dir = TempDir::new().unwrap();
        let seed_path = temp_dir.path().join(".family.seed");

        let seed = FamilySeed::generate_and_write(&seed_path).unwrap();
        seed.configure_beardog_env().unwrap();

        let env_value = std::env::var("BEARDOG_FAMILY_SEED_FILE").unwrap();
        assert_eq!(env_value, seed_path.to_str().unwrap());
    }
}

