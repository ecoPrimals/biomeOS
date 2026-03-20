// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Dark Forest Rendezvous Handler
//!
//! Implements the Tower's role as a rendezvous point for Pixel-USB handshakes.
//! The Tower NEVER holds plaintext data — it only relays encrypted Dark Forest beacons.
//!
//! ## Deep Debt Evolution (Feb 11, 2026)
//!
//! - Removed direct `AtomicClient::unix()` calls to BearDog
//! - Uses shared `beacon_verification` module (Neural API → socket fallback)
//! - Single source of truth for decrypt validation (`success && !plaintext.is_empty()`)
//! - No hardcoded primal names or socket paths
//!
//! ## Protocol Flow
//!
//! ```text
//! Pixel ──[encrypted_beacon]──> Tower Rendezvous <──[encrypted_beacon]── USB
//!                                    │
//!                    ┌────────────────┼────────────────┐
//!                    │                │                │
//!              1. Verify         2. Store         3. Match
//!              (beacon_verification)  (ephemeral       (same lineage
//!                                     slot)            hash → pair)
//! ```
//!
//! ## Security Model
//!
//! - Tower verifies family membership via shared beacon_verification module
//! - Tower does NOT store plaintext — only encrypted beacon blobs
//! - Rendezvous slots expire after 5 minutes
//! - Only same-lineage nodes can be paired

use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tracing::info;

use crate::beacon_verification;

/// Rendezvous slot: holds an encrypted beacon waiting for its pair
#[derive(Debug, Clone, Serialize)]
struct RendezvousSlot {
    /// The encrypted beacon blob (opaque to Tower)
    encrypted_beacon: String,
    /// Node identifier (from decrypted beacon, hashed)
    node_hash: String,
    /// Lineage hash (for matching family members)
    lineage_hash: String,
    /// When this slot was created
    created_at: u64,
    /// When this slot expires
    expires_at: u64,
    /// Optional connection info (STUN results, relay endpoints) for NAT traversal
    #[serde(skip_serializing_if = "Option::is_none")]
    connection_info: Option<biomeos_core::connection_strategy::PeerConnectionInfo>,
}

/// Shared rendezvous state
///
/// Deep Debt Evolution: No direct primal socket knowledge.
/// All verification routed through `beacon_verification` module.
#[derive(Clone)]
pub struct RendezvousState {
    /// Active rendezvous slots: lineage_hash → Vec<RendezvousSlot>
    slots: Arc<RwLock<HashMap<String, Vec<RendezvousSlot>>>>,
    /// Family ID for beacon decryption context
    family_id: String,
    /// Neural API socket path (discovered at runtime)
    neural_api_socket: Option<String>,
}

impl RendezvousState {
    /// Create a new rendezvous state
    ///
    /// Deep Debt Evolution: No socket parameter needed.
    /// Discovery happens at runtime via `beacon_verification::discover_neural_api_socket()`.
    pub fn new(_deprecated_socket: &str) -> Self {
        let family_id = biomeos_core::family_discovery::get_family_id();
        let neural_api_socket = beacon_verification::discover_neural_api_socket(&family_id);

        Self {
            slots: Arc::new(RwLock::new(HashMap::new())),
            family_id,
            neural_api_socket,
        }
    }

    /// Clean expired slots
    async fn clean_expired(&self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let mut slots = self.slots.write().await;
        slots.retain(|_, v| {
            v.retain(|slot| slot.expires_at > now);
            !v.is_empty()
        });
    }

    /// Verify family membership via shared beacon verification
    ///
    /// Deep Debt Evolution: Single source of truth.
    /// Routes through Neural API → socket discovery fallback.
    async fn verify_beacon(&self, dark_forest_token: &str) -> Option<String> {
        let result = beacon_verification::verify_dark_forest_token(
            self.neural_api_socket.as_deref(),
            &self.family_id,
            dark_forest_token,
        )
        .await?;

        Some(result.family_id)
    }

