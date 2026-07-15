// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::*;

#[test]
fn test_compute_cleanup_plan_empty() {
    let fossils: Vec<FossilIndexEntry> = vec![];
    let cutoff = Utc::now();
    let plan = compute_cleanup_plan(&fossils, cutoff);
    assert_eq!(plan.count, 0);
    assert!(plan.to_remove.is_empty());
    assert_eq!(plan.freed_bytes, 0);
}

#[test]
fn test_scan_old_logs_nonexistent() {
    let result = scan_old_logs(Path::new("/nonexistent/path/12345"));
    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());
}

#[test]
fn test_compute_cleanup_plan_with_fossils() {
    let temp = tempfile::tempdir().expect("temp dir");
    let fossil_path = temp.path().join("fossil1");
    std::fs::write(&fossil_path, b"fake fossil data").expect("write");

    let fossils = vec![FossilIndexEntry {
        node_id: "node-1".into(),
        session_started: Utc::now() - chrono::Duration::days(60),
        archival_reason: ArchivalReason::GracefulShutdown,
        fossil_path,
        issue_count: 0,
        encrypted: false,
    }];
    let cutoff = Utc::now() - chrono::Duration::days(30);
    let plan = compute_cleanup_plan(&fossils, cutoff);
    assert_eq!(plan.count, 1);
    assert_eq!(plan.to_remove.len(), 1);
    assert!(plan.freed_bytes > 0);
}

#[test]
fn test_compute_cleanup_plan_fossil_after_cutoff() {
    let fossils = vec![FossilIndexEntry {
        node_id: "node-recent".into(),
        session_started: Utc::now() - chrono::Duration::days(5),
        archival_reason: ArchivalReason::GracefulShutdown,
        fossil_path: PathBuf::from("/tmp/recent"),
        issue_count: 0,
        encrypted: false,
    }];
    let cutoff = Utc::now() - chrono::Duration::days(30);
    let plan = compute_cleanup_plan(&fossils, cutoff);
    assert_eq!(plan.count, 0);
    assert!(plan.to_remove.is_empty());
}

#[test]
fn test_cleanup_plan_default() {
    let plan = CleanupPlan::default();
    assert!(plan.to_remove.is_empty());
    assert_eq!(plan.freed_bytes, 0);
    assert_eq!(plan.count, 0);
}

#[test]
fn test_cleanup_plan_debug() {
    let plan = CleanupPlan {
        to_remove: vec![PathBuf::from("/tmp/a")],
        freed_bytes: 1024,
        count: 1,
    };
    let _ = format!("{plan:?}");
}

#[test]
fn test_scan_old_logs_existing_dir_with_logs() {
    let temp = tempfile::tempdir().expect("temp dir");
    std::fs::write(temp.path().join("a.log"), b"log").expect("write");
    std::fs::write(temp.path().join("b.txt"), b"not log").expect("write");
    std::fs::write(temp.path().join("c.log"), b"log2").expect("write");

    let result = scan_old_logs(temp.path());
    assert!(result.is_ok());
    let logs = result.unwrap();
    assert_eq!(logs.len(), 2);
}

#[test]
fn test_scan_old_logs_empty_dir() {
    let temp = tempfile::tempdir().expect("temp dir");
    let result = scan_old_logs(temp.path());
    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());
}

#[test]
fn test_scan_old_logs_ignores_non_log() {
    let temp = tempfile::tempdir().expect("temp dir");
    std::fs::write(temp.path().join("a.txt"), b"x").expect("write");
    std::fs::write(temp.path().join("b.yaml"), b"y").expect("write");
    let result = scan_old_logs(temp.path());
    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());
}

#[test]
fn test_compute_cleanup_plan_exactly_at_cutoff() {
    let temp = tempfile::tempdir().expect("temp dir");
    let p = temp.path().join("f");
    std::fs::write(&p, b"x").expect("write");
    let t = Utc::now() - chrono::Duration::days(30);
    let fossils = vec![FossilIndexEntry {
        node_id: "n".into(),
        session_started: t,
        archival_reason: ArchivalReason::GracefulShutdown,
        fossil_path: p,
        issue_count: 0,
        encrypted: false,
    }];
    let plan = compute_cleanup_plan(&fossils, t);
    assert_eq!(plan.count, 0, "session_started < cutoff is strict");
}

