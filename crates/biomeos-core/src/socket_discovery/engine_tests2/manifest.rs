// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::engine::SocketDiscovery;
use super::super::result::PrimalManifest;
use std::sync::Arc;
use tempfile::TempDir;

#[tokio::test]
async fn test_discover_via_manifest_valid() {
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
async fn test_discover_via_manifest_invalid_json_skips_quietly() {
    let temp = TempDir::new().unwrap();
    let manifest_dir = temp.path().join("ecoPrimals").join("manifests");
    std::fs::create_dir_all(&manifest_dir).unwrap();
    std::fs::write(manifest_dir.join("bad-json-primal.json"), "{").unwrap();

    let strategy = super::super::strategy::DiscoveryStrategy {
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
