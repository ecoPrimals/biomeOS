// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Unit tests for [`super::ModelCache`] (local cache, HF import, mesh stubs).

#![expect(clippy::unwrap_used, reason = "test assertions")]

use super::super::ModelCache;
use super::super::types::{ModelCacheConfig, ModelResolution};

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
