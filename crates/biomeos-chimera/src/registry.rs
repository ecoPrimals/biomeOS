// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Chimera Registry
//!
//! Manages a collection of chimera definitions, providing discovery and lookup.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use dashmap::DashMap;
use tracing::{debug, info, warn};
use walkdir::WalkDir;

use crate::definition::ChimeraDefinition;
use crate::error::{ChimeraError, ChimeraResult};

/// Registry of available chimera definitions
#[derive(Debug)]
pub struct ChimeraRegistry {
    /// Loaded chimera definitions (thread-safe)
    definitions: DashMap<String, Arc<ChimeraDefinition>>,

    /// Source directories
    source_dirs: Vec<PathBuf>,
}

impl ChimeraRegistry {
    /// Create a new empty registry
    #[must_use]
    pub fn new() -> Self {
        Self {
            definitions: DashMap::new(),
            source_dirs: Vec::new(),
        }
    }

    /// Create a registry from a definitions directory
    ///
    /// # Errors
    ///
    /// Returns an error if the directory cannot be read or contains invalid chimera definitions
    pub fn from_directory(path: impl AsRef<Path>) -> ChimeraResult<Self> {
        let mut registry = Self::new();
        registry.load_directory(path)?;
        Ok(registry)
    }

    /// Load chimera definitions from a directory
    ///
    /// # Errors
    ///
    /// Returns an error if the directory cannot be read or any chimera definition is invalid
    pub fn load_directory(&mut self, path: impl AsRef<Path>) -> ChimeraResult<usize> {
        let path = path.as_ref();

        if !path.exists() {
            return Err(ChimeraError::DefinitionNotFound {
                path: path.to_path_buf(),
            });
        }

        self.source_dirs.push(path.to_path_buf());

        let mut count = 0;
        for entry in WalkDir::new(path)
            .max_depth(2)
            .into_iter()
            .filter_map(Result::ok)
        {
            let file_path = entry.path();

            // Only process YAML files
            if file_path
                .extension()
                .is_some_and(|ext| ext == "yaml" || ext == "yml")
            {
                match self.load_file(file_path) {
                    Ok(()) => count += 1,
                    Err(e) => {
                        warn!("Failed to load chimera from {:?}: {}", file_path, e);
                    }
                }
            }
        }

        info!("Loaded {} chimera definitions from {:?}", count, path);
        Ok(count)
    }

    /// Load a single chimera definition file
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or parsed as a valid chimera definition
    pub fn load_file(&mut self, path: impl AsRef<Path>) -> ChimeraResult<()> {
        let path = path.as_ref();
        debug!("Loading chimera from {:?}", path);

        let definition = ChimeraDefinition::from_file(path)?;
        let id = definition.chimera.id.clone();

        self.definitions.insert(id.clone(), Arc::new(definition));
        debug!("Registered chimera: {}", id);

        Ok(())
    }

    /// Get a chimera by ID
    #[must_use]
    pub fn get(&self, id: &str) -> Option<Arc<ChimeraDefinition>> {
        self.definitions.get(id).map(|r| Arc::clone(r.value()))
    }

    /// Check if a chimera exists
    #[must_use]
    pub fn contains(&self, id: &str) -> bool {
        self.definitions.contains_key(id)
    }

    /// List all chimera IDs
    #[must_use]
    pub fn list(&self) -> Vec<String> {
        self.definitions.iter().map(|r| r.key().clone()).collect()
    }

    /// Get all chimera definitions
    #[must_use]
    pub fn all(&self) -> Vec<Arc<ChimeraDefinition>> {
        self.definitions
            .iter()
            .map(|r| Arc::clone(r.value()))
            .collect()
    }

    /// Number of registered chimeras
    #[must_use]
    pub fn len(&self) -> usize {
        self.definitions.len()
    }

