//! Family Credentials Management
//!
//! This module provides secure handling of family seeds and credentials for
//! genetic lineage authentication in the biomeOS ecosystem.
//!
//! ## Security Model
//!
//! Family seeds are cryptographic secrets that establish genetic lineage.
//! They must be:
//! - Never logged or printed
//! - Zeroized on drop
//! - Loaded from secure sources only
//! - Validated before use
//!
//! ## Usage
//!
//! ```ignore
//! // Load from environment (development/testing)
//! let creds = FamilyCredentials::from_env()?;
//!
//! // Load from encrypted file (production)
//! let key = load_encryption_key()?;
//! let creds = FamilyCredentials::from_encrypted_file("family.enc", &key)?;
//!
//! // Use credentials
//! let family_id = creds.family_id();
//! let seed = creds.seed_ref(); // Temporary reference, never cloned
//! ```

// Use BirdSongError if available, otherwise use a local error type
#[cfg(feature = "http-transport")]
use crate::adaptive_client::BirdSongError;

#[cfg(not(feature = "http-transport"))]
use CredentialsError as BirdSongError;

use biomeos_types::identifiers::FamilyId;
use serde::Deserialize;
use std::path::Path;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Error type for credentials when HTTP transport is not available
#[cfg(not(feature = "http-transport"))]
#[derive(Debug, thiserror::Error)]
pub enum CredentialsError {
    #[error("Invalid credentials: {0}")]
    InvalidCredentials(String),

    #[error("Missing required credentials: {0}")]
    MissingCredentials(String),
}

/// Secure wrapper for family seed that zeroizes on drop
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct SecretSeed {
    /// Base64-encoded seed bytes
    seed: String,
}

impl SecretSeed {
    /// Create a new secret seed from base64 string
    pub fn new(seed: String) -> Result<Self, BirdSongError> {
        // Validate base64 format
        base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &seed).map_err(|e| {
            BirdSongError::InvalidCredentials(format!("Invalid base64 seed: {}", e))
        })?;

        Ok(Self { seed })
    }

    /// Get a reference to the seed (temporary use only)
    pub fn as_str(&self) -> &str {
        &self.seed
    }

    /// Validate seed format and length
    pub fn validate(&self) -> Result<(), BirdSongError> {
        let decoded =
            base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &self.seed)
                .map_err(|e| BirdSongError::InvalidCredentials(format!("Invalid base64: {}", e)))?;

        // Require minimum seed length (32 bytes = 256 bits)
        if decoded.len() < 32 {
            return Err(BirdSongError::InvalidCredentials(format!(
                "Seed too short: {} bytes (minimum 32)",
                decoded.len()
            )));
        }

        Ok(())
    }
}

impl std::fmt::Debug for SecretSeed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("SecretSeed([REDACTED])")
    }
}

/// Family credentials containing ID and seed
#[derive(Clone)]
pub struct FamilyCredentials {
    family_id: FamilyId,
    seed: SecretSeed,
}

impl FamilyCredentials {
    /// Create new family credentials
    pub fn new(family_id: FamilyId, seed: SecretSeed) -> Result<Self, BirdSongError> {
        let creds = Self { family_id, seed };
        creds.validate()?;
        Ok(creds)
    }

    /// Load credentials from environment variables
    ///
    /// Expects:
    /// - `FAMILY_ID` or `BEARDOG_FAMILY_ID`
    /// - `FAMILY_SEED` or `BEARDOG_FAMILY_SEED`
    pub fn from_env() -> Result<Self, BirdSongError> {
        let family_id_str = std::env::var("FAMILY_ID")
            .or_else(|_| std::env::var("BEARDOG_FAMILY_ID"))
            .map_err(|_| {
                BirdSongError::InvalidCredentials(
                    "Missing FAMILY_ID or BEARDOG_FAMILY_ID environment variable".to_string(),
                )
            })?;

        let family_seed = std::env::var("FAMILY_SEED")
            .or_else(|_| std::env::var("BEARDOG_FAMILY_SEED"))
            .map_err(|_| {
                BirdSongError::InvalidCredentials(
                    "Missing FAMILY_SEED or BEARDOG_FAMILY_SEED environment variable".to_string(),
                )
            })?;

        let family_id = FamilyId::new(family_id_str);

        let seed = SecretSeed::new(family_seed)?;

        Self::new(family_id, seed)
    }

