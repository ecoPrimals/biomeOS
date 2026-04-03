// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Tier-2 and integration socket discovery engine tests (split from `engine_tests2.rs`).
//! Covers tiered TCP/abstract fallback chains, manifest-under-XDG ordering,
//! live registry sockets, beardog capability paths, and additional verification.
#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]

use super::engine::SocketDiscovery;
use super::result::DiscoveryMethod;
use super::strategy::DiscoveryStrategy;
use super::transport::TransportEndpoint;
use tempfile::TempDir;

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