    /// Hash a node identity via shared crypto routing
    ///
    /// Deep Debt Evolution: Uses `beacon_verification::hash_via_capability()`
    /// instead of direct BearDog socket calls.
    async fn hash_node_identity(&self, token: &str, epoch: u64) -> String {
        let data = format!("{}:{}", token, epoch / 300);

        beacon_verification::hash_via_capability(
            self.neural_api_socket.as_deref(),
            &self.family_id,
            &data,
        )
        .await
        .unwrap_or_else(|| format!("anon-{epoch}"))
    }
}

/// Request to post a beacon to the rendezvous point
#[derive(Debug, Deserialize)]
pub struct RendezvousPostRequest {
    /// Encrypted Dark Forest beacon (base64 or raw noise bytes as base64)
    pub encrypted_beacon: String,
    /// Dark Forest verification token (proves family membership)
    pub dark_forest_token: String,
    /// Optional connection info for NAT traversal strategy selection.
    /// Includes STUN results, relay endpoints, and self-hosted STUN server addresses.
    /// When present, the matching peer receives this to optimize their connection tier.
    #[serde(default)]
    pub connection_info: Option<biomeos_core::connection_strategy::PeerConnectionInfo>,
}

/// Response from posting a beacon
#[derive(Debug, Serialize)]
pub struct RendezvousPostResponse {
    /// Whether the beacon was accepted
    pub accepted: bool,
    /// Rendezvous slot ID (for polling)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slot_id: Option<String>,
    /// If a matching peer was already waiting, their beacon
    #[serde(skip_serializing_if = "Option::is_none")]
    pub peer_beacon: Option<String>,
    /// Matched peer's connection info (STUN results, relay endpoints) for NAT traversal
    #[serde(skip_serializing_if = "Option::is_none")]
    pub peer_connection_info: Option<biomeos_core::connection_strategy::PeerConnectionInfo>,
    /// Number of peers waiting in the same lineage group
    pub peers_waiting: usize,
}

/// Request to check for a matching peer
#[derive(Debug, Deserialize)]
pub struct RendezvousCheckRequest {
    /// Dark Forest verification token
    pub dark_forest_token: String,
}

/// Response from checking for a peer
#[derive(Debug, Serialize)]
pub struct RendezvousCheckResponse {
    /// Whether a matching peer was found
    pub matched: bool,
    /// Matching peer's encrypted beacon (if found)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub peer_beacon: Option<String>,
    /// Matched peer's connection info (STUN results, relay endpoints) for NAT traversal
    #[serde(skip_serializing_if = "Option::is_none")]
    pub peer_connection_info: Option<biomeos_core::connection_strategy::PeerConnectionInfo>,
    /// Number of peers in rendezvous
    pub peers_waiting: usize,
}

