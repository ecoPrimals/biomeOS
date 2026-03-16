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
            println!("    1. Download: python3 -c \"from transformers import AutoModel; AutoModel.from_pretrained('{model_id}')\"");
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
            println!("    1. Download: python3 -c \"from transformers import AutoModel; AutoModel.from_pretrained('{model_id}')\"");
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
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;
    use crate::ModelCacheCommand;

    #[test]
    fn test_format_size_mb() {
        assert_eq!(format_size_mb(0), "0.0 MB");
        assert_eq!(format_size_mb(1_048_576), "1.0 MB");
        assert_eq!(format_size_mb(1_573_286), "1.5 MB"); // 1.5 * 1024^2
        assert_eq!(format_size_mb(104_857_600), "100.0 MB");
    }

    #[test]
    fn test_format_size_gb() {
        assert_eq!(format_size_gb(0), "0.0 GB");
        assert_eq!(format_size_gb(1_073_741_824), "1.0 GB");
        assert_eq!(format_size_gb(2_147_483_648), "2.0 GB");
    }

    #[test]
    fn test_hf_dir_to_model_id() {
        assert_eq!(
            hf_dir_to_model_id("models--TinyLlama--TinyLlama-1.1B-Chat-v1.0"),
            Some("TinyLlama/TinyLlama-1.1B-Chat-v1.0".to_string())
        );
        assert_eq!(
            hf_dir_to_model_id("models--meta-llama--Llama-2-7b-hf"),
            Some("meta-llama/Llama-2-7b-hf".to_string())
        );
        assert_eq!(
            hf_dir_to_model_id("models--simple"),
            Some("simple".to_string())
        );
        assert_eq!(hf_dir_to_model_id("other--prefix"), None);
        assert_eq!(hf_dir_to_model_id(""), None);
        assert_eq!(hf_dir_to_model_id("models--"), Some("".to_string()));
        assert_eq!(
            hf_dir_to_model_id("models--single--level"),
            Some("single/level".to_string())
        );
    }

    #[test]
    fn test_format_size_mb_large() {
        assert_eq!(format_size_mb(1_073_741_824), "1024.0 MB");
        assert_eq!(format_size_mb(2_097_152_000), "2000.0 MB");
    }

    #[test]
    fn test_format_size_gb_fractional() {
        assert_eq!(format_size_gb(1_610_612_736), "1.5 GB");
        assert_eq!(format_size_gb(5_368_709_120), "5.0 GB");
    }

    #[tokio::test]
    async fn test_run_list_empty_cache() {
        let temp = tempfile::tempdir().expect("temp dir");
        let cache_dir = temp.path().join("model-cache");
        std::fs::create_dir_all(&cache_dir).expect("create cache dir");

        let result = run_with(cache_dir, None, ModelCacheCommand::List).await;
        assert!(result.is_ok(), "list should succeed: {:?}", result.err());
    }

    #[tokio::test]
    async fn test_run_status() {
        let temp = tempfile::tempdir().expect("temp dir");
        let cache_dir = temp.path().join("model-cache");
        std::fs::create_dir_all(&cache_dir).expect("create cache dir");

        let result = run_with(cache_dir, None, ModelCacheCommand::Status).await;
        assert!(result.is_ok(), "status should succeed: {:?}", result.err());
    }

    #[tokio::test]
    async fn test_run_resolve_not_found() {
        let temp = tempfile::tempdir().expect("temp dir");
        let cache_dir = temp.path().join("model-cache");
        std::fs::create_dir_all(&cache_dir).expect("create cache dir");

        let result = run_with(
            cache_dir,
            None,
            ModelCacheCommand::Resolve {
                model_id: "nonexistent/model-xyz-123".to_string(),
            },
        )
        .await;
        assert!(
            result.is_ok(),
            "resolve should succeed (NotFound path): {:?}",
            result.err()
        );
    }

    #[tokio::test]
    async fn test_run_import_hf_empty() {
        let temp = tempfile::tempdir().expect("temp dir");
        let cache_dir = temp.path().join("model-cache");
        let hf_hub = temp.path().join("hf-hub");
        std::fs::create_dir_all(&cache_dir).expect("create cache dir");
        std::fs::create_dir_all(&hf_hub).expect("create HF hub dir");

        let result = run_with(cache_dir, Some(hf_hub), ModelCacheCommand::ImportHf).await;
        assert!(
            result.is_ok(),
            "import-hf should succeed: {:?}",
            result.err()
        );
    }

    #[tokio::test]
    async fn test_run_register_model_nonexistent_path_fails() {
        let temp = tempfile::tempdir().expect("temp dir");
        let cache_dir = temp.path().join("model-cache");
        std::fs::create_dir_all(&cache_dir).expect("create cache dir");

        let nonexistent = temp.path().join("nonexistent-model-dir-xyz");
        let result = run_with(
            cache_dir,
            None,
            ModelCacheCommand::Register {
                model_id: "test/nonexistent".to_string(),
                path: nonexistent,
            },
        )
        .await;
        assert!(
            result.is_err(),
            "register with nonexistent path should fail: {:?}",
            result
        );
    }

    #[tokio::test]
    async fn test_run_register_model() {
        let temp = tempfile::tempdir().expect("temp dir");
        let cache_dir = temp.path().join("model-cache");
        std::fs::create_dir_all(&cache_dir).expect("create cache dir");

        let model_dir = temp.path().join("test-model");
        std::fs::create_dir_all(&model_dir).expect("create model dir");
        std::fs::write(model_dir.join("config.json"), "{}").expect("write config");

        let result = run_with(
            cache_dir,
            None,
            ModelCacheCommand::Register {
                model_id: "test/register-model".to_string(),
                path: model_dir,
            },
        )
        .await;
        assert!(
            result.is_ok(),
            "register should succeed: {:?}",
            result.err()
        );
    }

    #[tokio::test]
    async fn test_run_list_with_models() {
        let temp = tempfile::tempdir().expect("temp dir");
        let cache_dir = temp.path().join("model-cache");
        std::fs::create_dir_all(&cache_dir).expect("create cache dir");

        let model_dir = temp.path().join("list-test-model");
        std::fs::create_dir_all(&model_dir).expect("create model dir");
        std::fs::write(model_dir.join("config.json"), "{}").expect("write config");

        run_with(
            cache_dir.clone(),
            None,
            ModelCacheCommand::Register {
                model_id: "test/list-model".to_string(),
                path: model_dir,
            },
        )
        .await
        .expect("register should succeed");

        let result = run_with(cache_dir, None, ModelCacheCommand::List).await;
        assert!(result.is_ok(), "list should succeed: {:?}", result.err());
    }

    #[tokio::test]
    async fn test_run_resolve_local() {
        let temp = tempfile::tempdir().expect("temp dir");
        let cache_dir = temp.path().join("model-cache");
        std::fs::create_dir_all(&cache_dir).expect("create cache dir");

        let model_dir = temp.path().join("resolve-test-model");
        std::fs::create_dir_all(&model_dir).expect("create model dir");
        std::fs::write(model_dir.join("config.json"), "{}").expect("write config");

        run_with(
            cache_dir.clone(),
            None,
            ModelCacheCommand::Register {
                model_id: "test/resolve-local".to_string(),
                path: model_dir.clone(),
            },
        )
        .await
        .expect("register should succeed");

        let result = run_with(
            cache_dir,
            None,
            ModelCacheCommand::Resolve {
                model_id: "test/resolve-local".to_string(),
            },
        )
        .await;
        assert!(result.is_ok(), "resolve should succeed: {:?}", result.err());
    }

    #[tokio::test]
    async fn test_show_status_with_hf_cache() {
        let temp = tempfile::tempdir().expect("temp dir");
        let cache_dir = temp.path().join("model-cache");
        std::fs::create_dir_all(&cache_dir).expect("create cache dir");
        let hf_hub = temp.path().join("hf-hub");
        std::fs::create_dir_all(&hf_hub).expect("create HF hub dir");
        std::fs::create_dir_all(hf_hub.join("models--test--model-xyz"))
            .expect("create HF model dir");

        let result = run_with(cache_dir, Some(hf_hub), ModelCacheCommand::Status).await;
        assert!(
            result.is_ok(),
            "status with HF cache should succeed: {:?}",
            result.err()
        );
    }

    #[tokio::test]
    async fn test_show_status_with_hf_cache_and_unregistered() {
        let temp = tempfile::tempdir().expect("temp dir");
        let cache_dir = temp.path().join("model-cache");
        std::fs::create_dir_all(&cache_dir).expect("create cache dir");
        let hf_hub = temp.path().join("hf-hub");
        std::fs::create_dir_all(hf_hub.join("models--unreg--model")).expect("create HF model dir");

        let result = run_with(cache_dir, Some(hf_hub), ModelCacheCommand::Status).await;
        assert!(
            result.is_ok(),
            "status with unregistered HF models should succeed: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_format_size_mb_zero() {
        assert_eq!(format_size_mb(0), "0.0 MB");
    }

    #[test]
    fn test_format_size_gb_zero() {
        assert_eq!(format_size_gb(0), "0.0 GB");
    }

    #[tokio::test]
    async fn test_run_import_hf_with_existing_models() {
        let temp = tempfile::tempdir().expect("temp dir");
        let cache_dir = temp.path().join("model-cache");
        std::fs::create_dir_all(&cache_dir).expect("create cache dir");
        let model_dir = temp.path().join("existing-model");
        std::fs::create_dir_all(&model_dir).expect("create model dir");
        std::fs::write(model_dir.join("config.json"), "{}").expect("write config");

        run_with(
            cache_dir.clone(),
            None,
            ModelCacheCommand::Register {
                model_id: "test/existing".to_string(),
                path: model_dir,
            },
        )
        .await
        .expect("register");

        let result = run_with(cache_dir.clone(), None, ModelCacheCommand::ImportHf).await;
        assert!(
            result.is_ok(),
            "import-hf with existing models should succeed: {:?}",
            result.err()
        );
    }

    #[tokio::test]
    async fn test_run_resolve_with_empty_model_id() {
        let temp = tempfile::tempdir().expect("temp dir");
        let cache_dir = temp.path().join("model-cache");
        std::fs::create_dir_all(&cache_dir).expect("create cache dir");

        let result = run_with(
            cache_dir,
            None,
            ModelCacheCommand::Resolve {
                model_id: "".to_string(),
            },
        )
        .await;
        assert!(result.is_ok(), "resolve empty model_id should not panic");
    }

    #[tokio::test]
    async fn test_run_status_no_hf_path() {
        let temp = tempfile::tempdir().expect("temp dir");
        let cache_dir = temp.path().join("model-cache");
        std::fs::create_dir_all(&cache_dir).expect("create cache dir");

        let result = run_with(cache_dir, None, ModelCacheCommand::Status).await;
        assert!(result.is_ok(), "status without HF should succeed");
    }

    #[test]
    fn test_hf_dir_to_model_id_edge_cases() {
        assert_eq!(
            hf_dir_to_model_id("models--a--b--c"),
            Some("a/b/c".to_string())
        );
        assert_eq!(
            hf_dir_to_model_id("models--single"),
            Some("single".to_string())
        );
    }

    #[test]
    fn test_format_size_mb_large_values() {
        // 1 TB in bytes
        assert_eq!(format_size_mb(1_099_511_627_776), "1048576.0 MB");
        // Edge: single byte
        assert_eq!(format_size_mb(1), "0.0 MB");
    }

    #[test]
    fn test_format_size_gb_large_values() {
        assert_eq!(format_size_gb(10_737_418_240), "10.0 GB");
        assert_eq!(format_size_gb(1), "0.0 GB");
    }

    #[test]
    fn test_hf_dir_to_model_id_multiple_dashes() {
        assert_eq!(
            hf_dir_to_model_id("models--org--repo--sub--path"),
            Some("org/repo/sub/path".to_string())
        );
    }

    #[test]
    fn test_hf_dir_to_model_id_no_prefix() {
        assert_eq!(hf_dir_to_model_id("random-dir"), None);
        assert_eq!(hf_dir_to_model_id("models"), None);
    }
}
