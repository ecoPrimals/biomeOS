// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]

use super::super::*;
use std::path::PathBuf;
use std::time::Duration;

#[test]
fn test_nucleus_mode_from_str_valid() {
    assert!(matches!(
        "tower".parse::<NucleusMode>().expect("tower should parse"),
        NucleusMode::Tower
    ));
    assert!(matches!(
        "Tower"
            .parse::<NucleusMode>()
            .expect("Tower should parse (case insensitive)"),
        NucleusMode::Tower
    ));
    assert!(matches!(
        "node".parse::<NucleusMode>().expect("node should parse"),
        NucleusMode::Node
    ));
    assert!(matches!(
        "nest".parse::<NucleusMode>().expect("nest should parse"),
        NucleusMode::Nest
    ));
    assert!(matches!(
        "full".parse::<NucleusMode>().expect("full should parse"),
        NucleusMode::Full
    ));
    assert!(matches!(
        "nucleus"
            .parse::<NucleusMode>()
            .expect("nucleus should parse"),
        NucleusMode::Full
    ));
    assert!(matches!(
        "core".parse::<NucleusMode>().expect("core should parse"),
        NucleusMode::Core
    ));
}

#[test]
fn test_nucleus_mode_from_str_invalid() {
    let err = "invalid".parse::<NucleusMode>().unwrap_err();
    assert!(err.to_string().contains("Unknown nucleus mode"));
    assert!(err.to_string().contains("invalid"));
    assert!(err.to_string().contains("tower|node|nest|core|full"));

    let err2 = "".parse::<NucleusMode>().unwrap_err();
    assert!(err2.to_string().contains("Unknown nucleus mode"));
}

#[test]
fn test_nucleus_mode_primals() {
    assert_eq!(
        NucleusMode::Tower.primals(),
        vec!["beardog", "songbird", "skunkbat"],
        "Tower mode primals"
    );
    assert_eq!(
        NucleusMode::Node.primals(),
        vec![
            "beardog",
            "songbird",
            "skunkbat",
            "toadstool",
            "coralreef",
            "barracuda"
        ],
        "Node mode primals"
    );
    assert_eq!(
        NucleusMode::Nest.primals(),
        vec![
            "beardog",
            "songbird",
            "skunkbat",
            "nestgate",
            "rhizocrypt",
            "loamspine",
            "sweetgrass",
            "squirrel"
        ],
        "Nest mode primals"
    );
    assert_eq!(
        NucleusMode::Core.primals(),
        vec!["beardog", "songbird", "nestgate", "toadstool", "squirrel"],
        "Core mode primals (legacy 5-primal compat)"
    );
    assert_eq!(
        NucleusMode::Full.primals().len(),
        12,
        "Full mode should launch 12 primals (all ecosystem primals except biomeOS itself)"
    );
    let full = NucleusMode::Full.primals();
    assert_eq!(full[0], "beardog", "beardog starts first (security root)");
    assert!(full.contains(&"petaltongue"), "Full includes petalTongue");
    assert!(full.contains(&"sweetgrass"), "Full includes sweetGrass");
}

#[test]
fn test_generate_jwt_secret_is_valid_base64() {
    let secret = generate_jwt_secret();
    assert!(!secret.is_empty());
    for c in secret.chars() {
        assert!(
            c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '=',
            "Invalid base64 char: {c:?}"
        );
    }
}

#[test]
fn test_generate_jwt_secret_is_unique() {
    let s1 = generate_jwt_secret();
    let s2 = generate_jwt_secret();
    assert_ne!(s1, s2, "Two consecutive JWT secrets should differ");
}

#[test]
fn test_generate_jwt_secret_length() {
    let secret = generate_jwt_secret();
    assert_eq!(secret.len(), 64, "48 bytes -> 64 base64 chars");
}

#[test]
fn test_socket_path_for_capability_uses_taxonomy() {
    let socket_dir = std::path::Path::new("/tmp/sock");
    let family_id = "fam1";
    let path = socket_path_for_capability(socket_dir, family_id, "security");
    assert!(path.to_string_lossy().contains("beardog"));
    assert!(path.to_string_lossy().contains("fam1"));
    assert!(path.to_string_lossy().ends_with(".sock"));
    let path2 = socket_path_for_capability(socket_dir, family_id, "discovery");
    assert!(path2.to_string_lossy().contains("songbird"));
}

