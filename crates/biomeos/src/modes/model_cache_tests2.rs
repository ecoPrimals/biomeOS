// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Continuation of model-cache CLI tests (split from `model_cache_tests.rs`).

#![expect(clippy::expect_used, reason = "test assertions use expect for clarity")]

use super::*;
use crate::ModelCacheCommand;
use biomeos_test_utils::TestEnvGuard;
use serial_test::serial;

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

#[tokio::test]
#[serial]
async fn test_run_list_uses_home_for_default_model_cache() {
    let temp = tempfile::tempdir().expect("temp dir");
    let home = temp.path().to_string_lossy();
    let _guard = TestEnvGuard::set("HOME", home.as_ref());
    let result = run(ModelCacheCommand::List).await;
    assert!(
        result.is_ok(),
        "production run(List) with isolated HOME should succeed: {:?}",
        result.err()
    );
}

#[tokio::test]
#[serial]
async fn test_run_status_production_uses_home_for_hf_scan() {
    let temp = tempfile::tempdir().expect("temp dir");
    let home = temp.path();
    let _guard = TestEnvGuard::set("HOME", home.to_string_lossy().as_ref());
    let hub = home.join(".cache/huggingface/hub");
    std::fs::create_dir_all(hub.join("models--org--model-xyz")).expect("hf hub layout");
    let result = run(ModelCacheCommand::Status).await;
    assert!(
        result.is_ok(),
        "production run(Status) with HF cache under HOME: {:?}",
        result.err()
    );
}

#[tokio::test]
#[serial]
async fn test_run_resolve_production_not_found() {
    let temp = tempfile::tempdir().expect("temp dir");
    let _guard = TestEnvGuard::set("HOME", temp.path().to_string_lossy().as_ref());
    let result = run(ModelCacheCommand::Resolve {
        model_id: "no/such/model-for-run-test".to_string(),
    })
    .await;
    assert!(result.is_ok(), "resolve NotFound path: {:?}", result.err());
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
#[serial]
async fn test_run_import_hf_production_shows_already_cached_section() {
    let temp = tempfile::tempdir().expect("temp");
    let home = temp.path();
    let _guard = TestEnvGuard::set("HOME", home.to_string_lossy().as_ref());
    let cache_dir = home.join(".biomeos/model-cache");
    std::fs::create_dir_all(&cache_dir).expect("cache");

    let model_dir = temp.path().join("pre-for-import-prod");
    std::fs::create_dir_all(&model_dir).expect("create");
    std::fs::write(model_dir.join("config.json"), "{}").expect("write");

    run_with(
        cache_dir.clone(),
        None,
        ModelCacheCommand::Register {
            model_id: "test/import-prod-existing".to_string(),
            path: model_dir,
        },
    )
    .await
    .expect("register");

    let hub = home.join(".cache/huggingface/hub");
    std::fs::create_dir_all(&hub).expect("hub");

    let result = run(ModelCacheCommand::ImportHf).await;
    assert!(
        result.is_ok(),
        "production ImportHf with prior cache: {:?}",
        result.err()
    );
}

#[tokio::test]
#[serial]
async fn test_run_import_hf_production_imports_from_default_hf_hub_layout() {
    let temp = tempfile::tempdir().expect("temp");
    let home = temp.path();
    let _guard = TestEnvGuard::set("HOME", home.to_string_lossy().as_ref());
    let cache_dir = home.join(".biomeos/model-cache");
    std::fs::create_dir_all(&cache_dir).expect("cache");

    let hub = home.join(".cache/huggingface/hub");
    let snap = hub
        .join("models--prod--default-hub")
        .join("snapshots")
        .join("snapsha");
    std::fs::create_dir_all(&snap).expect("hf layout");
    std::fs::write(snap.join("config.json"), "{}").expect("config");

    let result = run(ModelCacheCommand::ImportHf).await;
    assert!(
        result.is_ok(),
        "production ImportHf from ~/.cache/huggingface/hub: {:?}",
        result.err()
    );
}

#[tokio::test]
#[serial]
async fn test_run_resolve_production_local_entry() {
    let temp = tempfile::tempdir().expect("temp");
    let home = temp.path();
    let _guard = TestEnvGuard::set("HOME", home.to_string_lossy().as_ref());
    let cache_dir = home.join(".biomeos/model-cache");
    std::fs::create_dir_all(&cache_dir).expect("cache");

    let model_dir = temp.path().join("resolve-prod-model");
    std::fs::create_dir_all(&model_dir).expect("create");
    std::fs::write(model_dir.join("config.json"), "{}").expect("write");

    run_with(
        cache_dir.clone(),
        None,
        ModelCacheCommand::Register {
            model_id: "test/resolve-production-local".to_string(),
            path: model_dir,
        },
    )
    .await
    .expect("register");

    let result = run(ModelCacheCommand::Resolve {
        model_id: "test/resolve-production-local".to_string(),
    })
    .await;
    assert!(
        result.is_ok(),
        "production resolve local branch: {:?}",
        result.err()
    );
}

#[tokio::test]
#[serial]
async fn test_run_status_production_hf_unregistered_count_and_hint() {
    let temp = tempfile::tempdir().expect("temp");
    let home = temp.path();
    let _guard = TestEnvGuard::set("HOME", home.to_string_lossy().as_ref());
    let hub = home.join(".cache/huggingface/hub");
    std::fs::create_dir_all(hub.join("models--orphan--model-xyz")).expect("hf model dir");

    let result = run(ModelCacheCommand::Status).await;
    assert!(
        result.is_ok(),
        "production status with unregistered HF dirs: {:?}",
        result.err()
    );
}

#[tokio::test]
#[serial]
async fn test_run_register_production_success_path() {
    let temp = tempfile::tempdir().expect("temp");
    let home = temp.path();
    let _guard = TestEnvGuard::set("HOME", home.to_string_lossy().as_ref());
    let cache_dir = home.join(".biomeos/model-cache");
    std::fs::create_dir_all(&cache_dir).expect("cache");

    let model_dir = temp.path().join("register-prod-model");
    std::fs::create_dir_all(&model_dir).expect("create");
    std::fs::write(model_dir.join("config.json"), "{}").expect("write");

    let result = run(ModelCacheCommand::Register {
        model_id: "test/register-production-path".to_string(),
        path: model_dir,
    })
    .await;
    assert!(result.is_ok(), "production register: {:?}", result.err());
}