#[test]
fn test_compute_cleanup_plan_just_before_cutoff() {
    let temp = tempfile::tempdir().expect("temp dir");
    let p = temp.path().join("f2");
    std::fs::write(&p, b"x").expect("write");
    let old = Utc::now() - chrono::Duration::days(31);
    let cutoff = Utc::now() - chrono::Duration::days(30);
    let fossils = vec![FossilIndexEntry {
        node_id: "n".into(),
        session_started: old,
        archival_reason: ArchivalReason::AutomaticRotation,
        fossil_path: p,
        issue_count: 1,
        encrypted: true,
    }];
    let plan = compute_cleanup_plan(&fossils, cutoff);
    assert_eq!(plan.count, 1);
}

#[test]
fn test_compute_cleanup_plan_fossil_path_nonexistent() {
    let fossils = vec![FossilIndexEntry {
        node_id: "node-old".into(),
        session_started: Utc::now() - chrono::Duration::days(60),
        archival_reason: ArchivalReason::GracefulShutdown,
        fossil_path: PathBuf::from("/nonexistent/path/12345"),
        issue_count: 0,
        encrypted: false,
    }];
    let cutoff = Utc::now() - chrono::Duration::days(30);
    let plan = compute_cleanup_plan(&fossils, cutoff);
    assert_eq!(plan.count, 0, "nonexistent path is not added to to_remove");
    assert_eq!(plan.freed_bytes, 0);
}

#[test]
fn test_compute_cleanup_plan_zero_byte_file_freed_zero() {
    let temp = tempfile::tempdir().expect("temp dir");
    let p = temp.path().join("empty-fossil");
    std::fs::write(&p, []).expect("empty");
    let fossils = vec![FossilIndexEntry {
        node_id: "n".into(),
        session_started: Utc::now() - chrono::Duration::days(90),
        archival_reason: ArchivalReason::GracefulShutdown,
        fossil_path: p,
        issue_count: 0,
        encrypted: false,
    }];
    let plan = compute_cleanup_plan(&fossils, Utc::now() - chrono::Duration::days(30));
    assert_eq!(plan.count, 1);
    assert_eq!(plan.freed_bytes, 0);
}

#[test]
fn test_compute_cleanup_plan_multiple_fossils_accumulates_bytes() {
    let temp = tempfile::tempdir().expect("temp dir");
    let p1 = temp.path().join("f1");
    let p2 = temp.path().join("f2");
    std::fs::write(&p1, vec![0u8; 100]).expect("write");
    std::fs::write(&p2, vec![0u8; 200]).expect("write");
    let old = Utc::now() - chrono::Duration::days(100);
    let fossils = vec![
        FossilIndexEntry {
            node_id: "a".into(),
            session_started: old,
            archival_reason: ArchivalReason::GracefulShutdown,
            fossil_path: p1,
            issue_count: 0,
            encrypted: false,
        },
        FossilIndexEntry {
            node_id: "b".into(),
            session_started: old,
            archival_reason: ArchivalReason::Manual,
            fossil_path: p2,
            issue_count: 1,
            encrypted: false,
        },
    ];
    let plan = compute_cleanup_plan(&fossils, Utc::now() - chrono::Duration::days(30));
    assert_eq!(plan.count, 2);
    assert_eq!(plan.freed_bytes, 300);
}

#[test]
fn test_cleanup_plan_clone() {
    let a = CleanupPlan {
        to_remove: vec![PathBuf::from("/x")],
        freed_bytes: 10,
        count: 1,
    };
    let b = a.clone();
    assert_eq!(b.count, a.count);
    assert_eq!(b.freed_bytes, a.freed_bytes);
}

#[test]
fn test_scan_old_logs_subdirectory_ignored() {
    let temp = tempfile::tempdir().expect("temp dir");
    std::fs::create_dir_all(temp.path().join("nested")).expect("dir");
    std::fs::write(temp.path().join("top.log"), b"x").expect("write");
    let logs = scan_old_logs(temp.path()).expect("scan");
    assert_eq!(logs.len(), 1);
}

#[test]
fn test_scan_old_logs_read_dir_error_unlikely() {
    let temp = tempfile::tempdir().expect("temp dir");
    let f = temp.path().join("not-a-dir");
    std::fs::write(&f, b"x").expect("write");
    let result = scan_old_logs(&f);
    assert!(result.is_err());
}
