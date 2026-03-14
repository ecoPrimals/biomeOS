// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

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

pub(crate) fn default_format() -> String {
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
