// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Dark Forest Rendezvous Handler
//!
//! Implements the Tower's role as a rendezvous point for Pixel-USB handshakes.
//! The Tower NEVER holds plaintext data — it only relays encrypted Dark Forest beacons.
//!
//! ## Deep Debt Evolution (Feb 11, 2026)
//!
//! - Removed direct `AtomicClient::unix()` calls to `BearDog`
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
//! - Tower verifies family membership via shared `beacon_verification` module
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
    /// Active rendezvous slots: `lineage_hash` → Vec<RendezvousSlot>
    slots: Arc<RwLock<HashMap<String, Vec<RendezvousSlot>>>>,
    /// Family ID for beacon decryption context
    family_id: String,
    /// Neural API socket path (discovered at runtime)
    neural_api_socket: Option<String>,
}

impl RendezvousState {
    /// Create a new rendezvous state via runtime discovery.
    pub fn new() -> Self {
        let family_id = biomeos_core::family_discovery::get_family_id();
        let neural_api_socket = beacon_verification::discover_neural_api_socket(&family_id);

        Self {
            slots: Arc::new(RwLock::new(HashMap::new())),
            family_id,
            neural_api_socket,
        }
    }

    #[cfg(test)]
    pub(crate) fn new_for_test(family_id: &str, neural_socket_tier1: Option<&str>) -> Self {
        let neural_api_socket =
            beacon_verification::discover_neural_api_socket_from(family_id, neural_socket_tier1);
        Self {
            slots: Arc::new(RwLock::new(HashMap::new())),
            family_id: family_id.to_string(),
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
    /// instead of direct `BearDog` socket calls.
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
/// Tower verifies family membership via `beacon_verification`, then stores
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
#[path = "rendezvous_tests.rs"]
mod tests;
