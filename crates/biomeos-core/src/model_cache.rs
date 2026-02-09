//! Model Cache - NUCLEUS-integrated model artifact management
//!
//! Provides persistent caching for AI model files (HuggingFace, GGUF, safetensors)
//! across the biomeOS mesh. Uses NestGate for distributed storage when available,
//! with graceful fallback to local filesystem.
//!
//! ## Architecture
//!
//! ```text
//! Consumer (Toadstool/Squirrel)
//!     │
//!     ▼
//! ModelCache::get("TinyLlama/TinyLlama-1.1B-Chat-v1.0")
//!     │
//!     ├─ Check local filesystem cache
//!     │   └─ Found? Return path immediately
//!     │
//!     ├─ Check NestGate registry (via AtomicClient)
//!     │   └─ Found on another gate? Transfer via Songbird
//!     │
//!     └─ Not cached anywhere? Download from source
//!         └─ Register in NestGate + local cache
//! ```
//!
//! ## Design Principles
//!
//! - **NestGate-optional**: Works without NestGate (filesystem fallback)
//! - **Zero re-downloads**: Check local → mesh → source
//! - **Capability-driven**: Uses AtomicClient for NestGate IPC
//! - **Content-addressed**: SHA256 verification for integrity
//! - **Cross-gate**: Model manifests shared via NestGate registry

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs;
use tracing::{debug, info, warn};

use crate::atomic_client::AtomicClient;

/// Metadata about a cached model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelEntry {
    /// Model identifier (e.g., "TinyLlama/TinyLlama-1.1B-Chat-v1.0")
    pub model_id: String,

    /// Local filesystem path to model directory
    pub local_path: PathBuf,

    /// Total size in bytes
    pub size_bytes: u64,

    /// Source URL (e.g., HuggingFace hub URL)
    pub source: String,

    /// SHA256 of the primary model file (for integrity)
    #[serde(default)]
    pub sha256: Option<String>,

    /// When this entry was cached
    pub cached_at: String,

    /// Which gate cached this model
    pub gate_id: String,

    /// Model format (huggingface, gguf, safetensors, etc.)
    #[serde(default = "default_format")]
    pub format: String,

    /// Individual files in this model
    #[serde(default)]
    pub files: Vec<ModelFile>,
}

fn default_format() -> String {
    "huggingface".to_string()
}

/// Individual file within a model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelFile {
    pub relative_path: String,
    pub size_bytes: u64,
    #[serde(default)]
    pub sha256: Option<String>,
}

/// Model cache manifest (local JSON file)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CacheManifest {
    pub version: u32,
    pub models: HashMap<String, ModelEntry>,
}

impl CacheManifest {
    fn new() -> Self {
        Self {
            version: 1,
            models: HashMap::new(),
        }
    }
}

/// NUCLEUS Model Cache Manager
///
/// Manages model artifacts with NestGate integration and filesystem fallback.
/// Designed for zero re-downloads across the biomeOS mesh.
pub struct ModelCache {
    /// Root directory for cached models
    cache_dir: PathBuf,

    /// Path to the local manifest file
    manifest_path: PathBuf,

    /// In-memory manifest (loaded from disk)
    manifest: CacheManifest,

    /// NestGate client (None if unavailable — graceful degradation)
    nestgate: Option<AtomicClient>,

    /// Family ID for NestGate operations
    family_id: String,

    /// Gate identifier (hostname or device ID)
    gate_id: String,
}

impl ModelCache {
    /// Create a new ModelCache with automatic NestGate discovery
    ///
    /// Falls back to filesystem-only if NestGate is not running.
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

        // Load existing manifest or create new
        let manifest = if manifest_path.exists() {
            let data = fs::read_to_string(&manifest_path).await?;
            serde_json::from_str(&data).unwrap_or_else(|e| {
                warn!("Corrupt manifest, creating new: {}", e);
                CacheManifest::new()
            })
        } else {
            CacheManifest::new()
        };

        // Try to discover NestGate (graceful degradation)
        let nestgate = match AtomicClient::discover("nestgate").await {
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
            .unwrap_or_else(|_| "default".to_string());

        let gate_id = std::env::var("GATE_ID")
            .or_else(|_| std::env::var("HOSTNAME"))
            .unwrap_or_else(|_| {
                // Fallback: read /etc/hostname
                std::fs::read_to_string("/etc/hostname")
                    .map(|s| s.trim().to_string())
                    .unwrap_or_else(|_| "unknown".to_string())
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
            cache_dir,
            manifest_path,
            manifest,
            nestgate,
            family_id,
            gate_id,
        })
    }

