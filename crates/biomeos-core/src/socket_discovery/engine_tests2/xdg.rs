// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::engine::SocketDiscovery;
use super::super::result::DiscoveryMethod;
use tempfile::TempDir;

#[tokio::test]
async fn test_discover_via_xdg_path_exists() {
    let temp_dir = TempDir::new().unwrap();
    let membrane_dir = temp_dir.path().join("membrane");
    std::fs::create_dir_all(&membrane_dir).unwrap();

    let socket_path = membrane_dir.join("xdg-primal-test.sock");
    std::fs::File::create(&socket_path).unwrap();

    let discovery = SocketDiscovery::new("test").with_xdg_override(temp_dir.path());
    let result = discovery.discover_primal("xdg-primal").await;

    assert!(
        result.is_some(),
        "XDG discovery should find socket when path exists"
    );
    if let Some(socket) = result {
        assert_eq!(socket.discovered_via, DiscoveryMethod::XdgRuntime);
    }
}

#[tokio::test]
async fn test_xdg_override_nonexistent_skips_xdg_discovery() {
    let bogus = std::path::PathBuf::from("/nonexistent/xdg/override/path/012345");
    let discovery = SocketDiscovery::new("test").with_xdg_override(&bogus);
    let result = discovery.discover_primal("any-primal").await;
    assert!(result.is_none());
}

/// `discover_via_xdg` returns when the family-scoped path exists (no connect probe on this branch).
#[tokio::test]
async fn test_discover_via_xdg_family_scoped_path_exists_without_connect() {
    let temp = TempDir::new().unwrap();
    let biomeos = temp.path().join("membrane");
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
    let biomeos = temp.path().join("membrane");
    std::fs::create_dir_all(&biomeos).unwrap();
    let sock = biomeos.join("legacy-only.sock");
    std::fs::File::create(&sock).unwrap();

    let discovery = SocketDiscovery::new("fam").with_xdg_override(temp.path());
    let r = discovery.discover_primal("legacy-only").await;
    assert!(r.is_some());
}

#[tokio::test]
async fn test_discover_via_xdg_primal_family_sock_plain_file() {
    let temp = TempDir::new().unwrap();
    let biomeos = temp.path().join("membrane");
    std::fs::create_dir_all(&biomeos).unwrap();
    let plain = biomeos.join("plain-fam.sock");
    std::fs::File::create(&plain).unwrap();

    let discovery = SocketDiscovery::new("fam").with_xdg_override(temp.path());
    let r = discovery.discover_primal("plain").await;
    assert!(r.is_some());
    assert_eq!(r.unwrap().path, plain);
}

#[tokio::test]
async fn test_get_xdg_runtime_dir_helper() {
    let _ = SocketDiscovery::get_xdg_runtime_dir();
}
