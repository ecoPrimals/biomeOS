// SPDX-License-Identifier: AGPL-3.0-or-later
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
use std::{
    collections::{HashMap, HashSet},
    convert::Infallible,
    sync::Arc,
    time::Duration,
};
use tokio::sync::RwLock;
use tokio_stream::StreamExt as TokioStreamExt;
use tracing::{info, warn};

use crate::AppState;
use biomeos_core::{DiscoveredPrimal, HealthStatus};

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

/// Build a snapshot of the current primal state for the next change-detection pass.
fn build_primal_snapshot(primal: &DiscoveredPrimal, name: String) -> PrimalSnapshot {
    PrimalSnapshot {
        name,
        health: primal.health,
        family_id: primal
            .family_id
            .as_ref()
            .map(std::string::ToString::to_string),
        capabilities_count: primal.capabilities.len(),
    }
}

/// Events for a primal that was not in the previous map (discovery).
fn detect_new_primal_events(
    primal: &DiscoveredPrimal,
    primal_id: &str,
    name: &str,
) -> Vec<EcosystemEvent> {
    let mut out = Vec::new();
    info!("🆕 SSE: New primal discovered: {}", name);
    out.push(EcosystemEvent::PrimalDiscovered {
        primal_id: primal_id.to_string(),
        name: name.to_string(),
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

    if let Some(family_id) = &primal.family_id {
        out.push(EcosystemEvent::FamilyJoined {
            primal_id: primal_id.to_string(),
            name: name.to_string(),
            family_id: family_id.as_str().to_string(),
        });
    }
    out
}

/// Health, capability, and family delta events for a known primal.
fn detect_primal_update_events(
    primal: &DiscoveredPrimal,
    prev_snapshot: &PrimalSnapshot,
    primal_id: &str,
    name: &str,
) -> Vec<EcosystemEvent> {
    let mut out = Vec::new();

    if prev_snapshot.health != primal.health {
        info!(
            "💊 SSE: Health changed for {}: {:?} -> {:?}",
            name, prev_snapshot.health, primal.health
        );
        out.push(EcosystemEvent::HealthChanged {
            primal_id: primal_id.to_string(),
            name: name.to_string(),
            old_health: format!("{:?}", prev_snapshot.health),
            new_health: format!("{:?}", primal.health),
        });
    }

    let cap_count = primal.capabilities.len();
    if prev_snapshot.capabilities_count != cap_count {
        info!(
            "🔒 SSE: Capabilities updated for {}: {} -> {}",
            name, prev_snapshot.capabilities_count, cap_count
        );
        out.push(EcosystemEvent::TrustUpdated {
            primal_id: primal_id.to_string(),
            name: name.to_string(),
            trust_level: cap_count as u8,
        });
    }

    let prev_family = prev_snapshot.family_id.as_deref();
    let curr_family = primal
        .family_id
        .as_ref()
        .map(biomeos_types::FamilyId::as_str);

    if prev_family != curr_family {
        if let Some(family_id) = &primal.family_id {
            info!("👨‍👩‍👧‍👦 SSE: Family joined for {}: {}", name, family_id);
            out.push(EcosystemEvent::FamilyJoined {
                primal_id: primal_id.to_string(),
                name: name.to_string(),
                family_id: family_id.to_string(),
            });
        }
    }
    out
}

/// Primals present in the previous map but not in the current discovery set.
fn detect_primal_removals(
    prev: &mut HashMap<String, PrimalSnapshot>,
    current_ids: &HashSet<String>,
    current_primals_len: usize,
) -> Vec<EcosystemEvent> {
    let mut out = Vec::new();
    prev.retain(|id, snapshot| {
        if current_ids.contains(id) {
            true
        } else {
            info!("👋 SSE: Primal removed: {}", snapshot.name);
            out.push(EcosystemEvent::TopologyChanged {
                nodes: current_primals_len,
                edges: 0, // We'd need topology calculation for accurate count
                change: "primal_removed".to_string(),
            });
            false
        }
    });
    out
}

/// Terminal heartbeat for a successful discovery result.
fn detect_heartbeat_event(current_primals: &[DiscoveredPrimal]) -> EcosystemEvent {
    let healthy_count = current_primals
        .iter()
        .filter(|p| matches!(p.health, HealthStatus::Healthy))
        .count();
    let families: Vec<String> = current_primals
        .iter()
        .filter_map(|p| p.family_id.as_ref().map(std::string::ToString::to_string))
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    EcosystemEvent::Heartbeat {
        timestamp: current_timestamp(),
        primals_count: current_primals.len(),
        healthy_count,
        families,
    }
}

/// Detect changes in the ecosystem and emit appropriate events
async fn detect_and_emit_changes(
    state: Arc<AppState>,
    previous_state: Arc<RwLock<EcosystemState>>,
) -> Vec<EcosystemEvent> {
    let mut events = Vec::new();

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

    let mut prev = previous_state.write().await;

    for primal in &current_primals {
        let primal_id = primal.id.to_string();
        let name = primal.name.clone();

        match prev.primals.get(&primal_id) {
            None => {
                events.extend(detect_new_primal_events(
                    primal,
                    primal_id.as_str(),
                    name.as_str(),
                ));
            }
            Some(prev_snapshot) => {
                events.extend(detect_primal_update_events(
                    primal,
                    prev_snapshot,
                    primal_id.as_str(),
                    name.as_str(),
                ));
            }
        }

        let snapshot = build_primal_snapshot(primal, name);
        prev.primals.insert(primal_id, snapshot);
    }

    let current_ids: HashSet<String> = current_primals.iter().map(|p| p.id.to_string()).collect();
    events.extend(detect_primal_removals(
        &mut prev.primals,
        &current_ids,
        current_primals.len(),
    ));

    events.push(detect_heartbeat_event(&current_primals));

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
#[expect(
    clippy::unwrap_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
#[expect(
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
#[path = "events_tests.rs"]
mod tests;
