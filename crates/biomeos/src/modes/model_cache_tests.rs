// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

#![allow(clippy::unwrap_used, clippy::expect_used)]

use super::*;
use crate::ModelCacheCommand;

#[test]
fn test_format_size_mb() {
    assert_eq!(format_size_mb(0), "0.0 MB");
    assert_eq!(format_size_mb(1_048_576), "1.0 MB");
    assert_eq!(format_size_mb(1_573_286), "1.5 MB"); // 1.5 * 1024^2
    assert_eq!(format_size_mb(104_857_600), "100.0 MB");
}

#[test]
fn test_format_size_gb() {
    assert_eq!(format_size_gb(0), "0.0 GB");
    assert_eq!(format_size_gb(1_073_741_824), "1.0 GB");
    assert_eq!(format_size_gb(2_147_483_648), "2.0 GB");
}

#[test]
fn test_hf_dir_to_model_id() {
    assert_eq!(
        hf_dir_to_model_id("models--TinyLlama--TinyLlama-1.1B-Chat-v1.0"),
        Some("TinyLlama/TinyLlama-1.1B-Chat-v1.0".to_string())
    );
    assert_eq!(
        hf_dir_to_model_id("models--meta-llama--Llama-2-7b-hf"),
        Some("meta-llama/Llama-2-7b-hf".to_string())
    );
    assert_eq!(
        hf_dir_to_model_id("models--simple"),
        Some("simple".to_string())
    );
    assert_eq!(hf_dir_to_model_id("other--prefix"), None);
    assert_eq!(hf_dir_to_model_id(""), None);
    assert_eq!(hf_dir_to_model_id("models--"), Some("".to_string()));
    assert_eq!(
        hf_dir_to_model_id("models--single--level"),
        Some("single/level".to_string())
    );
}

#[test]
fn test_format_size_mb_large() {
    assert_eq!(format_size_mb(1_073_741_824), "1024.0 MB");
    assert_eq!(format_size_mb(2_097_152_000), "2000.0 MB");
}

#[test]
fn test_format_size_gb_fractional() {
    assert_eq!(format_size_gb(1_610_612_736), "1.5 GB");
    assert_eq!(format_size_gb(5_368_709_120), "5.0 GB");
}

#[tokio::test]
async fn test_run_list_empty_cache() {
    let temp = tempfile::tempdir().expect("temp dir");
    let cache_dir = temp.path().join("model-cache");
    std::fs::create_dir_all(&cache_dir).expect("create cache dir");

    let result = run_with(cache_dir, None, ModelCacheCommand::List).await;
    assert!(result.is_ok(), "list should succeed: {:?}", result.err());
}

#[tokio::test]
async fn test_run_status() {
    let temp = tempfile::tempdir().expect("temp dir");
    let cache_dir = temp.path().join("model-cache");
    std::fs::create_dir_all(&cache_dir).expect("create cache dir");

    let result = run_with(cache_dir, None, ModelCacheCommand::Status).await;
    assert!(result.is_ok(), "status should succeed: {:?}", result.err());
}

#[tokio::test]
async fn test_run_resolve_not_found() {
    let temp = tempfile::tempdir().expect("temp dir");
    let cache_dir = temp.path().join("model-cache");
    std::fs::create_dir_all(&cache_dir).expect("create cache dir");

    let result = run_with(
        cache_dir,
        None,
        ModelCacheCommand::Resolve {
            model_id: "nonexistent/model-xyz-123".to_string(),
        },
    )
    .await;
    assert!(
        result.is_ok(),
        "resolve should succeed (NotFound path): {:?}",
        result.err()
    );
}

#[tokio::test]
async fn test_run_import_hf_empty() {
    let temp = tempfile::tempdir().expect("temp dir");
    let cache_dir = temp.path().join("model-cache");
    let hf_hub = temp.path().join("hf-hub");
    std::fs::create_dir_all(&cache_dir).expect("create cache dir");
    std::fs::create_dir_all(&hf_hub).expect("create HF hub dir");

    let result = run_with(cache_dir, Some(hf_hub), ModelCacheCommand::ImportHf).await;
    assert!(
        result.is_ok(),
        "import-hf should succeed: {:?}",
        result.err()
    );
}

#[tokio::test]
async fn test_run_register_model_nonexistent_path_fails() {
    let temp = tempfile::tempdir().expect("temp dir");
    let cache_dir = temp.path().join("model-cache");
    std::fs::create_dir_all(&cache_dir).expect("create cache dir");

    let nonexistent = temp.path().join("nonexistent-model-dir-xyz");
    let result = run_with(
        cache_dir,
        None,
        ModelCacheCommand::Register {
            model_id: "test/nonexistent".to_string(),
            path: nonexistent,
        },
    )
    .await;
    assert!(
        result.is_err(),
        "register with nonexistent path should fail: {:?}",
        result
    );
}

