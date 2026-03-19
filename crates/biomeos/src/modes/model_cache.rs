// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Model Cache mode - Manage cached AI models across the NUCLEUS mesh
//!
//! Wraps biomeos-core::model_cache with CLI interface.

use anyhow::Result;
use biomeos_core::model_cache::{ModelCache, ModelResolution};
use std::path::Path;
#[cfg(test)]
use std::path::PathBuf;

use crate::ModelCacheCommand;

/// Format bytes as MB string (e.g. "123.4 MB")
pub(crate) fn format_size_mb(bytes: u64) -> String {
    // u64->f64: precision loss acceptable for size display (exact up to 2^53 bytes)
    format!("{:.1} MB", bytes as f64 / 1_048_576.0)
}

/// Format bytes as GB string (e.g. "1.2 GB")
pub(crate) fn format_size_gb(bytes: u64) -> String {
    // u64->f64: precision loss acceptable for size display (exact up to 2^53 bytes)
    format!("{:.1} GB", bytes as f64 / 1_073_741_824.0)
}

/// Convert HuggingFace cache dir name to model ID (e.g. "models--org--name" -> "org/name")
pub(crate) fn hf_dir_to_model_id(dir_name: &str) -> Option<String> {
    dir_name
        .strip_prefix("models--")
        .map(|s| s.replace("--", "/"))
}

/// Run model cache command
pub async fn run(command: ModelCacheCommand) -> Result<()> {
    match command {
        ModelCacheCommand::ImportHf => import_huggingface().await,
        ModelCacheCommand::List => list_models().await,
        ModelCacheCommand::Resolve { model_id } => resolve_model(&model_id).await,
        ModelCacheCommand::Register { model_id, path } => register_model(&model_id, &path).await,
        ModelCacheCommand::Status => show_status().await,
    }
}

#[cfg(test)]
pub async fn run_with(
    cache_dir: PathBuf,
    hf_hub_dir: Option<PathBuf>,
    command: ModelCacheCommand,
) -> Result<()> {
    match command {
        ModelCacheCommand::ImportHf => {
            import_huggingface_with(&cache_dir, hf_hub_dir.as_deref()).await
        }
        ModelCacheCommand::List => list_models_with(&cache_dir).await,
        ModelCacheCommand::Resolve { model_id } => resolve_model_with(&cache_dir, &model_id).await,
        ModelCacheCommand::Register { model_id, path } => {
            register_model_with(&cache_dir, &model_id, &path).await
        }
        ModelCacheCommand::Status => show_status_with(&cache_dir, hf_hub_dir.as_deref()).await,
    }
}

/// Import all models from HuggingFace cache
async fn import_huggingface() -> Result<()> {
    println!("\n  NUCLEUS Model Cache - HuggingFace Import");
    println!("  =========================================\n");

    let mut cache = ModelCache::new().await?;

    // First show what's already cached
    let existing = cache.list_models();
    if !existing.is_empty() {
        println!("  Already cached:");
        for model in &existing {
            println!(
                "    {} ({})",
                model.model_id,
                format_size_mb(model.size_bytes)
            );
        }
        println!();
    }

    // Import from HuggingFace
    let imported = cache.import_huggingface_cache().await?;

    if imported.is_empty() {
        println!("  No new models found in HuggingFace cache.");
        println!("  (All models already registered or no HF cache found)");
    } else {
        println!("  Imported {} new models:", imported.len());
        for model_id in &imported {
            if let Some(entry) = cache.get_model(model_id) {
                println!(
                    "    + {} ({}, {})",
                    model_id,
                    format_size_mb(entry.size_bytes),
                    entry.format
                );
            }
        }
    }

    // Summary
    let all = cache.list_models();
    let total_size: u64 = all.iter().map(|m| m.size_bytes).sum();
    println!();
    println!(
        "  Total: {} models, {} cached",
        all.len(),
        format_size_gb(total_size)
    );
    println!();

    Ok(())
}

#[cfg(test)]
async fn import_huggingface_with(cache_dir: &Path, hf_hub_dir: Option<&Path>) -> Result<()> {
    println!("\n  NUCLEUS Model Cache - HuggingFace Import");
    println!("  =========================================\n");

    let mut cache = ModelCache::with_cache_dir(cache_dir.to_path_buf()).await?;

    let existing = cache.list_models();
    if !existing.is_empty() {
        println!("  Already cached:");
        for model in &existing {
            println!(
                "    {} ({})",
                model.model_id,
                format_size_mb(model.size_bytes)
            );
        }
        println!();
    }

    let imported = if let Some(hf) = hf_hub_dir {
        cache.import_huggingface_cache_from(hf).await?
    } else {
        vec![]
    };

    if imported.is_empty() {
        println!("  No new models found in HuggingFace cache.");
        println!("  (All models already registered or no HF cache found)");
    } else {
        println!("  Imported {} new models:", imported.len());
        for model_id in &imported {
            if let Some(entry) = cache.get_model(model_id) {
                println!(
                    "    + {} ({}, {})",
                    model_id,
                    format_size_mb(entry.size_bytes),
                    entry.format
                );
            }
        }
    }

    let all = cache.list_models();
    let total_size: u64 = all.iter().map(|m| m.size_bytes).sum();
    println!();
    println!(
        "  Total: {} models, {} cached",
        all.len(),
        format_size_gb(total_size)
    );
    println!();

    Ok(())
}

