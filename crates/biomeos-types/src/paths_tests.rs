// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Tests for `SystemPaths` XDG path resolution (extracted from `paths.rs`).

#![expect(clippy::unwrap_used, reason = "test assertions use unwrap for clarity")]

use super::*;
use crate::primal_names;
use tempfile::tempdir;

#[test]
fn test_system_paths_with_base() {
    let temp = tempdir().unwrap();
    let paths = SystemPaths::with_base(temp.path()).unwrap();

    assert!(paths.runtime_dir().exists());
    assert!(paths.data_dir().exists());
    assert!(paths.config_dir().exists());
    assert!(paths.cache_dir().exists());
    assert!(paths.state_dir().exists());
}

#[test]
fn test_primal_socket_path() {
    let temp = tempdir().unwrap();
    let paths = SystemPaths::with_base(temp.path()).unwrap();

    let socket = paths.primal_socket("beardog-main");
    assert_eq!(socket.file_name().unwrap(), "beardog-main.sock");
    assert!(socket.starts_with(paths.runtime_dir()));
}

#[test]
fn test_database_paths() {
    let temp = tempdir().unwrap();
    let paths = SystemPaths::with_base(temp.path()).unwrap();

    let metrics_db = paths.metrics_db();
    assert_eq!(metrics_db.file_name().unwrap(), "metrics.db");
    assert!(metrics_db.starts_with(paths.data_dir()));

    let custom_db = paths.database("custom");
    assert_eq!(custom_db.file_name().unwrap(), "custom.db");
}

#[test]
fn test_config_paths() {
    let temp = tempdir().unwrap();
    let paths = SystemPaths::with_base(temp.path()).unwrap();

    let main_config = paths.main_config();
    assert_eq!(main_config.file_name().unwrap(), "biomeos.toml");
    assert!(main_config.starts_with(paths.config_dir()));

    let niche_dir = paths.niche_dir();
    assert_eq!(niche_dir.file_name().unwrap(), "niches");
}

#[test]
fn test_log_paths() {
    let temp = tempdir().unwrap();
    let paths = SystemPaths::with_base(temp.path()).unwrap();

    let log = paths.log_file("test-service");
    assert!(log.to_string_lossy().contains("test-service.log"));
    assert!(log.starts_with(paths.state_dir()));
}

#[test]
fn test_genetic_seed_path() {
    let temp = tempdir().unwrap();
    let paths = SystemPaths::with_base(temp.path()).unwrap();

    let seed = paths.genetic_seed("family-alpha");
    assert!(seed.to_string_lossy().contains("family-alpha.seed"));
    assert!(seed.starts_with(paths.data_dir()));
}

#[test]
fn test_new_lazy_and_ensure_all_dirs() {
    let temp = tempdir().unwrap();
    let base = temp.path().join("lazy-base");
    std::fs::create_dir_all(&base).unwrap();

    let _paths = SystemPaths::with_base(&base).unwrap();
    let lazy_paths = SystemPaths::new_lazy();
    let _ = lazy_paths.runtime_dir();
    let _ = lazy_paths.data_dir();
    let _ = lazy_paths.config_dir();
    let _ = lazy_paths.cache_dir();
    let _ = lazy_paths.state_dir();

    let paths_with_base = SystemPaths::with_base(&base).unwrap();
    assert!(paths_with_base.ensure_all_dirs().is_ok());
}

#[test]
fn test_default_impl() {
    let paths = SystemPaths::default();
    assert!(!paths.runtime_dir().as_os_str().is_empty());
}

