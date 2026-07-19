use super::*;
use crate::capability_taxonomy::category::CapabilityCategory;

// -------------------------------------------------------------------------
// fmt::Display
// -------------------------------------------------------------------------

#[test]
fn display_standard_variants() {
    assert_eq!(format!("{}", CapabilityTaxonomy::Encryption), "encryption");
    assert_eq!(format!("{}", CapabilityTaxonomy::Discovery), "discovery");
    assert_eq!(
        format!("{}", CapabilityTaxonomy::WorkloadExecution),
        "workloadexecution"
    );
    assert_eq!(
        format!("{}", CapabilityTaxonomy::DataStorage),
        "datastorage"
    );
    assert_eq!(
        format!("{}", CapabilityTaxonomy::VisualRendering),
        "visualrendering"
    );
    assert_eq!(
        format!("{}", CapabilityTaxonomy::AiCoordination),
        "aicoordination"
    );
}

#[test]
fn display_custom_variant() {
    assert_eq!(
        format!("{}", CapabilityTaxonomy::Custom("my-cap".to_string())),
        "custom:my-cap"
    );
    assert_eq!(
        format!("{}", CapabilityTaxonomy::Custom(String::new())),
        "custom:"
    );
}

// -------------------------------------------------------------------------
// known_primals()
// -------------------------------------------------------------------------

#[test]
fn known_primals_returns_core_primals() {
    let primals = CapabilityTaxonomy::known_primals_with(false);
    assert_eq!(primals, crate::primal_names::BOOTSTRAP_CORE_SET);
    assert!(primals.contains(&"beardog"));
    assert!(primals.contains(&"songbird"));
    assert!(primals.contains(&"toadstool"));
    assert!(primals.contains(&"barracuda"));
    assert!(primals.contains(&"coralreef"));
    assert!(primals.contains(&"nestgate"));
    assert!(primals.contains(&"squirrel"));
    assert_eq!(primals.len(), 7);
}

#[test]
fn known_primals_strict_discovery_returns_empty() {
    let primals = CapabilityTaxonomy::known_primals_with(true);
    assert!(primals.is_empty());
}

// -------------------------------------------------------------------------
// from_str_flexible() / parsing
// -------------------------------------------------------------------------

#[test]
fn from_str_flexible_case_insensitive() {
    assert_eq!(
        CapabilityTaxonomy::from_str_flexible("ENCRYPTION"),
        Some(CapabilityTaxonomy::Encryption)
    );
    assert_eq!(
        CapabilityTaxonomy::from_str_flexible("Discovery"),
        Some(CapabilityTaxonomy::Discovery)
    );
    assert_eq!(
        CapabilityTaxonomy::from_str_flexible("WORKLOAD_EXECUTION"),
        Some(CapabilityTaxonomy::WorkloadExecution)
    );
}

#[test]
fn from_str_flexible_aliases() {
    assert_eq!(
        CapabilityTaxonomy::from_str_flexible("security"),
        Some(CapabilityTaxonomy::Encryption)
    );
    assert_eq!(
        CapabilityTaxonomy::from_str_flexible("crypto"),
        Some(CapabilityTaxonomy::Encryption)
    );
    assert_eq!(
        CapabilityTaxonomy::from_str_flexible("network"),
        Some(CapabilityTaxonomy::Discovery)
    );
    assert_eq!(
        CapabilityTaxonomy::from_str_flexible("compute"),
        Some(CapabilityTaxonomy::WorkloadExecution)
    );
    assert_eq!(
        CapabilityTaxonomy::from_str_flexible("storage"),
        Some(CapabilityTaxonomy::DataStorage)
    );
    assert_eq!(
        CapabilityTaxonomy::from_str_flexible("ai"),
        Some(CapabilityTaxonomy::AiCoordination)
    );
    assert_eq!(
        CapabilityTaxonomy::from_str_flexible("mcp"),
        Some(CapabilityTaxonomy::McpServer)
    );
}

#[test]
fn from_str_flexible_edge_cases() {
    assert_eq!(CapabilityTaxonomy::from_str_flexible(""), None);
    assert_eq!(CapabilityTaxonomy::from_str_flexible("unknown"), None);
    assert_eq!(CapabilityTaxonomy::from_str_flexible(" "), None);
    assert_eq!(CapabilityTaxonomy::from_str_flexible("encryption "), None);
}
fn from_str_flexible_http_bridge_and_federation_aliases() {
    assert_eq!(
        CapabilityTaxonomy::from_str_flexible("http_bridge"),
        Some(CapabilityTaxonomy::Discovery)
    );
    assert_eq!(
        CapabilityTaxonomy::from_str_flexible("federation"),
        Some(CapabilityTaxonomy::P2PFederation)
    );
    assert_eq!(
        CapabilityTaxonomy::from_str_flexible("dedup"),
        Some(CapabilityTaxonomy::Deduplication)
    );
}
