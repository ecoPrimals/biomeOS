// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

use super::*;

fn resolve(cap: &str) -> Option<&'static str> {
    CapabilityTaxonomy::from_str_flexible(cap).and_then(|c| c.default_primal_with(false))
}

// -------------------------------------------------------------------------
// resolve_to_primal() - each capability category
// -------------------------------------------------------------------------

#[test]
fn resolve_to_primal_security_beardog() {
    assert_eq!(resolve("encryption"), Some("beardog"));
    assert_eq!(resolve("identity"), Some("beardog"));
    assert_eq!(resolve("trust"), Some("beardog"));
    assert_eq!(resolve("key_management"), Some("beardog"));
    assert_eq!(resolve("hardware_security"), Some("beardog"));
    assert_eq!(resolve("secure_tunneling"), Some("beardog"));
}

#[test]
fn resolve_to_primal_discovery_songbird() {
    assert_eq!(resolve("discovery"), Some("songbird"));
    assert_eq!(resolve("p2p_federation"), Some("songbird"));
    assert_eq!(resolve("tunneling"), Some("songbird"));
    assert_eq!(resolve("routing"), Some("songbird"));
    assert_eq!(resolve("genetic_routing"), Some("songbird"));
    assert_eq!(resolve("mesh_relay"), Some("songbird"));
    assert_eq!(resolve("hole_punch"), Some("songbird"));
    assert_eq!(resolve("stun_client"), Some("songbird"));
    assert_eq!(resolve("onion_service"), Some("songbird"));
    assert_eq!(resolve("relay_server"), Some("songbird"));
}

#[test]
fn resolve_to_primal_compute_toadstool() {
    assert_eq!(resolve("workload_execution"), Some("toadstool"));
    assert_eq!(resolve("resource_scheduling"), Some("toadstool"));
    assert_eq!(resolve("process_isolation"), Some("toadstool"));
    assert_eq!(resolve("fractal_compute"), Some("toadstool"));
    assert_eq!(resolve("gpu_acceleration"), Some("toadstool"));
}

#[test]
fn resolve_to_primal_storage_nestgate() {
    assert_eq!(resolve("data_storage"), Some("nestgate"));
    assert_eq!(resolve("provenance"), Some("nestgate"));
    assert_eq!(resolve("compression"), Some("nestgate"));
    assert_eq!(resolve("replication"), Some("nestgate"));
    assert_eq!(resolve("deduplication"), Some("nestgate"));
    assert_eq!(resolve("content_addressed"), Some("nestgate"));
}

#[test]
fn resolve_to_primal_ai_squirrel() {
    assert_eq!(resolve("ai_coordination"), Some("squirrel"));
    assert_eq!(resolve("ai_multi_provider"), Some("squirrel"));
    assert_eq!(resolve("mcp_server"), Some("squirrel"));
    assert_eq!(resolve("ai_capability_discovery"), Some("squirrel"));
}

#[test]
fn resolve_to_primal_orchestration_biomeos() {
    assert_eq!(resolve("lifecycle_management"), Some("biomeos"));
    assert_eq!(resolve("health_monitoring"), Some("biomeos"));
    assert_eq!(resolve("config_management"), Some("biomeos"));
    assert_eq!(resolve("metrics_collection"), Some("biomeos"));
    assert_eq!(resolve("log_aggregation"), Some("biomeos"));
    assert_eq!(resolve("graph_orchestration"), Some("biomeos"));
    assert_eq!(resolve("spore_deployment"), Some("biomeos"));
    assert_eq!(resolve("genetic_lineage"), Some("biomeos"));
    assert_eq!(resolve("niche_deployment"), Some("biomeos"));
}

#[test]
fn resolve_to_primal_unknown_and_empty() {
    assert_eq!(resolve("unknown"), None);
    assert_eq!(resolve(""), None);
    assert_eq!(resolve("nonexistent_capability"), None);
}

// -------------------------------------------------------------------------
// default_primal() - each variant
// -------------------------------------------------------------------------

