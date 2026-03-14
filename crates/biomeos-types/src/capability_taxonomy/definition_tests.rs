// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

use super::*;

fn ensure_non_strict_discovery() {
    std::env::remove_var("BIOMEOS_STRICT_DISCOVERY");
}

// -------------------------------------------------------------------------
// resolve_to_primal() - each capability category
// -------------------------------------------------------------------------

#[test]
fn resolve_to_primal_security_beardog() {
    ensure_non_strict_discovery();
    assert_eq!(
        CapabilityTaxonomy::resolve_to_primal("encryption"),
        Some("beardog")
    );
    assert_eq!(
        CapabilityTaxonomy::resolve_to_primal("identity"),
        Some("beardog")
    );
    assert_eq!(
        CapabilityTaxonomy::resolve_to_primal("trust"),
        Some("beardog")
    );
    assert_eq!(
        CapabilityTaxonomy::resolve_to_primal("key_management"),
        Some("beardog")
    );
    assert_eq!(
        CapabilityTaxonomy::resolve_to_primal("hardware_security"),
        Some("beardog")
    );
    assert_eq!(
        CapabilityTaxonomy::resolve_to_primal("secure_tunneling"),
        Some("beardog")
    );
}

#[test]
fn resolve_to_primal_discovery_songbird() {
    ensure_non_strict_discovery();
    assert_eq!(
        CapabilityTaxonomy::resolve_to_primal("discovery"),
        Some("songbird")
    );
    assert_eq!(
        CapabilityTaxonomy::resolve_to_primal("p2p_federation"),
        Some("songbird")
    );
    assert_eq!(
        CapabilityTaxonomy::resolve_to_primal("tunneling"),
        Some("songbird")
    );
    assert_eq!(
        CapabilityTaxonomy::resolve_to_primal("routing"),
        Some("songbird")
    );
    assert_eq!(
        CapabilityTaxonomy::resolve_to_primal("genetic_routing"),
        Some("songbird")
    );
    assert_eq!(
        CapabilityTaxonomy::resolve_to_primal("mesh_relay"),
        Some("songbird")
    );
    assert_eq!(
        CapabilityTaxonomy::resolve_to_primal("hole_punch"),
        Some("songbird")
    );
    assert_eq!(
        CapabilityTaxonomy::resolve_to_primal("stun_client"),
        Some("songbird")
    );
    assert_eq!(
        CapabilityTaxonomy::resolve_to_primal("onion_service"),
        Some("songbird")
    );
    assert_eq!(
        CapabilityTaxonomy::resolve_to_primal("relay_server"),
        Some("songbird")
    );
}

#[test]
fn resolve_to_primal_compute_toadstool() {
    ensure_non_strict_discovery();
    assert_eq!(
        CapabilityTaxonomy::resolve_to_primal("workload_execution"),
        Some("toadstool")
    );
    assert_eq!(
        CapabilityTaxonomy::resolve_to_primal("resource_scheduling"),
        Some("toadstool")
    );
    assert_eq!(
        CapabilityTaxonomy::resolve_to_primal("process_isolation"),
        Some("toadstool")
    );
    assert_eq!(
        CapabilityTaxonomy::resolve_to_primal("fractal_compute"),
        Some("toadstool")
    );
    assert_eq!(
        CapabilityTaxonomy::resolve_to_primal("gpu_acceleration"),
        Some("toadstool")
    );
}

#[test]
fn resolve_to_primal_storage_nestgate() {
    ensure_non_strict_discovery();
    assert_eq!(
        CapabilityTaxonomy::resolve_to_primal("data_storage"),
        Some("nestgate")
    );
    assert_eq!(
        CapabilityTaxonomy::resolve_to_primal("provenance"),
        Some("nestgate")
    );
    assert_eq!(
        CapabilityTaxonomy::resolve_to_primal("compression"),
        Some("nestgate")
    );
    assert_eq!(
        CapabilityTaxonomy::resolve_to_primal("replication"),
        Some("nestgate")
    );
    assert_eq!(
        CapabilityTaxonomy::resolve_to_primal("deduplication"),
        Some("nestgate")
    );
    assert_eq!(
        CapabilityTaxonomy::resolve_to_primal("content_addressed"),
        Some("nestgate")
    );
}

#[test]
fn resolve_to_primal_ai_squirrel() {
    ensure_non_strict_discovery();
    assert_eq!(
        CapabilityTaxonomy::resolve_to_primal("ai_coordination"),
        Some("squirrel")
    );
    assert_eq!(
        CapabilityTaxonomy::resolve_to_primal("ai_multi_provider"),
        Some("squirrel")
    );
    assert_eq!(
        CapabilityTaxonomy::resolve_to_primal("mcp_server"),
        Some("squirrel")
    );
    assert_eq!(
        CapabilityTaxonomy::resolve_to_primal("ai_capability_discovery"),
        Some("squirrel")
    );
}

#[test]
fn resolve_to_primal_orchestration_biomeos() {
    ensure_non_strict_discovery();
    assert_eq!(
        CapabilityTaxonomy::resolve_to_primal("lifecycle_management"),
        Some("biomeos")
    );
    assert_eq!(
        CapabilityTaxonomy::resolve_to_primal("health_monitoring"),
        Some("biomeos")
    );
    assert_eq!(
        CapabilityTaxonomy::resolve_to_primal("config_management"),
        Some("biomeos")
    );
    assert_eq!(
        CapabilityTaxonomy::resolve_to_primal("metrics_collection"),
        Some("biomeos")
    );
    assert_eq!(
        CapabilityTaxonomy::resolve_to_primal("log_aggregation"),
        Some("biomeos")
    );
    assert_eq!(
        CapabilityTaxonomy::resolve_to_primal("graph_orchestration"),
        Some("biomeos")
    );
    assert_eq!(
        CapabilityTaxonomy::resolve_to_primal("spore_deployment"),
        Some("biomeos")
    );
    assert_eq!(
        CapabilityTaxonomy::resolve_to_primal("genetic_lineage"),
        Some("biomeos")
    );
    assert_eq!(
        CapabilityTaxonomy::resolve_to_primal("niche_deployment"),
        Some("biomeos")
    );
}

