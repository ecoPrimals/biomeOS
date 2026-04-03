// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Unit tests for [`super::ModelCache`] (local cache, HF import, mesh stubs).

#![allow(clippy::unwrap_used)]

use super::ModelCache;
use super::types::{ModelCacheConfig, ModelResolution};

#[tokio::test]
async fn test_has_model_false_for_nonexistent() {
    let tmp = tempfile::tempdir().unwrap();
    let cache = ModelCache::with_cache_dir(tmp.path().to_path_buf())
        .await
        .unwrap();
    assert!(!cache.has_model("nonexistent/model"));
}

#[tokio::test]
async fn test_get_model_path_none_for_nonexistent() {
    let tmp = tempfile::tempdir().unwrap();
    let cache = ModelCache::with_cache_dir(tmp.path().to_path_buf())
        .await
        .unwrap();
    assert!(cache.get_model_path("nonexistent/model").is_none());
    assert!(cache.get_model("nonexistent/model").is_none());
}

#[tokio::test]
async fn test_list_models_empty() {
    let tmp = tempfile::tempdir().unwrap();
    let cache = ModelCache::with_cache_dir(tmp.path().to_path_buf())
        .await
        .unwrap();
    assert!(cache.list_models().is_empty());
}

#[tokio::test]
async fn test_register_model_validates_path_exists() {
    let tmp = tempfile::tempdir().unwrap();
    let mut cache = ModelCache::with_cache_dir(tmp.path().join("cache"))
        .await
        .unwrap();
    let result = cache
        .register_model("m", tmp.path().join("nonexistent").as_path(), "src")
        .await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("does not exist"));
}

#[tokio::test]
async fn test_resolve_returns_not_found_when_empty() {
    let tmp = tempfile::tempdir().unwrap();
    let cache = ModelCache::with_cache_dir(tmp.path().to_path_buf())
        .await
        .unwrap();
    let res = cache.resolve("any/model").await;
    assert!(matches!(res, ModelResolution::NotFound));
}

#[tokio::test]
async fn test_register_and_resolve_local() {
    let tmp = tempfile::tempdir().unwrap();
    let model_dir = tmp.path().join("m");
    std::fs::create_dir_all(&model_dir).unwrap();
    std::fs::write(model_dir.join("model.safetensors"), b"data").unwrap();

    let mut cache = ModelCache::with_cache_dir(tmp.path().join("cache"))
        .await
        .unwrap();
    cache
        .register_model("test/m", &model_dir, "test://")
        .await
        .unwrap();

    let res = cache.resolve("test/m").await;
    match res {
        ModelResolution::Local(e) => {
            assert_eq!(e.model_id, "test/m");
            assert_eq!(e.format, "safetensors");
        }
        _ => panic!("expected Local"),
    }
}

#[tokio::test]
async fn test_scan_model_dir_nested() {
    let tmp = tempfile::tempdir().unwrap();
    let model_dir = tmp.path().join("nested");
    std::fs::create_dir_all(model_dir.join("subdir")).unwrap();
    std::fs::write(model_dir.join("a.bin"), b"a").unwrap();
    std::fs::write(model_dir.join("subdir").join("b.bin"), b"b").unwrap();

    let mut cache = ModelCache::with_cache_dir(tmp.path().join("cache"))
        .await
        .unwrap();
    cache
        .register_model("nested", &model_dir, "test://")
        .await
        .unwrap();

    let entry = cache.get_model("nested").unwrap();
    assert!(entry.size_bytes >= 2);
    assert!(entry.files.len() >= 2);
}

#[tokio::test]
async fn test_detect_format_gguf() {
    let tmp = tempfile::tempdir().unwrap();
    let model_dir = tmp.path().join("gguf");
    std::fs::create_dir_all(&model_dir).unwrap();
    std::fs::write(model_dir.join("model.gguf"), b"gguf").unwrap();

    let mut cache = ModelCache::with_cache_dir(tmp.path().join("cache"))
        .await
        .unwrap();
    cache
        .register_model("gguf/m", &model_dir, "test://")
        .await
        .unwrap();
    assert_eq!(cache.get_model("gguf/m").unwrap().format, "gguf");
}

