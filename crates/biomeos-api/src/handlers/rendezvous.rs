//! Dark Forest Rendezvous Handler
//!
//! Implements the Tower's role as a rendezvous point for Pixel-USB handshakes.
//! The Tower NEVER holds plaintext data — it only relays encrypted Dark Forest beacons.
//!
//! ## Protocol Flow
//!
//! ```text
//! Pixel ──[encrypted_beacon]──> Tower Rendezvous <──[encrypted_beacon]── USB
//!                                    │
//!                    ┌────────────────┼────────────────┐
//!                    │                │                │
//!              1. Verify         2. Store         3. Match
//!              (BearDog           (ephemeral       (same lineage
//!               decrypt)           slot)            hash → pair)
//!                    │                │                │
//!                    └────────────────┼────────────────┘
//!                                    │
//!         Pixel gets USB's beacon ◄──┴──► USB gets Pixel's beacon
//! ```
//!
//! ## Security Model
//!
//! - Tower verifies family membership via Dark Forest decryption (BearDog)
//! - Tower does NOT store plaintext — only encrypted beacon blobs
//! - Rendezvous slots expire after 5 minutes
//! - Only same-lineage nodes can be paired
//! - Rate-limited to prevent enumeration

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use base64::Engine;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tracing::info;

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
}

/// Shared rendezvous state
#[derive(Clone)]
pub struct RendezvousState {
    /// Active rendezvous slots: lineage_hash → Vec<RendezvousSlot>
    slots: Arc<RwLock<HashMap<String, Vec<RendezvousSlot>>>>,
    /// BearDog socket for beacon verification
    beardog_socket: String,
}

