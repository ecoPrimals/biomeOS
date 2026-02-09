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
    Introduced { by: BeaconId },

    /// Met through cluster membership
    #[serde(rename = "cluster")]
    Cluster { cluster_id: String },

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
    fn test_meeting_record() {
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

        let json = serde_json::to_string(&record).unwrap();
        assert!(json.contains("alice-laptop"));
        assert!(json.contains("direct"));
    }

    #[test]
    fn test_meeting_relationship_variants() {
        let direct = MeetingRelationship::Direct;
        let json = serde_json::to_string(&direct).unwrap();
        assert!(json.contains("direct"));

        let introduced = MeetingRelationship::Introduced {
            by: BeaconId::from_hex("introducer123"),
        };
        let json = serde_json::to_string(&introduced).unwrap();
        assert!(json.contains("introduced"));

        let cluster = MeetingRelationship::Cluster {
            cluster_id: "cluster-abc".to_string(),
        };
        let json = serde_json::to_string(&cluster).unwrap();
        assert!(json.contains("cluster"));

        let same_lineage = MeetingRelationship::SameLineage;
        let json = serde_json::to_string(&same_lineage).unwrap();
        assert!(json.contains("same_lineage"));
    }

    #[test]
    fn test_meeting_visibility_variants() {
        let mutual = MeetingVisibility::Mutual;
        let json = serde_json::to_string(&mutual).unwrap();
        assert!(json.contains("mutual"));

        let one_way_in = MeetingVisibility::OneWayIn;
        let json = serde_json::to_string(&one_way_in).unwrap();
        assert!(json.contains("one_way_in"));

        let one_way_out = MeetingVisibility::OneWayOut;
        let json = serde_json::to_string(&one_way_out).unwrap();
        assert!(json.contains("one_way_out"));
    }

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
    fn test_current_timestamp() {
        let ts1 = current_timestamp();
        std::thread::sleep(std::time::Duration::from_millis(10));
        let ts2 = current_timestamp();

        assert!(ts1 > 1577836800); // Jan 1, 2020
        assert!(ts2 >= ts1);
    }

    #[test]
    fn test_manifest_save_load_roundtrip() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let manifest_path = temp_dir.path().join(".beacon.genetics.json");

        let original = BeaconGeneticsManifest::new(BeaconId::from_hex("test123"), "lineage_hint");
        original.save(&manifest_path).unwrap();

        let loaded = BeaconGeneticsManifest::load(&manifest_path).unwrap();

        assert_eq!(original.own_beacon_id.0, loaded.own_beacon_id.0);
        assert_eq!(original.lineage_hint, loaded.lineage_hint);
    }
}
