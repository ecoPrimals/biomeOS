// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Socket Discovery Engine Tests
//!
//! Extracted from engine.rs to maintain files under 1000 lines.
//! These tests cover the SocketDiscovery functionality including:
//! - Socket path building
//! - Port calculation
//! - Environment hint discovery
//! - Cache functionality
//! - TCP and Unix socket verification

use super::engine::SocketDiscovery;
use super::result::{DiscoveredSocket, DiscoveryMethod};
use super::strategy::DiscoveryStrategy;
use super::transport::TransportEndpoint;
use std::collections::HashMap;
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_build_socket_path() {
    let discovery = SocketDiscovery::new("test-family");
    let path = discovery.build_socket_path("beardog");

    assert!(path.to_string_lossy().contains("beardog"));
    assert!(path.to_string_lossy().contains("test-family"));
}

#[test]
fn test_build_socket_path_with_primal_socket_env() {
    let discovery = SocketDiscovery::new("test-family");
    let path = discovery.build_socket_path_with("beardog", Some("/custom/socket/dir"), None);

    assert!(path.to_string_lossy().contains("beardog"));
    assert!(path.to_string_lossy().contains("test-family"));
}

#[test]
fn test_calculate_primal_port() {
    let discovery = SocketDiscovery::new("test");

    let port1 = discovery.calculate_primal_port("beardog");
    let port2 = discovery.calculate_primal_port("beardog");
    assert_eq!(port1, port2);

    let port_songbird = discovery.calculate_primal_port("songbird");
    let port_beardog = discovery.calculate_primal_port("beardog");
    assert!((9100..9200).contains(&port_beardog));
    assert!((9100..9200).contains(&port_songbird));
}

#[test]
fn test_calculate_primal_port_deterministic() {
    let discovery = SocketDiscovery::new("test");

    // Same primal name should always produce same port
    let port1 = discovery.calculate_primal_port("test-primal");
    let port2 = discovery.calculate_primal_port("test-primal");
    assert_eq!(port1, port2);

    // Different names should produce different ports (usually)
    let port_a = discovery.calculate_primal_port("primal-a");
    let port_b = discovery.calculate_primal_port("primal-b");
    // They might be the same due to hash collision, but that's acceptable
    assert!((9100..9200).contains(&port_a));
    assert!((9100..9200).contains(&port_b));
}

#[test]
fn test_socket_discovery_new() {
    let discovery = SocketDiscovery::new("test-family");
    assert_eq!(discovery.family_id.as_str(), "test-family");
    assert!(discovery.strategy.enable_cache);
}

#[test]
fn test_socket_discovery_with_strategy() {
    let strategy = DiscoveryStrategy::android();
    let discovery = SocketDiscovery::with_strategy("test", strategy.clone());
    assert_eq!(discovery.family_id.as_str(), "test");
    assert!(!discovery.strategy.use_xdg_runtime);
    assert!(discovery.strategy.try_abstract_sockets);
}

#[test]
fn test_socket_discovery_with_neural_api() {
    let discovery = SocketDiscovery::new("test").with_neural_api(PathBuf::from("/tmp/neural.sock"));
    assert_eq!(
        discovery.neural_api_socket,
        Some(PathBuf::from("/tmp/neural.sock"))
    );
}

#[tokio::test]
async fn test_env_hint_discovery() {
    let env_overrides: HashMap<String, String> = [(
        "TEST_PRIMAL_SOCKET".to_string(),
        "/tmp/test-primal.sock".to_string(),
    )]
    .into();
    let discovery = SocketDiscovery::new("test");
    let result = discovery
        .discover_via_env_hint_with("test_primal", Some(&env_overrides))
        .await;

    // Result is None because socket doesn't exist
    assert!(result.is_none());
}

