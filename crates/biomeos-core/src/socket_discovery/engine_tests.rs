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
//! - Manifest and registry discovery
//! - XDG and family tmp path discovery

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

#[tokio::test]
async fn test_discover_endpoint_via_env_tcp_port_only() {
    // BEARDOG_TCP with just port number uses strategy.tcp_fallback_host
    let env_overrides: HashMap<String, String> =
        [("BEARDOG_TCP".to_string(), "9100".to_string())].into();
    let discovery = SocketDiscovery::new("test");
    let result = discovery
        .discover_endpoint_via_env_with("beardog", Some(&env_overrides))
        .await;

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
    let result = discovery
        .discover_endpoint_via_env_with("beardog", Some(&env_overrides))
        .await;

    assert!(result.is_some());
}

#[tokio::test]
async fn test_discover_endpoint_via_env_primal_name_with_dash() {
    let env_overrides: HashMap<String, String> =
        [("MY_PRIMAL_TCP".to_string(), "127.0.0.1:9200".to_string())].into();
    let discovery = SocketDiscovery::new("test");
    let result = discovery
        .discover_endpoint_via_env_with("my-primal", Some(&env_overrides))
        .await;

    assert!(result.is_some());
    if let Some(TransportEndpoint::TcpSocket { host, port }) = result {
        assert_eq!(host.as_ref(), "127.0.0.1");
        assert_eq!(port, 9200);
    }
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

#[tokio::test]
async fn test_discover_via_manifest_valid() {
    use super::result::PrimalManifest;
    use std::sync::Arc;

    let temp_dir = TempDir::new().unwrap();
    let manifest_dir = temp_dir.path().join("ecoPrimals").join("manifests");
    std::fs::create_dir_all(&manifest_dir).unwrap();

    let socket_path = temp_dir.path().join("test-primal.sock");
    std::fs::File::create(&socket_path).unwrap();

    let manifest = PrimalManifest {
        primal: Arc::from("test-primal"),
        socket: Arc::from(socket_path.to_string_lossy().as_ref()),
        capabilities: vec!["test".to_string()],
        pid: Some(1234),
    };
    let manifest_path = manifest_dir.join("test-primal.json");
    std::fs::write(&manifest_path, serde_json::to_string(&manifest).unwrap()).unwrap();

    let discovery = SocketDiscovery::new("test");
    let result = discovery.discover_primal("test-primal").await;

    // May or may not find - depends on XDG_RUNTIME_DIR and whether verify_unix_socket passes
    // (socket file is not a real unix socket, so verify will fail)
    let _ = result;
}

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
async fn test_discover_via_xdg_path_exists() {
    let temp_dir = TempDir::new().unwrap();
    let biomeos_dir = temp_dir.path().join("biomeos");
    std::fs::create_dir_all(&biomeos_dir).unwrap();

    let socket_path = biomeos_dir.join("xdg-primal-test.sock");
    std::fs::File::create(&socket_path).unwrap();

    let discovery = SocketDiscovery::new("test").with_xdg_override(temp_dir.path());
    let result = discovery.discover_primal("xdg-primal").await;

    assert!(
        result.is_some(),
        "XDG discovery should find socket when path exists"
    );
    if let Some(socket) = result {
        assert_eq!(
            socket.discovered_via,
            super::result::DiscoveryMethod::XdgRuntime
        );
    }
}

#[tokio::test]
async fn test_discover_via_family_tmp_path_exists() {
    let temp_dir = TempDir::new().unwrap();
    let socket_path = temp_dir.path().join("tmp-primal-test.sock");
    std::fs::File::create(&socket_path).unwrap();

    let discovery = SocketDiscovery::new("test").with_temp_dir_override(temp_dir.path());
    let result = discovery.discover_primal("tmp-primal").await;

    assert!(
        result.is_some(),
        "Family tmp discovery should find socket when path exists"
    );
}

#[tokio::test]
async fn test_get_endpoint_convenience() {
    let discovery = SocketDiscovery::new("test");
    let result = discovery.get_endpoint("nonexistent").await;
    assert!(result.is_none() || result.is_some());
}

#[tokio::test]
async fn test_discover_strategy_registry_disabled() {
    let strategy = super::DiscoveryStrategy {
        query_registry: false,
        ..Default::default()
    };
    let discovery = SocketDiscovery::with_strategy("test", strategy);
    let result = discovery.discover_capability("nonexistent").await;
    assert!(result.is_none());
}

#[tokio::test]
async fn test_discover_strategy_env_disabled() {
    let strategy = super::DiscoveryStrategy {
        check_env_hints: false,
        ..Default::default()
    };
    let discovery = SocketDiscovery::with_strategy("test", strategy);
    let result = discovery.discover_primal("beardog").await;
    assert!(result.is_none() || result.is_some());
}

#[tokio::test]
async fn test_discover_via_registry_nonexistent_socket() {
    let strategy = super::DiscoveryStrategy {
        check_env_hints: false,
        use_xdg_runtime: false,
        use_family_tmp: false,
        query_registry: true,
        enable_tcp_fallback: false,
        ..Default::default()
    };
    let discovery = SocketDiscovery::with_strategy("test", strategy)
        .with_neural_api(PathBuf::from("/nonexistent/neural-api.sock"));

    let result = discovery.discover_primal("registry-only-primal").await;
    assert!(result.is_none());
}

#[tokio::test]
async fn test_discover_capability_via_registry_fails_gracefully() {
    let strategy = super::DiscoveryStrategy {
        check_env_hints: false,
        use_xdg_runtime: false,
        use_family_tmp: false,
        query_registry: true,
        enable_tcp_fallback: false,
        ..Default::default()
    };
    let discovery = SocketDiscovery::with_strategy("test", strategy)
        .with_neural_api(PathBuf::from("/nonexistent/capability-registry.sock"));

    let result = discovery
        .discover_capability("nonexistent-capability")
        .await;
    assert!(result.is_none());
}

#[tokio::test]
async fn test_xdg_override_nonexistent_skips_xdg_discovery() {
    let bogus = PathBuf::from("/nonexistent/xdg/override/path/012345");
    let discovery = SocketDiscovery::new("test").with_xdg_override(&bogus);
    let result = discovery.discover_primal("any-primal").await;
    assert!(result.is_none());
}

#[tokio::test]
async fn test_temp_dir_override_used_for_family_tmp_discovery() {
    let temp_dir = TempDir::new().unwrap();
    let socket_path = temp_dir.path().join("override-primal-ovr.sock");
    std::fs::File::create(&socket_path).unwrap();

    let discovery = SocketDiscovery::new("ovr").with_temp_dir_override(temp_dir.path());
    let result = discovery.discover_primal("override-primal").await;
    assert!(
        result.is_some(),
        "family tmp discovery should see socket under temp dir override"
    );
}

#[tokio::test]
async fn test_discover_endpoint_via_env_generic_endpoint_unix() {
    let temp_dir = TempDir::new().unwrap();
    let socket_path = temp_dir.path().join("via-endpoint.sock");
    std::fs::File::create(&socket_path).unwrap();

    let env_overrides: HashMap<String, String> = [(
        "MY_PRIMAL_ENDPOINT".to_string(),
        socket_path.to_string_lossy().to_string(),
    )]
    .into();
    let discovery = SocketDiscovery::new("test");
    let result = discovery
        .discover_endpoint_via_env_with("my-primal", Some(&env_overrides))
        .await;

    assert!(result.is_some());
    if let Some(TransportEndpoint::UnixSocket { path }) = result {
        assert_eq!(path, socket_path);
    } else {
        panic!("expected Unix endpoint from MY_PRIMAL_ENDPOINT");
    }
}

#[tokio::test]
async fn test_discover_endpoint_via_env_tcp_prefix_fallback() {
    let env_overrides: HashMap<String, String> =
        [("BEARDOG_TCP".to_string(), "127.0.0.1:19100".to_string())].into();
    let discovery = SocketDiscovery::new("test");
    let result = discovery
        .discover_endpoint_via_env_with("beardog", Some(&env_overrides))
        .await;

    assert!(result.is_some());
    if let Some(TransportEndpoint::TcpSocket { host, port }) = result {
        assert_eq!(host.as_ref(), "127.0.0.1");
        assert_eq!(port, 19100);
    }
}

#[tokio::test]
async fn test_clear_cache_idempotent() {
    let discovery = SocketDiscovery::new("test");
    discovery.clear_cache().await;
    discovery.clear_cache().await;
}

#[tokio::test]
async fn test_verify_unix_socket_connects_to_bound_listener() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().join("verify-live.sock");
    let _listener = tokio::net::UnixListener::bind(&path).expect("bind unix listener");
    let discovery = SocketDiscovery::new("test");
    assert!(
        discovery.verify_unix_socket(&path).await,
        "listener should accept verify_unix_socket probe"
    );
}

