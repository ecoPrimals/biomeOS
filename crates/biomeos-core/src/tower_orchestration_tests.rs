// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

#![expect(clippy::unwrap_used, reason = "test")]

use super::*;
use crate::{PrimalMetadata, TowerConfig, TowerPrimalConfig};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

fn mock_env(vars: &HashMap<String, String>) -> impl Fn(&str) -> Option<String> + '_ {
    move |key: &str| vars.get(key).cloned()
}

#[test]
fn pid_file_path_uses_xdg_runtime_dir() {
    let mut env = HashMap::new();
    env.insert("XDG_RUNTIME_DIR".to_string(), "/run/user/1000".to_string());
    let path = pid_file_path(&mock_env(&env));
    assert_eq!(path, PathBuf::from("/run/user/1000/biomeos/tower.pid"));
}

#[test]
fn pid_file_path_falls_back_to_family_id() {
    let mut env = HashMap::new();
    env.insert("BIOMEOS_FAMILY_ID".to_string(), "nat0".to_string());
    let path = pid_file_path(&mock_env(&env));
    assert_eq!(path, PathBuf::from("/tmp/biomeos-nat0/tower.pid"));
}

#[test]
fn pid_file_path_falls_back_to_default() {
    let env: HashMap<String, String> = HashMap::new();
    let path = pid_file_path(&mock_env(&env));
    assert_eq!(path, PathBuf::from("/tmp/biomeos-default/tower.pid"));
}

#[test]
fn pid_file_path_prefers_biomeos_family_over_family_id() {
    let mut env = HashMap::new();
    env.insert("BIOMEOS_FAMILY_ID".to_string(), "preferred".to_string());
    env.insert("FAMILY_ID".to_string(), "fallback".to_string());
    let path = pid_file_path(&mock_env(&env));
    assert_eq!(path, PathBuf::from("/tmp/biomeos-preferred/tower.pid"));
}

#[test]
fn socket_dir_path_uses_biomeos_socket_dir() {
    let mut env = HashMap::new();
    env.insert(
        "BIOMEOS_SOCKET_DIR".to_string(),
        "/custom/sockets".to_string(),
    );
    let path = socket_dir_path(&mock_env(&env));
    assert_eq!(path, PathBuf::from("/custom/sockets"));
}

#[test]
fn socket_dir_path_uses_xdg_runtime_dir() {
    let mut env = HashMap::new();
    env.insert("XDG_RUNTIME_DIR".to_string(), "/run/user/1000".to_string());
    let path = socket_dir_path(&mock_env(&env));
    assert_eq!(path, PathBuf::from("/run/user/1000/biomeos/sockets"));
}

#[test]
fn socket_dir_path_falls_back_to_family_tmp() {
    let mut env = HashMap::new();
    env.insert("FAMILY_ID".to_string(), "gamma".to_string());
    let path = socket_dir_path(&mock_env(&env));
    assert_eq!(path, PathBuf::from("/tmp/biomeos-gamma/sockets"));
}

#[test]
fn write_and_read_pid_file() {
    let dir = tempfile::tempdir().unwrap();
    let pid_path = dir.path().join("tower.pid");

    write_pid_file(&pid_path).unwrap();

    let pid = read_pid(&pid_path).unwrap();
    #[expect(
        clippy::cast_possible_wrap,
        reason = "PID fits i32 on all supported platforms"
    )]
    let expected = std::process::id() as i32;
    assert_eq!(pid, expected);
}

#[test]
fn cleanup_pid_file_removes_file() {
    let dir = tempfile::tempdir().unwrap();
    let pid_path = dir.path().join("tower.pid");
    std::fs::write(&pid_path, "12345").unwrap();

    assert!(pid_path.exists());
    cleanup_pid_file(&pid_path);
    assert!(!pid_path.exists());
}

#[test]
fn cleanup_pid_file_noop_if_missing() {
    let dir = tempfile::tempdir().unwrap();
    let pid_path = dir.path().join("nonexistent.pid");
    cleanup_pid_file(&pid_path);
}

#[test]
fn read_pid_returns_error_on_invalid_content() {
    let dir = tempfile::tempdir().unwrap();
    let pid_path = dir.path().join("bad.pid");
    std::fs::write(&pid_path, "not-a-number").unwrap();

    assert!(read_pid(&pid_path).is_err());
}

