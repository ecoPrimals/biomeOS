// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

#![expect(clippy::unwrap_used, reason = "test assertions")]
#![expect(clippy::expect_used, reason = "test assertions")]

use super::*;
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
fn test_cli_parse_neural_api() {
    let cli = Cli::parse_from(["biomeos", "neural-api"]);
    match &cli.mode {
        Mode::NeuralApi {
            graphs_dir,
            family_id,
            socket,
            port,
            tcp_only,
        } => {
            assert_eq!(graphs_dir, &PathBuf::from("graphs"));
            assert!(family_id.is_none());
            assert!(socket.is_none());
            assert!(port.is_none());
            assert!(!tcp_only);
        }
        _ => panic!("expected NeuralApi mode"),
    }
}

#[test]
fn test_cli_parse_neural_api_with_opts() {
    let cli = Cli::parse_from([
        "biomeos",
        "neural-api",
        "--graphs-dir",
        "/tmp/graphs",
        "--family-id",
        "fam1",
        "--socket",
        "/tmp/api.sock",
    ]);
    match &cli.mode {
        Mode::NeuralApi {
            graphs_dir,
            family_id,
            socket,
            port,
            tcp_only,
        } => {
            assert_eq!(graphs_dir, &PathBuf::from("/tmp/graphs"));
            assert_eq!(family_id.as_deref(), Some("fam1"));
            assert_eq!(
                socket.as_ref().map(PathBuf::as_path),
                Some(std::path::Path::new("/tmp/api.sock"))
            );
            assert!(port.is_none());
            assert!(!tcp_only);
        }
        _ => panic!("expected NeuralApi mode"),
    }
}

#[test]
fn test_cli_parse_neural_api_tcp_only() {
    let cli = Cli::parse_from(["biomeos", "neural-api", "--port", "9000", "--tcp-only"]);
    match &cli.mode {
        Mode::NeuralApi { port, tcp_only, .. } => {
            assert_eq!(*port, Some(9000));
            assert!(tcp_only);
        }
        _ => panic!("expected NeuralApi mode"),
    }
}

#[test]
fn test_cli_parse_neural_api_tcp_port_no_tcp_only() {
    let cli = Cli::parse_from(["biomeos", "neural-api", "--port", "8080"]);
    match &cli.mode {
        Mode::NeuralApi { port, tcp_only, .. } => {
            assert_eq!(*port, Some(8080));
            assert!(!tcp_only);
        }
        _ => panic!("expected NeuralApi mode"),
    }
}

#[test]
fn test_cli_parse_deploy() {
    let cli = Cli::parse_from(["biomeos", "deploy", "graph.json"]);
    match &cli.mode {
        Mode::Deploy {
            graph,
            validate_only,
            dry_run,
        } => {
            assert_eq!(graph, &PathBuf::from("graph.json"));
            assert!(!*validate_only);
            assert!(!*dry_run);
        }
        _ => panic!("expected Deploy mode"),
    }
}

#[test]
fn test_cli_parse_deploy_validate_dry_run() {
    let cli = Cli::parse_from([
        "biomeos",
        "deploy",
        "g.json",
        "--validate-only",
        "--dry-run",
    ]);
    match &cli.mode {
        Mode::Deploy {
            validate_only,
            dry_run,
            ..
        } => {
            assert!(*validate_only);
            assert!(*dry_run);
        }
        _ => panic!("expected Deploy mode"),
    }
}

#[test]
fn test_cli_parse_api() {
    let cli = Cli::parse_from(["biomeos", "api"]);
    match &cli.mode {
        Mode::Api {
            port,
            socket,
            unix_only,
        } => {
            assert!(port.is_none());
            assert!(socket.is_none());
            assert!(!*unix_only);
        }
        _ => panic!("expected Api mode"),
    }
}

#[test]
fn test_cli_parse_api_with_port_and_socket() {
    let cli = Cli::parse_from(["biomeos", "api", "-p", "8080", "--socket", "/tmp/api.sock"]);
    match &cli.mode {
        Mode::Api {
            port,
            socket,
            unix_only,
        } => {
            assert_eq!(*port, Some(8080));
            assert_eq!(
                socket.as_ref().map(PathBuf::as_path),
                Some(std::path::Path::new("/tmp/api.sock"))
            );
            assert!(!*unix_only);
        }
        _ => panic!("expected Api mode"),
    }
}

#[test]
fn test_cli_parse_verify_lineage() {
    let cli = Cli::parse_from(["biomeos", "verify-lineage", "/path/to/spore"]);
    match &cli.mode {
        Mode::VerifyLineage { path, detailed } => {
            assert_eq!(path, &PathBuf::from("/path/to/spore"));
            assert!(!*detailed);
        }
        _ => panic!("expected VerifyLineage mode"),
    }
}

