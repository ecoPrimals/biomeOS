// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Chimera definition types
//!
//! Defines the structure for chimera YAML definitions.

use std::collections::HashMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::component::Component;
use crate::error::{ChimeraError, ChimeraResult};
use crate::fusion::Fusion;

/// Complete chimera definition loaded from YAML
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChimeraDefinition {
    /// Core chimera metadata
    pub chimera: ChimeraMetadata,

    /// Component definitions (primals to pull from)
    pub components: HashMap<String, Component>,

    /// Fusion configuration (how components are combined)
    pub fusion: Fusion,

    /// Resource requirements
    #[serde(default)]
    pub resources: ResourceSpec,

    /// Deployment hints
    #[serde(default)]
    pub deployment: DeploymentSpec,

    /// Health check configuration
    #[serde(default)]
    pub health: HealthSpec,
}

/// Chimera metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChimeraMetadata {
    /// Unique identifier (e.g., "p2p-secure")
    pub id: String,

    /// Human-readable name
    pub name: String,

    /// Semantic version
    pub version: String,

    /// Detailed description
    pub description: String,
}

/// Resource requirements specification
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResourceSpec {
    /// CPU cores required
    #[serde(default)]
    pub cpu_cores: Option<u32>,

    /// Memory in MB
    #[serde(default)]
    pub memory_mb: Option<u64>,

    /// Storage in MB
    #[serde(default)]
    pub storage_mb: Option<u64>,

    /// GPU requirements
    #[serde(default)]
    pub gpu: Option<GpuSpec>,

    /// Network requirements
    #[serde(default)]
    pub network: Option<NetworkSpec>,

    /// Per-instance resources (for array components)
    #[serde(default)]
    pub per_instance: Option<Box<ResourceSpec>>,
}

/// GPU specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuSpec {
    /// Whether GPU is required
    #[serde(default)]
    pub required: bool,

    /// Whether GPU is optional but beneficial
    #[serde(default)]
    pub optional: bool,

    /// Minimum VRAM in MB
    #[serde(default)]
    pub min_vram_mb: Option<u64>,
}

/// Network specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkSpec {
    /// Ports required
    #[serde(default)]
    pub ports: Vec<u16>,

    /// Latency target in milliseconds
    #[serde(default)]
    pub latency_target_ms: Option<u32>,

    /// Jitter tolerance in milliseconds
    #[serde(default)]
    pub jitter_tolerance_ms: Option<u32>,
}

/// Deployment configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DeploymentSpec {
    /// Whether only one instance can run
    #[serde(default)]
    pub singleton: bool,

    /// Resource requirements for the deployment
    #[serde(default)]
    pub requirements: DeploymentRequirements,

    /// Scaling configuration
    #[serde(default)]
    pub scaling: Option<ScalingSpec>,
}

/// Resource requirements for deployment
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DeploymentRequirements {
    /// Whether network access is required
    #[serde(default)]
    pub network: bool,

    /// Whether GPU is required
    #[serde(default)]
    pub gpu: bool,

    /// Whether the chimera can participate in federation
    #[serde(default)]
    pub federation: bool,
}

/// Scaling specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalingSpec {
    /// Metric to scale on
    pub metric: String,

    /// Scaling thresholds
    #[serde(default)]
    pub thresholds: Vec<ScalingThreshold>,
}

/// Scaling threshold
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalingThreshold {
    /// Threshold value (context-dependent on metric)
    #[serde(flatten)]
    pub values: HashMap<String, serde_json::Value>,
}

/// Health check configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HealthSpec {
    /// Individual health checks
    #[serde(default)]
    pub checks: Vec<HealthCheck>,
}

/// Individual health check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    /// Check name
    pub name: String,

    /// Check type (e.g., "`beardog.key_available`")
    #[serde(rename = "type")]
    pub check_type: String,

    /// Additional check parameters
    #[serde(flatten)]
    pub params: HashMap<String, serde_json::Value>,
}

impl ChimeraDefinition {
    /// Load a chimera definition from a YAML file
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - File cannot be read
    /// - YAML parsing fails
    /// - Validation fails
    pub fn from_file(path: impl AsRef<Path>) -> ChimeraResult<Self> {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path)?;

        let def: Self = serde_yaml::from_str(&content).map_err(|e| {
            ChimeraError::parse_with_source(
                path.file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown"),
                e.to_string(),
                path,
            )
        })?;

        def.validate()?;
        Ok(def)
    }

    /// Load from YAML string
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - YAML parsing fails
    /// - Validation fails
    pub fn from_yaml(yaml: &str) -> ChimeraResult<Self> {
        let def: Self = serde_yaml::from_str(yaml)?;
        def.validate()?;
        Ok(def)
    }

    /// Validate the chimera definition
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - No components are defined
    /// - Fusion bindings reference non-existent components
    /// - Duplicate component names exist
    pub fn validate(&self) -> ChimeraResult<()> {
        // Ensure we have at least one component
        if self.components.is_empty() {
            return Err(ChimeraError::parse(
                &self.chimera.id,
                "Chimera must have at least one component",
            ));
        }

        // Validate fusion references exist in components
        for (name, binding) in &self.fusion.bindings {
            if let Some(ref provider) = binding.provider {
                let primal = provider.split('.').next().unwrap_or("");
                // Handle array notation (e.g., "songbird[]")
                let primal = primal.trim_end_matches("[]");
                if !self.components.contains_key(primal) {
                    return Err(ChimeraError::fusion(
                        &self.chimera.id,
                        name,
                        format!("Provider references unknown primal: {primal}"),
                    ));
                }
            }
        }

        Ok(())
    }

    /// Get list of required primal names
    pub fn required_primals(&self) -> Vec<&str> {
        self.components.keys().map(String::as_str).collect()
    }

    /// Check if this chimera uses arrays (multiple instances of a component)
    #[must_use]
    pub fn uses_arrays(&self) -> bool {
        self.components.values().any(|c| c.array.is_some())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_minimal_chimera() {
        let yaml = r#"
chimera:
  id: test-chimera
  name: Test Chimera
  version: "1.0.0"
  description: A test chimera

components:
  beardog:
    source: primals/beardog
    version: ">=1.0.0"
    modules: []

fusion:
  bindings: {}
  api:
    endpoints: []
"#;
        let def = ChimeraDefinition::from_yaml(yaml).unwrap();
        assert_eq!(def.chimera.id, "test-chimera");
        assert_eq!(def.required_primals(), vec!["beardog"]);
    }
}
