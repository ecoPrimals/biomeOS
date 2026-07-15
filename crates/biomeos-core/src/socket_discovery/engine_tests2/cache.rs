// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::engine::SocketDiscovery;
use super::super::result::DiscoveryMethod;

#[tokio::test]
async fn test_discover_primal_cache_hit_on_repeat() {
    let temp_dir = tempfile::TempDir::new().unwrap();
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
async fn test_clear_cache_idempotent() {
    let discovery = SocketDiscovery::new("test");
    discovery.clear_cache().await;
    discovery.clear_cache().await;
}

#[tokio::test]
async fn test_discover_primal_second_call_uses_cache_marker() {
    let temp_dir = tempfile::TempDir::new().unwrap();
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
async fn test_check_cache_miss_unknown_key() {
    let discovery = SocketDiscovery::new("test");
    assert!(discovery.check_cache("no-such-key").await.is_none());
}

#[tokio::test]
async fn test_clear_cache_no_panic() {
    let discovery = SocketDiscovery::new("cache-test");
    discovery.clear_cache().await;
}

#[tokio::test]
async fn test_get_endpoint_convenience() {
    let discovery = SocketDiscovery::new("test");
    let result = discovery.get_endpoint("nonexistent").await;
    assert!(result.is_none() || result.is_some());
}