impl RendezvousState {
    pub fn new(beardog_socket: &str) -> Self {
        Self {
            slots: Arc::new(RwLock::new(HashMap::new())),
            beardog_socket: beardog_socket.to_string(),
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
}

/// Request to post a beacon to the rendezvous point
#[derive(Debug, Deserialize)]
pub struct RendezvousPostRequest {
    /// Encrypted Dark Forest beacon (base64 or raw noise bytes as base64)
    pub encrypted_beacon: String,
    /// Dark Forest verification token (proves family membership)
    pub dark_forest_token: String,
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
    /// Number of peers in rendezvous
    pub peers_waiting: usize,
}

/// POST /api/v1/rendezvous/beacon — Post an encrypted beacon for rendezvous
///
/// The Pixel or USB posts their Dark Forest beacon here.
/// Tower verifies family membership, then stores the beacon in an ephemeral slot.
/// If a matching family member is already waiting, returns their beacon immediately.
pub async fn post_beacon(
    State(state): State<Arc<RendezvousState>>,
    Json(request): Json<RendezvousPostRequest>,
) -> impl IntoResponse {
    // Clean expired slots first
    state.clean_expired().await;

    // Verify family membership via BearDog
    let client = biomeos_core::AtomicClient::unix(&state.beardog_socket)
        .with_timeout(Duration::from_secs(5));

    let verify_result = client
        .call(
            "beacon.try_decrypt",
            serde_json::json!({
                "data": request.dark_forest_token
            }),
        )
        .await;

    // Extract lineage hash from successful decryption
    let lineage_hash = match verify_result {
        Ok(result) => {
            let has_plaintext = result.get("plaintext").is_some();
            let decrypted = result
                .get("decrypted")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            if !has_plaintext && !decrypted {
                return (StatusCode::FORBIDDEN, Json(serde_json::json!({}))).into_response();
            }

            // Use family hash from decrypted beacon, or derive from token
            result
                .get("plaintext")
                .and_then(|p| p.get("family_hash"))
                .and_then(|h| h.as_str())
                .unwrap_or("unknown")
                .to_string()
        }
        Err(_) => {
            // Not family — silent rejection
            return (StatusCode::FORBIDDEN, Json(serde_json::json!({}))).into_response();
        }
    };

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    // Hash the node identity for the slot (don't store plaintext node ID)
    let node_hash = {
        let hash_result = client
            .call(
                "crypto.blake3_hash",
                serde_json::json!({
                    "data": base64::engine::general_purpose::STANDARD.encode(
                        format!("{}:{}", request.dark_forest_token, now / 300).as_bytes()
                    )
                }),
            )
            .await;

        match hash_result {
            Ok(r) => r
                .get("hash")
                .and_then(|h| h.as_str())
                .unwrap_or("unknown")
                .to_string(),
            Err(_) => format!("anon-{}", now),
        }
    };

    let slot = RendezvousSlot {
        encrypted_beacon: request.encrypted_beacon,
        node_hash: node_hash.clone(),
        lineage_hash: lineage_hash.clone(),
        created_at: now,
        expires_at: now + 300, // 5 minute TTL
    };

    let mut slots = state.slots.write().await;
    let lineage_slots = slots.entry(lineage_hash.clone()).or_default();

    // Check if a matching peer is already waiting
    let peer_beacon = if !lineage_slots.is_empty() {
        // Find a slot from a DIFFERENT node (not ourselves)
        let peer_idx = lineage_slots.iter().position(|s| s.node_hash != node_hash);

        peer_idx.map(|idx| {
            let peer = lineage_slots.remove(idx);
            info!(
                "🤝 Rendezvous matched! Lineage: {}...",
                &lineage_hash[..8.min(lineage_hash.len())]
            );
            peer.encrypted_beacon
        })
    } else {
        None
    };

    let peers_waiting = lineage_slots.len();

    // Store our beacon (even if we got a match, in case of multiple peers)
    lineage_slots.push(slot);

    let response = RendezvousPostResponse {
        accepted: true,
        slot_id: Some(node_hash),
        peer_beacon,
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

    // Verify family membership
    let client = biomeos_core::AtomicClient::unix(&state.beardog_socket)
        .with_timeout(Duration::from_secs(5));

    let verify_result = client
        .call(
            "beacon.try_decrypt",
            serde_json::json!({
                "data": request.dark_forest_token
            }),
        )
        .await;

    let lineage_hash = match verify_result {
        Ok(result) => {
            let has_plaintext = result.get("plaintext").is_some();
            let decrypted = result
                .get("decrypted")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            if !has_plaintext && !decrypted {
                return (StatusCode::FORBIDDEN, Json(serde_json::json!({}))).into_response();
            }

            result
                .get("plaintext")
                .and_then(|p| p.get("family_hash"))
                .and_then(|h| h.as_str())
                .unwrap_or("unknown")
                .to_string()
        }
        Err(_) => {
            return (StatusCode::FORBIDDEN, Json(serde_json::json!({}))).into_response();
        }
    };

    let slots = state.slots.read().await;
    let lineage_slots = slots.get(&lineage_hash);

    let (matched, peer_beacon, peers_waiting) = match lineage_slots {
        Some(slots) if !slots.is_empty() => {
            // Return the first available peer beacon
            let peer = slots.first().map(|s| s.encrypted_beacon.clone());
            (peer.is_some(), peer, slots.len())
        }
        _ => (false, None, 0),
    };

    let response = RendezvousCheckResponse {
        matched,
        peer_beacon,
        peers_waiting,
    };

    (StatusCode::OK, Json(response)).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========== RendezvousState Tests ==========

    #[test]
    fn test_rendezvous_state_creation() {
        let state = RendezvousState::new("/tmp/test-beardog.sock");
        assert_eq!(state.beardog_socket, "/tmp/test-beardog.sock");
    }

    #[test]
    fn test_rendezvous_state_clone() {
        let state = RendezvousState::new("/tmp/test.sock");
        let cloned = state.clone();
        assert_eq!(cloned.beardog_socket, "/tmp/test.sock");
    }

    #[tokio::test]
    async fn test_clean_expired_removes_old_slots() {
        let state = RendezvousState::new("/tmp/test.sock");

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
            }],
        );
        drop(slots);

        state.clean_expired().await;

        let slots = state.slots.read().await;
        assert!(slots.is_empty());
    }

