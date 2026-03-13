// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Niche definition types

use std::collections::HashMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::{NicheError, NicheResult};
use crate::interaction::Interaction;
use crate::organism::OrganismSpec;

/// Complete niche (biome) definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NicheDefinition {
    /// Core niche metadata
    pub niche: NicheMetadata,

    /// Organisms in this niche (chimeras + primals)
    pub organisms: OrganismSpec,

    /// Interactions between organisms
    #[serde(default)]
    pub interactions: Vec<Interaction>,

    /// Customization options for BYOB
    #[serde(default)]
    pub customization: Vec<NicheCustomization>,

    /// Resource configuration
    #[serde(default)]
    pub resources: NicheResources,

    /// Networking configuration
    #[serde(default)]
    pub networking: NicheNetworking,

    /// Security configuration
    #[serde(default)]
    pub security: NicheSecurity,
}

/// Niche metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NicheMetadata {
    /// Unique identifier
    pub id: String,

    /// Human-readable name
    pub name: String,

    /// Version
    pub version: String,

    /// Description
    pub description: String,

    /// Category (gaming, research, development, etc.)
    #[serde(default)]
    pub category: String,

    /// Difficulty level
    #[serde(default)]
    pub difficulty: String,

    /// Author
    #[serde(default)]
    pub author: String,

    /// Feature list
    #[serde(default)]
    pub features: Vec<String>,
}

/// Customization option for BYOB
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NicheCustomization {
    /// Option ID
    pub id: String,

    /// Display name
    pub name: String,

    /// Description
    #[serde(default)]
    pub description: String,

    /// Option type (select, number, boolean, text)
    #[serde(rename = "type")]
    pub option_type: String,

    /// Available options (for select type)
    #[serde(default)]
    pub options: Vec<String>,

    /// Default value
    #[serde(default)]
    pub default: serde_json::Value,

    /// Whether this option is required
    #[serde(default)]
    pub required: bool,

    /// Min value (for number type)
    #[serde(default)]
    pub min: Option<i64>,

    /// Max value (for number type)
    #[serde(default)]
    pub max: Option<i64>,
}

/// Niche resource configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NicheResources {
    /// Base resource requirements
    #[serde(default)]
    pub base: ResourceLimits,

    /// Per-user/connection resources
    #[serde(default)]
    pub per_user: Option<ResourceLimits>,

    /// GPU requirements
    #[serde(default)]
    pub gpu: Option<GpuRequirements>,
}

/// Resource limits
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// CPU cores
    #[serde(default)]
    pub cpu_cores: Option<u32>,

    /// Memory in MB or GB (with suffix)
    #[serde(default)]
    pub memory_gb: Option<u32>,

    /// Storage in GB
    #[serde(default)]
    pub storage_gb: Option<u32>,
}

/// GPU requirements
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GpuRequirements {
    /// Whether GPU is required
    #[serde(default)]
    pub required: bool,

    /// Minimum VRAM in GB
    #[serde(default)]
    pub min_vram_gb: Option<u32>,
}

/// Niche networking configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NicheNetworking {
    /// Network mode (mesh, bridge, host)
    #[serde(default)]
    pub mode: String,

    /// Port mappings
    #[serde(default)]
    pub ports: HashMap<String, u16>,

    /// Network requirements
    #[serde(default)]
    pub requirements: NetworkRequirements,
}

/// Network requirements
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NetworkRequirements {
    /// Target latency in ms
    #[serde(default)]
    pub latency_target_ms: Option<u32>,

    /// Jitter tolerance in ms
    #[serde(default)]
    pub jitter_tolerance_ms: Option<u32>,

    /// Packet loss tolerance (0.0-1.0)
    #[serde(default)]
    pub packet_loss_tolerance_percent: Option<f32>,
}

/// Niche security configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NicheSecurity {
    /// Encryption method
    #[serde(default)]
    pub encryption: String,

    /// Authentication method
    #[serde(default)]
    pub authentication: String,

    /// Trust model
    #[serde(default)]
    pub trust_model: String,

    /// Whether audit logging is enabled
    #[serde(default)]
    pub audit_logging: bool,
}

