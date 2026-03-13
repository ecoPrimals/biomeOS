// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Beacon Genetics Types - Mitochondrial DNA Model
//!
//! Core data types for the two-seed Dark Forest discovery architecture.
//!
//! ## Key Concepts
//!
//! - **BeaconId**: Public identifier (safe to share)
//! - **MeetingRecord**: Metadata about a peer meeting (seed stored separately)
//! - **ClusterMembership**: Cluster beacon membership
//! - **BeaconGeneticsManifest**: Complete beacon genetics for a node

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
pub fn current_timestamp() -> Timestamp {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

// ============================================================================
// BEACON ID
// ============================================================================

/// Beacon ID (16 bytes, safe to share)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct BeaconId(pub String);

impl BeaconId {
    /// Create from hex string
    pub fn from_hex(hex: &str) -> Self {
        Self(hex.to_string())
    }

    /// Get short display form (first 8 chars)
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

    /// Meetings (beacon_id -> metadata)
    pub meetings: HashMap<String, MeetingRecord>,

    /// Cluster memberships
    pub clusters: Vec<ClusterMembership>,

    /// Who we've shared OUR seed with
    pub shared_with: HashSet<String>,
}

impl BeaconGeneticsManifest {
    /// Create new empty manifest
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
                format!("Failed to read beacon genetics: {}", e),
            ))
        })?;

        serde_json::from_str(&contents)
            .map_err(|e| SporeError::DeserializationError(format!("Invalid JSON: {}", e)))
    }

    /// Save to JSON file
    pub fn save(&self, path: &Path) -> SporeResult<()> {
        let contents = serde_json::to_string_pretty(self)
            .map_err(|e| SporeError::SerializationError(e.to_string()))?;

        std::fs::write(path, contents).map_err(|e| {
            SporeError::IoError(std::io::Error::other(format!(
                "Failed to write beacon genetics: {}",
                e
            )))
        })
    }

    /// Get meeting by beacon ID
    pub fn get_meeting(&self, beacon_id: &BeaconId) -> Option<&MeetingRecord> {
        self.meetings.get(&beacon_id.0)
    }

    /// Add or update a meeting
    pub fn add_meeting(&mut self, beacon_id: BeaconId, record: MeetingRecord) {
        self.meetings.insert(beacon_id.0, record);
    }

    /// List all known beacon IDs
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
mod tests {
    use super::*;

    // ═══════════════════════════════════════════════════════════════
    // BeaconId tests
    // ═══════════════════════════════════════════════════════════════

    #[test]
    fn test_beacon_id_display() {
        let id = BeaconId::from_hex("a3f912b7deadbeef");
        assert_eq!(format!("{}", id), "a3f912b7deadbeef");
        assert_eq!(id.short(), "a3f912b7");
    }

    #[test]
    fn test_beacon_id_short_handling() {
        let short_id = BeaconId::from_hex("abc");
        assert_eq!(short_id.short(), "abc");
    }

    #[test]
    fn test_beacon_id_exactly_8_chars() {
        let id = BeaconId::from_hex("12345678");
        assert_eq!(id.short(), "12345678");
    }

    #[test]
    fn test_beacon_id_empty() {
        let id = BeaconId::from_hex("");
        assert_eq!(id.short(), "");
        assert_eq!(format!("{}", id), "");
    }

    #[test]
    fn test_beacon_id_clone_and_eq() {
        let a = BeaconId::from_hex("aabb");
        let b = a.clone();
        assert_eq!(a, b);
    }

    #[test]
    fn test_beacon_id_hash() {
        let a = BeaconId::from_hex("same");
        let b = BeaconId::from_hex("same");
        let mut set = std::collections::HashSet::new();
        set.insert(a);
        set.insert(b);
        assert_eq!(set.len(), 1, "equal IDs should deduplicate");
    }

    #[test]
    fn test_beacon_id_serde_roundtrip() {
        let id = BeaconId::from_hex("deadbeef12345678");
        let json = serde_json::to_string(&id).expect("serialize");
        let restored: BeaconId = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(id, restored);
    }