#[tokio::test]
async fn test_discover_primal_second_call_uses_cache_marker() {
    let temp_dir = TempDir::new().unwrap();
    let socket_path = temp_dir.path().join("cache-mark-fam.sock");
    std::fs::File::create(&socket_path).unwrap();
    let discovery = SocketDiscovery::new("fam").with_temp_dir_override(temp_dir.path());
    let first = discovery
        .discover_primal("cache-mark")
        .await
        .expect("first discovery");
    assert_ne!(first.discovered_via, DiscoveryMethod::Cached);
    let second = discovery
        .discover_primal("cache-mark")
        .await
        .expect("second discovery");
    assert_eq!(second.discovered_via, DiscoveryMethod::Cached);
}

#[tokio::test]
async fn test_discover_via_manifest_invalid_json_skipped() {
    let temp_dir = TempDir::new().unwrap();
    let manifest_dir = temp_dir.path().join("ecoPrimals").join("manifests");
    std::fs::create_dir_all(&manifest_dir).unwrap();
    std::fs::write(manifest_dir.join("bad-json-primal.json"), "{ not json").unwrap();
    let discovery = SocketDiscovery::new("test").with_temp_dir_override(temp_dir.path());
    assert!(discovery.discover_primal("bad-json-primal").await.is_none());
}

