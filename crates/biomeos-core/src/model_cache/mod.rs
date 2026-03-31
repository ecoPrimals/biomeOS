// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Model Cache - NUCLEUS-integrated model artifact management
//!
//! Provides persistent caching for AI model files (`HuggingFace`, GGUF, safetensors)
//! across the biomeOS mesh. Uses `NestGate` for distributed storage when available,
//! with graceful fallback to local filesystem.

mod cache;
mod types;

#[cfg(test)]
mod cache_extra_tests;
#[cfg(test)]
mod tests;

pub use cache::ModelCache;
pub use types::{ModelCacheConfig, ModelEntry, ModelFile, ModelResolution};
