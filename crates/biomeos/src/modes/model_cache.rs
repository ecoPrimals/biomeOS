//! Model Cache mode - Manage cached AI models across the NUCLEUS mesh
//!
//! Wraps biomeos-core::model_cache with CLI interface.

use anyhow::Result;
use biomeos_core::model_cache::{ModelCache, ModelResolution};
use std::path::Path;

use crate::ModelCacheCommand;

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
                "    {} ({:.1} MB)",
                model.model_id,
                model.size_bytes as f64 / 1_048_576.0
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
                    "    + {} ({:.1} MB, {})",
                    model_id,
                    entry.size_bytes as f64 / 1_048_576.0,
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
        "  Total: {} models, {:.1} GB cached",
        all.len(),
        total_size as f64 / 1_073_741_824.0
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
            "  {:<40} {:>10}  {:>9.1} MB  {}",
            model.model_id,
            model.format,
            model.size_bytes as f64 / 1_048_576.0,
            model.local_path.display()
        );
    }

    let total_size: u64 = models.iter().map(|m| m.size_bytes).sum();
    println!();
    println!(
        "  Total: {} models, {:.1} GB",
        models.len(),
        total_size as f64 / 1_073_741_824.0
    );
    println!();

    Ok(())
}

/// Resolve a model across the mesh
async fn resolve_model(model_id: &str) -> Result<()> {
    println!("\n  Resolving: {}\n", model_id);

    let cache = ModelCache::new().await?;
    let resolution = cache.resolve(model_id).await;

    match &resolution {
        ModelResolution::Local(entry) => {
            println!("  FOUND (local cache)");
            println!("    Path:   {}", entry.local_path.display());
            println!(
                "    Size:   {:.1} MB",
                entry.size_bytes as f64 / 1_048_576.0
            );
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
            println!(
                "    Size:   {:.1} MB",
                entry.size_bytes as f64 / 1_048_576.0
            );
            println!(
                "    Transfer needed: Use Songbird to fetch from {}",
                entry.gate_id
            );
        }
        ModelResolution::NotFound => {
            println!("  NOT FOUND in local cache or mesh.");
            println!();
            println!("  To cache this model:");
            println!("    1. Download: python3 -c \"from transformers import AutoModel; AutoModel.from_pretrained('{}')\"", model_id);
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
        println!(
            "    Size:   {:.1} MB",
            entry.size_bytes as f64 / 1_048_576.0
        );
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
    println!(
        "    Size:      {:.1} GB",
        total_size as f64 / 1_073_741_824.0
    );

    // Check NestGate connection
    let nestgate_status = if cache.list_mesh_models().await.is_empty() {
        "offline (filesystem-only mode)"
    } else {
        "connected (mesh registry active)"
    };
    println!("    NestGate:  {}", nestgate_status);

    // Check HuggingFace cache
    let hf_cache = std::env::var("HOME")
        .map(|h| std::path::Path::new(&h).join(".cache/huggingface/hub"))
        .ok();

    if let Some(ref hf_path) = hf_cache {
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
                    let model_id = e
                        .file_name()
                        .to_string_lossy()
                        .strip_prefix("models--")
                        .unwrap_or_default()
                        .replace("--", "/");
                    !cache.has_model(&model_id)
                })
                .count();

            println!();
            println!("  HuggingFace cache:");
            println!("    Models:       {}", hf_models.len());
            println!("    Unregistered: {}", unregistered);
            if unregistered > 0 {
                println!("    Run 'biomeos model-cache import-hf' to register them");
            }
        }
    }

    println!();
    Ok(())
}
