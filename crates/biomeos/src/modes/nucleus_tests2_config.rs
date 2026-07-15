// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::*;
use std::path::PathBuf;

#[test]
fn test_socket_path_for_capability_unknown_returns_unknown_sock() {
    let path = socket_path_for_capability(
        std::path::Path::new("/tmp"),
        "fam1",
        "arbitrary-unknown-capability",
    );
    assert!(path.to_string_lossy().contains("unknown"));
    assert!(path.to_string_lossy().ends_with(".sock"));
}

#[test]
fn test_resolve_startup_config_family_from_env_when_not_provided() {
    let config = resolve_startup_config_with("tower", "node1", None, Some("/tmp/sock"));
    assert!(config.is_ok());
    let config = config.unwrap();
    assert!(!config.family_id.is_empty());
}

#[test]
fn test_format_nucleus_summary_coordinated_mode() {
    let lines = format_nucleus_summary(
        &[("beardog".to_string(), 999)],
        std::path::Path::new("/run/sock"),
        "fam1",
        "node1",
        NucleusMode::Full,
        "coordinated",
    );
    assert!(lines.iter().any(|l| l.contains("coordinated")));
    assert!(lines.iter().any(|l| l.contains("999")));
}

#[test]
fn test_generate_jwt_secret_produces_nonempty_string() {
    let secret = generate_jwt_secret();
    assert!(!secret.is_empty());
    assert!(secret.len() > 32, "JWT secret should be substantial");
}

#[test]
fn test_resolve_startup_config_uses_biomeos_socket_dir_env() {
    let config = resolve_startup_config_with(
        "tower",
        "node1",
        Some("fam1"),
        Some("/tmp/nucleus-env-test"),
    )
    .expect("should succeed");
    assert_eq!(config.socket_dir, PathBuf::from("/tmp/nucleus-env-test"));
}

#[test]
fn test_nucleus_mode_debug() {
    let _ = format!("{:?}", NucleusMode::Full);
}

#[test]
fn test_resolve_startup_config_with_explicit_family() {
    let c =
        resolve_startup_config_with("nest", "n1", Some("myfam"), Some("/tmp/sock-nest")).unwrap();
    assert_eq!(c.family_id, "myfam");
    assert!(matches!(c.mode, NucleusMode::Nest));
}

#[test]
fn test_socket_path_for_capability_registry_alias() {
    // "registry" is a taxonomy alias for Discovery → songbird
    let p = socket_path_for_capability(std::path::Path::new("/run"), "fam", "registry");
    assert!(p.to_string_lossy().contains("songbird"));
}

#[test]
fn test_format_nucleus_summary_full_mode_label() {
    let lines = format_nucleus_summary(
        &[],
        std::path::Path::new("/x"),
        "f",
        "n",
        NucleusMode::Full,
        "bootstrap",
    );
    assert!(lines.iter().any(|l| l.contains("Full")));
}

#[test]
fn test_nucleus_mode_from_str_nucleus_alias() {
    let m: NucleusMode = "nucleus".parse().expect("parse");
    assert!(matches!(m, NucleusMode::Full));
}

#[test]
fn test_socket_path_for_capability_encryption_alias() {
    let p = socket_path_for_capability(std::path::Path::new("/s"), "fam", "encryption");
    assert!(p.to_string_lossy().contains("beardog"));
}

#[test]
fn test_resolve_startup_config_with_explicit_family_override() {
    let c = resolve_startup_config_with("full", "n1", Some("explicit-fam"), Some("/tmp/sock-full"))
        .unwrap();
    assert_eq!(c.family_id, "explicit-fam");
    assert!(matches!(c.mode, NucleusMode::Full));
}

#[test]
fn test_format_nucleus_summary_includes_socket_paths_for_all_children() {
    let lines = format_nucleus_summary(
        &[("beardog".to_string(), 10), ("songbird".to_string(), 11)],
        std::path::Path::new("/run/s"),
        "fam",
        "node",
        NucleusMode::Nest,
        "bootstrap",
    );
    assert!(lines.iter().any(|l| l.contains("beardog")));
    assert!(lines.iter().any(|l| l.contains("songbird")));
}

#[test]
fn test_nucleus_mode_clone_and_copy() {
    let mode = NucleusMode::Full;
    let cloned = mode;
    assert!(matches!(cloned, NucleusMode::Full));
}