#[test]
fn test_cli_parse_verify_lineage_detailed() {
    let cli = Cli::parse_from(["biomeos", "verify-lineage", "/p", "--detailed"]);
    match &cli.mode {
        Mode::VerifyLineage { detailed, .. } => assert!(*detailed),
        _ => panic!("expected VerifyLineage mode"),
    }
}

#[test]
fn test_cli_parse_nucleus() {
    let cli = Cli::parse_from(["biomeos", "nucleus", "--node-id", "node1"]);
    match &cli.mode {
        Mode::Nucleus {
            mode,
            node_id,
            family_id,
        } => {
            assert_eq!(mode, "full");
            assert_eq!(node_id, "node1");
            assert!(family_id.is_none());
        }
        _ => panic!("expected Nucleus mode"),
    }
}

#[test]
fn test_cli_parse_nucleus_with_mode_and_family() {
    let cli = Cli::parse_from([
        "biomeos",
        "nucleus",
        "--mode",
        "tower",
        "--node-id",
        "n1",
        "--family-id",
        "fam1",
    ]);
    match &cli.mode {
        Mode::Nucleus {
            mode,
            node_id,
            family_id,
        } => {
            assert_eq!(mode, "tower");
            assert_eq!(node_id, "n1");
            assert_eq!(family_id.as_deref(), Some("fam1"));
        }
        _ => panic!("expected Nucleus mode"),
    }
}

