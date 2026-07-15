// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::stop_tower;

#[test]
fn stop_tower_errors_when_no_pid_file() {
    let dir = tempfile::tempdir().unwrap();
    let pid_path = dir.path().join("tower.pid");
    assert!(stop_tower(&pid_path).is_err());
}

#[cfg(unix)]
#[test]
fn stop_tower_rejects_non_positive_pid_in_file() {
    let dir = tempfile::tempdir().unwrap();
    let pid_path = dir.path().join("tower.pid");
    std::fs::write(&pid_path, "-1").unwrap();
    assert!(stop_tower(&pid_path).is_err());
}

#[test]
fn stop_tower_rejects_zero_pid() {
    let dir = tempfile::tempdir().unwrap();
    let pid_path = dir.path().join("tower.pid");
    std::fs::write(&pid_path, "0").unwrap();
    let err = stop_tower(&pid_path).unwrap_err();
    assert!(err.to_string().contains("Invalid PID"), "{err}");
}

#[cfg(unix)]
#[test]
fn stop_tower_cleans_up_stale_pid() {
    let dir = tempfile::tempdir().unwrap();
    let pid_path = dir.path().join("tower.pid");
    std::fs::write(&pid_path, "2147483646").unwrap();
    let _ = stop_tower(&pid_path);
    assert!(!pid_path.exists());
}
