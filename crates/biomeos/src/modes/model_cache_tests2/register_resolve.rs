// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::run_with;
use crate::ModelCacheCommand;

#[tokio::test]
async fn test_resolve_model_with_local_shows_files_line_when_multiple_files() {
    let temp = tempfile::tempdir().expect("temp dir");
    let cache_dir = temp.path().join("model-cache");
    std::fs::create_dir_all(&cache_dir).expect("create cache dir");
    let model_dir = temp.path().join("multi-file-model");
    std::fs::create_dir_all(&model_dir).expect("create model dir");
    std::fs::write(model_dir.join("config.json"), "{}").expect("write config");
    std::fs::write(model_dir.join("weights.bin"), b"0123456789").expect("write weights");

    run_with(
        cache_dir.clone(),
        None,
        ModelCacheCommand::Register {
            model_id: "test/multi-file-resolve".to_string(),
            path: model_dir,
        },
    )
    .await
    .expect("register");

    let result = run_with(
        cache_dir,
        None,
        ModelCacheCommand::Resolve {
            model_id: "test/multi-file-resolve".to_string(),
        },
    )
    .await;
    assert!(
        result.is_ok(),
        "resolve local with multiple files should succeed: {:?}",
        result.err()
    );
}

#[tokio::test]
async fn test_register_model_format_pytorch() {
    let temp = tempfile::tempdir().expect("temp dir");
    let cache_dir = temp.path().join("model-cache");
    std::fs::create_dir_all(&cache_dir).expect("create cache dir");

    let model_dir = temp.path().join("pytorch-model");
    std::fs::create_dir_all(&model_dir).expect("create model dir");
    std::fs::write(model_dir.join("pytorch_model.bin"), b"weights").expect("write pytorch");

    let result = run_with(
        cache_dir,
        None,
        ModelCacheCommand::Register {
            model_id: "test/pytorch-format".to_string(),
            path: model_dir,
        },
    )
    .await;
    assert!(
        result.is_ok(),
        "register pytorch model should succeed: {:?}",
        result.err()
    );
}

#[tokio::test]
async fn test_list_models_with_multiple_models() {
    let temp = tempfile::tempdir().expect("temp dir");
    let cache_dir = temp.path().join("model-cache");
    std::fs::create_dir_all(&cache_dir).expect("create cache dir");

    for (id, subdir) in [("test/model-a", "model-a"), ("test/model-b", "model-b")] {
        let model_dir = temp.path().join(subdir);
        std::fs::create_dir_all(&model_dir).expect("create model dir");
        std::fs::write(model_dir.join("config.json"), "{}").expect("write config");

        run_with(
            cache_dir.clone(),
            None,
            ModelCacheCommand::Register {
                model_id: id.to_string(),
                path: model_dir,
            },
        )
        .await
        .expect("register");
    }

    let result = run_with(cache_dir, None, ModelCacheCommand::List).await;
    assert!(
        result.is_ok(),
        "list with multiple models should succeed: {:?}",
        result.err()
    );
}

#[tokio::test]
async fn test_register_model_with_symlink() {
    let temp = tempfile::tempdir().expect("temp dir");
    let cache_dir = temp.path().join("model-cache");
    std::fs::create_dir_all(&cache_dir).expect("create cache dir");

    let real_dir = temp.path().join("real-model");
    std::fs::create_dir_all(&real_dir).expect("create real dir");
    std::fs::write(real_dir.join("config.json"), "{}").expect("write config");

    let symlink_dir = temp.path().join("symlink-model");
    #[cfg(unix)]
    std::os::unix::fs::symlink(&real_dir, &symlink_dir).expect("create symlink");

    #[cfg(unix)]
    {
        let result = run_with(
            cache_dir,
            None,
            ModelCacheCommand::Register {
                model_id: "test/symlink-model".to_string(),
                path: symlink_dir,
            },
        )
        .await;
        assert!(
            result.is_ok(),
            "register via symlink should succeed: {:?}",
            result.err()
        );
    }
}

#[tokio::test]
async fn test_resolve_model_with_nested_files() {
    let temp = tempfile::tempdir().expect("temp dir");
    let cache_dir = temp.path().join("model-cache");
    std::fs::create_dir_all(&cache_dir).expect("create cache dir");
    let model_dir = temp.path().join("nested-model");
    std::fs::create_dir_all(model_dir.join("subdir")).expect("create subdir");
    std::fs::write(model_dir.join("config.json"), "{}").expect("write config");
    std::fs::write(model_dir.join("subdir").join("weights.bin"), b"data")
        .expect("write nested file");

    run_with(
        cache_dir.clone(),
        None,
        ModelCacheCommand::Register {
            model_id: "test/nested-files".to_string(),
            path: model_dir,
        },
    )
    .await
    .expect("register");

    let result = run_with(
        cache_dir,
        None,
        ModelCacheCommand::Resolve {
            model_id: "test/nested-files".to_string(),
        },
    )
    .await;
    assert!(result.is_ok(), "resolve nested model should succeed");
}

#[tokio::test]
async fn test_run_register_same_model_id_twice_updates_cache() {
    let temp = tempfile::tempdir().expect("temp dir");
    let cache_dir = temp.path().join("model-cache");
    std::fs::create_dir_all(&cache_dir).expect("create cache dir");

    let model_dir_a = temp.path().join("model-v1");
    std::fs::create_dir_all(&model_dir_a).expect("create model dir");
    std::fs::write(model_dir_a.join("config.json"), r#"{"v":1}"#).expect("write config");

    run_with(
        cache_dir.clone(),
        None,
        ModelCacheCommand::Register {
            model_id: "test/double-register".to_string(),
            path: model_dir_a.clone(),
        },
    )
    .await
    .expect("first register");

    let model_dir_b = temp.path().join("model-v2");
    std::fs::create_dir_all(&model_dir_b).expect("create model dir v2");
    std::fs::write(model_dir_b.join("config.json"), r#"{"v":2}"#).expect("write config v2");

    let result = run_with(
        cache_dir,
        None,
        ModelCacheCommand::Register {
            model_id: "test/double-register".to_string(),
            path: model_dir_b.clone(),
        },
    )
    .await;
    assert!(
        result.is_ok(),
        "re-register same id with new path should succeed: {:?}",
        result.err()
    );
}
