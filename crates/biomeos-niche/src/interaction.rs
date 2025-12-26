//! Interaction definitions for niches
//!
//! Defines how organisms communicate within a niche.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// An interaction between organisms in a niche
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Interaction {
    /// Source organism (format: "category.name" or just "name")
    pub from: String,

    /// Target organism
    pub to: String,

    /// Interaction type
    #[serde(rename = "type")]
    pub interaction_type: String,

    /// Interaction-specific configuration
    #[serde(default)]
    pub config: HashMap<String, serde_json::Value>,
}

impl Interaction {
    /// Create a new interaction
    pub fn new(
        from: impl Into<String>,
        to: impl Into<String>,
        interaction_type: impl Into<String>,
    ) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
            interaction_type: interaction_type.into(),
            config: HashMap::new(),
        }
    }

    /// Add configuration
    #[must_use]
    pub fn with_config(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.config.insert(key.into(), value);
        self
    }

    /// Get the source organism name (without category prefix)
    #[must_use]
    pub fn from_name(&self) -> &str {
        self.from.split('.').next_back().unwrap_or(&self.from)
    }

    /// Get the target organism name (without category prefix)
    #[must_use]
    pub fn to_name(&self) -> &str {
        self.to.split('.').next_back().unwrap_or(&self.to)
    }

    /// Get the source category if specified
    #[must_use]
    pub fn from_category(&self) -> Option<&str> {
        let parts: Vec<&str> = self.from.split('.').collect();
        if parts.len() > 1 {
            Some(parts[0])
        } else {
            None
        }
    }

    /// Get the target category if specified
    #[must_use]
    pub fn to_category(&self) -> Option<&str> {
        let parts: Vec<&str> = self.to.split('.').collect();
        if parts.len() > 1 {
            Some(parts[0])
        } else {
            None
        }
    }
}

/// Common interaction types
pub mod types {
    /// Data streaming between organisms
    pub const STREAM: &str = "stream";

    /// Request/response pattern
    pub const REQUEST: &str = "request";

    /// Event notification
    pub const EVENT: &str = "event";

    /// Encryption layer
    pub const ENCRYPTION: &str = "encryption_layer";

    /// Identity binding
    pub const IDENTITY: &str = "identity_binding";

    /// State synchronization
    pub const SYNC: &str = "state_sync";

    /// Action verification
    pub const VERIFY: &str = "action_verification";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interaction() {
        let interaction =
            Interaction::new("chimeras.gaming_mesh", "primals.anti_cheat", types::VERIFY)
                .with_config("verify_before_apply", serde_json::Value::Bool(true));

        assert_eq!(interaction.from_name(), "gaming_mesh");
        assert_eq!(interaction.to_name(), "anti_cheat");
        assert_eq!(interaction.from_category(), Some("chimeras"));
        assert_eq!(interaction.to_category(), Some("primals"));
    }
}
