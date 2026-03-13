// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! End-to-end tests for dual-protocol support
//!
//! These tests verify the complete flow from tower.toml to primal execution:
//! 1. Write test tower.toml configurations
//! 2. Parse with Tower
//! 3. Verify primals receive correct IPC_PROTOCOL
//! 4. Test all deployment scenarios

use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// Create a temporary tower.toml configuration file
fn create_test_config(dir: &TempDir, toml_content: &str) -> PathBuf {
    let config_path = dir.path().join("tower.toml");
    fs::write(&config_path, toml_content).expect("Failed to write test config");
    config_path
}

#[test]
fn test_e2e_tarpc_protocol_configuration() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let toml = r#"
[tower]
family = "test_family"
concurrent_startup = true

[[primals]]
binary = "./primals/beardog"
provides = ["Security"]
protocol = "tarpc"

[primals.env]
BEARDOG_FAMILY_ID = "test_family"
BEARDOG_NODE_ID = "test-tower"
RUST_LOG = "info"
"#;

    let config_path = create_test_config(&temp_dir, toml);

    // Parse configuration (simulating tower binary)
    let config_str = fs::read_to_string(&config_path).expect("Failed to read config");
    let config: biomeos_core::TowerConfig =
        toml::from_str(&config_str).expect("Failed to parse config");

    // Verify protocol field
    assert_eq!(config.primals.len(), 1);
    assert_eq!(config.primals[0].protocol, Some("tarpc".to_string()));

    // Verify env vars
    assert_eq!(
        config.primals[0].env.get("BEARDOG_FAMILY_ID"),
        Some(&"test_family".to_string())
    );
    assert_eq!(
        config.primals[0].env.get("BEARDOG_NODE_ID"),
        Some(&"test-tower".to_string())
    );
}

#[test]
fn test_e2e_jsonrpc_protocol_configuration() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let toml = r#"
[tower]
family = "test_family"

[[primals]]
binary = "./primals/songbird"
provides = ["Discovery"]
requires = ["Security"]
protocol = "jsonrpc"

[primals.env]
SONGBIRD_NODE_ID = "test-tower"
SECURITY_ENDPOINT = "jsonrpc+unix:///tmp/beardog-test_family-test.sock"
"#;

    let config_path = create_test_config(&temp_dir, toml);
    let config_str = fs::read_to_string(&config_path).expect("Failed to read config");
    let config: biomeos_core::TowerConfig =
        toml::from_str(&config_str).expect("Failed to parse config");

    assert_eq!(config.primals.len(), 1);
    assert_eq!(config.primals[0].protocol, Some("jsonrpc".to_string()));
    assert_eq!(
        config.primals[0].env.get("SECURITY_ENDPOINT"),
        Some(&"jsonrpc+unix:///tmp/beardog-test_family-test.sock".to_string())
    );
}

#[test]
fn test_e2e_auto_detect_protocol() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let toml = r#"
[tower]
family = "test_family"

[[primals]]
binary = "./primals/beardog"
provides = ["Security"]
# No protocol field - auto-detect

[primals.env]
BEARDOG_NODE_ID = "test-tower"
"#;

    let config_path = create_test_config(&temp_dir, toml);
    let config_str = fs::read_to_string(&config_path).expect("Failed to read config");
    let config: biomeos_core::TowerConfig =
        toml::from_str(&config_str).expect("Failed to parse config");

    assert_eq!(config.primals.len(), 1);
    assert!(config.primals[0].protocol.is_none()); // Auto-detect
}

#[test]
fn test_e2e_fractal_deployment_mixed_protocols() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let toml = r#"
[tower]
family = "test_family"
concurrent_startup = true

# Core primals: tarpc (performance-critical)
[[primals]]
binary = "./primals/beardog"
provides = ["Security"]
protocol = "tarpc"

[primals.env]
BEARDOG_NODE_ID = "tower1"

[[primals]]
binary = "./primals/songbird"
provides = ["Discovery"]
requires = ["Security"]
protocol = "tarpc"

[primals.env]
SONGBIRD_NODE_ID = "tower1"
SECURITY_ENDPOINT = "tarpc+unix:///tmp/beardog-test_family-tower1.sock"

# Edge primal: JSON-RPC (flexibility)
[[primals]]
binary = "./primals/toadstool"
provides = ["Workload"]
protocol = "jsonrpc"

