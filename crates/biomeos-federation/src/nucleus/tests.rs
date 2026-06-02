// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

#![expect(
    clippy::unwrap_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
#![expect(
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]

use super::*;

fn make_verified(
    name: &str,
    node_id: &str,
    family_id: Option<&str>,
    trust: TrustLevel,
    caps: Vec<Capability>,
    endpoints: Vec<PrimalEndpoint>,
) -> VerifiedPrimal {
    let mut cap_set = CapabilitySet::new();
    for c in caps {
        cap_set.add(c);
    }
    VerifiedPrimal {
        name: name.into(),
        node_id: node_id.into(),
        family_id: family_id.map(String::from),
        endpoints,
        capabilities: cap_set,
        identity_proof: IdentityProof {
            node_id: node_id.into(),
            family_id: family_id.map(String::from),
            signature: "test-sig".into(),
            challenge: "test-challenge".into(),
            public_key: "test-pubkey".into(),
            timestamp: 1000,
        },
        trust_level: trust,
        discovered_at: 1000,
        verified_at: 1001,
        metadata: HashMap::new(),
    }
}

#[test]
fn test_selection_criteria_debug() {
    let c = SelectionCriteria::ByCapability(Capability::Storage);
    assert!(format!("{c:?}").contains("ByCapability"));

    let c2 = SelectionCriteria::ByNodeId("node-1".into());
    assert!(format!("{c2:?}").contains("node-1"));

    let c3 = SelectionCriteria::Any;
    assert!(format!("{c3:?}").contains("Any"));
}

#[test]
fn test_selection_criteria_clone() {
    let c = SelectionCriteria::ByFamily("fam-1".into());
    let c2 = c;
    assert!(format!("{c2:?}").contains("fam-1"));
}

#[test]
fn test_verified_primal_serde_roundtrip() {
    let vp = make_verified(
        "beardog",
        "node-bd",
        Some("fam-x"),
        TrustLevel::High,
        vec![Capability::Storage, Capability::Compute],
        vec![PrimalEndpoint::UnixSocket {
            path: PathBuf::from("/tmp/beardog.sock"),
        }],
    );
    let json = serde_json::to_string(&vp).expect("serialize");
    let restored: VerifiedPrimal = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(restored.name, "beardog");
    assert_eq!(restored.node_id, "node-bd");
    assert_eq!(restored.trust_level, TrustLevel::High);
    assert!(restored.capabilities.has(&Capability::Storage));
}

#[test]
fn test_verified_primal_clone() {
    let vp = make_verified("test", "n", None, TrustLevel::Basic, vec![], vec![]);
    let c = vp.clone();
    assert_eq!(c.name, vp.name);
}

#[test]
fn test_new_creates_empty() {
    let disc = SecureNucleusDiscovery::new();
    assert!(disc.verified_primals.is_empty());
    assert!(disc.songbird.is_none());
    assert!(disc.security_client.is_none());
    assert!(disc.family_id.is_none());
}

#[test]
fn test_default_same_as_new() {
    let disc = SecureNucleusDiscovery::default();
    assert!(disc.verified_primals.is_empty());
}

#[test]
fn test_inject_single_primal() {
    let vp = make_verified("songbird", "s1", None, TrustLevel::Basic, vec![], vec![]);
    let disc = SecureNucleusDiscovery::new().inject_primal_for_testing(vp);

    assert_eq!(disc.all().len(), 1);
    assert_eq!(disc.get_all("songbird").len(), 1);
}

#[test]
fn test_inject_multiple_instances_same_name() {
    let vp1 = make_verified("beardog", "bd-1", None, TrustLevel::Basic, vec![], vec![]);
    let vp2 = make_verified("beardog", "bd-2", None, TrustLevel::High, vec![], vec![]);
    let disc = SecureNucleusDiscovery::new()
        .inject_primal_for_testing(vp1)
        .inject_primal_for_testing(vp2);

    assert_eq!(disc.get_all("beardog").len(), 2);
}

#[test]
fn test_get_by_capability_returns_highest_trust() {
    let low = make_verified(
        "a",
        "a1",
        None,
        TrustLevel::Basic,
        vec![Capability::Storage],
        vec![],
    );
    let high = make_verified(
        "b",
        "b1",
        None,
        TrustLevel::High,
        vec![Capability::Storage],
        vec![],
    );
    let disc = SecureNucleusDiscovery::new()
        .inject_primal_for_testing(low)
        .inject_primal_for_testing(high);

    let result = disc.get(SelectionCriteria::ByCapability(Capability::Storage));
    assert!(result.is_some());
    assert_eq!(result.expect("should find").trust_level, TrustLevel::High);
}

#[test]
fn test_get_by_capability_none_when_missing() {
    let disc = SecureNucleusDiscovery::new();
    assert!(
        disc.get(SelectionCriteria::ByCapability(Capability::Compute))
            .is_none()
    );
}

#[test]
fn test_get_by_node_id() {
    let vp = make_verified("x", "node-alpha", None, TrustLevel::Basic, vec![], vec![]);
    let disc = SecureNucleusDiscovery::new().inject_primal_for_testing(vp);

    let found = disc.get(SelectionCriteria::ByNodeId("node-alpha".into()));
    assert!(found.is_some());
    assert_eq!(found.expect("should find").node_id, "node-alpha");
}

