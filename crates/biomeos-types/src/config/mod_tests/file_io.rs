// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use crate::config::*;

#[test]
fn test_config_file_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("test-config.yaml");

    let config = BiomeOSConfig::builder()
        .name("file-test")
        .version("1.5.0")
        .port(7777)
        .build()?;

    config.to_file(&path)?;
    let loaded = BiomeOSConfig::from_file(&path)?;

    assert_eq!(loaded.metadata.name, "file-test");
    assert_eq!(loaded.metadata.version, "1.5.0");
    assert_eq!(loaded.network.port, 7777);
    Ok(())
}

#[test]
fn test_config_from_file_not_found() {
    let result = BiomeOSConfig::from_file("/nonexistent/path/config.yaml");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Failed to read"));
}

#[test]
fn test_config_from_file_invalid_yaml() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("bad.yaml");
    std::fs::write(&path, "not: valid: yaml: [[")?;

    let result = BiomeOSConfig::from_file(&path);
    assert!(result.is_err());
    Ok(())
}

#[test]
fn test_config_from_file_invalid_values() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("invalid.yaml");
    // Port 0 should fail validation
    std::fs::write(&path, "network:\n  port: 0\n")?;

    let result = BiomeOSConfig::from_file(&path);
    assert!(result.is_err());
    Ok(())
}

#[test]
fn test_to_file_io_error() {
    let config = BiomeOSConfig::default();
    let result = config.to_file("/nonexistent/readonly/path/config.yaml");
    assert!(result.is_err());
}
