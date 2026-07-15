// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::list_active_sockets;
use std::path::Path;

#[test]
fn list_active_sockets_finds_sock_files() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("beardog-nat0.sock"), "").unwrap();
    std::fs::write(dir.path().join("songbird-nat0.sock"), "").unwrap();
    std::fs::write(dir.path().join("other.txt"), "").unwrap();

    let sockets = list_active_sockets(dir.path());
    assert_eq!(sockets.len(), 2);
    assert!(sockets.iter().any(|s| s.contains("beardog")));
    assert!(sockets.iter().any(|s| s.contains("songbird")));
}

#[test]
fn list_active_sockets_empty_on_missing_dir() {
    let sockets = list_active_sockets(Path::new("/nonexistent/path"));
    assert!(sockets.is_empty());
}

#[test]
fn list_active_sockets_ignores_non_sock_files() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("config.toml"), "").unwrap();
    std::fs::write(dir.path().join("data.json"), "").unwrap();
    std::fs::write(dir.path().join("README.md"), "").unwrap();
    let sockets = list_active_sockets(dir.path());
    assert!(sockets.is_empty());
}

#[test]
fn list_active_sockets_empty_directory() {
    let dir = tempfile::tempdir().unwrap();
    let sockets = list_active_sockets(dir.path());
    assert!(sockets.is_empty());
}
