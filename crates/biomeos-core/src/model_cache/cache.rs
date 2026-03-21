// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Model cache management - NestGate integration and filesystem fallback

use anyhow::{Context, Result};
use serde_json::json;
use std::path::{Path, PathBuf};
use tokio::fs;
use tracing::{debug, info, warn};

use crate::atomic_client::AtomicClient;

use super::types::{CacheManifest, ModelEntry, ModelFile, ModelResolution};

/// NUCLEUS Model Cache Manager
pub struct ModelCache {
    /// Base directory for cached models; used for logging and future path operations.
    _cache_dir: PathBuf,

    manifest_path: PathBuf,

    manifest: CacheManifest,

    nestgate: Option<AtomicClient>,

    family_id: String,

    gate_id: String,
}

impl ModelCache {
    /// Create a new ModelCache with automatic NestGate discovery
    pub async fn new() -> Result<Self> {
        let cache_dir = Self::default_cache_dir()?;
        Self::with_cache_dir(cache_dir).await
    }

    /// Create a ModelCache with a specific cache directory
    pub async fn with_cache_dir(cache_dir: PathBuf) -> Result<Self> {
        fs::create_dir_all(&cache_dir)
            .await
            .context("Failed to create model cache directory")?;

        let manifest_path = cache_dir.join("manifest.json");

        let manifest = if manifest_path.exists() {
            let data = fs::read_to_string(&manifest_path).await?;
            serde_json::from_str(&data).unwrap_or_else(|e| {
                warn!("Corrupt manifest, creating new: {}", e);
                CacheManifest::new()
            })
        } else {
            CacheManifest::new()
        };

        let storage_primal = biomeos_types::CapabilityTaxonomy::DataStorage
            .default_primal()
            .unwrap_or(biomeos_types::primal_names::NESTGATE);
        let nestgate = match AtomicClient::discover(storage_primal).await {
            Ok(client) => {
                info!("NestGate connected for model registry");
                Some(client)
            }
            Err(e) => {
                debug!("NestGate not available (filesystem fallback): {}", e);
                None
            }
        };

        let family_id = std::env::var("FAMILY_ID")
            .or_else(|_| std::env::var("NODE_FAMILY_ID"))
            .or_else(|_| std::env::var("BIOMEOS_FAMILY_ID"))
            .unwrap_or_else(|_| "default".to_string());

        let gate_id = std::env::var("GATE_ID")
            .or_else(|_| std::env::var("HOSTNAME"))
            .unwrap_or_else(|_| {
                std::fs::read_to_string("/etc/hostname")
                    .map_or_else(|_| "unknown".to_string(), |s| s.trim().to_string())
            });

        info!(
            "Model cache initialized: {} ({} models cached, NestGate: {})",
            cache_dir.display(),
            manifest.models.len(),
            if nestgate.is_some() {
                "connected"
            } else {
                "offline"
            }
        );

        Ok(Self {
            _cache_dir: cache_dir,
            manifest_path,
            manifest,
            nestgate,
            family_id,
            gate_id,
        })
    }

    fn default_cache_dir() -> Result<PathBuf> {
        let home = std::env::var("HOME").context("HOME not set")?;
        Ok(PathBuf::from(home).join(".biomeos").join("model-cache"))
    }

    /// Check if a model is cached locally
    pub fn has_model(&self, model_id: &str) -> bool {
        if let Some(entry) = self.manifest.models.get(model_id) {
            entry.local_path.exists()
        } else {
            false
        }
    }

    /// Get the local path for a cached model
    pub fn get_model_path(&self, model_id: &str) -> Option<&Path> {
        self.manifest
            .models
            .get(model_id)
            .filter(|e| e.local_path.exists())
            .map(|e| e.local_path.as_path())
    }

    /// Get full model entry with metadata
    pub fn get_model(&self, model_id: &str) -> Option<&ModelEntry> {
        self.manifest
            .models
            .get(model_id)
            .filter(|e| e.local_path.exists())
    }

    /// List all cached models
    pub fn list_models(&self) -> Vec<&ModelEntry> {
        self.manifest
            .models
            .values()
            .filter(|e| e.local_path.exists())
            .collect()
    }

