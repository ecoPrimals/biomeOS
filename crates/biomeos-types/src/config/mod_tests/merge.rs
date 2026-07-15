// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::common::test_env_config;
use crate::config::*;

#[test]
fn test_merge_basic() -> Result<(), Box<dyn std::error::Error>> {
    let mut base = BiomeOSConfig::default();
    let other = BiomeOSConfig::builder()
        .version("3.0.0")
        .port(9090)
        .build()?;

    base.merge(other)?;
    assert_eq!(base.metadata.version, "3.0.0");
    assert_eq!(base.network.port, 9090);
    Ok(())
}

#[test]
fn test_merge_preserves_metadata_custom() -> Result<(), Box<dyn std::error::Error>> {
    let mut base = BiomeOSConfig::default();
    base.metadata
        .custom
        .insert("base_key".to_string(), serde_json::json!("base_value"));

    let mut other = BiomeOSConfig::default();
    other
        .metadata
        .custom
        .insert("other_key".to_string(), serde_json::json!("other_value"));

    base.merge(other)?;
    assert!(base.metadata.custom.contains_key("base_key"));
    assert!(base.metadata.custom.contains_key("other_key"));
    Ok(())
}

#[test]
fn test_merge_environments() -> Result<(), Box<dyn std::error::Error>> {
    let mut base = BiomeOSConfig::default();
    base.environments
        .insert("dev".to_string(), test_env_config());

    let mut other = BiomeOSConfig::default();
    other
        .environments
        .insert("prod".to_string(), test_env_config());

    base.merge(other)?;
    assert!(base.environments.contains_key("dev"));
    assert!(base.environments.contains_key("prod"));
    Ok(())
}

#[test]
fn test_merge_invalid_result_fails() {
    let mut base = BiomeOSConfig::default();
    let mut other = BiomeOSConfig::default();
    other.network.port = 0; // Invalid

    let result = base.merge(other);
    assert!(result.is_err());
}

#[test]
fn test_merge_updates_modified_at() -> Result<(), Box<dyn std::error::Error>> {
    let mut base = BiomeOSConfig::default();
    let original_modified = base.metadata.modified_at;

    let other = BiomeOSConfig::default();
    base.merge(other)?;

    assert!(base.metadata.modified_at >= original_modified);
    Ok(())
}

#[test]
fn test_merge_invalidates_on_bad_config() {
    let mut base = BiomeOSConfig::default();
    let mut other = BiomeOSConfig::default();
    other.system.timeouts.default_request_timeout = std::time::Duration::from_secs(0);
    let result = base.merge(other);
    assert!(result.is_err());
}
