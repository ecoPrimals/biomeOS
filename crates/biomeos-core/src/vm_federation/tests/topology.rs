// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use std::path::{Path, PathBuf};

use super::topology_path_for_cli;

#[test]
fn test_topology_path_for_cli_valid() {
    let path = Path::new("/tmp/topology.yaml");
    assert_eq!(topology_path_for_cli(path).unwrap(), "/tmp/topology.yaml");
}

#[cfg(unix)]
#[test]
fn test_topology_path_for_cli_rejects_non_utf8_path() {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;
    let path = PathBuf::from(OsString::from_vec(vec![0xFF, 0xFE, 0xFD]));
    let err = topology_path_for_cli(&path).unwrap_err();
    assert!(err.to_string().contains("UTF-8") || err.to_string().contains("utf-8"));
}