#[test]
fn test_cli_parse_continuous() {
    let cli = Cli::parse_from(["biomeos", "continuous", "graph.json"]);
    match &cli.mode {
        Mode::Continuous { graph, dry_run } => {
            assert_eq!(graph, &PathBuf::from("graph.json"));
            assert!(!*dry_run);
        }
        _ => panic!("expected Continuous mode"),
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
fn test_cli_parse_verbose_and_log_level() {
    let cli = Cli::parse_from(["biomeos", "--verbose", "--log-level", "debug", "version"]);
    assert!(cli.verbose);
    assert_eq!(cli.log_level, "debug");
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
fn test_cli_parse_cli_mode() {
    let cli = Cli::parse_from(["biomeos", "cli"]);
    match &cli.mode {
        Mode::Cli {} => {}
        _ => panic!("expected Cli mode"),
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
fn test_cli_parse_continuous_dry_run() {
    let cli = Cli::parse_from(["biomeos", "continuous", "graph.json", "--dry-run"]);
    match &cli.mode {
        Mode::Continuous { graph, dry_run } => {
            assert_eq!(graph, &PathBuf::from("graph.json"));
            assert!(*dry_run);
        }
        _ => panic!("expected Continuous mode"),
    }
}

#[test]
fn test_cli_parse_api_unix_only() {
    let cli = Cli::parse_from(["biomeos", "api", "--unix-only"]);
    match &cli.mode {
        Mode::Api { unix_only, .. } => assert!(*unix_only),
        _ => panic!("expected Api mode"),
    }
}

#[test]
fn test_init_logging_verbose_overrides_level() {
    let result = init_logging("warn", true);
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_dispatch_mode_version() {
    let cli = Cli::parse_from(["biomeos", "version"]);
    let result = dispatch_mode(cli).await;
    result.expect("dispatch version should succeed");
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

#[test]
fn test_cli_parse_fails_nucleus_missing_node_id() {
    let result = Cli::try_parse_from(["biomeos", "nucleus"]);
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

#[tokio::test]
async fn test_dispatch_mode_doctor() {
    let cli = Cli::parse_from(["biomeos", "doctor"]);
    let result = dispatch_mode(cli).await;
    result.expect("dispatch doctor should succeed");
}

#[tokio::test]
async fn test_dispatch_mode_cli() {
    let cli = Cli::parse_from(["biomeos", "cli"]);
    let result = dispatch_mode(cli).await;
    result.expect("dispatch cli should succeed");
}

#[tokio::test]
async fn test_dispatch_mode_model_cache_list() {
    let cli = Cli::parse_from(["biomeos", "model-cache", "list"]);
    let result = dispatch_mode(cli).await;
    result.expect("dispatch model-cache list should succeed");
}

#[tokio::test]
async fn test_dispatch_mode_plasmodium_status() {
    let cli = Cli::parse_from(["biomeos", "plasmodium", "status"]);
    let result = dispatch_mode(cli).await;
    result.expect("dispatch plasmodium status should succeed");
}

#[tokio::test]
async fn test_dispatch_mode_deploy_fails_nonexistent_graph() {
    let cli = Cli::parse_from([
        "biomeos",
        "deploy",
        "/nonexistent/path/to/graph-xyz-12345.json",
    ]);
    let result = dispatch_mode(cli).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_dispatch_mode_verify_lineage_fails_nonexistent_path() {
    let cli = Cli::parse_from([
        "biomeos",
        "verify-lineage",
        "/nonexistent/path/to/spore-xyz-12345",
    ]);
    let result = dispatch_mode(cli).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_dispatch_mode_version_detailed() {
    let cli = Cli::parse_from(["biomeos", "version", "--detailed"]);
    let result = dispatch_mode(cli).await;
    result.expect("dispatch version --detailed should succeed");
}

#[tokio::test]
async fn test_dispatch_mode_doctor_json() {
    let cli = Cli::parse_from(["biomeos", "doctor", "-f", "json"]);
    let result = dispatch_mode(cli).await;
    result.expect("dispatch doctor json should succeed");
}

#[tokio::test]
async fn test_dispatch_mode_doctor_subsystem() {
    let cli = Cli::parse_from(["biomeos", "doctor", "--subsystem", "config"]);
    let result = dispatch_mode(cli).await;
    result.expect("dispatch doctor subsystem should succeed");
}

#[tokio::test]
async fn test_dispatch_mode_genome_list() {
    let _guard = CWD_TEST_LOCK.lock().await;
    let tmp = tempfile::tempdir().expect("tempdir");
    let plasmidbindir = tmp.path().join("plasmidBin");
    std::fs::create_dir_all(&plasmidbindir).expect("create plasmidBin dir");
    let cli = Cli::parse_from(["biomeos", "genome", "list", plasmidbindir.to_str().unwrap()]);
    let result = dispatch_mode(cli).await;
    result.expect("dispatch genome list empty dir should succeed");
}

#[tokio::test]
async fn test_dispatch_mode_deploy_validate_only_nonexistent() {
    let cli = Cli::parse_from([
        "biomeos",
        "deploy",
        "/nonexistent/graph-test-validate-12345.toml",
        "--validate-only",
    ]);
    let result = dispatch_mode(cli).await;
    assert!(result.is_err(), "validate-only on nonexistent should fail");
}

#[tokio::test]
async fn test_dispatch_mode_deploy_dry_run_nonexistent() {
    let cli = Cli::parse_from([
        "biomeos",
        "deploy",
        "/nonexistent/graph-test-dryrun-12345.toml",
        "--dry-run",
    ]);
    let result = dispatch_mode(cli).await;
    assert!(result.is_err(), "dry-run on nonexistent should fail");
}

#[tokio::test]
async fn test_dispatch_mode_model_cache_status() {
    let cli = Cli::parse_from(["biomeos", "model-cache", "status"]);
    let result = dispatch_mode(cli).await;
    result.expect("dispatch model-cache status should succeed");
}

#[tokio::test]
async fn test_dispatch_mode_model_cache_import_hf() {
    let cli = Cli::parse_from(["biomeos", "model-cache", "import-hf"]);
    let _result = dispatch_mode(cli).await;
}

#[tokio::test]
async fn test_dispatch_mode_model_cache_resolve() {
    let cli = Cli::parse_from([
        "biomeos",
        "model-cache",
        "resolve",
        "nonexistent/model-12345",
    ]);
    let _result = dispatch_mode(cli).await;
}

#[tokio::test]
async fn test_dispatch_mode_plasmodium_gates() {
    let cli = Cli::parse_from(["biomeos", "plasmodium", "gates"]);
    let result = dispatch_mode(cli).await;
    result.expect("dispatch plasmodium gates should succeed");
}

#[tokio::test]
async fn test_dispatch_mode_plasmodium_models() {
    let cli = Cli::parse_from(["biomeos", "plasmodium", "models"]);
    let result = dispatch_mode(cli).await;
    result.expect("dispatch plasmodium models should succeed");
}

#[tokio::test]
async fn test_dispatch_mode_verify_lineage_detailed_fails() {
    let cli = Cli::parse_from([
        "biomeos",
        "verify-lineage",
        "/nonexistent/detailed-test-12345",
        "--detailed",
    ]);
    let result = dispatch_mode(cli).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_dispatch_mode_continuous_nonexistent() {
    let cli = Cli::parse_from([
        "biomeos",
        "continuous",
        "/nonexistent/continuous-test-12345.toml",
    ]);
    let result = dispatch_mode(cli).await;
    assert!(
        result.is_err(),
        "continuous on nonexistent graph should fail"
    );
}

#[tokio::test]
async fn test_dispatch_mode_rootpulse_status() {
    let cli = Cli::parse_from(["biomeos", "rootpulse", "status"]);
    let _result = dispatch_mode(cli).await;
}

#[tokio::test]
async fn test_dispatch_mode_enroll_fails_without_seed() {
    let cli = Cli::parse_from([
        "biomeos",
        "enroll",
        "--family-id",
        "test-family-12345",
        "--node-id",
        "test-node-12345",
    ]);
    let result = dispatch_mode(cli).await;
    assert!(
        result.is_err(),
        "enroll without valid family seed file should fail"
    );
}
