// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::engine::SocketDiscovery;
use super::super::result::DiscoveryMethod;
use tempfile::TempDir;

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
