// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::*;

#[test]
fn test_get_socket_dir_returns_valid_path() {
    let socket_dir = get_socket_dir();
    // Should return a path that contains "biomeos" or is a valid directory pattern
    assert!(
        socket_dir.contains("biomeos") || socket_dir.starts_with('/'),
        "Socket dir should be valid path: {socket_dir}"
    );
}

#[test]
fn test_get_socket_dir_resolves() {
    // Verify socket directory resolution works (uses 5-tier strategy)
    let socket_dir = get_socket_dir();
    assert!(!socket_dir.is_empty(), "Socket dir should not be empty");
}

#[test]
fn test_get_socket_dir_respects_primal_socket_env() {
    let temp = tempfile::tempdir().expect("tempdir");

    let dir = get_socket_dir_from(Some(temp.path().to_str().expect("utf8")));
    assert!(
        dir.starts_with(temp.path().to_str().unwrap()),
        "socket dir should use PRIMAL_SOCKET override: got {dir}"
    );
}