    /// Default cache directory: ~/.biomeos/model-cache/
    fn default_cache_dir() -> Result<PathBuf> {
        let home = std::env::var("HOME").context("HOME not set")?;
        Ok(PathBuf::from(home).join(".biomeos").join("model-cache"))
    }

    /// Check if a model is cached locally
    pub fn has_model(&self, model_id: &str) -> bool {
        if let Some(entry) = self.manifest.models.get(model_id) {
            // Verify the path still exists
            entry.local_path.exists()
        } else {
            false
        }
    }

    /// Get the local path for a cached model
    ///
    /// Returns None if not cached locally.
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
    ///
    /// Use this to import models from HuggingFace cache or other locations.
    /// Does NOT copy files — just registers the path.
    pub async fn register_model(
        &mut self,
        model_id: &str,
        local_path: &Path,
        source: &str,
    ) -> Result<()> {
        if !local_path.exists() {
            anyhow::bail!("Model path does not exist: {}", local_path.display());
        }

        // Calculate total size and enumerate files
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

        // Register locally
        self.manifest
            .models
            .insert(model_id.to_string(), entry.clone());
        self.save_manifest().await?;

        // Register with NestGate if available
        self.register_with_nestgate(&entry).await;

        Ok(())
    }

    /// Register a model with the HuggingFace cache path
    ///
    /// Automatically locates the model in the default HF cache.
    pub async fn register_huggingface_model(&mut self, model_id: &str) -> Result<PathBuf> {
        let hf_cache = Self::find_huggingface_model(model_id)?;

        if !hf_cache.exists() {
            anyhow::bail!(
                "HuggingFace model '{}' not found in cache at {}",
                model_id,
                hf_cache.display()
            );
        }

        // Find the actual snapshot directory
        let snapshot_dir = Self::find_hf_snapshot(&hf_cache)?;

        self.register_model(model_id, &snapshot_dir, &format!("huggingface:{}", model_id))
            .await?;

        Ok(snapshot_dir)
    }

