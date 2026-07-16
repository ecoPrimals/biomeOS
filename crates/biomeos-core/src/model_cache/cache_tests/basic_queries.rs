// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Unit tests for [`super::ModelCache`] (local cache, HF import, mesh stubs).

#![expect(clippy::unwrap_used, reason = "test assertions")]

use super::super::ModelCache;
use super::super::types::{ModelCacheConfig, ModelResolution};

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
async fn test_family_id_accessor() {
    let tmp = tempfile::tempdir().unwrap();
    let cache = ModelCache::with_cache_dir(tmp.path().to_path_buf())
        .await
        .unwrap();
    assert!(!cache.family_id().is_empty());
}