#[test]
fn read_pid_returns_error_on_missing_file() {
    let dir = tempfile::tempdir().unwrap();
    let pid_path = dir.path().join("missing.pid");
    assert!(read_pid(&pid_path).is_err());
}

#[test]
fn list_active_sockets_finds_sock_files() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("beardog-nat0.sock"), "").unwrap();
    std::fs::write(dir.path().join("songbird-nat0.sock"), "").unwrap();
    std::fs::write(dir.path().join("other.txt"), "").unwrap();

    let sockets = list_active_sockets(dir.path());
    assert_eq!(sockets.len(), 2);
    assert!(sockets.iter().any(|s| s.contains("beardog")));
    assert!(sockets.iter().any(|s| s.contains("songbird")));
}

#[test]
fn list_active_sockets_empty_on_missing_dir() {
    let sockets = list_active_sockets(Path::new("/nonexistent/path"));
    assert!(sockets.is_empty());
}

#[test]
fn format_capabilities_returns_known_categories() {
    let caps = format_capabilities();
    assert!(caps.len() >= 7);
    assert!(caps.iter().any(|(name, _)| *name == "Security"));
    assert!(caps.iter().any(|(name, _)| *name == "Discovery"));
    assert!(caps.iter().any(|(name, _)| *name == "AI"));
}

#[test]
fn tower_status_not_running_when_no_pid_file() {
    let dir = tempfile::tempdir().unwrap();
    let pid_path = dir.path().join("tower.pid");
    let env: HashMap<String, String> = HashMap::new();

    let status = tower_status(&pid_path, &mock_env(&env)).unwrap();
    assert!(matches!(status, TowerStatusReport::NotRunning));
}

#[test]
fn stop_tower_errors_when_no_pid_file() {
    let dir = tempfile::tempdir().unwrap();
    let pid_path = dir.path().join("tower.pid");
    assert!(stop_tower(&pid_path).is_err());
}

#[tokio::test]
async fn metadata_to_primal_converts_correctly() {
    let metadata = PrimalMetadata {
        id: "test-primal".to_string(),
        binary: PathBuf::from("/usr/bin/test-primal"),
        provides: vec!["security".to_string(), "crypto".to_string()],
        requires: vec!["discovery".to_string()],
        version: Some("1.0.0".to_string()),
        name: Some("Test Primal".to_string()),
    };

    let primal = metadata_to_primal(&metadata);
    assert!(primal.is_ok());
}

#[tokio::test]
async fn collect_primals_from_empty_config() {
    let config = TowerConfig::default_config();
    let primals = collect_primals(&config, None).await.unwrap();
    assert!(primals.is_empty());
}

#[test]
fn pid_file_path_uses_family_id_env_alone() {
    let mut env = HashMap::new();
    env.insert("FAMILY_ID".to_string(), "only_family".to_string());
    let path = pid_file_path(&mock_env(&env));
    assert_eq!(path, PathBuf::from("/tmp/biomeos-only_family/tower.pid"));
}

#[test]
fn socket_dir_prefers_biomeos_socket_dir_over_xdg() {
    let mut env = HashMap::new();
    env.insert(
        "BIOMEOS_SOCKET_DIR".to_string(),
        "/sock/override".to_string(),
    );
    env.insert("XDG_RUNTIME_DIR".to_string(), "/run/user/1".to_string());
    assert_eq!(
        socket_dir_path(&mock_env(&env)),
        PathBuf::from("/sock/override")
    );
}

#[test]
fn read_pid_trims_whitespace() {
    let dir = tempfile::tempdir().unwrap();
    let pid_path = dir.path().join("tower.pid");
    std::fs::write(&pid_path, "  424242 \n").unwrap();
    assert_eq!(read_pid(&pid_path).unwrap(), 424_242);
}

#[test]
fn std_env_lookup_missing_var_returns_none() {
    assert!(std_env_lookup("BIOMEOS_TEST_ENV_KEY_THAT_SHOULD_NOT_EXIST_XYZ").is_none());
}

