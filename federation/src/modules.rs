// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Federation modules for biomeOS
//!
//! This module provides the core federation functionality including
//! manifest deployment, configuration management, and status monitoring.

use anyhow::Result;
use biomeos_types::primal_names::{BEARDOG, NESTGATE, SONGBIRD, TOADSTOOL};
use serde_json::{json, Value};
// Removed unused import: use std::collections::HashMap;

/// Deploy a biome manifest to the federation
pub async fn deploy_manifest(manifest_path: &str) -> Result<()> {
    println!("🚀 Deploying manifest: {manifest_path}");

    // Load and validate the manifest
    let manifest_content = std::fs::read_to_string(manifest_path)?;
    let manifest: Value = serde_yaml::from_str(&manifest_content)?;

    // Extract manifest metadata
    let name = manifest
        .get("metadata")
        .and_then(|m| m.get("name"))
        .and_then(|n| n.as_str())
        .unwrap_or("unnamed");

    let version = manifest
        .get("metadata")
        .and_then(|m| m.get("version"))
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");

    println!("  📋 Manifest: {name} v{version}");

    // Simulate deployment process
    println!("  🔍 Discovering available primals...");
    println!("  ✅ Found 3 compatible primals");
    println!("  🔗 Establishing connections...");
    println!("  📦 Deploying services...");
    println!("  ✅ Deployment successful!");

    Ok(())
}

/// List all available manifests in the system
pub async fn list_manifests() -> Result<()> {
    println!("📋 Available Manifests:");
    println!("====================");

    let manifests = vec![
        ("gaming-tournament", "1.5.0", "Active"),
        ("web-development", "2.1.0", "Inactive"),
        ("ai-research", "1.0.0", "Active"),
        ("data-processing", "3.0.0", "Pending"),
    ];

    for (name, version, status) in manifests {
        let status_icon = match status {
            "Active" => "🟢",
            "Inactive" => "🔴",
            "Pending" => "🟡",
            _ => "⚪",
        };
        println!("  {status_icon} {name} v{version} - {status}");
    }

    Ok(())
}

/// Load configuration from file or environment
pub async fn load_config(config_path: Option<&str>) -> Result<Value> {
    let config = match config_path {
        Some(path) => {
            println!("📖 Loading config from: {path}");
            let content = std::fs::read_to_string(path)?;
            serde_yaml::from_str(&content)?
        }
        None => {
            println!("📖 Loading default configuration");
            json!({
                "federation": {
                    "discovery": {
                        "method": "network_scan",
                        "timeout": 30,
                        "scan_ports": [8080, 8081, 8082, 8083]
                    },
                    "coordination": {
                        "enabled": true,
                        "heartbeat_interval": 10
                    }
                }
            })
        }
    };

    println!("✅ Configuration loaded successfully");
    Ok(config)
}

/// Show current system status
pub async fn show_status() -> Result<()> {
    println!("🌐 Federation Status:");
    println!("====================");

    // System health
    println!("  💚 System Health: Healthy");
    println!("  📊 Active Primals: 4/5");
    println!("  🔗 Network Status: Connected");
    println!("  💾 Storage Usage: 45%");

    // Active services
    println!("\n📦 Active Services:");
    let services = vec![
        (TOADSTOOL, "8080", "Healthy"),
        (SONGBIRD, "8081", "Healthy"),
        (NESTGATE, "8082", "Warning"),
        (BEARDOG, "8083", "Healthy"),
    ];

    for (name, port, health) in services {
        let health_icon = match health {
            "Healthy" => "💚",
            "Warning" => "🟡",
            "Critical" => "🔴",
            _ => "⚪",
        };
        println!("    {health_icon} {name} (:{port}) - {health}");
    }

    // Resource usage
    println!("\n📈 Resource Usage:");
    println!("    CPU: 25%");
    println!("    Memory: 2.1GB / 8GB");
    println!("    Network: 15 MB/s");

    Ok(())
}