/// POST /api/v1/rendezvous/beacon — Post an encrypted beacon for rendezvous
///
/// The Pixel or USB posts their Dark Forest beacon here.
/// Tower verifies family membership via beacon_verification, then stores
/// the beacon in an ephemeral slot. If a matching family member is already
/// waiting, returns their beacon immediately.
pub async fn post_beacon(
    State(state): State<Arc<RendezvousState>>,
    Json(request): Json<RendezvousPostRequest>,
) -> impl IntoResponse {
    // Clean expired slots first
    state.clean_expired().await;

    // Verify family membership via shared beacon verification
    let Some(lineage_hash) = state.verify_beacon(&request.dark_forest_token).await else {
        // Not family — silent rejection (Dark Forest: reveal nothing)
        return (StatusCode::FORBIDDEN, Json(serde_json::json!({}))).into_response();
    };

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    // Hash the node identity (routed through capability)
    let node_hash = state
        .hash_node_identity(&request.dark_forest_token, now)
        .await;

    let slot = RendezvousSlot {
        encrypted_beacon: request.encrypted_beacon,
        node_hash: node_hash.clone(),
        lineage_hash: lineage_hash.clone(),
        created_at: now,
        expires_at: now + 300, // 5 minute TTL
        connection_info: request.connection_info,
    };

    let mut slots = state.slots.write().await;
    let lineage_slots = slots.entry(lineage_hash.clone()).or_default();

    // Check if a matching peer is already waiting
    let (peer_beacon, peer_connection_info) = if lineage_slots.is_empty() {
        (None, None)
    } else {
        // Find a slot from a DIFFERENT node (not ourselves)
        let peer_idx = lineage_slots.iter().position(|s| s.node_hash != node_hash);

        match peer_idx {
            Some(idx) => {
                let peer = lineage_slots.remove(idx);
                info!(
                    "🤝 Rendezvous matched! Lineage: {}...",
                    &lineage_hash[..8.min(lineage_hash.len())]
                );
                (Some(peer.encrypted_beacon), peer.connection_info)
            }
            None => (None, None),
        }
    };

    let peers_waiting = lineage_slots.len();

    // Store our beacon (even if we got a match, in case of multiple peers)
    lineage_slots.push(slot);

    let response = RendezvousPostResponse {
        accepted: true,
        slot_id: Some(node_hash),
        peer_beacon,
        peer_connection_info,
        peers_waiting,
    };

    (StatusCode::OK, Json(response)).into_response()
}

