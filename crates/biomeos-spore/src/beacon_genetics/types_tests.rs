use super::*;

// ═══════════════════════════════════════════════════════════════
// BeaconId tests
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_beacon_id_display() {
    let id = BeaconId::from_hex("a3f912b7deadbeef");
    assert_eq!(format!("{id}"), "a3f912b7deadbeef");
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
    assert_eq!(format!("{id}"), "");
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
        first_met: 1_234_567_890,
        last_seen: 1_234_567_899,
        endpoints: vec!["192.0.2.100:9900".to_string()],
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
    assert_eq!(restored.first_met, 1_234_567_890);
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
    assert_eq!(cloned.node_name, record.node_name);
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
        assert!(json.contains(expected), "expected '{expected}' in {json}");
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
    assert_eq!(c.cluster_id, m.cluster_id);
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
    let dbg = format!("{result:?}");
    assert!(dbg.contains("SyncResult"));
}

// ═══════════════════════════════════════════════════════════════
// Timestamp tests
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_current_timestamp() {
    let ts1 = current_timestamp();
    let ts2 = current_timestamp();

    assert!(ts1 > 1_577_836_800); // Jan 1, 2020
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

    let mut original = BeaconGeneticsManifest::new(BeaconId::from_hex("test123"), "lineage_hint");

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
    let dbg = format!("{manifest:?}");
    assert!(dbg.contains("BeaconGeneticsManifest"));
}
