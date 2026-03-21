// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Organism types for niches
//!
//! Organisms are the living components of a niche - either chimeras or standalone primals.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Specification of all organisms in a niche
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OrganismSpec {
    /// Chimera organisms
    #[serde(default)]
    pub chimeras: HashMap<String, ChimeraOrganism>,

    /// Standalone primal organisms
    #[serde(default)]
    pub primals: HashMap<String, PrimalOrganism>,
}

/// An organism in a niche
#[derive(Debug, Clone)]
pub enum Organism {
    /// A chimera organism
    Chimera(ChimeraOrganism),
    /// A standalone primal organism
    Primal(PrimalOrganism),
}

/// Type of organism
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrganismType {
    /// A chimera (amalgam of primals)
    Chimera,
    /// A standalone primal
    Primal,
}

/// A chimera organism in a niche
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChimeraOrganism {
    /// Chimera type/ID (references chimera definition)
    #[serde(rename = "type")]
    pub chimera_type: String,

    /// Whether this chimera is required
    #[serde(default)]
    pub required: bool,

    /// Chimera-specific configuration
    #[serde(default)]
    pub config: HashMap<String, serde_json::Value>,

    /// Fusion overrides for this instance
    #[serde(default)]
    pub fusion_overrides: HashMap<String, serde_json::Value>,
}

/// A standalone primal organism in a niche
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalOrganism {
    /// Primal type (beardog, songbird, etc.)
    #[serde(rename = "type")]
    pub primal_type: String,

    /// Whether this primal is required
    #[serde(default)]
    pub required: bool,

    /// Role within the niche
    #[serde(default)]
    pub role: String,

    /// Primal-specific configuration
    #[serde(default)]
    pub config: HashMap<String, serde_json::Value>,

    /// Dependencies on other organisms
    #[serde(default)]
    pub dependencies: Vec<String>,
}

impl OrganismSpec {
    /// Create an empty organism spec
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a chimera
    #[must_use]
    pub fn with_chimera(mut self, name: impl Into<String>, chimera: ChimeraOrganism) -> Self {
        self.chimeras.insert(name.into(), chimera);
        self
    }

    /// Add a primal
    #[must_use]
    pub fn with_primal(mut self, name: impl Into<String>, primal: PrimalOrganism) -> Self {
        self.primals.insert(name.into(), primal);
        self
    }

    /// Get all organism names
    pub fn all_names(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self.chimeras.keys().map(String::as_str).collect();
        names.extend(self.primals.keys().map(String::as_str));
        names
    }

    /// Get an organism by name
    #[must_use]
    pub fn get(&self, name: &str) -> Option<Organism> {
        if let Some(chimera) = self.chimeras.get(name) {
            return Some(Organism::Chimera(chimera.clone()));
        }
        if let Some(primal) = self.primals.get(name) {
            return Some(Organism::Primal(primal.clone()));
        }
        None
    }

    /// Check if an organism exists
    #[must_use]
    pub fn contains(&self, name: &str) -> bool {
        self.chimeras.contains_key(name) || self.primals.contains_key(name)
    }

    /// Total organism count
    #[must_use]
    pub fn len(&self) -> usize {
        self.chimeras.len() + self.primals.len()
    }

    /// Check if empty
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.chimeras.is_empty() && self.primals.is_empty()
    }

    /// Get required organisms
    #[must_use]
    pub fn required(&self) -> Vec<(&str, OrganismType)> {
        let mut required = Vec::new();

        for (name, chimera) in &self.chimeras {
            if chimera.required {
                required.push((name.as_str(), OrganismType::Chimera));
            }
        }

        for (name, primal) in &self.primals {
            if primal.required {
                required.push((name.as_str(), OrganismType::Primal));
            }
        }

        required
    }
}

impl ChimeraOrganism {
    /// Create a new chimera organism
    pub fn new(chimera_type: impl Into<String>) -> Self {
        Self {
            chimera_type: chimera_type.into(),
            required: false,
            config: HashMap::new(),
            fusion_overrides: HashMap::new(),
        }
    }

    /// Set as required
    #[must_use]
    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    /// Add configuration
    #[must_use]
    pub fn with_config(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.config.insert(key.into(), value);
        self
    }
}

impl PrimalOrganism {
    /// Create a new primal organism
    pub fn new(primal_type: impl Into<String>) -> Self {
        Self {
            primal_type: primal_type.into(),
            required: false,
            role: String::new(),
            config: HashMap::new(),
            dependencies: Vec::new(),
        }
    }

    /// Set as required
    #[must_use]
    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    /// Set the role
    #[must_use]
    pub fn with_role(mut self, role: impl Into<String>) -> Self {
        self.role = role.into();
        self
    }

    /// Add dependencies
    #[must_use]
    pub fn with_dependencies(mut self, deps: Vec<String>) -> Self {
        self.dependencies = deps;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_organism_spec() {
        let spec = OrganismSpec::new()
            .with_chimera("mesh", ChimeraOrganism::new("p2p-secure").required())
            .with_primal(
                "storage",
                PrimalOrganism::new("nestgate").with_role("replays"),
            );

        assert_eq!(spec.len(), 2);
        assert!(spec.contains("mesh"));
        assert!(spec.contains("storage"));

        let required = spec.required();
        assert_eq!(required.len(), 1);
        assert_eq!(required[0].0, "mesh");
    }
}
