// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::engine::SocketDiscovery;
use tempfile::TempDir;

#[tokio::test]
async fn test_discover_via_socket_registry_structure() {
    let temp_dir = TempDir::new().unwrap();
    let biomeos_dir = temp_dir.path().join("biomeos");
    std::fs::create_dir_all(&biomeos_dir).unwrap();

    let registry = serde_json::json!({
        "version": "1.0",
        "entries": [{
            "primal": "registry-primal",
            "socket": "/tmp/registry-primal.sock",
            "capabilities": ["discovery"]
        }]
    });
    std::fs::write(
        biomeos_dir.join("socket-registry.json"),
        serde_json::to_string_pretty(&registry).unwrap(),
    )
    .unwrap();

    let discovery = SocketDiscovery::new("test").with_xdg_override(temp_dir.path());
    let _result = discovery.discover_primal("registry-primal").await;
}

#[tokio::test]
async fn test_discover_via_socket_registry_invalid_json_skips() {
    let temp = TempDir::new().unwrap();
    let biomeos = temp.path().join("biomeos");
    std::fs::create_dir_all(&biomeos).unwrap();
    std::fs::write(biomeos.join("socket-registry.json"), "{ not json").unwrap();

    let discovery = SocketDiscovery::new("test").with_xdg_override(temp.path());
    assert!(discovery.discover_primal("only-registry").await.is_none());
}

#[tokio::test]
async fn test_discover_via_registry_nonexistent_socket() {
    let strategy = super::super::strategy::DiscoveryStrategy {
        check_env_hints: false,
        use_xdg_runtime: false,
        use_family_tmp: false,
        query_registry: true,
        enable_tcp_fallback: false,
        ..Default::default()
    };
    let discovery = SocketDiscovery::with_strategy("test", strategy)
        .with_neural_api(std::path::PathBuf::from("/nonexistent/neural-api.sock"));

    let result = discovery.discover_primal("registry-only-primal").await;
    assert!(result.is_none());
}

#[tokio::test]
async fn test_discover_capability_via_registry_fails_gracefully() {
    let strategy = super::super::strategy::DiscoveryStrategy {
        check_env_hints: false,
        use_xdg_runtime: false,
        use_family_tmp: false,
        query_registry: true,
        enable_tcp_fallback: false,
        ..Default::default()
    };
    let discovery = SocketDiscovery::with_strategy("test", strategy)
        .with_neural_api(std::path::PathBuf::from("/nonexistent/capability-registry.sock"));

    let result = discovery
        .discover_capability("nonexistent-capability")
        .await;
    assert!(result.is_none());
}