#[test]
fn test_socket_path_for_capability_taxonomy_resolution() {
    let socket_dir = std::path::Path::new("/tmp/sock");
    let family_id = "fam1";
    // Taxonomy maps "encryption" → beardog
    let enc = socket_path_for_capability(socket_dir, family_id, "encryption");
    assert!(enc.to_string_lossy().contains("beardog"));
    // Taxonomy maps "discovery" → songbird
    let disc = socket_path_for_capability(socket_dir, family_id, "discovery");
    assert!(disc.to_string_lossy().contains("songbird"));
    // "registry" is an alias for Discovery → songbird
    let reg = socket_path_for_capability(socket_dir, family_id, "registry");
    assert!(reg.to_string_lossy().contains("songbird"));
    // Unknown capabilities → "unknown" (no hardcoded fallback)
    let unknown = socket_path_for_capability(socket_dir, family_id, "unknown-cap");
    assert!(unknown.to_string_lossy().contains("unknown"));
}

#[test]
fn test_build_primal_command_beardog() {
    let cmd = build_primal_command(
        "beardog",
        std::path::Path::new("/usr/bin/beardog"),
        std::path::Path::new("/tmp/sockets"),
        "fam123",
        "node1",
    );
    assert_eq!(cmd.get_program(), std::path::Path::new("/usr/bin/beardog"));
    let args: Vec<_> = cmd.get_args().collect();
    assert!(args.iter().any(|a| a.to_str() == Some("server")));
    assert!(args.iter().any(|a| a.to_str() == Some("--socket")));
}

#[test]
fn test_build_primal_command_songbird() {
    let cmd = build_primal_command(
        "songbird",
        std::path::Path::new("/usr/bin/songbird"),
        std::path::Path::new("/tmp/sock"),
        "fam1",
        "node1",
    );
    let envs: Vec<_> = cmd.get_envs().collect();
    assert!(
        envs.iter()
            .any(|(k, _)| k == &std::ffi::OsStr::new("SONGBIRD_SECURITY_PROVIDER")),
        "songbird should have SONGBIRD_SECURITY_PROVIDER from capability_sockets"
    );
    assert!(
        envs.iter()
            .any(|(k, _)| k == &std::ffi::OsStr::new("FAMILY_ID"))
    );
    // $node_id substitution → SONGBIRD_NODE_ID = "node1"
    assert!(
        envs.iter()
            .any(|(k, v)| k == &std::ffi::OsStr::new("SONGBIRD_NODE_ID")
                && v == &Some(std::ffi::OsStr::new("node1"))),
        "songbird should have SONGBIRD_NODE_ID from $node_id substitution"
    );
    // Static bind address for cross-gate federation
    assert!(
        envs.iter().any(
            |(k, v)| k == &std::ffi::OsStr::new("SONGBIRD_PRODUCTION_BIND_ADDRESS")
                && v == &Some(std::ffi::OsStr::new("0.0.0.0"))
        ),
        "songbird should have SONGBIRD_PRODUCTION_BIND_ADDRESS=0.0.0.0 for cross-gate federation"
    );
    let args: Vec<_> = cmd.get_args().collect();
    assert!(
        !args.iter().any(|a| a.to_str() == Some("--family-id")),
        "songbird should NOT receive --family-id CLI flag"
    );
}

#[test]
fn test_build_primal_command_nestgate() {
    let cmd = build_primal_command(
        "nestgate",
        std::path::Path::new("/usr/bin/nestgate"),
        std::path::Path::new("/tmp/sock"),
        "fam1",
        "node1",
    );
    let args: Vec<_> = cmd.get_args().collect();
    assert!(args.iter().any(|a| a.to_str() == Some("daemon")));
    assert!(args.iter().any(|a| a.to_str() == Some("--family-id")));
}

