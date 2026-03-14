// SPDX-License-Identifier: AGPL-3.0-only
//
// Copyright 2025 ecoPrimals Project
// Licensed under the Affero General Public License v3.0.
// See LICENSE file in the project root or visit https://www.gnu.org/licenses/agpl-3.0.html

//! Shared types for spore operations

use crate::spore_types::SporeType;
use serde::{Deserialize, Serialize};

/// Configuration for spore creation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SporeConfig {
    /// Human-readable label for this spore
    pub label: String,

    /// Node ID for this tower (e.g., "tower1")
    pub node_id: String,

    /// Family ID for this ecosystem (derived from `.family.seed`, e.g., "cf7e8729dc4ff05f")
    #[serde(default = "default_family_id")]
    pub family_id: String,

    /// Type of spore (Cold = storage, Live = deployable)
    #[serde(default)]
    pub spore_type: SporeType,
}

/// Returns the default family ID from `FAMILY_ID` env var or `"default"`
pub fn default_family_id() -> String {
    std::env::var("FAMILY_ID").unwrap_or_else(|_| "default".to_string())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_spore_config_serde_roundtrip() {
        let config = SporeConfig {
            label: "test-spore".to_string(),
            node_id: "tower1".to_string(),
            family_id: "cf7e8729dc4ff05f".to_string(),
            spore_type: SporeType::Live,
        };
        let json = serde_json::to_string(&config).unwrap();
        let restored: SporeConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config.label, restored.label);
        assert_eq!(config.node_id, restored.node_id);
        assert_eq!(config.family_id, restored.family_id);
        assert_eq!(config.spore_type, restored.spore_type);
    }

    #[test]
    #[ignore = "Env var test - run with --test-threads=1"]
    fn test_spore_config_default_family_id() {
        std::env::remove_var("FAMILY_ID");
        assert_eq!(default_family_id(), "default");

        std::env::set_var("FAMILY_ID", "custom_family");
        assert_eq!(default_family_id(), "custom_family");
        std::env::remove_var("FAMILY_ID");
    }

    #[test]
    fn test_spore_config_deserialize_with_defaults() {
        let json = r#"{"label":"x","node_id":"n1","family_id":"f1"}"#;
        let config: SporeConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.label, "x");
        assert_eq!(config.node_id, "n1");
        assert_eq!(config.family_id, "f1");
        assert_eq!(config.spore_type, SporeType::Live);
    }
}
