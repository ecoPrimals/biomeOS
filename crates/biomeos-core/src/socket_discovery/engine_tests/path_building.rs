// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::engine::SocketDiscovery;
use super::super::strategy::DiscoveryStrategy;
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
    let discovery = SocketDiscovery::with_strategy("test", strategy);
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