    /// Register an existing model directory in the cache
    pub async fn register_model(
        &mut self,
        model_id: &str,
        local_path: &Path,
        source: &str,
    ) -> Result<()> {
        if !local_path.exists() {
            anyhow::bail!("Model path does not exist: {}", local_path.display());
        }

        let (size_bytes, files) = Self::scan_model_dir(local_path).await?;

        let entry = ModelEntry {
            model_id: model_id.to_string(),
            local_path: local_path.to_path_buf(),
            size_bytes,
            source: source.to_string(),
            sha256: None,
            cached_at: chrono::Utc::now().to_rfc3339(),
            gate_id: self.gate_id.clone(),
            format: Self::detect_format(local_path).await,
            files,
        };

        info!(
            "Registered model '{}' ({:.1} MB) at {}",
            model_id,
            size_bytes as f64 / 1_048_576.0,
            local_path.display()
        );

        self.manifest
            .models
            .insert(model_id.to_string(), entry.clone());
        self.save_manifest().await?;

        self.register_with_nestgate(&entry).await;

        Ok(())
    }

    /// Register a model with the HuggingFace cache path
    pub async fn register_huggingface_model(&mut self, model_id: &str) -> Result<PathBuf> {
        let hf_hub = Self::huggingface_hub_dir()?;
        self.register_huggingface_model_from_hub(model_id, &hf_hub)
            .await
    }

    /// Register a model from a specific HuggingFace hub directory
    pub async fn register_huggingface_model_from_hub(
        &mut self,
        model_id: &str,
        hf_hub: &Path,
    ) -> Result<PathBuf> {
        let hf_cache = hf_hub.join(format!("models--{}", model_id.replace('/', "--")));

        if !hf_cache.exists() {
            anyhow::bail!(
                "HuggingFace model '{}' not found in cache at {}",
                model_id,
                hf_cache.display()
            );
        }

        let snapshot_dir = Self::find_hf_snapshot(&hf_cache)?;

        self.register_model(model_id, &snapshot_dir, &format!("huggingface:{model_id}"))
            .await?;

        Ok(snapshot_dir)
    }

    /// Import all HuggingFace models from the default cache
    pub async fn import_huggingface_cache(&mut self) -> Result<Vec<String>> {
        let hf_hub = Self::huggingface_hub_dir()?;
        self.import_huggingface_cache_from(&hf_hub).await
    }

    /// Import all HuggingFace models from a specific hub directory
    pub async fn import_huggingface_cache_from(&mut self, hf_hub: &Path) -> Result<Vec<String>> {
        if !hf_hub.exists() {
            return Ok(vec![]);
        }

        let mut imported = Vec::new();

        let mut entries = fs::read_dir(hf_hub).await?;
        while let Some(entry) = entries.next_entry().await? {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("models--") {
                let model_id = name
                    .strip_prefix("models--")
                    .unwrap_or(&name)
                    .replace("--", "/");

                if !self.has_model(&model_id) {
                    match self
                        .register_huggingface_model_from_hub(&model_id, hf_hub)
                        .await
                    {
                        Ok(_) => {
                            imported.push(model_id);
                        }
                        Err(e) => {
                            debug!("Skipping {}: {}", name, e);
                        }
                    }
                }
            }
        }

        if !imported.is_empty() {
            info!("Imported {} models from HuggingFace cache", imported.len());
        }

        Ok(imported)
    }

    /// Check the mesh (NestGate) for a model available on another gate
    pub async fn find_on_mesh(&self, model_id: &str) -> Option<ModelEntry> {
        let client = self.nestgate.as_ref()?;
        let key = format!("model-cache:{model_id}");

        let exists = match client
            .call(
                "storage.exists",
                json!({
                    "family_id": self.family_id,
                    "key": key
                }),
            )
            .await
        {
            Ok(response) => response
                .get("exists")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false),
            Err(e) => {
                debug!("NestGate mesh existence check failed: {}", e);
                return None;
            }
        };

        if !exists {
            return None;
        }

        let result = client
            .call(
                "storage.retrieve",
                json!({
                    "family_id": self.family_id,
                    "key": key
                }),
            )
            .await;