#[test]
fn test_all_path_resolution_methods() {
    let temp = tempdir().unwrap();
    let paths = SystemPaths::with_base(temp.path()).unwrap();

    let pid = paths.pid_file("test-service");
    assert!(pid.to_string_lossy().contains("test-service.pid"));

    let lock = paths.lock_file("test-lock");
    assert!(lock.to_string_lossy().contains("test-lock.lock"));

    let spore = paths.spore_dir();
    assert!(spore.ends_with("spores"));

    let temp_ws = paths.temp_workspace("my-workspace");
    assert!(temp_ws.to_string_lossy().contains("my-workspace"));

    let download = paths.download_cache();
    assert!(download.ends_with("downloads"));

    let fossil = paths.fossil_record_dir();
    assert!(fossil.ends_with("fossil-record"));

    let audit = paths.audit_log();
    assert!(audit.ends_with("audit.log"));

    let graph = paths.graph_dir();
    assert!(graph.ends_with("graphs"));
}

#[test]
fn test_path_error_display() {
    let err = PathError::InvalidPath("bad-path".to_string());
    assert!(err.to_string().contains("Invalid path"));
    assert!(err.to_string().contains("bad-path"));
}

#[test]
fn test_xdg_runtime_dir_override() {
    let temp = tempdir().unwrap();
    let xdg_runtime = temp.path().join("xdg-runtime");
    std::fs::create_dir_all(&xdg_runtime).unwrap();

    let paths = SystemPaths::new_with_xdg_overrides(Some(&xdg_runtime), None::<&Path>).unwrap();
    assert!(
        paths
            .runtime_dir()
            .to_string_lossy()
            .contains("xdg-runtime")
    );
}

#[test]
fn test_xdg_data_home_override() {
    let temp = tempdir().unwrap();
    let xdg_data = temp.path().join("xdg-data");
    std::fs::create_dir_all(&xdg_data).unwrap();

    let paths = SystemPaths::new_with_xdg_overrides(None::<&Path>, Some(&xdg_data)).unwrap();
    assert!(paths.data_dir().to_string_lossy().contains("xdg-data"));
}

#[test]
fn test_empty_primal_id_in_socket() {
    let temp = tempdir().unwrap();
    let paths = SystemPaths::with_base(temp.path()).unwrap();
    let socket = paths.primal_socket("");
    assert!(socket.ends_with(".sock"));
}

#[test]
fn test_safe_uid() {
    let uid = safe_uid();
    assert_ne!(uid, 0, "safe_uid should return non-zero value");
}

#[test]
fn test_path_error_create_dir_failed_display() {
    let err = PathError::CreateDirFailed {
        path: "/invalid/path".to_string(),
        source: std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied"),
    };
    let s = err.to_string();
    assert!(s.contains("Failed to create directory"));
    assert!(s.contains("/invalid/path"));
}

#[test]
fn test_path_error_no_home_dir_display() {
    let err = PathError::NoHomeDir;
    let s = err.to_string();
    assert!(s.to_lowercase().contains("home"));
}

#[test]
fn test_path_error_invalid_path_display() {
    let err = PathError::InvalidPath("bad/path".to_string());
    let s = err.to_string();
    assert!(s.contains("Invalid path"));
    assert!(s.contains("bad/path"));
}

#[test]
fn test_path_error_debug() {
    let err = PathError::NoHomeDir;
    let s = format!("{:?}", err);
    assert!(s.contains("NoHomeDir"));
}

#[test]
fn test_spore_dir_path() {
    let temp = tempdir().unwrap();
    let paths = SystemPaths::with_base(temp.path()).unwrap();
    let spore = paths.spore_dir();
    assert!(spore.ends_with("spores"));
}

#[test]
fn test_graph_dir_path() {
    let temp = tempdir().unwrap();
    let paths = SystemPaths::with_base(temp.path()).unwrap();
    let graph = paths.graph_dir();
    assert!(graph.ends_with("graphs"));
}

#[test]
fn test_new_lazy_default_paths() {
    let paths = SystemPaths::new_lazy();
    assert!(!paths.runtime_dir().as_os_str().is_empty());
    assert!(!paths.data_dir().as_os_str().is_empty());
    assert!(!paths.config_dir().as_os_str().is_empty());
    assert!(!paths.cache_dir().as_os_str().is_empty());
    assert!(!paths.state_dir().as_os_str().is_empty());
}

