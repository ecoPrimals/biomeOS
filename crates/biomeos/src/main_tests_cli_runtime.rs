// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

#![expect(clippy::unwrap_used, reason = "test assertions")]
#![expect(clippy::expect_used, reason = "test assertions")]

use super::super::*;

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
            bind,
            btsp_optional,
            bind_mode: _,
        } => {
            assert_eq!(graphs_dir, &PathBuf::from("graphs"));
            assert!(family_id.is_none());
            assert!(socket.is_none());
            assert!(port.is_none());
            assert!(!tcp_only);
            assert!(bind.is_none());
            assert!(!btsp_optional);
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
            bind,
            btsp_optional,
            bind_mode: _,
        } => {
            assert_eq!(graphs_dir, &PathBuf::from("/tmp/graphs"));
            assert_eq!(family_id.as_deref(), Some("fam1"));
            let _ = bind;
            assert_eq!(
                socket.as_ref().map(PathBuf::as_path),
                Some(std::path::Path::new("/tmp/api.sock"))
            );
            assert!(port.is_none());
            assert!(!tcp_only);
            assert!(!btsp_optional);
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
            ..
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
            bind,
            unix_only,
        } => {
            assert!(port.is_none());
            assert!(socket.is_none());
            assert!(bind.is_none());
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
            bind,
            unix_only,
        } => {
            let _ = bind;
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
fn test_cli_parse_api_unix_only() {
    let cli = Cli::parse_from(["biomeos", "api", "--unix-only"]);
    match &cli.mode {
        Mode::Api { unix_only, .. } => assert!(*unix_only),
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