#[tokio::test]
async fn test_run_register_model() {
    let temp = tempfile::tempdir().expect("temp dir");
    let cache_dir = temp.path().join("model-cache");
    std::fs::create_dir_all(&cache_dir).expect("create cache dir");

    let model_dir = temp.path().join("test-model");
    std::fs::create_dir_all(&model_dir).expect("create model dir");
    std::fs::write(model_dir.join("config.json"), "{}").expect("write config");

    let result = run_with(
        cache_dir,
        None,
        ModelCacheCommand::Register {
            model_id: "test/register-model".to_string(),
            path: model_dir,
        },
    )
    .await;
    assert!(
        result.is_ok(),
        "register should succeed: {:?}",
        result.err()
    );
}

#[tokio::test]
async fn test_run_list_with_models() {
    let temp = tempfile::tempdir().expect("temp dir");
    let cache_dir = temp.path().join("model-cache");
    std::fs::create_dir_all(&cache_dir).expect("create cache dir");

    let model_dir = temp.path().join("list-test-model");
    std::fs::create_dir_all(&model_dir).expect("create model dir");
    std::fs::write(model_dir.join("config.json"), "{}").expect("write config");

    run_with(
        cache_dir.clone(),
        None,
        ModelCacheCommand::Register {
            model_id: "test/list-model".to_string(),
            path: model_dir,
        },
    )
    .await
    .expect("register should succeed");

    let result = run_with(cache_dir, None, ModelCacheCommand::List).await;
    assert!(result.is_ok(), "list should succeed: {:?}", result.err());
}

#[tokio::test]
async fn test_run_resolve_local() {
    let temp = tempfile::tempdir().expect("temp dir");
    let cache_dir = temp.path().join("model-cache");
    std::fs::create_dir_all(&cache_dir).expect("create cache dir");

    let model_dir = temp.path().join("resolve-test-model");
    std::fs::create_dir_all(&model_dir).expect("create model dir");
    std::fs::write(model_dir.join("config.json"), "{}").expect("write config");

    run_with(
        cache_dir.clone(),
        None,
        ModelCacheCommand::Register {
            model_id: "test/resolve-local".to_string(),
            path: model_dir.clone(),
        },
    )
    .await
    .expect("register should succeed");

    let result = run_with(
        cache_dir,
        None,
        ModelCacheCommand::Resolve {
            model_id: "test/resolve-local".to_string(),
        },
    )
    .await;
    assert!(result.is_ok(), "resolve should succeed: {:?}", result.err());
}

#[tokio::test]
async fn test_show_status_with_hf_cache() {
    let temp = tempfile::tempdir().expect("temp dir");
    let cache_dir = temp.path().join("model-cache");
    std::fs::create_dir_all(&cache_dir).expect("create cache dir");
    let hf_hub = temp.path().join("hf-hub");
    std::fs::create_dir_all(&hf_hub).expect("create HF hub dir");
    std::fs::create_dir_all(hf_hub.join("models--test--model-xyz")).expect("create HF model dir");

    let result = run_with(cache_dir, Some(hf_hub), ModelCacheCommand::Status).await;
    assert!(
        result.is_ok(),
        "status with HF cache should succeed: {:?}",
        result.err()
    );
}

#[tokio::test]
async fn test_show_status_with_hf_cache_and_unregistered() {
    let temp = tempfile::tempdir().expect("temp dir");
    let cache_dir = temp.path().join("model-cache");
    std::fs::create_dir_all(&cache_dir).expect("create cache dir");
    let hf_hub = temp.path().join("hf-hub");
    std::fs::create_dir_all(hf_hub.join("models--unreg--model")).expect("create HF model dir");

    let result = run_with(cache_dir, Some(hf_hub), ModelCacheCommand::Status).await;
    assert!(
        result.is_ok(),
        "status with unregistered HF models should succeed: {:?}",
        result.err()
    );
}

#[test]
fn test_format_size_mb_zero() {
    assert_eq!(format_size_mb(0), "0.0 MB");
}

#[test]
fn test_format_size_gb_zero() {
    assert_eq!(format_size_gb(0), "0.0 GB");
}

