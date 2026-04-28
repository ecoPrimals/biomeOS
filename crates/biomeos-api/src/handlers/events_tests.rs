// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::*;
use biomeos_core::{DiscoveryError, DiscoveryResult, PrimalDiscovery, PrimalType};
use biomeos_types::{Endpoint, FamilyId, PrimalId};
use semver::Version;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

struct MockDiscovery {
    primals: Vec<biomeos_core::DiscoveredPrimal>,
}

impl PrimalDiscovery for MockDiscovery {
    fn discover(
        &self,
        _endpoint: &biomeos_types::Endpoint,
    ) -> Pin<Box<dyn Future<Output = DiscoveryResult<biomeos_core::DiscoveredPrimal>> + Send + '_>>
    {
        Box::pin(async move {
            Err(DiscoveryError::NotFound {
                endpoint: "mock".to_string(),
            })
        })
    }

    fn discover_all(
        &self,
    ) -> Pin<
        Box<dyn Future<Output = DiscoveryResult<Vec<biomeos_core::DiscoveredPrimal>>> + Send + '_>,
    > {
        let primals = self.primals.clone();
        Box::pin(async move { Ok(primals) })
    }

    fn check_health(
        &self,
        _id: &PrimalId,
    ) -> Pin<Box<dyn Future<Output = DiscoveryResult<biomeos_core::HealthStatus>> + Send + '_>>
    {
        Box::pin(async move { Ok(biomeos_core::HealthStatus::Healthy) })
    }
}

fn make_primal(
    id: &str,
    name: &str,
    health: biomeos_core::HealthStatus,
    family_id: Option<&str>,
    capabilities: Vec<&str>,
) -> biomeos_core::DiscoveredPrimal {
    biomeos_core::DiscoveredPrimal {
        id: PrimalId::new_unchecked(id),
        name: name.to_string(),
        primal_type: PrimalType::Security,
        version: Version::new(1, 0, 0),
        health,
        capabilities: capabilities.into_iter().map(Into::into).collect(),
        endpoint: Endpoint::new("http://localhost:9000").expect("valid endpoint"),
        family_id: family_id.map(FamilyId::new),
        metadata: serde_json::json!({}),
    }
}

#[test]
fn test_primal_discovered_event() {
    let event = EcosystemEvent::PrimalDiscovered {
        primal_id: "test-primal".to_string(),
        name: "Test".to_string(),
        primal_type: "security".to_string(),
        family_id: Some("test-family".to_string()),
        capabilities: vec!["btsp".to_string(), "birdsong".to_string()],
    };

    let json = serde_json::to_string(&event).unwrap();
    assert!(json.contains("primal_discovered"));
    assert!(json.contains("test-primal"));
    assert!(json.contains("test-family"));
    assert!(json.contains("btsp"));
}

#[test]
fn test_health_changed_event() {
    let event = EcosystemEvent::HealthChanged {
        primal_id: "test-primal".to_string(),
        name: "Test".to_string(),
        old_health: "healthy".to_string(),
        new_health: "degraded".to_string(),
    };

    let json = serde_json::to_string(&event).unwrap();
    assert!(json.contains("health_changed"));
    assert!(json.contains("healthy"));
    assert!(json.contains("degraded"));
}

#[test]
fn test_heartbeat_event() {
    let event = EcosystemEvent::Heartbeat {
        timestamp: 1_234_567_890,
        primals_count: 5,
        healthy_count: 4,
        families: vec!["iidn".to_string(), "test-family".to_string()],
    };

    let json = serde_json::to_string(&event).unwrap();
    assert!(json.contains("heartbeat"));
    assert!(json.contains("1234567890"));
    assert!(json.contains("\"primals_count\":5"));
    assert!(json.contains("\"healthy_count\":4"));
    assert!(json.contains("iidn"));
}

#[test]
fn test_family_joined_event() {
    let event = EcosystemEvent::FamilyJoined {
        primal_id: "test-primal".to_string(),
        name: "Test".to_string(),
        family_id: "iidn".to_string(),
    };

    let json = serde_json::to_string(&event).unwrap();
    assert!(json.contains("family_joined"));
    assert!(json.contains("iidn"));
}

#[test]
fn test_trust_updated_event() {
    let event = EcosystemEvent::TrustUpdated {
        primal_id: "test-primal".to_string(),
        name: "Test".to_string(),
        trust_level: 3,
    };

    let json = serde_json::to_string(&event).unwrap();
    assert!(json.contains("trust_updated"));
    assert!(json.contains("\"trust_level\":3"));
}

