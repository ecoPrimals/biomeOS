// SPDX-License-Identifier: APGL-3.0-or-later WITH Sovran-Exemption-1.0
//
// Copyright 2025 ecoPrimals Project
// Licensed under the Affero General Public License v3.0 or later with Sovran Exemption 1.0.
// See LICENSE file in the project root or visit https://www.gnu.org/licenses/agpl-3.0.html

//! Shared types for BearDog client operations

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Encrypted data result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedData {
    /// Base64-encoded ciphertext
    pub ciphertext: String,

    /// Key ID used for encryption
    pub key_id: String,

    /// Algorithm used
    pub algorithm: String,

    /// Additional metadata
    #[serde(default)]
    pub metadata: Value,
}

/// Digital signature
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signature {
    /// Base64-encoded signature
    pub signature: String,

    /// Key ID used for signing
    pub key_id: String,

    /// Signature algorithm
    pub algorithm: String,
}

/// Key metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyInfo {
    /// Key identifier
    pub key_id: String,

    /// Key type (e.g., "RSA", "Ed25519")
    pub key_type: String,

    /// Key status
    pub status: String,

    /// Creation timestamp
    pub created_at: String,

    /// Additional metadata
    #[serde(default)]
    pub metadata: Value,
}

/// Access control request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessRequest {
    /// Subject requesting access (user, service, etc.)
    pub subject: String,

    /// Resource being accessed
    pub resource: String,

    /// Action being performed
    pub action: String,

    /// Additional context
    #[serde(default)]
    pub context: Value,
}

/// Access control decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessDecision {
    /// Decision: "allow" or "deny"
    pub decision: String,

    /// Reason for decision
    pub reason: String,

    /// Confidence score (0.0 - 1.0)
    #[serde(default)]
    pub confidence: f64,

    /// Additional metadata
    #[serde(default)]
    pub metadata: Value,
}

/// Audit log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    /// Entry ID
    pub id: String,

    /// Timestamp
    pub timestamp: String,

    /// Event type
    pub event_type: String,

    /// Subject (who)
    pub subject: String,

    /// Resource (what)
    pub resource: String,

    /// Action (how)
    pub action: String,

    /// Outcome
    pub outcome: String,

    /// Additional details
    #[serde(default)]
    pub details: Value,
}

/// BTSP tunnel information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunnelInfo {
    /// Unique tunnel identifier
    pub tunnel_id: String,

    /// Peer node ID
    pub peer_id: String,

    /// Peer endpoint
    pub peer_endpoint: String,

    /// Tunnel status
    pub status: String,

    /// Creation timestamp
    pub created_at: String,

    /// Additional metadata
    #[serde(default)]
    pub metadata: Value,
}

/// BTSP tunnel status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunnelStatus {
    /// Tunnel ID
    pub tunnel_id: String,

    /// Current state
    pub state: String,

    /// Bytes sent
    #[serde(default)]
    pub bytes_sent: u64,

    /// Bytes received
    #[serde(default)]
    pub bytes_received: u64,

    /// Last activity timestamp
    #[serde(default)]
    pub last_activity: Option<String>,

    /// Error message (if any)
    #[serde(default)]
    pub error: Option<String>,
}

