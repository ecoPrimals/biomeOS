//! Adapter cache - persist learned interfaces

use super::types::PrimalAdapter;
use anyhow::{Context, Result};
use std::path::PathBuf;

/// Cache for storing discovered adapters
pub struct AdapterCache {
    cache_dir: PathBuf,
}

impl AdapterCache {
    /// Create new cache
    pub fn new() -> Result<Self> {
        // Use etcetera (Pure Rust!) for home directory
        use etcetera::base_strategy::{choose_base_strategy, BaseStrategy};
        let strategy = choose_base_strategy().context("Could not determine base strategy")?;
        let cache_dir = strategy.home_dir().join(".biomeos").join("primal_adapters");

        std::fs::create_dir_all(&cache_dir)?;

        Ok(Self { cache_dir })
    }

    /// Get cache file path for a primal
    fn cache_path(&self, primal_name: &str) -> PathBuf {
        self.cache_dir.join(format!("{}.yaml", primal_name))
    }

    /// Save adapter to cache
    pub fn save(&self, adapter: &PrimalAdapter) -> Result<()> {
        let path = self.cache_path(&adapter.name);
        let yaml = serde_yaml::to_string(adapter)?;
        std::fs::write(path, yaml)?;
        Ok(())
    }

    /// Load adapter from cache
    pub fn load(&self, primal_name: &str) -> Result<PrimalAdapter> {
        let path = self.cache_path(primal_name);
        let yaml = std::fs::read_to_string(path)?;
        let adapter: PrimalAdapter = serde_yaml::from_str(&yaml)?;
        Ok(adapter)
    }

    /// Check if adapter is cached
    pub fn exists(&self, primal_name: &str) -> bool {
        self.cache_path(primal_name).exists()
    }

    /// Invalidate cache for a primal
    pub fn invalidate(&self, primal_name: &str) -> Result<()> {
        let path = self.cache_path(primal_name);
        if path.exists() {
            std::fs::remove_file(path)?;
        }
        Ok(())
    }

    /// List all cached adapters
    pub fn list(&self) -> Result<Vec<String>> {
        let mut names = Vec::new();

        for entry in std::fs::read_dir(&self.cache_dir)? {
            let entry = entry?;
            if let Some(name) = entry.path().file_stem() {
                if let Some(name_str) = name.to_str() {
                    names.push(name_str.to_string());
                }
            }
        }

        Ok(names)
    }
}

impl Default for AdapterCache {
    fn default() -> Self {
        // Use panicking default since this is called implicitly
        // Production code should explicitly call new() and handle errors
        Self::new().unwrap_or_else(|e| {
            tracing::error!("Failed to create adapter cache: {}", e);
            panic!("Could not initialize adapter cache: {}", e)
        })
    }
}

/// Save adapter to cache (convenience function)
pub fn save_adapter(adapter: &PrimalAdapter) -> Result<()> {
    let cache = AdapterCache::new()?;
    cache.save(adapter)
}

/// Load adapter from cache (convenience function)
pub fn load_adapter(primal_name: &str) -> Result<PrimalAdapter> {
    let cache = AdapterCache::new()?;
    cache.load(primal_name)
}

/// Check if adapter is cached
#[allow(dead_code)] // Used by external tools and future features
pub fn is_cached(primal_name: &str) -> bool {
    AdapterCache::new()
        .map(|cache| cache.exists(primal_name))
        .unwrap_or(false)
}