#[test]
fn tower_status_invalid_pid_zero() {
    let dir = tempfile::tempdir().unwrap();
    let pid_path = dir.path().join("tower.pid");
    std::fs::write(&pid_path, "0").unwrap();
    let env: HashMap<String, String> = HashMap::new();
    let status = tower_status(&pid_path, &mock_env(&env)).unwrap();
    assert!(matches!(status, TowerStatusReport::InvalidPid));
}

#[test]
fn tower_status_invalid_pid_negative() {
    let dir = tempfile::tempdir().unwrap();
    let pid_path = dir.path().join("tower.pid");
    std::fs::write(&pid_path, "-3").unwrap();
    let env: HashMap<String, String> = HashMap::new();
    let status = tower_status(&pid_path, &mock_env(&env)).unwrap();
    assert!(matches!(status, TowerStatusReport::InvalidPid));
}

#[cfg(unix)]
#[test]
fn stop_tower_rejects_non_positive_pid_in_file() {
    let dir = tempfile::tempdir().unwrap();
    let pid_path = dir.path().join("tower.pid");
    std::fs::write(&pid_path, "-1").unwrap();
    assert!(stop_tower(&pid_path).is_err());
}

#[cfg(unix)]
#[test]
fn tower_status_running_for_current_process() {
    let dir = tempfile::tempdir().unwrap();
    let pid_path = dir.path().join("tower.pid");
    #[expect(
        clippy::cast_possible_wrap,
        reason = "PID fits i32 on all supported platforms"
    )]
    let my_pid = std::process::id() as i32;
    std::fs::write(&pid_path, my_pid.to_string()).unwrap();
    let mut env = HashMap::new();
    env.insert("BIOMEOS_FAMILY_ID".to_string(), "fam-x".to_string());
    let status = tower_status(&pid_path, &mock_env(&env)).unwrap();
    match status {
        TowerStatusReport::Running {
            pid,
            family_id,
            socket_dir,
            ..
        } => {
            assert_eq!(pid, my_pid);
            assert_eq!(family_id.as_deref(), Some("fam-x"));
            assert!(!socket_dir.as_os_str().is_empty());
        }
        other => panic!("expected Running, got {other:?}"),
    }
}

#[cfg(unix)]
#[test]
fn tower_status_stale_when_pid_not_running() {
    let dir = tempfile::tempdir().unwrap();
    let pid_path = dir.path().join("tower.pid");
    std::fs::write(&pid_path, "2147483645").unwrap();
    let env: HashMap<String, String> = HashMap::new();
    let status = tower_status(&pid_path, &mock_env(&env)).unwrap();
    match status {
        TowerStatusReport::Stale { pid } => assert_eq!(pid, 2_147_483_645),
        other => panic!("expected Stale, got {other:?}"),
    }
    assert!(!pid_path.exists());
}

#[cfg(not(unix))]
#[test]
fn tower_status_non_unix_skips_ps_check() {
    let dir = tempfile::tempdir().unwrap();
    let pid_path = dir.path().join("tower.pid");
    std::fs::write(&pid_path, "9999").unwrap();
    let env: HashMap<String, String> = HashMap::new();
    let status = tower_status(&pid_path, &mock_env(&env)).unwrap();
    match status {
        TowerStatusReport::Running {
            pid,
            sockets,
            family_id,
            ..
        } => {
            assert_eq!(pid, 9999);
            assert!(sockets.is_empty());
            assert!(family_id.is_none());
        }
        other => panic!("expected Running, got {other:?}"),
    }
}

#[tokio::test]
async fn metadata_to_primal_accepts_empty_capability_lists() {
    let metadata = PrimalMetadata {
        id: "empty-caps".to_string(),
        binary: PathBuf::from("/bin/true"),
        provides: vec![],
        requires: vec![],
        version: None,
        name: None,
    };
    let primal = metadata_to_primal(&metadata).unwrap();
    assert!(primal.provides().is_empty());
    assert!(primal.requires().is_empty());
}

#[tokio::test]
async fn config_to_primal_explicit_capabilities_skips_auto_discover() {
    let config = TowerPrimalConfig {
        binary: PathBuf::from("/bin/true"),
        id: Some("explicit".to_string()),
        provides: vec!["alpha".to_string()],
        requires: vec!["beta".to_string()],
        http_port: 0,
        protocol: None,
        env: HashMap::new(),
        auto_discover: true,
    };
    let primal = config_to_primal(&config).await.unwrap();
    assert_eq!(primal.provides().len(), 1);
    assert_eq!(primal.requires().len(), 1);
}

