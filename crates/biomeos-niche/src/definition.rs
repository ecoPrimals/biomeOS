//! Niche definition types

use std::collections::HashMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::organism::OrganismSpec;
use crate::interaction::Interaction;
use crate::error::{NicheError, NicheResult};

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
    pub fn from_file(path: impl AsRef<Path>) -> NicheResult<Self> {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path)?;
        Self::from_yaml(&content)
    }

    /// Load from YAML string
    pub fn from_yaml(yaml: &str) -> NicheResult<Self> {
        let def: Self = serde_yaml::from_str(yaml)?;
        def.validate()?;
        Ok(def)
    }

    /// Validate the niche definition
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
            let from_name = interaction.from.split('.').last().unwrap_or(&interaction.from);
            let to_name = interaction.to.split('.').last().unwrap_or(&interaction.to);

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
    pub fn apply_customization(&mut self, values: &HashMap<String, serde_json::Value>) -> NicheResult<()> {
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
}

