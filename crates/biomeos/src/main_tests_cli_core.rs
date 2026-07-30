// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::*;
use clap::CommandFactory;

#[test]
fn test_cli_parse_version() {
    let cli = Cli::parse_from(["biomeos", "version"]);
    match &cli.mode {
        Mode::Version { detailed } => assert!(!detailed),
        _ => panic!("expected Version mode"),
    }
}

#[test]
fn test_cli_parse_version_detailed() {
    let cli = Cli::parse_from(["biomeos", "version", "--detailed"]);
    match &cli.mode {
        Mode::Version { detailed } => assert!(*detailed),
        _ => panic!("expected Version mode"),
    }
}

#[test]
fn test_cli_parse_doctor() {
    let cli = Cli::parse_from(["biomeos", "doctor"]);
    match &cli.mode {
        Mode::Doctor { format, .. } => assert_eq!(format, "text"),
        _ => panic!("expected Doctor mode"),
    }
}

#[test]
fn test_cli_parse_doctor_json_format() {
    let cli = Cli::parse_from(["biomeos", "doctor", "-f", "json"]);
    match &cli.mode {
        Mode::Doctor { format, .. } => assert_eq!(format, "json"),
        _ => panic!("expected Doctor mode"),
    }
}

#[test]
fn test_cli_parse_verbose_and_log_level() {
    let cli = Cli::parse_from(["biomeos", "--verbose", "--log-level", "debug", "version"]);
    assert!(cli.verbose);
    assert_eq!(cli.log_level, "debug");
}

#[test]
fn test_cli_parse_cli_mode() {
    let cli = Cli::parse_from(["biomeos", "cli"]);
    match &cli.mode {
        Mode::Cli {} => {}
        _ => panic!("expected Cli mode"),
    }
}

#[test]
fn test_init_logging_verbose_overrides_level() {
    let result = init_logging("warn", true);
    assert!(result.is_ok());
}

#[test]
fn test_init_logging() {
    let result = init_logging("warn", false);
    assert!(result.is_ok());
}

#[test]
fn test_cli_help_contains_biomeos() {
    let help = Cli::command().render_help().to_string();
    assert!(help.contains("biomeOS"), "Help should mention biomeOS");
    assert!(help.contains("nucleus"), "Help should mention nucleus");
    assert!(help.contains("version"), "Help should mention version");
}

#[test]
fn test_cli_version_output() {
    let version = Cli::command().render_version();
    assert!(!version.is_empty());
    assert!(version.contains('.'), "Version should have semver format");
}
