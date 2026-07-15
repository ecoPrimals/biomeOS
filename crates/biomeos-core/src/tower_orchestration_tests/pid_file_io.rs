// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::{cleanup_pid_file, read_pid, write_pid_file};

#[test]
fn write_and_read_pid_file() {
    let dir = tempfile::tempdir().unwrap();
    let pid_path = dir.path().join("tower.pid");

    write_pid_file(&pid_path).unwrap();

    let pid = read_pid(&pid_path).unwrap();
    #[expect(
        clippy::cast_possible_wrap,
        reason = "PID fits i32 on all supported platforms"
    )]
    let expected = std::process::id() as i32;
    assert_eq!(pid, expected);
}

#[test]
fn cleanup_pid_file_removes_file() {
    let dir = tempfile::tempdir().unwrap();
    let pid_path = dir.path().join("tower.pid");
    std::fs::write(&pid_path, "12345").unwrap();

    assert!(pid_path.exists());
    cleanup_pid_file(&pid_path);
    assert!(!pid_path.exists());
}

#[test]
fn cleanup_pid_file_noop_if_missing() {
    let dir = tempfile::tempdir().unwrap();
    let pid_path = dir.path().join("nonexistent.pid");
    cleanup_pid_file(&pid_path);
}

#[test]
fn read_pid_returns_error_on_invalid_content() {
    let dir = tempfile::tempdir().unwrap();
    let pid_path = dir.path().join("bad.pid");
    std::fs::write(&pid_path, "not-a-number").unwrap();

    assert!(read_pid(&pid_path).is_err());
}

#[test]
fn read_pid_returns_error_on_missing_file() {
    let dir = tempfile::tempdir().unwrap();
    let pid_path = dir.path().join("missing.pid");
    assert!(read_pid(&pid_path).is_err());
}

#[test]
fn read_pid_trims_whitespace() {
    let dir = tempfile::tempdir().unwrap();
    let pid_path = dir.path().join("tower.pid");
    std::fs::write(&pid_path, "  424242 \n").unwrap();
    assert_eq!(read_pid(&pid_path).unwrap(), 424_242);
}

#[test]
fn write_pid_file_creates_nested_parent_directories() {
    let dir = tempfile::tempdir().unwrap();
    let pid_path = dir.path().join("a").join("b").join("c").join("tower.pid");
    write_pid_file(&pid_path).unwrap();
    assert!(pid_path.exists());
    let pid = read_pid(&pid_path).unwrap();
    #[expect(
        clippy::cast_possible_wrap,
        reason = "PID fits i32 on all supported platforms"
    )]
    let expected = std::process::id() as i32;
    assert_eq!(pid, expected);
}

#[test]
fn read_pid_parses_large_pid() {
    let dir = tempfile::tempdir().unwrap();
    let pid_path = dir.path().join("tower.pid");
    std::fs::write(&pid_path, "2147483647").unwrap();
    let pid = read_pid(&pid_path).unwrap();
    assert_eq!(pid, i32::MAX);
}

#[test]
fn read_pid_rejects_empty_file() {
    let dir = tempfile::tempdir().unwrap();
    let pid_path = dir.path().join("tower.pid");
    std::fs::write(&pid_path, "").unwrap();
    assert!(read_pid(&pid_path).is_err());
}
