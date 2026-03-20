// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Extra integration tests for [`super::cache::ModelCache`] (HF paths, errors, manifest).

#![expect(
    clippy::unwrap_used,
    reason = "test setup uses tempfile and infallible fixtures"
)]
#![expect(clippy::expect_used, reason = "test assertions use expect for clarity")]

use super::ModelCache;
use super::types::{CacheManifest, ModelEntry, ModelFile, ModelResolution};
use biomeos_test_utils::TestEnvGuard;
use std::path::PathBuf;
use tempfile::TempDir;

fn hf_models_dir(hf_hub: &std::path::Path, model_id: &str) -> PathBuf {
    hf_hub.join(format!("models--{}", model_id.replace('/', "--")))
}

#[tokio::test]
async fn test_register_huggingface_from_hub_missing_snapshots_dir() {
    let tmp = TempDir::new().unwrap();
    let hf_hub = tmp.path().join("hub");
    let model_dir = hf_models_dir(&hf_hub, "org/missing-snapshots");
    std::fs::create_dir_all(&model_dir).unwrap();
    // no `snapshots/` subdirectory

    let mut cache = ModelCache::with_cache_dir(tmp.path().join("cache"))
        .await
        .unwrap();
    let err = cache
        .register_huggingface_model_from_hub("org/missing-snapshots", &hf_hub)
        .await
        .expect_err("expected snapshots error");
    assert!(err.to_string().contains("snapshots"));
}

#[tokio::test]
async fn test_register_huggingface_from_hub_empty_snapshots_dir() {
    let tmp = TempDir::new().unwrap();
    let hf_hub = tmp.path().join("hub");
    let model_dir = hf_models_dir(&hf_hub, "org/empty-snap");
    std::fs::create_dir_all(model_dir.join("snapshots")).unwrap();

    let mut cache = ModelCache::with_cache_dir(tmp.path().join("cache"))
        .await
        .unwrap();
    let err = cache
        .register_huggingface_model_from_hub("org/empty-snap", &hf_hub)
        .await
        .expect_err("expected no snapshot");
    assert!(err.to_string().contains("snapshot"));
}

#[tokio::test]
#[serial_test::serial]
async fn test_register_huggingface_model_uses_hf_home_hub_path() {
    let tmp = TempDir::new().unwrap();
    let hf_home = tmp.path().join("hf_home");
    let hf_hub = hf_home.join("hub");
    let model_id = "demo/Demo-Model";
    let snapshot = hf_models_dir(&hf_hub, model_id)
        .join("snapshots")
        .join("abc123");
    std::fs::create_dir_all(&snapshot).unwrap();
    std::fs::write(snapshot.join("model.safetensors"), b"x").unwrap();

    let _hf_guard = TestEnvGuard::set(
        "HF_HOME",
        hf_home.to_str().expect("HF_HOME temp path must be UTF-8"),
    );
    let mut cache = ModelCache::with_cache_dir(tmp.path().join("cache"))
        .await
        .unwrap();
    let path = cache
        .register_huggingface_model(model_id)
        .await
        .expect("register");
    assert_eq!(path, snapshot);
}

#[tokio::test]
async fn test_import_huggingface_skips_entries_that_fail_to_register() {
    let tmp = TempDir::new().unwrap();
    let hf_hub = tmp.path().join("hub");
    let bad_name = "broken/model";
    let bad_dir = hf_models_dir(&hf_hub, bad_name);
    std::fs::create_dir_all(bad_dir.join("snapshots")).unwrap();

    let mut cache = ModelCache::with_cache_dir(tmp.path().join("cache"))
        .await
        .unwrap();
    let imported = cache.import_huggingface_cache_from(&hf_hub).await.unwrap();
    assert!(imported.is_empty());
}

#[tokio::test]
async fn test_register_model_symlinked_file_in_tree() {
    let tmp = TempDir::new().unwrap();
    let target = tmp.path().join("real.bin");
    std::fs::write(&target, b"blob").unwrap();

    let model_dir = tmp.path().join("model-with-link");
    std::fs::create_dir_all(&model_dir).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        symlink(&target, model_dir.join("weights.bin")).unwrap();
    }
    #[cfg(not(unix))]
    {
        std::fs::copy(&target, model_dir.join("weights.bin")).unwrap();
    }

    let mut cache = ModelCache::with_cache_dir(tmp.path().join("cache"))
        .await
        .unwrap();
    cache
        .register_model("link/test", &model_dir, "test://")
        .await
        .unwrap();
    let entry = cache.get_model("link/test").expect("entry");
    assert!(entry.size_bytes >= 4);
}

#[tokio::test]
async fn test_cache_manifest_roundtrip_serialization() {
    let mut m = CacheManifest::new();
    m.models.insert(
        "k".to_string(),
        ModelEntry {
            model_id: "k".to_string(),
            local_path: PathBuf::from("/tmp/k"),
            size_bytes: 1,
            source: "s".to_string(),
            sha256: None,
            cached_at: "t".to_string(),
            gate_id: "g".to_string(),
            format: "huggingface".to_string(),
            files: vec![ModelFile {
                relative_path: "f".to_string(),
                size_bytes: 1,
                sha256: None,
            }],
        },
    );
    let v = serde_json::to_value(&m).expect("json");
    let back: CacheManifest = serde_json::from_value(v).expect("back");
    assert_eq!(back.version, 1);
    assert_eq!(back.models.len(), 1);
}

#[tokio::test]
async fn test_resolve_not_found_after_manifest_stale_path() {
    let tmp = TempDir::new().unwrap();
    let model_dir = tmp.path().join("gone");
    std::fs::create_dir_all(&model_dir).unwrap();
    std::fs::write(model_dir.join("x.safetensors"), b"x").unwrap();

    let cache_dir = tmp.path().join("cache");
    {
        let mut cache = ModelCache::with_cache_dir(cache_dir.clone()).await.unwrap();
        cache
            .register_model("stale/m", &model_dir, "test://")
            .await
            .unwrap();
    }
    std::fs::remove_dir_all(&model_dir).unwrap();

    let cache = ModelCache::with_cache_dir(cache_dir).await.unwrap();
    let res = cache.resolve("stale/m").await;
    assert!(matches!(res, ModelResolution::NotFound));
}