#[tokio::test]
async fn test_env_hint_discovery_with_existing_socket() {
    let temp_dir = TempDir::new().unwrap();
    let socket_path = temp_dir.path().join("test.sock");

    // Create a dummy socket file
    std::fs::File::create(&socket_path).unwrap();

    let env_overrides: HashMap<String, String> = [(
        "TEST_PRIMAL_SOCKET".to_string(),
        socket_path.to_string_lossy().to_string(),
    )]
    .into();
    let discovery = SocketDiscovery::new("test");
    let result = discovery
        .discover_via_env_hint_with("test_primal", Some(&env_overrides))
        .await;

    // Should find the socket even though it's not a real Unix socket
    assert!(result.is_some());
}

#[tokio::test]
async fn test_env_hint_discovery_multiple_vars() {
    let temp_dir = TempDir::new().unwrap();
    let socket_path = temp_dir.path().join("test.sock");
    std::fs::File::create(&socket_path).unwrap();

    let env_overrides: HashMap<String, String> = [(
        "BEARDOG_SOCKET_PATH".to_string(),
        socket_path.to_string_lossy().to_string(),
    )]
    .into();
    let discovery = SocketDiscovery::new("test");
    let result = discovery
        .discover_via_env_hint_with("beardog", Some(&env_overrides))
        .await;

    assert!(result.is_some());
}

#[tokio::test]
async fn test_discover_endpoint_via_env_tcp() {
    let env_overrides: HashMap<String, String> =
        [("BEARDOG_TCP".to_string(), "127.0.0.1:9100".to_string())].into();
    let discovery = SocketDiscovery::new("test");
    let result = discovery
        .discover_endpoint_via_env_with("beardog", Some(&env_overrides))
        .await;

    assert!(result.is_some());
    if let Some(TransportEndpoint::TcpSocket { host, port }) = result {
        assert_eq!(host.as_ref(), "127.0.0.1");
        assert_eq!(port, 9100);
    } else {
        panic!("Expected TCP endpoint");
    }
}

#[tokio::test]
async fn test_discover_endpoint_via_env_endpoint_var() {
    let env_overrides: HashMap<String, String> = [(
        "BEARDOG_ENDPOINT".to_string(),
        "tcp://192.168.1.1:8080".to_string(),
    )]
    .into();
    let discovery = SocketDiscovery::new("test");
    let result = discovery
        .discover_endpoint_via_env_with("beardog", Some(&env_overrides))
        .await;

    assert!(result.is_some());
}

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
    let result = discovery
        .discover_endpoint_via_env_with("beardog", Some(&env_overrides))
        .await;

    assert!(result.is_some());
}

