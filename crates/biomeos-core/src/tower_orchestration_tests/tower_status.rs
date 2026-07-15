// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::common::mock_env;
use super::super::{tower_status, TowerStatusReport};
use std::collections::HashMap;
use std::path::PathBuf;

#[test]
fn tower_status_not_running_when_no_pid_file() {
    let dir = tempfile::tempdir().unwrap();
    let pid_path = dir.path().join("tower.pid");
    let env: HashMap<String, String> = HashMap::new();

    let status = tower_status(&pid_path, &mock_env(&env)).unwrap();
    assert!(matches!(status, TowerStatusReport::NotRunning));
}

#[test]
fn tower_status_invalid_pid_zero() {
    let dir = tempfile::tempdir().unwrap();
    let pid_path = dir.path().join("tower.pid");
    std::fs::write(&pid_path, "0").unwrap();
    let env: HashMap<String, String> = HashMap::new();
    let status = tower_status(&pid_path, &mock_env(&env)).unwrap();
    assert!(matches!(status, TowerStatusReport::InvalidPid));
}

#[test]
fn tower_status_invalid_pid_negative() {
    let dir = tempfile::tempdir().unwrap();
    let pid_path = dir.path().join("tower.pid");
    std::fs::write(&pid_path, "-3").unwrap();
    let env: HashMap<String, String> = HashMap::new();
    let status = tower_status(&pid_path, &mock_env(&env)).unwrap();
    assert!(matches!(status, TowerStatusReport::InvalidPid));
}

#[cfg(unix)]
#[test]
fn tower_status_running_for_current_process() {
    let dir = tempfile::tempdir().unwrap();
    let pid_path = dir.path().join("tower.pid");
    #[expect(
        clippy::cast_possible_wrap,
        reason = "PID fits i32 on all supported platforms"
    )]
    let my_pid = std::process::id() as i32;
    std::fs::write(&pid_path, my_pid.to_string()).unwrap();
    let mut env = HashMap::new();
    env.insert("BIOMEOS_FAMILY_ID".to_string(), "fam-x".to_string());
    let status = tower_status(&pid_path, &mock_env(&env)).unwrap();
    match status {
        TowerStatusReport::Running {
            pid,
            family_id,
            socket_dir,
            ..
        } => {
            assert_eq!(pid, my_pid);
            assert_eq!(family_id.as_deref(), Some("fam-x"));
            assert!(!socket_dir.as_os_str().is_empty());
        }
        other => panic!("expected Running, got {other:?}"),
    }
}

#[cfg(unix)]
#[test]
fn tower_status_stale_when_pid_not_running() {
    let dir = tempfile::tempdir().unwrap();
    let pid_path = dir.path().join("tower.pid");
    std::fs::write(&pid_path, "2147483645").unwrap();
    let env: HashMap<String, String> = HashMap::new();
    let status = tower_status(&pid_path, &mock_env(&env)).unwrap();
    match status {
        TowerStatusReport::Stale { pid } => assert_eq!(pid, 2_147_483_645),
        other => panic!("expected Stale, got {other:?}"),
    }
    assert!(!pid_path.exists());
}

#[cfg(not(unix))]
#[test]
fn tower_status_non_unix_skips_ps_check() {
    let dir = tempfile::tempdir().unwrap();
    let pid_path = dir.path().join("tower.pid");
    std::fs::write(&pid_path, "9999").unwrap();
    let env: HashMap<String, String> = HashMap::new();
    let status = tower_status(&pid_path, &mock_env(&env)).unwrap();
    match status {
        TowerStatusReport::Running {
            pid,
            sockets,
            family_id,
            ..
        } => {
            assert_eq!(pid, 9999);
            assert!(sockets.is_empty());
            assert!(family_id.is_none());
        }
        other => panic!("expected Running, got {other:?}"),
    }
}

#[test]
fn tower_status_report_debug_formatting() {
    let not_running = TowerStatusReport::NotRunning;
    assert!(format!("{not_running:?}").contains("NotRunning"));

    let invalid = TowerStatusReport::InvalidPid;
    assert!(format!("{invalid:?}").contains("InvalidPid"));

    let stale = TowerStatusReport::Stale { pid: 999 };
    let stale_dbg = format!("{stale:?}");
    assert!(stale_dbg.contains("Stale"));
    assert!(stale_dbg.contains("999"));

    let running = TowerStatusReport::Running {
        pid: 42,
        socket_dir: PathBuf::from("/tmp/sockets"),
        sockets: vec!["test.sock".to_string()],
        family_id: Some("fam-z".to_string()),
    };
    let running_dbg = format!("{running:?}");
    assert!(running_dbg.contains("Running"));
    assert!(running_dbg.contains("42"));
    assert!(running_dbg.contains("fam-z"));
}

#[cfg(unix)]
#[test]
fn tower_status_running_includes_family_id_from_env() {
    let dir = tempfile::tempdir().unwrap();
    let pid_path = dir.path().join("tower.pid");
    #[expect(
        clippy::cast_possible_wrap,
        reason = "PID fits i32 on all supported platforms"
    )]
    let my_pid = std::process::id() as i32;
    std::fs::write(&pid_path, my_pid.to_string()).unwrap();
    let mut env = HashMap::new();
    env.insert("FAMILY_ID".to_string(), "fam-only".to_string());
    let status = tower_status(&pid_path, &mock_env(&env)).unwrap();
    match status {
        TowerStatusReport::Running { family_id, .. } => {
            assert_eq!(family_id.as_deref(), Some("fam-only"));
        }
        other => panic!("expected Running, got {other:?}"),
    }
}

#[test]
fn std_env_lookup_missing_var_returns_none() {
    assert!(super::super::std_env_lookup("BIOMEOS_TEST_ENV_KEY_THAT_SHOULD_NOT_EXIST_XYZ").is_none());
}