/// List all cached models
async fn list_models() -> Result<()> {
    println!("\n  NUCLEUS Model Cache");
    println!("  ===================\n");

    let cache = ModelCache::new().await?;
    let models = cache.list_models();

    if models.is_empty() {
        println!("  No models cached.");
        println!("  Run 'biomeos model-cache import-hf' to import from HuggingFace.");
        return Ok(());
    }

    println!("  {:<40} {:>10}  {:>12}  PATH", "MODEL", "FORMAT", "SIZE",);
    println!("  {}", "-".repeat(90));

    for model in &models {
        println!(
            "  {:<40} {:>10}  {:>9}  {}",
            model.model_id,
            model.format,
            format_size_mb(model.size_bytes),
            model.local_path.display()
        );
    }

    let total_size: u64 = models.iter().map(|m| m.size_bytes).sum();
    println!();
    println!(
        "  Total: {} models, {}",
        models.len(),
        format_size_gb(total_size)
    );
    println!();

    Ok(())
}

#[cfg(test)]
async fn list_models_with(cache_dir: &Path) -> Result<()> {
    println!("\n  NUCLEUS Model Cache");
    println!("  ===================\n");

    let cache = ModelCache::with_cache_dir(cache_dir.to_path_buf()).await?;
    let models = cache.list_models();

    if models.is_empty() {
        println!("  No models cached.");
        println!("  Run 'biomeos model-cache import-hf' to import from HuggingFace.");
        return Ok(());
    }

    println!("  {:<40} {:>10}  {:>12}  PATH", "MODEL", "FORMAT", "SIZE",);
    println!("  {}", "-".repeat(90));

    for model in &models {
        println!(
            "  {:<40} {:>10}  {:>9}  {}",
            model.model_id,
            model.format,
            format_size_mb(model.size_bytes),
            model.local_path.display()
        );
    }

    let total_size: u64 = models.iter().map(|m| m.size_bytes).sum();
    println!();
    println!(
        "  Total: {} models, {}",
        models.len(),
        format_size_gb(total_size)
    );
    println!();

    Ok(())
}

/// Resolve a model across the mesh
async fn resolve_model(model_id: &str) -> Result<()> {
    println!("\n  Resolving: {model_id}\n");

    let cache = ModelCache::new().await?;
    let resolution = cache.resolve(model_id).await;

    match &resolution {
        ModelResolution::Local(entry) => {
            println!("  FOUND (local cache)");
            println!("    Path:   {}", entry.local_path.display());
            println!("    Size:   {}", format_size_mb(entry.size_bytes));
            println!("    Format: {}", entry.format);
            println!("    Cached: {}", entry.cached_at);
            println!("    Gate:   {}", entry.gate_id);
            if !entry.files.is_empty() {
                println!("    Files:  {}", entry.files.len());
            }
        }
        ModelResolution::Remote(entry) => {
            println!("  FOUND (remote gate)");
            println!("    Gate:   {}", entry.gate_id);
            println!("    Size:   {}", format_size_mb(entry.size_bytes));
            println!(
                "    Transfer needed: Use Songbird to fetch from {}",
                entry.gate_id
            );
        }
        ModelResolution::NotFound => {
            println!("  NOT FOUND in local cache or mesh.");
            println!();
            println!("  To cache this model:");
            println!(
                "    1. Download: python3 -c \"from transformers import AutoModel; AutoModel.from_pretrained('{model_id}')\""
            );
            println!("    2. Register: biomeos model-cache import-hf");
        }
    }
    println!();

    Ok(())
}

#[cfg(test)]
async fn resolve_model_with(cache_dir: &Path, model_id: &str) -> Result<()> {
    println!("\n  Resolving: {model_id}\n");

    let cache = ModelCache::with_cache_dir(cache_dir.to_path_buf()).await?;
    let resolution = cache.resolve(model_id).await;

    match &resolution {
        ModelResolution::Local(entry) => {
            println!("  FOUND (local cache)");
            println!("    Path:   {}", entry.local_path.display());
            println!("    Size:   {}", format_size_mb(entry.size_bytes));
            println!("    Format: {}", entry.format);
            println!("    Cached: {}", entry.cached_at);
            println!("    Gate:   {}", entry.gate_id);
            if !entry.files.is_empty() {
                println!("    Files:  {}", entry.files.len());
            }
        }
        ModelResolution::Remote(entry) => {
            println!("  FOUND (remote gate)");
            println!("    Gate:   {}", entry.gate_id);
            println!("    Size:   {}", format_size_mb(entry.size_bytes));
            println!(
                "    Transfer needed: Use Songbird to fetch from {}",
                entry.gate_id
            );
        }
        ModelResolution::NotFound => {
            println!("  NOT FOUND in local cache or mesh.");
            println!();
            println!("  To cache this model:");
            println!(
                "    1. Download: python3 -c \"from transformers import AutoModel; AutoModel.from_pretrained('{model_id}')\""
            );
            println!("    2. Register: biomeos model-cache import-hf");
        }
    }
    println!();

    Ok(())
}

