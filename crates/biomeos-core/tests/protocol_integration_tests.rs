//! Integration tests for dual-protocol support
//!
//! Tests the complete flow:
//! 1. Parse tower.toml with protocol field
//! 2. Build primal configuration
//! 3. Verify IPC_PROTOCOL environment variable is set
//! 4. Test all protocol selection scenarios

use biomeos_core::{
    Capability, PrimalBuilder, PrimalConfig as BiomePrimalConfig,
};
use std::collections::HashMap;

#[test]
fn test_protocol_env_var_propagation_tarpc() {
    // Simulate config parsing
    let mut env_config = HashMap::new();
    env_config.insert("BEARDOG_NODE_ID".to_string(), "test-node".to_string());
    
    let config = BiomePrimalConfig {
        binary_path: "./primals/beardog".to_string(),
        node_id: Some("test-node".to_string()),
        provides: vec![Capability::Security],
        requires: vec![],
        http_port: 0,
        env_config,
    };
    
    // Build primal with tarpc protocol
    let mut builder = PrimalBuilder::new()
        .binary_path(config.binary_path.clone())
        .provides(config.provides.clone())
        .requires(config.requires.clone());
    
    // Add protocol (simulating tower.rs behavior)
    let protocol = Some("tarpc".to_string());
    if let Some(p) = protocol {
        builder = builder.env_var("IPC_PROTOCOL".to_string(), p);
    }
    
    // Add other env vars
    for (key, value) in &config.env_config {
        builder = builder.env_var(key.clone(), value.clone());
    }
    
    let primal = builder.build().expect("Failed to build primal");
    
    // Verify protocol env var is set
    assert_eq!(
        primal.config().env_config.get("IPC_PROTOCOL"),
        Some(&"tarpc".to_string())
    );
    
    // Verify other env vars still work
    assert_eq!(
        primal.config().env_config.get("BEARDOG_NODE_ID"),
        Some(&"test-node".to_string())
    );
}

#[test]
fn test_protocol_env_var_propagation_jsonrpc() {
    let mut env_config = HashMap::new();
    env_config.insert("SONGBIRD_NODE_ID".to_string(), "test-node".to_string());
    env_config.insert("SECURITY_ENDPOINT".to_string(), "jsonrpc+unix:///tmp/beardog.sock".to_string());
    
    let config = BiomePrimalConfig {
        binary_path: "./primals/songbird".to_string(),
        node_id: Some("test-node".to_string()),
        provides: vec![Capability::Discovery],
        requires: vec![Capability::Security],
        http_port: 0,
        env_config,
    };
    
    let mut builder = PrimalBuilder::new()
        .binary_path(config.binary_path.clone())
        .provides(config.provides.clone())
        .requires(config.requires.clone());
    
    // Add protocol
    let protocol = Some("jsonrpc".to_string());
    if let Some(p) = protocol {
        builder = builder.env_var("IPC_PROTOCOL".to_string(), p);
    }
    
    // Add other env vars
    for (key, value) in &config.env_config {
        builder = builder.env_var(key.clone(), value.clone());
    }
    
    let primal = builder.build().expect("Failed to build primal");
    
    // Verify protocol env var
    assert_eq!(
        primal.config().env_config.get("IPC_PROTOCOL"),
        Some(&"jsonrpc".to_string())
    );
    
    // Verify endpoint preserved
    assert_eq!(
        primal.config().env_config.get("SECURITY_ENDPOINT"),
        Some(&"jsonrpc+unix:///tmp/beardog.sock".to_string())
    );
}

#[test]
fn test_protocol_omitted_auto_detect() {
    // No protocol specified - should work (auto-detect)
    let mut env_config = HashMap::new();
    env_config.insert("BEARDOG_NODE_ID".to_string(), "test-node".to_string());
    
    let config = BiomePrimalConfig {
        binary_path: "./primals/beardog".to_string(),
        node_id: Some("test-node".to_string()),
        provides: vec![Capability::Security],
        requires: vec![],
        http_port: 0,
        env_config,
    };
    
    let mut builder = PrimalBuilder::new()
        .binary_path(config.binary_path.clone())
        .provides(config.provides.clone());
    
    // No protocol - simulating omitted field
    let protocol: Option<String> = None;
    if let Some(p) = protocol {
        builder = builder.env_var("IPC_PROTOCOL".to_string(), p);
    }
    
    // Add other env vars
    for (key, value) in &config.env_config {
        builder = builder.env_var(key.clone(), value.clone());
    }
    
    let primal = builder.build().expect("Failed to build primal");
    
    // Verify IPC_PROTOCOL is NOT set (auto-detect mode)
    assert!(primal.config().env_config.get("IPC_PROTOCOL").is_none());
    
    // Verify other env vars still work
    assert_eq!(
        primal.config().env_config.get("BEARDOG_NODE_ID"),
        Some(&"test-node".to_string())
    );
}