    // ═══════════════════════════════════════════════════════════════
    // MeetingRecord tests
    // ═══════════════════════════════════════════════════════════════

    #[test]
    fn test_meeting_record_serde_roundtrip() {
        let record = MeetingRecord {
            node_name: "alice-laptop".to_string(),
            first_met: 1234567890,
            last_seen: 1234567899,
            endpoints: vec!["192.168.1.100:9900".to_string()],
            capabilities_hint: vec!["compute".to_string()],
            notes: "Met at conference".to_string(),
            relationship: MeetingRelationship::Direct,
            visibility: MeetingVisibility::Mutual,
            seed_file: "a3f912b7.seed".to_string(),
        };

        let json = serde_json::to_string(&record).expect("serialize");
        assert!(json.contains("alice-laptop"));
        assert!(json.contains("direct"));

        let restored: MeetingRecord = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(restored.node_name, "alice-laptop");
        assert_eq!(restored.first_met, 1234567890);
        assert_eq!(restored.visibility, MeetingVisibility::Mutual);
    }

    #[test]
    fn test_meeting_record_clone() {
        let record = MeetingRecord {
            node_name: "test".into(),
            first_met: 0,
            last_seen: 0,
            endpoints: vec![],
            capabilities_hint: vec![],
            notes: String::new(),
            relationship: MeetingRelationship::Direct,
            visibility: MeetingVisibility::Mutual,
            seed_file: "test.seed".into(),
        };
        let cloned = record.clone();
        assert_eq!(cloned.node_name, "test");
    }

    // ═══════════════════════════════════════════════════════════════
    // MeetingRelationship tests
    // ═══════════════════════════════════════════════════════════════

    #[test]
    fn test_meeting_relationship_all_variants_serde() {
        let direct = MeetingRelationship::Direct;
        let json = serde_json::to_string(&direct).expect("serialize");
        assert!(json.contains("direct"));
        let _: MeetingRelationship = serde_json::from_str(&json).expect("deserialize");

        let introduced = MeetingRelationship::Introduced {
            by: BeaconId::from_hex("introducer123"),
        };
        let json = serde_json::to_string(&introduced).expect("serialize");
        assert!(json.contains("introduced"));
        let _: MeetingRelationship = serde_json::from_str(&json).expect("deserialize");

        let cluster = MeetingRelationship::Cluster {
            cluster_id: "cluster-abc".to_string(),
        };
        let json = serde_json::to_string(&cluster).expect("serialize");
        assert!(json.contains("cluster"));
        let _: MeetingRelationship = serde_json::from_str(&json).expect("deserialize");

        let same_lineage = MeetingRelationship::SameLineage;
        let json = serde_json::to_string(&same_lineage).expect("serialize");
        assert!(json.contains("same_lineage"));
        let _: MeetingRelationship = serde_json::from_str(&json).expect("deserialize");

        let federated = MeetingRelationship::Federated;
        let json = serde_json::to_string(&federated).expect("serialize");
        assert!(json.contains("federated"));
        let _: MeetingRelationship = serde_json::from_str(&json).expect("deserialize");
    }

    // ═══════════════════════════════════════════════════════════════
    // MeetingVisibility tests
    // ═══════════════════════════════════════════════════════════════

    #[test]
    fn test_meeting_visibility_all_variants() {
        for (vis, expected) in [
            (MeetingVisibility::Mutual, "mutual"),
            (MeetingVisibility::OneWayIn, "one_way_in"),
            (MeetingVisibility::OneWayOut, "one_way_out"),
        ] {
            let json = serde_json::to_string(&vis).expect("serialize");
            assert!(
                json.contains(expected),
                "expected '{}' in {}",
                expected,
                json
            );
            let restored: MeetingVisibility = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(vis, restored);
        }
    }

