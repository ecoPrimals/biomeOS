// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

#![allow(clippy::unwrap_used, clippy::expect_used)]

use super::*;
use clap::{CommandFactory, Parser};
use std::path::PathBuf;

#[test]
fn test_parse_ai_intent_health() {
    assert_eq!(parse_ai_intent("health status"), AiIntent::Health);
    assert_eq!(parse_ai_intent("check health"), AiIntent::Health);
}

#[test]
fn test_parse_ai_intent_discover() {
    assert_eq!(parse_ai_intent("discover primals"), AiIntent::Discover);
}

#[test]
fn test_parse_ai_intent_deploy() {
    assert_eq!(parse_ai_intent("deploy help"), AiIntent::Deploy);
}

#[test]
fn test_parse_ai_intent_status() {
    assert_eq!(parse_ai_intent("status"), AiIntent::Status);
}

#[test]
fn test_parse_ai_intent_unknown() {
    assert_eq!(parse_ai_intent("random query"), AiIntent::Unknown);
}

#[test]
fn test_parse_ai_intent_case_insensitive() {
    assert_eq!(parse_ai_intent("HEALTH check"), AiIntent::Health);
    assert_eq!(parse_ai_intent("DISCOVER services"), AiIntent::Discover);
    assert_eq!(parse_ai_intent("Deploy manifest"), AiIntent::Deploy);
}

#[test]
fn test_parse_ai_intent_priority_first_match() {
    assert_eq!(parse_ai_intent("health and discover"), AiIntent::Health);
}

#[test]
fn test_format_ai_response_health() {
    let lines = format_ai_response(&AiIntent::Health);
    assert!(!lines.is_empty());
    assert!(lines[0].contains("Health"));
    assert!(
        lines
            .iter()
            .any(|l| l.contains("Universal BiomeOS Manager"))
    );
}

#[test]
fn test_format_ai_response_discover() {
    let lines = format_ai_response(&AiIntent::Discover);
    assert!(!lines.is_empty());
    assert!(lines[0].contains("Primal Discovery"));
    assert!(lines.iter().any(|l| l.contains("capability-based")));
}

#[test]
fn test_format_ai_response_deploy() {
    let lines = format_ai_response(&AiIntent::Deploy);
    assert!(!lines.is_empty());
    assert!(lines[0].contains("Deployment"));
    assert!(lines.iter().any(|l| l.contains("biome.yaml")));
}

#[test]
fn test_format_ai_response_status() {
    let lines = format_ai_response(&AiIntent::Status);
    assert!(lines.iter().any(|l| l.contains("Suggestions")));
}

#[test]
fn test_format_ai_response_unknown() {
    let lines = format_ai_response(&AiIntent::Unknown);
    assert!(lines.iter().any(|l| l.contains("Suggestions")));
}

#[test]
fn test_cli_parse_verify_nucleus() {
    let cli = Cli::try_parse_from(["biomeos", "verify", "nucleus"]).expect("parse ok");
    match &cli.command {
        Commands::Verify(args) => match &args.target {
            verify::VerifyTarget::Nucleus { path } => {
                assert_eq!(path, &PathBuf::from("plasmidBin"));
            }
            _ => panic!("expected Nucleus target"),
        },
        _ => panic!("expected Verify command"),
    }
}

#[test]
fn test_cli_parse_verify_nucleus_custom_path() {
    let cli = Cli::try_parse_from([
        "biomeos",
        "verify",
        "nucleus",
        "--path",
        "/custom/plasmidBin",
    ])
    .expect("parse ok");
    match &cli.command {
        Commands::Verify(args) => match &args.target {
            verify::VerifyTarget::Nucleus { path } => {
                assert_eq!(path, &PathBuf::from("/custom/plasmidBin"));
            }
            _ => panic!("expected Nucleus target"),
        },
        _ => panic!("expected Verify command"),
    }
}

#[test]
fn test_cli_parse_verify_spore() {
    let cli = Cli::try_parse_from(["biomeos", "verify", "spore", "/media/usb/biomeOS"])
        .expect("parse ok");
    match &cli.command {
        Commands::Verify(args) => match &args.target {
            verify::VerifyTarget::Spore { mount_point } => {
                assert_eq!(mount_point, &PathBuf::from("/media/usb/biomeOS"));
            }
            _ => panic!("expected Spore target"),
        },
        _ => panic!("expected Verify command"),
    }
}

#[test]
fn test_cli_parse_verify_all() {
    let cli = Cli::try_parse_from(["biomeos", "verify", "all"]).expect("parse ok");
    match &cli.command {
        Commands::Verify(args) => match &args.target {
            verify::VerifyTarget::All { verbose } => assert!(!*verbose),
            _ => panic!("expected All target"),
        },
        _ => panic!("expected Verify command"),
    }
}