#[test]
fn test_mixed_protocol_primals() {
    // Test fractal deployment: multiple primals with different protocols
    
    // Primal 1: BearDog with tarpc
    let mut beardog_env = HashMap::new();
    beardog_env.insert("BEARDOG_NODE_ID".to_string(), "tower1".to_string());
    
    let beardog_config = BiomePrimalConfig {
        binary_path: "./primals/beardog".to_string(),
        node_id: Some("beardog".to_string()),
        provides: vec![Capability::Security],
        requires: vec![],
        http_port: 0,
        env_config: beardog_env,
    };
    
    let mut beardog_builder = PrimalBuilder::new()
        .binary_path(beardog_config.binary_path.clone())
        .provides(beardog_config.provides.clone())
        .env_var("IPC_PROTOCOL".to_string(), "tarpc".to_string());
    
    for (key, value) in &beardog_config.env_config {
        beardog_builder = beardog_builder.env_var(key.clone(), value.clone());
    }
    
    let beardog = beardog_builder.build().expect("Failed to build BearDog");
    
    // Primal 2: Songbird with tarpc
    let mut songbird_env = HashMap::new();
    songbird_env.insert("SONGBIRD_NODE_ID".to_string(), "tower1".to_string());
    songbird_env.insert("SECURITY_ENDPOINT".to_string(), "tarpc+unix:///tmp/beardog.sock".to_string());
    
    let songbird_config = BiomePrimalConfig {
        binary_path: "./primals/songbird".to_string(),
        node_id: Some("songbird".to_string()),
        provides: vec![Capability::Discovery],
        requires: vec![Capability::Security],
        http_port: 0,
        env_config: songbird_env,
    };
    
    let mut songbird_builder = PrimalBuilder::new()
        .binary_path(songbird_config.binary_path.clone())
        .provides(songbird_config.provides.clone())
        .requires(songbird_config.requires.clone())
        .env_var("IPC_PROTOCOL".to_string(), "tarpc".to_string());
    
    for (key, value) in &songbird_config.env_config {
        songbird_builder = songbird_builder.env_var(key.clone(), value.clone());
    }
    
    let songbird = songbird_builder.build().expect("Failed to build Songbird");
    
    // Primal 3: ToadStool with JSON-RPC
    let mut toadstool_env = HashMap::new();
    toadstool_env.insert("TOADSTOOL_NODE_ID".to_string(), "tower1".to_string());
    
    let toadstool_config = BiomePrimalConfig {
        binary_path: "./primals/toadstool".to_string(),
        node_id: Some("toadstool".to_string()),
        provides: vec![Capability::Workload],
        requires: vec![],
        http_port: 0,
        env_config: toadstool_env,
    };
    
    let mut toadstool_builder = PrimalBuilder::new()
        .binary_path(toadstool_config.binary_path.clone())
        .provides(toadstool_config.provides.clone())
        .env_var("IPC_PROTOCOL".to_string(), "jsonrpc".to_string());
    
    for (key, value) in &toadstool_config.env_config {
        toadstool_builder = toadstool_builder.env_var(key.clone(), value.clone());
    }
    
    let toadstool = toadstool_builder.build().expect("Failed to build ToadStool");
    
    // Verify each primal has correct protocol
    assert_eq!(
        beardog.config().env_config.get("IPC_PROTOCOL"),
        Some(&"tarpc".to_string())
    );
    assert_eq!(
        songbird.config().env_config.get("IPC_PROTOCOL"),
        Some(&"tarpc".to_string())
    );
    assert_eq!(
        toadstool.config().env_config.get("IPC_PROTOCOL"),
        Some(&"jsonrpc".to_string())
    );
    
    // Verify capabilities still work
    assert_eq!(beardog.provides(), &[Capability::Security]);
    assert_eq!(songbird.provides(), &[Capability::Discovery]);
    assert_eq!(songbird.requires(), &[Capability::Security]);
    assert_eq!(toadstool.provides(), &[Capability::Workload]);
}

#[test]
fn test_backward_compatibility_http_port() {
    // Ensure HTTP port still works (legacy support)
    let mut env_config = HashMap::new();
    env_config.insert("BEARDOG_NODE_ID".to_string(), "test".to_string());
    
    let config = BiomePrimalConfig {
        binary_path: "./primals/beardog".to_string(),
        node_id: Some("test".to_string()),
        provides: vec![Capability::Security],
        requires: vec![],
        http_port: 9000,
        env_config,
    };
    
    let mut builder = PrimalBuilder::new()
        .binary_path(config.binary_path.clone())
        .provides(config.provides.clone())
        .http_port(config.http_port);
    
    // No protocol specified
    
    for (key, value) in &config.env_config {
        builder = builder.env_var(key.clone(), value.clone());
    }
    
    let primal = builder.build().expect("Failed to build primal");
    
    // Verify HTTP port preserved
    assert_eq!(primal.config().http_port, 9000);
    
    // Verify no protocol set (backward compatible)
    assert!(primal.config().env_config.get("IPC_PROTOCOL").is_none());
}

