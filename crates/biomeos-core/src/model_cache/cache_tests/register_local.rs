// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Unit tests for [`super::ModelCache`] (local cache, HF import, mesh stubs).

#![expect(clippy::unwrap_used, reason = "test assertions")]

use super::super::ModelCache;
use super::super::types::ModelResolution;

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