/// Same layout as `new()` with all `XDG_*` env vars set, via explicit paths.
#[test]
fn test_system_paths_new_respects_all_xdg_env_overrides() {
    let temp = tempdir().unwrap();
    let run = temp.path().join("xdg-run");
    let data = temp.path().join("xdg-data");
    let cfg = temp.path().join("xdg-cfg");
    let cache = temp.path().join("xdg-cache");
    let state = temp.path().join("xdg-state");
    for p in [&run, &data, &cfg, &cache, &state] {
        std::fs::create_dir_all(p).unwrap();
    }
    let paths = SystemPaths::from_overrides(
        run.join(primal_names::BIOMEOS),
        data.join(primal_names::BIOMEOS),
        cfg.join(primal_names::BIOMEOS),
        cache.join(primal_names::BIOMEOS),
        state.join(primal_names::BIOMEOS),
    )
    .unwrap();
    assert!(paths.runtime_dir().starts_with(&run));
    assert!(paths.data_dir().starts_with(&data));
    assert!(paths.config_dir().starts_with(&cfg));
    assert!(paths.cache_dir().starts_with(&cache));
    assert!(paths.state_dir().starts_with(&state));
}

/// Fallback runtime dir includes a user segment (`biomeos-$USER`).
#[test]
fn test_runtime_dir_fallback_includes_user_from_env() {
    let temp = tempdir().unwrap();
    let runtime = temp.path().join("biomeos-pathstestuser");
    let data = temp.path().join("xdg-data");
    let cfg = temp.path().join("xdg-cfg");
    let cache = temp.path().join("xdg-cache");
    let state = temp.path().join("xdg-state");
    for p in [&runtime, &data, &cfg, &cache, &state] {
        std::fs::create_dir_all(p).unwrap();
    }
    let paths = SystemPaths::from_overrides(
        runtime,
        data.join(primal_names::BIOMEOS),
        cfg.join(primal_names::BIOMEOS),
        cache.join(primal_names::BIOMEOS),
        state.join(primal_names::BIOMEOS),
    )
    .unwrap();
    let s = paths.runtime_dir().to_string_lossy();
    assert!(
        s.contains("pathstestuser"),
        "expected username in fallback runtime path: {s}"
    );
}

/// State dir at `$HOME/.local/state/biomeos` when not using `XDG_STATE_HOME`.
#[test]
fn test_state_dir_prefers_home_local_state_without_xdg_state() {
    let temp = tempdir().unwrap();
    let home = temp.path().join("home-branch");
    std::fs::create_dir_all(&home).unwrap();
    for p in [
        temp.path().join("rt"),
        temp.path().join("dh"),
        temp.path().join("ch"),
        temp.path().join("ca"),
    ] {
        std::fs::create_dir_all(&p).unwrap();
    }
    let expected = home.join(".local/state").join(primal_names::BIOMEOS);
    std::fs::create_dir_all(&expected).unwrap();

    let paths = SystemPaths::from_overrides(
        temp.path().join("rt").join(primal_names::BIOMEOS),
        temp.path().join("dh").join(primal_names::BIOMEOS),
        temp.path().join("ch").join(primal_names::BIOMEOS),
        temp.path().join("ca").join(primal_names::BIOMEOS),
        expected.clone(),
    )
    .unwrap();
    assert_eq!(paths.state_dir(), &expected);
}