    /// Load credentials from encrypted file (future enhancement)
    ///
    /// **NOTE**: This is a development placeholder. Production systems should implement
    /// age-encrypted files or integrate with system keychains (e.g., `keyring` crate).
    ///
    /// Current implementation loads from plaintext JSON for development only.
    ///
    /// Future implementation should use:
    /// - age encryption (github.com/FiloSottile/age)
    /// - System keychain integration (keyring-rs)
    /// - Environment-based key derivation
    #[allow(dead_code)]
    pub fn from_encrypted_file(
        path: impl AsRef<Path>,
        _encryption_key: &[u8],
    ) -> Result<Self, BirdSongError> {
        // Future: Implement age-encrypted file format
        // For now, load from plaintext JSON (development only)
        let contents = std::fs::read_to_string(path.as_ref()).map_err(|e| {
            BirdSongError::InvalidCredentials(format!("Failed to read file: {}", e))
        })?;

        #[derive(Deserialize)]
        struct FileFormat {
            family_id: String,
            family_seed: String,
        }

        let file: FileFormat = serde_json::from_str(&contents)
            .map_err(|e| BirdSongError::InvalidCredentials(format!("Invalid JSON: {}", e)))?;

        let family_id = FamilyId::new(file.family_id);

        let seed = SecretSeed::new(file.family_seed)?;

        Self::new(family_id, seed)
    }

    /// Get the family ID
    pub fn family_id(&self) -> &FamilyId {
        &self.family_id
    }

    /// Get a reference to the seed (temporary use only)
    pub fn seed_ref(&self) -> &str {
        self.seed.as_str()
    }

    /// Validate credentials
    pub fn validate(&self) -> Result<(), BirdSongError> {
        // Validate family ID format
        if self.family_id.as_str().is_empty() {
            return Err(BirdSongError::InvalidCredentials(
                "Family ID cannot be empty".to_string(),
            ));
        }

        // Validate seed
        self.seed.validate()?;

        Ok(())
    }
}

impl std::fmt::Debug for FamilyCredentials {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FamilyCredentials")
            .field("family_id", &self.family_id)
            .field("seed", &"[REDACTED]")
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secret_seed_validation() {
        // Valid seed (32+ bytes when decoded)
        let valid_seed = "iIDnVX3Tein1LFkrkkq7Wo3wsxPNek9XZqp0VL4Kn88=";
        let seed = SecretSeed::new(valid_seed.to_string()).unwrap();
        assert!(seed.validate().is_ok());

        // Invalid base64
        let invalid_seed = "not-valid-base64!!!";
        assert!(SecretSeed::new(invalid_seed.to_string()).is_err());

        // Too short (less than 32 bytes)
        let short_seed =
            base64::Engine::encode(&base64::engine::general_purpose::STANDARD, b"short");
        let seed = SecretSeed::new(short_seed).unwrap();
        assert!(seed.validate().is_err());
    }

    #[test]
    fn test_family_credentials_validation() {
        let family_id = FamilyId::new("test-family");
        let valid_seed =
            SecretSeed::new("iIDnVX3Tein1LFkrkkq7Wo3wsxPNek9XZqp0VL4Kn88=".to_string()).unwrap();

        let creds = FamilyCredentials::new(family_id, valid_seed).unwrap();
        assert_eq!(creds.family_id().as_str(), "test-family");
        assert!(creds.validate().is_ok());
    }

    #[test]
    fn test_secret_seed_debug() {
        let seed =
            SecretSeed::new("iIDnVX3Tein1LFkrkkq7Wo3wsxPNek9XZqp0VL4Kn88=".to_string()).unwrap();
        let debug_str = format!("{:?}", seed);
        assert!(debug_str.contains("REDACTED"));
        assert!(!debug_str.contains("iIDn"));
    }

    #[test]
    fn test_family_credentials_debug() {
        let family_id = FamilyId::new("test-family");
        let seed =
            SecretSeed::new("iIDnVX3Tein1LFkrkkq7Wo3wsxPNek9XZqp0VL4Kn88=".to_string()).unwrap();
        let creds = FamilyCredentials::new(family_id, seed).unwrap();

        let debug_str = format!("{:?}", creds);
        assert!(debug_str.contains("test-family"));
        assert!(debug_str.contains("REDACTED"));
        assert!(!debug_str.contains("iIDn"));
    }

    #[test]
    #[ignore] // Skip - modifies global env state, not thread-safe for parallel tests
    fn test_from_env_missing() {
        // NOTE: This test modifies global environment, skip in parallel test runs
        // Run with: cargo test test_from_env_missing -- --ignored
        
        // Clear environment
        std::env::remove_var("FAMILY_ID");
        std::env::remove_var("BEARDOG_FAMILY_ID");
        std::env::remove_var("FAMILY_SEED");
        std::env::remove_var("BEARDOG_FAMILY_SEED");

        assert!(FamilyCredentials::from_env().is_err());
    }

    #[test]
    #[ignore] // Skip - modifies global env state, not thread-safe for parallel tests
    fn test_from_env_success() {
        // NOTE: This test modifies global environment, skip in parallel test runs
        // Run with: cargo test test_from_env_success -- --ignored
        
        std::env::set_var("FAMILY_ID", "test-family");
        std::env::set_var(
            "FAMILY_SEED",
            "iIDnVX3Tein1LFkrkkq7Wo3wsxPNek9XZqp0VL4Kn88=",
        );

        let creds = FamilyCredentials::from_env().unwrap();
        assert_eq!(creds.family_id().as_str(), "test-family");

        // Cleanup
        std::env::remove_var("FAMILY_ID");
        std::env::remove_var("FAMILY_SEED");
    }
}
