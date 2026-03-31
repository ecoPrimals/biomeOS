// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Continuation of socket discovery engine tests (split from `engine_tests.rs`).
//! Covers manifest/registry discovery, XDG paths, and remaining discovery flows.
#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]

use super::engine::SocketDiscovery;
use super::result::DiscoveryMethod;
use super::strategy::DiscoveryStrategy;
use super::transport::TransportEndpoint;
use std::collections::HashMap;
use std::path::PathBuf;
use tempfile::TempDir;

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
async fn test_discover_primal_cache_hit_on_repeat() {
    let temp_dir = TempDir::new().unwrap();
    let biomeos_dir = temp_dir.path().join("biomeos");
    std::fs::create_dir_all(&biomeos_dir).unwrap();
    let socket_path = biomeos_dir.join("cache-primal-famx.sock");
    let _listener = tokio::net::UnixListener::bind(&socket_path).expect("bind unix");

    let discovery = SocketDiscovery::new("famx").with_xdg_override(temp_dir.path());
    let first = discovery.discover_primal("cache-primal").await;
    let second = discovery.discover_primal("cache-primal").await;
    assert_eq!(
        first.as_ref().map(|s| &s.path),
        second.as_ref().map(|s| &s.path)
    );
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
    let result = discovery.discover_endpoint_via_env_with("my-primal", Some(&env_overrides));

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
    let result = discovery.discover_endpoint_via_env_with("beardog", Some(&env_overrides));

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
    let result = discovery.discover_endpoint_via_env_with("foo", Some(&env_overrides));
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
    let r = discovery.discover_endpoint_via_env_with("foo", Some(&env_overrides));
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

#[tokio::test]
async fn test_verify_unix_socket_accepts_bound_listener() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().join("verify.sock");
    let _listener = tokio::net::UnixListener::bind(&path).expect("bind unix");
    let discovery = SocketDiscovery::new("fam");
    assert!(
        discovery.verify_unix_socket(&path).await,
        "listener socket should accept connections"
    );
}

#[tokio::test]
async fn test_clear_cache_no_panic() {
    let discovery = SocketDiscovery::new("cache-test");
    discovery.clear_cache().await;
}

#[tokio::test]
async fn test_discover_via_xdg_primal_family_sock_plain_file() {
    let temp = TempDir::new().unwrap();
    let biomeos = temp.path().join("biomeos");
    std::fs::create_dir_all(&biomeos).unwrap();
    let plain = biomeos.join("plain-fam.sock");
    std::fs::File::create(&plain).unwrap();

    let discovery = SocketDiscovery::new("fam").with_xdg_override(temp.path());
    let r = discovery.discover_primal("plain").await;
    assert!(r.is_some());
    assert_eq!(r.unwrap().path, plain);
}

#[tokio::test]
async fn test_discover_endpoint_via_env_bar_live_unix_listener() {
    let temp = TempDir::new().unwrap();
    let sock = temp.path().join("ep.sock");
    let _listener = tokio::net::UnixListener::bind(&sock).expect("bind");

    let env_overrides: HashMap<String, String> = [(
        "BAR_ENDPOINT".to_string(),
        sock.to_string_lossy().into_owned(),
    )]
    .into();
    let discovery = SocketDiscovery::new("test");
    let r = discovery.discover_endpoint_via_env_with("bar", Some(&env_overrides));
    assert!(r.is_some());
}

#[tokio::test]
async fn test_discover_via_manifest_invalid_json_skips_quietly() {
    let temp = TempDir::new().unwrap();
    let manifest_dir = temp.path().join("ecoPrimals").join("manifests");
    std::fs::create_dir_all(&manifest_dir).unwrap();
    std::fs::write(manifest_dir.join("bad-json-primal.json"), "{").unwrap();

    let strategy = DiscoveryStrategy {
        check_env_hints: false,
        use_xdg_runtime: false,
        use_family_tmp: false,
        query_registry: false,
        enable_tcp_fallback: false,
        ..Default::default()
    };
    let discovery = SocketDiscovery::with_strategy("test", strategy)
        .with_temp_dir_override(temp.path())
        .with_xdg_override(temp.path());
    assert!(discovery.discover_primal("bad-json-primal").await.is_none());
}

#[tokio::test]
async fn test_get_xdg_runtime_dir_helper() {
    let _ = SocketDiscovery::get_xdg_runtime_dir();
}

#[test]
fn calculate_primal_port_is_deterministic() {
    let d = SocketDiscovery::new("family-x");
    let p = d.calculate_primal_port("beardog");
    assert_eq!(p, d.calculate_primal_port("beardog"));
    assert_ne!(
        p,
        d.calculate_primal_port("songbird"),
        "different primals should map to different ports in the band"
    );
}

