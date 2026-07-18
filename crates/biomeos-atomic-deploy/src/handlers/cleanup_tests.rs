// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Tests for `handlers/cleanup.rs` (`cleanup.sockets`).

#![expect(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

use serde_json::json;
use tempfile::tempdir;

use super::cleanup::CleanupHandler;

#[cfg(unix)]
#[tokio::test]
async fn cleanup_removes_stale_sockets() {
    let temp = tempdir().expect("temp dir");
    let stale = temp.path().join("stale-fam1.sock");
    std::fs::write(&stale, "").expect("create stale socket file");
    let stale_pid = temp.path().join("stale-fam1.pid");
    std::fs::write(&stale_pid, "99999").expect("create stale pid file");

    let result = CleanupHandler::cleanup_sockets(Some(&json!({
        "socket_dir": temp.path().to_string_lossy(),
    })))
    .await
    .expect("cleanup should succeed");

    let removed = result["removed"].as_array().expect("removed array");
    assert_eq!(removed.len(), 1);
    assert!(removed[0].as_str().unwrap().ends_with("stale-fam1.sock"));
    assert!(!stale.exists(), "stale socket should be removed");
    assert!(!stale_pid.exists(), "companion pid file should be removed");
    assert!(result["active"].as_array().unwrap().is_empty());
}

#[cfg(unix)]
#[tokio::test]
async fn cleanup_preserves_live_sockets() {
    let temp = tempdir().expect("temp dir");
    let live_path = temp.path().join("live-fam1.sock");
    let _listener = std::os::unix::net::UnixListener::bind(&live_path).expect("bind socket");

    let result = CleanupHandler::cleanup_sockets(Some(&json!({
        "socket_dir": temp.path().to_string_lossy(),
    })))
    .await
    .expect("cleanup should succeed");

    let active = result["active"].as_array().expect("active array");
    assert_eq!(active.len(), 1);
    assert!(active[0].as_str().unwrap().ends_with("live-fam1.sock"));
    assert!(live_path.exists(), "live socket should remain");
    assert!(result["removed"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn cleanup_nonexistent_dir_returns_empty() {
    let temp = tempdir().expect("temp dir");
    let missing = temp.path().join("does_not_exist");

    let result = CleanupHandler::cleanup_sockets(Some(&json!({
        "socket_dir": missing.to_string_lossy(),
    })))
    .await
    .expect("cleanup should succeed for missing dir");

    assert!(result["removed"].as_array().unwrap().is_empty());
    assert!(result["active"].as_array().unwrap().is_empty());
}
