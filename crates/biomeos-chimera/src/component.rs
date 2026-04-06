// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Component definitions for chimeras
//!
//! Components represent the primal parts that are pulled into a chimera.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// A component pulled from a primal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Component {
    /// Source path for the primal (e.g., "primals/beardog")
    pub source: String,

    /// Version constraint (e.g., ">=2.0.0")
    pub version: String,

    /// Modules to include from this primal
    #[serde(default)]
    pub modules: Vec<ComponentModule>,

    /// Array configuration (for multiple instances)
    #[serde(default)]
    pub array: Option<ArraySpec>,
}

/// A module from a primal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentModule {
    /// Module name (e.g., "btsp", "`genetic_crypto`")
    pub name: String,

    /// Human-readable description
    #[serde(default)]
    pub description: String,

    /// Features to enable from this module
    #[serde(default)]
    pub features: Vec<String>,

    /// Module-specific configuration
    #[serde(default)]
    pub config: HashMap<String, serde_json::Value>,
}

/// Array specification for components that can have multiple instances
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArraySpec {
    /// Whether array mode is enabled
    #[serde(default)]
    pub enabled: bool,

    /// Minimum instances
    #[serde(default = "default_min_instances")]
    pub min: u32,

    /// Maximum instances
    #[serde(default = "default_max_instances")]
    pub max: u32,
}

const fn default_min_instances() -> u32 {
    1
}

const fn default_max_instances() -> u32 {
    16
}

/// Source specification for a primal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalSource {
    /// Local path to primal
    pub path: Option<String>,

    /// Git repository URL
    pub git: Option<String>,

    /// Git branch or tag
    pub branch: Option<String>,

    /// Registry name (for future primal registry)
    pub registry: Option<String>,
}

impl Component {
    /// Create a new component with basic info
    pub fn new(source: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            source: source.into(),
            version: version.into(),
            modules: Vec::new(),
            array: None,
        }
    }

    /// Add a module to this component
    #[must_use]
    pub fn with_module(mut self, module: ComponentModule) -> Self {
        self.modules.push(module);
        self
    }

    /// Enable array mode
    #[must_use]
    pub const fn with_array(mut self, min: u32, max: u32) -> Self {
        self.array = Some(ArraySpec {
            enabled: true,
            min,
            max,
        });
        self
    }

    /// Check if this component is in array mode
    #[must_use]
    pub fn is_array(&self) -> bool {
        self.array.as_ref().is_some_and(|a| a.enabled)
    }

    /// Get the module names
    #[must_use]
    pub fn module_names(&self) -> Vec<&str> {
        self.modules.iter().map(|m| m.name.as_str()).collect()
    }

    /// Get a specific module by name
    #[must_use]
    pub fn get_module(&self, name: &str) -> Option<&ComponentModule> {
        self.modules.iter().find(|m| m.name == name)
    }
}

impl ComponentModule {
    /// Create a new module
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: String::new(),
            features: Vec::new(),
            config: HashMap::new(),
        }
    }

    /// Add a description
    #[must_use]
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    /// Add features
    #[must_use]
    pub fn with_features(mut self, features: Vec<String>) -> Self {
        self.features = features;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_component_builder() {
        let component = Component::new("primals/beardog", ">=2.0.0")
            .with_module(
                ComponentModule::new("btsp")
                    .with_description("BTSP tunnel")
                    .with_features(vec!["tunnel".into(), "encryption".into()]),
            )
            .with_array(1, 64);

        assert!(component.is_array());
        assert_eq!(component.module_names(), vec!["btsp"]);
    }
}
