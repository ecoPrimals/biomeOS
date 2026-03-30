// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Dark Forest beacon types
//!
//! Plaintext and encrypted beacon structures for the `BirdSong` Dark Forest
//! trust model.

use serde::{Deserialize, Serialize};

/// Plaintext beacon data (before encryption)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeaconPlaintext {
    /// Hash of `family_id` (not the actual ID)
    pub family_hash: String,
    /// Node identifier
    pub node_id: String,
    /// Unix timestamp
    pub timestamp: u64,
    /// `BearDog` socket path
    pub socket_path: String,
    /// Capabilities (hashed)
    pub capabilities_hash: String,
    /// Optional: lineage mode (genesis/sibling)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lineage_mode: Option<String>,
}

/// Encrypted beacon (what gets broadcast)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedBeacon {
    /// Encrypted payload (base64)
    pub ciphertext: String,
    /// Nonce used for encryption (base64)
    pub nonce: String,
    /// Authentication tag (base64)
    pub tag: String,
    /// Version for protocol evolution
    pub version: u8,
}

/// Discovery result from scanning for encrypted beacons
#[derive(Debug, Clone)]
pub struct DiscoveredPeer {
    /// Decrypted beacon data
    pub beacon: BeaconPlaintext,
    /// Whether lineage has been verified
    pub lineage_verified: bool,
    /// Session key (if derived)
    pub session_key: Option<String>,
}