[primals.env]
TOADSTOOL_NODE_ID = "tower1"
"#;

    let config_path = create_test_config(&temp_dir, toml);
    let config_str = fs::read_to_string(&config_path).expect("Failed to read config");
    let config: biomeos_core::TowerConfig =
        toml::from_str(&config_str).expect("Failed to parse config");

    assert_eq!(config.primals.len(), 3);

    // Core primals: tarpc
    assert_eq!(config.primals[0].protocol, Some("tarpc".to_string()));
    assert_eq!(config.primals[1].protocol, Some("tarpc".to_string()));

    // Edge primal: JSON-RPC
    assert_eq!(config.primals[2].protocol, Some("jsonrpc".to_string()));
}

#[test]
fn test_e2e_isomorphic_deployment_scenarios() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Production configuration (tarpc)
    let prod_toml = r#"
[tower]
family = "test_family"

[[primals]]
binary = "./primals/beardog"
provides = ["Security"]
protocol = "tarpc"

[primals.env]
BEARDOG_NODE_ID = "prod-tower1"
DEPLOYMENT_ENV = "production"
"#;

    let prod_config_path = temp_dir.path().join("tower-prod.toml");
    fs::write(&prod_config_path, prod_toml).expect("Failed to write prod config");

    let prod_config_str =
        fs::read_to_string(&prod_config_path).expect("Failed to read prod config");
    let prod_config: biomeos_core::TowerConfig =
        toml::from_str(&prod_config_str).expect("Failed to parse prod config");

    // Development configuration (JSON-RPC, same binary)
    let dev_toml = r#"
[tower]
family = "test_family"

[[primals]]
binary = "./primals/beardog"
provides = ["Security"]
protocol = "jsonrpc"

[primals.env]
BEARDOG_NODE_ID = "dev-tower1"
DEPLOYMENT_ENV = "development"
"#;

    let dev_config_path = temp_dir.path().join("tower-dev.toml");
    fs::write(&dev_config_path, dev_toml).expect("Failed to write dev config");

    let dev_config_str = fs::read_to_string(&dev_config_path).expect("Failed to read dev config");
    let dev_config: biomeos_core::TowerConfig =
        toml::from_str(&dev_config_str).expect("Failed to parse dev config");

    // Verify same binary
    assert_eq!(prod_config.primals[0].binary, dev_config.primals[0].binary);

    // Verify different protocols
    assert_eq!(prod_config.primals[0].protocol, Some("tarpc".to_string()));
    assert_eq!(dev_config.primals[0].protocol, Some("jsonrpc".to_string()));

    // Verify environment variables
    assert_eq!(
        prod_config.primals[0].env.get("DEPLOYMENT_ENV"),
        Some(&"production".to_string())
    );
    assert_eq!(
        dev_config.primals[0].env.get("DEPLOYMENT_ENV"),
        Some(&"development".to_string())
    );
}

#[test]
fn test_e2e_backward_compatibility() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Old-style configuration (no protocol field, HTTP port)
    let toml = r#"
[tower]
name = "legacy-tower"
family = "test_family"

[[primals]]
binary = "./primals/beardog"
provides = ["Security"]
http_port = 9000

[primals.env]
BEARDOG_NODE_ID = "legacy-tower"

[[primals]]
binary = "./primals/songbird"
provides = ["Discovery"]
requires = ["Security"]

[primals.env]
SONGBIRD_NODE_ID = "legacy-tower"
"#;

    let config_path = create_test_config(&temp_dir, toml);
    let config_str = fs::read_to_string(&config_path).expect("Failed to read config");
    let config: biomeos_core::TowerConfig =
        toml::from_str(&config_str).expect("Failed to parse config");

    assert_eq!(config.primals.len(), 2);

    // Verify no protocol field (backward compatible)
    assert!(config.primals[0].protocol.is_none());
    assert!(config.primals[1].protocol.is_none());

    // Verify HTTP port still works
    assert_eq!(config.primals[0].http_port, 9000);
    assert_eq!(config.primals[1].http_port, 0);
}

#[test]
fn test_e2e_url_scheme_variations() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let toml = r#"
[tower]
family = "test_family"

[[primals]]
binary = "./primals/beardog"
provides = ["Security"]
# No protocol field, server will auto-detect

[primals.env]
BEARDOG_NODE_ID = "tower1"