#[tokio::test]
async fn test_detect_format_pytorch() {
    let tmp = tempfile::tempdir().unwrap();
    let model_dir = tmp.path().join("pytorch");
    std::fs::create_dir_all(&model_dir).unwrap();
    std::fs::write(model_dir.join("pytorch_model.bin"), b"pytorch").unwrap();

    let mut cache = ModelCache::with_cache_dir(tmp.path().join("cache"))
        .await
        .unwrap();
    cache
        .register_model("pytorch/m", &model_dir, "test://")
        .await
        .unwrap();
    assert_eq!(cache.get_model("pytorch/m").unwrap().format, "pytorch");
}

#[tokio::test]
async fn test_has_model_true_after_register() {
    let tmp = tempfile::tempdir().unwrap();
    let model_dir = tmp.path().join("m");
    std::fs::create_dir_all(&model_dir).unwrap();
    std::fs::write(model_dir.join("model.safetensors"), b"x").unwrap();

    let mut cache = ModelCache::with_cache_dir(tmp.path().join("cache"))
        .await
        .unwrap();
    assert!(!cache.has_model("test/m"));
    cache
        .register_model("test/m", &model_dir, "test://")
        .await
        .unwrap();
    assert!(cache.has_model("test/m"));
}

#[tokio::test]
async fn test_list_models_after_register() {
    let tmp = tempfile::tempdir().unwrap();
    let model_dir = tmp.path().join("m");
    std::fs::create_dir_all(&model_dir).unwrap();
    std::fs::write(model_dir.join("model.safetensors"), b"x").unwrap();

    let mut cache = ModelCache::with_cache_dir(tmp.path().join("cache"))
        .await
        .unwrap();
    cache
        .register_model("list/test", &model_dir, "test://")
        .await
        .unwrap();

    let models = cache.list_models();
    assert_eq!(models.len(), 1);
    assert_eq!(models[0].model_id, "list/test");
}

#[tokio::test]
async fn test_import_huggingface_cache_empty_dir() {
    let tmp = tempfile::tempdir().unwrap();
    let empty_hub = tmp.path().join("hub");
    std::fs::create_dir_all(&empty_hub).unwrap();

    let mut cache = ModelCache::with_cache_dir(tmp.path().join("cache"))
        .await
        .unwrap();
    let imported = cache
        .import_huggingface_cache_from(&empty_hub)
        .await
        .unwrap();
    assert!(imported.is_empty());
}

#[tokio::test]
async fn test_import_huggingface_cache_nonexistent_dir() {
    let tmp = tempfile::tempdir().unwrap();
    let mut cache = ModelCache::with_cache_dir(tmp.path().join("cache"))
        .await
        .unwrap();
    let imported = cache
        .import_huggingface_cache_from(&tmp.path().join("nonexistent"))
        .await
        .unwrap();
    assert!(imported.is_empty());
}

#[tokio::test]
async fn test_register_huggingface_model_not_found() {
    let tmp = tempfile::tempdir().unwrap();
    let hf_hub = tmp.path().join("hub");
    std::fs::create_dir_all(&hf_hub).unwrap();

    let mut cache = ModelCache::with_cache_dir(tmp.path().join("cache"))
        .await
        .unwrap();
    let result = cache
        .register_huggingface_model_from_hub("nonexistent/model", &hf_hub)
        .await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not found"));
}

#[tokio::test]
async fn test_find_on_mesh_no_nestgate() {
    let tmp = tempfile::tempdir().unwrap();
    let cache = ModelCache::with_cache_dir(tmp.path().to_path_buf())
        .await
        .unwrap();
    // With no NestGate, find_on_mesh returns None
    let result = cache.find_on_mesh("any/model").await;
    assert!(result.is_none());
}

