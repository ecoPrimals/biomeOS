// SPDX-License-Identifier: AGPL-3.0-only
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