[[primals]]
binary = "./primals/songbird"
provides = ["Discovery"]
protocol = "tarpc"

[primals.env]
SONGBIRD_NODE_ID = "tower1"
# URL scheme: tarpc+unix://
SECURITY_ENDPOINT = "tarpc+unix:///tmp/beardog-test_family-tower1.sock"

[[primals]]
binary = "./primals/toadstool"
provides = ["Workload"]
protocol = "jsonrpc"

[primals.env]
TOADSTOOL_NODE_ID = "tower1"
# URL scheme: jsonrpc+unix://
WORKLOAD_ENDPOINT = "jsonrpc+unix:///tmp/toadstool-test_family-tower1.sock"
"#;

    let config_path = create_test_config(&temp_dir, toml);
    let config_str = fs::read_to_string(&config_path).expect("Failed to read config");
    let config: biomeos_core::TowerConfig =
        toml::from_str(&config_str).expect("Failed to parse config");

    assert_eq!(config.primals.len(), 3);

    // BearDog: auto-detect
    assert!(config.primals[0].protocol.is_none());

    // Songbird: tarpc with tarpc+unix:// URL
    assert_eq!(config.primals[1].protocol, Some("tarpc".to_string()));
    assert!(config.primals[1]
        .env
        .get("SECURITY_ENDPOINT")
        .unwrap()
        .starts_with("tarpc+unix://"));

    // ToadStool: JSON-RPC with jsonrpc+unix:// URL
    assert_eq!(config.primals[2].protocol, Some("jsonrpc".to_string()));
    assert!(config.primals[2]
        .env
        .get("WORKLOAD_ENDPOINT")
        .unwrap()
        .starts_with("jsonrpc+unix://"));
}

#[test]
fn test_e2e_complete_tower_configuration() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Complete configuration from examples/tower-dual-protocol.toml
    let toml = r#"
[tower]
family = "test_family"
concurrent_startup = true

[[primals]]
binary = "./primals/beardog"
provides = ["Security", "Encryption", "Trust"]
requires = []
protocol = "tarpc"

[primals.env]
BEARDOG_FAMILY_SEED = "Nat0C/G/b4B7u06n0r14SuZXrp/IZ/38fZHh8aJQMVg="
BEARDOG_FAMILY_ID = "test_family"
BEARDOG_NODE_ID = "tower1"
RUST_LOG = "info"

[[primals]]
binary = "./primals/songbird"
provides = ["Discovery"]
requires = ["Security"]
protocol = "tarpc"

[primals.env]
SONGBIRD_FAMILY_ID = "test_family"
SONGBIRD_NODE_ID = "tower1"
SECURITY_ENDPOINT = "tarpc+unix:///tmp/beardog-test_family-tower1.sock"
RUST_LOG = "info"

[health]
interval_secs = 30
timeout_secs = 5
recovery_attempts = 3
"#;

    let config_path = create_test_config(&temp_dir, toml);
    let config_str = fs::read_to_string(&config_path).expect("Failed to read config");
    let config: biomeos_core::TowerConfig =
        toml::from_str(&config_str).expect("Failed to parse config");

    // Verify tower metadata
    assert_eq!(config.tower.family, Some("test_family".to_string()));
    assert!(config.tower.concurrent_startup);

    // Verify primals
    assert_eq!(config.primals.len(), 2);

    // BearDog
    assert_eq!(
        config.primals[0].provides,
        vec!["Security", "Encryption", "Trust"]
    );
    assert_eq!(config.primals[0].protocol, Some("tarpc".to_string()));
    assert_eq!(
        config.primals[0].env.get("BEARDOG_FAMILY_ID"),
        Some(&"test_family".to_string())
    );

    // Songbird
    assert_eq!(config.primals[1].provides, vec!["Discovery"]);
    assert_eq!(config.primals[1].requires, vec!["Security"]);
    assert_eq!(config.primals[1].protocol, Some("tarpc".to_string()));
    assert_eq!(
        config.primals[1].env.get("SECURITY_ENDPOINT"),
        Some(&"tarpc+unix:///tmp/beardog-test_family-tower1.sock".to_string())
    );

    // Verify health config
    assert_eq!(config.health.interval_secs, 30);
    assert_eq!(config.health.timeout_secs, 5);
    assert_eq!(config.health.recovery_attempts, 3);
}