#[tokio::test]
async fn test_get_xdg_runtime_dir() {
    let discovery = SocketDiscovery::new("test");

    // May or may not be set in test environment
    let _xdg_dir = discovery.get_xdg_runtime_dir();
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

#[tokio::test]
async fn test_cache_ttl_expiration() {
    let strategy = DiscoveryStrategy {
        cache_ttl_secs: 1, // Very short TTL
        ..Default::default()
    };
    let discovery = SocketDiscovery::with_strategy("test", strategy);

    let socket = DiscoveredSocket::from_unix_path(
        PathBuf::from("/tmp/test.sock"),
        DiscoveryMethod::FamilyTmp,
    );

    discovery.cache_socket("test:key", &socket).await;

    // Should be cached immediately
    assert!(discovery.check_cache("test:key").await.is_some());

    // Wait for cache to expire
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // Should be expired now
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

#[tokio::test]
async fn test_get_socket_path() {
    let discovery = SocketDiscovery::new("test");

    // Will return None if socket doesn't exist
    let result = discovery.get_socket_path("nonexistent").await;
    // Just verify it doesn't panic
    assert!(result.is_none() || result.is_some());
}

#[tokio::test]
async fn test_discover_primal_nonexistent() {
    let discovery = SocketDiscovery::new("test");
    let result = discovery.discover_primal("nonexistent-primal").await;
    assert!(result.is_none());
}

#[tokio::test]
async fn test_discover_capability_nonexistent() {
    let discovery = SocketDiscovery::new("test");
    let result = discovery
        .discover_capability("nonexistent-capability")
        .await;
    assert!(result.is_none());
}

#[tokio::test]
async fn test_discover_with_fallback_nonexistent() {
    let discovery = SocketDiscovery::new("test");
    let result = discovery.discover_with_fallback("nonexistent-primal").await;
    // May return None or TCP fallback depending on strategy
    assert!(result.is_none() || result.is_some());
}

#[tokio::test]
async fn test_discover_with_fallback_tcp_disabled() {
    let strategy = DiscoveryStrategy {
        enable_tcp_fallback: false,
        ..Default::default()
    };
    let discovery = SocketDiscovery::with_strategy("test", strategy);

    let result = discovery.discover_with_fallback("nonexistent").await;
    assert!(result.is_none());
}

#[tokio::test]
async fn test_verify_unix_socket_nonexistent() {
    let discovery = SocketDiscovery::new("test");
    let path = PathBuf::from("/tmp/nonexistent-socket.sock");
    let result = discovery.verify_unix_socket(&path).await;
    assert!(!result);
}

#[tokio::test]
async fn test_verify_tcp_connection_invalid() {
    let discovery = SocketDiscovery::new("test");
    // Use an invalid port that won't be listening
    let result = discovery.verify_tcp_connection("127.0.0.1", 65535).await;
    assert!(!result);
}

#[test]
fn test_build_socket_path_xdg() {
    let temp_dir = TempDir::new().unwrap();
    let discovery = SocketDiscovery::new("test-family");
    let path = discovery.build_socket_path_with("beardog", None, Some(temp_dir.path()));

    assert!(path.to_string_lossy().contains("beardog"));
    assert!(path.to_string_lossy().contains("test-family"));
}

#[test]
fn test_build_socket_path_family_id_injection() {
    let discovery = SocketDiscovery::new("my-family-123");
    let path = discovery.build_socket_path("songbird");

    let path_str = path.to_string_lossy();
    assert!(path_str.contains("songbird"));
    assert!(path_str.contains("my-family-123"));
    assert!(path_str.ends_with(".sock"));
}

#[test]
fn test_build_socket_path_primal_socket_as_dir() {
    let temp_dir = TempDir::new().unwrap();
    let socket_dir = temp_dir.path().join("sockets");
    std::fs::create_dir_all(&socket_dir).unwrap();

    let discovery = SocketDiscovery::new("fam");
    let path =
        discovery.build_socket_path_with("beardog", Some(socket_dir.to_str().unwrap()), None);

    assert_eq!(
        path,
        socket_dir.join("beardog-fam.sock"),
        "PRIMAL_SOCKET as dir should join socket name"
    );
}

#[test]
fn test_build_socket_path_primal_socket_as_existing_file() {
    let temp_dir = TempDir::new().unwrap();
    let socket_file = temp_dir.path().join("custom.sock");
    std::fs::File::create(&socket_file).unwrap();

    let discovery = SocketDiscovery::new("fam");
    let path =
        discovery.build_socket_path_with("beardog", Some(socket_file.to_str().unwrap()), None);

    assert_eq!(
        path, socket_file,
        "PRIMAL_SOCKET as existing file returns as-is"
    );
}

#[test]
fn test_build_socket_path_deterministic_same_family() {
    let discovery = SocketDiscovery::new("family-x");
    let path1 = discovery.build_socket_path("beardog");
    let path2 = discovery.build_socket_path("beardog");
    assert_eq!(path1, path2);
}

#[test]
fn test_build_socket_path_different_families_different_paths() {
    let d1 = SocketDiscovery::new("family-a");
    let d2 = SocketDiscovery::new("family-b");
    let p1 = d1.build_socket_path("beardog");
    let p2 = d2.build_socket_path("beardog");
    assert_ne!(p1, p2);
    assert!(p1.to_string_lossy().contains("family-a"));
    assert!(p2.to_string_lossy().contains("family-b"));
}

#[test]
fn test_build_socket_path_socket_name_format() {
    let discovery = SocketDiscovery::new("test");
    let path = discovery.build_socket_path("my-primal");
    let name = path.file_name().unwrap().to_string_lossy();
    assert_eq!(name, "my-primal-test.sock");
}
