// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

#![allow(clippy::unwrap_used, clippy::expect_used)]

//! Tests for Universal BiomeOS Manager
//!
//! Basic smoke tests - will expand as implementation grows

#[cfg(test)]
mod manager_basic_tests {
    use crate::universal_biomeos_manager::UniversalBiomeOSManager;
    use biomeos_types::BiomeOSConfig;

    #[tokio::test]
    async fn test_manager_creation_with_config() {
        let config = BiomeOSConfig::default();
        let result = UniversalBiomeOSManager::new(config).await;
        assert!(result.is_ok(), "Manager should create successfully");
    }

    #[tokio::test]
    async fn test_manager_drop() {
        let config = BiomeOSConfig::default();
        {
            let _manager = UniversalBiomeOSManager::new(config).await.unwrap();
        }
        // If we reach here, manager was dropped without panic - test passes
    }

    #[test]
    fn test_config_has_defaults() {
        let config = BiomeOSConfig::default();
        assert!(!config.metadata.version.is_empty());
    }
}

// More tests will be added as we implement functionality
// For now, these verify the basic infrastructure compiles and runs
