// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Nucleus mode tests (part 1 of 2; see `nucleus_tests2.rs`).

#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]

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
fn test_generate_jwt_secret_is_valid_base64() {
    let secret = super::generate_jwt_secret();
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
    let s1 = super::generate_jwt_secret();
    let s2 = super::generate_jwt_secret();
    assert_ne!(s1, s2, "Two consecutive JWT secrets should differ");
}

#[test]
fn test_generate_jwt_secret_length() {
    let secret = super::generate_jwt_secret();
    assert_eq!(secret.len(), 64, "48 bytes -> 64 base64 chars");
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
    assert!(
        envs.iter()
            .any(|(k, _)| k == &std::ffi::OsStr::new("SONGBIRD_SECURITY_PROVIDER"))
    );
    assert!(
        envs.iter()
            .any(|(k, _)| k == &std::ffi::OsStr::new("FAMILY_ID"))
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
fn test_generate_jwt_secret_decodes_to_48_bytes() {
    use base64::Engine;
    let secret = super::generate_jwt_secret();
    let decoded = base64::engine::general_purpose::STANDARD
        .decode(&secret)
        .unwrap();
    assert_eq!(decoded.len(), 48);
}

#[test]
fn test_resolve_socket_dir_default() {
    let result = resolve_socket_dir_with(None);
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
        "Expected parse error: {err}"
    );
}

#[test]
#[cfg(unix)]
fn test_discover_binaries_finds_in_path() {
    use std::os::unix::fs::PermissionsExt;

    let temp_dir = tempfile::tempdir().expect("temp dir");
    let unique_name = "biomeos_test_binary_xyz";
    let binary_path = temp_dir.path().join(unique_name);
    std::fs::write(&binary_path, "#!/bin/sh\nexit 0").expect("write test binary");
    let mut perms = std::fs::metadata(&binary_path).unwrap().permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(&binary_path, perms).unwrap();

    let path_dirs = vec![temp_dir.path()];
    let map = discover_binaries_with(&[unique_name], None, &path_dirs, None)
        .expect("discover should succeed");

    assert!(
        map.contains_key(unique_name),
        "{unique_name} should be found in PATH, got: {map:?}"
    );
}

#[tokio::test]
async fn test_wait_for_socket_times_out_on_nonexistent() {
    let temp = tempfile::tempdir().expect("temp dir");
    let path = temp.path().join("nonexistent.sock");
    let result = wait_for_socket(&path, Duration::from_millis(50), Duration::ZERO).await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("did not appear"));
}

#[tokio::test]
async fn test_wait_for_socket_succeeds_when_file_exists() {
    let temp = tempfile::tempdir().expect("temp dir");
    let path = temp.path().join("test.sock");
    std::fs::write(&path, "").expect("create socket file");
    let result = wait_for_socket(&path, Duration::from_secs(1), Duration::ZERO).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_detect_ecosystem_bootstrap_when_dir_nonexistent() {
    let temp = tempfile::tempdir().expect("temp dir");
    let nonexistent = temp.path().join("nonexistent_subdir");
    let state = detect_ecosystem(&nonexistent, "test-family").await;
    assert!(
        matches!(state, EcosystemState::Bootstrap),
        "Expected Bootstrap when dir does not exist, got: {state:?}"
    );
}

#[tokio::test]
async fn test_detect_ecosystem_bootstrap_when_dir_empty() {
    let temp = tempfile::tempdir().expect("temp dir");
    let state = detect_ecosystem(temp.path(), "test-family").await;
    assert!(
        matches!(state, EcosystemState::Bootstrap),
        "Expected Bootstrap when dir is empty, got: {state:?}"
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
        "Expected Bootstrap when sockets exist but don't respond, got: {state:?}"
    );
}

#[test]
fn test_generate_jwt_secret_chars_are_base64() {
    let secret = super::generate_jwt_secret();
    assert!(
        secret
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '='),
        "All chars should be valid base64"
    );
}

#[test]
fn test_build_primal_command_nestgate_has_jwt_secret() {
    let cmd = build_primal_command(
        "nestgate",
        std::path::Path::new("/usr/bin/nestgate"),
        std::path::Path::new("/tmp/sock"),
        "fam1",
        "node1",
    );
    let envs: Vec<_> = cmd.get_envs().collect();
    let jwt = envs
        .iter()
        .find(|(k, _)| k == &std::ffi::OsStr::new("NESTGATE_JWT_SECRET"));
    assert!(jwt.is_some(), "NESTGATE_JWT_SECRET should be set");
    let (_, v) = jwt.unwrap();
    assert!(v.is_some(), "JWT secret value should be present");
    let secret = v.unwrap().to_string_lossy();
    assert!(!secret.is_empty(), "JWT secret should be non-empty");
}

