// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

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
            max_age: 31536000,
            include_subdomains: true,
            preload: true,
        };
        assert_eq!(config.max_age, 31536000);
        assert!(config.preload);
    }
}