#[test]
fn test_build_primal_command_toadstool() {
    let cmd = build_primal_command(
        "toadstool",
        std::path::Path::new("/usr/bin/toadstool"),
        std::path::Path::new("/tmp/sock"),
        "fam1",
        "node1",
    );
    let envs: Vec<_> = cmd.get_envs().collect();
    assert!(
        envs.iter()
            .any(|(k, _)| k == &std::ffi::OsStr::new("TOADSTOOL_FAMILY_ID"))
    );
}

#[test]
fn test_build_primal_command_squirrel() {
    let cmd = build_primal_command(
        "squirrel",
        std::path::Path::new("/usr/bin/squirrel"),
        std::path::Path::new("/tmp/sock"),
        "fam1",
        "node1",
    );
    let envs: Vec<_> = cmd.get_envs().collect();
    assert!(
        envs.iter()
            .any(|(k, _)| k == &std::ffi::OsStr::new("BIOMEOS_DISCOVERY_SOCKET"))
    );
}

#[test]
fn test_build_primal_command_unknown_primal() {
    let cmd = build_primal_command(
        "unknown_primal",
        std::path::Path::new("/usr/bin/unknown"),
        std::path::Path::new("/tmp/sock"),
        "fam1",
        "node1",
    );
    let args: Vec<_> = cmd.get_args().collect();
    assert!(args.iter().any(|a| a.to_str() == Some("server")));
}

#[test]
fn test_format_nucleus_summary() {
    let children = vec![
        ("beardog".to_string(), 1234),
        ("songbird".to_string(), 1235),
    ];
    let lines = format_nucleus_summary(
        &children,
        std::path::Path::new("/tmp/sock"),
        "fam1",
        "node1",
        NucleusMode::Tower,
        "bootstrap",
    );
    assert!(!lines.is_empty());
    assert!(lines.iter().any(|l| l.contains("NUCLEUS started")));
    assert!(lines.iter().any(|l| l.contains("Family:")));
    assert!(lines.iter().any(|l| l.contains("Node:")));
    assert!(lines.iter().any(|l| l.contains("beardog")));
    assert!(lines.iter().any(|l| l.contains("1234")));
}

#[test]
fn test_format_nucleus_summary_empty_children() {
    let lines = format_nucleus_summary(
        &[],
        std::path::Path::new("/tmp/sock"),
        "fam1",
        "node1",
        NucleusMode::Node,
        "coordinated",
    );
    assert!(lines.iter().any(|l| l.contains("coordinated")));
    assert!(lines.iter().any(|l| l.contains("Health check")));
}

#[test]
fn test_ecosystem_state_debug() {
    let _ = format!("{:?}", EcosystemState::Bootstrap);
    let _ = format!(
        "{:?}",
        EcosystemState::Coordinated {
            active_primals: vec!["beardog".to_string()],
        }
    );
}

#[test]
fn test_startup_config_debug() {
    let config = StartupConfig {
        mode: NucleusMode::Tower,
        node_id: "n1".to_string(),
        family_id: "f1".to_string(),
        socket_dir: PathBuf::from("/tmp"),
    };
    let _ = format!("{config:?}");
}

#[test]
fn test_resolve_startup_config_invalid_mode() {
    let result = resolve_startup_config("invalid", "node1", None);
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Unknown nucleus mode")
    );
}

#[test]
fn test_resolve_startup_config_valid() {
    let config =
        resolve_startup_config_with("tower", "node1", Some("fam1"), Some("/tmp/test-nucleus"))
            .unwrap();
    assert!(matches!(config.mode, NucleusMode::Tower));
    assert_eq!(config.node_id, "node1");
    assert_eq!(config.family_id, "fam1");
    assert_eq!(
        config.socket_dir,
        std::path::PathBuf::from("/tmp/test-nucleus")
    );
}

#[test]
fn test_resolve_socket_dir_env_override() {
    let result =
        resolve_socket_dir_with(Some("/tmp/biomeos-test-socket-dir")).expect("should succeed");
    assert_eq!(
        result,
        std::path::PathBuf::from("/tmp/biomeos-test-socket-dir")
    );
}