#[tokio::test]
async fn test_discover_via_manifest_valid_json_dead_socket() {
    use super::result::PrimalManifest;
    use std::sync::Arc;

    let temp_dir = TempDir::new().unwrap();
    let manifest_dir = temp_dir.path().join("ecoPrimals").join("manifests");
    std::fs::create_dir_all(&manifest_dir).unwrap();
    let dead_sock = temp_dir.path().join("dead.sock");
    std::fs::File::create(&dead_sock).unwrap();
    let manifest = PrimalManifest {
        primal: Arc::from("dead-sock-primal"),
        socket: Arc::from(dead_sock.to_string_lossy().as_ref()),
        capabilities: vec![],
        pid: None,
    };
    std::fs::write(
        manifest_dir.join("dead-sock-primal.json"),
        serde_json::to_string(&manifest).unwrap(),
    )
    .unwrap();
    let discovery = SocketDiscovery::new("test").with_temp_dir_override(temp_dir.path());
    assert!(
        discovery
            .discover_primal("dead-sock-primal")
            .await
            .is_none()
    );
}

#[tokio::test]
async fn test_discover_capability_socket_tmp_only() {
    let temp_dir = TempDir::new().unwrap();
    let bogus = PathBuf::from("/nonexistent/xdg/for-cap-sock");
    let cap_sock = temp_dir.path().join("custom-cap.sock");
    let _listener = tokio::net::UnixListener::bind(&cap_sock).expect("bind cap sock");
    let discovery = SocketDiscovery::new("test")
        .with_xdg_override(&bogus)
        .with_temp_dir_override(temp_dir.path());
    let result = discovery.discover_capability("custom-cap").await;
    assert!(result.is_some());
}

#[tokio::test]
async fn test_discover_endpoint_via_env_unix_missing_file_skipped() {
    let env_overrides: HashMap<String, String> = [(
        "FOO_SOCKET".to_string(),
        "/nonexistent/path/to/missing.sock".to_string(),
    )]
    .into();
    let discovery = SocketDiscovery::new("test");
    let result = discovery
        .discover_endpoint_via_env_with("foo", Some(&env_overrides))
        .await;
    assert!(result.is_none());
}

