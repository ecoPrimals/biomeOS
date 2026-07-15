// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use crate::config::*;

#[test]
fn test_biomeos_config_default() {
    let config = BiomeOSConfig::default();
    assert!(!config.metadata.version.is_empty());
    assert!(!config.metadata.name.is_empty());
    assert_eq!(config.metadata.version, "1.0.0");
    assert_eq!(config.metadata.name, "default-biome-config");
}

#[test]
fn test_config_metadata_default() {
    let metadata = ConfigMetadata::default();
    assert_eq!(metadata.version, "1.0.0");
    assert_eq!(metadata.name, "default-biome-config");
    assert!(metadata.description.is_none());
    assert!(metadata.author.is_none());
    assert!(metadata.tags.is_empty());
    assert!(metadata.custom.is_empty());
}

#[test]
fn test_config_metadata_timestamps() {
    let before = chrono::Utc::now();
    let metadata = ConfigMetadata::default();
    let after = chrono::Utc::now();

    assert!(metadata.created_at >= before);
    assert!(metadata.created_at <= after);
    assert!(metadata.modified_at >= before);
    assert!(metadata.modified_at <= after);
}

#[test]
fn test_config_clone() {
    let config = BiomeOSConfig::default();
    let cloned = config.clone();
    assert_eq!(config.metadata.version, cloned.metadata.version);
    assert_eq!(config.metadata.name, cloned.metadata.name);
    assert_eq!(config.network.port, cloned.network.port);
}

#[test]
fn test_config_debug() {
    let config = BiomeOSConfig::default();
    let debug_str = format!("{config:?}");
    assert!(debug_str.contains("BiomeOSConfig"));
    assert!(debug_str.contains("metadata"));
}

#[test]
fn test_config_metadata_default_version_and_name() {
    let metadata = ConfigMetadata::default();
    assert_eq!(metadata.version, "1.0.0");
    assert_eq!(metadata.name, "default-biome-config");
}