    /// Check if registry is empty
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.definitions.is_empty()
    }

    /// Reload all definitions from source directories
    ///
    /// # Errors
    ///
    /// Returns an error if any source directory cannot be read or contains invalid definitions
    pub fn reload(&mut self) -> ChimeraResult<usize> {
        self.definitions.clear();

        let dirs: Vec<_> = self.source_dirs.clone();
        let mut total = 0;

        for dir in dirs {
            // Temporarily remove from source_dirs to avoid double-adding
            self.source_dirs.retain(|d| d != &dir);
            total += self.load_directory(&dir)?;
        }

        Ok(total)
    }

    /// Get chimeras by required primal
    #[must_use]
    pub fn by_primal(&self, primal: &str) -> Vec<Arc<ChimeraDefinition>> {
        self.definitions
            .iter()
            .filter(|r| r.value().required_primals().contains(&primal))
            .map(|r| Arc::clone(r.value()))
            .collect()
    }

    /// Get summary information
    #[must_use]
    pub fn summary(&self) -> HashMap<String, ChimeraSummary> {
        self.definitions
            .iter()
            .map(|r| {
                let def = r.value();
                (
                    r.key().clone(),
                    ChimeraSummary {
                        name: def.chimera.name.clone(),
                        version: def.chimera.version.clone(),
                        primals: def
                            .required_primals()
                            .iter()
                            .map(|s| (*s).to_string())
                            .collect(),
                        uses_arrays: def.uses_arrays(),
                    },
                )
            })
            .collect()
    }
}

impl Default for ChimeraRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Summary information about a chimera
#[derive(Debug, Clone)]
pub struct ChimeraSummary {
    /// Chimera name
    pub name: String,
    /// Version
    pub version: String,
    /// Required primals
    pub primals: Vec<String>,
    /// Whether it uses component arrays
    pub uses_arrays: bool,
}

#[cfg(test)]
mod tests {
    #![expect(clippy::unwrap_used, reason = "test assertions")]

    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    fn create_test_chimera(dir: &Path, id: &str) -> PathBuf {
        let yaml = format!(
            r#"
chimera:
  id: {id}
  name: Test {id}
  version: "1.0.0"
  description: Test chimera

components:
  beardog:
    source: primals/beardog
    version: ">=1.0.0"
    modules: []

fusion:
  bindings: {{}}
  api:
    endpoints: []
"#
        );

        let path = dir.join(format!("{id}.yaml"));
        let mut file = std::fs::File::create(&path).unwrap();
        file.write_all(yaml.as_bytes()).unwrap();
        path
    }

    #[test]
    fn test_registry_load_directory() {
        let temp_dir = TempDir::new().unwrap();
        create_test_chimera(temp_dir.path(), "chimera-a");
        create_test_chimera(temp_dir.path(), "chimera-b");

        let registry = ChimeraRegistry::from_directory(temp_dir.path()).unwrap();

        assert_eq!(registry.len(), 2);
        assert!(registry.contains("chimera-a"));
        assert!(registry.contains("chimera-b"));
    }

    #[test]
    fn test_registry_by_primal() {
        let temp_dir = TempDir::new().unwrap();
        create_test_chimera(temp_dir.path(), "test-chimera");

        let registry = ChimeraRegistry::from_directory(temp_dir.path()).unwrap();

        let beardog_chimeras = registry.by_primal("beardog");
        assert_eq!(beardog_chimeras.len(), 1);

        let songbird_chimeras = registry.by_primal("songbird");
        assert!(songbird_chimeras.is_empty());
    }

    #[test]
    fn test_registry_get_and_contains() {
        let temp_dir = TempDir::new().unwrap();
        create_test_chimera(temp_dir.path(), "chimera-x");

        let registry = ChimeraRegistry::from_directory(temp_dir.path()).unwrap();

        assert!(registry.contains("chimera-x"));
        assert!(!registry.contains("nonexistent"));

        let def = registry.get("chimera-x");
        assert!(def.is_some());
        assert_eq!(def.unwrap().chimera.id, "chimera-x");

        assert!(registry.get("nonexistent").is_none());
    }

