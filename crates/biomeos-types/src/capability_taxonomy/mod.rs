// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Primal Capability Taxonomy - Deep Debt Evolution
//!
//! Defines well-known capabilities that primals can provide.
//! Enables capability-based discovery instead of hardcoded primal names.

mod category;
mod definition;
mod helpers;

pub use category::CapabilityCategory;
pub use definition::CapabilityTaxonomy;
pub use helpers::capabilities_for_primal;

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test assertions use unwrap for clarity")]
mod tests {
    use super::*;

    #[test]
    fn test_capability_serde_roundtrip() {
        let cap = CapabilityTaxonomy::Encryption;
        let json = serde_json::to_string(&cap).unwrap();
        let back: CapabilityTaxonomy = serde_json::from_str(&json).unwrap();
        assert_eq!(cap, back);
    }

    #[test]
    fn test_capability_custom_serde() {
        let cap = CapabilityTaxonomy::Custom("my-custom-cap".to_string());
        let json = serde_json::to_string(&cap).unwrap();
        let back: CapabilityTaxonomy = serde_json::from_str(&json).unwrap();
        assert_eq!(cap, back);
    }

    #[test]
    fn test_capability_category_serde() {
        for cat in [
            CapabilityCategory::Security,
            CapabilityCategory::Communication,
            CapabilityCategory::Compute,
            CapabilityCategory::Storage,
            CapabilityCategory::UserInterface,
            CapabilityCategory::Orchestration,
            CapabilityCategory::AI,
            CapabilityCategory::Specialized,
        ] {
            let json = serde_json::to_string(&cat).unwrap();
            let back: CapabilityCategory = serde_json::from_str(&json).unwrap();
            assert_eq!(cat, back);
        }
    }

    #[test]
    fn test_capability_description() {
        assert_eq!(
            CapabilityTaxonomy::Encryption.description(),
            "Encrypt and decrypt data"
        );
        assert_eq!(
            CapabilityTaxonomy::Discovery.description(),
            "Discover other primals and nodes"
        );
    }

    #[test]
    fn test_capability_category() {
        assert_eq!(
            CapabilityTaxonomy::Encryption.category(),
            CapabilityCategory::Security
        );
        assert_eq!(
            CapabilityTaxonomy::Discovery.category(),
            CapabilityCategory::Communication
        );
    }

    #[test]
    fn test_capability_from_str() {
        assert_eq!(
            CapabilityTaxonomy::from_str_flexible("encryption"),
            Some(CapabilityTaxonomy::Encryption)
        );
        assert_eq!(
            CapabilityTaxonomy::from_str_flexible("DISCOVERY"),
            Some(CapabilityTaxonomy::Discovery)
        );
        assert_eq!(CapabilityTaxonomy::from_str_flexible("unknown"), None);
    }

    #[test]
    fn test_capability_display() {
        assert_eq!(format!("{}", CapabilityTaxonomy::Encryption), "encryption");
        assert_eq!(
            format!("{}", CapabilityTaxonomy::Custom("my-cap".to_string())),
            "custom:my-cap"
        );
    }

    #[test]
    fn test_resolve_to_primal() {
        assert_eq!(
            CapabilityTaxonomy::resolve_to_primal("encryption"),
            Some("beardog")
        );
        assert_eq!(
            CapabilityTaxonomy::resolve_to_primal("discovery"),
            Some("songbird")
        );
        assert_eq!(CapabilityTaxonomy::resolve_to_primal("unknown"), None);
    }
}