#[test]
fn default_primal_security_beardog() {
    assert_eq!(
        CapabilityTaxonomy::Encryption.default_primal_with(false),
        Some("beardog")
    );
    assert_eq!(
        CapabilityTaxonomy::Identity.default_primal_with(false),
        Some("beardog")
    );
    assert_eq!(
        CapabilityTaxonomy::Trust.default_primal_with(false),
        Some("beardog")
    );
    assert_eq!(
        CapabilityTaxonomy::KeyManagement.default_primal_with(false),
        Some("beardog")
    );
    assert_eq!(
        CapabilityTaxonomy::HardwareSecurity.default_primal_with(false),
        Some("beardog")
    );
    assert_eq!(
        CapabilityTaxonomy::SecureTunneling.default_primal_with(false),
        Some("beardog")
    );
}

#[test]
fn default_primal_communication_songbird() {
    assert_eq!(
        CapabilityTaxonomy::Discovery.default_primal_with(false),
        Some("songbird")
    );
    assert_eq!(
        CapabilityTaxonomy::P2PFederation.default_primal_with(false),
        Some("songbird")
    );
    assert_eq!(
        CapabilityTaxonomy::BluetoothGenesis.default_primal_with(false),
        Some("songbird")
    );
}

#[test]
fn default_primal_compute_toadstool() {
    assert_eq!(
        CapabilityTaxonomy::WorkloadExecution.default_primal_with(false),
        Some("toadstool")
    );
    assert_eq!(
        CapabilityTaxonomy::GpuAcceleration.default_primal_with(false),
        Some("toadstool")
    );
}

#[test]
fn default_primal_storage_nestgate() {
    assert_eq!(
        CapabilityTaxonomy::DataStorage.default_primal_with(false),
        Some("nestgate")
    );
    assert_eq!(
        CapabilityTaxonomy::Provenance.default_primal_with(false),
        Some("nestgate")
    );
}

#[test]
fn default_primal_ai_squirrel() {
    assert_eq!(
        CapabilityTaxonomy::AiCoordination.default_primal_with(false),
        Some("squirrel")
    );
    assert_eq!(
        CapabilityTaxonomy::McpServer.default_primal_with(false),
        Some("squirrel")
    );
}

#[test]
fn default_primal_orchestration_biomeos() {
    assert_eq!(
        CapabilityTaxonomy::LifecycleManagement.default_primal_with(false),
        Some("biomeos")
    );
    assert_eq!(
        CapabilityTaxonomy::HealthMonitoring.default_primal_with(false),
        Some("biomeos")
    );
}

#[test]
fn default_primal_ui_and_medical_none() {
    assert_eq!(
        CapabilityTaxonomy::VisualRendering.default_primal_with(false),
        None
    );
    assert_eq!(
        CapabilityTaxonomy::InputHandling.default_primal_with(false),
        None
    );
    assert_eq!(
        CapabilityTaxonomy::BiosignalProcessing.default_primal_with(false),
        None
    );
    assert_eq!(
        CapabilityTaxonomy::SurgicalToolSimulation.default_primal_with(false),
        None
    );
}

#[test]
fn default_primal_custom_none() {
    assert_eq!(
        CapabilityTaxonomy::Custom("foo".to_string()).default_primal_with(false),
        None
    );
}

#[test]
fn default_primal_strict_discovery_returns_none() {
    assert_eq!(
        CapabilityTaxonomy::Encryption.default_primal_with(true),
        None
    );
    assert_eq!(
        CapabilityTaxonomy::Discovery.default_primal_with(true),
        None
    );
}

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
        format!("{}", CapabilityTaxonomy::Custom("".to_string())),
        "custom:"
    );
}

// -------------------------------------------------------------------------
// known_primals()
// -------------------------------------------------------------------------

#[test]
fn known_primals_returns_core_primals() {
    let primals = CapabilityTaxonomy::known_primals_with(false);
    assert_eq!(primals, crate::primal_names::CORE_PRIMALS);
    assert!(primals.contains(&"beardog"));
    assert!(primals.contains(&"songbird"));
    assert!(primals.contains(&"toadstool"));
    assert!(primals.contains(&"nestgate"));
    assert!(primals.contains(&"squirrel"));
    assert_eq!(primals.len(), 5);
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
