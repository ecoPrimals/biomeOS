// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Pure unit tests for fossil formatting, filtering, cleanup planning, and scanning.
//! No I/O side-effects, no env mutation, no serialization constraints.

use super::*;

#[test]
fn test_filter_sessions_no_filter() {
    let sessions = vec![
        ActiveLogSession::new("node-1".into(), "deploy-1".into()),
        ActiveLogSession::new("node-2".into(), "deploy-1".into()),
    ];
    let filtered = filter_sessions(&sessions, None);
    assert_eq!(filtered.len(), 2);
}

#[test]
fn test_filter_sessions_with_filter() {
    let sessions = vec![
        ActiveLogSession::new("node-1".into(), "deploy-1".into()),
        ActiveLogSession::new("node-2".into(), "deploy-1".into()),
        ActiveLogSession::new("node-10".into(), "deploy-1".into()),
    ];
    let filtered = filter_sessions(&sessions, Some("node-1"));
    assert_eq!(filtered.len(), 2); // node-1 and node-10
}

#[test]
fn test_filter_sessions_empty_match() {
    let sessions = vec![ActiveLogSession::new("node-1".into(), "deploy-1".into())];
    let filtered = filter_sessions(&sessions, Some("node-99"));
    assert!(filtered.is_empty());
}

#[test]
fn test_format_session_display() {
    let mut session = ActiveLogSession::new("node-1".into(), "deploy-1".into());
    session.add_process(1234);
    session.add_log_file(LogFile {
        primal: "tower".into(),
        path: PathBuf::from("/tmp/tower.log"),
        pid: Some(1234),
        size_bytes: 1024,
        last_modified: Utc::now(),
    });

    let lines = format_session_display(&session);
    assert!(lines.iter().any(|l| l.contains("node-1")));
    assert!(lines.iter().any(|l| l.contains("PIDs")));
    assert!(lines.iter().any(|l| l.contains("tower")));
}

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
fn test_format_session_display_no_logs_no_pids() {
    let session = ActiveLogSession::new("node-minimal".into(), "deploy-1".into());
    let lines = format_session_display(&session);
    assert!(lines.iter().any(|l| l.contains("node-minimal")));
    assert!(!lines.iter().any(|l| l.contains("PIDs:")));
    assert!(!lines.iter().any(|l| l.contains("Logs:")));
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
fn test_fossil_args_debug() {
    let args = FossilArgs {
        action: FossilAction::Active { node: None },
    };
    let _ = format!("{args:?}");
}

#[test]
fn test_fossil_action_variants_debug() {
    let _ = format!(
        "{:?}",
        FossilAction::Fossil {
            node: Some("n".into()),
            limit: 5,
            show: Some(1),
        }
    );
    let _ = format!(
        "{:?}",
        FossilAction::Archive {
            node_id: "n".into()
        }
    );
    let _ = format!(
        "{:?}",
        FossilAction::Clean {
            older_than: 30,
            dry_run: true,
        }
    );
    let _ = format!(
        "{:?}",
        FossilAction::Migrate {
            from: PathBuf::from("/tmp"),
            dry_run: false,
        }
    );
    let _ = format!("{:?}", FossilAction::CleanupStale);
}

#[test]
fn test_format_session_display_multiple_logs() {
    let mut session = ActiveLogSession::new("node-multi".into(), "deploy-1".into());
    session.add_process(1111);
    session.add_log_file(LogFile {
        primal: "tower".into(),
        path: PathBuf::from("/tmp/tower.log"),
        pid: Some(1111),
        size_bytes: 2048,
        last_modified: Utc::now(),
    });
    session.add_log_file(LogFile {
        primal: "beardog".into(),
        path: PathBuf::from("/tmp/beardog.log"),
        pid: None,
        size_bytes: 512,
        last_modified: Utc::now(),
    });
    let lines = format_session_display(&session);
    assert!(lines.iter().any(|l| l.contains("tower")));
    assert!(lines.iter().any(|l| l.contains("beardog")));
    assert!(lines.iter().any(|l| l.contains("closed")));
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
fn test_filter_sessions_empty_input() {
    let sessions: Vec<ActiveLogSession> = vec![];
    assert!(filter_sessions(&sessions, None).is_empty());
    assert!(filter_sessions(&sessions, Some("x")).is_empty());
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
fn test_format_session_display_log_closed_no_pid() {
    let mut session = ActiveLogSession::new("n".into(), "d".into());
    session.add_log_file(LogFile {
        primal: "p".into(),
        path: PathBuf::from("/tmp/p.log"),
        pid: None,
        size_bytes: 4096,
        last_modified: Utc::now(),
    });
    let lines = format_session_display(&session);
    assert!(lines.iter().any(|l| l.contains("closed")));
    assert!(
        lines
            .iter()
            .any(|l| l.contains("4 KB") || l.contains("3 KB"))
    );
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
fn test_format_session_display_zero_kb_log() {
    let mut session = ActiveLogSession::new("n".into(), "d".into());
    session.add_log_file(LogFile {
        primal: "tiny".into(),
        path: PathBuf::from("/tmp/tiny.log"),
        pid: Some(1),
        size_bytes: 0,
        last_modified: Utc::now(),
    });
    let lines = format_session_display(&session);
    assert!(lines.iter().any(|l| l.contains("0 KB")));
}

#[test]
fn test_filter_sessions_filter_empty_string_matches_all() {
    let sessions = vec![ActiveLogSession::new("a".into(), "d".into())];
    let filtered = filter_sessions(&sessions, Some(""));
    assert_eq!(filtered.len(), 1);
}

#[test]
fn test_format_session_display_duration_hours_only() {
    use chrono::Duration as ChronoDuration;
    let mut session = ActiveLogSession::new("dur-node".into(), "deploy-1".into());
    session.started_at = Utc::now() - ChronoDuration::hours(3);
    let lines = format_session_display(&session);
    assert!(lines.iter().any(|l| l.contains("3h")));
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
fn test_format_session_display_many_pids() {
    let mut session = ActiveLogSession::new("pid-node".into(), "d".into());
    for p in [1u32, 2, 3, 4, 5] {
        session.add_process(p);
    }
    let lines = format_session_display(&session);
    assert!(lines.iter().any(|l| l.contains("PIDs:")));
    assert!(lines.iter().any(|l| l.contains('5')));
}

#[test]
fn test_filter_sessions_overlapping_substrings() {
    let sessions = vec![
        ActiveLogSession::new("alpha-node".into(), "d".into()),
        ActiveLogSession::new("alphabet-extra".into(), "d".into()),
    ];
    let filtered = filter_sessions(&sessions, Some("alpha"));
    assert_eq!(filtered.len(), 2);
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