#[tokio::test]
async fn config_to_primal_auto_discover_fallback_when_query_fails() {
    let config = TowerPrimalConfig {
        binary: PathBuf::from("/bin/false"),
        id: Some("fallback-id".to_string()),
        provides: vec![],
        requires: vec![],
        http_port: 0,
        protocol: None,
        env: HashMap::new(),
        auto_discover: true,
    };
    let primal = config_to_primal(&config).await.unwrap();
    assert!(primal.provides().is_empty());
}

#[tokio::test]
async fn config_to_primal_applies_env_protocol_and_http_port() {
    let mut env_map = HashMap::new();
    env_map.insert("MY_VAR".to_string(), "1".to_string());
    let config = TowerPrimalConfig {
        binary: PathBuf::from("/bin/true"),
        id: Some("rich".to_string()),
        provides: vec!["p".to_string()],
        requires: vec![],
        http_port: 9000,
        protocol: Some("jsonrpc".to_string()),
        env: env_map,
        auto_discover: false,
    };
    let primal = config_to_primal(&config).await.unwrap();
    assert_eq!(primal.provides().len(), 1);
}

#[tokio::test]
async fn collect_primals_merges_config_primal_with_empty_scan() {
    let scan = tempfile::tempdir().unwrap();
    let mut tower = TowerConfig::default_config();
    tower.primals.push(TowerPrimalConfig {
        binary: PathBuf::from("/bin/true"),
        id: Some("from-config".to_string()),
        provides: vec!["x".to_string()],
        requires: vec![],
        http_port: 0,
        protocol: None,
        env: HashMap::new(),
        auto_discover: false,
    });
    let primals = collect_primals(&tower, Some(scan.path())).await.unwrap();
    assert_eq!(primals.len(), 1);
}

#[tokio::test]
#[cfg(unix)]
async fn collect_primals_discovers_executable_in_scan_dir() {
    let scan = tempfile::tempdir().unwrap();
    let bin = scan.path().join("scan-primal");
    std::fs::copy("/bin/true", &bin).unwrap();
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(&bin, std::fs::Permissions::from_mode(0o755)).unwrap();

    let primals = collect_primals(&TowerConfig::default_config(), Some(scan.path()))
        .await
        .unwrap();
    assert_eq!(primals.len(), 1);
}

// ========================================================================
// run_tower early-exit paths
// ========================================================================

#[tokio::test]
async fn run_tower_returns_ok_when_no_primals_configured() {
    let dir = tempfile::tempdir().unwrap();
    let config_path = dir.path().join("tower.toml");
    let env: HashMap<String, String> = HashMap::new();
    let result = run_tower(&config_path, None, false, &mock_env(&env)).await;
    assert!(result.is_ok(), "early exit with no primals should succeed");
}

