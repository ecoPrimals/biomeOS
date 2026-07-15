// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::engine::SocketDiscovery;
use super::super::strategy::DiscoveryStrategy;
use std::path::PathBuf;

#[tokio::test]
async fn test_discover_capability_socket_tmp_only() {
    let temp_dir = tempfile::TempDir::new().unwrap();
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