        match result {
            Ok(response) => {
                if let Some(data) = response.get("data") {
                    if data.is_null() {
                        None
                    } else {
                        serde_json::from_value(data.clone()).ok()
                    }
                } else {
                    None
                }
            }
            Err(e) => {
                debug!("NestGate mesh retrieve failed: {}", e);
                None
            }
        }
    }

    /// List all models known across the mesh
    pub async fn list_mesh_models(&self) -> Vec<ModelEntry> {
        let Some(client) = self.nestgate.as_ref() else {
            return vec![];
        };

        let result = client
            .call(
                "storage.list",
                json!({
                    "family_id": self.family_id,
                    "prefix": "model-cache:"
                }),
            )
            .await;

        match result {
            Ok(response) => {
                let keys = response
                    .get("keys")
                    .and_then(|k| k.as_array())
                    .cloned()
                    .unwrap_or_default();

                let mut models = Vec::new();
                for key in keys {
                    if let Some(key_str) = key.as_str() {
                        let model_id = key_str.strip_prefix("model-cache:").unwrap_or(key_str);
                        if let Some(entry) = self.find_on_mesh(model_id).await {
                            models.push(entry);
                        }
                    }
                }
                models
            }
            Err(e) => {
                debug!("NestGate mesh list failed: {}", e);
                vec![]
            }
        }
    }

    /// Resolve a model: check local cache, then mesh
    pub async fn resolve(&self, model_id: &str) -> ModelResolution {
        if let Some(entry) = self.get_model(model_id) {
            return ModelResolution::Local(entry.clone());
        }

        if let Some(entry) = self.find_on_mesh(model_id).await {
            return ModelResolution::Remote(entry);
        }

        ModelResolution::NotFound
    }

    async fn save_manifest(&self) -> Result<()> {
        let data = serde_json::to_string_pretty(&self.manifest)?;
        fs::write(&self.manifest_path, data).await?;
        Ok(())
    }

    async fn register_with_nestgate(&self, entry: &ModelEntry) {
        let Some(client) = self.nestgate.as_ref() else {
            return;
        };

        let result = client
            .call(
                "storage.store",
                json!({
                    "family_id": self.family_id,
                    "key": format!("model-cache:{}", entry.model_id),
                    "value": serde_json::to_value(entry).unwrap_or_default()
                }),
            )
            .await;

        match result {
            Ok(_) => {
                info!(
                    "Registered '{}' with NestGate mesh registry",
                    entry.model_id
                );
            }
            Err(e) => {
                warn!(
                    "NestGate registration failed (model still cached locally): {}",
                    e
                );
            }
        }
    }

    async fn scan_model_dir(dir: &Path) -> Result<(u64, Vec<ModelFile>)> {
        let mut total_size = 0u64;
        let mut files = Vec::new();

        let mut stack = vec![dir.to_path_buf()];
        while let Some(current) = stack.pop() {
            let mut entries = fs::read_dir(&current).await?;
            while let Some(entry) = entries.next_entry().await? {
                let real_path = fs::canonicalize(entry.path())
                    .await
                    .unwrap_or_else(|_| entry.path());
                let metadata = match fs::metadata(&real_path).await {
                    Ok(m) => m,
                    Err(_) => match std::fs::symlink_metadata(entry.path()) {
                        Ok(m) => m,
                        Err(_) => continue,
                    },
                };

                if metadata.is_dir() {
                    stack.push(entry.path());
                } else {
                    let size = metadata.len();
                    total_size += size;

                    let relative = entry
                        .path()
                        .strip_prefix(dir)
                        .unwrap_or(&entry.path())
                        .to_string_lossy()
                        .to_string();

                    files.push(ModelFile {
                        relative_path: relative,
                        size_bytes: size,
                        sha256: None,
                    });
                }
            }
        }

        Ok((total_size, files))
    }

    async fn detect_format(dir: &Path) -> String {
        if let Ok(mut entries) = fs::read_dir(dir).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.ends_with(".safetensors") {
                    return "safetensors".to_string();
                }
                if std::path::Path::new(&name)
                    .extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("gguf"))
                {
                    return "gguf".to_string();
                }
                if std::path::Path::new(&name)
                    .extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("bin"))
                    && name.contains("pytorch")
                {
                    return "pytorch".to_string();
                }
            }
        }
        "huggingface".to_string()
    }

    fn huggingface_hub_dir() -> Result<PathBuf> {
        if let Ok(cache) = std::env::var("HF_HOME") {
            return Ok(PathBuf::from(cache).join("hub"));
        }
        let home = std::env::var("HOME").context("HOME not set")?;
        Ok(PathBuf::from(home)
            .join(".cache")
            .join("huggingface")
            .join("hub"))
    }

    fn find_hf_snapshot(model_dir: &Path) -> Result<PathBuf> {
        let snapshots_dir = model_dir.join("snapshots");
        if !snapshots_dir.exists() {
            anyhow::bail!("No snapshots directory in {}", model_dir.display());
        }

        let mut entries: Vec<_> = std::fs::read_dir(&snapshots_dir)?
            .filter_map(std::result::Result::ok)
            .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
            .collect();

        entries.sort_by_key(std::fs::DirEntry::file_name);

        entries
            .last()
            .map(std::fs::DirEntry::path)
            .ok_or_else(|| anyhow::anyhow!("No snapshot found in {}", snapshots_dir.display()))
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

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
}