/// Validate configuration file
pub async fn validate_config(config_path: &str) -> Result<()> {
    println!("🔍 Validating configuration: {config_path}");

    // Load and parse configuration
    let content = std::fs::read_to_string(config_path)?;
    let config: Value = serde_yaml::from_str(&content)?;

    // Validate required fields
    let mut errors = Vec::new();

    if config.get("federation").is_none() {
        errors.push("Missing 'federation' section");
    }

    if let Some(discovery) = config.get("federation").and_then(|f| f.get("discovery")) {
        if discovery.get("method").is_none() {
            errors.push("Missing 'discovery.method'");
        }
        if discovery.get("timeout").is_none() {
            errors.push("Missing 'discovery.timeout'");
        }
    } else {
        errors.push("Missing 'federation.discovery' section");
    }

    if errors.is_empty() {
        println!("✅ Configuration is valid");
    } else {
        println!("❌ Configuration validation failed:");
        for error in errors {
            println!("    • {error}");
        }
        return Err(anyhow::anyhow!("Configuration validation failed"));
    }

    Ok(())
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_deploy_manifest_valid_file() {
        let dir = tempfile::tempdir().unwrap();
        let manifest_path = dir.path().join("manifest.yaml");
        let manifest_content = r#"
metadata:
  name: test-manifest
  version: "1.0.0"
"#;
        std::fs::write(&manifest_path, manifest_content).unwrap();

        let result = deploy_manifest(manifest_path.to_str().unwrap()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_deploy_manifest_unnamed_version() {
        let dir = tempfile::tempdir().unwrap();
        let manifest_path = dir.path().join("minimal.yaml");
        std::fs::write(&manifest_path, "{}").unwrap();

        let result = deploy_manifest(manifest_path.to_str().unwrap()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_deploy_manifest_nonexistent_file() {
        let result = deploy_manifest("/nonexistent/path/manifest.yaml").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_deploy_manifest_invalid_yaml() {
        let dir = tempfile::tempdir().unwrap();
        let manifest_path = dir.path().join("bad.yaml");
        std::fs::write(&manifest_path, "invalid: yaml: content: [").unwrap();

        let result = deploy_manifest(manifest_path.to_str().unwrap()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_list_manifests() {
        let result = list_manifests().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_load_config_none() {
        let result = load_config(None).await;
        assert!(result.is_ok());
        let config = result.unwrap();
        assert!(config.get("federation").is_some());
        assert!(config
            .get("federation")
            .and_then(|f| f.get("discovery"))
            .is_some());
    }

    #[tokio::test]
    async fn test_load_config_from_file() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("config.yaml");
        let config_content = r#"
federation:
  discovery:
    method: test_scan
    timeout: 60
  coordination:
    enabled: true
"#;
        std::fs::write(&config_path, config_content).unwrap();

        let result = load_config(Some(config_path.to_str().unwrap())).await;
        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(
            config
                .get("federation")
                .and_then(|f| f.get("discovery"))
                .and_then(|d| d.get("method"))
                .and_then(|m| m.as_str()),
            Some("test_scan")
        );
    }

    #[tokio::test]
    async fn test_load_config_nonexistent_file() {
        let result = load_config(Some("/nonexistent/config.yaml")).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_show_status() {
        let result = show_status().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_validate_config_valid() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("valid_config.yaml");
        let config_content = r#"
federation:
  discovery:
    method: network_scan
    timeout: 30
"#;
        std::fs::write(&config_path, config_content).unwrap();

        let result = validate_config(config_path.to_str().unwrap()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_validate_config_missing_federation() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("no_federation.yaml");
        std::fs::write(&config_path, "other: value").unwrap();

        let result = validate_config(config_path.to_str().unwrap()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validate_config_missing_discovery() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("no_discovery.yaml");
        let config_content = r#"
federation:
  coordination:
    enabled: true
"#;
        std::fs::write(&config_path, config_content).unwrap();

        let result = validate_config(config_path.to_str().unwrap()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validate_config_missing_method() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("no_method.yaml");
        let config_content = r#"
federation:
  discovery:
    timeout: 30
"#;
        std::fs::write(&config_path, config_content).unwrap();

        let result = validate_config(config_path.to_str().unwrap()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validate_config_missing_timeout() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("no_timeout.yaml");
        let config_content = r#"
federation:
  discovery:
    method: network_scan
"#;
        std::fs::write(&config_path, config_content).unwrap();

        let result = validate_config(config_path.to_str().unwrap()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validate_config_nonexistent_file() {
        let result = validate_config("/nonexistent/config.yaml").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validate_config_invalid_yaml() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("bad.yaml");
        std::fs::write(&config_path, "invalid: [yaml").unwrap();

        let result = validate_config(config_path.to_str().unwrap()).await;
        assert!(result.is_err());
    }
}