    #[test]
    fn test_registry_list_and_all() {
        let temp_dir = TempDir::new().unwrap();
        create_test_chimera(temp_dir.path(), "a");
        create_test_chimera(temp_dir.path(), "b");

        let registry = ChimeraRegistry::from_directory(temp_dir.path()).unwrap();

        let list = registry.list();
        assert_eq!(list.len(), 2);
        assert!(list.contains(&"a".to_string()));
        assert!(list.contains(&"b".to_string()));

        let all = registry.all();
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn test_registry_is_empty_and_len() {
        let registry = ChimeraRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);

        let temp_dir = TempDir::new().unwrap();
        create_test_chimera(temp_dir.path(), "single");
        let registry = ChimeraRegistry::from_directory(temp_dir.path()).unwrap();
        assert!(!registry.is_empty());
        assert_eq!(registry.len(), 1);
    }

    #[test]
    fn test_registry_load_file() {
        let temp_dir = TempDir::new().unwrap();
        let path = create_test_chimera(temp_dir.path(), "loaded");

        let mut registry = ChimeraRegistry::new();
        registry.load_file(&path).unwrap();
        assert_eq!(registry.len(), 1);
        assert!(registry.contains("loaded"));
    }

    #[test]
    fn test_registry_from_nonexistent_dir() {
        let result = ChimeraRegistry::from_directory("/nonexistent/path/12345");
        assert!(result.is_err());
    }

    #[test]
    fn test_registry_summary() {
        let temp_dir = TempDir::new().unwrap();
        create_test_chimera(temp_dir.path(), "summary-test");

        let registry = ChimeraRegistry::from_directory(temp_dir.path()).unwrap();
        let summary = registry.summary();

        assert_eq!(summary.len(), 1);
        let s = summary.get("summary-test").unwrap();
        assert_eq!(s.name, "Test summary-test");
        assert_eq!(s.version, "1.0.0");
        assert!(s.primals.contains(&"beardog".to_string()));
    }

    #[test]
    fn test_registry_reload() {
        let temp_dir = TempDir::new().unwrap();
        create_test_chimera(temp_dir.path(), "reload-test");

        let mut registry = ChimeraRegistry::from_directory(temp_dir.path()).unwrap();
        assert_eq!(registry.len(), 1);

        let count = registry.reload().unwrap();
        assert_eq!(count, 1);
        assert_eq!(registry.len(), 1);
    }

    #[test]
    fn test_chimera_summary_struct() {
        let summary = ChimeraSummary {
            name: "Test".to_string(),
            version: "1.0.0".to_string(),
            primals: vec!["beardog".to_string(), "songbird".to_string()],
            uses_arrays: true,
        };
        assert_eq!(summary.name, "Test");
        assert_eq!(summary.primals.len(), 2);
        assert!(summary.uses_arrays);
    }

    fn create_chimera_with_array(dir: &Path, id: &str) -> PathBuf {
        let yaml = format!(
            r#"
chimera:
  id: {id}
  name: Test {id}
  version: "1.0.0"
  description: Test chimera with array

components:
  beardog:
    source: primals/beardog
    version: ">=1.0.0"
    modules: []
    array:
      enabled: true
      min: 2
      max: 8

fusion:
  bindings: {{}}
  api:
    endpoints: []
"#
        );
        let path = dir.join(format!("{id}.yaml"));
        let mut file = std::fs::File::create(&path).unwrap();
        file.write_all(yaml.as_bytes()).unwrap();
        path
    }

    fn create_chimera_with_songbird(dir: &Path, id: &str) -> PathBuf {
        let yaml = format!(
            r#"
chimera:
  id: {id}
  name: Test {id}
  version: "1.0.0"
  description: Test chimera with songbird

components:
  songbird:
    source: primals/songbird
    version: ">=1.0.0"
    modules: []

fusion:
  bindings: {{}}
  api:
    endpoints: []
"#
        );
        let path = dir.join(format!("{id}.yaml"));
        let mut file = std::fs::File::create(&path).unwrap();
        file.write_all(yaml.as_bytes()).unwrap();
        path
    }

    #[test]
    fn test_registry_default() {
        let registry = ChimeraRegistry::default();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
    }