#[tokio::test]
async fn test_list_mesh_models_no_nestgate() {
    let tmp = tempfile::tempdir().unwrap();
    let cache = ModelCache::with_cache_dir(tmp.path().to_path_buf())
        .await
        .unwrap();
    let models = cache.list_mesh_models().await;
    assert!(models.is_empty());
}

#[tokio::test]
async fn test_manifest_persists_after_register() {
    let tmp = tempfile::tempdir().unwrap();
    let model_dir = tmp.path().join("m");
    std::fs::create_dir_all(&model_dir).unwrap();
    std::fs::write(model_dir.join("model.safetensors"), b"x").unwrap();

    let cache_dir = tmp.path().join("cache");
    {
        let mut cache = ModelCache::with_cache_dir(cache_dir.clone()).await.unwrap();
        cache
            .register_model("persist/test", &model_dir, "test://")
            .await
            .unwrap();
    }

    let cache2 = ModelCache::with_cache_dir(cache_dir).await.unwrap();
    assert!(cache2.has_model("persist/test"));
}

#[tokio::test]
async fn test_has_model_false_when_path_deleted() {
    let tmp = tempfile::tempdir().unwrap();
    let model_dir = tmp.path().join("deleted");
    std::fs::create_dir_all(&model_dir).unwrap();
    std::fs::write(model_dir.join("model.safetensors"), b"x").unwrap();

    let mut cache = ModelCache::with_cache_dir(tmp.path().join("cache"))
        .await
        .unwrap();
    cache
        .register_model("deleted/m", &model_dir, "test://")
        .await
        .unwrap();
    assert!(cache.has_model("deleted/m"));

    std::fs::remove_dir_all(&model_dir).unwrap();
    assert!(!cache.has_model("deleted/m"));
}

#[tokio::test]
async fn test_get_model_path_none_when_deleted() {
    let tmp = tempfile::tempdir().unwrap();
    let model_dir = tmp.path().join("m");
    std::fs::create_dir_all(&model_dir).unwrap();
    std::fs::write(model_dir.join("model.safetensors"), b"x").unwrap();

    let mut cache = ModelCache::with_cache_dir(tmp.path().join("cache"))
        .await
        .unwrap();
    cache
        .register_model("path/test", &model_dir, "test://")
        .await
        .unwrap();

    std::fs::remove_dir_all(&model_dir).unwrap();
    assert!(cache.get_model_path("path/test").is_none());
}

#[tokio::test]
async fn test_resolve_local_then_remote_order() {
    let tmp = tempfile::tempdir().unwrap();
    let model_dir = tmp.path().join("local");
    std::fs::create_dir_all(&model_dir).unwrap();
    std::fs::write(model_dir.join("model.safetensors"), b"local").unwrap();

    let mut cache = ModelCache::with_cache_dir(tmp.path().join("cache"))
        .await
        .unwrap();
    cache
        .register_model("order/test", &model_dir, "local://")
        .await
        .unwrap();

    let res = cache.resolve("order/test").await;
    assert!(matches!(res, ModelResolution::Local(_)));
}

#[tokio::test]
async fn test_register_huggingface_model() {
    let tmp = tempfile::tempdir().unwrap();
    let mut cache = ModelCache::with_cache_dir(tmp.path().join("cache"))
        .await
        .unwrap();
    let result = cache.register_huggingface_model("nonexistent/model").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_import_huggingface_cache_from_nonexistent() {
    let tmp = tempfile::tempdir().unwrap();
    let mut cache = ModelCache::with_cache_dir(tmp.path().join("cache"))
        .await
        .unwrap();
    let imported = cache
        .import_huggingface_cache_from(&tmp.path().join("nonexistent-hub"))
        .await
        .unwrap();
    assert!(imported.is_empty());
}

#[tokio::test]
async fn test_detect_format_huggingface_default() {
    let tmp = tempfile::tempdir().unwrap();
    let model_dir = tmp.path().join("hf");
    std::fs::create_dir_all(&model_dir).unwrap();
    std::fs::write(model_dir.join("config.json"), b"{}").unwrap();

    let mut cache = ModelCache::with_cache_dir(tmp.path().join("cache"))
        .await
        .unwrap();
    cache
        .register_model("hf/default", &model_dir, "test://")
        .await
        .unwrap();
    assert_eq!(cache.get_model("hf/default").unwrap().format, "huggingface");
}

#[tokio::test]
async fn test_corrupt_manifest_json_falls_back_to_empty() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(tmp.path()).unwrap();
    std::fs::write(tmp.path().join("manifest.json"), "{ not valid json").unwrap();
    let cache = ModelCache::with_cache_dir(tmp.path().to_path_buf())
        .await
        .unwrap();
    assert!(!cache.has_model("any"));
    assert!(cache.list_models().is_empty());
}

