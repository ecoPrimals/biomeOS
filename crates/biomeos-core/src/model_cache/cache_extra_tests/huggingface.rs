// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::ModelCache;
use super::super::types::{CacheManifest, ModelCacheConfig, ModelEntry, ModelFile, ModelResolution};
use std::path::PathBuf;
use tempfile::TempDir;

use super::hf_models_dir;

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

    let config = ModelCacheConfig {
        cache_dir: tmp.path().join("cache"),
        family_id: "default".into(),
        gate_id: "test-gate".into(),
        hf_home: Some(hf_home),
    };
    let mut cache = ModelCache::with_config(config).await.expect("with_config");
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
async fn test_register_huggingface_from_hub_selects_last_sorted_snapshot_dir() {
    let tmp = TempDir::new().unwrap();
    let hf_hub = tmp.path().join("hub");
    let model_id = "org/multi-snap";
    let base = hf_models_dir(&hf_hub, model_id).join("snapshots");
    std::fs::create_dir_all(base.join("aaa-rev")).unwrap();
    std::fs::create_dir_all(base.join("zzz-rev")).unwrap();
    std::fs::write(base.join("zzz-rev").join("model.safetensors"), b"last").unwrap();

    let mut cache = ModelCache::with_cache_dir(tmp.path().join("cache"))
        .await
        .unwrap();
    let path = cache
        .register_huggingface_model_from_hub(model_id, &hf_hub)
        .await
        .expect("register");
    assert!(path.ends_with("zzz-rev"));
    assert!(path.join("model.safetensors").exists());
}

#[tokio::test]
async fn test_import_huggingface_cache_from_imports_models_prefix_directories() {
    let tmp = TempDir::new().unwrap();
    let hf_hub = tmp.path().join("hub");
    let model_id = "import/ok-model";
    let snap = hf_models_dir(&hf_hub, model_id)
        .join("snapshots")
        .join("snapsha");
    std::fs::create_dir_all(&snap).unwrap();
    std::fs::write(snap.join("model.safetensors"), b"weights").unwrap();

    let mut cache = ModelCache::with_cache_dir(tmp.path().join("cache"))
        .await
        .unwrap();
    let imported = cache
        .import_huggingface_cache_from(&hf_hub)
        .await
        .expect("import");
    assert_eq!(imported, vec![model_id.to_string()]);
    assert!(cache.has_model(model_id));
}

#[tokio::test]
#[cfg(unix)]
async fn test_register_model_skips_broken_symlink_during_scan() {
    use std::os::unix::fs::symlink;

    let tmp = TempDir::new().unwrap();
    let model_dir = tmp.path().join("with-broken");
    std::fs::create_dir_all(&model_dir).unwrap();
    std::fs::write(model_dir.join("real.safetensors"), b"x").unwrap();
    symlink(
        tmp.path().join("nonexistent-target-xyz"),
        model_dir.join("broken.link"),
    )
    .unwrap();

    let mut cache = ModelCache::with_cache_dir(tmp.path().join("cache"))
        .await
        .unwrap();
    cache
        .register_model("broken/sym", &model_dir, "test://")
        .await
        .expect("register with broken symlink in tree");
    let entry = cache.get_model("broken/sym").expect("entry");
    assert!(entry.size_bytes >= 1);
}

#[tokio::test]
async fn test_register_huggingface_model_resolves_hub_via_home_cache_path() {
    let tmp = TempDir::new().unwrap();
    let home = tmp.path().join("h");
    std::fs::create_dir_all(&home).unwrap();
    let hf_home = home.join(".cache").join("huggingface");
    let hf_hub = hf_home.join("hub");
    let model_id = "homecache/FromHome";
    let snap = hf_models_dir(&hf_hub, model_id)
        .join("snapshots")
        .join("only");
    std::fs::create_dir_all(&snap).unwrap();
    std::fs::write(snap.join("model.safetensors"), b"y").unwrap();

    let config = ModelCacheConfig {
        cache_dir: tmp.path().join("cache"),
        family_id: "default".into(),
        gate_id: "g".into(),
        hf_home: Some(hf_home),
    };
    let mut cache = ModelCache::with_config(config).await.unwrap();
    let path = cache
        .register_huggingface_model(model_id)
        .await
        .expect("register via HOME/.cache/huggingface/hub equivalent");
    assert!(path.join("model.safetensors").exists());
}
