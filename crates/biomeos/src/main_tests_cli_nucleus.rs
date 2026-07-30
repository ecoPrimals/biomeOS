// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::*;

#[test]
fn test_cli_parse_nucleus_start() {
    let cli = Cli::parse_from(["biomeos", "nucleus", "start", "--node-id", "node1"]);
    match &cli.mode {
        Mode::Nucleus { command } => match command {
            NucleusCommand::Start {
                mode,
                node_id,
                family_id,
                port,
                tcp_only,
                bind,
                bind_mode: _,
            } => {
                assert_eq!(mode, "full");
                assert_eq!(node_id, "node1");
                assert!(family_id.is_none());
                assert!(port.is_none());
                assert!(!tcp_only);
                assert!(bind.is_none());
            }
            _ => panic!("expected Nucleus Start subcommand"),
        },
        _ => panic!("expected Nucleus mode"),
    }
}

#[test]
fn test_cli_parse_nucleus_start_with_mode_and_family() {
    let cli = Cli::parse_from([
        "biomeos",
        "nucleus",
        "start",
        "--mode",
        "tower",
        "--node-id",
        "n1",
        "--family-id",
        "fam1",
    ]);
    match &cli.mode {
        Mode::Nucleus { command } => match command {
            NucleusCommand::Start {
                mode,
                node_id,
                family_id,
                ..
            } => {
                assert_eq!(mode, "tower");
                assert_eq!(node_id, "n1");
                assert_eq!(family_id.as_deref(), Some("fam1"));
            }
            _ => panic!("expected Nucleus Start subcommand"),
        },
        _ => panic!("expected Nucleus mode"),
    }
}

#[test]
fn test_cli_parse_nucleus_ingest() {
    let cli = Cli::parse_from(["biomeos", "nucleus", "ingest", "/tmp/pseudospore"]);
    match &cli.mode {
        Mode::Nucleus { command } => match command {
            NucleusCommand::Ingest {
                pseudospore_dir,
                dry_run,
                ..
            } => {
                assert_eq!(pseudospore_dir, &PathBuf::from("/tmp/pseudospore"));
                assert!(!dry_run);
            }
            _ => panic!("expected Nucleus Ingest subcommand"),
        },
        _ => panic!("expected Nucleus mode"),
    }
}

#[test]
fn test_cli_parse_nucleus_emit() {
    let cli = Cli::parse_from(["biomeos", "nucleus", "emit", "spore-abc123"]);
    match &cli.mode {
        Mode::Nucleus { command } => match command {
            NucleusCommand::Emit {
                spore_id, dry_run, ..
            } => {
                assert_eq!(spore_id, "spore-abc123");
                assert!(!dry_run);
            }
            _ => panic!("expected Nucleus Emit subcommand"),
        },
        _ => panic!("expected Nucleus mode"),
    }
}
