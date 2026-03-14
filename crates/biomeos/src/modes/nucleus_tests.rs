// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

use super::*;
use std::path::PathBuf;

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
}

#[test]
fn test_nucleus_mode_from_str_invalid() {
    let err = "invalid".parse::<NucleusMode>().unwrap_err();
    assert!(err.to_string().contains("Unknown nucleus mode"));
    assert!(err.to_string().contains("invalid"));
    assert!(err.to_string().contains("tower|node|nest|full"));

    let err2 = "".parse::<NucleusMode>().unwrap_err();
    assert!(err2.to_string().contains("Unknown nucleus mode"));
}

#[test]
fn test_nucleus_mode_primals() {
    assert_eq!(
        NucleusMode::Tower.primals(),
        vec!["beardog", "songbird"],
        "Tower mode primals"
    );
    assert_eq!(
        NucleusMode::Node.primals(),
        vec!["beardog", "songbird", "toadstool"],
        "Node mode primals"
    );
    assert_eq!(
        NucleusMode::Nest.primals(),
        vec!["beardog", "songbird", "nestgate", "squirrel"],
        "Nest mode primals"
    );
    assert_eq!(
        NucleusMode::Full.primals(),
        vec!["beardog", "songbird", "nestgate", "toadstool", "squirrel"],
        "Full mode primals"
    );
}

#[test]
fn test_base64_encode_empty() {
    assert_eq!(base64_encode(&[]), "");
}

#[test]
fn test_base64_encode_single_byte() {
    let result = base64_encode(&[0x4d]);
    assert_eq!(result.len(), 4);
    assert!(result.ends_with("=="));
}

#[test]
fn test_base64_encode_three_bytes() {
    let result = base64_encode(b"Man");
    assert_eq!(result, "TWFu");
}

#[test]
fn test_base64_encode_roundtrip_alphabet() {
    let data = b"Hello, World!";
    let encoded = base64_encode(data);
    assert!(!encoded.is_empty());
    assert!(encoded.len() <= data.len().div_ceil(3) * 4 + 4);
    for c in encoded.chars() {
        assert!(
            c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '=',
            "Invalid base64 char: {:?}",
            c
        );
    }
}

#[test]
fn test_socket_path_for_capability_uses_taxonomy() {
    let socket_dir = std::path::Path::new("/tmp/sock");
    let family_id = "fam1";
    let path = super::socket_path_for_capability(socket_dir, family_id, "security");
    assert!(path.to_string_lossy().contains("beardog"));
    assert!(path.to_string_lossy().contains("fam1"));
    assert!(path.to_string_lossy().ends_with(".sock"));
    let path2 = super::socket_path_for_capability(socket_dir, family_id, "discovery");
    assert!(path2.to_string_lossy().contains("songbird"));
}

#[test]
fn test_socket_path_for_capability_fallbacks() {
    let socket_dir = std::path::Path::new("/tmp/sock");
    let family_id = "fam1";
    let enc = super::socket_path_for_capability(socket_dir, family_id, "encryption");
    assert!(enc.to_string_lossy().contains("beardog"));
    let reg = super::socket_path_for_capability(socket_dir, family_id, "registry");
    assert!(reg.to_string_lossy().contains("songbird"));
    let unknown = super::socket_path_for_capability(socket_dir, family_id, "unknown-cap");
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
    assert!(envs
        .iter()
        .any(|(k, _)| k == &std::ffi::OsStr::new("SONGBIRD_SECURITY_PROVIDER")));
    assert!(envs
        .iter()
        .any(|(k, _)| k == &std::ffi::OsStr::new("FAMILY_ID")));
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
    assert!(envs
        .iter()
        .any(|(k, _)| k == &std::ffi::OsStr::new("TOADSTOOL_FAMILY_ID")));
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
    assert!(envs
        .iter()
        .any(|(k, _)| k == &std::ffi::OsStr::new("BIOMEOS_DISCOVERY_SOCKET")));
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
        &NucleusMode::Tower,
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
        &NucleusMode::Node,
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
    let _ = format!("{:?}", config);
}

#[test]
fn test_resolve_startup_config_invalid_mode() {
    let result = resolve_startup_config("invalid", "node1", None);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Unknown nucleus mode"));
}

#[test]
#[ignore = "env-var test is thread-unsafe; run with --test-threads=1"]
fn test_resolve_startup_config_valid() {
    let config = resolve_startup_config("tower", "node1", Some("fam1")).unwrap();
    assert!(matches!(config.mode, NucleusMode::Tower));
    assert_eq!(config.node_id, "node1");
    assert_eq!(config.family_id, "fam1");
    assert!(!config.socket_dir.as_os_str().is_empty());
}

#[test]
#[ignore = "env-var test is thread-unsafe; run with --test-threads=1"]
fn test_resolve_socket_dir_env_override() {
    let test_path = "/tmp/biomeos-test-socket-dir";
    std::env::set_var("BIOMEOS_SOCKET_DIR", test_path);
    let result = resolve_socket_dir().expect("resolve_socket_dir should succeed");
    std::env::remove_var("BIOMEOS_SOCKET_DIR");
    assert_eq!(result, std::path::PathBuf::from(test_path));
}

