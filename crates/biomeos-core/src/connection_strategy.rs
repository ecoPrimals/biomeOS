//! Connection Strategy Orchestrator
//!
//! Selects the optimal connection tier for reaching a peer, using the Neural API
//! for capability routing. This is the biomeOS-owned "brain" that decides WHAT to
//! do — primals (Songbird, BearDog) decide HOW.
//!
//! ## Multi-Tier Strategy
//!
//! ```text
//! Tier 1: LAN Direct      — mesh.auto_discover → direct socket/TCP
//! Tier 2: Direct Punch    — Both non-symmetric NAT → punch.request
//! Tier 3: Coordinated     — Either symmetric NAT → relay + stun.probe_port_pattern
//!         Punch              → punch.coordinate with predicted ports
//! Tier 4: Pure Relay      — Random NAT / punch fails → relay.allocate (always works)
//! ```
//!
//! ## Ownership Boundary
//!
//! biomeOS decides which tier to attempt. It calls Neural API `capability.call`
//! for each step. The actual UDP/STUN/relay operations are performed by Songbird.
//! Cryptographic authorization is performed by BearDog.
//!
//! See: `docs/handoffs/RELAY_ASSISTED_COORDINATED_PUNCH_HANDOFF_FEB11_2026.md`

use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

use crate::atomic_client::AtomicClient;

// ============================================================================
// Types
// ============================================================================

/// Result of a connection strategy decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionResult {
    /// Which tier succeeded
    pub tier: ConnectionTier,
    /// The connection endpoint (socket path, address, relay session ID)
    pub endpoint: String,
    /// How long the strategy took (ms)
    pub elapsed_ms: u64,
    /// Tiers that were attempted before success
    pub tiers_attempted: Vec<ConnectionTier>,
}

/// Connection tier in priority order
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionTier {
    /// Tier 1: Direct LAN connection (no NAT traversal needed)
    LanDirect,
    /// Tier 2: Direct UDP hole punch (both peers have predictable NAT)
    DirectPunch,
    /// Tier 3: Relay-assisted coordinated punch (symmetric NAT, port prediction)
    CoordinatedPunch,
    /// Tier 4: Pure relay (always works, higher latency)
    PureRelay,
}

impl std::fmt::Display for ConnectionTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LanDirect => write!(f, "LAN Direct"),
            Self::DirectPunch => write!(f, "Direct Punch"),
            Self::CoordinatedPunch => write!(f, "Coordinated Punch"),
            Self::PureRelay => write!(f, "Pure Relay"),
        }
    }
}

/// NAT type as detected by STUN
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NatType {
    /// No NAT (public IP)
    None,
    /// Full cone NAT (most permissive)
    FullCone,
    /// Address-restricted cone NAT
    AddressRestricted,
    /// Port-restricted cone NAT
    PortRestricted,
    /// Symmetric NAT (hardest to traverse)
    Symmetric,
    /// Could not determine
    Unknown,
}

impl NatType {
    /// Whether this NAT type is symmetric (requiring relay-assisted punch)
    pub fn is_symmetric(&self) -> bool {
        matches!(self, Self::Symmetric)
    }

    /// Whether direct hole punching is likely to succeed
    pub fn supports_direct_punch(&self) -> bool {
        matches!(
            self,
            Self::None | Self::FullCone | Self::AddressRestricted | Self::PortRestricted
        )
    }

    /// Parse from a STUN detection result string
    pub fn from_detection(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "none" | "public" | "open" => Self::None,
            "full_cone" | "full-cone" | "fullcone" => Self::FullCone,
            "address_restricted" | "address-restricted" => Self::AddressRestricted,
            "port_restricted" | "port-restricted" => Self::PortRestricted,
            "symmetric" => Self::Symmetric,
            _ => Self::Unknown,
        }
    }
}

/// Port allocation pattern detected by multi-probe STUN
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PortPattern {
    /// Ports allocated sequentially (step between consecutive allocations)
    Sequential {
        /// Step between consecutive port allocations
        step: i32,
        /// Last observed port
        last_port: u16,
        /// Predicted next port
        predicted_next: u16,
        /// Confidence in prediction (0.0 - 1.0)
        confidence: f64,
    },
    /// Random port allocation (unpredictable)
    Random {
        /// Observed ports
        observed: Vec<u16>,
    },
    /// Could not determine pattern
    Unknown,
}

impl PortPattern {
    /// Whether the pattern is predictable enough for coordinated punch
    pub fn is_predictable(&self) -> bool {
        matches!(
            self,
            Self::Sequential { confidence, .. } if *confidence >= 0.6
        )
    }

