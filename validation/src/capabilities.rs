//! Capability-based primal deployment
//!
//! Deploy primals based on required capabilities, not hardcoded names.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// A capability that can be provided by a primal
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Capability {
    /// P2P coordination and mDNS discovery
    P2PCoordination,
    /// Identity and authentication
    Identity,
    /// Encrypted storage
    Storage,
    /// Compute orchestration
    Compute,
    /// Time-series and lineage tracking
    TemporalTracking,
    /// Encryption services
    Encryption,
    /// State management
    StateManagement,
    /// UI/visualization
    Visualization,
    /// Custom capability
    Custom(String),
}

impl Capability {
    /// Human-readable description
    #[must_use]
    pub fn description(&self) -> &str {
        match self {
            Self::P2PCoordination => "P2P coordination and service discovery",
            Self::Identity => "Identity and authentication",
            Self::Storage => "Encrypted storage and replication",
            Self::Compute => "Distributed compute orchestration",
            Self::TemporalTracking => "Time-series and lineage tracking",
            Self::Encryption => "Encryption and cryptographic services",
            Self::StateManagement => "State management and coordination",
            Self::Visualization => "UI and visualization",
            Self::Custom(name) => name,
        }
    }

    /// Example primals that provide this capability
    /// (For documentation/debugging only - not used for deployment)
    #[must_use]
    pub fn example_providers(&self) -> Vec<&str> {
        match self {
            Self::P2PCoordination => vec!["songbird", "custom-p2p"],
            Self::Identity => vec!["beardog", "custom-identity"],
            Self::Storage => vec!["nestgate", "custom-storage"],
            Self::Compute => vec!["toadstool", "custom-compute"],
            Self::TemporalTracking => vec!["sweetgrass", "custom-temporal"],
            Self::Encryption => vec!["rhizocrypt", "custom-crypto"],
            Self::StateManagement => vec!["loamspine", "custom-state"],
            Self::Visualization => vec!["petaltongue", "custom-ui"],
            Self::Custom(_) => vec!["user-defined"],
        }
    }
}

/// A deployment profile based on required capabilities
#[derive(Debug, Clone)]
pub struct CapabilityProfile {
    pub name: String,
    pub required_capabilities: HashSet<Capability>,
    pub optional_capabilities: HashSet<Capability>,
}

impl CapabilityProfile {
    /// Create a new capability profile
    #[must_use]
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            required_capabilities: HashSet::new(),
            optional_capabilities: HashSet::new(),
        }
    }

    /// Add a required capability
    pub fn require(&mut self, capability: Capability) -> &mut Self {
        self.required_capabilities.insert(capability);
        self
    }

    /// Add an optional capability
    pub fn optional(&mut self, capability: Capability) -> &mut Self {
        self.optional_capabilities.insert(capability);
        self
    }

    /// Minimal federation profile
    #[must_use]
    pub fn minimal_federation() -> Self {
        let mut profile = Self::new("minimal-federation");
        profile.require(Capability::P2PCoordination);
        profile
    }

    /// Full federation profile with identity
    #[must_use]
    pub fn full_federation() -> Self {
        let mut profile = Self::new("full-federation");
        profile
            .require(Capability::P2PCoordination)
            .require(Capability::Identity)
            .optional(Capability::Storage);
        profile
    }

    /// Compute node profile
    #[must_use]
    pub fn compute_node() -> Self {
        let mut profile = Self::new("compute-node");
        profile
            .require(Capability::P2PCoordination)
            .require(Capability::Compute)
            .optional(Capability::Identity);
        profile
    }

    /// Storage node profile
    #[must_use]
    pub fn storage_node() -> Self {
        let mut profile = Self::new("storage-node");
        profile
            .require(Capability::P2PCoordination)
            .require(Capability::Storage)
            .require(Capability::Encryption)
            .optional(Capability::Identity);
        profile
    }

    /// Full ecosystem profile
    #[must_use]
    pub fn full_ecosystem() -> Self {
        let mut profile = Self::new("full-ecosystem");
        profile
            .require(Capability::P2PCoordination)
            .require(Capability::Identity)
            .require(Capability::Storage)
            .require(Capability::Encryption)
            .optional(Capability::Compute)
            .optional(Capability::TemporalTracking)
            .optional(Capability::Visualization);
        profile
    }

    /// List all capabilities
    pub fn all_capabilities(&self) -> Vec<&Capability> {
        let mut caps: Vec<_> = self
            .required_capabilities
            .iter()
            .chain(self.optional_capabilities.iter())
            .collect();
        caps.sort_by_key(|c| format!("{:?}", c));
        caps
    }

    /// Human-readable description
    #[must_use]
    pub fn description(&self) -> String {
        let required: Vec<_> = self.required_capabilities.iter().collect();
        let optional: Vec<_> = self.optional_capabilities.iter().collect();

        let mut desc = format!("{} profile", self.name);
        
        if !required.is_empty() {
            desc.push_str("\n  Required: ");
            desc.push_str(
                &required
                    .iter()
                    .map(|c| format!("{:?}", c))
                    .collect::<Vec<_>>()
                    .join(", "),
            );
        }

        if !optional.is_empty() {
            desc.push_str("\n  Optional: ");
            desc.push_str(
                &optional
                    .iter()
                    .map(|c| format!("{:?}", c))
                    .collect::<Vec<_>>()
                    .join(", "),
            );
        }

        desc
    }
}

/// Primal binary metadata (discovered at runtime)
#[derive(Debug, Clone)]
pub struct PrimalBinary {
    pub path: std::path::PathBuf,
    pub capabilities: HashSet<Capability>,
}

impl PrimalBinary {
    /// Discover primals in a directory
    pub fn discover_in(_dir: &std::path::Path) -> Vec<Self> {
        // Future: Implement filesystem-based capability discovery
        // Currently, primals are discovered at runtime via Songbird
        // This method will be enhanced once the primal CLI supports
        // the `capabilities` subcommand for static discovery
        Vec::new()
    }

    /// Check if this primal provides a capability
    #[must_use]
    pub fn provides(&self, capability: &Capability) -> bool {
        self.capabilities.contains(capability)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capability_description() {
        let cap = Capability::P2PCoordination;
        assert!(!cap.description().is_empty());
    }

    #[test]
    fn test_minimal_federation_profile() {
        let profile = CapabilityProfile::minimal_federation();
        assert_eq!(profile.required_capabilities.len(), 1);
        assert!(profile.required_capabilities.contains(&Capability::P2PCoordination));
    }

    #[test]
    fn test_full_ecosystem_profile() {
        let profile = CapabilityProfile::full_ecosystem();
        assert!(profile.required_capabilities.len() >= 4);
        assert!(profile.required_capabilities.contains(&Capability::P2PCoordination));
        assert!(profile.required_capabilities.contains(&Capability::Identity));
    }

    #[test]
    fn test_profile_description() {
        let profile = CapabilityProfile::minimal_federation();
        let desc = profile.description();
        assert!(desc.contains("minimal-federation"));
        assert!(desc.contains("Required"));
    }
}