#[test]
fn test_topology_changed_event() {
    let event = EcosystemEvent::TopologyChanged {
        nodes: 5,
        edges: 3,
        change: "primal_added".to_string(),
    };

    let json = serde_json::to_string(&event).unwrap();
    assert!(json.contains("topology_changed"));
    assert!(json.contains("\"nodes\":5"));
    assert!(json.contains("primal_added"));
}

#[test]
fn test_ecosystem_event_debug() {
    let event = EcosystemEvent::Heartbeat {
        timestamp: 0,
        primals_count: 0,
        healthy_count: 0,
        families: vec![],
    };
    let debug_str = format!("{event:?}");
    assert!(debug_str.contains("Heartbeat"));
}

#[test]
fn test_ecosystem_event_clone() {
    let event = EcosystemEvent::TrustUpdated {
        primal_id: "test".to_string(),
        name: "Test".to_string(),
        trust_level: 2,
    };
    let json1 = serde_json::to_string(&event).unwrap();
    let json2 = serde_json::to_string(&event).unwrap();
    assert_eq!(json1, json2);
}

#[test]
fn test_current_timestamp_returns_reasonable_value() {
    let ts = current_timestamp();
    // Should be after 2020 (timestamp > 1577836800)
    assert!(ts > 1_577_836_800, "Timestamp should be after 2020");
    // Should be before 2050 (timestamp < 2524608000)
    assert!(ts < 2_524_608_000, "Timestamp should be before 2050");
}

#[test]
fn test_primal_discovered_without_family() {
    let event = EcosystemEvent::PrimalDiscovered {
        primal_id: "orphan".to_string(),
        name: "Orphan".to_string(),
        primal_type: "unknown".to_string(),
        family_id: None,
        capabilities: vec![],
    };

    let json = serde_json::to_string(&event).unwrap();
    assert!(json.contains("orphan"));
    assert!(json.contains("\"family_id\":null"));
}

#[test]
fn test_ecosystem_state_default() {
    let state = EcosystemState {
        primals: HashMap::new(),
    };
    assert!(state.primals.is_empty());
}

#[test]
fn test_primal_snapshot_clone() {
    let snapshot = PrimalSnapshot {
        name: "test".to_string(),
        health: HealthStatus::Healthy,
        family_id: Some("fam".to_string()),
        capabilities_count: 5,
    };
    assert_eq!(snapshot.name, "test");
    assert_eq!(snapshot.capabilities_count, 5);
}

// ========== Serialization roundtrip tests ==========

#[test]
fn test_ecosystem_event_roundtrip_primal_discovered() {
    let event = EcosystemEvent::PrimalDiscovered {
        primal_id: "p1".to_string(),
        name: "Primal1".to_string(),
        primal_type: "security".to_string(),
        family_id: Some("fam1".to_string()),
        capabilities: vec!["btsp".to_string(), "birdsong".to_string()],
    };
    let json = serde_json::to_string(&event).expect("serialize");
    let back: EcosystemEvent = serde_json::from_str(&json).expect("deserialize");
    match (&event, &back) {
        (
            EcosystemEvent::PrimalDiscovered { primal_id: a, .. },
            EcosystemEvent::PrimalDiscovered { primal_id: b, .. },
        ) => assert_eq!(a, b),
        _ => panic!("variant mismatch"),
    }
}

#[test]
fn test_ecosystem_event_roundtrip_heartbeat() {
    let event = EcosystemEvent::Heartbeat {
        timestamp: 999,
        primals_count: 0,
        healthy_count: 0,
        families: vec![],
    };
    let json = serde_json::to_string(&event).expect("serialize");
    let back: EcosystemEvent = serde_json::from_str(&json).expect("deserialize");
    match &back {
        EcosystemEvent::Heartbeat {
            timestamp,
            primals_count,
            healthy_count,
            families,
        } => {
            assert_eq!(*timestamp, 999);
            assert_eq!(*primals_count, 0);
            assert_eq!(*healthy_count, 0);
            assert!(families.is_empty());
        }
        _ => panic!("expected Heartbeat"),
    }
}

#[test]
fn test_ecosystem_event_empty_capabilities() {
    let event = EcosystemEvent::PrimalDiscovered {
        primal_id: "empty-cap".to_string(),
        name: "Empty".to_string(),
        primal_type: "unknown".to_string(),
        family_id: None,
        capabilities: vec![],
    };
    let json = serde_json::to_string(&event).expect("serialize");
    let back: EcosystemEvent = serde_json::from_str(&json).expect("deserialize");
    match &back {
        EcosystemEvent::PrimalDiscovered { capabilities, .. } => {
            assert!(capabilities.is_empty());
        }
        _ => panic!("expected PrimalDiscovered"),
    }
}

// ========== detect_and_emit_changes with mock state ==========

