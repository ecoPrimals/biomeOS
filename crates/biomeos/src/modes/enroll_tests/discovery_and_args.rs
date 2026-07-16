// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::*;
use std::path::PathBuf;

#[test]
fn test_get_machine_id() {
    let _ = get_machine_id();
}

#[test]
fn test_discover_security_socket_handles_missing() {
    assert!(discover_security_socket_in(None, None).is_none());
}

#[test]
fn test_enroll_args_construction() {
    let args = EnrollArgs {
        family_id: "fam123".to_string(),
        node_id: "tower".to_string(),
        device_id: Some("dev456".to_string()),
        family_seed: std::path::PathBuf::from(".family.seed"),
        lineage_seed: std::path::PathBuf::from(".lineage.seed"),
        security_socket: None,
        security_socket_dir: None,
        force: false,
    };
    assert_eq!(args.family_id, "fam123");
    assert_eq!(args.node_id, "tower");
    assert_eq!(args.device_id, Some("dev456".to_string()));
    assert!(!args.force);
    assert_eq!(args.family_seed, std::path::PathBuf::from(".family.seed"));
    assert_eq!(args.lineage_seed, std::path::PathBuf::from(".lineage.seed"));
}

#[test]
fn test_enroll_args_with_custom_paths() {
    let custom_family = PathBuf::from("/custom/.family.seed");
    let custom_lineage = PathBuf::from("/custom/.lineage.seed");
    let args = EnrollArgs {
        family_id: "f".to_string(),
        node_id: "n".to_string(),
        device_id: None,
        family_seed: custom_family.clone(),
        lineage_seed: custom_lineage.clone(),
        security_socket: Some("/tmp/beardog.sock".to_string()),
        security_socket_dir: None,
        force: true,
    };
    assert_eq!(args.family_seed, custom_family);
    assert_eq!(args.lineage_seed, custom_lineage);
    assert!(args.force);
    assert_eq!(args.security_socket, Some("/tmp/beardog.sock".to_string()));
}

#[test]
fn test_discover_security_socket_finds_default_socket() {
    let temp = tempfile::tempdir().expect("temp dir");
    let membrane_dir = temp.path().join("membrane");
    std::fs::create_dir_all(&membrane_dir).expect("create membrane dir");
    let socket_path = membrane_dir.join("beardog.sock");
    std::fs::write(&socket_path, "").expect("create socket file");

    let result = discover_security_socket_in(Some(temp.path()), None);
    assert!(
        result.is_some(),
        "Should find socket when socket_dir/membrane/beardog.sock exists"
    );
    assert!(result.unwrap().contains("beardog.sock"));
}

#[test]
fn test_discover_security_socket_finds_family_suffixed_socket() {
    let temp = tempfile::tempdir().expect("temp dir");
    let membrane_dir = temp.path().join("membrane");
    std::fs::create_dir_all(&membrane_dir).expect("create membrane dir");
    let socket_path = membrane_dir.join("beardog-testfamily123.sock");
    std::fs::write(&socket_path, "").expect("create socket file");

    let result = discover_security_socket_in(Some(temp.path()), Some("testfamily123"));
    assert!(
        result.is_some(),
        "Should find beardog-{{family_id}}.sock when socket_dir and family_id provided"
    );
    assert!(result.unwrap().contains("beardog-testfamily123.sock"));
}