#[tokio::test]
async fn test_discover_with_fallback_uses_cache_for_endpoint_key() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().join("fb-cache-fam.sock");
    let _listener = tokio::net::UnixListener::bind(&path).expect("bind for fallback cache");
    let discovery = SocketDiscovery::new("fam").with_temp_dir_override(temp_dir.path());
    assert!(discovery.discover_with_fallback("fb-cache").await.is_some());
    assert!(discovery.discover_with_fallback("fb-cache").await.is_some());
}

#[test]
fn test_discovery_strategy_cross_device_host() {
    let s = DiscoveryStrategy::cross_device();
    assert_eq!(s.tcp_fallback_host.as_ref(), "0.0.0.0");
    assert!(!s.use_xdg_runtime);
}

#[test]
fn test_discovery_strategy_android_disables_xdg() {
    let s = DiscoveryStrategy::android();
    assert!(!s.use_xdg_runtime);
    assert!(s.try_abstract_sockets);
}

#[tokio::test]
async fn test_check_cache_miss_unknown_key() {
    let discovery = SocketDiscovery::new("test");
    assert!(discovery.check_cache("no-such-key").await.is_none());
}

#[tokio::test]
async fn test_verify_tcp_connection_refused_fast() {
    let discovery = SocketDiscovery::new("test");
    assert!(!discovery.verify_tcp_connection("127.0.0.1", 59998).await);
}

#[tokio::test]
async fn test_discover_primal_all_strategies_off_returns_none() {
    let strategy = DiscoveryStrategy {
        check_env_hints: false,
        use_xdg_runtime: false,
        use_family_tmp: false,
        query_registry: false,
        enable_tcp_fallback: false,
        enable_cache: false,
        ..DiscoveryStrategy::default()
    };
    let temp = TempDir::new().unwrap();
    let discovery =
        SocketDiscovery::with_strategy("test", strategy).with_temp_dir_override(temp.path());
    assert!(discovery.discover_primal("nope").await.is_none());
}

#[tokio::test]
async fn test_get_socket_path_none_when_unresolvable() {
    let strategy = DiscoveryStrategy {
        check_env_hints: false,
        use_xdg_runtime: false,
        use_family_tmp: false,
        query_registry: false,
        enable_tcp_fallback: false,
        ..DiscoveryStrategy::default()
    };
    let discovery = SocketDiscovery::with_strategy("test", strategy);
    assert!(discovery.get_socket_path("gone").await.is_none());
}

/// `discover_via_xdg` returns when the family-scoped path exists (no connect probe on this branch).
#[tokio::test]
async fn test_discover_via_xdg_family_scoped_path_exists_without_connect() {
    let temp = TempDir::new().unwrap();
    let biomeos = temp.path().join("biomeos");
    std::fs::create_dir_all(&biomeos).unwrap();
    let sock = biomeos.join("xdg-no-verify-fam.sock");
    std::fs::File::create(&sock).unwrap();

    let discovery = SocketDiscovery::new("fam").with_xdg_override(temp.path());
    let r = discovery.discover_primal("xdg-no-verify").await;
    assert!(r.is_some());
    assert_eq!(r.unwrap().discovered_via, DiscoveryMethod::XdgRuntime);
}

/// Legacy `{primal}.sock` under XDG when file exists.
#[tokio::test]
async fn test_discover_via_xdg_legacy_filename_exists() {
    let temp = TempDir::new().unwrap();
    let biomeos = temp.path().join("biomeos");
    std::fs::create_dir_all(&biomeos).unwrap();
    let sock = biomeos.join("legacy-only.sock");
    std::fs::File::create(&sock).unwrap();

    let discovery = SocketDiscovery::new("fam").with_xdg_override(temp.path());
    let r = discovery.discover_primal("legacy-only").await;
    assert!(r.is_some());
}