/// GET /api/v1/rendezvous/check — Check if a matching peer has arrived
///
/// The Pixel or USB polls this to see if their counterpart has posted a beacon.
pub async fn check_peer(
    State(state): State<Arc<RendezvousState>>,
    Json(request): Json<RendezvousCheckRequest>,
) -> impl IntoResponse {
    state.clean_expired().await;

    // Verify family membership via shared beacon verification
    let Some(lineage_hash) = state.verify_beacon(&request.dark_forest_token).await else {
        return (StatusCode::FORBIDDEN, Json(serde_json::json!({}))).into_response();
    };

    let slots = state.slots.read().await;
    let lineage_slots = slots.get(&lineage_hash);

    let (matched, peer_beacon, peer_connection_info, peers_waiting) = match lineage_slots {
        Some(slots) if !slots.is_empty() => {
            // Return the first available peer's beacon and connection info
            let peer = slots.first();
            let beacon = peer.map(|s| s.encrypted_beacon.clone());
            let conn_info = peer.and_then(|s| s.connection_info.clone());
            (beacon.is_some(), beacon, conn_info, slots.len())
        }
        _ => (false, None, None, 0),
    };

    let response = RendezvousCheckResponse {
        matched,
        peer_beacon,
        peer_connection_info,
        peers_waiting,
    };

    (StatusCode::OK, Json(response)).into_response()
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
mod tests {
    use super::*;

    // ========== RendezvousState Tests ==========

    #[test]
    fn test_rendezvous_state_creation() {
        let state = RendezvousState::new("");
        assert!(!state.family_id.is_empty());
    }

    #[test]
    fn test_rendezvous_state_clone() {
        let state = RendezvousState::new("");
        let cloned = state.clone();
        assert_eq!(cloned.family_id, state.family_id);
    }

    #[tokio::test]
    async fn test_clean_expired_removes_old_slots() {
        let state = RendezvousState::new("");

        // Add an expired slot
        let mut slots = state.slots.write().await;
        slots.insert(
            "lineage1".to_string(),
            vec![RendezvousSlot {
                encrypted_beacon: "test".to_string(),
                node_hash: "node1".to_string(),
                lineage_hash: "lineage1".to_string(),
                created_at: 0,
                expires_at: 1, // Expired long ago
                connection_info: None,
            }],
        );
        drop(slots);

        state.clean_expired().await;

        let slots = state.slots.read().await;
        assert!(slots.is_empty());
    }

    #[tokio::test]
    async fn test_clean_expired_keeps_valid_slots() {
        let state = RendezvousState::new("");

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let mut slots = state.slots.write().await;
        slots.insert(
            "lineage1".to_string(),
            vec![RendezvousSlot {
                encrypted_beacon: "valid".to_string(),
                node_hash: "node1".to_string(),
                lineage_hash: "lineage1".to_string(),
                created_at: now,
                expires_at: now + 300, // 5 minutes from now
                connection_info: None,
            }],
        );
        drop(slots);

        state.clean_expired().await;

        let slots = state.slots.read().await;
        assert_eq!(slots.len(), 1);
        assert_eq!(slots["lineage1"][0].encrypted_beacon, "valid");
    }

    #[tokio::test]
    async fn test_clean_expired_mixed_slots() {
        let state = RendezvousState::new("");

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let mut slots = state.slots.write().await;
        slots.insert(
            "lineage1".to_string(),
            vec![
                RendezvousSlot {
                    encrypted_beacon: "expired".to_string(),
                    node_hash: "node1".to_string(),
                    lineage_hash: "lineage1".to_string(),
                    created_at: 0,
                    expires_at: 1, // Expired
                    connection_info: None,
                },
                RendezvousSlot {
                    encrypted_beacon: "valid".to_string(),
                    node_hash: "node2".to_string(),
                    lineage_hash: "lineage1".to_string(),
                    created_at: now,
                    expires_at: now + 300, // Valid
                    connection_info: None,
                },
            ],
        );
        drop(slots);

        state.clean_expired().await;

        let slots = state.slots.read().await;
        assert_eq!(slots.len(), 1);
        assert_eq!(slots["lineage1"].len(), 1);
        assert_eq!(slots["lineage1"][0].encrypted_beacon, "valid");
    }

    // ========== Request/Response Serialization Tests ==========

    #[test]
    fn test_rendezvous_post_request_deserialize() {
        let json = serde_json::json!({
            "encrypted_beacon": "base64data",
            "dark_forest_token": "tokendata"
        });
        let request: RendezvousPostRequest = serde_json::from_value(json).expect("deserialize");
        assert_eq!(request.encrypted_beacon, "base64data");
        assert_eq!(request.dark_forest_token, "tokendata");
    }

    #[test]
    fn test_rendezvous_post_response_serialize() {
        let response = RendezvousPostResponse {
            accepted: true,
            slot_id: Some("slot-abc".to_string()),
            peer_beacon: None,
            peer_connection_info: None,
            peers_waiting: 2,
        };

        let json = serde_json::to_string(&response).expect("serialize");
        assert!(json.contains("\"accepted\":true"));
        assert!(json.contains("\"slot_id\":\"slot-abc\""));
        assert!(!json.contains("peer_beacon")); // skip_serializing_if = None
        assert!(!json.contains("peer_connection_info")); // skip_serializing_if = None
        assert!(json.contains("\"peers_waiting\":2"));
    }

    #[test]
    fn test_rendezvous_post_response_with_peer() {
        let response = RendezvousPostResponse {
            accepted: true,
            slot_id: Some("slot-123".to_string()),
            peer_beacon: Some("encrypted_peer_data".to_string()),
            peer_connection_info: None,
            peers_waiting: 0,
        };

        let json = serde_json::to_string(&response).expect("serialize");
        assert!(json.contains("encrypted_peer_data"));
    }

    #[test]
    fn test_rendezvous_post_response_with_connection_info() {
        use biomeos_core::connection_strategy::{PeerConnectionInfo, StunResults};

        let response = RendezvousPostResponse {
            accepted: true,
            slot_id: Some("slot-456".to_string()),
            peer_beacon: Some("peer_data".to_string()),
            peer_connection_info: Some(PeerConnectionInfo {
                stun_results: Some(StunResults {
                    public_addr: "1.2.3.4:41200".to_string(),
                    nat_type: "symmetric".to_string(),
                }),
                relay_endpoint: Some("192.168.1.144:3479".to_string()),
                stun_server: None,
            }),
            peers_waiting: 1,
        };

        let json = serde_json::to_string(&response).expect("serialize");
        assert!(json.contains("peer_connection_info"));
        assert!(json.contains("1.2.3.4:41200"));
        assert!(json.contains("symmetric"));
        assert!(json.contains("192.168.1.144:3479"));
    }

    #[test]
    fn test_rendezvous_check_request_deserialize() {
        let json = serde_json::json!({
            "dark_forest_token": "check-token"
        });
        let request: RendezvousCheckRequest = serde_json::from_value(json).expect("deserialize");
        assert_eq!(request.dark_forest_token, "check-token");
    }

    #[test]
    fn test_rendezvous_check_response_no_match() {
        let response = RendezvousCheckResponse {
            matched: false,
            peer_beacon: None,
            peer_connection_info: None,
            peers_waiting: 0,
        };

        let json = serde_json::to_string(&response).expect("serialize");
        assert!(json.contains("\"matched\":false"));
        assert!(!json.contains("peer_beacon")); // skip_serializing_if = None
        assert!(!json.contains("peer_connection_info")); // skip_serializing_if = None
    }

    #[test]
    fn test_rendezvous_check_response_with_match() {
        let response = RendezvousCheckResponse {
            matched: true,
            peer_beacon: Some("matched_beacon_data".to_string()),
            peer_connection_info: None,
            peers_waiting: 3,
        };

        let json = serde_json::to_string(&response).expect("serialize");
        assert!(json.contains("\"matched\":true"));
        assert!(json.contains("matched_beacon_data"));
        assert!(json.contains("\"peers_waiting\":3"));
    }

    // ========== RendezvousSlot Tests ==========

    #[test]
    fn test_rendezvous_slot_clone() {
        let slot = RendezvousSlot {
            encrypted_beacon: "beacon".to_string(),
            node_hash: "hash".to_string(),
            lineage_hash: "lineage".to_string(),
            created_at: 1000,
            expires_at: 1300,
            connection_info: None,
        };

        assert_eq!(slot.encrypted_beacon, "beacon");
        assert_eq!(slot.node_hash, "hash");
        assert_eq!(slot.lineage_hash, "lineage");
        assert_eq!(slot.created_at, 1000);
        assert_eq!(slot.expires_at, 1300);
        assert!(slot.connection_info.is_none());
    }

    #[test]
    fn test_rendezvous_slot_serialization() {
        let slot = RendezvousSlot {
            encrypted_beacon: "enc_data".to_string(),
            node_hash: "nh".to_string(),
            lineage_hash: "lh".to_string(),
            created_at: 100,
            expires_at: 400,
            connection_info: None,
        };

        let json = serde_json::to_string(&slot).expect("serialize");
        assert!(json.contains("enc_data"));
        assert!(json.contains("\"created_at\":100"));
        assert!(json.contains("\"expires_at\":400"));
        assert!(!json.contains("connection_info")); // skip_serializing_if = None
    }

    #[tokio::test]
    async fn test_multiple_lineage_groups() {
        let state = RendezvousState::new("");

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let mut slots = state.slots.write().await;

        slots.insert(
            "family-a".to_string(),
            vec![RendezvousSlot {
                encrypted_beacon: "beacon-a1".to_string(),
                node_hash: "node-a1".to_string(),
                lineage_hash: "family-a".to_string(),
                created_at: now,
                expires_at: now + 300,
                connection_info: None,
            }],
        );
        slots.insert(
            "family-b".to_string(),
            vec![
                RendezvousSlot {
                    encrypted_beacon: "beacon-b1".to_string(),
                    node_hash: "node-b1".to_string(),
                    lineage_hash: "family-b".to_string(),
                    created_at: now,
                    expires_at: now + 300,
                    connection_info: None,
                },
                RendezvousSlot {
                    encrypted_beacon: "beacon-b2".to_string(),
                    node_hash: "node-b2".to_string(),
                    lineage_hash: "family-b".to_string(),
                    created_at: now,
                    expires_at: now + 300,
                    connection_info: None,
                },
            ],
        );
        drop(slots);

        let slots = state.slots.read().await;
        assert_eq!(slots.len(), 2);
        assert_eq!(slots["family-a"].len(), 1);
        assert_eq!(slots["family-b"].len(), 2);
    }

    // ========== Request validation and edge cases ==========

    #[test]
    fn test_rendezvous_post_request_empty_beacon() {
        let json = serde_json::json!({
            "encrypted_beacon": "",
            "dark_forest_token": "some-token"
        });
        let request: RendezvousPostRequest = serde_json::from_value(json).expect("deserialize");
        assert!(request.encrypted_beacon.is_empty());
        assert_eq!(request.dark_forest_token, "some-token");
    }

    #[test]
    fn test_rendezvous_post_request_connection_info_default() {
        let json = r#"{"encrypted_beacon": "x", "dark_forest_token": "t"}"#;
        let request: RendezvousPostRequest = serde_json::from_str(json).expect("deserialize");
        assert!(
            request.connection_info.is_none(),
            "connection_info should default to None"
        );
    }

    #[test]
    fn test_rendezvous_post_request_with_connection_info() {
        let json = serde_json::json!({
            "encrypted_beacon": "beacon",
            "dark_forest_token": "token",
            "connection_info": {
                "stun_results": {"public_addr": "1.2.3.4:41200", "nat_type": "symmetric"},
                "relay_endpoint": "192.168.1.1:3479"
            }
        });
        let request: RendezvousPostRequest = serde_json::from_value(json).expect("deserialize");
        assert!(request.connection_info.is_some());
        let info = request.connection_info.as_ref().unwrap();
        assert!(info.stun_results.is_some());
        assert_eq!(
            info.stun_results.as_ref().unwrap().public_addr,
            "1.2.3.4:41200"
        );
    }

    #[test]
    fn test_rendezvous_check_request_empty_token() {
        let json = serde_json::json!({"dark_forest_token": ""});
        let request: RendezvousCheckRequest = serde_json::from_value(json).expect("deserialize");
        assert!(request.dark_forest_token.is_empty());
    }

    // ========== Handler integration - invalid token returns 403 ==========

    #[tokio::test]
    async fn test_post_beacon_invalid_token_returns_forbidden() {
        let state = Arc::new(RendezvousState::new(""));
        let request = RendezvousPostRequest {
            encrypted_beacon: "fake-beacon-data".to_string(),
            dark_forest_token: "invalid-token-no-socket".to_string(),
            connection_info: None,
        };

        let response = post_beacon(axum::extract::State(state), axum::Json(request)).await;

        let (parts, _body) = response.into_response().into_parts();
        assert_eq!(
            parts.status,
            axum::http::StatusCode::FORBIDDEN,
            "invalid token should return 403 Forbidden (Dark Forest: no socket = not family)"
        );
    }

    #[tokio::test]
    async fn test_check_peer_invalid_token_returns_forbidden() {
        let state = Arc::new(RendezvousState::new(""));
        let request = RendezvousCheckRequest {
            dark_forest_token: "invalid-token".to_string(),
        };

        let response = check_peer(axum::extract::State(state), axum::Json(request)).await;

        let (parts, _) = response.into_response().into_parts();
        assert_eq!(
            parts.status,
            axum::http::StatusCode::FORBIDDEN,
            "invalid token should return 403 Forbidden"
        );
    }

    #[test]
    fn test_rendezvous_slot_expiry_logic() {
        let slot = RendezvousSlot {
            encrypted_beacon: "x".to_string(),
            node_hash: "n".to_string(),
            lineage_hash: "l".to_string(),
            created_at: 100,
            expires_at: 400,
            connection_info: None,
        };
        assert!(slot.expires_at > slot.created_at);
        assert_eq!(slot.expires_at - slot.created_at, 300);
    }
}