#[tokio::test]
async fn test_run_import_hf_with_existing_models() {
    let temp = tempfile::tempdir().expect("temp dir");
    let cache_dir = temp.path().join("model-cache");
    std::fs::create_dir_all(&cache_dir).expect("create cache dir");
    let model_dir = temp.path().join("existing-model");
    std::fs::create_dir_all(&model_dir).expect("create model dir");
    std::fs::write(model_dir.join("config.json"), "{}").expect("write config");

    run_with(
        cache_dir.clone(),
        None,
        ModelCacheCommand::Register {
            model_id: "test/existing".to_string(),
            path: model_dir,
        },
    )
    .await
    .expect("register");

    let result = run_with(cache_dir.clone(), None, ModelCacheCommand::ImportHf).await;
    assert!(
        result.is_ok(),
        "import-hf with existing models should succeed: {:?}",
        result.err()
    );
}

#[tokio::test]
async fn test_run_resolve_with_empty_model_id() {
    let temp = tempfile::tempdir().expect("temp dir");
    let cache_dir = temp.path().join("model-cache");
    std::fs::create_dir_all(&cache_dir).expect("create cache dir");

    let result = run_with(
        cache_dir,
        None,
        ModelCacheCommand::Resolve {
            model_id: "".to_string(),
        },
    )
    .await;
    assert!(result.is_ok(), "resolve empty model_id should not panic");
}

#[tokio::test]
async fn test_run_status_no_hf_path() {
    let temp = tempfile::tempdir().expect("temp dir");
    let cache_dir = temp.path().join("model-cache");
    std::fs::create_dir_all(&cache_dir).expect("create cache dir");

    let result = run_with(cache_dir, None, ModelCacheCommand::Status).await;
    assert!(result.is_ok(), "status without HF should succeed");
}

#[test]
fn test_hf_dir_to_model_id_edge_cases() {
    assert_eq!(
        hf_dir_to_model_id("models--a--b--c"),
        Some("a/b/c".to_string())
    );
    assert_eq!(
        hf_dir_to_model_id("models--single"),
        Some("single".to_string())
    );
}

#[test]
fn test_format_size_mb_large_values() {
    // 1 TB in bytes
    assert_eq!(format_size_mb(1_099_511_627_776), "1048576.0 MB");
    // Edge: single byte
    assert_eq!(format_size_mb(1), "0.0 MB");
}

#[test]
fn test_format_size_gb_large_values() {
    assert_eq!(format_size_gb(10_737_418_240), "10.0 GB");
    assert_eq!(format_size_gb(1), "0.0 GB");
}

#[test]
fn test_hf_dir_to_model_id_multiple_dashes() {
    assert_eq!(
        hf_dir_to_model_id("models--org--repo--sub--path"),
        Some("org/repo/sub/path".to_string())
    );
}

#[test]
fn test_hf_dir_to_model_id_no_prefix() {
    assert_eq!(hf_dir_to_model_id("random-dir"), None);
    assert_eq!(hf_dir_to_model_id("models"), None);
}

#[tokio::test]
async fn test_run_import_hf_with_importable_model() {
    let temp = tempfile::tempdir().expect("temp dir");
    let cache_dir = temp.path().join("model-cache");
    let hf_hub = temp.path().join("hf-hub");
    std::fs::create_dir_all(&cache_dir).expect("create cache dir");
    let model_dir = hf_hub
        .join("models--test--importable-model")
        .join("snapshots")
        .join("abc123hash");
    std::fs::create_dir_all(&model_dir).expect("create HF model structure");
    std::fs::write(model_dir.join("config.json"), "{}").expect("write config");

    let result = run_with(cache_dir, Some(hf_hub), ModelCacheCommand::ImportHf).await;
    assert!(
        result.is_ok(),
        "import-hf with valid HF structure should succeed: {:?}",
        result.err()
    );
}

#[tokio::test]
async fn test_resolve_local_with_files() {
    let temp = tempfile::tempdir().expect("temp dir");
    let cache_dir = temp.path().join("model-cache");
    std::fs::create_dir_all(&cache_dir).expect("create cache dir");
    let model_dir = temp.path().join("multi-file-model");
    std::fs::create_dir_all(&model_dir).expect("create model dir");
    std::fs::write(model_dir.join("config.json"), "{}").expect("write config");
    std::fs::write(model_dir.join("model.safetensors"), b"data").expect("write safetensors");

    run_with(
        cache_dir.clone(),
        None,
        ModelCacheCommand::Register {
            model_id: "test/multi-file".to_string(),
            path: model_dir,
        },
    )
    .await
    .expect("register");

    let result = run_with(
        cache_dir,
        None,
        ModelCacheCommand::Resolve {
            model_id: "test/multi-file".to_string(),
        },
    )
    .await;
    assert!(result.is_ok(), "resolve multi-file model should succeed");
}

#[test]
fn test_format_size_mb_small_fractional() {
    assert_eq!(format_size_mb(524_288), "0.5 MB");
}

#[test]
fn test_format_size_gb_small() {
    assert_eq!(format_size_gb(536_870_912), "0.5 GB");
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
