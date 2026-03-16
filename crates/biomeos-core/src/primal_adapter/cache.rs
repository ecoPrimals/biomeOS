// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

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
        use etcetera::base_strategy::{BaseStrategy, choose_base_strategy};
        let strategy = choose_base_strategy().context("Could not determine base strategy")?;
        let cache_dir = strategy.home_dir().join(".biomeos").join("primal_adapters");

        std::fs::create_dir_all(&cache_dir)?;

        Ok(Self { cache_dir })
    }

    /// Create cache with a specific directory (for testing and custom installations)
    pub fn with_cache_dir(cache_dir: PathBuf) -> Result<Self> {
        std::fs::create_dir_all(&cache_dir).context("Failed to create adapter cache directory")?;
        Ok(Self { cache_dir })
    }

    /// Get cache file path for a primal
    fn cache_path(&self, primal_name: &str) -> PathBuf {
        self.cache_dir.join(format!("{primal_name}.yaml"))
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
            if let Some(name) = entry.path().file_stem()
                && let Some(name_str) = name.to_str()
            {
                names.push(name_str.to_string());
            }
        }

        Ok(names)
    }
}

// EVOLVED: Removed panicking Default impl
// Use AdapterCache::new() explicitly for proper error handling
// If you need a Default, create infallible fallback with in-memory only cache

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primal_adapter::types::{
        LifecycleCapabilities, PortConfigMethod, PrimalCapabilities, PrimalInterface, PrimalState,
    };
    use tempfile::TempDir;

    fn make_test_adapter(name: &str) -> PrimalAdapter {
        PrimalAdapter {
            name: name.to_string(),
            binary: std::path::PathBuf::from("/usr/bin/test-primal"),
            interface: PrimalInterface::Direct {
                args: vec!["serve".to_string()],
            },
            capabilities: PrimalCapabilities {
                lifecycle: LifecycleCapabilities::default(),
                health_check: None,
                port_config: PortConfigMethod::EnvVar("PORT".to_string()),
                has_version_cmd: true,
                has_fast_help: true,
            },
            state: PrimalState::NotStarted,
            discovered_at: chrono::Utc::now(),
            version: Some("1.0.0".to_string()),
        }
    }

    #[test]
    fn test_adapter_cache_save_load() {
        let tmp = TempDir::new().expect("create temp dir");
        let cache = AdapterCache::with_cache_dir(tmp.path().to_path_buf()).expect("create cache");

        let adapter = make_test_adapter("squirrel");
        cache.save(&adapter).expect("save");

        assert!(cache.exists("squirrel"));

        let loaded = cache.load("squirrel").expect("load");
        assert_eq!(loaded.name, "squirrel");
        assert_eq!(loaded.version, Some("1.0.0".to_string()));
    }

    #[test]
    fn test_adapter_cache_invalidate() {
        let tmp = TempDir::new().expect("create temp dir");
        let cache = AdapterCache::with_cache_dir(tmp.path().to_path_buf()).expect("create cache");

        let adapter = make_test_adapter("songbird");
        cache.save(&adapter).expect("save");
        assert!(cache.exists("songbird"));

        cache.invalidate("songbird").expect("invalidate");
        assert!(!cache.exists("songbird"));

        let result = cache.load("songbird");
        assert!(result.is_err());
    }

    #[test]
    fn test_adapter_cache_list() {
        let tmp = TempDir::new().expect("create temp dir");
        let cache = AdapterCache::with_cache_dir(tmp.path().to_path_buf()).expect("create cache");

        cache.save(&make_test_adapter("squirrel")).expect("save");
        cache.save(&make_test_adapter("songbird")).expect("save");

        let names = cache.list().expect("list");
        assert_eq!(names.len(), 2);
        assert!(names.contains(&"squirrel".to_string()));
        assert!(names.contains(&"songbird".to_string()));
    }

    #[test]
    fn test_adapter_cache_exists_false() {
        let tmp = TempDir::new().expect("create temp dir");
        let cache = AdapterCache::with_cache_dir(tmp.path().to_path_buf()).expect("create cache");
        assert!(!cache.exists("nonexistent"));
    }

    #[test]
    fn test_adapter_cache_load_missing() {
        let tmp = TempDir::new().expect("create temp dir");
        let cache = AdapterCache::with_cache_dir(tmp.path().to_path_buf()).expect("create cache");
        let result = cache.load("nonexistent");
        assert!(result.is_err());
    }
}
