// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

// =============================================================================
// Niche Manifest Parser - TOML-based with Graph Support
// =============================================================================
//
// Modern idiomatic Rust TOML parser:
// - Backward compatible (graphs are optional!)
// - Capability-based (no hardcoding)
// - Clear error messages
// - Graph reference validation
//
// =============================================================================

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Result type for niche operations
pub type Result<T> = std::result::Result<T, NicheError>;

/// Niche-related errors
#[derive(Debug, thiserror::Error)]
pub enum NicheError {
    /// I/O error reading niche file
    #[error("Failed to read niche file: {0}")]
    IoError(#[from] std::io::Error),

    /// TOML parsing error
    #[error("Failed to parse TOML: {0}")]
    ParseError(#[from] toml::de::Error),

    /// Manifest validation error
    #[error("Validation error: {0}")]
    ValidationError(String),

    /// Referenced graph file not found
    #[error("Graph file not found: {0}")]
    GraphNotFound(String),
}

/// A niche manifest (TOML-based)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NicheManifest {
    /// Niche metadata
    pub niche: NicheMetadata,

    /// Primals to deploy
    pub primals: Vec<PrimalSpec>,

    /// Graph definitions (NEURAL API!)
    #[serde(default)]
    pub graphs: Vec<GraphRef>,

    /// Additional configuration sections (flexible)
    #[serde(flatten)]
    pub config: HashMap<String, toml::Value>,
}

/// Niche metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NicheMetadata {
    /// Niche name (unique identifier)
    pub name: String,
    /// Niche version (semver)
    pub version: String,

    /// Niche type (e.g. "development", "production")
    #[serde(rename = "type")]
    pub niche_type: String,

    /// Human-readable description
    pub description: String,
    /// Target architecture (e.g. "x86_64", "aarch64")
    pub architecture: String,

    /// Optional path to the family seed file for identity derivation
    #[serde(default)]
    pub family_seed_file: Option<String>,
}

/// Primal specification in a niche
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalSpec {
    /// Binary path (relative to niche file or absolute)
    pub binary: String,

    /// Capabilities this primal provides
    pub provides: Vec<String>,

    /// Capabilities this primal requires
    #[serde(default)]
    pub requires: Vec<String>,

    /// Whether this primal is optional
    #[serde(default)]
    pub optional: bool,

    /// Environment variables
    #[serde(default)]
    pub env: HashMap<String, String>,
}

/// Graph reference in a niche (NEURAL API!)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphRef {
    /// Graph name
    pub name: String,

    /// Path to graph TOML file (relative to niche or absolute)
    pub path: String,

    /// Description of what this graph does
    #[serde(default)]
    pub description: Option<String>,

    /// Whether this is the default graph for deployment
    #[serde(default)]
    pub default: bool,
}

impl NicheManifest {
    /// Parse a niche manifest from a TOML file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(&path)?;
        let mut manifest: NicheManifest = toml::from_str(&content)?;

        // Resolve relative paths
        if let Some(parent) = path.as_ref().parent() {
            manifest.resolve_paths(parent);
        }

        // Validate
        manifest.validate()?;

