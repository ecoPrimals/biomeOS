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
async fn test_env_hint_discovery() {
    let env_overrides: HashMap<String, String> = [(
        "TEST_PRIMAL_SOCKET".to_string(),
        "/tmp/test-primal.sock".to_string(),
    )]
    .into();
    let discovery = SocketDiscovery::new("test");
    let result = discovery.discover_via_env_hint_with("test_primal", Some(&env_overrides));

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
    let result = discovery.discover_via_env_hint_with("test_primal", Some(&env_overrides));

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
    let result = discovery.discover_via_env_hint_with("beardog", Some(&env_overrides));

    assert!(result.is_some());
}

#[tokio::test]
async fn test_discover_endpoint_via_env_tcp() {
    let env_overrides: HashMap<String, String> =
        [("BEARDOG_TCP".to_string(), "127.0.0.1:9100".to_string())].into();
    let discovery = SocketDiscovery::new("test");
    let result = discovery.discover_endpoint_via_env_with("beardog", Some(&env_overrides));

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
        "tcp://192.0.2.1:8080".to_string(),
    )]
    .into();
    let discovery = SocketDiscovery::new("test");
    let result = discovery.discover_endpoint_via_env_with("beardog", Some(&env_overrides));

    assert!(result.is_some());
}
#[tokio::test]
async fn test_discover_endpoint_via_env_tcp_port_only() {
    // BEARDOG_TCP with just port number uses strategy.tcp_fallback_host
    let env_overrides: HashMap<String, String> =
        [("BEARDOG_TCP".to_string(), "9100".to_string())].into();
    let discovery = SocketDiscovery::new("test");
    let result = discovery.discover_endpoint_via_env_with("beardog", Some(&env_overrides));

    // May or may not succeed depending on whether 127.0.0.1:9100 is listening
    // Just verify it doesn't panic - TCP verification happens in try_tcp_fallback
    let _ = result;
}

#[tokio::test]
async fn test_discover_endpoint_via_env_biomeos_prefix() {
    let temp_dir = TempDir::new().unwrap();
    let socket_path = temp_dir.path().join("beardog.sock");
    std::fs::File::create(&socket_path).unwrap();

    let env_overrides: HashMap<String, String> = [(
        "BIOMEOS_BEARDOG_SOCKET".to_string(),
        socket_path.to_string_lossy().to_string(),
    )]
    .into();
    let discovery = SocketDiscovery::new("test");
    let result = discovery.discover_endpoint_via_env_with("beardog", Some(&env_overrides));

    assert!(result.is_some());
}

#[tokio::test]
async fn test_discover_endpoint_via_env_primal_name_with_dash() {
    let env_overrides: HashMap<String, String> =
        [("MY_PRIMAL_TCP".to_string(), "127.0.0.1:9200".to_string())].into();
    let discovery = SocketDiscovery::new("test");
    let result = discovery.discover_endpoint_via_env_with("my-primal", Some(&env_overrides));

    assert!(result.is_some());
    if let Some(TransportEndpoint::TcpSocket { host, port }) = result {
        assert_eq!(host.as_ref(), "127.0.0.1");
        assert_eq!(port, 9200);
    }
}
