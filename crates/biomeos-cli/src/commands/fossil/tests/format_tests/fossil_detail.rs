// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::*;

#[test]
fn test_format_fossil_detail() {
    let fossil = FossilIndexEntry {
        node_id: "node-1".into(),
        session_started: Utc::now(),
        archival_reason: ArchivalReason::GracefulShutdown,
        fossil_path: PathBuf::from("/tmp/fossil1"),
        issue_count: 2,
        encrypted: false,
    };

    let lines = format_fossil_detail(&fossil);
    assert!(lines.iter().any(|l| l.contains("node-1")));
    assert!(lines.iter().any(|l| l.contains("Issues: 2")));
    assert!(lines.iter().any(|l| l.contains("Encrypted: No")));
}

#[test]
fn test_format_fossil_detail_encrypted() {
    let fossil = FossilIndexEntry {
        node_id: "node-enc".into(),
        session_started: Utc::now(),
        archival_reason: ArchivalReason::Manual,
        fossil_path: PathBuf::from("/tmp/encrypted-fossil"),
        issue_count: 0,
        encrypted: true,
    };
    let lines = format_fossil_detail(&fossil);
    assert!(lines.iter().any(|l| l.contains("Encrypted: Yes")));
    assert!(lines.iter().any(|l| l.contains("node-enc")));
}

#[test]
fn test_format_fossil_detail_crash_reason() {
    let fossil = FossilIndexEntry {
        node_id: "node-crash".into(),
        session_started: Utc::now(),
        archival_reason: ArchivalReason::Crash { exit_code: 137 },
        fossil_path: PathBuf::from("/tmp/crash-fossil"),
        issue_count: 3,
        encrypted: false,
    };
    let lines = format_fossil_detail(&fossil);
    assert!(lines.iter().any(|l| l.contains("Crash")));
    assert!(lines.iter().any(|l| l.contains("node-crash")));
}

#[test]
fn test_format_fossil_detail_all_archival_reasons() {
    for reason in [
        ArchivalReason::GracefulShutdown,
        ArchivalReason::Manual,
        ArchivalReason::AutomaticRotation,
    ] {
        let fossil = FossilIndexEntry {
            node_id: "node".into(),
            session_started: Utc::now(),
            archival_reason: reason,
            fossil_path: PathBuf::from("/tmp/f"),
            issue_count: 0,
            encrypted: false,
        };
        let lines = format_fossil_detail(&fossil);
        assert!(lines.iter().any(|l| l.contains("node")));
        assert!(lines.iter().any(|l| l.contains("Path:")));
    }
}

#[test]
fn test_format_fossil_detail_redeployment_reboot() {
    for reason in [ArchivalReason::Redeployment, ArchivalReason::Reboot] {
        let fossil = FossilIndexEntry {
            node_id: "n".into(),
            session_started: Utc::now(),
            archival_reason: reason,
            fossil_path: PathBuf::from("/tmp/f"),
            issue_count: 0,
            encrypted: false,
        };
        let lines = format_fossil_detail(&fossil);
        assert!(lines.iter().any(|l| l.contains("Node: n")));
    }
}

#[test]
fn test_format_fossil_detail_issue_zero() {
    let fossil = FossilIndexEntry {
        node_id: "z".into(),
        session_started: Utc::now(),
        archival_reason: ArchivalReason::GracefulShutdown,
        fossil_path: PathBuf::from("/tmp/z"),
        issue_count: 0,
        encrypted: false,
    };
    let lines = format_fossil_detail(&fossil);
    assert!(lines.iter().any(|l| l.contains("Issues: 0")));
}

#[test]
fn test_format_fossil_detail_long_path_display() {
    let long = PathBuf::from("/var/biomeos/fossils/").join("a".repeat(80));
    let fossil = FossilIndexEntry {
        node_id: "n".into(),
        session_started: Utc::now(),
        archival_reason: ArchivalReason::AutomaticRotation,
        fossil_path: long.clone(),
        issue_count: 99,
        encrypted: true,
    };
    let lines = format_fossil_detail(&fossil);
    assert!(lines.iter().any(|l| l.contains(&fossil.node_id)));
    assert!(lines.iter().any(|l| l.contains("Issues: 99")));
    assert!(
        lines
            .iter()
            .any(|l| l.contains(long.to_string_lossy().as_ref()))
    );
}