    #[tokio::test]
    async fn test_clean_expired_keeps_valid_slots() {
        let state = RendezvousState::new("/tmp/test.sock");

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
        let state = RendezvousState::new("/tmp/test.sock");

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
                },
                RendezvousSlot {
                    encrypted_beacon: "valid".to_string(),
                    node_hash: "node2".to_string(),
                    lineage_hash: "lineage1".to_string(),
                    created_at: now,
                    expires_at: now + 300, // Valid
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
            peers_waiting: 2,
        };

        let json = serde_json::to_string(&response).expect("serialize");
        assert!(json.contains("\"accepted\":true"));
        assert!(json.contains("\"slot_id\":\"slot-abc\""));
        assert!(!json.contains("peer_beacon")); // skip_serializing_if = None
        assert!(json.contains("\"peers_waiting\":2"));
    }

    #[test]
    fn test_rendezvous_post_response_with_peer() {
        let response = RendezvousPostResponse {
            accepted: true,
            slot_id: Some("slot-123".to_string()),
            peer_beacon: Some("encrypted_peer_data".to_string()),
            peers_waiting: 0,
        };

        let json = serde_json::to_string(&response).expect("serialize");
        assert!(json.contains("encrypted_peer_data"));
    }

    #[test]
    fn test_rendezvous_check_request_deserialize() {
        let json = serde_json::json!({
            "dark_forest_token": "check-token"
        });
        let request: RendezvousCheckRequest =
            serde_json::from_value(json).expect("deserialize");
        assert_eq!(request.dark_forest_token, "check-token");
    }

    #[test]
    fn test_rendezvous_check_response_no_match() {
        let response = RendezvousCheckResponse {
            matched: false,
            peer_beacon: None,
            peers_waiting: 0,
        };

        let json = serde_json::to_string(&response).expect("serialize");
        assert!(json.contains("\"matched\":false"));
        assert!(!json.contains("peer_beacon")); // skip_serializing_if = None
    }

    #[test]
    fn test_rendezvous_check_response_with_match() {
        let response = RendezvousCheckResponse {
            matched: true,
            peer_beacon: Some("matched_beacon_data".to_string()),
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
        };

        let cloned = slot.clone();
        assert_eq!(cloned.encrypted_beacon, "beacon");
        assert_eq!(cloned.node_hash, "hash");
        assert_eq!(cloned.lineage_hash, "lineage");
        assert_eq!(cloned.created_at, 1000);
        assert_eq!(cloned.expires_at, 1300);
    }

    #[test]
    fn test_rendezvous_slot_serialization() {
        let slot = RendezvousSlot {
            encrypted_beacon: "enc_data".to_string(),
            node_hash: "nh".to_string(),
            lineage_hash: "lh".to_string(),
            created_at: 100,
            expires_at: 400,
        };

        let json = serde_json::to_string(&slot).expect("serialize");
        assert!(json.contains("enc_data"));
        assert!(json.contains("\"created_at\":100"));
        assert!(json.contains("\"expires_at\":400"));
    }

    #[tokio::test]
    async fn test_multiple_lineage_groups() {
        let state = RendezvousState::new("/tmp/test.sock");

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let mut slots = state.slots.write().await;

        // Add slots for two different lineage groups
        slots.insert(
            "family-a".to_string(),
            vec![RendezvousSlot {
                encrypted_beacon: "beacon-a1".to_string(),
                node_hash: "node-a1".to_string(),
                lineage_hash: "family-a".to_string(),
                created_at: now,
                expires_at: now + 300,
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
                },
                RendezvousSlot {
                    encrypted_beacon: "beacon-b2".to_string(),
                    node_hash: "node-b2".to_string(),
                    lineage_hash: "family-b".to_string(),
                    created_at: now,
                    expires_at: now + 300,
                },
            ],
        );
        drop(slots);

        let slots = state.slots.read().await;
        assert_eq!(slots.len(), 2);
        assert_eq!(slots["family-a"].len(), 1);
        assert_eq!(slots["family-b"].len(), 2);
    }
}
