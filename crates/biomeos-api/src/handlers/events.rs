// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Real-time events via Server-Sent Events (SSE)
//!
//! Provides live updates to clients about ecosystem changes.

use axum::{
    extract::State,
    response::sse::{Event, KeepAlive, Sse},
};
use futures::stream::{self, Stream, StreamExt as FuturesStreamExt};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, convert::Infallible, sync::Arc, time::Duration};
use tokio::sync::RwLock;
use tokio_stream::StreamExt as TokioStreamExt;
use tracing::{info, warn};

use crate::AppState;
use biomeos_core::HealthStatus;

/// Event types that can be streamed
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EcosystemEvent {
    /// A new primal was discovered
    PrimalDiscovered {
        primal_id: String,
        name: String,
        primal_type: String,
        family_id: Option<String>,
        capabilities: Vec<String>,
    },

    /// A primal's health changed
    HealthChanged {
        primal_id: String,
        name: String,
        old_health: String,
        new_health: String,
    },

    /// Topology changed (new connection)
    TopologyChanged {
        nodes: usize,
        edges: usize,
        change: String, // "primal_added", "primal_removed", "edge_added"
    },

    /// Family relationship established
    FamilyJoined {
        primal_id: String,
        name: String,
        family_id: String,
    },

    /// Trust level updated
    TrustUpdated {
        primal_id: String,
        name: String,
        trust_level: u8,
    },

    /// Periodic heartbeat with full state
    Heartbeat {
        timestamp: u64,
        primals_count: usize,
        healthy_count: usize,
        families: Vec<String>,
    },
}

/// Tracks ecosystem state for change detection
#[derive(Debug, Clone)]
struct EcosystemState {
    primals: HashMap<String, PrimalSnapshot>,
}

#[derive(Debug, Clone)]
struct PrimalSnapshot {
    name: String,
    health: HealthStatus,
    family_id: Option<String>,
    capabilities_count: usize,
}

/// GET /api/v1/events/stream
///
/// Server-Sent Events endpoint for real-time updates (PUSH-BASED).
///
/// Two event sources merged into one stream:
/// 1. **Graph events** — pushed instantly from `GraphEventBroadcaster`
///    (tick completions, session state changes, node events)
/// 2. **Ecosystem events** — discovery-based change detection at 5s intervals
///    (primal discovered, health changed, topology, heartbeat)
pub async fn event_stream(
    State(state): State<Arc<AppState>>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    info!("📡 New SSE client connected (push-based)");

    // --- Stream 1: Push-based graph events from broadcaster ---
    let mut graph_rx = state.event_broadcaster().subscribe();
    let graph_stream = async_stream::stream! {
        loop {
            match graph_rx.recv().await {
                Ok(event) => {
                    yield event;
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                    warn!("SSE client lagged, skipped {} graph events", n);
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                    break;
                }
            }
        }
    };
    let graph_sse = FuturesStreamExt::filter_map(graph_stream, |event| async move {
        match Event::default().json_data(&event) {
            Ok(sse_event) => Some(Ok::<_, Infallible>(sse_event)),
            Err(e) => {
                tracing::error!("Failed to serialize GraphEvent to SSE: {}", e);
                None
            }
        }
    });

    // --- Stream 2: Ecosystem change detection (retained for backward compat) ---
    let previous_state = Arc::new(RwLock::new(EcosystemState {
        primals: HashMap::new(),
    }));

    let eco_stream = FuturesStreamExt::then(
        stream::repeat_with(move || {
            let state = state.clone();
            let previous_state = previous_state.clone();
            async move { detect_and_emit_changes(state, previous_state).await }
        }),
        |fut| fut,
    );
    let eco_stream = FuturesStreamExt::flat_map(eco_stream, stream::iter);
    let eco_stream = TokioStreamExt::throttle(eco_stream, Duration::from_secs(5));
    let eco_sse = FuturesStreamExt::filter_map(eco_stream, |event| async move {
        match Event::default().json_data(&event) {
            Ok(sse_event) => Some(Ok::<_, Infallible>(sse_event)),
            Err(e) => {
                tracing::error!("Failed to serialize EcosystemEvent to SSE: {}", e);
                None
            }
        }
    });

    // --- Merge both streams: graph events arrive instantly, ecosystem polls at 5s ---
    let merged = futures::stream::select(graph_sse, eco_sse);

    Sse::new(merged).keep_alive(KeepAlive::default())
}

