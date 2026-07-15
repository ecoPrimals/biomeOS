// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::run_with;
use crate::ModelCacheCommand;

#[tokio::test]
async fn test_run_command_dispatch() {
    let temp = tempfile::tempdir().expect("temp dir");
    let cache_dir = temp.path().join("model-cache");
    std::fs::create_dir_all(&cache_dir).expect("create cache dir");

    let result = run_with(cache_dir.clone(), None, ModelCacheCommand::List).await;
    assert!(result.is_ok(), "list should succeed: {:?}", result.err());

    let result = run_with(cache_dir.clone(), None, ModelCacheCommand::Status).await;
    assert!(result.is_ok(), "status should succeed: {:?}", result.err());

    let result = run_with(
        cache_dir,
        None,
        ModelCacheCommand::Resolve {
            model_id: "test/dispatch".to_string(),
        },
    )
    .await;
    assert!(result.is_ok(), "resolve should succeed: {:?}", result.err());
}

#[tokio::test]
async fn test_run_with_corrupt_manifest_still_lists() {
    let temp = tempfile::tempdir().expect("temp dir");
    let cache_dir = temp.path().join("model-cache");
    std::fs::create_dir_all(&cache_dir).expect("create cache dir");
    std::fs::write(cache_dir.join("manifest.json"), "not valid json {{{").expect("write corrupt");

    let result = run_with(cache_dir, None, ModelCacheCommand::List).await;
    assert!(
        result.is_ok(),
        "corrupt manifest should be recovered (empty manifest): {:?}",
        result.err()
    );
}

#[tokio::test]
async fn test_run_register_model_path_is_file_errors() {
    let temp = tempfile::tempdir().expect("temp dir");
    let cache_dir = temp.path().join("model-cache");
    std::fs::create_dir_all(&cache_dir).expect("create cache dir");
    let file_path = temp.path().join("not-a-directory.bin");
    std::fs::write(&file_path, b"x").expect("write file");

    let result = run_with(
        cache_dir,
        None,
        ModelCacheCommand::Register {
            model_id: "test/file-as-model".to_string(),
            path: file_path,
        },
    )
    .await;
    assert!(
        result.is_err(),
        "register should fail when path is a file (scan/register): {:?}",
        result
    );
}