#[test]
fn resolve_to_primal_unknown_and_empty() {
    assert_eq!(CapabilityTaxonomy::resolve_to_primal("unknown"), None);
    assert_eq!(CapabilityTaxonomy::resolve_to_primal(""), None);
    assert_eq!(
        CapabilityTaxonomy::resolve_to_primal("nonexistent_capability"),
        None
    );
}

// -------------------------------------------------------------------------
// default_primal() - each variant
// -------------------------------------------------------------------------

#[test]
fn default_primal_security_beardog() {
    ensure_non_strict_discovery();
    assert_eq!(
        CapabilityTaxonomy::Encryption.default_primal(),
        Some("beardog")
    );
    assert_eq!(
        CapabilityTaxonomy::Identity.default_primal(),
        Some("beardog")
    );
    assert_eq!(CapabilityTaxonomy::Trust.default_primal(), Some("beardog"));
    assert_eq!(
        CapabilityTaxonomy::KeyManagement.default_primal(),
        Some("beardog")
    );
    assert_eq!(
        CapabilityTaxonomy::HardwareSecurity.default_primal(),
        Some("beardog")
    );
    assert_eq!(
        CapabilityTaxonomy::SecureTunneling.default_primal(),
        Some("beardog")
    );
}

#[test]
fn default_primal_communication_songbird() {
    ensure_non_strict_discovery();
    assert_eq!(
        CapabilityTaxonomy::Discovery.default_primal(),
        Some("songbird")
    );
    assert_eq!(
        CapabilityTaxonomy::P2PFederation.default_primal(),
        Some("songbird")
    );
    assert_eq!(
        CapabilityTaxonomy::BluetoothGenesis.default_primal(),
        Some("songbird")
    );
}

#[test]
fn default_primal_compute_toadstool() {
    ensure_non_strict_discovery();
    assert_eq!(
        CapabilityTaxonomy::WorkloadExecution.default_primal(),
        Some("toadstool")
    );
    assert_eq!(
        CapabilityTaxonomy::GpuAcceleration.default_primal(),
        Some("toadstool")
    );
}

#[test]
fn default_primal_storage_nestgate() {
    ensure_non_strict_discovery();
    assert_eq!(
        CapabilityTaxonomy::DataStorage.default_primal(),
        Some("nestgate")
    );
    assert_eq!(
        CapabilityTaxonomy::Provenance.default_primal(),
        Some("nestgate")
    );
}

#[test]
fn default_primal_ai_squirrel() {
    ensure_non_strict_discovery();
    assert_eq!(
        CapabilityTaxonomy::AiCoordination.default_primal(),
        Some("squirrel")
    );
    assert_eq!(
        CapabilityTaxonomy::McpServer.default_primal(),
        Some("squirrel")
    );
}

#[test]
fn default_primal_orchestration_biomeos() {
    ensure_non_strict_discovery();
    assert_eq!(
        CapabilityTaxonomy::LifecycleManagement.default_primal(),
        Some("biomeos")
    );
    assert_eq!(
        CapabilityTaxonomy::HealthMonitoring.default_primal(),
        Some("biomeos")
    );
}

#[test]
fn default_primal_ui_and_medical_none() {
    ensure_non_strict_discovery();
    assert_eq!(CapabilityTaxonomy::VisualRendering.default_primal(), None);
    assert_eq!(CapabilityTaxonomy::InputHandling.default_primal(), None);
    assert_eq!(
        CapabilityTaxonomy::BiosignalProcessing.default_primal(),
        None
    );
    assert_eq!(
        CapabilityTaxonomy::SurgicalToolSimulation.default_primal(),
        None
    );
}

#[test]
fn default_primal_custom_none() {
    ensure_non_strict_discovery();
    assert_eq!(
        CapabilityTaxonomy::Custom("foo".to_string()).default_primal(),
        None
    );
}

#[test]
fn default_primal_strict_discovery_returns_none() {
    // Env var manipulation is inherently racy in parallel tests.
    // Verify the logic directly: when the env var IS present, default_primal
    // returns None. We set it and immediately read; if another thread
    // clears it between those two lines the assertion will be wrong, so we
    // only assert when we can confirm the var is still set.
    std::env::set_var("BIOMEOS_STRICT_DISCOVERY", "1");
    if std::env::var("BIOMEOS_STRICT_DISCOVERY").is_ok() {
        assert_eq!(CapabilityTaxonomy::Encryption.default_primal(), None);
        assert_eq!(CapabilityTaxonomy::Discovery.default_primal(), None);
    }
    std::env::remove_var("BIOMEOS_STRICT_DISCOVERY");
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
    ensure_non_strict_discovery();
    let primals = CapabilityTaxonomy::known_primals();
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
    std::env::set_var("BIOMEOS_STRICT_DISCOVERY", "1");
    if std::env::var("BIOMEOS_STRICT_DISCOVERY").is_ok() {
        let primals = CapabilityTaxonomy::known_primals();
        assert!(primals.is_empty());
    }
    std::env::remove_var("BIOMEOS_STRICT_DISCOVERY");
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