/// Covers `import_huggingface_cache_from` branch where `has_model` is true (skip register).
#[tokio::test]
async fn test_family_id_accessor() {
    let tmp = tempfile::tempdir().unwrap();
    let cache = ModelCache::with_cache_dir(tmp.path().to_path_buf())
        .await
        .unwrap();
    assert!(!cache.family_id().is_empty());
}

#[tokio::test]
async fn test_huggingface_hub_dir_uses_config_hf_home() {
    let tmp = tempfile::tempdir().unwrap();
    let mut config = ModelCacheConfig::from_env();
    config.cache_dir = tmp.path().join("cache");
    config.hf_home = Some(tmp.path().join("my_hf"));
    std::fs::create_dir_all(&config.cache_dir).unwrap();
    let cache = ModelCache::with_config(config).await.unwrap();
    let hub = cache.huggingface_hub_dir().unwrap();
    assert_eq!(hub, tmp.path().join("my_hf").join("hub"));
}

#[tokio::test]
async fn test_register_huggingface_missing_snapshots_dir() {
    let tmp = tempfile::tempdir().unwrap();
    let hf_hub = tmp.path().join("hub");
    let model_root = hf_hub.join("models--org--model");
    std::fs::create_dir_all(&model_root).unwrap();
    // no snapshots/ under model cache layout
    let mut cache = ModelCache::with_cache_dir(tmp.path().join("cache"))
        .await
        .unwrap();
    let result = cache
        .register_huggingface_model_from_hub("org/model", &hf_hub)
        .await;
    assert!(
        result.is_err(),
        "expected missing snapshots, got {result:?}"
    );
    let err = result.unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("snapshots") || msg.contains("snapshot"),
        "unexpected: {msg}"
    );
}

#[tokio::test]
async fn test_register_huggingface_empty_snapshots_folder() {
    let tmp = tempfile::tempdir().unwrap();
    let hf_hub = tmp.path().join("hub");
    let snaps = hf_hub.join("models--x--y").join("snapshots");
    std::fs::create_dir_all(&snaps).unwrap();
    let mut cache = ModelCache::with_cache_dir(tmp.path().join("cache"))
        .await
        .unwrap();
    let result = cache
        .register_huggingface_model_from_hub("x/y", &hf_hub)
        .await;
    assert!(
        result.is_err(),
        "expected empty snapshots to fail: {result:?}"
    );
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("snapshot") || msg.contains("No"));
}

#[tokio::test]
async fn test_import_huggingface_skips_models_already_registered() {
    let tmp = tempfile::tempdir().unwrap();
    let hf_hub = tmp.path().join("hub");
    let model_id = "skip/already";
    let snap = hf_hub
        .join(format!("models--{}", model_id.replace('/', "--")))
        .join("snapshots")
        .join("s1");
    std::fs::create_dir_all(&snap).unwrap();
    std::fs::write(snap.join("model.safetensors"), b"w").unwrap();

    let mut cache = ModelCache::with_cache_dir(tmp.path().join("cache"))
        .await
        .unwrap();
    cache
        .register_huggingface_model_from_hub(model_id, &hf_hub)
        .await
        .unwrap();
    let imported = cache.import_huggingface_cache_from(&hf_hub).await.unwrap();
    assert!(imported.is_empty());
}