    /// Parse from a JSON value returned by `stun.probe_port_pattern`
    pub fn from_json(value: &serde_json::Value) -> Self {
        if let Some(pattern_type) = value.get("type").and_then(|v| v.as_str()) {
            match pattern_type {
                "sequential" => {
                    let step = value.get("step").and_then(|v| v.as_i64()).unwrap_or(1) as i32;
                    let last_port =
                        value.get("last_port").and_then(|v| v.as_u64()).unwrap_or(0) as u16;
                    let predicted_next = value
                        .get("predicted_next")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0) as u16;
                    let confidence = value
                        .get("confidence")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0);
                    Self::Sequential {
                        step,
                        last_port,
                        predicted_next,
                        confidence,
                    }
                }
                "random" => {
                    let observed = value
                        .get("observed")
                        .and_then(|v| v.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|v| v.as_u64().map(|n| n as u16))
                                .collect()
                        })
                        .unwrap_or_default();
                    Self::Random { observed }
                }
                _ => Self::Unknown,
            }
        } else {
            Self::Unknown
        }
    }
}

/// Connection info exchanged during rendezvous
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerConnectionInfo {
    /// STUN-discovered public address
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stun_results: Option<StunResults>,
    /// Family relay endpoint (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relay_endpoint: Option<String>,
    /// Self-hosted STUN server address (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stun_server: Option<String>,
}

/// STUN probe results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StunResults {
    /// Our public address as seen by STUN
    pub public_addr: String,
    /// Detected NAT type
    pub nat_type: String,
}

