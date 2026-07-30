// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::*;

#[test]
fn test_cli_parse_fails_nucleus_missing_subcommand() {
    let result = Cli::try_parse_from(["biomeos", "nucleus"]);
    assert!(result.is_err(), "nucleus without subcommand should fail");
}

#[test]
fn test_cli_parse_fails_nucleus_start_missing_node_id() {
    let result = Cli::try_parse_from(["biomeos", "nucleus", "start"]);
    assert!(result.is_err());
    let err = match result {
        Ok(_) => panic!("expected parse failure"),
        Err(e) => e.to_string(),
    };
    assert!(
        err.contains("node-id") || err.contains("node_id") || err.contains("required"),
        "Expected missing node-id error: {err}"
    );
}

#[test]
fn test_cli_parse_fails_unknown_subcommand() {
    let result = Cli::try_parse_from(["biomeos", "unknown-mode-xyz"]);
    assert!(result.is_err());
}

#[test]
fn test_cli_parse_fails_deploy_missing_graph() {
    let result = Cli::try_parse_from(["biomeos", "deploy"]);
    assert!(result.is_err());
}

#[test]
fn test_cli_parse_fails_rootpulse_commit_missing_args() {
    let result = Cli::try_parse_from(["biomeos", "rootpulse", "commit"]);
    assert!(result.is_err());
}