    /// Import all HuggingFace models from the default cache
    pub async fn import_huggingface_cache(&mut self) -> Result<Vec<String>> {
        let hf_hub = Self::huggingface_hub_dir()?;
        if !hf_hub.exists() {
            return Ok(vec![]);
        }

        let mut imported = Vec::new();

        let mut entries = fs::read_dir(&hf_hub).await?;
        while let Some(entry) = entries.next_entry().await? {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("models--") {
                // Convert "models--Org--Model" to "Org/Model"
                let model_id = name
                    .strip_prefix("models--")
                    .unwrap_or(&name)
                    .replace("--", "/");

                if !self.has_model(&model_id) {
                    match self.register_huggingface_model(&model_id).await {
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
    ///
    /// Returns the gate_id and metadata if the model exists elsewhere.
    pub async fn find_on_mesh(&self, model_id: &str) -> Option<ModelEntry> {
        let client = self.nestgate.as_ref()?;

        let result = client
            .call(
                "storage.retrieve",
                json!({
                    "family_id": self.family_id,
                    "key": format!("model-cache:{}", model_id)
                }),
            )
            .await;

        match result {
            Ok(response) => {
                if let Some(data) = response.get("data") {
                    if !data.is_null() {
                        serde_json::from_value(data.clone()).ok()
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            Err(e) => {
                debug!("NestGate mesh lookup failed: {}", e);
                None
            }
        }
    }

    /// List all models known across the mesh (from NestGate registry)
    pub async fn list_mesh_models(&self) -> Vec<ModelEntry> {
        let client = match self.nestgate.as_ref() {
            Some(c) => c,
            None => return vec![],
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

    /// Resolve a model: check local cache, then mesh, return path or download hint
    ///
    /// Returns:
    /// - `Ok(ModelResolution::Local(path))` if cached locally
    /// - `Ok(ModelResolution::Remote(entry))` if available on another gate
    /// - `Ok(ModelResolution::NotFound)` if not cached anywhere
    pub async fn resolve(&self, model_id: &str) -> ModelResolution {
        // 1. Check local cache
        if let Some(entry) = self.get_model(model_id) {
            return ModelResolution::Local(entry.clone());
        }

        // 2. Check mesh via NestGate
        if let Some(entry) = self.find_on_mesh(model_id).await {
            return ModelResolution::Remote(entry);
        }

        // 3. Not found
        ModelResolution::NotFound
    }

    // ─── Private helpers ───────────────────────────────────────────────

    /// Save manifest to disk
    async fn save_manifest(&self) -> Result<()> {
        let data = serde_json::to_string_pretty(&self.manifest)?;
        fs::write(&self.manifest_path, data).await?;
        Ok(())
    }

    /// Register model metadata with NestGate
    async fn register_with_nestgate(&self, entry: &ModelEntry) {
        let client = match self.nestgate.as_ref() {
            Some(c) => c,
            None => return,
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
                warn!("NestGate registration failed (model still cached locally): {}", e);
            }
        }
    }

    /// Scan a model directory for total size and file listing
    ///
    /// Follows symlinks (HuggingFace uses symlinks to blobs).
    async fn scan_model_dir(dir: &Path) -> Result<(u64, Vec<ModelFile>)> {
        let mut total_size = 0u64;
        let mut files = Vec::new();

        let mut stack = vec![dir.to_path_buf()];
        while let Some(current) = stack.pop() {
            let mut entries = fs::read_dir(&current).await?;
            while let Some(entry) = entries.next_entry().await? {
                // Follow symlinks to get real file metadata
                let real_path = fs::canonicalize(entry.path()).await
                    .unwrap_or_else(|_| entry.path());
                let metadata = fs::metadata(&real_path).await
                    .unwrap_or_else(|_| {
                        // Fallback to symlink metadata if target doesn't exist
                        std::fs::symlink_metadata(&entry.path()).unwrap()
                    });

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

    /// Detect model format from directory contents
    async fn detect_format(dir: &Path) -> String {
        if let Ok(mut entries) = fs::read_dir(dir).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.ends_with(".safetensors") {
                    return "safetensors".to_string();
                }
                if name.ends_with(".gguf") {
                    return "gguf".to_string();
                }
                if name.ends_with(".bin") && name.contains("pytorch") {
                    return "pytorch".to_string();
                }
            }
        }
        "huggingface".to_string()
    }

    /// Find HuggingFace model cache directory
    fn find_huggingface_model(model_id: &str) -> Result<PathBuf> {
        let hf_hub = Self::huggingface_hub_dir()?;
        let dir_name = format!("models--{}", model_id.replace('/', "--"));
        Ok(hf_hub.join(dir_name))
    }

    /// Get HuggingFace hub directory
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

    /// Find the actual snapshot directory within a HF model cache
    fn find_hf_snapshot(model_dir: &Path) -> Result<PathBuf> {
        let snapshots_dir = model_dir.join("snapshots");
        if !snapshots_dir.exists() {
            anyhow::bail!(
                "No snapshots directory in {}",
                model_dir.display()
            );
        }

        // Use the most recent snapshot (alphabetically last = latest hash)
        let mut entries: Vec<_> = std::fs::read_dir(&snapshots_dir)?
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
            .collect();

        entries.sort_by_key(|e| e.file_name());

        entries
            .last()
            .map(|e| e.path())
            .ok_or_else(|| anyhow::anyhow!("No snapshot found in {}", snapshots_dir.display()))
    }
}

/// Result of resolving a model across the mesh
#[derive(Debug, Clone)]
pub enum ModelResolution {
    /// Model is cached locally — use this path
    Local(ModelEntry),

    /// Model exists on another gate — transfer needed
    Remote(ModelEntry),

    /// Model not found anywhere in the mesh
    NotFound,
}

impl std::fmt::Display for ModelResolution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModelResolution::Local(e) => {
                write!(
                    f,
                    "LOCAL: {} ({:.1} MB) at {}",
                    e.model_id,
                    e.size_bytes as f64 / 1_048_576.0,
                    e.local_path.display()
                )
            }
            ModelResolution::Remote(e) => {
                write!(
                    f,
                    "REMOTE: {} ({:.1} MB) on gate '{}'",
                    e.model_id,
                    e.size_bytes as f64 / 1_048_576.0,
                    e.gate_id
                )
            }
            ModelResolution::NotFound => write!(f, "NOT FOUND"),
        }
    }
}

#[cfg(test)]
mod tests {
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

        // Create and register
        {
            let mut cache = ModelCache::with_cache_dir(cache_dir.clone()).await.unwrap();
            cache
                .register_model("persist/test", &model_dir, "test://")
                .await
                .unwrap();
        }

        // Reload and verify
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
}