        Ok(manifest)
    }

    /// Parse from TOML string
    pub fn from_toml(content: &str) -> Result<Self> {
        let manifest: NicheManifest = toml::from_str(content)?;
        manifest.validate_structure()?;
        Ok(manifest)
    }

    /// Parse from TOML string without graph file validation (for tests)
    #[cfg(test)]
    pub fn from_toml_unchecked(content: &str) -> Result<Self> {
        let manifest: NicheManifest = toml::from_str(content)?;
        manifest.validate_structure()?;
        Ok(manifest)
    }

    /// Resolve relative paths to absolute
    fn resolve_paths(&mut self, base_path: &Path) {
        // Resolve primal binary paths
        for primal in &mut self.primals {
            #[allow(clippy::collapsible_if)]
            if primal.binary.starts_with("./") || primal.binary.starts_with("../") {
                if let Ok(absolute) = base_path.join(&primal.binary).canonicalize() {
                    primal.binary = absolute.to_string_lossy().to_string();
                }
            }
        }

        // Resolve graph paths
        for graph in &mut self.graphs {
            #[allow(clippy::collapsible_if)]
            if graph.path.starts_with("./") || graph.path.starts_with("../") {
                if let Ok(absolute) = base_path.join(&graph.path).canonicalize() {
                    graph.path = absolute.to_string_lossy().to_string();
                }
            }
        }

        // Resolve family seed file
        #[allow(clippy::collapsible_if)]
        if let Some(ref seed_file) = self.niche.family_seed_file {
            if seed_file.starts_with("./") || seed_file.starts_with("../") {
                if let Ok(absolute) = base_path.join(seed_file).canonicalize() {
                    self.niche.family_seed_file = Some(absolute.to_string_lossy().to_string());
                }
            }
        }
    }

    /// Validate the manifest (including file existence checks)
    pub fn validate(&self) -> Result<()> {
        self.validate_structure()?;
        self.validate_graph_refs()?;
        Ok(())
    }

    /// Validate manifest structure (without file existence checks)
    pub fn validate_structure(&self) -> Result<()> {
        // Check niche metadata
        if self.niche.name.is_empty() {
            return Err(NicheError::ValidationError(
                "Niche name cannot be empty".to_string(),
            ));
        }

        if self.primals.is_empty() {
            return Err(NicheError::ValidationError(
                "Niche must have at least one primal".to_string(),
            ));
        }

        // Validate primal dependencies (all requires must be satisfied)
        self.validate_primal_dependencies()?;

        // Check for duplicate graph names
        let mut seen_names = std::collections::HashSet::new();
        for graph in &self.graphs {
            if !seen_names.insert(&graph.name) {
                return Err(NicheError::ValidationError(format!(
                    "Duplicate graph name: {}",
                    graph.name
                )));
            }
        }

        // Warn if multiple default graphs
        let default_count = self.graphs.iter().filter(|g| g.default).count();
        if default_count > 1 {
            return Err(NicheError::ValidationError(format!(
                "Multiple default graphs ({default_count}), only one allowed"
            )));
        }

        Ok(())
    }

    /// Validate that all primal dependencies are satisfied
    fn validate_primal_dependencies(&self) -> Result<()> {
        // Collect all capabilities provided
        let mut provided: std::collections::HashSet<String> = std::collections::HashSet::new();

        for primal in &self.primals {
            if !primal.optional {
                provided.extend(primal.provides.clone());
            }
        }

        // Check that all required capabilities are provided
        for primal in &self.primals {
            if primal.optional {
                continue; // Optional primals don't block deployment
            }

            for required_cap in &primal.requires {
                if !provided.contains(required_cap) {
                    return Err(NicheError::ValidationError(format!(
                        "Primal '{}' requires capability '{}' but no primal provides it",
                        primal.binary, required_cap
                    )));
                }
            }
        }

        Ok(())
    }

    /// Validate that graph files exist
    fn validate_graph_refs(&self) -> Result<()> {
        for graph in &self.graphs {
            let path = Path::new(&graph.path);
            if !path.exists() {
                return Err(NicheError::GraphNotFound(graph.path.clone()));
            }
        }
        Ok(())
    }

    /// Get the default graph (for `biomeos deploy --niche <name>`)
    pub fn get_default_graph(&self) -> Option<&GraphRef> {
        self.graphs.iter().find(|g| g.default)
    }

    /// Get a graph by name
    pub fn get_graph(&self, name: &str) -> Option<&GraphRef> {
        self.graphs.iter().find(|g| g.name == name)
    }

    /// Get all capabilities provided by this niche
    pub fn get_all_capabilities(&self) -> Vec<String> {
        let mut caps: Vec<String> = self
            .primals
            .iter()
            .filter(|p| !p.optional)
            .flat_map(|p| p.provides.clone())
            .collect();

        caps.sort();
        caps.dedup();
        caps
    }

    /// Check if this niche provides a specific capability
    pub fn provides_capability(&self, capability: &str) -> bool {
        self.primals
            .iter()
            .filter(|p| !p.optional)
            .any(|p| p.provides.contains(&capability.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_minimal_niche() {
        let toml = r#"
[niche]
name = "test-niche"
version = "1.0.0"
type = "test"
description = "Test niche"
architecture = "test"

[[primals]]
binary = "./test"
provides = ["test"]
"#;

        let manifest = NicheManifest::from_toml(toml).unwrap();
        assert_eq!(manifest.niche.name, "test-niche");
        assert_eq!(manifest.primals.len(), 1);
        assert_eq!(manifest.graphs.len(), 0); // No graphs is OK!
    }

    #[test]
    fn test_parse_niche_with_graphs() {
        let toml = r#"
[niche]
name = "test-niche"
version = "1.0.0"
type = "test"
description = "Test niche"
architecture = "test"

[[primals]]
binary = "./test"
provides = ["test"]

[[graphs]]
name = "deploy"
path = "./graphs/deploy.toml"
default = true
"#;

        let manifest = NicheManifest::from_toml(toml).unwrap();
        assert_eq!(manifest.graphs.len(), 1);
        assert!(manifest.get_default_graph().is_some());
    }

    #[test]
    fn test_validate_primal_dependencies() {
        let toml = r#"
[niche]
name = "test-niche"
version = "1.0.0"
type = "test"
description = "Test niche"
architecture = "test"

[[primals]]
binary = "./primal1"
provides = ["security"]

[[primals]]
binary = "./primal2"
provides = ["discovery"]
requires = ["security"]
"#;

        let manifest = NicheManifest::from_toml(toml).unwrap();
        // Should pass validation (security is provided)
        assert!(manifest.validate().is_ok());
    }

    #[test]
    fn test_missing_dependency_fails() {
        let toml = r#"
[niche]
name = "test-niche"
version = "1.0.0"
type = "test"
description = "Test niche"
architecture = "test"

[[primals]]
binary = "./primal1"
provides = ["discovery"]
requires = ["security"]
"#;

        let manifest = NicheManifest::from_toml(toml);
        // Should fail validation (security not provided)
        assert!(manifest.is_err());
    }

    #[test]
    fn test_optional_primal_no_validation() {
        let toml = r#"
[niche]
name = "test-niche"
version = "1.0.0"
type = "test"
description = "Test niche"
architecture = "test"

[[primals]]
binary = "./primal1"
provides = ["discovery"]

[[primals]]
binary = "./primal2"
provides = ["compute"]
requires = ["missing-capability"]
optional = true
"#;

        let manifest = NicheManifest::from_toml(toml).unwrap();
        // Should pass (optional primal doesn't block)
        assert!(manifest.validate_structure().is_ok());
    }

    #[test]
    fn test_duplicate_graph_names_fails() {
        let toml = r#"
[niche]
name = "test-niche"
version = "1.0.0"
type = "test"
description = "Test niche"
architecture = "test"

[[primals]]
binary = "./test"
provides = ["test"]

[[graphs]]
name = "deploy"
path = "./graph1.toml"

[[graphs]]
name = "deploy"
path = "./graph2.toml"
"#;

        let manifest = NicheManifest::from_toml(toml);
        // Should fail (duplicate graph names)
        assert!(manifest.is_err());
    }
}
