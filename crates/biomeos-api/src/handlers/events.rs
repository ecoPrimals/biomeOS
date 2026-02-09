//! Real-time events via Server-Sent Events (SSE)
//!
//! Provides live updates to clients about ecosystem changes.

use axum::{
    extract::State,
    response::sse::{Event, KeepAlive, Sse},
};
use futures::stream::{self, Stream, StreamExt as FuturesStreamExt};
use serde::Serialize;
use std::{collections::HashMap, convert::Infallible, sync::Arc, time::Duration};
use tokio::sync::RwLock;
use tokio_stream::StreamExt as TokioStreamExt;
use tracing::{info, warn};

use crate::AppState;
use biomeos_core::HealthStatus;

/// Event types that can be streamed
#[derive(Debug, Clone, Serialize)]
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
/// Server-Sent Events endpoint for real-time updates
///
/// Streams ecosystem changes including:
/// - New primal discoveries
/// - Health status changes
/// - Topology updates
/// - Trust level changes
/// - Periodic heartbeats
pub async fn event_stream(
    State(state): State<Arc<AppState>>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    info!("📡 New SSE client connected");

    // Track previous state for change detection
    let previous_state = Arc::new(RwLock::new(EcosystemState {
        primals: HashMap::new(),
    }));

    let stream = FuturesStreamExt::then(
        stream::repeat_with(move || {
            let state = state.clone();
            let previous_state = previous_state.clone();

            async move { detect_and_emit_changes(state, previous_state).await }
        }),
        |fut| fut,
    );

    let stream = FuturesStreamExt::flat_map(stream, stream::iter);
    let stream = TokioStreamExt::throttle(stream, Duration::from_secs(5));
    let stream = FuturesStreamExt::filter_map(stream, |event| async move {
        // Attempt to serialize the event to JSON for SSE
        // If serialization fails (highly unlikely), skip the event and log the error
        match Event::default().json_data(&event) {
            Ok(sse_event) => Some(Ok(sse_event)),
            Err(e) => {
                tracing::error!("Failed to serialize BiomeEvent to SSE: {}", e);
                None // Skip this event
            }
        }
    });

    Sse::new(stream).keep_alive(KeepAlive::default())
}

/// Detect changes in the ecosystem and emit appropriate events
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

        match prev.primals.get(&primal_id) {
            None => {
                // New primal discovered!
                info!("🆕 SSE: New primal discovered: {}", primal.name);
                events.push(EcosystemEvent::PrimalDiscovered {
                    primal_id: primal_id.clone(),
                    name: primal.name.clone(),
                    primal_type: format!("{:?}", primal.primal_type),
                    family_id: primal.family_id.as_ref().map(|f| f.to_string()),
                    capabilities: primal
                        .capabilities
                        .iter()
                        .map(|c| format!("{:?}", c))
                        .collect(),
                });

                // If it has a family, emit family joined event
                if let Some(family_id) = &primal.family_id {
                    events.push(EcosystemEvent::FamilyJoined {
                        primal_id: primal_id.clone(),
                        name: primal.name.clone(),
                        family_id: family_id.to_string(),
                    });
                }
            }
            Some(prev_snapshot) => {
                // Check for health changes
                if prev_snapshot.health != primal.health {
                    info!(
                        "💊 SSE: Health changed for {}: {:?} -> {:?}",
                        primal.name, prev_snapshot.health, primal.health
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
                        primal.name, prev_snapshot.capabilities_count, cap_count
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
                let curr_family = primal.family_id.as_ref().map(|f| f.as_str());

                if prev_family != curr_family {
                    if let Some(family_id) = &primal.family_id {
                        info!("👨‍👩‍👧‍👦 SSE: Family joined for {}: {}", primal.name, family_id);
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
                family_id: primal.family_id.as_ref().map(|f| f.to_string()),
                capabilities_count: primal.capabilities.len(),
            },
        );
    }

    // Check for removed primals
    let current_ids: std::collections::HashSet<_> =
        current_primals.iter().map(|p| p.id.to_string()).collect();

    prev.primals.retain(|id, snapshot| {
        if !current_ids.contains(id) {
            info!("👋 SSE: Primal removed: {}", snapshot.name);
            events.push(EcosystemEvent::TopologyChanged {
                nodes: current_primals.len(),
                edges: 0, // We'd need topology calculation for accurate count
                change: "primal_removed".to_string(),
            });
            false
        } else {
            true
        }
    });

    // Always end with heartbeat
    let healthy_count = current_primals
        .iter()
        .filter(|p| matches!(p.health, HealthStatus::Healthy))
        .count();

    let families: Vec<String> = current_primals
        .iter()
        .filter_map(|p| p.family_id.as_ref().map(|f| f.to_string()))
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
            timestamp: 1234567890,
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
        let debug_str = format!("{:?}", event);
        assert!(debug_str.contains("Heartbeat"));
    }

    #[test]
    fn test_ecosystem_event_clone() {
        let event = EcosystemEvent::TrustUpdated {
            primal_id: "test".to_string(),
            name: "Test".to_string(),
            trust_level: 2,
        };
        let cloned = event.clone();
        let json1 = serde_json::to_string(&event).unwrap();
        let json2 = serde_json::to_string(&cloned).unwrap();
        assert_eq!(json1, json2);
    }

    #[test]
    fn test_current_timestamp_returns_reasonable_value() {
        let ts = current_timestamp();
        // Should be after 2020 (timestamp > 1577836800)
        assert!(ts > 1577836800, "Timestamp should be after 2020");
        // Should be before 2050 (timestamp < 2524608000)
        assert!(ts < 2524608000, "Timestamp should be before 2050");
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
        let cloned = snapshot.clone();
        assert_eq!(cloned.name, "test");
        assert_eq!(cloned.capabilities_count, 5);
    }
}
