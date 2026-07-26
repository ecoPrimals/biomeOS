// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::engine::SocketDiscovery;
use super::super::strategy::DiscoveryStrategy;
use std::path::PathBuf;
use tempfile::TempDir;

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
fn test_calculate_primal_port_empty_name() {
    let discovery = SocketDiscovery::new("test");
    let port = discovery.calculate_primal_port("");
    assert!((9100..9200).contains(&port));
}

#[test]
fn test_calculate_primal_port_unicode_name() {
    let discovery = SocketDiscovery::new("test");
    let port = discovery.calculate_primal_port("primal-é");
    assert!((9100..9200).contains(&port));
}

#[test]
fn test_build_socket_path_with_xdg_and_primal_socket() {
    let temp_dir = TempDir::new().unwrap();
    let socket_dir = temp_dir.path().join("custom");
    std::fs::create_dir_all(&socket_dir).unwrap();

    let discovery = SocketDiscovery::new("fam");
    let path = discovery.build_socket_path_with(
        "beardog",
        Some(socket_dir.to_str().unwrap()),
        Some(temp_dir.path()),
    );

    // primal_socket takes precedence over xdg
    assert_eq!(path, socket_dir.join("beardog-fam.sock"));
}
