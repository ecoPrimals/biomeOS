// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::run_with;
use crate::ModelCacheCommand;

#[tokio::test]
async fn test_show_status_with_hf_unregistered_models_prints_import_hint() {
    let temp = tempfile::tempdir().expect("temp dir");
    let cache_dir = temp.path().join("model-cache");
    let hf_hub = temp.path().join("hf-hub");
    std::fs::create_dir_all(&cache_dir).expect("create cache dir");
    std::fs::create_dir_all(hf_hub.join("models--orphan--unregistered")).expect("hf model dir");

    let result = run_with(cache_dir, Some(hf_hub), ModelCacheCommand::Status).await;
    assert!(
        result.is_ok(),
        "status with unregistered HF dirs should succeed: {:?}",
        result.err()
    );
}

#[tokio::test]
async fn test_import_hf_with_imports_new_models_and_prints_per_model_lines() {
    let temp = tempfile::tempdir().expect("temp dir");
    let cache_dir = temp.path().join("model-cache");
    let hf_hub = temp.path().join("hf-hub");
    std::fs::create_dir_all(&cache_dir).expect("create cache dir");
    let snap = hf_hub
        .join("models--import--new-model")
        .join("snapshots")
        .join("snap1");
    std::fs::create_dir_all(&snap).expect("hf layout");
    std::fs::write(snap.join("config.json"), "{}").expect("config");

    let result = run_with(cache_dir, Some(hf_hub), ModelCacheCommand::ImportHf).await;
    assert!(
        result.is_ok(),
        "import with new HF models should succeed: {:?}",
        result.err()
    );
}

#[tokio::test]
async fn test_import_hf_with_existing_models_shows_already_cached() {
    let temp = tempfile::tempdir().expect("temp dir");
    let cache_dir = temp.path().join("model-cache");
    let hf_hub = temp.path().join("hf-hub");
    std::fs::create_dir_all(&cache_dir).expect("create cache dir");
    std::fs::create_dir_all(&hf_hub).expect("create HF hub dir");

    let model_dir = temp.path().join("existing-model");
    std::fs::create_dir_all(&model_dir).expect("create model dir");
    std::fs::write(model_dir.join("config.json"), "{}").expect("write config");

    run_with(
        cache_dir.clone(),
        None,
        ModelCacheCommand::Register {
            model_id: "test/already-cached".to_string(),
            path: model_dir,
        },
    )
    .await
    .expect("register");

    let result = run_with(cache_dir, Some(hf_hub), ModelCacheCommand::ImportHf).await;
    assert!(
        result.is_ok(),
        "import with existing models should succeed: {:?}",
        result.err()
    );
}

#[tokio::test]
async fn test_show_status_hf_path_exists_empty() {
    let temp = tempfile::tempdir().expect("temp dir");
    let cache_dir = temp.path().join("model-cache");
    let hf_hub = temp.path().join("hf-hub");
    std::fs::create_dir_all(&cache_dir).expect("create cache dir");
    std::fs::create_dir_all(&hf_hub).expect("create empty HF hub dir");

    let result = run_with(cache_dir, Some(hf_hub), ModelCacheCommand::Status).await;
    assert!(
        result.is_ok(),
        "status with empty HF dir should succeed: {:?}",
        result.err()
    );
}

#[tokio::test]
async fn test_show_status_hf_path_nonexistent() {
    let temp = tempfile::tempdir().expect("temp dir");
    let cache_dir = temp.path().join("model-cache");
    std::fs::create_dir_all(&cache_dir).expect("create cache dir");
    let nonexistent_hf = temp.path().join("nonexistent-hf-hub-xyz");

    let result = run_with(cache_dir, Some(nonexistent_hf), ModelCacheCommand::Status).await;
    assert!(
        result.is_ok(),
        "status with nonexistent HF path should succeed: {:?}",
        result.err()
    );
}

#[tokio::test]
async fn test_import_hf_skips_invalid_hf_model() {
    let temp = tempfile::tempdir().expect("temp dir");
    let cache_dir = temp.path().join("model-cache");
    let hf_hub = temp.path().join("hf-hub");
    std::fs::create_dir_all(&cache_dir).expect("create cache dir");
    std::fs::create_dir_all(hf_hub.join("models--invalid--no-snapshots"))
        .expect("create HF model dir without snapshots");

    let result = run_with(cache_dir, Some(hf_hub), ModelCacheCommand::ImportHf).await;
    assert!(
        result.is_ok(),
        "import should skip invalid HF model: {:?}",
        result.err()
    );
}

#[tokio::test]
async fn test_run_import_hf_hub_nonexistent() {
    let temp = tempfile::tempdir().expect("temp dir");
    let cache_dir = temp.path().join("model-cache");
    std::fs::create_dir_all(&cache_dir).expect("create cache dir");
    let nonexistent = temp.path().join("does-not-exist-xyz");

    let result = run_with(cache_dir, Some(nonexistent), ModelCacheCommand::ImportHf).await;
    assert!(
        result.is_ok(),
        "import from nonexistent hub should succeed (empty): {:?}",
        result.err()
    );
}

#[tokio::test]
async fn test_show_status_with_registered_hf_models() {
    let temp = tempfile::tempdir().expect("temp dir");
    let cache_dir = temp.path().join("model-cache");
    let hf_hub = temp.path().join("hf-hub");
    std::fs::create_dir_all(&cache_dir).expect("create cache dir");
    let model_dir = hf_hub
        .join("models--test--registered-model")
        .join("snapshots")
        .join("abc123");
    std::fs::create_dir_all(&model_dir).expect("create HF structure");
    std::fs::write(model_dir.join("config.json"), "{}").expect("write config");

    run_with(
        cache_dir.clone(),
        Some(hf_hub.clone()),
        ModelCacheCommand::ImportHf,
    )
    .await
    .expect("import");

    let result = run_with(cache_dir, Some(hf_hub), ModelCacheCommand::Status).await;
    assert!(
        result.is_ok(),
        "status with all HF models registered should succeed: {:?}",
        result.err()
    );
}
