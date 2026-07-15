// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::benchscale_create_argv;
use super::benchscale_subcommand_argv;

#[test]
fn test_benchscale_create_argv() {
    let argv = benchscale_create_argv("my-fed", "/tmp/topology.yaml");
    assert_eq!(argv[0], "run");
    assert_eq!(argv[3], "create");
    assert_eq!(argv[4], "my-fed");
    assert_eq!(argv[6], "/tmp/topology.yaml");
    assert_eq!(argv.len(), 9);
}

#[test]
fn test_benchscale_subcommand_argv() {
    for cmd in ["start", "stop", "destroy", "test", "status"] {
        let argv = benchscale_subcommand_argv(cmd, "fed1");
        assert_eq!(argv[3], cmd);
        assert_eq!(argv[4], "fed1");
        assert_eq!(argv.len(), 5);
    }
}
