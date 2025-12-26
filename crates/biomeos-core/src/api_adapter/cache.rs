//! API Adapter Caching
//!
//! Caches discovered API patterns for fast reuse.
//! Avoids re-discovery on every startup.

use super::ApiAdapter;
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// Get the API adapter cache directory
pub fn get_cache_dir() -> Result<PathBuf> {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .context("Could not determine home directory")?;

    let cache_dir = Path::new(&home)
        .join(".cache")
        .join("biomeos")
        .join("api_adapters");

    fs::create_dir_all(&cache_dir)?;

    Ok(cache_dir)
}

/// Save an API adapter to cache
pub fn save_adapter(adapter: &ApiAdapter) -> Result<()> {
    let cache_dir = get_cache_dir()?;
    let cache_file = cache_dir.join(format!("{}.json", adapter.primal_name()));

    let json = serde_json::to_string_pretty(adapter)?;
    fs::write(&cache_file, json)?;

    println!(
        "💾 Cached API adapter for {} at {}",
        adapter.primal_name(),
        cache_file.display()
    );

    Ok(())
}

/// Load an API adapter from cache
pub fn load_adapter(primal_name: &str) -> Result<Option<ApiAdapter>> {
    let cache_dir = get_cache_dir()?;
    let cache_file = cache_dir.join(format!("{}.json", primal_name));

    if !cache_file.exists() {
        return Ok(None);
    }

    let json = fs::read_to_string(&cache_file)?;
    let adapter: ApiAdapter = serde_json::from_str(&json)?;

    println!("📦 Loaded cached API adapter for {}", primal_name);

    Ok(Some(adapter))
}

/// Check if an adapter is cached
pub fn is_cached(primal_name: &str) -> bool {
    let cache_dir = match get_cache_dir() {
        Ok(dir) => dir,
        Err(_) => return false,
    };

    cache_dir.join(format!("{}.json", primal_name)).exists()
}

/// Clear cache for a specific primal
pub fn clear_cache(primal_name: &str) -> Result<()> {
    let cache_dir = get_cache_dir()?;
    let cache_file = cache_dir.join(format!("{}.json", primal_name));

    if cache_file.exists() {
        fs::remove_file(&cache_file)?;
        println!("🗑️  Cleared cache for {}", primal_name);
    }

    Ok(())
}

/// Clear all cached adapters
pub fn clear_all_cache() -> Result<()> {
    let cache_dir = get_cache_dir()?;

    if cache_dir.exists() {
        for entry in fs::read_dir(&cache_dir)? {
            let entry = entry?;
            if entry.path().extension().and_then(|s| s.to_str()) == Some("json") {
                fs::remove_file(entry.path())?;
            }
        }
        println!("🗑️  Cleared all API adapter cache");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_cache_dir() {
        let cache_dir = get_cache_dir();
        assert!(cache_dir.is_ok());
    }
}
