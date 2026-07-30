// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::*;

#[test]
fn test_cli_parse_model_cache_list() {
    let cli = Cli::parse_from(["biomeos", "model-cache", "list"]);
    match &cli.mode {
        Mode::ModelCache { command } => match command {
            ModelCacheCommand::List => {}
            _ => panic!("expected List subcommand"),
        },
        _ => panic!("expected ModelCache mode"),
    }
}

#[test]
fn test_cli_parse_genome_list_default_dir() {
    let cli = Cli::parse_from(["biomeos", "genome", "list"]);
    match &cli.mode {
        Mode::Genome { command } => match command {
            GenomeCommand::List(args) => {
                assert_eq!(args.directory, PathBuf::from("plasmidBin"));
            }
            _ => panic!("expected List subcommand"),
        },
        _ => panic!("expected Genome mode"),
    }
}

#[test]
fn test_cli_parse_genome_build() {
    let cli = Cli::parse_from([
        "biomeos",
        "genome",
        "build",
        "--output",
        "/tmp/out.genome",
        "--name",
        "test",
    ]);
    match &cli.mode {
        Mode::Genome { command } => match command {
            GenomeCommand::Build(args) => {
                assert_eq!(args.output, PathBuf::from("/tmp/out.genome"));
                assert_eq!(args.name.as_deref(), Some("test"));
            }
            _ => panic!("expected Build subcommand"),
        },
        _ => panic!("expected Genome mode"),
    }
}

#[test]
fn test_cli_parse_genome_verify() {
    let cli = Cli::parse_from(["biomeos", "genome", "verify", "/path/to/genome.genome"]);
    match &cli.mode {
        Mode::Genome { command } => match command {
            GenomeCommand::Verify(args) => {
                assert_eq!(args.path, PathBuf::from("/path/to/genome.genome"));
            }
            _ => panic!("expected Verify subcommand"),
        },
        _ => panic!("expected Genome mode"),
    }
}

#[test]
fn test_cli_parse_model_cache_resolve() {
    let cli = Cli::parse_from([
        "biomeos",
        "model-cache",
        "resolve",
        "TinyLlama/TinyLlama-1.1B",
    ]);
    match &cli.mode {
        Mode::ModelCache { command } => match command {
            ModelCacheCommand::Resolve { model_id } => {
                assert_eq!(model_id, "TinyLlama/TinyLlama-1.1B");
            }
            _ => panic!("expected Resolve subcommand"),
        },
        _ => panic!("expected ModelCache mode"),
    }
}

#[test]
fn test_cli_parse_model_cache_register() {
    let cli = Cli::parse_from([
        "biomeos",
        "model-cache",
        "register",
        "test/model",
        "/path/to/model",
    ]);
    match &cli.mode {
        Mode::ModelCache { command } => match command {
            ModelCacheCommand::Register { model_id, path } => {
                assert_eq!(model_id, "test/model");
                assert_eq!(path, &PathBuf::from("/path/to/model"));
            }
            _ => panic!("expected Register subcommand"),
        },
        _ => panic!("expected ModelCache mode"),
    }
}

#[test]
fn test_cli_parse_model_cache_status() {
    let cli = Cli::parse_from(["biomeos", "model-cache", "status"]);
    match &cli.mode {
        Mode::ModelCache { command } => match command {
            ModelCacheCommand::Status => {}
            _ => panic!("expected Status subcommand"),
        },
        _ => panic!("expected ModelCache mode"),
    }
}

#[test]
fn test_cli_parse_model_cache_import_hf() {
    let cli = Cli::parse_from(["biomeos", "model-cache", "import-hf"]);
    match &cli.mode {
        Mode::ModelCache { command } => match command {
            ModelCacheCommand::ImportHf => {}
            _ => panic!("expected ImportHf subcommand"),
        },
        _ => panic!("expected ModelCache mode"),
    }
}

#[test]
fn test_cli_parse_plasmodium_status() {
    let cli = Cli::parse_from(["biomeos", "plasmodium", "status"]);
    match &cli.mode {
        Mode::Plasmodium { command } => match command {
            PlasmodiumCommand::Status => {}
            _ => panic!("expected Status subcommand"),
        },
        _ => panic!("expected Plasmodium mode"),
    }
}

