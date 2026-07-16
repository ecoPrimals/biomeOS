// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::ModelCache;
use super::super::types::{CacheManifest, ModelCacheConfig, ModelEntry, ModelFile, ModelResolution};
use std::path::PathBuf;
use tempfile::TempDir;

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
    let iso = TempDir::new().unwrap();
    let iso_path = iso.path().to_str().unwrap();
    temp_env::async_with_vars(
        [
            ("BIOMEOS_SOCKET_DIR", Some(iso_path)),
            ("XDG_RUNTIME_DIR", Some(iso_path)),
        ],
        async {
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
        },
    )
    .await;
}
#[tokio::test]
async fn test_model_cache_new_creates_default_cache_under_home() {
    let tmp = TempDir::new().unwrap();
    let home = tmp.path();
    let cache_dir = home.join(".cache").join("biomeos").join("models");
    let config = ModelCacheConfig {
        cache_dir: cache_dir.clone(),
        family_id: "default".into(),
        gate_id: "test-gate".into(),
        hf_home: None,
    };

    ModelCache::with_config(config).await.expect("with_config");

    assert!(
        cache_dir.is_dir(),
        "expected default cache dir at {{HOME}}/.cache/biomeos/models, got none"
    );
}

#[tokio::test]
async fn test_model_cache_new_succeeds_even_without_home() {
    let tmp = TempDir::new().unwrap();
    let config = ModelCacheConfig {
        cache_dir: tmp.path().join("models"),
        family_id: "default".into(),
        gate_id: "gate-1".into(),
        hf_home: None,
    };
    let result = ModelCache::with_config(config).await;
    assert!(
        result.is_ok(),
        "ModelCache::with_config should succeed with an explicit cache dir"
    );
}

#[tokio::test]
async fn test_model_cache_new_ignores_hf_home_for_default_cache_location() {
    let tmp = TempDir::new().unwrap();
    let home = tmp.path().join("home-a");
    std::fs::create_dir_all(&home).unwrap();
    let hf_home = tmp.path().join("hf-alt");
    std::fs::create_dir_all(&hf_home).unwrap();

    let cache_dir = home.join(".cache").join("biomeos").join("models");
    let config = ModelCacheConfig {
        cache_dir: cache_dir.clone(),
        family_id: "default".into(),
        gate_id: "g".into(),
        hf_home: Some(hf_home.clone()),
    };

    ModelCache::with_config(config).await.expect("with_config");

    assert!(cache_dir.is_dir());
    assert!(
        !hf_home.join("biomeos").exists(),
        "HF_HOME must not relocate the model-cache root used by ModelCache::with_config"
    );
}
