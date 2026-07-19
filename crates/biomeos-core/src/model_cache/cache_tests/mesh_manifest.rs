// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Unit tests for [`super::ModelCache`] (local cache, HF import, mesh stubs).

#![expect(clippy::unwrap_used, reason = "test assertions")]

use super::super::ModelCache;
use super::super::types::{ModelCacheConfig, ModelResolution};

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
    let iso = tempfile::tempdir().unwrap();
    let iso_path = iso.path().to_str().unwrap();
    temp_env::async_with_vars(
        [
            ("BIOMEOS_SOCKET_DIR", Some(iso_path)),
            ("XDG_RUNTIME_DIR", Some(iso_path)),
        ],
        async {
            let cache = ModelCache::with_cache_dir(tmp.path().to_path_buf())
                .await
                .unwrap();
            let models = cache.list_mesh_models().await;
            assert!(models.is_empty());
        },
    )
    .await;
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