#[test]
fn test_system_paths_new_reads_xdg_and_home_from_env() {
    let temp = tempdir().unwrap();
    let home = temp.path().join("home");
    let run = temp.path().join("xdg-run");
    let data = temp.path().join("xdg-data");
    let cfg = temp.path().join("xdg-cfg");
    let cache = temp.path().join("xdg-cache");
    let state = temp.path().join("xdg-state");
    for p in [&home, &run, &data, &cfg, &cache, &state] {
        std::fs::create_dir_all(p).unwrap();
    }

    let paths = temp_env::with_vars(
        [
            ("HOME", Some(home.as_os_str())),
            ("XDG_RUNTIME_DIR", Some(run.as_os_str())),
            ("XDG_DATA_HOME", Some(data.as_os_str())),
            ("XDG_CONFIG_HOME", Some(cfg.as_os_str())),
            ("XDG_CACHE_HOME", Some(cache.as_os_str())),
            ("XDG_STATE_HOME", Some(state.as_os_str())),
        ],
        SystemPaths::new,
    )
    .unwrap();

    assert!(paths.runtime_dir().starts_with(&run));
    assert!(paths.data_dir().starts_with(&data));
    assert!(paths.config_dir().starts_with(&cfg));
    assert!(paths.cache_dir().starts_with(&cache));
    assert!(paths.state_dir().starts_with(&state));
}

#[test]
fn test_runtime_dir_from_xdg_parent_some_joins_biomeos_leaf() {
    let temp = tempdir().unwrap();
    let parent = temp.path().join("xdg-runtime-parent");
    let got = SystemPaths::runtime_dir_from_xdg_parent(Some(&parent));
    assert_eq!(got, parent.join(primal_names::BIOMEOS));
    assert!(got.ends_with(primal_names::BIOMEOS));
}

#[test]
fn test_runtime_dir_from_xdg_parent_none_uses_temp_biomeos_username() {
    let got = temp_env::with_vars(
        [
            ("USER", Some("envtestuser-paths".as_ref())),
            ("USERNAME", None::<&std::ffi::OsStr>),
        ],
        || SystemPaths::runtime_dir_from_xdg_parent(None),
    );
    let lossy = got.to_string_lossy();
    assert!(
        lossy.contains("biomeos-envtestuser-paths"),
        "expected USER in temp-dir fallback: {lossy}"
    );
    assert!(
        lossy.contains("biomeos-"),
        "expected temp-dir fallback prefix: {lossy}"
    );
}

#[test]
fn test_get_username_fallback_to_default_via_runtime_path() {
    let path = temp_env::with_vars(
        [
            ("USER", None::<&std::ffi::OsStr>),
            ("USERNAME", None::<&std::ffi::OsStr>),
        ],
        || SystemPaths::runtime_dir_from_xdg_parent(None),
    );
    assert!(
        path.to_string_lossy()
            .contains(&format!("{}-default", primal_names::BIOMEOS)),
        "expected get_username() fallback 'default' in path: {}",
        path.display()
    );
}

#[cfg(unix)]
#[test]
fn test_ensure_dir_create_dir_failed_on_readonly_parent() {
    use std::os::unix::fs::PermissionsExt;

    let temp = tempdir().unwrap();
    let ro = temp.path().join("readonly-parent");
    std::fs::create_dir_all(&ro).unwrap();
    std::fs::set_permissions(&ro, std::fs::Permissions::from_mode(0o555)).unwrap();

    let bad_runtime = ro.join("nested-biomeos");
    let writable = temp.path().join("writable");
    std::fs::create_dir_all(&writable).unwrap();
    let bio = primal_names::BIOMEOS;
    let data = writable.join("data").join(bio);
    let cfg = writable.join("cfg").join(bio);
    let cache = writable.join("cache").join(bio);
    let state = writable.join("state").join(bio);

    let err = SystemPaths::from_overrides(bad_runtime, data, cfg, cache, state).unwrap_err();
    match err {
        PathError::CreateDirFailed { path, .. } => {
            assert!(
                path.contains("nested-biomeos") || path.contains("readonly-parent"),
                "unexpected path in error: {path}"
            );
        }
        other => panic!("expected CreateDirFailed, got {other:?}"),
    }
}

#[test]
fn test_path_error_invalid_path_and_no_home_debug_display() {
    let inv = PathError::InvalidPath("x/y".to_string());
    assert!(inv.to_string().contains("Invalid path"));
    assert!(inv.to_string().contains("x/y"));
    let inv_dbg = format!("{inv:?}");
    assert!(inv_dbg.contains("InvalidPath") || inv_dbg.contains("x/y"));

    let no_home = PathError::NoHomeDir;
    assert_eq!(no_home.to_string(), "Failed to determine home directory");
    assert!(format!("{no_home:?}").contains("NoHomeDir"));
}