/// Detect changes in the ecosystem and emit appropriate events
#[expect(clippy::too_many_lines, reason = "change detection logic")]
async fn detect_and_emit_changes(
    state: Arc<AppState>,
    previous_state: Arc<RwLock<EcosystemState>>,
) -> Vec<EcosystemEvent> {
    let mut events = Vec::new();

    // Discover current primals
    let current_primals = match state.discovery().discover_all().await {
        Ok(primals) => primals,
        Err(e) => {
            warn!("Failed to discover primals for SSE: {}", e);
            return vec![EcosystemEvent::Heartbeat {
                timestamp: current_timestamp(),
                primals_count: 0,
                healthy_count: 0,
                families: vec![],
            }];
        }
    };

    // Lock previous state
    let mut prev = previous_state.write().await;

    // Detect changes
    for primal in &current_primals {
        let primal_id = primal.id.to_string();
        let name = primal.name.as_str();

        match prev.primals.get(&primal_id) {
            None => {
                // New primal discovered!
                info!("🆕 SSE: New primal discovered: {}", name);
                events.push(EcosystemEvent::PrimalDiscovered {
                    primal_id: primal_id.clone(),
                    name: primal.name.clone(),
                    primal_type: format!("{:?}", primal.primal_type),
                    family_id: primal
                        .family_id
                        .as_ref()
                        .map(std::string::ToString::to_string),
                    capabilities: primal
                        .capabilities
                        .iter()
                        .map(|c| format!("{c:?}"))
                        .collect(),
                });

                // If it has a family, emit family joined event
                if let Some(family_id) = &primal.family_id {
                    events.push(EcosystemEvent::FamilyJoined {
                        primal_id: primal_id.clone(),
                        name: primal.name.clone(),
                        family_id: family_id.as_str().to_string(),
                    });
                }
            }
            Some(prev_snapshot) => {
                // Check for health changes
                if prev_snapshot.health != primal.health {
                    info!(
                        "💊 SSE: Health changed for {}: {:?} -> {:?}",
                        name, prev_snapshot.health, primal.health
                    );
                    events.push(EcosystemEvent::HealthChanged {
                        primal_id: primal_id.clone(),
                        name: primal.name.clone(),
                        old_health: format!("{:?}", prev_snapshot.health),
                        new_health: format!("{:?}", primal.health),
                    });
                }

                // Check for capability changes (could indicate trust changes)
                let cap_count = primal.capabilities.len();
                if prev_snapshot.capabilities_count != cap_count {
                    info!(
                        "🔒 SSE: Capabilities updated for {}: {} -> {}",
                        name, prev_snapshot.capabilities_count, cap_count
                    );
                    // Emit as trust update (more capabilities = higher trust)
                    events.push(EcosystemEvent::TrustUpdated {
                        primal_id: primal_id.clone(),
                        name: primal.name.clone(),
                        trust_level: cap_count as u8,
                    });
                }

                // Check for family changes
                let prev_family = prev_snapshot.family_id.as_deref();
                let curr_family = primal
                    .family_id
                    .as_ref()
                    .map(biomeos_types::FamilyId::as_str);

                if prev_family != curr_family {
                    if let Some(family_id) = &primal.family_id {
                        info!("👨‍👩‍👧‍👦 SSE: Family joined for {}: {}", name, family_id);
                        events.push(EcosystemEvent::FamilyJoined {
                            primal_id: primal_id.clone(),
                            name: primal.name.clone(),
                            family_id: family_id.to_string(),
                        });
                    }
                }
            }
        }

        // Update snapshot
        prev.primals.insert(
            primal_id,
            PrimalSnapshot {
                name: primal.name.clone(),
                health: primal.health,
                family_id: primal
                    .family_id
                    .as_ref()
                    .map(std::string::ToString::to_string),
                capabilities_count: primal.capabilities.len(),
            },
        );
    }

    // Check for removed primals
    let current_ids: std::collections::HashSet<_> =
        current_primals.iter().map(|p| p.id.to_string()).collect();

    prev.primals.retain(|id, snapshot| {
        if current_ids.contains(id) {
            true
        } else {
            info!("👋 SSE: Primal removed: {}", snapshot.name);
            events.push(EcosystemEvent::TopologyChanged {
                nodes: current_primals.len(),
                edges: 0, // We'd need topology calculation for accurate count
                change: "primal_removed".to_string(),
            });
            false
        }
    });

    // Always end with heartbeat
    let healthy_count = current_primals
        .iter()
        .filter(|p| matches!(p.health, HealthStatus::Healthy))
        .count();

    let families: Vec<String> = current_primals
        .iter()
        .filter_map(|p| p.family_id.as_ref().map(std::string::ToString::to_string))
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    events.push(EcosystemEvent::Heartbeat {
        timestamp: current_timestamp(),
        primals_count: current_primals.len(),
        healthy_count,
        families,
    });

    events
}

/// Get current Unix timestamp
fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or(std::time::Duration::from_secs(0)) // Safe fallback: epoch time
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use biomeos_core::{DiscoveryError, DiscoveryResult, PrimalDiscovery, PrimalType};
    use biomeos_types::{Endpoint, FamilyId, PrimalId};
    use semver::Version;
    use std::sync::Arc;

    struct MockDiscovery {
        primals: Vec<biomeos_core::DiscoveredPrimal>,
    }

    #[async_trait]
    impl PrimalDiscovery for MockDiscovery {
        async fn discover(
            &self,
            _endpoint: &biomeos_types::Endpoint,
        ) -> DiscoveryResult<biomeos_core::DiscoveredPrimal> {
            Err(DiscoveryError::NotFound {
                endpoint: "mock".to_string(),
            })
        }

        async fn discover_all(&self) -> DiscoveryResult<Vec<biomeos_core::DiscoveredPrimal>> {
            Ok(self.primals.clone())
        }

        async fn check_health(
            &self,
            _id: &PrimalId,
        ) -> DiscoveryResult<biomeos_core::HealthStatus> {
            Ok(biomeos_core::HealthStatus::Healthy)
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
        #[async_trait]
        impl PrimalDiscovery for FailingDiscovery {
            async fn discover(
                &self,
                _: &biomeos_types::Endpoint,
            ) -> DiscoveryResult<biomeos_core::DiscoveredPrimal> {
                Err(DiscoveryError::NotFound {
                    endpoint: "fail".to_string(),
                })
            }
            async fn discover_all(&self) -> DiscoveryResult<Vec<biomeos_core::DiscoveredPrimal>> {
                Err(DiscoveryError::Network("simulated failure".to_string()))
            }
            async fn check_health(&self, _: &PrimalId) -> DiscoveryResult<HealthStatus> {
                Ok(HealthStatus::Healthy)
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
}