#[test]
fn test_build_primal_command_squirrel_with_ai_providers() {
    let config = PrimalCommandConfig {
        name: "squirrel",
        binary: std::path::Path::new("/usr/bin/squirrel"),
        socket_dir: std::path::Path::new("/tmp/sock"),
        family_id: "fam1",
        node_id: "node1",
        anthropic_api_key: Some("test-key"),
        openai_api_key: None,
        ai_http_providers: None,
        ai_default_model: None,
    };
    let cmd = build_primal_command_with(&config);
    let envs: Vec<_> = cmd.get_envs().collect();
    let ai_providers = envs
        .iter()
        .find(|(k, _)| k == &std::ffi::OsStr::new("AI_HTTP_PROVIDERS"));
    assert!(
        ai_providers.is_some(),
        "AI_HTTP_PROVIDERS should be set when API key present"
    );
}

#[test]
fn test_build_primal_command_squirrel_with_openai_key() {
    let config = PrimalCommandConfig {
        name: "squirrel",
        binary: std::path::Path::new("/usr/bin/squirrel"),
        socket_dir: std::path::Path::new("/tmp/sock"),
        family_id: "fam1",
        node_id: "node1",
        anthropic_api_key: None,
        openai_api_key: Some("sk-test"),
        ai_http_providers: None,
        ai_default_model: None,
    };
    let cmd = build_primal_command_with(&config);
    let envs: Vec<_> = cmd.get_envs().collect();
    let ai_providers = envs
        .iter()
        .find(|(k, _)| k == &std::ffi::OsStr::new("AI_HTTP_PROVIDERS"));
    assert!(ai_providers.is_some());
}

#[test]
fn test_discover_binaries_finds_in_plasmidbin() {
    let temp = tempfile::tempdir().expect("temp dir");
    let plasmid_bin = temp.path().join("primals");
    std::fs::create_dir_all(&plasmid_bin).expect("create primals dir");
    let unique_name = "biomeos_test_primal_xyz";
    let binary_path = plasmid_bin.join(unique_name);
    std::fs::write(&binary_path, "#!/bin/sh\nexit 0").expect("write test binary");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&binary_path).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&binary_path, perms).unwrap();
    }

    let plasmid_bin_dir = plasmid_bin.parent().unwrap();
    let map = discover_binaries_with(&[unique_name], Some(plasmid_bin_dir), &[], None)
        .expect("discover should succeed");

    assert!(
        map.contains_key(unique_name),
        "{unique_name} should be found in plasmidBin, got: {map:?}"
    );
}

#[test]
fn test_discover_binaries_finds_in_plasmidbin_subdir() {
    let temp = tempfile::tempdir().expect("temp dir");
    let plasmid_bin = temp.path().join("primals");
    std::fs::create_dir_all(&plasmid_bin).expect("create primals dir");
    let unique_name = "biomeos_test_subdir_xyz";
    let primal_dir = plasmid_bin.join(unique_name);
    std::fs::create_dir_all(&primal_dir).expect("create primal subdir");
    let binary_path = primal_dir.join(unique_name);
    std::fs::write(&binary_path, "#!/bin/sh\nexit 0").expect("write test binary");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&binary_path).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&binary_path, perms).unwrap();
    }

    let plasmid_bin_dir = plasmid_bin.parent().unwrap();
    let map = discover_binaries_with(&[unique_name], Some(plasmid_bin_dir), &[], None)
        .expect("discover should succeed");

    assert!(
        map.contains_key(unique_name),
        "{unique_name} should be found in plasmidBin subdir, got: {map:?}"
    );
}

#[test]
fn test_discover_binaries_warns_on_missing() {
    let map =
        discover_binaries(&["nonexistent_primal_xyz_12345"]).expect("discover should succeed");
    assert!(map.is_empty());
}

#[test]
fn test_format_nucleus_summary_health_check_line() {
    let lines = format_nucleus_summary(
        &[("beardog".to_string(), 1)],
        std::path::Path::new("/tmp/sock"),
        "fam1",
        "node1",
        NucleusMode::Tower,
        "bootstrap",
    );
    assert!(lines.iter().any(|l| l.contains("nc -U")));
    assert!(lines.iter().any(|l| l.contains("health")));
}
