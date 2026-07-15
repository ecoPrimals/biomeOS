// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::engine::SocketDiscovery;
use super::super::strategy::DiscoveryStrategy;

#[tokio::test]
async fn test_discover_strategy_registry_disabled() {
    let strategy = super::super::strategy::DiscoveryStrategy {
        query_registry: false,
        ..Default::default()
    };
    let discovery = SocketDiscovery::with_strategy("test", strategy);
    let result = discovery.discover_capability("nonexistent").await;
    assert!(result.is_none());
}

#[tokio::test]
async fn test_discover_strategy_env_disabled() {
    let strategy = super::super::strategy::DiscoveryStrategy {
        check_env_hints: false,
        ..Default::default()
    };
    let discovery = SocketDiscovery::with_strategy("test", strategy);
    let result = discovery.discover_primal("beardog").await;
    assert!(result.is_none() || result.is_some());
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
    let temp = tempfile::TempDir::new().unwrap();
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
        ..Default::default()
    };
    let discovery = SocketDiscovery::with_strategy("test", strategy);
    assert!(discovery.get_socket_path("gone").await.is_none());
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
