// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Beacon Genetics Types - Mitochondrial DNA Model
//!
//! Core data types for the two-seed Dark Forest discovery architecture.
//!
//! ## Key Concepts
//!
//! - **`BeaconId`**: Public identifier (safe to share)
//! - **`MeetingRecord`**: Metadata about a peer meeting (seed stored separately)
//! - **`ClusterMembership`**: Cluster beacon membership
//! - **`BeaconGeneticsManifest`**: Complete beacon genetics for a node

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::Path;
use uuid::Uuid;

use crate::error::{SporeError, SporeResult};

// ============================================================================
// TIMESTAMP
// ============================================================================

/// Timestamp (Unix seconds)
pub type Timestamp = u64;

/// Get current Unix timestamp
#[must_use]
pub fn current_timestamp() -> Timestamp {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

// ============================================================================
// BEACON ID
// ============================================================================

/// Beacon ID (16 bytes, safe to share)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct BeaconId(pub String);

impl BeaconId {
    /// Create from hex string
    #[must_use]
    pub fn from_hex(hex: &str) -> Self {
        Self(hex.to_string())
    }

    /// Get short display form (first 8 chars)
    #[must_use]
    pub fn short(&self) -> &str {
        if self.0.len() >= 8 {
            &self.0[..8]
        } else {
            &self.0
        }
    }
}

impl std::fmt::Display for BeaconId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ============================================================================
// MEETING TYPES
// ============================================================================

/// How the meeting was established
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MeetingRelationship {
    /// Direct meeting (face-to-face or network)
    #[serde(rename = "direct")]
    Direct,

    /// Introduced by a mutual contact
    #[serde(rename = "introduced")]
    Introduced {
        /// Beacon ID of the mutual contact who introduced us
        by: BeaconId,
    },

    /// Met through cluster membership
    #[serde(rename = "cluster")]
    Cluster {
        /// Cluster identifier where the meeting occurred
        cluster_id: String,
    },

    /// Same lineage (auto-meet family devices)
    #[serde(rename = "same_lineage")]
    SameLineage,

    /// Federated partner
    #[serde(rename = "federated")]
    Federated,
}

/// Meeting visibility (mutual or one-way)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MeetingVisibility {
    /// I can see them, they can see me
    #[serde(rename = "mutual")]
    Mutual,

    /// I can see them, they can't see me
    #[serde(rename = "one_way_in")]
    OneWayIn,

    /// They can see me, I can't see them
    #[serde(rename = "one_way_out")]
    OneWayOut,
}

/// Record of a meeting - metadata (seed stored separately)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeetingRecord {
    /// Friendly name (human-readable)
    pub node_name: String,

    /// When we first met
    pub first_met: Timestamp,

    /// Last seen (updated on each broadcast)
    pub last_seen: Timestamp,

    /// Known endpoints
    pub endpoints: Vec<String>,

    /// Capabilities hint
    pub capabilities_hint: Vec<String>,

    /// Human notes
    pub notes: String,

    /// How this meeting was established
    pub relationship: MeetingRelationship,

    /// Whether this is one-way or mutual
    pub visibility: MeetingVisibility,

    /// Path to encrypted seed file (relative)
    pub seed_file: String,
}

// ============================================================================
// CLUSTER TYPES
// ============================================================================

/// Role in a cluster
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClusterRole {
    /// Entry point - external peers find us first
    #[serde(rename = "entry_point")]
    EntryPoint,

    /// Internal - only visible after entry point meeting
    #[serde(rename = "internal")]
    Internal,

    /// Hub - connects to other clusters
    #[serde(rename = "hub")]
    Hub,
}

/// Membership in a cluster beacon
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterMembership {
    /// Cluster identifier
    pub cluster_id: String,

    /// Our role in the cluster
    pub role: ClusterRole,

    /// When we joined
    pub joined_at: Timestamp,

    /// Members we know (their beacon IDs)
    pub known_members: Vec<BeaconId>,

    /// Path to encrypted cluster seed file
    pub seed_file: String,
}

// ============================================================================
// MANIFEST
// ============================================================================

/// Complete beacon genetics for a node (JSON serializable)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeaconGeneticsManifest {
    /// Version of the manifest format
    pub version: String,

    /// Our public beacon ID
    pub own_beacon_id: BeaconId,

    /// Hint to parent lineage (for sync)
    pub lineage_hint: String,

    /// Sync token for merge conflicts
    pub sync_token: String,

    /// Last sync timestamp
    pub last_sync: Timestamp,

    /// Meetings (`beacon_id` -> metadata)
    pub meetings: HashMap<String, MeetingRecord>,

    /// Cluster memberships
    pub clusters: Vec<ClusterMembership>,

    /// Who we've shared OUR seed with
    pub shared_with: HashSet<String>,
}

impl BeaconGeneticsManifest {
    /// Create new empty manifest
    #[must_use]
    pub fn new(own_beacon_id: BeaconId, lineage_hint: &str) -> Self {
        Self {
            version: "2.0.0".to_string(),
            own_beacon_id,
            lineage_hint: lineage_hint.to_string(),
            sync_token: Uuid::new_v4().to_string(),
            last_sync: current_timestamp(),
            meetings: HashMap::new(),
            clusters: Vec::new(),
            shared_with: HashSet::new(),
        }
    }

    /// Load from JSON file
    pub fn load(path: &Path) -> SporeResult<Self> {
        let contents = std::fs::read_to_string(path).map_err(|e| {
            SporeError::IoError(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Failed to read beacon genetics: {e}"),
            ))
        })?;

        serde_json::from_str(&contents)
            .map_err(|e| SporeError::DeserializationError(format!("Invalid JSON: {e}")))
    }

    /// Save to JSON file
    pub fn save(&self, path: &Path) -> SporeResult<()> {
        let contents = serde_json::to_string_pretty(self)
            .map_err(|e| SporeError::SerializationError(e.to_string()))?;

        std::fs::write(path, contents).map_err(|e| {
            SporeError::IoError(std::io::Error::other(format!(
                "Failed to write beacon genetics: {e}"
            )))
        })
    }

    /// Get meeting by beacon ID
    #[must_use]
    pub fn get_meeting(&self, beacon_id: &BeaconId) -> Option<&MeetingRecord> {
        self.meetings.get(&beacon_id.0)
    }

    /// Add or update a meeting
    pub fn add_meeting(&mut self, beacon_id: BeaconId, record: MeetingRecord) {
        self.meetings.insert(beacon_id.0, record);
    }

    /// List all known beacon IDs
    #[must_use]
    pub fn known_beacon_ids(&self) -> Vec<BeaconId> {
        self.meetings.keys().map(|k| BeaconId(k.clone())).collect()
    }
}

// ============================================================================
// SYNC RESULT
// ============================================================================

/// Result of sync operation
#[derive(Debug, Clone)]
pub struct SyncResult {
    /// Number of meetings added
    pub added: usize,
    /// Number of meetings updated
    pub updated: usize,
}

#[cfg(test)]
#[path = "types_tests.rs"]
mod tests;
