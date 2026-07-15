// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::common::test_env_config;
use crate::config::*;

#[test]
fn test_builder_basic() -> Result<(), Box<dyn std::error::Error>> {
    let config = BiomeOSConfig::builder()
        .name("test-config")
        .version("2.0.0")
        .port(8080)
        .build()?;

    assert_eq!(config.metadata.name, "test-config");
    assert_eq!(config.metadata.version, "2.0.0");
    assert_eq!(config.network.port, 8080);
    Ok(())
}

#[test]
fn test_builder_debug_mode() -> Result<(), Box<dyn std::error::Error>> {
    let config = BiomeOSConfig::builder().debug(true).build()?;
    assert!(config.features.debug);
    Ok(())
}

#[test]
fn test_builder_experimental() -> Result<(), Box<dyn std::error::Error>> {
    let config = BiomeOSConfig::builder().experimental(true).build()?;
    assert!(config.features.experimental);
    Ok(())
}

#[test]
fn test_builder_environment() -> Result<(), Box<dyn std::error::Error>> {
    let config = BiomeOSConfig::builder()
        .environment(Environment::Production)
        .build()?;
    assert_eq!(config.system.environment, Environment::Production);
    Ok(())
}

#[test]
fn test_builder_bind_address() -> Result<(), Box<dyn std::error::Error>> {
    let config = BiomeOSConfig::builder().bind_address("0.0.0.0").build()?;
    assert_eq!(config.network.bind_address, "0.0.0.0");
    Ok(())
}

#[test]
fn test_builder_log_level() -> Result<(), Box<dyn std::error::Error>> {
    let config = BiomeOSConfig::builder()
        .log_level(observability::LogLevel::Debug)
        .build()?;
    assert_eq!(format!("{:?}", config.observability.logging.level), "Debug");
    Ok(())
}

#[test]
fn test_builder_add_extension() -> Result<(), Box<dyn std::error::Error>> {
    let config = BiomeOSConfig::builder()
        .add_extension("custom_key", serde_json::json!("custom_value"))
        .build()?;
    assert_eq!(
        config.metadata.custom.get("custom_key"),
        Some(&serde_json::json!("custom_value"))
    );
    Ok(())
}

#[test]
fn test_builder_add_environment() -> Result<(), Box<dyn std::error::Error>> {
    let config = BiomeOSConfig::builder()
        .add_environment("staging", test_env_config())
        .build()?;
    assert!(config.environments.contains_key("staging"));
    Ok(())
}

#[test]
fn test_builder_invalid_port_fails() {
    let result = BiomeOSConfig::builder().port(0).build();
    assert!(result.is_err());
}

#[test]
fn test_builder_default_impl() {
    let builder = BiomeOSConfigBuilder::default();
    let config = builder.build();
    assert!(config.is_ok());
}

#[test]
fn test_builder_version() {
    let config = BiomeOSConfig::builder().version("3.1.0").build().unwrap();
    assert_eq!(config.metadata.version, "3.1.0");
}