#[test]
fn test_get_by_node_id_not_found() {
    let disc = SecureNucleusDiscovery::new();
    assert!(
        disc.get(SelectionCriteria::ByNodeId("nope".into()))
            .is_none()
    );
}

#[test]
fn test_get_by_family() {
    let vp = make_verified(
        "svc",
        "n1",
        Some("family-east"),
        TrustLevel::High,
        vec![],
        vec![],
    );
    let disc = SecureNucleusDiscovery::new().inject_primal_for_testing(vp);

    let found = disc.get(SelectionCriteria::ByFamily("family-east".into()));
    assert!(found.is_some());
}

#[test]
fn test_get_by_family_no_match() {
    let vp = make_verified("svc", "n1", Some("west"), TrustLevel::Basic, vec![], vec![]);
    let disc = SecureNucleusDiscovery::new().inject_primal_for_testing(vp);

    assert!(
        disc.get(SelectionCriteria::ByFamily("east".into()))
            .is_none()
    );
}

#[test]
fn test_get_by_socket() {
    let sock_path = PathBuf::from("/run/biomeos/beardog.sock");
    let vp = make_verified(
        "beardog",
        "bd",
        None,
        TrustLevel::Basic,
        vec![],
        vec![PrimalEndpoint::UnixSocket {
            path: sock_path.clone(),
        }],
    );
    let disc = SecureNucleusDiscovery::new().inject_primal_for_testing(vp);

    let found = disc.get(SelectionCriteria::BySocket(sock_path));
    assert!(found.is_some());
    assert_eq!(found.expect("should find").name, "beardog");
}

#[test]
fn test_get_by_socket_no_match() {
    let disc = SecureNucleusDiscovery::new();
    assert!(
        disc.get(SelectionCriteria::BySocket(PathBuf::from("/nope")))
            .is_none()
    );
}

#[test]
fn test_get_min_trust_level() {
    let low = make_verified("a", "a1", None, TrustLevel::Basic, vec![], vec![]);
    let high = make_verified("b", "b1", None, TrustLevel::Highest, vec![], vec![]);
    let disc = SecureNucleusDiscovery::new()
        .inject_primal_for_testing(low)
        .inject_primal_for_testing(high);

    let found = disc.get(SelectionCriteria::MinTrustLevel(TrustLevel::High));
    assert!(found.is_some());
    assert_eq!(found.expect("should find").trust_level, TrustLevel::Highest);
}

#[test]
fn test_get_min_trust_level_none_below() {
    let low = make_verified("a", "a1", None, TrustLevel::Basic, vec![], vec![]);
    let disc = SecureNucleusDiscovery::new().inject_primal_for_testing(low);

    assert!(
        disc.get(SelectionCriteria::MinTrustLevel(TrustLevel::High))
            .is_none()
    );
}

#[test]
fn test_get_any() {
    let vp = make_verified("svc", "n", None, TrustLevel::Basic, vec![], vec![]);
    let disc = SecureNucleusDiscovery::new().inject_primal_for_testing(vp);

    assert!(disc.get(SelectionCriteria::Any).is_some());
}

#[test]
fn test_get_any_empty() {
    let disc = SecureNucleusDiscovery::new();
    assert!(disc.get(SelectionCriteria::Any).is_none());
}

#[test]
fn test_with_capability() {
    let vp1 = make_verified(
        "store",
        "s1",
        None,
        TrustLevel::Basic,
        vec![Capability::Storage],
        vec![],
    );
    let vp2 = make_verified(
        "compute",
        "c1",
        None,
        TrustLevel::Basic,
        vec![Capability::Compute],
        vec![],
    );
    let disc = SecureNucleusDiscovery::new()
        .inject_primal_for_testing(vp1)
        .inject_primal_for_testing(vp2);

    let storage_providers = disc.with_capability(&Capability::Storage);
    assert_eq!(storage_providers.len(), 1);
    assert_eq!(storage_providers[0].name, "store");

    let compute_providers = disc.with_capability(&Capability::Compute);
    assert_eq!(compute_providers.len(), 1);

    let discovery_providers = disc.with_capability(&Capability::Discovery);
    assert!(discovery_providers.is_empty());
}

#[test]
fn test_all_empty() {
    let disc = SecureNucleusDiscovery::new();
    assert!(disc.all().is_empty());
}

#[test]
fn test_all_returns_all() {
    let vp1 = make_verified("a", "a1", None, TrustLevel::Basic, vec![], vec![]);
    let vp2 = make_verified("b", "b1", None, TrustLevel::High, vec![], vec![]);
    let disc = SecureNucleusDiscovery::new()
        .inject_primal_for_testing(vp1)
        .inject_primal_for_testing(vp2);

    assert_eq!(disc.all().len(), 2);
}

#[test]
fn test_get_all_unknown_name() {
    let disc = SecureNucleusDiscovery::new();
    assert!(disc.get_all("nonexistent").is_empty());
}

#[tokio::test]
async fn test_discover_secure_requires_clients() {
    let mut disc = SecureNucleusDiscovery::new();
    let result = disc.discover_secure().await;
    assert!(
        result.is_err(),
        "secure discovery without clients must fail"
    );
    let err_msg = format!("{}", result.unwrap_err());
    assert!(
        err_msg.contains("discovery")
            || err_msg.contains("security")
            || err_msg.contains("Songbird")
    );
}