#[tokio::test]
async fn run_tower_with_existing_empty_config_no_primals() {
    let dir = tempfile::tempdir().unwrap();
    let config_path = dir.path().join("tower.toml");
    std::fs::write(&config_path, "[tower]\nname = \"test-tower\"\n").unwrap();
    let env: HashMap<String, String> = HashMap::new();
    let result = run_tower(&config_path, None, false, &mock_env(&env)).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn run_tower_with_empty_scan_dir_no_primals() {
    let dir = tempfile::tempdir().unwrap();
    let config_path = dir.path().join("tower.toml");
    let scan = tempfile::tempdir().unwrap();
    let env: HashMap<String, String> = HashMap::new();
    let result = run_tower(
        &config_path,
        Some(scan.path().to_path_buf()),
        false,
        &mock_env(&env),
    )
    .await;
    assert!(result.is_ok());
}

// ========================================================================
// config_to_primal edge cases
// ========================================================================

#[tokio::test]
async fn config_to_primal_auto_discover_false_uses_config_directly() {
    let config = TowerPrimalConfig {
        binary: PathBuf::from("/bin/true"),
        id: Some("no-discover".to_string()),
        provides: vec!["cap-a".to_string()],
        requires: vec!["cap-b".to_string()],
        http_port: 0,
        protocol: None,
        env: HashMap::new(),
        auto_discover: false,
    };
    let primal = config_to_primal(&config).await.unwrap();
    assert_eq!(primal.provides().len(), 1);
    assert_eq!(primal.requires().len(), 1);
}

#[tokio::test]
async fn config_to_primal_id_none_derives_from_binary_stem() {
    let config = TowerPrimalConfig {
        binary: PathBuf::from("/usr/local/bin/my-primal"),
        id: None,
        provides: vec![],
        requires: vec![],
        http_port: 0,
        protocol: None,
        env: HashMap::new(),
        auto_discover: true,
    };
    let primal = config_to_primal(&config).await.unwrap();
    drop(primal);
}

#[tokio::test]
async fn config_to_primal_zero_http_port_not_set() {
    let config = TowerPrimalConfig {
        binary: PathBuf::from("/bin/true"),
        id: Some("zero-port".to_string()),
        provides: vec!["x".to_string()],
        requires: vec![],
        http_port: 0,
        protocol: Some("jsonrpc".to_string()),
        env: HashMap::new(),
        auto_discover: false,
    };
    let primal = config_to_primal(&config).await.unwrap();
    assert_eq!(primal.provides().len(), 1);
}

// ========================================================================
// stop_tower edge cases
// ========================================================================

#[test]
fn stop_tower_rejects_zero_pid() {
    let dir = tempfile::tempdir().unwrap();
    let pid_path = dir.path().join("tower.pid");
    std::fs::write(&pid_path, "0").unwrap();
    let err = stop_tower(&pid_path).unwrap_err();
    assert!(err.to_string().contains("Invalid PID"), "{err}");
}

#[cfg(unix)]
#[test]
fn stop_tower_cleans_up_stale_pid() {
    let dir = tempfile::tempdir().unwrap();
    let pid_path = dir.path().join("tower.pid");
    std::fs::write(&pid_path, "2147483646").unwrap();
    let _ = stop_tower(&pid_path);
    assert!(!pid_path.exists());
}

// ========================================================================
// socket_dir_path edge cases
// ========================================================================

#[test]
fn socket_dir_path_defaults_without_any_env() {
    let env: HashMap<String, String> = HashMap::new();
    let path = socket_dir_path(&mock_env(&env));
    assert_eq!(path, PathBuf::from("/tmp/biomeos-default/sockets"));
}

#[test]
fn socket_dir_path_uses_biomeos_family_id() {
    let mut env = HashMap::new();
    env.insert("BIOMEOS_FAMILY_ID".to_string(), "beta".to_string());
    let path = socket_dir_path(&mock_env(&env));
    assert_eq!(path, PathBuf::from("/tmp/biomeos-beta/sockets"));
}

#[test]
fn socket_dir_path_xdg_over_family_fallback() {
    let mut env = HashMap::new();
    env.insert("XDG_RUNTIME_DIR".to_string(), "/run/user/42".to_string());
    env.insert("BIOMEOS_FAMILY_ID".to_string(), "ignored".to_string());
    let path = socket_dir_path(&mock_env(&env));
    assert_eq!(path, PathBuf::from("/run/user/42/biomeos/sockets"));
}

// ========================================================================
// write_pid_file edge cases
// ========================================================================

#[test]
fn write_pid_file_creates_nested_parent_directories() {
    let dir = tempfile::tempdir().unwrap();
    let pid_path = dir.path().join("a").join("b").join("c").join("tower.pid");
    write_pid_file(&pid_path).unwrap();
    assert!(pid_path.exists());
    let pid = read_pid(&pid_path).unwrap();
    #[expect(
        clippy::cast_possible_wrap,
        reason = "PID fits i32 on all supported platforms"
    )]
    let expected = std::process::id() as i32;
    assert_eq!(pid, expected);
}

// ========================================================================
// list_active_sockets edge cases
// ========================================================================

#[test]
fn list_active_sockets_ignores_non_sock_files() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("config.toml"), "").unwrap();
    std::fs::write(dir.path().join("data.json"), "").unwrap();
    std::fs::write(dir.path().join("README.md"), "").unwrap();
    let sockets = list_active_sockets(dir.path());
    assert!(sockets.is_empty());
}