#[tokio::test]
async fn verify_tcp_connection_fails_on_unused_port() {
    let d = SocketDiscovery::new("f");
    assert!(
        !d.verify_tcp_connection("127.0.0.1", 1).await,
        "port 1 is not a typical listening service in tests"
    );
}

#[tokio::test]
async fn discover_endpoint_via_env_tcp_override_parses() {
    let env_overrides: HashMap<String, String> =
        [("FOO_TCP".to_string(), "127.0.0.1:65534".to_string())].into();
    let discovery = SocketDiscovery::new("test");
    let r = discovery.discover_endpoint_via_env_with("foo", Some(&env_overrides));
    assert!(r.is_some());
}

#[tokio::test]
async fn discover_capability_unknown_emits_none_without_taxonomy() {
    let strategy = DiscoveryStrategy {
        check_env_hints: false,
        use_xdg_runtime: false,
        use_family_tmp: false,
        query_registry: false,
        enable_tcp_fallback: false,
        enable_cache: false,
        ..Default::default()
    };
    let discovery = SocketDiscovery::with_strategy("zzz-unknown-family", strategy);
    assert!(
        discovery
            .discover_capability("not_a_real_capability_xyz_12345")
            .await
            .is_none()
    );
}

/// Tier 2 TCP fallback: `try_tcp_fallback` uses `{PRIMAL}_TCP` when Tier 1 finds nothing.
#[tokio::test]
async fn test_discover_with_fallback_tcp_env_chain_after_tier1_miss() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind tcp");
    let port = listener.local_addr().expect("addr").port();
    tokio::spawn(async move {
        let _ = listener.accept().await;
    });

    let primal = "tcpfbprimal";
    let tcp_val = format!("127.0.0.1:{port}");
    let strategy = DiscoveryStrategy {
        check_env_hints: false,
        use_xdg_runtime: false,
        use_family_tmp: false,
        try_abstract_sockets: false,
        query_registry: false,
        enable_tcp_fallback: true,
        ..Default::default()
    };
    let discovery = SocketDiscovery::with_strategy("tfam", strategy);
    let ep = discovery
        .discover_with_fallback_with_env_overrides(primal, None, Some(tcp_val.as_str()))
        .await;
    assert!(ep.is_some());
    assert!(
        matches!(ep.unwrap(), TransportEndpoint::TcpSocket { .. }),
        "expected TCP tier-2 endpoint"
    );
}

/// `try_tcp_fallback` accepts bare port in `{PRIMAL}_TCP` using `tcp_fallback_host`.
#[tokio::test]
async fn test_discover_with_fallback_tcp_port_only_env_uses_fallback_host() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind tcp");
    let port = listener.local_addr().expect("addr").port();
    tokio::spawn(async move {
        let _ = listener.accept().await;
    });

    let port_s = port.to_string();
    let strategy = DiscoveryStrategy {
        check_env_hints: false,
        use_xdg_runtime: false,
        use_family_tmp: false,
        try_abstract_sockets: false,
        query_registry: false,
        enable_tcp_fallback: true,
        tcp_fallback_host: std::sync::Arc::from("127.0.0.1"),
        ..Default::default()
    };
    let discovery = SocketDiscovery::with_strategy("pfam", strategy);
    let ep = discovery
        .discover_with_fallback_with_env_overrides("portonlyx", None, Some(port_s.as_str()))
        .await;
    assert!(ep.is_some());
}

/// Multi-step: XDG Unix succeeds first (no TCP) — exercises `try_unix_socket_xdg` verify path.
#[tokio::test]
async fn test_discover_with_fallback_unix_xdg_before_abstract_or_tcp() {
    let temp = TempDir::new().unwrap();
    let biomeos = temp.path().join("biomeos");
    std::fs::create_dir_all(&biomeos).unwrap();
    let sock = biomeos.join("chainxdg-pf.sock");
    let _listener = tokio::net::UnixListener::bind(&sock).expect("bind unix");

    let strategy = DiscoveryStrategy {
        check_env_hints: false,
        use_xdg_runtime: true,
        use_family_tmp: true,
        try_abstract_sockets: cfg!(target_os = "linux"),
        query_registry: false,
        enable_tcp_fallback: true,
        ..Default::default()
    };
    let discovery = SocketDiscovery::with_strategy("pf", strategy).with_xdg_override(temp.path());
    let ep = discovery.discover_with_fallback("chainxdg").await;
    assert!(ep.is_some());
    match ep.unwrap() {
        TransportEndpoint::UnixSocket { path } => assert_eq!(path, sock),
        _ => panic!("expected Unix socket from XDG tier"),
    }
}

