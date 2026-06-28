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
    let secret = generate_jwt_secret();
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
    let result = run(NucleusRunConfig {
        mode: "invalid_mode_xyz".to_string(),
        node_id: "node1".to_string(),
        family_id: None,
        tcp_port: None,
        tcp_only: false,
        bind: None,
    })
    .await;
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
    let secret = generate_jwt_secret();
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

#[tokio::test]
async fn test_cleanup_stale_sockets_removes_dead_sockets() {
    let temp = tempfile::tempdir().expect("temp dir");
    // Create a fake "stale" .sock file (no listener behind it)
    let stale = temp.path().join("stale-fam1.sock");
    std::fs::write(&stale, "").expect("create stale socket file");
    // Create a matching PID file
    let stale_pid = temp.path().join("stale-fam1.pid");
    std::fs::write(&stale_pid, "99999").expect("create stale pid file");
    // Create a non-socket file that should be left alone
    let other = temp.path().join("config.toml");
    std::fs::write(&other, "key = true").expect("create non-socket file");

    cleanup_stale_sockets(temp.path()).await;

    assert!(!stale.exists(), "Stale socket should be removed");
    assert!(
        !stale_pid.exists(),
        "Orphaned PID file should be removed alongside stale socket"
    );
    assert!(other.exists(), "Non-socket files should not be touched");
}

#[tokio::test]
async fn test_cleanup_stale_sockets_preserves_live_sockets() {
    let temp = tempfile::tempdir().expect("temp dir");
    let live_path = temp.path().join("live-fam1.sock");

    // Create a real listening socket
    let listener = std::os::unix::net::UnixListener::bind(&live_path).expect("bind socket");

    cleanup_stale_sockets(temp.path()).await;

    assert!(
        live_path.exists(),
        "Live socket (with listener) should NOT be removed"
    );
    drop(listener);
}

#[tokio::test]
async fn test_cleanup_stale_sockets_nonexistent_dir() {
    let temp = tempfile::tempdir().expect("temp dir");
    let nonexistent = temp.path().join("does_not_exist");
    // Should not panic
    cleanup_stale_sockets(&nonexistent).await;
}

#[tokio::test]
async fn test_cleanup_stale_sockets_orphaned_pid_without_sock() {
    let temp = tempfile::tempdir().expect("temp dir");
    // PID file with no companion .sock
    let orphan_pid = temp.path().join("gone-fam1.pid");
    std::fs::write(&orphan_pid, "12345").expect("create orphan pid");

    cleanup_stale_sockets(temp.path()).await;

    assert!(
        !orphan_pid.exists(),
        "PID file without companion .sock should be removed"
    );
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