    // ═══════════════════════════════════════════════════════════════
    // ClusterRole / ClusterMembership tests
    // ═══════════════════════════════════════════════════════════════

    #[test]
    fn test_cluster_role_serde() {
        for (role, expected) in [
            (ClusterRole::EntryPoint, "entry_point"),
            (ClusterRole::Internal, "internal"),
            (ClusterRole::Hub, "hub"),
        ] {
            let json = serde_json::to_string(&role).expect("serialize");
            assert!(json.contains(expected));
            let _: ClusterRole = serde_json::from_str(&json).expect("deserialize");
        }
    }

    #[test]
    fn test_cluster_membership_serde_roundtrip() {
        let membership = ClusterMembership {
            cluster_id: "cl-01".into(),
            role: ClusterRole::Hub,
            joined_at: 9999,
            known_members: vec![BeaconId::from_hex("aaa"), BeaconId::from_hex("bbb")],
            seed_file: "cluster-01.seed".into(),
        };
        let json = serde_json::to_string(&membership).expect("serialize");
        let restored: ClusterMembership = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(restored.cluster_id, "cl-01");
        assert_eq!(restored.known_members.len(), 2);
    }

    #[test]
    fn test_cluster_membership_clone() {
        let m = ClusterMembership {
            cluster_id: "cl".into(),
            role: ClusterRole::Internal,
            joined_at: 0,
            known_members: vec![],
            seed_file: "s".into(),
        };
        let c = m.clone();
        assert_eq!(c.cluster_id, "cl");
    }

    // ═══════════════════════════════════════════════════════════════
    // SyncResult tests
    // ═══════════════════════════════════════════════════════════════

    #[test]
    fn test_sync_result() {
        let result = SyncResult {
            added: 5,
            updated: 3,
        };
        assert_eq!(result.added, 5);
        assert_eq!(result.updated, 3);
    }

    #[test]
    fn test_sync_result_clone_debug() {
        let result = SyncResult {
            added: 1,
            updated: 2,
        };
        let cloned = result.clone();
        assert_eq!(cloned.added, 1);
        let dbg = format!("{:?}", result);
        assert!(dbg.contains("SyncResult"));
    }

    // ═══════════════════════════════════════════════════════════════
    // Timestamp tests
    // ═══════════════════════════════════════════════════════════════

    #[test]
    fn test_current_timestamp() {
        let ts1 = current_timestamp();
        std::thread::sleep(std::time::Duration::from_millis(10));
        let ts2 = current_timestamp();

        assert!(ts1 > 1577836800); // Jan 1, 2020
        assert!(ts2 >= ts1);
    }

    // ═══════════════════════════════════════════════════════════════
    // BeaconGeneticsManifest tests
    // ═══════════════════════════════════════════════════════════════

    #[test]
    fn test_manifest_new() {
        let manifest = BeaconGeneticsManifest::new(BeaconId::from_hex("abc123"), "hint");
        assert_eq!(manifest.version, "2.0.0");
        assert_eq!(manifest.own_beacon_id, BeaconId::from_hex("abc123"));
        assert_eq!(manifest.lineage_hint, "hint");
        assert!(manifest.meetings.is_empty());
        assert!(manifest.clusters.is_empty());
        assert!(manifest.shared_with.is_empty());
        assert!(!manifest.sync_token.is_empty());
        assert!(manifest.last_sync > 0);
    }

    #[test]
    fn test_manifest_add_and_get_meeting() {
        let mut manifest = BeaconGeneticsManifest::new(BeaconId::from_hex("own"), "hint");

        let beacon_id = BeaconId::from_hex("peer-abc");
        let record = MeetingRecord {
            node_name: "peer-node".into(),
            first_met: 100,
            last_seen: 200,
            endpoints: vec![],
            capabilities_hint: vec![],
            notes: String::new(),
            relationship: MeetingRelationship::Direct,
            visibility: MeetingVisibility::Mutual,
            seed_file: "peer.seed".into(),
        };

        manifest.add_meeting(beacon_id.clone(), record);
        assert_eq!(manifest.meetings.len(), 1);

        let got = manifest.get_meeting(&beacon_id);
        assert!(got.is_some());
        assert_eq!(got.expect("should find").node_name, "peer-node");
    }