impl NicheDefinition {
    /// Load from YAML file
    ///
    /// # Errors
    /// Returns an error if the file cannot be read or parsed.
    pub fn from_file(path: impl AsRef<Path>) -> NicheResult<Self> {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path)?;
        Self::from_yaml(&content)
    }

    /// Load from YAML string
    ///
    /// # Errors
    /// Returns an error if the YAML cannot be parsed or validation fails.
    pub fn from_yaml(yaml: &str) -> NicheResult<Self> {
        let def: Self = serde_yaml::from_str(yaml)?;
        def.validate()?;
        Ok(def)
    }

    /// Validate the niche definition
    ///
    /// # Errors
    /// Returns an error if validation fails.
    pub fn validate(&self) -> NicheResult<()> {
        // Check we have at least one organism
        if self.organisms.chimeras.is_empty() && self.organisms.primals.is_empty() {
            return Err(NicheError::parse(
                &self.niche.id,
                "Niche must have at least one organism (chimera or primal)",
            ));
        }

        // Validate interactions reference existing organisms
        for interaction in &self.interactions {
            // Extract organism name from "category.name" format
            let from_name = interaction
                .from
                .split('.')
                .next_back()
                .unwrap_or(&interaction.from);
            let to_name = interaction
                .to
                .split('.')
                .next_back()
                .unwrap_or(&interaction.to);

            let organism_exists = |name: &str| {
                self.organisms.chimeras.contains_key(name)
                    || self.organisms.primals.contains_key(name)
            };

            if !organism_exists(from_name) {
                return Err(NicheError::InvalidInteraction {
                    niche: self.niche.id.clone(),
                    message: format!("Unknown organism in 'from': {}", interaction.from),
                });
            }

            if !organism_exists(to_name) {
                return Err(NicheError::InvalidInteraction {
                    niche: self.niche.id.clone(),
                    message: format!("Unknown organism in 'to': {}", interaction.to),
                });
            }
        }

        Ok(())
    }

    /// Apply customization values
    ///
    /// # Errors
    /// Returns an error if required customization values are missing.
    pub fn apply_customization(
        &mut self,
        values: &HashMap<String, serde_json::Value>,
    ) -> NicheResult<()> {
        // This would apply template substitution
        // For now, just validate the values exist
        for custom in &self.customization {
            if custom.required && !values.contains_key(&custom.id) {
                return Err(NicheError::parse(
                    &self.niche.id,
                    format!("Required customization '{}' not provided", custom.id),
                ));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_minimal_niche() {
        let yaml = r#"
niche:
  id: test-niche
  name: Test Niche
  version: "1.0.0"
  description: A test niche

organisms:
  chimeras:
    secure_mesh:
      type: p2p-secure
      required: true
  primals: {}

interactions: []
"#;
        let def = NicheDefinition::from_yaml(yaml).unwrap();
        assert_eq!(def.niche.id, "test-niche");
    }

    #[test]
    fn test_validate_empty_organisms_fails() {
        let yaml = r#"
niche:
  id: empty-niche
  name: Empty
  version: "1.0.0"
  description: No organisms

organisms:
  chimeras: {}
  primals: {}

interactions: []
"#;
        let def: NicheDefinition = serde_yaml::from_str(yaml).unwrap();
        let result = def.validate();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("at least one organism"));
    }

    #[test]
    fn test_validate_invalid_interaction_from() {
        let yaml = r#"
niche:
  id: bad-niche
  name: Bad
  version: "1.0.0"
  description: Bad interaction

organisms:
  chimeras:
    mesh:
      type: p2p-secure
  primals: {}

interactions:
  - from: nonexistent.foo
    to: mesh.bar
    type: stream
"#;
        let def: NicheDefinition = serde_yaml::from_str(yaml).unwrap();
        let result = def.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unknown organism"));
    }

    #[test]
    fn test_apply_customization_missing_required() {
        let yaml = r#"
niche:
  id: custom-niche
  name: Custom
  version: "1.0.0"
  description: Has required customization

organisms:
  chimeras:
    mesh:
      type: p2p-secure
  primals: {}

customization:
  - id: required_option
    name: Required
    type: select
    required: true
"#;
        let mut def: NicheDefinition = serde_yaml::from_str(yaml).unwrap();
        let values = HashMap::new();
        let result = def.apply_customization(&values);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("required_option"));
    }

    #[test]
    fn test_apply_customization_with_values() {
        let yaml = r#"
niche:
  id: custom-niche
  name: Custom
  version: "1.0.0"
  description: Has required customization

organisms:
  chimeras:
    mesh:
      type: p2p-secure
  primals: {}

customization:
  - id: required_option
    name: Required
    type: select
    required: true
"#;
        let mut def: NicheDefinition = serde_yaml::from_str(yaml).unwrap();
        let mut values = HashMap::new();
        values.insert("required_option".to_string(), serde_json::json!("value"));
        let result = def.apply_customization(&values);
        assert!(result.is_ok());
    }

    #[test]
    fn test_niche_resources_default() {
        let resources = NicheResources::default();
        assert!(resources.base.cpu_cores.is_none());
        assert!(resources.base.memory_gb.is_none());
        assert!(resources.gpu.is_none());
    }

    #[test]
    fn test_niche_customization_serialization() {
        let custom = NicheCustomization {
            id: "custom_id".to_string(),
            name: "Custom Name".to_string(),
            description: "Custom desc".to_string(),
            option_type: "select".to_string(),
            options: vec!["a".to_string(), "b".to_string()],
            default: serde_json::json!("a"),
            required: true,
            min: Some(0),
            max: Some(100),
        };
        let json = serde_json::to_string(&custom).expect("serialize");
        let restored: NicheCustomization = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(custom.id, restored.id);
        assert_eq!(custom.required, restored.required);
    }

    #[test]
    fn test_network_requirements_default() {
        let req = NetworkRequirements::default();
        assert!(req.latency_target_ms.is_none());
        assert!(req.jitter_tolerance_ms.is_none());
    }
}