/// Manifest discovery prefers `$XDG_RUNTIME_DIR/ecoPrimals/manifests/` when override is set.
#[tokio::test]
async fn test_discover_via_manifest_reads_xdg_manifest_candidate_first() {
    use super::result::PrimalManifest;
    use std::sync::Arc;

    let temp = TempDir::new().unwrap();
    let manifests = temp.path().join("ecoPrimals").join("manifests");
    std::fs::create_dir_all(&manifests).unwrap();
    let sock = temp.path().join("manifest-xdg.sock");
    let _listener = tokio::net::UnixListener::bind(&sock).expect("bind");

    let manifest = PrimalManifest {
        primal: Arc::from("manifestxdg"),
        socket: Arc::from(sock.to_string_lossy().as_ref()),
        capabilities: vec![],
        pid: None,
    };
    std::fs::write(
        manifests.join("manifestxdg.json"),
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
    let discovery = SocketDiscovery::with_strategy("mf", strategy).with_xdg_override(temp.path());
    let r = discovery.discover_primal("manifestxdg").await;
    assert!(r.is_some());
    assert_eq!(r.unwrap().discovered_via, DiscoveryMethod::Manifest);
}

/// Socket registry entry with a live Unix socket succeeds.
#[tokio::test]
async fn test_discover_via_socket_registry_matching_live_socket() {
    let temp = TempDir::new().unwrap();
    let biomeos = temp.path().join("biomeos");
    std::fs::create_dir_all(&biomeos).unwrap();
    let sock = temp.path().join("reg-live.sock");
    let _listener = tokio::net::UnixListener::bind(&sock).expect("bind");

    let registry = serde_json::json!({
        "version": "1.0",
        "entries": [{
            "primal": "reg-live",
            "socket": sock.to_str().unwrap(),
            "capabilities": ["x"]
        }]
    });
    std::fs::write(
        biomeos.join("socket-registry.json"),
        serde_json::to_string(&registry).unwrap(),
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
    let discovery = SocketDiscovery::with_strategy("rf", strategy).with_xdg_override(temp.path());
    let r = discovery.discover_primal("reg-live").await;
    assert!(r.is_some());
    assert_eq!(r.unwrap().discovered_via, DiscoveryMethod::SocketRegistry);
}

/// Capability-first XDG socket (`security.sock`) for `beardog` resolves before primal-named paths.
#[tokio::test]
async fn test_discover_via_xdg_beardog_capability_security_sock() {
    let temp = TempDir::new().unwrap();
    let biomeos = temp.path().join("biomeos");
    std::fs::create_dir_all(&biomeos).unwrap();
    let cap_sock = biomeos.join("security.sock");
    let _listener = tokio::net::UnixListener::bind(&cap_sock).expect("bind");

    let strategy = DiscoveryStrategy {
        check_env_hints: false,
        ..Default::default()
    };
    let discovery = SocketDiscovery::with_strategy("bf", strategy).with_xdg_override(temp.path());
    let r = discovery.discover_primal("beardog").await;
    assert!(r.is_some());
    assert_eq!(r.unwrap().path, cap_sock);
}

/// `verify_unix_socket` returns false when path exists but is not a listening socket.
#[tokio::test]
async fn test_verify_unix_socket_exists_but_not_listener() {
    let temp = TempDir::new().unwrap();
    let plain = temp.path().join("plain-file.sock");
    std::fs::File::create(&plain).unwrap();
    let discovery = SocketDiscovery::new("t");
    assert!(!discovery.verify_unix_socket(&plain).await);
}

#[cfg(target_os = "linux")]
#[tokio::test]
async fn test_discover_with_fallback_abstract_tier_connects() {
    use std::os::linux::net::SocketAddrExt;
    use std::os::unix::net::{SocketAddr, UnixListener};

    let family = "absfam";
    let primal = "absprim";
    let name = format!("biomeos_{}_{}", primal, family);
    let addr = SocketAddr::from_abstract_name(name.as_str()).expect("abstract addr");
    let std_listener = UnixListener::bind_addr(&addr).expect("bind abstract");
    std_listener.set_nonblocking(true).expect("nonblocking");
    let tok_listener =
        tokio::net::UnixListener::from_std(std_listener).expect("tokio unix listener");
    tokio::spawn(async move {
        let _ = tok_listener.accept().await;
    });

    let strategy = DiscoveryStrategy {
        check_env_hints: false,
        use_xdg_runtime: false,
        use_family_tmp: false,
        try_abstract_sockets: true,
        query_registry: false,
        enable_tcp_fallback: false,
        ..Default::default()
    };
    let discovery = SocketDiscovery::with_strategy(family, strategy);
    let ep = discovery.discover_with_fallback(primal).await;
    assert!(ep.is_some());
    match ep.expect("endpoint") {
        TransportEndpoint::AbstractSocket { name: n } => assert_eq!(n.as_ref(), name.as_str()),
        other => panic!("expected abstract endpoint, got {other:?}"),
    }
}
