// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! TLS and data-in-transit configuration.
//!
//! Contains configuration for TLS versions, cipher suites, and HSTS.

use serde::{Deserialize, Serialize};

/// Data in transit encryption configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataInTransitConfig {
    /// Enable encryption
    pub enabled: bool,

    /// Minimum TLS version
    pub min_tls_version: String,

    /// Cipher suites
    pub cipher_suites: Vec<String>,
}

impl Default for DataInTransitConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            min_tls_version: "1.2".to_string(),
            cipher_suites: vec![],
        }
    }
}

/// HSTS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HstsConfig {
    /// Max age in seconds
    pub max_age: u64,

    /// Include subdomains
    pub include_subdomains: bool,

    /// Preload
    pub preload: bool,
}

#[expect(
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_in_transit_default() {
        let config = DataInTransitConfig::default();
        assert!(config.enabled);
        assert_eq!(config.min_tls_version, "1.2");
        assert!(config.cipher_suites.is_empty());
    }

    #[test]
    fn test_hsts_config_creation() {
        let config = HstsConfig {
            max_age: 31_536_000,
            include_subdomains: true,
            preload: true,
        };
        assert_eq!(config.max_age, 31_536_000);
        assert!(config.preload);
    }

    #[test]
    fn test_data_in_transit_serde_roundtrip() {
        let config = DataInTransitConfig {
            enabled: false,
            min_tls_version: "1.3".to_string(),
            cipher_suites: vec!["TLS_AES_256_GCM_SHA384".to_string()],
        };
        let json = serde_json::to_string(&config).expect("serialize");
        let deserialized: DataInTransitConfig = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(config.enabled, deserialized.enabled);
        assert_eq!(config.min_tls_version, deserialized.min_tls_version);
        assert_eq!(config.cipher_suites, deserialized.cipher_suites);
    }

    #[test]
    fn test_hsts_config_serde_roundtrip() {
        let config = HstsConfig {
            max_age: 86400,
            include_subdomains: false,
            preload: false,
        };
        let json = serde_json::to_string(&config).expect("serialize");
        let deserialized: HstsConfig = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(config.max_age, deserialized.max_age);
        assert_eq!(config.include_subdomains, deserialized.include_subdomains);
        assert_eq!(config.preload, deserialized.preload);
    }
}
