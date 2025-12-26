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
}
