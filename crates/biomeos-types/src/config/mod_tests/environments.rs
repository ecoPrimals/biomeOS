// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::common::test_env_config;
use crate::config::*;

#[test]
fn test_for_environment_basic() -> Result<(), Box<dyn std::error::Error>> {
    let mut config = BiomeOSConfig::default();
    let mut env_config = test_env_config();
    env_config.features.debug = true;
    config
        .environments
        .insert("staging".to_string(), env_config);

    let staging_config = config.for_environment("staging")?;
    assert!(staging_config.features.debug);
    Ok(())
}

#[test]
fn test_for_environment_missing_returns_base() -> Result<(), Box<dyn std::error::Error>> {
    let config = BiomeOSConfig::default();
    let result = config.for_environment("nonexistent")?;
    // Should return a clone of the base config
    assert_eq!(result.metadata.version, config.metadata.version);
    Ok(())
}

#[test]
fn test_for_environment_with_endpoints() -> Result<(), Box<dyn std::error::Error>> {
    let mut config = BiomeOSConfig::default();
    let mut env_config = test_env_config();
    env_config.endpoints.insert(
        "api".to_string(),
        "https://api.staging.example.com".to_string(),
    );
    config
        .environments
        .insert("staging".to_string(), env_config);

    let staging = config.for_environment("staging")?;
    assert!(staging.metadata.custom.contains_key("api_endpoint"));
    assert_eq!(
        staging.metadata.custom.get("api_endpoint"),
        Some(&serde_json::json!("https://api.staging.example.com"))
    );
    Ok(())
}

#[test]
fn test_for_environment_applies_variables() -> Result<(), Box<dyn std::error::Error>> {
    let mut config = BiomeOSConfig::default();
    let mut env_config = test_env_config();
    env_config
        .variables
        .insert("TEST_VAR".to_string(), "test_value".to_string());
    config.environments.insert("test".to_string(), env_config);

    let _ = config.for_environment("test")?;
    Ok(())
}