/// Register a model from a local path
async fn register_model(model_id: &str, path: &Path) -> Result<()> {
    println!("\n  Registering: {} -> {}\n", model_id, path.display());

    let mut cache = ModelCache::new().await?;
    cache
        .register_model(model_id, path, &format!("local:{}", path.display()))
        .await?;

    println!("  Registered successfully.");

    if let Some(entry) = cache.get_model(model_id) {
        println!("    Size:   {}", format_size_mb(entry.size_bytes));
        println!("    Format: {}", entry.format);
        println!("    Files:  {}", entry.files.len());
    }
    println!();

    Ok(())
}

#[cfg(test)]
async fn register_model_with(cache_dir: &Path, model_id: &str, path: &Path) -> Result<()> {
    println!("\n  Registering: {} -> {}\n", model_id, path.display());

    let mut cache = ModelCache::with_cache_dir(cache_dir.to_path_buf()).await?;
    cache
        .register_model(model_id, path, &format!("local:{}", path.display()))
        .await?;

    println!("  Registered successfully.");

    if let Some(entry) = cache.get_model(model_id) {
        println!("    Size:   {}", format_size_mb(entry.size_bytes));
        println!("    Format: {}", entry.format);
        println!("    Files:  {}", entry.files.len());
    }
    println!();

    Ok(())
}

/// Show model cache status
async fn show_status() -> Result<()> {
    println!("\n  NUCLEUS Model Cache Status");
    println!("  ==========================\n");

    let cache = ModelCache::new().await?;
    let models = cache.list_models();

    let total_size: u64 = models.iter().map(|m| m.size_bytes).sum();

    println!("  Local cache:");
    println!("    Models:    {}", models.len());
    println!("    Size:      {}", format_size_gb(total_size));

    // Check NestGate connection
    let nestgate_status = if cache.list_mesh_models().await.is_empty() {
        "offline (filesystem-only mode)"
    } else {
        "connected (mesh registry active)"
    };
    println!("    NestGate:  {nestgate_status}");

    // Check HuggingFace cache
    let hf_cache = std::env::var("HOME")
        .map(|h| std::path::Path::new(&h).join(".cache/huggingface/hub"))
        .ok();

    if let Some(ref hf_path) = hf_cache {
        if hf_path.exists() {
            let hf_models: Vec<_> = std::fs::read_dir(hf_path)
                .into_iter()
                .flatten()
                .filter_map(std::result::Result::ok)
                .filter(|e| e.file_name().to_string_lossy().starts_with("models--"))
                .collect();

            let unregistered = hf_models
                .iter()
                .filter(|e| {
                    hf_dir_to_model_id(&e.file_name().to_string_lossy())
                        .is_none_or(|id| !cache.has_model(&id))
                })
                .count();

            println!();
            println!("  HuggingFace cache:");
            println!("    Models:       {}", hf_models.len());
            println!("    Unregistered: {unregistered}");
            if unregistered > 0 {
                println!("    Run 'biomeos model-cache import-hf' to register them");
            }
        }
    }

    println!();
    Ok(())
}

#[cfg(test)]
async fn show_status_with(cache_dir: &Path, hf_hub_dir: Option<&Path>) -> Result<()> {
    println!("\n  NUCLEUS Model Cache Status");
    println!("  ==========================\n");

    let cache = ModelCache::with_cache_dir(cache_dir.to_path_buf()).await?;
    let models = cache.list_models();

    let total_size: u64 = models.iter().map(|m| m.size_bytes).sum();

    println!("  Local cache:");
    println!("    Models:    {}", models.len());
    println!("    Size:      {}", format_size_gb(total_size));

    let nestgate_status = if cache.list_mesh_models().await.is_empty() {
        "offline (filesystem-only mode)"
    } else {
        "connected (mesh registry active)"
    };
    println!("    NestGate:  {nestgate_status}");

    if let Some(hf_path) = hf_hub_dir {
        if hf_path.exists() {
            let hf_models: Vec<_> = std::fs::read_dir(hf_path)
                .into_iter()
                .flatten()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_name().to_string_lossy().starts_with("models--"))
                .collect();

            let unregistered = hf_models
                .iter()
                .filter(|e| {
                    hf_dir_to_model_id(&e.file_name().to_string_lossy())
                        .is_none_or(|id| !cache.has_model(&id))
                })
                .count();

            println!();
            println!("  HuggingFace cache:");
            println!("    Models:       {}", hf_models.len());
            println!("    Unregistered: {unregistered}");
            if unregistered > 0 {
                println!("    Run 'biomeos model-cache import-hf' to register them");
            }
        }
    }

    println!();
    Ok(())
}

#[cfg(test)]
#[path = "model_cache_tests.rs"]
mod tests;
