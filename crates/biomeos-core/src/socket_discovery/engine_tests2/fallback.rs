// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::engine::SocketDiscovery;
use super::super::result::PrimalManifest;
use super::super::strategy::DiscoveryStrategy;
use std::sync::Arc;

#[tokio::test]
async fn test_discover_with_fallback_uses_cache_for_endpoint_key() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let path = temp_dir.path().join("fb-cache-fam.sock");
    let _listener = tokio::net::UnixListener::bind(&path).expect("bind for fallback cache");
    let discovery = SocketDiscovery::new("fam").with_temp_dir_override(temp_dir.path());
    assert!(discovery.discover_with_fallback("fb-cache").await.is_some());
    assert!(discovery.discover_with_fallback("fb-cache").await.is_some());
}

#[tokio::test]
async fn test_discover_with_fallback_manifest_branch_sets_endpoint() {
    let temp = tempfile::TempDir::new().unwrap();
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