// ============================================================================
// Strategy Orchestrator
// ============================================================================

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
    {
        if let Some(peers) = result.get("peers").and_then(|p| p.as_array()) {
            let found = peers.iter().any(|p| {
                p.get("node_id")
                    .and_then(|id| id.as_str())
                    .map(|id| id == peer_id)
                    .unwrap_or(false)
            });
            if found {
                if let Some(peer) = peers.iter().find(|p| {
                    p.get("node_id")
                        .and_then(|id| id.as_str())
                        .map(|id| id == peer_id)
                        .unwrap_or(false)
                }) {
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
        }
    }
    debug!("   Tier 1: {} not found on LAN", peer_id);

    // ── Tier 2/3: NAT-aware strategy ───────────────────────────────────
    // Detect our NAT type
    let our_nat = detect_nat_type(&client).await;
    info!("🔍 Our NAT type: {:?}", our_nat);

    // Get peer NAT type from connection info (if available from rendezvous)
    let peer_nat = peer_connection_info
        .and_then(|info| info.stun_results.as_ref())
        .map(|stun| NatType::from_detection(&stun.nat_type))
        .unwrap_or(NatType::Unknown);

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
                .and_then(|v| v.as_bool())
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
    // This is the key innovation: use the relay as a coordination channel,
    // probe port patterns, then attempt a precisely timed punch.
    if our_nat.is_symmetric() || peer_nat.is_symmetric() || peer_nat == NatType::Unknown {
        info!(
            "🎯 Tier 3: Attempting relay-assisted coordinated punch to {}",
            peer_id
        );
        tiers_attempted.push(ConnectionTier::CoordinatedPunch);

        // Step 3a: Allocate relay session (immediate fallback path)
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
                // Step 3b: Probe our port pattern
                let our_pattern = probe_port_pattern(&client).await;

                if our_pattern.is_predictable() {
                    // Step 3c: Attempt coordinated punch via relay channel
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
                            .and_then(|v| v.as_bool())
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

                // Punch failed or skipped — relay is already allocated, use it
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

// ============================================================================
// Internal Helpers
// ============================================================================

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

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ── NatType tests ──────────────────────────────────────────────────

    #[test]
    fn test_nat_type_from_detection() {
        assert_eq!(NatType::from_detection("symmetric"), NatType::Symmetric);
        assert_eq!(NatType::from_detection("Symmetric"), NatType::Symmetric);
        assert_eq!(NatType::from_detection("full_cone"), NatType::FullCone);
        assert_eq!(NatType::from_detection("full-cone"), NatType::FullCone);
        assert_eq!(NatType::from_detection("none"), NatType::None);
        assert_eq!(NatType::from_detection("public"), NatType::None);
        assert_eq!(
            NatType::from_detection("address_restricted"),
            NatType::AddressRestricted
        );
        assert_eq!(
            NatType::from_detection("port_restricted"),
            NatType::PortRestricted
        );
        assert_eq!(NatType::from_detection("garbage"), NatType::Unknown);
    }

    #[test]
    fn test_nat_type_properties() {
        assert!(NatType::Symmetric.is_symmetric());
        assert!(!NatType::FullCone.is_symmetric());
        assert!(!NatType::None.is_symmetric());

        assert!(NatType::None.supports_direct_punch());
        assert!(NatType::FullCone.supports_direct_punch());
        assert!(NatType::PortRestricted.supports_direct_punch());
        assert!(!NatType::Symmetric.supports_direct_punch());
        assert!(!NatType::Unknown.supports_direct_punch());
    }

    // ── PortPattern tests ──────────────────────────────────────────────

    #[test]
    fn test_port_pattern_sequential_from_json() {
        let json = serde_json::json!({
            "type": "sequential",
            "step": 1,
            "last_port": 41204,
            "predicted_next": 41205,
            "confidence": 0.85
        });

        let pattern = PortPattern::from_json(&json);
        assert!(pattern.is_predictable());

        if let PortPattern::Sequential {
            step,
            last_port,
            predicted_next,
            confidence,
        } = pattern
        {
            assert_eq!(step, 1);
            assert_eq!(last_port, 41204);
            assert_eq!(predicted_next, 41205);
            assert!((confidence - 0.85).abs() < f64::EPSILON);
        } else {
            panic!("Expected Sequential pattern");
        }
    }

    #[test]
    fn test_port_pattern_random_from_json() {
        let json = serde_json::json!({
            "type": "random",
            "observed": [41200, 52300, 10500, 33000]
        });

        let pattern = PortPattern::from_json(&json);
        assert!(!pattern.is_predictable());

        if let PortPattern::Random { observed } = pattern {
            assert_eq!(observed.len(), 4);
            assert_eq!(observed[0], 41200);
        } else {
            panic!("Expected Random pattern");
        }
    }

    #[test]
    fn test_port_pattern_unknown_from_json() {
        let json = serde_json::json!({});
        let pattern = PortPattern::from_json(&json);
        assert!(!pattern.is_predictable());
        assert!(matches!(pattern, PortPattern::Unknown));
    }

    #[test]
    fn test_port_pattern_low_confidence_not_predictable() {
        let json = serde_json::json!({
            "type": "sequential",
            "step": 3,
            "last_port": 50000,
            "predicted_next": 50003,
            "confidence": 0.3
        });

        let pattern = PortPattern::from_json(&json);
        assert!(!pattern.is_predictable());
    }

    // ── ConnectionTier tests ───────────────────────────────────────────

    #[test]
    fn test_connection_tier_display() {
        assert_eq!(ConnectionTier::LanDirect.to_string(), "LAN Direct");
        assert_eq!(ConnectionTier::DirectPunch.to_string(), "Direct Punch");
        assert_eq!(
            ConnectionTier::CoordinatedPunch.to_string(),
            "Coordinated Punch"
        );
        assert_eq!(ConnectionTier::PureRelay.to_string(), "Pure Relay");
    }

    #[test]
    fn test_connection_result_serialization() {
        let result = ConnectionResult {
            tier: ConnectionTier::CoordinatedPunch,
            endpoint: "relay-session-abc123".to_string(),
            elapsed_ms: 450,
            tiers_attempted: vec![ConnectionTier::LanDirect, ConnectionTier::CoordinatedPunch],
        };

        let json = serde_json::to_string(&result).expect("serialize");
        assert!(json.contains("CoordinatedPunch"));
        assert!(json.contains("relay-session-abc123"));
        assert!(json.contains("450"));

        let deserialized: ConnectionResult = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deserialized.tier, ConnectionTier::CoordinatedPunch);
        assert_eq!(deserialized.tiers_attempted.len(), 2);
    }

    // ── PeerConnectionInfo tests ───────────────────────────────────────

    #[test]
    fn test_peer_connection_info_serialization() {
        let info = PeerConnectionInfo {
            stun_results: Some(StunResults {
                public_addr: "1.2.3.4:41200".to_string(),
                nat_type: "symmetric".to_string(),
            }),
            relay_endpoint: Some("192.168.1.144:3479".to_string()),
            stun_server: Some("192.168.1.144:3478".to_string()),
        };

        let json = serde_json::to_string(&info).expect("serialize");
        assert!(json.contains("1.2.3.4:41200"));
        assert!(json.contains("symmetric"));

        let deserialized: PeerConnectionInfo = serde_json::from_str(&json).expect("deserialize");
        assert!(deserialized.stun_results.is_some());
    }

    #[test]
    fn test_peer_connection_info_minimal() {
        let info = PeerConnectionInfo {
            stun_results: None,
            relay_endpoint: None,
            stun_server: None,
        };

        let json = serde_json::to_string(&info).expect("serialize");
        // With skip_serializing_if, empty fields should be omitted
        assert_eq!(json, "{}");
    }

    // ── NatType serde tests ────────────────────────────────────────────

    #[test]
    fn test_nat_type_serialization_roundtrip() {
        for nat in &[
            NatType::None,
            NatType::FullCone,
            NatType::AddressRestricted,
            NatType::PortRestricted,
            NatType::Symmetric,
            NatType::Unknown,
        ] {
            let json = serde_json::to_string(nat).expect("serialize");
            let deserialized: NatType = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(deserialized, *nat);
        }
    }
}
