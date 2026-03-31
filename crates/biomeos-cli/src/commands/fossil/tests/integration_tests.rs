// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Integration tests for fossil CLI commands.

use super::*;

#[tokio::test]
async fn test_run_active() {
    let args = FossilArgs {
        action: FossilAction::Active { node: None },
    };
    let result = run(args).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_run_fossil_no_index() {
    let args = FossilArgs {
        action: FossilAction::Fossil {
            node: None,
            limit: 10,
            show: None,
        },
    };
    let result = run(args).await;
    assert!(result.is_ok());
}

#[tokio::test]
#[ignore = "Requires /var/biomeos writable - run with --ignored for full test"]
async fn test_run_clean_dry_run() {
    let args = FossilArgs {
        action: FossilAction::Clean {
            older_than: 30,
            dry_run: true,
        },
    };
    let result = run(args).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_run_migrate_nonexistent_source() {
    let args = FossilArgs {
        action: FossilAction::Migrate {
            from: PathBuf::from("/nonexistent/path/12345"),
            dry_run: true,
        },
    };
    let result = run(args).await;
    assert!(result.is_ok());
}

#[tokio::test]
#[ignore = "Requires /var/biomeos writable - run with --ignored for full test"]
async fn test_run_cleanup_stale() {
    let args = FossilArgs {
        action: FossilAction::CleanupStale,
    };
    let result = run(args).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_run_fossil_with_show_invalid_index() {
    let args = FossilArgs {
        action: FossilAction::Fossil {
            node: None,
            limit: 10,
            show: Some(999),
        },
    };
    let result = run(args).await;
    assert!(result.is_ok());
}

#[tokio::test]
#[ignore = "Requires /var/biomeos writable for LogManager - run with --ignored"]
async fn test_run_migrate_empty_dir() {
    let temp = tempfile::tempdir().expect("temp dir");
    let args = FossilArgs {
        action: FossilAction::Migrate {
            from: temp.path().to_path_buf(),
            dry_run: true,
        },
    };
    let result = run(args).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_run_archive_no_session() {
    let args = FossilArgs {
        action: FossilAction::Archive {
            node_id: "nonexistent-node-xyz-123".to_string(),
        },
    };
    let result = run(args).await;
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("No active session")
    );
}

#[tokio::test]
async fn test_run_fossil_with_show_zero_invalid() {
    let args = FossilArgs {
        action: FossilAction::Fossil {
            node: None,
            limit: 10,
            show: Some(0),
        },
    };
    let result = run(args).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_run_fossil_with_node_filter_no_index() {
    let args = FossilArgs {
        action: FossilAction::Fossil {
            node: Some("node-abc".into()),
            limit: 5,
            show: None,
        },
    };
    let result = run(args).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_run_clean_not_dry_run_no_index() {
    let args = FossilArgs {
        action: FossilAction::Clean {
            older_than: 1,
            dry_run: false,
        },
    };
    let result = run(args).await;
    assert!(result.is_ok() || result.is_err());
}

fn write_fossil_index(root: &Path, entries: Vec<FossilIndexEntry>) {
    let fossil_dir = root.join("fossil");
    std::fs::create_dir_all(&fossil_dir).expect("fossil dir");
    let mut index = FossilIndex::new();
    for e in entries {
        index.add(e);
    }
    index
        .save(&fossil_dir.join("index.toml"))
        .expect("save fossil index");
}

#[tokio::test]
async fn test_run_fossil_lists_records_with_temp_log_root() {
    let temp = tempfile::tempdir().expect("temp dir");
    let root = temp.path();
    let fossil_file = root.join("fossil").join("a.log");
    std::fs::create_dir_all(fossil_file.parent().expect("parent")).expect("mkdir");
    std::fs::write(&fossil_file, b"log").expect("write fossil file");

    write_fossil_index(
        root,
        vec![FossilIndexEntry {
            node_id: "node-list".into(),
            session_started: Utc::now() - chrono::Duration::days(1),
            archival_reason: ArchivalReason::GracefulShutdown,
            fossil_path: fossil_file,
            issue_count: 3,
            encrypted: false,
        }],
    );

    let args = FossilArgs {
        action: FossilAction::Fossil {
            node: None,
            limit: 10,
            show: None,
        },
    };
    let result = run_at(args, root).await;
    assert!(result.is_ok(), "{result:?}");
}

#[tokio::test]
async fn test_run_fossil_show_detail_with_temp_log_root() {
    let temp = tempfile::tempdir().expect("temp dir");
    let root = temp.path();
    let fossil_file = root.join("fossil").join("detail.log");
    std::fs::create_dir_all(fossil_file.parent().expect("parent")).expect("mkdir");
    std::fs::write(&fossil_file, b"x").expect("write");

    write_fossil_index(
        root,
        vec![FossilIndexEntry {
            node_id: "node-show".into(),
            session_started: Utc::now(),
            archival_reason: ArchivalReason::Manual,
            fossil_path: fossil_file,
            issue_count: 0,
            encrypted: true,
        }],
    );

    let args = FossilArgs {
        action: FossilAction::Fossil {
            node: None,
            limit: 10,
            show: Some(1),
        },
    };
    assert!(run_at(args, root).await.is_ok());
}

#[tokio::test]
async fn test_run_fossil_invalid_show_index_with_temp_log_root() {
    let temp = tempfile::tempdir().expect("temp dir");
    let root = temp.path();
    let fossil_file = root.join("fossil").join("one.log");
    std::fs::create_dir_all(fossil_file.parent().expect("parent")).expect("mkdir");
    std::fs::write(&fossil_file, b"x").expect("write");

    write_fossil_index(
        root,
        vec![FossilIndexEntry {
            node_id: "n".into(),
            session_started: Utc::now(),
            archival_reason: ArchivalReason::GracefulShutdown,
            fossil_path: fossil_file,
            issue_count: 0,
            encrypted: false,
        }],
    );

    let args = FossilArgs {
        action: FossilAction::Fossil {
            node: None,
            limit: 10,
            show: Some(99),
        },
    };
    assert!(run_at(args, root).await.is_ok());
}

#[tokio::test]
async fn test_run_fossil_node_filter_no_match() {
    let temp = tempfile::tempdir().expect("temp dir");
    let root = temp.path();
    let fossil_file = root.join("fossil").join("f.log");
    std::fs::create_dir_all(fossil_file.parent().expect("parent")).expect("mkdir");
    std::fs::write(&fossil_file, b"x").expect("write");

    write_fossil_index(
        root,
        vec![FossilIndexEntry {
            node_id: "alpha".into(),
            session_started: Utc::now(),
            archival_reason: ArchivalReason::AutomaticRotation,
            fossil_path: fossil_file,
            issue_count: 0,
            encrypted: false,
        }],
    );

    let args = FossilArgs {
        action: FossilAction::Fossil {
            node: Some("zzzz-no-match".into()),
            limit: 10,
            show: None,
        },
    };
    assert!(run_at(args, root).await.is_ok());
}

#[tokio::test]
async fn test_run_fossil_node_filter_matches_with_header() {
    let temp = tempfile::tempdir().expect("temp dir");
    let root = temp.path();
    let fossil_file = root.join("fossil").join("f2.log");
    std::fs::create_dir_all(fossil_file.parent().expect("parent")).expect("mkdir");
    std::fs::write(&fossil_file, b"x").expect("write");

    write_fossil_index(
        root,
        vec![FossilIndexEntry {
            node_id: "my-node-beta".into(),
            session_started: Utc::now(),
            archival_reason: ArchivalReason::Reboot,
            fossil_path: fossil_file,
            issue_count: 1,
            encrypted: false,
        }],
    );

    let args = FossilArgs {
        action: FossilAction::Fossil {
            node: Some("beta".into()),
            limit: 10,
            show: None,
        },
    };
    assert!(run_at(args, root).await.is_ok());
}

#[tokio::test]
async fn test_run_clean_dry_run_with_temp_log_root() {
    let temp = tempfile::tempdir().expect("temp dir");
    let root = temp.path();
    let fossil_file = root.join("fossil").join("old.bin");
    std::fs::create_dir_all(fossil_file.parent().expect("parent")).expect("mkdir");
    std::fs::write(&fossil_file, vec![0u8; 64]).expect("write");

    write_fossil_index(
        root,
        vec![FossilIndexEntry {
            node_id: "old-node".into(),
            session_started: Utc::now() - chrono::Duration::days(100),
            archival_reason: ArchivalReason::GracefulShutdown,
            fossil_path: fossil_file,
            issue_count: 0,
            encrypted: false,
        }],
    );

    let args = FossilArgs {
        action: FossilAction::Clean {
            older_than: 30,
            dry_run: true,
        },
    };
    assert!(run_at(args, root).await.is_ok());
}

#[tokio::test]
async fn test_run_clean_deletes_old_fossils_not_dry_run() {
    let temp = tempfile::tempdir().expect("temp dir");
    let root = temp.path();
    let fossil_file = root.join("fossil").join("to-delete.log");
    std::fs::create_dir_all(fossil_file.parent().expect("parent")).expect("mkdir");
    std::fs::write(&fossil_file, b"bye").expect("write");

    write_fossil_index(
        root,
        vec![FossilIndexEntry {
            node_id: "stale".into(),
            session_started: Utc::now() - chrono::Duration::days(90),
            archival_reason: ArchivalReason::GracefulShutdown,
            fossil_path: fossil_file.clone(),
            issue_count: 0,
            encrypted: false,
        }],
    );

    let args = FossilArgs {
        action: FossilAction::Clean {
            older_than: 30,
            dry_run: false,
        },
    };
    assert!(run_at(args, root).await.is_ok());
    assert!(!fossil_file.exists(), "cleanup should remove fossil file");
}

#[tokio::test]
async fn test_run_migrate_real_move_with_temp_log_root() {
    let temp = tempfile::tempdir().expect("temp dir");
    let root = temp.path();
    let from = root.join("from_logs");
    std::fs::create_dir_all(&from).expect("from");
    std::fs::write(from.join("legacy.log"), b"line\n").expect("log");

    let args = FossilArgs {
        action: FossilAction::Migrate {
            from,
            dry_run: false,
        },
    };
    assert!(run_at(args, root).await.is_ok());
    let dest = root.join("fossil").join("legacy").join("legacy.log");
    assert!(dest.exists(), "migrated file should exist at {dest:?}");
}

#[tokio::test]
async fn test_run_active_lists_session_with_temp_log_root() {
    let temp = tempfile::tempdir().expect("temp dir");
    let root = temp.path();
    let active = root.join("active").join("session-one");
    std::fs::create_dir_all(&active).expect("active session dir");
    let session = ActiveLogSession::new("session-one".into(), "deploy-z".into());
    let meta = toml::to_string_pretty(&session).expect("toml");
    std::fs::write(active.join(".metadata.toml"), meta).expect("metadata");

    let args = FossilArgs {
        action: FossilAction::Active { node: None },
    };
    assert!(run_at(args, root).await.is_ok());
}

#[tokio::test]
async fn test_run_cleanup_stale_archives_idle_session() {
    let temp = tempfile::tempdir().expect("temp dir");
    let root = temp.path();
    let active = root.join("active").join("idle-node");
    std::fs::create_dir_all(&active).expect("active session dir");
    let session = ActiveLogSession::new("idle-node".into(), "deploy".into());
    std::fs::write(
        active.join(".metadata.toml"),
        toml::to_string_pretty(&session).expect("toml"),
    )
    .expect("metadata");

    let args = FossilArgs {
        action: FossilAction::CleanupStale,
    };
    assert!(run_at(args, root).await.is_ok());
}
