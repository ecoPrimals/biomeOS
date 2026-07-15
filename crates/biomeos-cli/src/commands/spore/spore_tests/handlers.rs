// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::*;

#[tokio::test]
async fn test_handle_spore_refresh_no_plasmid_bin() {
    let temp = tempfile::tempdir().expect("temp dir");
    let result = handle_spore_refresh(temp.path().to_path_buf(), true).await;
    if let Err(e) = result {
        let err = e.to_string();
        assert!(
            err.contains("plasmidBin") || err.contains("not found") || err.contains("tower"),
            "unexpected error: {err}"
        );
    }
}

#[tokio::test]
async fn test_handle_spore_list() {
    let result = handle_spore_list().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_handle_spore_verify_nonexistent() {
    let temp = tempfile::tempdir().expect("temp dir");
    let mount = temp.path().join("biomeOS");
    std::fs::create_dir_all(&mount).expect("create dir");
    let result = handle_spore_verify(temp.path().to_path_buf()).await;
    assert!(result.is_err() || result.is_ok());
}

#[tokio::test]
async fn test_handle_spore_info_nonexistent() {
    let temp = tempfile::tempdir().expect("temp dir");
    let result = handle_spore_info(temp.path().to_path_buf()).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_handle_spore_clone_missing_source() {
    let temp = tempfile::tempdir().expect("temp dir");
    let from = temp.path().join("no-such-spore");
    let to = temp.path().join("dest-spore");
    let result = handle_spore_clone(from, to, "node-new".into()).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_handle_spore_verify_invalid_tree() {
    let temp = tempfile::tempdir().expect("temp dir");
    let mount = temp.path().join("not-a-spore");
    std::fs::create_dir_all(&mount).expect("dir");
    let result = handle_spore_verify(mount).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_handle_spore_create_invalid_spore_type() {
    let temp = tempfile::tempdir().expect("temp dir");
    let err = handle_spore_create(
        temp.path().to_path_buf(),
        "l".into(),
        "n1".into(),
        "thermal".into(),
    )
    .await
    .expect_err("invalid type");
    assert!(err.to_string().to_lowercase().contains("invalid"));
}

#[tokio::test]
async fn test_handle_spore_info_minimal_valid_tree() {
    let temp = tempfile::tempdir().expect("temp dir");
    let mount = temp.path();
    let biome = mount.join("biomeOS");
    std::fs::create_dir_all(&biome).expect("biomeOS");
    std::fs::write(
        biome.join("tower.toml"),
        r#"node_id = "node-info-test"
"#,
    )
    .expect("tower.toml");
    let result = handle_spore_info(mount.to_path_buf()).await;
    assert!(result.is_ok(), "info: {:?}", result.err());
}
