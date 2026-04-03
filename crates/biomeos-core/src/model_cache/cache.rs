// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Model cache management - `NestGate` integration and filesystem fallback

use anyhow::{Context, Result};
use serde_json::json;
use std::path::{Path, PathBuf};
use tokio::fs;
use tracing::{debug, info, warn};

use crate::atomic_client::AtomicClient;

use super::types::{CacheManifest, ModelCacheConfig, ModelEntry, ModelFile, ModelResolution};

/// NUCLEUS Model Cache Manager
pub struct ModelCache {
    config: ModelCacheConfig,

    manifest_path: PathBuf,

    manifest: CacheManifest,

    nestgate: Option<AtomicClient>,
}

impl ModelCache {
    /// Create a new `ModelCache` with automatic `NestGate` discovery
    pub async fn new() -> Result<Self> {
        Self::with_config(ModelCacheConfig::from_env()).await
    }

    /// Create a `ModelCache` from explicit configuration (no implicit environment reads beyond
    /// [`AtomicClient::discover`]).
    pub async fn with_config(config: ModelCacheConfig) -> Result<Self> {
        let cache_dir = config.cache_dir.clone();
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
            config,
            manifest_path,
            manifest,
            nestgate,
        })
    }

    /// Create a `ModelCache` with a specific cache directory; other fields match [`ModelCacheConfig::from_env`].
    pub async fn with_cache_dir(cache_dir: PathBuf) -> Result<Self> {
        let mut config = ModelCacheConfig::from_env();
        config.cache_dir = cache_dir;
        Self::with_config(config).await
    }

    /// Family id used for mesh / NestGate keys.
    #[must_use]
    pub fn family_id(&self) -> &str {
        &self.config.family_id
    }

    /// Resolve the Hugging Face hub directory (`HF_HOME/hub` or `HOME/.cache/huggingface/hub`).
    pub fn huggingface_hub_dir(&self) -> Result<PathBuf> {
        if let Some(ref hf) = self.config.hf_home {
            return Ok(hf.join("hub"));
        }
        let home = std::env::var("HOME").context("HOME not set")?;
        Ok(PathBuf::from(home)
            .join(".cache")
            .join("huggingface")
            .join("hub"))
    }

    /// Check if a model is cached locally
    #[must_use]
    pub fn has_model(&self, model_id: &str) -> bool {
        if let Some(entry) = self.manifest.models.get(model_id) {
            entry.local_path.exists()
        } else {
            false
        }
    }

    /// Get the local path for a cached model
    #[must_use]
    pub fn get_model_path(&self, model_id: &str) -> Option<&Path> {
        self.manifest
            .models
            .get(model_id)
            .filter(|e| e.local_path.exists())
            .map(|e| e.local_path.as_path())
    }

    /// Get full model entry with metadata
    #[must_use]
    pub fn get_model(&self, model_id: &str) -> Option<&ModelEntry> {
        self.manifest
            .models
            .get(model_id)
            .filter(|e| e.local_path.exists())
    }

    /// List all cached models
    #[must_use]
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
            gate_id: self.config.gate_id.clone(),
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

    /// Register a model with the `HuggingFace` cache path
    pub async fn register_huggingface_model(&mut self, model_id: &str) -> Result<PathBuf> {
        let hf_hub = self.huggingface_hub_dir()?;
        self.register_huggingface_model_from_hub(model_id, &hf_hub)
            .await
    }

    /// Register a model from a specific `HuggingFace` hub directory
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

    /// Import all `HuggingFace` models from the default cache
    pub async fn import_huggingface_cache(&mut self) -> Result<Vec<String>> {
        let hf_hub = self.huggingface_hub_dir()?;
        self.import_huggingface_cache_from(&hf_hub).await
    }

    /// Import all `HuggingFace` models from a specific hub directory
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

    /// Check the mesh (`NestGate`) for a model available on another gate
    pub async fn find_on_mesh(&self, model_id: &str) -> Option<ModelEntry> {
        let client = self.nestgate.as_ref()?;
        let key = format!("model-cache:{model_id}");

        let exists = match client
            .call(
                "storage.exists",
                json!({
                    "family_id": self.config.family_id,
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
                    "family_id": self.config.family_id,
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
                    "family_id": self.config.family_id,
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
                    "family_id": self.config.family_id,
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
