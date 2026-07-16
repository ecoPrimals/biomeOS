use crate::capability_taxonomy::category::CapabilityCategory;
use super::*;

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
    assert_eq!(resolve("niche_deployment"), Some("biomeos"));
    // genetic_lineage is now owned by BearDog (HKDF derivation, lineage proofs)
    assert_eq!(resolve("genetic_lineage"), Some("beardog"));
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
#[test]
fn resolve_to_primal_public_matches_from_str_and_default() {
    assert_eq!(
        CapabilityTaxonomy::resolve_to_primal("encryption"),
        CapabilityTaxonomy::Encryption.default_primal_with(false)
    );
}

#[test]
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

#[test]
fn domain_mapping_spore_and_niche_resolve_to_biomeos() {
    assert_eq!(resolve("spore"), Some("biomeos"));
    assert_eq!(resolve("niche"), Some("biomeos"));
    assert_eq!(resolve("lifecycle"), Some("biomeos"));
}
