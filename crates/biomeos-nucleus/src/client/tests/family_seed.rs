// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::family_seed::load_family_seed_from_storage_with;

#[test]
fn test_get_family_seed_from_env_valid_base64() {
    use base64::{Engine, engine::general_purpose::STANDARD};
    let seed = b"family-seed-bytes";
    let b64 = STANDARD.encode(seed);
    let out = load_family_seed_from_storage_with(Some(b64.as_str()), None, false);
    assert_eq!(out.as_ref(), seed);
}

#[test]
fn test_get_family_seed_prefers_env_over_xdg_runtime() {
    use base64::{Engine, engine::general_purpose::STANDARD};
    let seed = b"env-wins";
    let b64 = STANDARD.encode(seed);
    let temp = tempfile::tempdir().expect("tempdir");
    let seed_path = temp.path().join("biomeos").join("family.seed");
    std::fs::create_dir_all(seed_path.parent().unwrap()).unwrap();
    std::fs::write(&seed_path, b"from-xdg-file").unwrap();

    let out = load_family_seed_from_storage_with(Some(b64.as_str()), Some(temp.path()), false);
    assert_eq!(out.as_ref(), seed);
}

#[test]
fn test_get_family_seed_invalid_base64_ignored() {
    let out = load_family_seed_from_storage_with(Some("@@@not-base64@@@"), None, false);
    assert!(out.is_empty());
}

#[test]
fn test_get_family_seed_from_xdg_file_when_env_unset() {
    let temp = tempfile::tempdir().expect("tempdir");
    let seed_path = temp.path().join("membrane").join("family.seed");
    std::fs::create_dir_all(seed_path.parent().unwrap()).unwrap();
    std::fs::write(&seed_path, b"seed-from-xdg").unwrap();
    let out = load_family_seed_from_storage_with(None, Some(temp.path()), false);
    assert_eq!(out.as_ref(), b"seed-from-xdg");
}

#[test]
fn test_get_family_seed_empty_when_no_sources() {
    let out = load_family_seed_from_storage_with(None, None, false);
    assert!(out.is_empty());
}
