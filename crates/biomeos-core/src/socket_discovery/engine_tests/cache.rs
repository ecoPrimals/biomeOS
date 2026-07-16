// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::engine::SocketDiscovery;
use super::super::result::{DiscoveredSocket, DiscoveryMethod};
use super::super::strategy::DiscoveryStrategy;
use super::super::transport::TransportEndpoint;
use std::collections::HashMap;
use std::path::PathBuf;
use tempfile::TempDir;

#[tokio::test]
async fn test_discover_endpoint_via_env_unix() {
    let temp_dir = TempDir::new().unwrap();
    let socket_path = temp_dir.path().join("beardog.sock");
    std::fs::File::create(&socket_path).unwrap();

    let env_overrides: HashMap<String, String> = [(
        "BEARDOG_SOCKET".to_string(),
        socket_path.to_string_lossy().to_string(),
    )]
    .into();
    let discovery = SocketDiscovery::new("test");
    let result = discovery.discover_endpoint_via_env_with("beardog", Some(&env_overrides));

    assert!(result.is_some());
}

#[tokio::test]
async fn test_get_xdg_runtime_dir() {
    // May or may not be set in test environment
    let _xdg_dir = SocketDiscovery::get_xdg_runtime_dir();
    // Just verify it doesn't panic
}

#[tokio::test]
async fn test_get_neural_api_socket() {
    let discovery = SocketDiscovery::new("test");

    // May or may not exist
    let _socket = discovery.get_neural_api_socket();
    // Just verify it doesn't panic
}

#[tokio::test]
async fn test_get_neural_api_socket_from_env() {
    let temp_dir = TempDir::new().unwrap();
    let socket_path = temp_dir.path().join("neural.sock");
    std::fs::File::create(&socket_path).unwrap();

    let discovery = SocketDiscovery::new("test");
    let result = discovery.get_neural_api_socket_with(Some(socket_path.as_path()));

    assert!(result.is_some());
}

#[tokio::test]
async fn test_cache_functionality() {
    let discovery = SocketDiscovery::new("test");

    let socket = DiscoveredSocket::from_unix_path(
        PathBuf::from("/tmp/test.sock"),
        DiscoveryMethod::FamilyTmp,
    )
    .with_primal_name("test")
    .with_capabilities(vec!["test".to_string()]);

    discovery.cache_socket("test:key", &socket).await;

    let cached = discovery.check_cache("test:key").await;
    assert!(cached.is_some());
    assert_eq!(cached.unwrap().discovered_via, DiscoveryMethod::Cached);

    discovery.clear_cache().await;
    let cleared = discovery.check_cache("test:key").await;
    assert!(cleared.is_none());
}

#[tokio::test(start_paused = true)]
async fn test_cache_ttl_expiration() {
    let strategy = DiscoveryStrategy {
        cache_ttl_secs: 1,
        ..Default::default()
    };
    let discovery = SocketDiscovery::with_strategy("test", strategy);

    let socket = DiscoveredSocket::from_unix_path(
        PathBuf::from("/tmp/test.sock"),
        DiscoveryMethod::FamilyTmp,
    );

    discovery.cache_socket("test:key", &socket).await;

    assert!(discovery.check_cache("test:key").await.is_some());

    tokio::time::advance(tokio::time::Duration::from_secs(2)).await;

    assert!(discovery.check_cache("test:key").await.is_none());
}

#[tokio::test]
async fn test_cache_disabled() {
    let strategy = DiscoveryStrategy {
        enable_cache: false,
        ..Default::default()
    };
    let discovery = SocketDiscovery::with_strategy("test", strategy);

    let socket = DiscoveredSocket::from_unix_path(
        PathBuf::from("/tmp/test.sock"),
        DiscoveryMethod::FamilyTmp,
    );

    discovery.cache_socket("test:key", &socket).await;

    // Cache should not be used
    assert!(discovery.check_cache("test:key").await.is_none());
}