    #[test]
    fn test_registry_load_directory_skips_invalid_yaml() {
        let temp_dir = TempDir::new().unwrap();
        create_test_chimera(temp_dir.path(), "valid");
        let invalid_path = temp_dir.path().join("invalid.yaml");
        std::fs::write(&invalid_path, "not: valid: yaml: [").unwrap();

        let registry = ChimeraRegistry::from_directory(temp_dir.path()).unwrap();

        assert_eq!(registry.len(), 1);
        assert!(registry.contains("valid"));
    }

    #[test]
    fn test_registry_load_directory_skips_non_yaml_files() {
        let temp_dir = TempDir::new().unwrap();
        create_test_chimera(temp_dir.path(), "chimera-a");
        std::fs::write(temp_dir.path().join("readme.txt"), "not a chimera").unwrap();
        std::fs::write(temp_dir.path().join("config.json"), "{}").unwrap();

        let registry = ChimeraRegistry::from_directory(temp_dir.path()).unwrap();

        assert_eq!(registry.len(), 1);
        assert!(registry.contains("chimera-a"));
    }

    #[test]
    fn test_registry_load_directory_nested_subdir() {
        let temp_dir = TempDir::new().unwrap();
        let subdir = temp_dir.path().join("subdir");
        std::fs::create_dir_all(&subdir).unwrap();
        create_test_chimera(temp_dir.path(), "root-chimera");
        create_test_chimera(&subdir, "nested-chimera");

        let registry = ChimeraRegistry::from_directory(temp_dir.path()).unwrap();

        assert_eq!(registry.len(), 2);
        assert!(registry.contains("root-chimera"));
        assert!(registry.contains("nested-chimera"));
    }

    #[test]
    fn test_registry_by_primal_multiple_chimeras() {
        let temp_dir = TempDir::new().unwrap();
        create_test_chimera(temp_dir.path(), "chimera-1");
        create_test_chimera(temp_dir.path(), "chimera-2");

        let registry = ChimeraRegistry::from_directory(temp_dir.path()).unwrap();

        let beardog_chimeras = registry.by_primal("beardog");
        assert_eq!(beardog_chimeras.len(), 2);
    }

    #[test]
    fn test_registry_summary_uses_arrays() {
        let temp_dir = TempDir::new().unwrap();
        create_chimera_with_array(temp_dir.path(), "array-chimera");

        let registry = ChimeraRegistry::from_directory(temp_dir.path()).unwrap();
        let summary = registry.summary();

        let s = summary.get("array-chimera").unwrap();
        assert!(s.uses_arrays);
        assert_eq!(s.primals, vec!["beardog"]);
    }

    #[test]
    fn test_registry_by_primal_songbird() {
        let temp_dir = TempDir::new().unwrap();
        create_chimera_with_songbird(temp_dir.path(), "songbird-chimera");

        let registry = ChimeraRegistry::from_directory(temp_dir.path()).unwrap();

        let songbird_chimeras = registry.by_primal("songbird");
        assert_eq!(songbird_chimeras.len(), 1);
        assert_eq!(songbird_chimeras[0].chimera.id, "songbird-chimera");
    }

    #[test]
    fn test_registry_reload_multiple_dirs() {
        let temp_dir1 = TempDir::new().unwrap();
        let temp_dir2 = TempDir::new().unwrap();
        create_test_chimera(temp_dir1.path(), "chimera-1");
        create_test_chimera(temp_dir2.path(), "chimera-2");

        let mut registry = ChimeraRegistry::new();
        registry.load_directory(temp_dir1.path()).unwrap();
        registry.load_directory(temp_dir2.path()).unwrap();
        assert_eq!(registry.len(), 2);

        let count = registry.reload().unwrap();
        assert_eq!(count, 2);
        assert_eq!(registry.len(), 2);
    }

    #[test]
    fn test_registry_load_file_invalid_returns_err() {
        let temp_dir = TempDir::new().unwrap();
        let invalid_path = temp_dir.path().join("bad.yaml");
        std::fs::write(&invalid_path, "invalid: [yaml").unwrap();

        let mut registry = ChimeraRegistry::new();
        let result = registry.load_file(&invalid_path);
        assert!(result.is_err());
    }
}