#[test]
fn test_safe_uid_is_u32_and_matches_system_paths_wrapper() {
    let uid: u32 = safe_uid();
    assert_eq!(uid, SystemPaths::safe_uid());
    #[cfg(target_os = "linux")]
    if std::path::Path::new("/proc/self/status").exists() {
        assert_ne!(
            uid, 65534,
            "on Linux with /proc/self/status, expect parsed real uid, not nobody fallback"
        );
    }
}

/// Covers `get_runtime_dir` when `$XDG_RUNTIME_DIR` is unset (`temp_dir` + `biomeos-$USER`).
#[test]
fn test_new_lazy_runtime_without_xdg_runtime_dir() {
    let paths = temp_env::with_var("XDG_RUNTIME_DIR", None::<&str>, SystemPaths::new_lazy);
    let s = paths.runtime_dir().to_string_lossy();
    assert!(
        s.contains("biomeos-"),
        "expected temp-style runtime dir when XDG_RUNTIME_DIR is unset: {s}"
    );
}

#[test]
fn test_default_runtime_dir_uses_xdg() {
    let dir = temp_env::with_var("XDG_RUNTIME_DIR", Some("/run/user/1000"), || {
        SystemPaths::default_runtime_dir()
    });
    assert_eq!(dir, PathBuf::from("/run/user/1000/biomeos"));
}

#[test]
fn test_default_runtime_dir_fallback() {
    let dir = temp_env::with_vars(
        [
            ("XDG_RUNTIME_DIR", None::<&str>),
            ("USER", Some("testuser")),
        ],
        SystemPaths::default_runtime_dir,
    );
    let s = dir.to_string_lossy();
    assert!(
        s.contains("biomeos-testuser"),
        "expected temp fallback containing biomeos-testuser: {s}"
    );
}

#[test]
fn test_neural_api_socket_path() {
    let path = temp_env::with_var("XDG_RUNTIME_DIR", Some("/run/user/1000"), || {
        SystemPaths::neural_api_socket("nat0")
    });
    assert_eq!(
        path,
        PathBuf::from("/run/user/1000/biomeos/neural-api-nat0.sock")
    );
}

#[test]
fn test_neural_api_socket_fallback() {
    let path = temp_env::with_vars(
        [
            ("XDG_RUNTIME_DIR", None::<&str>),
            ("USER", Some("testuser")),
        ],
        || SystemPaths::neural_api_socket("family1"),
    );
    let s = path.to_string_lossy();
    assert!(
        s.ends_with("neural-api-family1.sock"),
        "expected socket filename: {s}"
    );
    assert!(
        s.contains("biomeos-testuser"),
        "expected temp fallback dir: {s}"
    );
}

/// Covers etcetera fallbacks in `get_*_dir` and `get_state_dir`'s `data_dir/state` path when
/// `HOME` and XDG base vars are unset.
#[test]
fn test_new_lazy_etcetera_when_home_and_xdg_unset() {
    let paths = temp_env::with_vars(
        [
            ("HOME", None::<&str>),
            ("XDG_DATA_HOME", None::<&str>),
            ("XDG_CONFIG_HOME", None::<&str>),
            ("XDG_CACHE_HOME", None::<&str>),
            ("XDG_STATE_HOME", None::<&str>),
        ],
        SystemPaths::new_lazy,
    );
    assert!(!paths.data_dir().as_os_str().is_empty());
    assert!(!paths.config_dir().as_os_str().is_empty());
    assert!(!paths.cache_dir().as_os_str().is_empty());
    assert!(!paths.state_dir().as_os_str().is_empty());
    assert!(
        paths.state_dir().ends_with("state"),
        "expected state_dir …/biomeos/state: {:?}",
        paths.state_dir()
    );
}
