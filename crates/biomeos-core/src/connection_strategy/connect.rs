// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Multi-tier peer connection orchestration via Neural API `capability.call`.

use tracing::{debug, info, warn};

use crate::atomic_client::AtomicClient;

use super::{ConnectionResult, ConnectionTier, NatType, PeerConnectionInfo, PortPattern};

/// Orchestrate the best connection strategy to reach a peer.
///
/// Uses Neural API `capability.call` for all network operations.
/// biomeOS decides WHAT to do; Songbird/BearDog decide HOW.
///
/// # Arguments
///
/// * `peer_id` - Target peer node ID
/// * `neural_api_socket` - Path to the Neural API Unix socket
/// * `peer_connection_info` - Connection info received from rendezvous (if available)
///
/// # Returns
///
/// The tier and endpoint of the best connection, or an error if all tiers fail.
#[expect(
    clippy::too_many_lines,
    reason = "connection tiers and fallbacks are intentionally sequential in one function"
)]
pub async fn connect_to_peer(
    peer_id: &str,
    neural_api_socket: &str,
    peer_connection_info: Option<&PeerConnectionInfo>,
) -> anyhow::Result<ConnectionResult> {
    let start = std::time::Instant::now();
    let client = AtomicClient::unix(neural_api_socket);
    let mut tiers_attempted = Vec::new();

    // ── Tier 1: LAN Direct ─────────────────────────────────────────────
    info!("🔗 Tier 1: Checking LAN direct connectivity to {}", peer_id);
    tiers_attempted.push(ConnectionTier::LanDirect);

    if let Ok(result) = client
        .call(
            "capability.call",
            serde_json::json!({
                "capability": "mesh",
                "operation": "peers",
                "args": {}
            }),
        )
        .await
        && let Some(peers) = result.get("peers").and_then(|p| p.as_array())
    {
        let found = peers.iter().any(|p| {
            p.get("node_id")
                .and_then(|id| id.as_str())
                .is_some_and(|id| id == peer_id)
        });
        if found
            && let Some(peer) = peers.iter().find(|p| {
                p.get("node_id")
                    .and_then(|id| id.as_str())
                    .is_some_and(|id| id == peer_id)
            })
        {
            let endpoint = peer
                .get("endpoint")
                .and_then(|e| e.as_str())
                .unwrap_or("direct")
                .to_string();

            info!(
                "✅ Tier 1 SUCCESS: {} reachable on LAN at {}",
                peer_id, endpoint
            );
            return Ok(ConnectionResult {
                tier: ConnectionTier::LanDirect,
                endpoint,
                elapsed_ms: start.elapsed().as_millis() as u64,
                tiers_attempted,
            });
        }
    }
    debug!("   Tier 1: {} not found on LAN", peer_id);

    // ── Tier 2/3: NAT-aware strategy ───────────────────────────────────
    let our_nat = detect_nat_type(&client).await;
    info!("🔍 Our NAT type: {:?}", our_nat);

    let peer_nat = peer_connection_info
        .and_then(|info| info.stun_results.as_ref())
        .map_or(NatType::Unknown, |stun| {
            NatType::from_detection(&stun.nat_type)
        });

    debug!("🔍 Peer NAT type: {:?}", peer_nat);

    // ── Tier 2: Direct Punch (both non-symmetric) ──────────────────────
    if our_nat.supports_direct_punch() && peer_nat.supports_direct_punch() {
        info!("🥊 Tier 2: Attempting direct hole punch to {}", peer_id);
        tiers_attempted.push(ConnectionTier::DirectPunch);

        if let Ok(result) = client
            .call(
                "capability.call",
                serde_json::json!({
                    "capability": "punch",
                    "operation": "request",
                    "args": {
                        "peer_id": peer_id
                    }
                }),
            )
            .await
        {
            let success = result
                .get("success")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false);

            if success {
                let endpoint = result
                    .get("endpoint")
                    .and_then(|e| e.as_str())
                    .unwrap_or("punched")
                    .to_string();

                info!(
                    "✅ Tier 2 SUCCESS: Direct punch to {} at {}",
                    peer_id, endpoint
                );
                return Ok(ConnectionResult {
                    tier: ConnectionTier::DirectPunch,
                    endpoint,
                    elapsed_ms: start.elapsed().as_millis() as u64,
                    tiers_attempted,
                });
            }
        }
        debug!("   Tier 2: Direct punch failed");
    } else {
        debug!(
            "   Tier 2: Skipped (our={:?}, peer={:?})",
            our_nat, peer_nat
        );
    }

    // ── Tier 3: Relay-Assisted Coordinated Punch ───────────────────────
    if our_nat.is_symmetric() || peer_nat.is_symmetric() || peer_nat == NatType::Unknown {
        info!(
            "🎯 Tier 3: Attempting relay-assisted coordinated punch to {}",
            peer_id
        );
        tiers_attempted.push(ConnectionTier::CoordinatedPunch);

        let relay_result = client
            .call(
                "capability.call",
                serde_json::json!({
                    "capability": "relay",
                    "operation": "allocate",
                    "args": {
                        "peer_id": peer_id
                    }
                }),
            )
            .await;

        if let Ok(relay) = &relay_result {
            let relay_session_id = relay
                .get("session_id")
                .and_then(|s| s.as_str())
                .unwrap_or("");

            if !relay_session_id.is_empty() {
                let our_pattern = probe_port_pattern(&client).await;

                if our_pattern.is_predictable() {
                    let punch_result = client
                        .call(
                            "capability.call",
                            serde_json::json!({
                                "capability": "punch",
                                "operation": "coordinate",
                                "args": {
                                    "peer_id": peer_id,
                                    "relay_session_id": relay_session_id,
                                    "our_port_pattern": serde_json::to_value(&our_pattern)
                                        .unwrap_or(serde_json::json!(null))
                                }
                            }),
                        )
                        .await;

                    if let Ok(result) = punch_result {
                        let success = result
                            .get("success")
                            .and_then(serde_json::Value::as_bool)
                            .unwrap_or(false);

                        if success {
                            let endpoint = result
                                .get("endpoint")
                                .and_then(|e| e.as_str())
                                .unwrap_or("coordinated-punch")
                                .to_string();

                            info!(
                                "✅ Tier 3 SUCCESS: Coordinated punch to {} at {}",
                                peer_id, endpoint
                            );
                            return Ok(ConnectionResult {
                                tier: ConnectionTier::CoordinatedPunch,
                                endpoint,
                                elapsed_ms: start.elapsed().as_millis() as u64,
                                tiers_attempted,
                            });
                        }
                    }
                    debug!("   Tier 3: Coordinated punch failed, falling through to relay");
                } else {
                    debug!("   Tier 3: Port pattern not predictable, skipping coordinated punch");
                }

                info!(
                    "📡 Tier 4: Using pure relay for {} (session: {})",
                    peer_id, relay_session_id
                );
                tiers_attempted.push(ConnectionTier::PureRelay);
                return Ok(ConnectionResult {
                    tier: ConnectionTier::PureRelay,
                    endpoint: relay_session_id.to_string(),
                    elapsed_ms: start.elapsed().as_millis() as u64,
                    tiers_attempted,
                });
            }
        }
    }

    // ── Tier 4: Pure Relay (final fallback) ────────────────────────────
    info!("📡 Tier 4: Allocating pure relay for {}", peer_id);
    tiers_attempted.push(ConnectionTier::PureRelay);

    let relay_result = client
        .call(
            "capability.call",
            serde_json::json!({
                "capability": "relay",
                "operation": "allocate",
                "args": {
                    "peer_id": peer_id
                }
            }),
        )
        .await?;

    let session_id = relay_result
        .get("session_id")
        .and_then(|s| s.as_str())
        .unwrap_or("relay-fallback")
        .to_string();

    info!(
        "✅ Tier 4: Pure relay active for {} (session: {})",
        peer_id, session_id
    );
    Ok(ConnectionResult {
        tier: ConnectionTier::PureRelay,
        endpoint: session_id,
        elapsed_ms: start.elapsed().as_millis() as u64,
        tiers_attempted,
    })
}

/// Detect our NAT type via Neural API → stun.detect_nat_type
async fn detect_nat_type(client: &AtomicClient) -> NatType {
    match client
        .call(
            "capability.call",
            serde_json::json!({
                "capability": "stun",
                "operation": "detect_nat_type",
                "args": {}
            }),
        )
        .await
    {
        Ok(result) => {
            let nat_str = result
                .get("nat_type")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            NatType::from_detection(nat_str)
        }
        Err(e) => {
            warn!("STUN NAT detection failed: {} — assuming symmetric", e);
            NatType::Symmetric // Conservative: assume worst case
        }
    }
}

/// Probe our port allocation pattern via Neural API → stun.probe_port_pattern
async fn probe_port_pattern(client: &AtomicClient) -> PortPattern {
    match client
        .call(
            "capability.call",
            serde_json::json!({
                "capability": "stun",
                "operation": "probe_port_pattern",
                "args": {
                    "probes": 5
                }
            }),
        )
        .await
    {
        Ok(result) => PortPattern::from_json(&result),
        Err(e) => {
            warn!("Port pattern probing failed: {}", e);
            PortPattern::Unknown
        }
    }
}
