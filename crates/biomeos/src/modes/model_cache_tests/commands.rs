use super::*;

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
#[tokio::test]
async fn test_run_list_after_failed_register_leaves_cache_consistent() {
    let temp = tempfile::tempdir().expect("temp dir");
    let cache_dir = temp.path().join("model-cache");
    std::fs::create_dir_all(&cache_dir).expect("create cache dir");

    let bad = run_with(
        cache_dir.clone(),
        None,
        ModelCacheCommand::Register {
            model_id: "bad/path".to_string(),
            path: temp.path().join("missing-dir-xyz"),
        },
    )
    .await;
    assert!(bad.is_err());

    let list = run_with(cache_dir, None, ModelCacheCommand::List).await;
    assert!(list.is_ok());
}

#[tokio::test]
async fn test_status_after_register_and_resolve() {
    let temp = tempfile::tempdir().expect("temp dir");
    let cache_dir = temp.path().join("model-cache");
    std::fs::create_dir_all(&cache_dir).expect("create cache dir");
    let model_dir = temp.path().join("m");
    std::fs::create_dir_all(&model_dir).expect("create");
    std::fs::write(model_dir.join("config.json"), "{}").expect("write");

    run_with(
        cache_dir.clone(),
        None,
        ModelCacheCommand::Register {
            model_id: "org/status-test".to_string(),
            path: model_dir,
        },
    )
    .await
    .expect("register");

    let st = run_with(cache_dir, None, ModelCacheCommand::Status).await;
    assert!(st.is_ok());
}

#[tokio::test]
async fn test_import_hf_then_list() {
    let temp = tempfile::tempdir().expect("temp dir");
    let cache_dir = temp.path().join("model-cache");
    let hf_hub = temp.path().join("hf-hub");
    std::fs::create_dir_all(&cache_dir).expect("cache");
    std::fs::create_dir_all(hf_hub.join("models--a--b").join("snapshots").join("h")).expect("hf");
    std::fs::write(hf_hub.join("models--a--b/snapshots/h/config.json"), "{}").expect("cfg");

    run_with(cache_dir.clone(), Some(hf_hub), ModelCacheCommand::ImportHf)
        .await
        .expect("import");

    let list = run_with(cache_dir, None, ModelCacheCommand::List).await;
    assert!(list.is_ok());
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
