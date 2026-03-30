// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Model cache type definitions

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Metadata about a cached model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelEntry {
    /// Model identifier (e.g., "TinyLlama/TinyLlama-1.1B-Chat-v1.0")
    pub model_id: String,

    /// Local filesystem path to model directory
    pub local_path: PathBuf,

    /// Total size in bytes
    pub size_bytes: u64,

    /// Source URL (e.g., `HuggingFace` hub URL)
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

pub fn default_format() -> String {
    "huggingface".to_string()
}

/// Individual file within a model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelFile {
    /// File path relative to the model root
    pub relative_path: String,
    /// File size in bytes
    pub size_bytes: u64,
    /// Optional SHA-256 digest for integrity verification
    #[serde(default)]
    pub sha256: Option<String>,
}

/// Model cache manifest (local JSON file)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CacheManifest {
    /// Manifest format version
    pub version: u32,
    /// Cached models keyed by model ID
    pub models: HashMap<String, ModelEntry>,
}

impl CacheManifest {
    pub(crate) fn new() -> Self {
        Self {
            version: 1,
            models: HashMap::new(),
        }
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
            Self::Local(e) => {
                write!(
                    f,
                    "LOCAL: {} ({:.1} MB) at {}",
                    e.model_id,
                    e.size_bytes as f64 / 1_048_576.0,
                    e.local_path.display()
                )
            }
            Self::Remote(e) => {
                write!(
                    f,
                    "REMOTE: {} ({:.1} MB) on gate '{}'",
                    e.model_id,
                    e.size_bytes as f64 / 1_048_576.0,
                    e.gate_id
                )
            }
            Self::NotFound => write!(f, "NOT FOUND"),
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    #[test]
    fn model_resolution_display_local() {
        let entry = ModelEntry {
            model_id: "test/model".to_string(),
            local_path: PathBuf::from("/data/models/test"),
            size_bytes: 2_097_152,
            source: "https://huggingface.co/test".to_string(),
            sha256: None,
            cached_at: "2025-01-01".to_string(),
            gate_id: "gate-1".to_string(),
            format: "huggingface".to_string(),
            files: vec![],
        };
        let resolution = ModelResolution::Local(entry);
        let s = resolution.to_string();
        assert!(s.starts_with("LOCAL:"));
        assert!(s.contains("test/model"));
        assert!(s.contains("2.0 MB"));
        assert!(s.contains("/data/models/test"));
    }

    #[test]
    fn model_resolution_display_remote() {
        let entry = ModelEntry {
            model_id: "remote/model".to_string(),
            local_path: PathBuf::from("/local"),
            size_bytes: 1_048_576,
            source: "https://hf.co".to_string(),
            sha256: None,
            cached_at: "2025-01-01".to_string(),
            gate_id: "gate-alpha".to_string(),
            format: "gguf".to_string(),
            files: vec![],
        };
        let resolution = ModelResolution::Remote(entry);
        let s = resolution.to_string();
        assert!(s.starts_with("REMOTE:"));
        assert!(s.contains("remote/model"));
        assert!(s.contains("1.0 MB"));
        assert!(s.contains("gate-alpha"));
    }

    #[test]
    fn model_resolution_display_not_found() {
        let resolution = ModelResolution::NotFound;
        assert_eq!(resolution.to_string(), "NOT FOUND");
    }

    #[test]
    fn cache_manifest_new() {
        let manifest = CacheManifest::new();
        assert_eq!(manifest.version, 1);
        assert!(manifest.models.is_empty());
    }

    #[test]
    fn model_entry_deserialize_default_format() {
        let json = r#"{
            "model_id": "test/model",
            "local_path": "/data/test",
            "size_bytes": 1000,
            "source": "https://example.com",
            "cached_at": "2025-01-01",
            "gate_id": "gate-1"
        }"#;
        let entry: ModelEntry = serde_json::from_str(json).unwrap();
        assert_eq!(entry.format, "huggingface");
    }

    #[test]
    fn model_file_serde_roundtrip() {
        let file = ModelFile {
            relative_path: "model.safetensors".to_string(),
            size_bytes: 1024,
            sha256: Some("abc123".to_string()),
        };
        let json = serde_json::to_string(&file).unwrap();
        let restored: ModelFile = serde_json::from_str(&json).unwrap();
        assert_eq!(file.relative_path, restored.relative_path);
        assert_eq!(file.size_bytes, restored.size_bytes);
        assert_eq!(file.sha256, restored.sha256);
    }
}