#[test]
fn test_cli_parse_plasmodium_gates() {
    let cli = Cli::parse_from(["biomeos", "plasmodium", "gates"]);
    match &cli.mode {
        Mode::Plasmodium { command } => match command {
            PlasmodiumCommand::Gates => {}
            _ => panic!("expected Gates subcommand"),
        },
        _ => panic!("expected Plasmodium mode"),
    }
}

#[test]
fn test_cli_parse_plasmodium_models() {
    let cli = Cli::parse_from(["biomeos", "plasmodium", "models"]);
    match &cli.mode {
        Mode::Plasmodium { command } => match command {
            PlasmodiumCommand::Models => {}
            _ => panic!("expected Models subcommand"),
        },
        _ => panic!("expected Plasmodium mode"),
    }
}

#[test]
fn test_cli_parse_rootpulse_commit() {
    let cli = Cli::parse_from([
        "biomeos",
        "rootpulse",
        "commit",
        "--session-id",
        "sess-1",
        "--agent-did",
        "did:key:z6Mk",
    ]);
    match &cli.mode {
        Mode::RootPulse { command } => match command {
            RootPulseCommand::Commit {
                session_id,
                agent_did,
                dry_run,
                ..
            } => {
                assert_eq!(session_id, "sess-1");
                assert_eq!(agent_did, "did:key:z6Mk");
                assert!(!*dry_run);
            }
            _ => panic!("expected Commit subcommand"),
        },
        _ => panic!("expected RootPulse mode"),
    }
}

#[test]
fn test_cli_parse_rootpulse_branch() {
    let cli = Cli::parse_from([
        "biomeos",
        "rootpulse",
        "branch",
        "--session-id",
        "s1",
        "--branch-name",
        "feature",
        "--agent-did",
        "did:key:z6Mk",
    ]);
    match &cli.mode {
        Mode::RootPulse { command } => match command {
            RootPulseCommand::Branch {
                session_id,
                branch_name,
                agent_did,
                ..
            } => {
                assert_eq!(session_id, "s1");
                assert_eq!(branch_name, "feature");
                assert_eq!(agent_did, "did:key:z6Mk");
            }
            _ => panic!("expected Branch subcommand"),
        },
        _ => panic!("expected RootPulse mode"),
    }
}

#[test]
fn test_cli_parse_rootpulse_status() {
    let cli = Cli::parse_from(["biomeos", "rootpulse", "status"]);
    match &cli.mode {
        Mode::RootPulse { command } => match command {
            RootPulseCommand::Status { .. } => {}
            _ => panic!("expected Status subcommand"),
        },
        _ => panic!("expected RootPulse mode"),
    }
}

#[test]
fn test_cli_parse_graph_execute() {
    let cli = Cli::parse_from([
        "biomeos",
        "graph",
        "execute",
        "rootpulse_commit",
        "--param",
        "SESSION_ID=abc123",
        "--param",
        "AGENT_DID=did:key:z6Mk",
        "--dry-run",
    ]);
    match cli.mode {
        Mode::Graph {
            command:
                GraphCommand::Execute {
                    graph,
                    params,
                    dry_run,
                    ..
                },
        } => {
            assert_eq!(graph, "rootpulse_commit");
            assert_eq!(params.len(), 2);
            assert!(dry_run);
        }
        _ => panic!("Expected Graph Execute"),
    }
}

#[test]
fn test_cli_parse_graph_execute_toml_path() {
    let cli = Cli::parse_from([
        "biomeos",
        "graph",
        "execute",
        "graphs/rootpulse_commit.toml",
    ]);
    match cli.mode {
        Mode::Graph {
            command:
                GraphCommand::Execute {
                    graph,
                    params,
                    dry_run,
                    ..
                },
        } => {
            assert_eq!(graph, "graphs/rootpulse_commit.toml");
            assert!(params.is_empty());
            assert!(!dry_run);
        }
        _ => panic!("Expected Graph Execute"),
    }
}