#[test]
fn test_discover_binaries_empty_primals() {
    let map = discover_binaries(&[]).expect("empty primals should succeed");
    assert!(map.is_empty());
}

#[test]
fn test_nucleus_mode_from_str_case_insensitive() {
    assert!(matches!(
        "NODE".parse::<NucleusMode>().expect("parse"),
        NucleusMode::Node
    ));
    assert!(matches!(
        "FULL".parse::<NucleusMode>().expect("parse"),
        NucleusMode::Full
    ));
}

#[test]
fn test_base64_encode_two_bytes() {
    let result = base64_encode(&[0x4d, 0x61]);
    assert_eq!(result.len(), 4);
    assert!(result.ends_with("="));
}

#[test]
fn test_base64_encode_four_bytes() {
    let result = base64_encode(&[0x4d, 0x61, 0x6e, 0x21]);
    assert_eq!(result, "TWFuIQ==");
}

#[test]
#[ignore = "env-var test is thread-unsafe; run with --test-threads=1"]
fn test_resolve_socket_dir_default() {
    std::env::remove_var("BIOMEOS_SOCKET_DIR");
    let result = resolve_socket_dir();
    assert!(result.is_ok());
    let path = result.unwrap();
    assert!(!path.as_os_str().is_empty());
}

#[tokio::test]
async fn test_run_fails_on_invalid_mode() {
    let result = run("invalid_mode_xyz".to_string(), "node1".to_string(), None).await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("Unknown nucleus mode"),
        "Expected parse error: {}",
        err
    );
}

#[test]
#[cfg(unix)]
#[ignore = "env-var test is thread-unsafe; run with --test-threads=1"]
fn test_discover_binaries_finds_in_path() {
    use std::os::unix::fs::PermissionsExt;

    let temp_dir = tempfile::tempdir().expect("temp dir");
    let unique_name = "biomeos_test_binary_xyz";
    let binary_path = temp_dir.path().join(unique_name);
    std::fs::write(&binary_path, "#!/bin/sh\nexit 0").expect("write test binary");
    let mut perms = std::fs::metadata(&binary_path).unwrap().permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(&binary_path, perms).unwrap();

    let original_path = std::env::var("PATH").ok();
    let dir_str = temp_dir.path().to_string_lossy().into_owned();
    std::env::set_var(
        "PATH",
        format!("{}:{}", dir_str, original_path.as_deref().unwrap_or("")),
    );

    let map = discover_binaries(&[unique_name]).expect("discover should succeed");
    if let Some(original) = original_path {
        std::env::set_var("PATH", original);
    } else {
        std::env::remove_var("PATH");
    }

    assert!(
        map.contains_key(unique_name),
        "{} should be found in PATH, got: {:?}",
        unique_name,
        map
    );
}

#[tokio::test]
async fn test_wait_for_socket_times_out_on_nonexistent() {
    let temp = tempfile::tempdir().expect("temp dir");
    let path = temp.path().join("nonexistent.sock");
    let result = wait_for_socket(&path, Duration::from_millis(50)).await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("did not appear"));
}

#[tokio::test]
async fn test_wait_for_socket_succeeds_when_file_exists() {
    let temp = tempfile::tempdir().expect("temp dir");
    let path = temp.path().join("test.sock");
    std::fs::write(&path, "").expect("create socket file");
    let result = wait_for_socket(&path, Duration::from_secs(1)).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_detect_ecosystem_bootstrap_when_dir_nonexistent() {
    let temp = tempfile::tempdir().expect("temp dir");
    let nonexistent = temp.path().join("nonexistent_subdir");
    let state = detect_ecosystem(&nonexistent, "test-family").await;
    assert!(
        matches!(state, EcosystemState::Bootstrap),
        "Expected Bootstrap when dir does not exist, got: {:?}",
        state
    );
}

#[tokio::test]
async fn test_detect_ecosystem_bootstrap_when_dir_empty() {
    let temp = tempfile::tempdir().expect("temp dir");
    let state = detect_ecosystem(temp.path(), "test-family").await;
    assert!(
        matches!(state, EcosystemState::Bootstrap),
        "Expected Bootstrap when dir is empty, got: {:?}",
        state
    );
}

#[tokio::test]
async fn test_detect_ecosystem_bootstrap_when_stale_sockets_only() {
    let temp = tempfile::tempdir().expect("temp dir");
    let socket_path = temp.path().join("beardog-test-family.sock");
    std::fs::write(&socket_path, "").expect("create stale socket");
    let state = detect_ecosystem(temp.path(), "test-family").await;
    assert!(
        matches!(state, EcosystemState::Bootstrap),
        "Expected Bootstrap when sockets exist but don't respond, got: {:?}",
        state
    );
}

#[test]
fn test_base64_encode_standard_vectors() {
    assert_eq!(base64_encode(b""), "");
    assert_eq!(base64_encode(b"f"), "Zg==");
    assert_eq!(base64_encode(b"fo"), "Zm8=");
    assert_eq!(base64_encode(b"foo"), "Zm9v");
    assert_eq!(base64_encode(b"foob"), "Zm9vYg==");
    assert_eq!(base64_encode(b"fooba"), "Zm9vYmE=");
    assert_eq!(base64_encode(b"foobar"), "Zm9vYmFy");
}
