// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::*;

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
fn test_format_session_display_no_logs_no_pids() {
    let session = ActiveLogSession::new("node-minimal".into(), "deploy-1".into());
    let lines = format_session_display(&session);
    assert!(lines.iter().any(|l| l.contains("node-minimal")));
    assert!(!lines.iter().any(|l| l.contains("PIDs:")));
    assert!(!lines.iter().any(|l| l.contains("Logs:")));
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
fn test_filter_sessions_empty_input() {
    let sessions: Vec<ActiveLogSession> = vec![];
    assert!(filter_sessions(&sessions, None).is_empty());
    assert!(filter_sessions(&sessions, Some("x")).is_empty());
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
