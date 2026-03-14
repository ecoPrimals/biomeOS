// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Model cache tests

use std::path::PathBuf;

use super::cache::ModelCache;
use super::types::{ModelEntry, ModelFile, ModelResolution};

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod run {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_model_cache_creation() {
        let tmp = TempDir::new().unwrap();
        let cache = ModelCache::with_cache_dir(tmp.path().to_path_buf())
            .await
            .unwrap();

        assert!(cache.list_models().is_empty());
        assert!(!cache.has_model("nonexistent/model"));
        assert!(cache.get_model_path("nonexistent/model").is_none());
    }

    #[tokio::test]
    async fn test_register_model() {
        let tmp = TempDir::new().unwrap();
        let model_dir = tmp.path().join("test-model");
        std::fs::create_dir_all(&model_dir).unwrap();
        std::fs::write(model_dir.join("model.safetensors"), b"fake model data").unwrap();

        let cache_dir = tmp.path().join("cache");
        let mut cache = ModelCache::with_cache_dir(cache_dir).await.unwrap();

        cache
            .register_model("test/model", &model_dir, "test://source")
            .await
            .unwrap();

        assert!(cache.has_model("test/model"));
        assert_eq!(
            cache.get_model_path("test/model").unwrap(),
            model_dir.as_path()
        );

        let entry = cache.get_model("test/model").unwrap();
        assert_eq!(entry.format, "safetensors");
        assert!(entry.size_bytes > 0);
    }

    #[tokio::test]
    async fn test_manifest_persistence() {
        let tmp = TempDir::new().unwrap();
        let model_dir = tmp.path().join("persist-model");
        std::fs::create_dir_all(&model_dir).unwrap();
        std::fs::write(model_dir.join("weights.bin"), b"test").unwrap();

        let cache_dir = tmp.path().join("cache");

        {
            let mut cache = ModelCache::with_cache_dir(cache_dir.clone()).await.unwrap();
            cache
                .register_model("persist/test", &model_dir, "test://")
                .await
                .unwrap();
        }

        {
            let cache = ModelCache::with_cache_dir(cache_dir).await.unwrap();
            assert!(cache.has_model("persist/test"));
        }
    }

    #[tokio::test]
    async fn test_resolution_not_found() {
        let tmp = TempDir::new().unwrap();
        let cache = ModelCache::with_cache_dir(tmp.path().to_path_buf())
            .await
            .unwrap();

        let resolution = cache.resolve("nonexistent/model").await;
        assert!(matches!(resolution, ModelResolution::NotFound));
    }

    #[tokio::test]
    async fn test_register_model_nonexistent_path() {
        let tmp = TempDir::new().unwrap();
        let cache_dir = tmp.path().join("cache");
        let mut cache = ModelCache::with_cache_dir(cache_dir).await.unwrap();

        let result = cache
            .register_model(
                "test/model",
                tmp.path().join("nonexistent").as_path(),
                "test://",
            )
            .await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("does not exist"));
    }

    #[tokio::test]
    async fn test_resolution_local() {
        let tmp = TempDir::new().unwrap();
        let model_dir = tmp.path().join("local-model");
        std::fs::create_dir_all(&model_dir).unwrap();
        std::fs::write(model_dir.join("model.gguf"), b"gguf data").unwrap();

        let cache_dir = tmp.path().join("cache");
        let mut cache = ModelCache::with_cache_dir(cache_dir).await.unwrap();
        cache
            .register_model("local/test", &model_dir, "test://source")
            .await
            .unwrap();

        let resolution = cache.resolve("local/test").await;
        match &resolution {
            ModelResolution::Local(entry) => {
                assert_eq!(entry.model_id, "local/test");
                assert_eq!(entry.format, "gguf");
            }
            _ => panic!("Expected ModelResolution::Local, got {resolution:?}"),
        }
    }

    #[tokio::test]
    async fn test_model_resolution_display() {
        let tmp = TempDir::new().unwrap();
        let cache = ModelCache::with_cache_dir(tmp.path().to_path_buf())
            .await
            .unwrap();
        let resolution = cache.resolve("nonexistent/model").await;
        let s = format!("{resolution}");
        assert_eq!(s, "NOT FOUND");
    }

    #[tokio::test]
    async fn test_model_entry_serialization() {
        let entry = ModelEntry {
            model_id: "test/model".to_string(),
            local_path: PathBuf::from("/tmp/models/test"),
            size_bytes: 1024,
            source: "huggingface:test/model".to_string(),
            sha256: Some("abc123".to_string()),
            cached_at: chrono::Utc::now().to_rfc3339(),
            gate_id: "gate-1".to_string(),
            format: "safetensors".to_string(),
            files: vec![ModelFile {
                relative_path: "model.safetensors".to_string(),
                size_bytes: 1024,
                sha256: None,
            }],
        };
        let json = serde_json::to_value(&entry).expect("serialize");
        let restored: ModelEntry = serde_json::from_value(json).expect("deserialize");
        assert_eq!(entry.model_id, restored.model_id);
        assert_eq!(entry.format, restored.format);
    }

    #[tokio::test]
    async fn test_has_model_returns_false_when_path_deleted() {
        let tmp = TempDir::new().unwrap();
        let model_dir = tmp.path().join("ephemeral-model");
        std::fs::create_dir_all(&model_dir).unwrap();
        std::fs::write(model_dir.join("weights.bin"), b"data").unwrap();

        let cache_dir = tmp.path().join("cache");
        let mut cache = ModelCache::with_cache_dir(cache_dir).await.unwrap();
        cache
            .register_model("ephemeral/model", &model_dir, "test://")
            .await
            .unwrap();

        assert!(cache.has_model("ephemeral/model"));

        std::fs::remove_dir_all(&model_dir).unwrap();

        assert!(!cache.has_model("ephemeral/model"));
        assert!(cache.get_model_path("ephemeral/model").is_none());
    }

    #[tokio::test]
    async fn test_detect_format_pytorch() {
        let tmp = TempDir::new().unwrap();
        let model_dir = tmp.path().join("pytorch-model");
        std::fs::create_dir_all(&model_dir).unwrap();
        std::fs::write(model_dir.join("pytorch_model.bin"), b"pytorch").unwrap();

        let cache_dir = tmp.path().join("cache");
        let mut cache = ModelCache::with_cache_dir(cache_dir).await.unwrap();
        cache
            .register_model("pytorch/test", &model_dir, "test://")
            .await
            .unwrap();

        let entry = cache.get_model("pytorch/test").unwrap();
        assert_eq!(entry.format, "pytorch");
    }

    #[tokio::test]
    async fn test_detect_format_default_huggingface() {
        let tmp = TempDir::new().unwrap();
        let model_dir = tmp.path().join("hf-model");
        std::fs::create_dir_all(&model_dir).unwrap();
        std::fs::write(model_dir.join("config.json"), b"{}").unwrap();

        let cache_dir = tmp.path().join("cache");
        let mut cache = ModelCache::with_cache_dir(cache_dir).await.unwrap();
        cache
            .register_model("hf/test", &model_dir, "test://")
            .await
            .unwrap();

        let entry = cache.get_model("hf/test").unwrap();
        assert_eq!(entry.format, "huggingface");
    }

    #[tokio::test]
    async fn test_model_resolution_local_display() {
        let tmp = TempDir::new().unwrap();
        let model_dir = tmp.path().join("display-model");
        std::fs::create_dir_all(&model_dir).unwrap();
        std::fs::write(model_dir.join("model.gguf"), b"x").unwrap();

        let cache_dir = tmp.path().join("cache");
        let mut cache = ModelCache::with_cache_dir(cache_dir).await.unwrap();
        cache
            .register_model("display/test", &model_dir, "test://")
            .await
            .unwrap();

        let resolution = cache.resolve("display/test").await;
        let s = format!("{resolution}");
        assert!(s.starts_with("LOCAL:"));
        assert!(s.contains("display/test"));
    }

    #[tokio::test]
    async fn test_model_resolution_remote_display() {
        let entry = ModelEntry {
            model_id: "remote/model".to_string(),
            local_path: PathBuf::from("/tmp/remote"),
            size_bytes: 1024 * 1024,
            source: "mesh".to_string(),
            sha256: None,
            cached_at: chrono::Utc::now().to_rfc3339(),
            gate_id: "gate-1".to_string(),
            format: "gguf".to_string(),
            files: vec![],
        };
        let resolution = ModelResolution::Remote(entry);
        let s = format!("{resolution}");
        assert!(s.starts_with("REMOTE:"));
        assert!(s.contains("remote/model"));
        assert!(s.contains("gate-1"));
    }

    #[tokio::test]
    async fn test_corrupt_manifest_recovery() {
        let tmp = TempDir::new().unwrap();
        let cache_dir = tmp.path().join("cache");
        std::fs::create_dir_all(&cache_dir).unwrap();
        std::fs::write(cache_dir.join("manifest.json"), "invalid json {{{").unwrap();

        let cache = ModelCache::with_cache_dir(cache_dir).await.unwrap();
        assert!(cache.list_models().is_empty());
    }

    #[tokio::test]
    async fn test_list_models_filters_missing_paths() {
        let tmp = TempDir::new().unwrap();
        let cache_dir = tmp.path().join("cache");
        let mut cache = ModelCache::with_cache_dir(cache_dir).await.unwrap();

        let model_dir = tmp.path().join("list-model");
        std::fs::create_dir_all(&model_dir).unwrap();
        std::fs::write(model_dir.join("model.safetensors"), b"data").unwrap();
        cache
            .register_model("list/test", &model_dir, "test://")
            .await
            .unwrap();

        let models = cache.list_models();
        assert_eq!(models.len(), 1);

        std::fs::remove_dir_all(&model_dir).unwrap();
        let models_after = cache.list_models();
        assert!(models_after.is_empty());
    }

    #[tokio::test]
    async fn test_model_file_serialization() {
        let file = ModelFile {
            relative_path: "model.safetensors".to_string(),
            size_bytes: 1024,
            sha256: Some("abc123".to_string()),
        };
        let json = serde_json::to_value(&file).unwrap();
        let restored: ModelFile = serde_json::from_value(json).unwrap();
        assert_eq!(file.relative_path, restored.relative_path);
        assert_eq!(file.sha256, restored.sha256);
    }

    #[tokio::test]
    async fn test_cache_manifest_default() {
        let manifest = crate::model_cache::types::CacheManifest::default();
        assert_eq!(manifest.version, 0);
        assert!(manifest.models.is_empty());
    }
}