#[test]
fn list_active_sockets_empty_directory() {
    let dir = tempfile::tempdir().unwrap();
    let sockets = list_active_sockets(dir.path());
    assert!(sockets.is_empty());
}

// ========================================================================
// TowerStatusReport and format_capabilities
// ========================================================================

#[test]
fn tower_status_report_debug_formatting() {
    let not_running = TowerStatusReport::NotRunning;
    assert!(format!("{not_running:?}").contains("NotRunning"));

    let invalid = TowerStatusReport::InvalidPid;
    assert!(format!("{invalid:?}").contains("InvalidPid"));

    let stale = TowerStatusReport::Stale { pid: 999 };
    let stale_dbg = format!("{stale:?}");
    assert!(stale_dbg.contains("Stale"));
    assert!(stale_dbg.contains("999"));

    let running = TowerStatusReport::Running {
        pid: 42,
        socket_dir: PathBuf::from("/tmp/sockets"),
        sockets: vec!["test.sock".to_string()],
        family_id: Some("fam-z".to_string()),
    };
    let running_dbg = format!("{running:?}");
    assert!(running_dbg.contains("Running"));
    assert!(running_dbg.contains("42"));
    assert!(running_dbg.contains("fam-z"));
}

#[test]
fn format_capabilities_includes_all_known_categories() {
    let caps = format_capabilities();
    assert!(caps.len() >= 8);
    let names: Vec<&str> = caps.iter().map(|(n, _)| *n).collect();
    assert!(names.contains(&"Security"));
    assert!(names.contains(&"Discovery"));
    assert!(names.contains(&"Compute"));
    assert!(names.contains(&"AI"));
    assert!(names.contains(&"Storage"));
    assert!(names.contains(&"Observability"));
    assert!(names.contains(&"Federation"));
    assert!(names.contains(&"Network"));
}

// ========================================================================
// pid_file_path with FAMILY_ID only
// ========================================================================

#[test]
fn pid_file_path_xdg_takes_precedence_over_both_family_vars() {
    let mut env = HashMap::new();
    env.insert("XDG_RUNTIME_DIR".to_string(), "/xdg/rt".to_string());
    env.insert("BIOMEOS_FAMILY_ID".to_string(), "fam-a".to_string());
    env.insert("FAMILY_ID".to_string(), "fam-b".to_string());
    let path = pid_file_path(&mock_env(&env));
    assert_eq!(path, PathBuf::from("/xdg/rt/biomeos/tower.pid"));
}

// ========================================================================
// tower_status with family info
// ========================================================================

#[cfg(unix)]
#[test]
fn tower_status_running_includes_family_id_from_env() {
    let dir = tempfile::tempdir().unwrap();
    let pid_path = dir.path().join("tower.pid");
    #[expect(
        clippy::cast_possible_wrap,
        reason = "PID fits i32 on all supported platforms"
    )]
    let my_pid = std::process::id() as i32;
    std::fs::write(&pid_path, my_pid.to_string()).unwrap();
    let mut env = HashMap::new();
    env.insert("FAMILY_ID".to_string(), "fam-only".to_string());
    let status = tower_status(&pid_path, &mock_env(&env)).unwrap();
    match status {
        TowerStatusReport::Running { family_id, .. } => {
            assert_eq!(family_id.as_deref(), Some("fam-only"));
        }
        other => panic!("expected Running, got {other:?}"),
    }
}

// ========================================================================
// read_pid edge cases
// ========================================================================

#[test]
fn read_pid_parses_large_pid() {
    let dir = tempfile::tempdir().unwrap();
    let pid_path = dir.path().join("tower.pid");
    std::fs::write(&pid_path, "2147483647").unwrap();
    let pid = read_pid(&pid_path).unwrap();
    assert_eq!(pid, i32::MAX);
}

#[test]
fn read_pid_rejects_empty_file() {
    let dir = tempfile::tempdir().unwrap();
    let pid_path = dir.path().join("tower.pid");
    std::fs::write(&pid_path, "").unwrap();
    assert!(read_pid(&pid_path).is_err());
}
