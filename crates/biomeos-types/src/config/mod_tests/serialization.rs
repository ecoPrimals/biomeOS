// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use crate::config::*;
use std::collections::HashMap;

#[test]
fn test_config_serde_json_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
    let config = BiomeOSConfig::builder()
        .name("json-test")
        .port(4321)
        .debug(true)
        .build()?;

    let json = serde_json::to_string(&config)?;
    let parsed: BiomeOSConfig = serde_json::from_str(&json)?;

    assert_eq!(parsed.metadata.name, "json-test");
    assert_eq!(parsed.network.port, 4321);
    assert!(parsed.features.debug);
    Ok(())
}

#[test]
fn test_config_serde_yaml_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
    let config = BiomeOSConfig::builder()
        .name("yaml-test")
        .version("2.0.0")
        .build()?;

    let yaml = serde_yaml::to_string(&config)?;
    let parsed: BiomeOSConfig = serde_yaml::from_str(&yaml)?;

    assert_eq!(parsed.metadata.name, "yaml-test");
    assert_eq!(parsed.metadata.version, "2.0.0");
    Ok(())
}

#[test]
fn test_partial_yaml_deserialize() -> Result<(), Box<dyn std::error::Error>> {
    let yaml = r#"
metadata:
  version: "1.0.0"
  name: "test-config"
"#;
    let config: BiomeOSConfig = serde_yaml::from_str(yaml)?;
    assert_eq!(config.metadata.version, "1.0.0");
    assert_eq!(config.metadata.name, "test-config");
    Ok(())
}

#[test]
fn test_empty_yaml_uses_defaults() -> Result<(), Box<dyn std::error::Error>> {
    let yaml = "{}";
    let config: BiomeOSConfig = serde_yaml::from_str(yaml)?;
    assert_eq!(config.metadata.version, "1.0.0");
    assert_eq!(config.metadata.name, "default-biome-config");
    Ok(())
}

#[test]
fn test_partial_json_uses_defaults() -> Result<(), Box<dyn std::error::Error>> {
    // Only metadata section — other sections use #[serde(default)]
    let json = r#"{"metadata": {"version": "2.0.0"}}"#;
    let config: BiomeOSConfig = serde_json::from_str(json)?;
    assert_eq!(config.metadata.version, "2.0.0");
    // Network should have defaults
    assert!(config.network.port > 0);
    Ok(())
}

#[test]
fn test_metadata_serde_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
    let mut metadata = ConfigMetadata {
        description: Some("Test description".to_string()),
        author: Some("test-author".to_string()),
        tags: vec!["tag1".to_string(), "tag2".to_string()],
        ..Default::default()
    };
    metadata
        .custom
        .insert("key".to_string(), serde_json::json!({"nested": true}));

    let json = serde_json::to_string(&metadata)?;
    let parsed: ConfigMetadata = serde_json::from_str(&json)?;

    assert_eq!(parsed.description, Some("Test description".to_string()));
    assert_eq!(parsed.author, Some("test-author".to_string()));
    assert_eq!(parsed.tags.len(), 2);
    assert!(parsed.custom.contains_key("key"));
    Ok(())
}

#[test]
fn test_config_metadata_serde_with_optionals() -> Result<(), Box<dyn std::error::Error>> {
    let metadata = ConfigMetadata {
        version: "2.0.0".to_string(),
        name: "custom".to_string(),
        description: Some("desc".to_string()),
        author: Some("author".to_string()),
        created_at: chrono::Utc::now(),
        modified_at: chrono::Utc::now(),
        tags: vec!["a".to_string(), "b".to_string()],
        custom: HashMap::new(),
    };
    let json = serde_json::to_string(&metadata)?;
    let parsed: ConfigMetadata = serde_json::from_str(&json)?;
    assert_eq!(parsed.description, Some("desc".to_string()));
    assert_eq!(parsed.tags.len(), 2);
    Ok(())
}