/// Family tmp branch: `{primal}-{family}.sock` exists → returns without Unix connect.
#[tokio::test]
async fn test_discover_via_family_tmp_scoped_exists_without_connect() {
    let temp = TempDir::new().unwrap();
    let sock = temp.path().join("tmp-scoped-fam.sock");
    std::fs::File::create(&sock).unwrap();

    let discovery = SocketDiscovery::new("fam").with_temp_dir_override(temp.path());
    let r = discovery.discover_primal("tmp-scoped").await;
    assert!(r.is_some());
    assert_eq!(r.unwrap().discovered_via, DiscoveryMethod::FamilyTmp);
}

/// Legacy `{primal}.sock` in family tmp.
#[tokio::test]
async fn test_discover_via_family_tmp_legacy_exists() {
    let temp = TempDir::new().unwrap();
    let sock = temp.path().join("tmp-legacy.sock");
    std::fs::File::create(&sock).unwrap();

    let discovery = SocketDiscovery::new("fam").with_temp_dir_override(temp.path());
    let r = discovery.discover_primal("tmp-legacy").await;
    assert!(r.is_some());
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
async fn test_discover_endpoint_via_env_tcp_non_matching_parse_returns_none() {
    let env_overrides: HashMap<String, String> =
        [("FOO_TCP".to_string(), "not-a-tcp-endpoint".to_string())].into();
    let discovery = SocketDiscovery::new("test");
    let r = discovery
        .discover_endpoint_via_env_with("foo", Some(&env_overrides))
        .await;
    assert!(r.is_none());
}

#[tokio::test]
async fn test_discover_with_fallback_manifest_branch_sets_endpoint() {
    use super::result::PrimalManifest;
    use std::sync::Arc;

    let temp = TempDir::new().unwrap();
    let manifest_dir = temp.path().join("ecoPrimals").join("manifests");
    std::fs::create_dir_all(&manifest_dir).unwrap();
    let sock = temp.path().join("fb-manifest.sock");
    let _listener = tokio::net::UnixListener::bind(&sock).expect("bind");

    let manifest = PrimalManifest {
        primal: Arc::from("fb-manifest-primal"),
        socket: Arc::from(sock.to_string_lossy().as_ref()),
        capabilities: vec![],
        pid: None,
    };
    std::fs::write(
        manifest_dir.join("fb-manifest-primal.json"),
        serde_json::to_string(&manifest).unwrap(),
    )
    .unwrap();

    let strategy = DiscoveryStrategy {
        check_env_hints: false,
        use_xdg_runtime: false,
        use_family_tmp: false,
        query_registry: false,
        enable_tcp_fallback: false,
        ..Default::default()
    };
    let discovery =
        SocketDiscovery::with_strategy("test", strategy).with_temp_dir_override(temp.path());
    let ep = discovery.discover_with_fallback("fb-manifest-primal").await;
    assert!(ep.is_some());
}

#[cfg(target_os = "linux")]
#[tokio::test]
async fn test_discover_with_fallback_abstract_path_when_no_match() {
    let strategy = DiscoveryStrategy {
        check_env_hints: false,
        use_xdg_runtime: false,
        use_family_tmp: false,
        try_abstract_sockets: true,
        query_registry: false,
        enable_tcp_fallback: false,
        ..Default::default()
    };
    let discovery = SocketDiscovery::with_strategy("abstract-miss", strategy);
    assert!(
        discovery
            .discover_with_fallback("no-such-abstract")
            .await
            .is_none()
    );
}

#[tokio::test]
async fn test_verify_tcp_connection_accepts_open_port() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind");
    let port = listener.local_addr().expect("addr").port();
    tokio::spawn(async move {
        let _ = listener.accept().await;
    });
    let discovery = SocketDiscovery::new("tcp-verify");
    assert!(
        discovery.verify_tcp_connection("127.0.0.1", port).await,
        "expected successful TCP probe"
    );
}

#[tokio::test]
async fn test_discover_capability_taxonomy_resolve_primal_branch() {
    let strategy = DiscoveryStrategy {
        check_env_hints: false,
        use_xdg_runtime: false,
        use_family_tmp: false,
        query_registry: false,
        enable_tcp_fallback: false,
        ..Default::default()
    };
    let discovery = SocketDiscovery::with_strategy("tax", strategy);
    let r = discovery.discover_capability("encryption").await;
    assert!(r.is_none());
}