#[test]
fn test_cli_parse_verify_all_verbose() {
    let cli = Cli::try_parse_from(["biomeos", "verify", "all", "--verbose"]).expect("parse ok");
    match &cli.command {
        Commands::Verify(args) => match &args.target {
            verify::VerifyTarget::All { verbose } => assert!(*verbose),
            _ => panic!("expected All target"),
        },
        _ => panic!("expected Verify command"),
    }
}

#[test]
fn test_cli_parse_discover_defaults() {
    let cli = Cli::try_parse_from(["biomeos", "discover"]).expect("parse ok");
    match &cli.command {
        Commands::Discover {
            endpoint,
            capabilities,
            method,
            registry,
            detailed,
        } => {
            assert!(endpoint.is_none());
            assert!(capabilities.is_none());
            assert!(matches!(
                method,
                biomeos_cli::commands::discover::DiscoveryMethod::CapabilityBased
            ));
            assert!(registry.is_none());
            assert!(!*detailed);
        }
        _ => panic!("expected Discover command"),
    }
}

#[test]
fn test_cli_parse_discover_with_options() {
    let cli = Cli::try_parse_from([
        "biomeos",
        "discover",
        "-e",
        "http://localhost:8080",
        "-c",
        "storage/file",
        "--method",
        "registry-based",
        "--registry",
        "http://registry:5000",
        "--detailed",
    ])
    .expect("parse ok");
    match &cli.command {
        Commands::Discover {
            endpoint,
            capabilities,
            method,
            registry,
            detailed,
        } => {
            assert_eq!(endpoint.as_deref(), Some("http://localhost:8080"));
            assert_eq!(capabilities.as_deref(), Some("storage/file"));
            assert!(matches!(
                method,
                biomeos_cli::commands::discover::DiscoveryMethod::RegistryBased
            ));
            assert_eq!(registry.as_deref(), Some("http://registry:5000"));
            assert!(*detailed);
        }
        _ => panic!("expected Discover command"),
    }
}

#[test]
fn test_cli_parse_genome_create() {
    let cli = Cli::try_parse_from([
        "biomeos",
        "genome",
        "create",
        "--binary",
        "/tmp/beardog",
        "--output",
        "/tmp/out.json",
    ])
    .expect("parse ok");
    match &cli.command {
        Commands::Genome { action } => match action {
            GenomeAction::Create(args) => {
                assert_eq!(args.binary, PathBuf::from("/tmp/beardog"));
                assert_eq!(args.output, PathBuf::from("/tmp/out.json"));
                assert_eq!(args.arch, "x86_64");
            }
            _ => panic!("expected Create action"),
        },
        _ => panic!("expected Genome command"),
    }
}

#[test]
fn test_cli_parse_chimera_list() {
    let cli = Cli::try_parse_from(["biomeos", "chimera", "list"]).expect("parse ok");
    match &cli.command {
        Commands::Chimera { action } => match action {
            ChimeraAction::List => {}
            _ => panic!("expected List action"),
        },
        _ => panic!("expected Chimera command"),
    }
}

#[test]
fn test_cli_parse_chimera_show() {
    let cli = Cli::try_parse_from(["biomeos", "chimera", "show", "my-chimera"]).expect("parse ok");
    match &cli.command {
        Commands::Chimera { action } => match action {
            ChimeraAction::Show { id } => assert_eq!(id, "my-chimera"),
            _ => panic!("expected Show action"),
        },
        _ => panic!("expected Chimera command"),
    }
}

#[test]
fn test_cli_parse_ai_command() {
    let cli =
        Cli::try_parse_from(["biomeos", "ai", "what is the health status?"]).expect("parse ok");
    match &cli.command {
        Commands::Ai { query, context } => {
            assert_eq!(query, "what is the health status?");
            assert!(context.is_none());
        }
        _ => panic!("expected Ai command"),
    }
}

#[test]
fn test_cli_parse_log_level_and_output() {
    let cli = Cli::try_parse_from([
        "biomeos",
        "--log-level",
        "debug",
        "--output",
        "json",
        "verify",
        "nucleus",
    ])
    .expect("parse ok");
    assert_eq!(cli.log_level, "debug");
    assert!(matches!(cli.output, OutputFormat::Json));
}

#[test]
fn test_cli_help_contains_biomeos() {
    let help = Cli::command().render_help().to_string();
    assert!(help.contains("biomeOS"));
    assert!(help.contains("verify"));
    assert!(help.contains("discover"));
}

#[test]
fn test_cli_parse_fails_unknown_command() {
    let result = Cli::try_parse_from(["biomeos", "unknown-xyz"]);
    assert!(result.is_err());
}