    #[test]
    fn test_manifest_known_beacon_ids() {
        let mut manifest = BeaconGeneticsManifest::new(BeaconId::from_hex("own"), "hint");

        manifest.add_meeting(
            BeaconId::from_hex("id1"),
            MeetingRecord {
                node_name: "n1".into(),
                first_met: 0,
                last_seen: 0,
                endpoints: vec![],
                capabilities_hint: vec![],
                notes: String::new(),
                relationship: MeetingRelationship::Direct,
                visibility: MeetingVisibility::Mutual,
                seed_file: "s".into(),
            },
        );
        manifest.add_meeting(
            BeaconId::from_hex("id2"),
            MeetingRecord {
                node_name: "n2".into(),
                first_met: 0,
                last_seen: 0,
                endpoints: vec![],
                capabilities_hint: vec![],
                notes: String::new(),
                relationship: MeetingRelationship::Federated,
                visibility: MeetingVisibility::OneWayIn,
                seed_file: "s2".into(),
            },
        );

        let ids = manifest.known_beacon_ids();
        assert_eq!(ids.len(), 2);
    }

    #[test]
    fn test_manifest_get_meeting_not_found() {
        let manifest = BeaconGeneticsManifest::new(BeaconId::from_hex("own"), "hint");
        assert!(manifest.get_meeting(&BeaconId::from_hex("nope")).is_none());
    }

    #[test]
    fn test_manifest_save_load_roundtrip() {
        let temp_dir = tempfile::TempDir::new().expect("create temp dir");
        let manifest_path = temp_dir.path().join(".beacon.genetics.json");

        let mut original =
            BeaconGeneticsManifest::new(BeaconId::from_hex("test123"), "lineage_hint");

        // Add some data to ensure round-trip fidelity
        original.shared_with.insert("peer-1".into());
        original.add_meeting(
            BeaconId::from_hex("meeting-id"),
            MeetingRecord {
                node_name: "alice".into(),
                first_met: 100,
                last_seen: 200,
                endpoints: vec!["10.0.0.1:9000".into()],
                capabilities_hint: vec!["compute".into()],
                notes: "test".into(),
                relationship: MeetingRelationship::Direct,
                visibility: MeetingVisibility::Mutual,
                seed_file: "seed.enc".into(),
            },
        );

        original.save(&manifest_path).expect("save");
        let loaded = BeaconGeneticsManifest::load(&manifest_path).expect("load");

        assert_eq!(original.own_beacon_id.0, loaded.own_beacon_id.0);
        assert_eq!(original.lineage_hint, loaded.lineage_hint);
        assert_eq!(loaded.meetings.len(), 1);
        assert!(loaded.shared_with.contains("peer-1"));
    }

    #[test]
    fn test_manifest_load_nonexistent() {
        let result = BeaconGeneticsManifest::load(Path::new("/nonexistent/path.json"));
        assert!(result.is_err());
    }

    #[test]
    fn test_manifest_serde_roundtrip() {
        let manifest = BeaconGeneticsManifest::new(BeaconId::from_hex("serde-test"), "hint");
        let json = serde_json::to_string(&manifest).expect("serialize");
        let restored: BeaconGeneticsManifest = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(restored.own_beacon_id, manifest.own_beacon_id);
        assert_eq!(restored.version, "2.0.0");
    }

    #[test]
    fn test_manifest_clone_and_debug() {
        let manifest = BeaconGeneticsManifest::new(BeaconId::from_hex("clone"), "h");
        let cloned = manifest.clone();
        assert_eq!(cloned.own_beacon_id, manifest.own_beacon_id);
        let dbg = format!("{:?}", manifest);
        assert!(dbg.contains("BeaconGeneticsManifest"));
    }
}
