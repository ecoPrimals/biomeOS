// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Encryption and key management configuration.
//!
//! Contains configuration for data-at-rest encryption, key storage,
//! and key derivation.

use serde::{Deserialize, Serialize};
use std::time::Duration;

use super::tls::DataInTransitConfig;

/// Encryption configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EncryptionConfig {
    /// Data at rest encryption
    #[serde(default)]
    pub at_rest: DataAtRestConfig,

    /// Data in transit encryption
    #[serde(default)]
    pub in_transit: DataInTransitConfig,

    /// Key management
    #[serde(default)]
    pub key_management: KeyManagementConfig,
}

/// Data at rest encryption configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataAtRestConfig {
    /// Enable encryption
    pub enabled: bool,

    /// Encryption algorithm
    pub algorithm: EncryptionAlgorithm,

    /// Key rotation interval
    pub key_rotation_interval: Duration,
}

impl Default for DataAtRestConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            algorithm: EncryptionAlgorithm::AES256GCM,
            key_rotation_interval: Duration::from_secs(30 * 24 * 60 * 60), // 30 days
        }
    }
}

/// Encryption algorithms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EncryptionAlgorithm {
    /// AES-256 in GCM mode (authenticated encryption)
    AES256GCM,
    /// AES-256 in CBC mode
    AES256CBC,
    /// ChaCha20-Poly1305 (authenticated encryption)
    ChaCha20Poly1305,
    /// Custom encryption algorithm
    Custom(String),
}

/// Key management configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyManagementConfig {
    /// Key storage backend
    pub backend: KeyStorageBackend,

    /// Master key ID
    pub master_key_id: Option<String>,

    /// Key derivation configuration
    pub derivation: KeyDerivationConfig,
}

impl Default for KeyManagementConfig {
    fn default() -> Self {
        Self {
            backend: KeyStorageBackend::Local,
            master_key_id: None,
            derivation: KeyDerivationConfig::default(),
        }
    }
}

/// Key storage backends
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KeyStorageBackend {
    /// Local filesystem storage
    Local,
    /// HashiCorp Vault
    Vault,
    /// Hardware Security Module
    Hsm,
    /// Cloud Key Management Service
    Kms,
    /// Custom key storage
    Custom(String),
}

/// Key derivation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyDerivationConfig {
    /// Derivation algorithm
    pub algorithm: KeyDerivationAlgorithm,

    /// Iteration count
    pub iterations: u32,

    /// Salt length
    pub salt_length: usize,
}

impl Default for KeyDerivationConfig {
    fn default() -> Self {
        Self {
            algorithm: KeyDerivationAlgorithm::PBKDF2,
            iterations: 100000,
            salt_length: 32,
        }
    }
}

/// Key derivation algorithms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KeyDerivationAlgorithm {
    /// PBKDF2 (Password-Based Key Derivation Function 2)
    PBKDF2,
    /// Scrypt memory-hard function
    Scrypt,
    /// Argon2 memory-hard function
    Argon2,
    /// Custom key derivation
    Custom(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_config_default() {
        let config = EncryptionConfig::default();
        assert!(!config.at_rest.enabled);
        assert!(config.in_transit.enabled);
    }

    #[test]
    fn test_data_at_rest_default() {
        let config = DataAtRestConfig::default();
        assert!(!config.enabled);
        assert!(matches!(config.algorithm, EncryptionAlgorithm::AES256GCM));
        assert_eq!(
            config.key_rotation_interval,
            Duration::from_secs(30 * 24 * 60 * 60)
        );
    }

    #[test]
    fn test_key_management_default() {
        let config = KeyManagementConfig::default();
        assert!(matches!(config.backend, KeyStorageBackend::Local));
        assert!(config.master_key_id.is_none());
    }

    #[test]
    fn test_key_derivation_default() {
        let config = KeyDerivationConfig::default();
        assert!(matches!(config.algorithm, KeyDerivationAlgorithm::PBKDF2));
        assert_eq!(config.iterations, 100000);
        assert_eq!(config.salt_length, 32);
    }

    #[test]
    fn test_encryption_algorithm_serialization() {
        for alg in [
            EncryptionAlgorithm::AES256GCM,
            EncryptionAlgorithm::AES256CBC,
            EncryptionAlgorithm::ChaCha20Poly1305,
            EncryptionAlgorithm::Custom("xchacha20".to_string()),
        ] {
            let json = serde_json::to_string(&alg).expect("serialize");
            let _: EncryptionAlgorithm = serde_json::from_str(&json).expect("deserialize");
        }
    }

    #[test]
    fn test_key_storage_backend_serialization() {
        for backend in [
            KeyStorageBackend::Local,
            KeyStorageBackend::Vault,
            KeyStorageBackend::Hsm,
            KeyStorageBackend::Kms,
            KeyStorageBackend::Custom("sops".to_string()),
        ] {
            let json = serde_json::to_string(&backend).expect("serialize");
            let _: KeyStorageBackend = serde_json::from_str(&json).expect("deserialize");
        }
    }

    #[test]
    fn test_key_derivation_algorithm_serialization() {
        for alg in [
            KeyDerivationAlgorithm::PBKDF2,
            KeyDerivationAlgorithm::Scrypt,
            KeyDerivationAlgorithm::Argon2,
            KeyDerivationAlgorithm::Custom("hkdf".to_string()),
        ] {
            let json = serde_json::to_string(&alg).expect("serialize");
            let _: KeyDerivationAlgorithm = serde_json::from_str(&json).expect("deserialize");
        }
    }

    #[test]
    fn test_encryption_algorithm_debug() {
        let alg = EncryptionAlgorithm::ChaCha20Poly1305;
        let debug = format!("{alg:?}");
        assert!(debug.contains("ChaCha20"));
    }
}
