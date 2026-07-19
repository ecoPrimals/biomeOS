use super::*;
use crate::capability_taxonomy::category::CapabilityCategory;

// -------------------------------------------------------------------------
// Custom variant
// -------------------------------------------------------------------------

#[test]
fn custom_description() {
    let cap = CapabilityTaxonomy::Custom("foo".to_string());
    assert_eq!(cap.description().as_ref(), "Custom: foo");
}

#[test]
fn custom_category() {
    assert_eq!(
        CapabilityTaxonomy::Custom("x".to_string()).category(),
        CapabilityCategory::Specialized
    );
}

#[test]
fn custom_equality() {
    assert_eq!(
        CapabilityTaxonomy::Custom("a".to_string()),
        CapabilityTaxonomy::Custom("a".to_string())
    );
    assert_ne!(
        CapabilityTaxonomy::Custom("a".to_string()),
        CapabilityTaxonomy::Custom("b".to_string())
    );
}

// -------------------------------------------------------------------------
// description() and category()
// -------------------------------------------------------------------------

#[test]
fn description_security() {
    assert_eq!(
        CapabilityTaxonomy::Encryption.description().as_ref(),
        "Encrypt and decrypt data"
    );
    assert_eq!(
        CapabilityTaxonomy::Identity.description().as_ref(),
        "Verify cryptographic identities"
    );
}

#[test]
fn category_mapping() {
    assert_eq!(
        CapabilityTaxonomy::Encryption.category(),
        CapabilityCategory::Security
    );
    assert_eq!(
        CapabilityTaxonomy::Discovery.category(),
        CapabilityCategory::Communication
    );
    assert_eq!(
        CapabilityTaxonomy::WorkloadExecution.category(),
        CapabilityCategory::Compute
    );
    assert_eq!(
        CapabilityTaxonomy::DataStorage.category(),
        CapabilityCategory::Storage
    );
    assert_eq!(
        CapabilityTaxonomy::VisualRendering.category(),
        CapabilityCategory::UserInterface
    );
    assert_eq!(
        CapabilityTaxonomy::LifecycleManagement.category(),
        CapabilityCategory::Orchestration
    );
    assert_eq!(
        CapabilityTaxonomy::AiCoordination.category(),
        CapabilityCategory::AI
    );
    assert_eq!(
        CapabilityTaxonomy::BluetoothGenesis.category(),
        CapabilityCategory::Specialized
    );
}

// -------------------------------------------------------------------------
// Serde round-trip
// -------------------------------------------------------------------------

#[test]
fn serde_roundtrip_standard() {
    let cap = CapabilityTaxonomy::Encryption;
    let json = serde_json::to_string(&cap).unwrap();
    assert_eq!(json, r#""encryption""#);
    let parsed: CapabilityTaxonomy = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, cap);
}

#[test]
fn serde_roundtrip_custom() {
    let cap = CapabilityTaxonomy::Custom("my-cap".to_string());
    let json = serde_json::to_string(&cap).unwrap();
    let parsed: CapabilityTaxonomy = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, cap);
}

#[test]
fn representative_for_category_maps_to_expected_variants() {
    assert_eq!(
        CapabilityTaxonomy::representative_for_category(CapabilityCategory::Security),
        Some(CapabilityTaxonomy::Encryption)
    );
    assert_eq!(
        CapabilityTaxonomy::representative_for_category(CapabilityCategory::Communication),
        Some(CapabilityTaxonomy::Discovery)
    );
    assert_eq!(
        CapabilityTaxonomy::representative_for_category(CapabilityCategory::Compute),
        Some(CapabilityTaxonomy::WorkloadExecution)
    );
    assert_eq!(
        CapabilityTaxonomy::representative_for_category(CapabilityCategory::Storage),
        Some(CapabilityTaxonomy::DataStorage)
    );
    assert_eq!(
        CapabilityTaxonomy::representative_for_category(CapabilityCategory::AI),
        Some(CapabilityTaxonomy::AiCoordination)
    );
    assert_eq!(
        CapabilityTaxonomy::representative_for_category(CapabilityCategory::Orchestration),
        Some(CapabilityTaxonomy::LifecycleManagement)
    );
    assert_eq!(
        CapabilityTaxonomy::representative_for_category(CapabilityCategory::UserInterface),
        Some(CapabilityTaxonomy::VisualRendering)
    );
    assert_eq!(
        CapabilityTaxonomy::representative_for_category(CapabilityCategory::Specialized),
        Some(CapabilityTaxonomy::GeneticLineage)
    );
}