#[tokio::test]
async fn test_detect_and_emit_changes_new_primal() {
    let primals = vec![make_primal(
        "beardog-1",
        "BearDog",
        HealthStatus::Healthy,
        Some("fam1"),
        vec!["btsp"],
    )];
    let discovery = MockDiscovery { primals };
    let state = Arc::new(
        crate::AppState::builder()
            .discovery(discovery)
            .build_with_defaults()
            .expect("build state"),
    );
    let previous_state = Arc::new(RwLock::new(EcosystemState {
        primals: HashMap::new(),
    }));

    let events = detect_and_emit_changes(state, previous_state).await;

    // Should have PrimalDiscovered, FamilyJoined, Heartbeat
    assert!(!events.is_empty(), "should emit events");
    let has_primal_discovered = events
        .iter()
        .any(|e| matches!(e, EcosystemEvent::PrimalDiscovered { .. }));
    assert!(has_primal_discovered, "should emit PrimalDiscovered");
    let has_heartbeat = events
        .iter()
        .any(|e| matches!(e, EcosystemEvent::Heartbeat { .. }));
    assert!(has_heartbeat, "should end with Heartbeat");
}

#[tokio::test]
async fn test_detect_and_emit_changes_discovery_error_returns_heartbeat() {
    struct FailingDiscovery;
    impl PrimalDiscovery for FailingDiscovery {
        fn discover(
            &self,
            _: &biomeos_types::Endpoint,
        ) -> Pin<
            Box<dyn Future<Output = DiscoveryResult<biomeos_core::DiscoveredPrimal>> + Send + '_>,
        > {
            Box::pin(async move {
                Err(DiscoveryError::NotFound {
                    endpoint: "fail".to_string(),
                })
            })
        }
        fn discover_all(
            &self,
        ) -> Pin<
            Box<
                dyn Future<Output = DiscoveryResult<Vec<biomeos_core::DiscoveredPrimal>>>
                    + Send
                    + '_,
            >,
        > {
            Box::pin(async move { Err(DiscoveryError::Network("simulated failure".to_string())) })
        }
        fn check_health(
            &self,
            _: &PrimalId,
        ) -> Pin<Box<dyn Future<Output = DiscoveryResult<HealthStatus>> + Send + '_>> {
            Box::pin(async move { Ok(HealthStatus::Healthy) })
        }
    }
    let state = Arc::new(
        crate::AppState::builder()
            .discovery(FailingDiscovery)
            .build_with_defaults()
            .expect("build state"),
    );
    let previous_state = Arc::new(RwLock::new(EcosystemState {
        primals: HashMap::new(),
    }));

    let events = detect_and_emit_changes(state, previous_state).await;

    // On discovery error, should return single Heartbeat with zeros
    assert_eq!(events.len(), 1, "should return single heartbeat on error");
    match &events[0] {
        EcosystemEvent::Heartbeat {
            primals_count,
            healthy_count,
            families,
            ..
        } => {
            assert_eq!(*primals_count, 0);
            assert_eq!(*healthy_count, 0);
            assert!(families.is_empty());
        }
        _ => panic!("expected Heartbeat on discovery error"),
    }
}

#[tokio::test]
async fn test_detect_and_emit_changes_health_change() {
    let primals = vec![
        make_primal("p1", "P1", HealthStatus::Healthy, Some("fam"), vec!["cap1"]),
        make_primal(
            "p2",
            "P2",
            HealthStatus::Degraded,
            Some("fam"),
            vec!["cap1", "cap2"],
        ),
    ];
    let discovery = MockDiscovery { primals };
    let state = Arc::new(
        crate::AppState::builder()
            .discovery(discovery)
            .build_with_defaults()
            .expect("build state"),
    );
    let mut initial_primals = HashMap::new();
    initial_primals.insert(
        "p1".to_string(),
        PrimalSnapshot {
            name: "P1".to_string(),
            health: HealthStatus::Healthy,
            family_id: Some("fam".to_string()),
            capabilities_count: 1,
        },
    );
    initial_primals.insert(
        "p2".to_string(),
        PrimalSnapshot {
            name: "P2".to_string(),
            health: HealthStatus::Healthy,
            family_id: Some("fam".to_string()),
            capabilities_count: 1,
        },
    );
    let previous_state = Arc::new(RwLock::new(EcosystemState {
        primals: initial_primals,
    }));

    let events = detect_and_emit_changes(state, previous_state).await;

    let health_changed = events
        .iter()
        .find(|e| matches!(e, EcosystemEvent::HealthChanged { .. }));
    assert!(health_changed.is_some(), "should emit HealthChanged for p2");
    let trust_updated = events
        .iter()
        .find(|e| matches!(e, EcosystemEvent::TrustUpdated { .. }));
    assert!(
        trust_updated.is_some(),
        "should emit TrustUpdated for capability change"
    );
}
