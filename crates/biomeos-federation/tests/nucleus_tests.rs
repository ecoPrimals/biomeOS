//! NUCLEUS (Secure Discovery Protocol) Tests
//!
//! Comprehensive test suite for the 5-layer secure discovery protocol.

use biomeos_federation::{
    Capability, CapabilitySet, IdentityProof, SecureNucleusDiscovery, SelectionCriteria,
    TrustLevel, VerifiedPrimal,
};
use std::path::PathBuf;

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_nucleus_initialization() {
    let nucleus = SecureNucleusDiscovery::new();

    // Should initialize with empty registry
    let primals = nucleus.all();
    assert_eq!(primals.len(), 0, "New NUCLEUS should have no primals");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_nucleus_with_clients() {
    // Test initialization with clients
    let nucleus = SecureNucleusDiscovery::with_clients(None, None);

    let primals = nucleus.all();
    assert_eq!(
        primals.len(),
        0,
        "NUCLEUS with no clients should have no primals"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_insecure_discovery() {
    let mut nucleus = SecureNucleusDiscovery::new();

    // Insecure discovery should work without Songbird/BearDog
    let result = nucleus.discover_insecure().await;

    // Should not error (even if no primals found)
    assert!(
        result.is_ok(),
        "Insecure discovery should not error: {:?}",
        result
    );

    // Should return empty list if no primals available
    let primals = result.unwrap();
    // Can't assert exact count as it depends on system state
    println!("Discovered {} primals (insecure mode)", primals.len());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_selection_by_capability() {
    let nucleus = SecureNucleusDiscovery::new();

    // Create a mock verified primal
    let primal = create_mock_verified_primal(
        "beardog",
        "node-alpha",
        vec!["security", "encryption"],
        TrustLevel::High,
    );

    // Add to registry using test injection
    let nucleus = add_primal_to_nucleus(nucleus, primal);

    // Test selection by capability
    let capability = Capability::Custom("encryption".to_string());
    let found = nucleus.get(SelectionCriteria::ByCapability(capability));

    assert!(
        found.is_some(),
        "Should find primal with encryption capability"
    );
    assert_eq!(found.unwrap().name, "beardog");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_selection_by_node_id() {
    let nucleus = SecureNucleusDiscovery::new();

    let primal = create_mock_verified_primal(
        "songbird",
        "node-beta",
        vec!["discovery", "p2p"],
        TrustLevel::Elevated,
    );

    let nucleus = add_primal_to_nucleus(nucleus, primal);

    // Test selection by node ID
    let found = nucleus.get(SelectionCriteria::ByNodeId("node-beta".to_string()));

    assert!(found.is_some(), "Should find primal with node-beta ID");
    assert_eq!(found.unwrap().name, "songbird");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_selection_by_family() {
    use biomeos_types::identifiers::FamilyId;
    let nucleus = SecureNucleusDiscovery::new();
    let test_family = FamilyId::generate().to_string();

    let primal = create_mock_verified_primal_with_family(
        "beardog",
        "node-alpha",
        Some(test_family.clone()),
        vec!["security"],
        TrustLevel::High,
    );

    let nucleus = add_primal_to_nucleus(nucleus, primal);

    // Test selection by family
    let found = nucleus.get(SelectionCriteria::ByFamily(test_family.clone()));

    assert!(found.is_some(), "Should find primal in test family");
    assert_eq!(found.unwrap().family_id, Some(test_family));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_selection_min_trust_level() {
    let nucleus = SecureNucleusDiscovery::new();

    // Add primals with different trust levels
    let low_trust =
        create_mock_verified_primal("unknown-primal", "node-unknown", vec![], TrustLevel::Basic);

    let high_trust = create_mock_verified_primal(
        "beardog",
        "node-alpha",
        vec!["security"],
        TrustLevel::Highest,
    );

    let nucleus = add_primal_to_nucleus(nucleus, low_trust);
    let nucleus = add_primal_to_nucleus(nucleus, high_trust);

    // Test minimum trust level
    let found = nucleus.get(SelectionCriteria::MinTrustLevel(TrustLevel::High));

    assert!(found.is_some(), "Should find high trust primal");
    assert!(found.unwrap().trust_level >= TrustLevel::High);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_multiple_instances() {
    let nucleus = SecureNucleusDiscovery::new();

    // Add multiple instances of the same primal
    let beardog1 =
        create_mock_verified_primal("beardog", "node-alpha", vec!["security"], TrustLevel::High);

    let beardog2 = create_mock_verified_primal(
        "beardog",
        "node-beta",
        vec!["security"],
        TrustLevel::Elevated,
    );

    let nucleus = add_primal_to_nucleus(nucleus, beardog1);
    let nucleus = add_primal_to_nucleus(nucleus, beardog2);

    // Test get_all returns all instances
    let all_beardogs = nucleus.get_all("beardog");

    assert_eq!(all_beardogs.len(), 2, "Should find both beardog instances");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_with_capability() {
    let nucleus = SecureNucleusDiscovery::new();

    let beardog = create_mock_verified_primal(
        "beardog",
        "node-alpha",
        vec!["security", "encryption"],
        TrustLevel::High,
    );

    let songbird = create_mock_verified_primal(
        "songbird",
        "node-beta",
        vec!["discovery", "p2p"],
        TrustLevel::Elevated,
    );

    let nucleus = add_primal_to_nucleus(nucleus, beardog);
    let nucleus = add_primal_to_nucleus(nucleus, songbird);

    // Test with_capability
    let security_cap = Capability::Custom("security".to_string());
    let security_primals = nucleus.with_capability(&security_cap);

    assert_eq!(security_primals.len(), 1, "Should find 1 security primal");
    assert_eq!(security_primals[0].name, "beardog");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_trust_level_ordering() {
    // Verify trust levels are ordered correctly
    assert!(TrustLevel::Unknown < TrustLevel::Basic);
    assert!(TrustLevel::Basic < TrustLevel::Elevated);
    assert!(TrustLevel::Elevated < TrustLevel::High);
    assert!(TrustLevel::High < TrustLevel::Highest);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_selection_prefers_highest_trust() {
    let nucleus = SecureNucleusDiscovery::new();

    let encryption_cap = Capability::Custom("encryption".to_string());

    // Add multiple primals with same capability but different trust
    let beardog_low = create_mock_verified_primal(
        "beardog-1",
        "node-gamma",
        vec!["encryption"],
        TrustLevel::Basic,
    );

    let beardog_high = create_mock_verified_primal(
        "beardog-2",
        "node-alpha",
        vec!["encryption"],
        TrustLevel::Highest,
    );

    let nucleus = add_primal_to_nucleus(nucleus, beardog_low);
    let nucleus = add_primal_to_nucleus(nucleus, beardog_high);

    // ByCapability should select highest trust
    let found = nucleus.get(SelectionCriteria::ByCapability(encryption_cap));

    assert!(found.is_some());
    assert_eq!(found.unwrap().trust_level, TrustLevel::Highest);
    assert_eq!(found.unwrap().name, "beardog-2");
}

// Helper functions

fn create_mock_verified_primal(
    name: &str,
    node_id: &str,
    capabilities: Vec<&str>,
    trust_level: TrustLevel,
) -> VerifiedPrimal {
    create_mock_verified_primal_with_family(name, node_id, None, capabilities, trust_level)
}

fn create_mock_verified_primal_with_family(
    name: &str,
    node_id: &str,
    family_id: Option<String>,
    capabilities: Vec<&str>,
    trust_level: TrustLevel,
) -> VerifiedPrimal {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let mut capability_set = CapabilitySet::new();
    for cap in capabilities {
        capability_set.add(Capability::Custom(cap.to_string()));
    }

    VerifiedPrimal {
        name: name.to_string(),
        node_id: node_id.to_string(),
        family_id,
        endpoints: vec![],
        capabilities: capability_set,
        identity_proof: IdentityProof {
            node_id: node_id.to_string(),
            signature: "test-signature".to_string(),
            challenge: "test-challenge".to_string(),
            public_key: "test-pubkey".to_string(),
            timestamp: now,
        },
        trust_level,
        discovered_at: now,
        verified_at: now,
        metadata: std::collections::HashMap::new(),
    }
}

fn add_primal_to_nucleus(
    nucleus: SecureNucleusDiscovery,
    primal: VerifiedPrimal,
) -> SecureNucleusDiscovery {
    nucleus.inject_primal_for_testing(primal)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_identity_proof_structure() {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let proof = IdentityProof {
        node_id: "node-alpha".to_string(),
        signature: "ed25519-signature".to_string(),
        challenge: "random-challenge".to_string(),
        public_key: "ed25519-pubkey".to_string(),
        timestamp: now,
    };

    assert_eq!(proof.node_id, "node-alpha");
    assert!(!proof.signature.is_empty());
    assert!(!proof.challenge.is_empty());
    assert!(!proof.public_key.is_empty());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_selection_by_socket() {
    let nucleus = SecureNucleusDiscovery::new();

    let socket_path = PathBuf::from("/tmp/beardog-node-alpha.sock");

    // Test the SelectionCriteria API
    let _criteria = SelectionCriteria::BySocket(socket_path.clone());

    // Note: Currently returns None since we can't inject socket endpoints
    // This tests the API exists and compiles
    let found = nucleus.get(SelectionCriteria::BySocket(socket_path));
    assert!(
        found.is_none(),
        "Should not find primal without proper endpoint injection"
    );
}

#[test]
fn test_trust_level_serialization() {
    // Verify trust levels can be serialized/deserialized
    let trust = TrustLevel::High;
    let json = serde_json::to_string(&trust).unwrap();
    let deserialized: TrustLevel = serde_json::from_str(&json).unwrap();

    assert_eq!(trust, deserialized);
}