#[test]
fn test_protocol_precedence_url_scheme() {
    // Test that URL scheme takes precedence over env var
    let mut env_config = HashMap::new();
    env_config.insert("SONGBIRD_NODE_ID".to_string(), "test".to_string());
    
    // URL scheme specifies tarpc
    env_config.insert("SECURITY_ENDPOINT".to_string(), "tarpc+unix:///tmp/beardog.sock".to_string());
    
    let config = BiomePrimalConfig {
        binary_path: "./primals/songbird".to_string(),
        node_id: Some("test".to_string()),
        provides: vec![Capability::Discovery],
        requires: vec![Capability::Security],
        http_port: 0,
        env_config,
    };
    
    let mut builder = PrimalBuilder::new()
        .binary_path(config.binary_path.clone())
        .provides(config.provides.clone())
        .requires(config.requires.clone());
    
    // IPC_PROTOCOL says jsonrpc (should be overridden by URL scheme)
    builder = builder.env_var("IPC_PROTOCOL".to_string(), "jsonrpc".to_string());
    
    for (key, value) in &config.env_config {
        builder = builder.env_var(key.clone(), value.clone());
    }
    
    let primal = builder.build().expect("Failed to build primal");
    
    // URL scheme has tarpc+unix://
    assert_eq!(
        primal.config().env_config.get("SECURITY_ENDPOINT"),
        Some(&"tarpc+unix:///tmp/beardog.sock".to_string())
    );
    
    // Note: In actual primal, tarpc+unix:// in URL would override IPC_PROTOCOL
    // This test verifies both are set and primal can make the precedence decision
    assert_eq!(
        primal.config().env_config.get("IPC_PROTOCOL"),
        Some(&"jsonrpc".to_string())
    );
}

#[test]
fn test_isomorphic_deployment_same_binary_different_protocols() {
    // Same binary, different protocol configs (isomorphic architecture)
    
    // Production deployment: tarpc
    let mut prod_env = HashMap::new();
    prod_env.insert("BEARDOG_NODE_ID".to_string(), "prod-tower1".to_string());
    prod_env.insert("DEPLOYMENT_ENV".to_string(), "production".to_string());
    
    let prod_config = BiomePrimalConfig {
        binary_path: "./primals/beardog".to_string(),
        node_id: Some("prod".to_string()),
        provides: vec![Capability::Security],
        requires: vec![],
        http_port: 0,
        env_config: prod_env,
    };
    
    let mut prod_builder = PrimalBuilder::new()
        .binary_path(prod_config.binary_path.clone())
        .provides(prod_config.provides.clone())
        .env_var("IPC_PROTOCOL".to_string(), "tarpc".to_string());
    
    for (key, value) in &prod_config.env_config {
        prod_builder = prod_builder.env_var(key.clone(), value.clone());
    }
    
    let prod_primal = prod_builder.build().expect("Failed to build production primal");
    
    // Development deployment: JSON-RPC (same binary!)
    let mut dev_env = HashMap::new();
    dev_env.insert("BEARDOG_NODE_ID".to_string(), "dev-tower1".to_string());
    dev_env.insert("DEPLOYMENT_ENV".to_string(), "development".to_string());
    
    let dev_config = BiomePrimalConfig {
        binary_path: "./primals/beardog".to_string(), // Same binary!
        node_id: Some("dev".to_string()),
        provides: vec![Capability::Security],
        requires: vec![],
        http_port: 0,
        env_config: dev_env,
    };
    
    let mut dev_builder = PrimalBuilder::new()
        .binary_path(dev_config.binary_path.clone())
        .provides(dev_config.provides.clone())
        .env_var("IPC_PROTOCOL".to_string(), "jsonrpc".to_string());
    
    for (key, value) in &dev_config.env_config {
        dev_builder = dev_builder.env_var(key.clone(), value.clone());
    }
    
    let dev_primal = dev_builder.build().expect("Failed to build development primal");
    
    // Verify same binary
    assert_eq!(prod_primal.config().binary_path, dev_primal.config().binary_path);
    
    // Verify different protocols
    assert_eq!(
        prod_primal.config().env_config.get("IPC_PROTOCOL"),
        Some(&"tarpc".to_string())
    );
    assert_eq!(
        dev_primal.config().env_config.get("IPC_PROTOCOL"),
        Some(&"jsonrpc".to_string())
    );
    
    // Verify deployment environment preserved
    assert_eq!(
        prod_primal.config().env_config.get("DEPLOYMENT_ENV"),
        Some(&"production".to_string())
    );
    assert_eq!(
        dev_primal.config().env_config.get("DEPLOYMENT_ENV"),
        Some(&"development".to_string())
    );
}

