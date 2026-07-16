use super::super::*;
use std::path::PathBuf;
use tempfile::tempdir;

#[test]
fn test_runtime_dir_from_xdg_parent_some_joins_membrane_leaf() {
    let temp = tempdir().unwrap();
    let parent = temp.path().join("xdg-runtime-parent");
    let got = SystemPaths::runtime_dir_from_xdg_parent(Some(&parent));
    assert_eq!(got, parent.join(primal_names::MEMBRANE_DIR));
    assert!(got.ends_with(primal_names::MEMBRANE_DIR));
}

#[test]
fn test_runtime_dir_from_xdg_parent_none_uses_canonical_membrane() {
    let got = SystemPaths::runtime_dir_from_xdg_parent(None);
    let lossy = got.to_string_lossy();
    assert_eq!(
        lossy, "/run/membrane",
        "expected canonical /run/membrane path: {lossy}"
    );
}

#[test]
fn test_runtime_dir_from_xdg_parent_none_is_deterministic() {
    let path = SystemPaths::runtime_dir_from_xdg_parent(None);
    assert_eq!(
        path,
        PathBuf::from("/run/membrane"),
        "runtime_dir_from_xdg_parent(None) must be deterministic"
    );
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
    let paths = temp_env::with_vars(
        [
            ("XDG_RUNTIME_DIR", None::<&str>),
            ("BIOMEOS_SOCKET_DIR", None::<&str>),
        ],
        SystemPaths::new_lazy,
    );
    let s = paths.runtime_dir().to_string_lossy();
    assert_eq!(
        s, "/run/membrane",
        "expected /run/membrane when XDG_RUNTIME_DIR is unset: {s}"
    );
}

#[test]
fn test_default_runtime_dir_uses_xdg() {
    let dir = temp_env::with_var("XDG_RUNTIME_DIR", Some("/run/user/1000"), || {
        SystemPaths::default_runtime_dir()
    });
    assert_eq!(dir, PathBuf::from("/run/user/1000/membrane"));
}

#[test]
fn test_default_runtime_dir_fallback() {
    let dir = temp_env::with_vars(
        [
            ("XDG_RUNTIME_DIR", None::<&str>),
            ("USER", Some("testuser")),
            ("BIOMEOS_SOCKET_DIR", None::<&str>),
        ],
        SystemPaths::default_runtime_dir,
    );
    assert_eq!(
        dir,
        PathBuf::from("/run/membrane"),
        "expected canonical /run/membrane fallback"
    );
}

#[test]
fn test_neural_api_socket_path() {
    let path = temp_env::with_var("XDG_RUNTIME_DIR", Some("/run/user/1000"), || {
        SystemPaths::neural_api_socket("nat0")
    });
    assert_eq!(
        path,
        PathBuf::from("/run/user/1000/membrane/neural-api-nat0.sock")
    );
}

#[test]
fn test_neural_api_socket_fallback() {
    let path = temp_env::with_vars(
        [
            ("XDG_RUNTIME_DIR", None::<&str>),
            ("USER", Some("testuser")),
            ("BIOMEOS_SOCKET_DIR", None::<&str>),
        ],
        || SystemPaths::neural_api_socket("family1"),
    );
    let s = path.to_string_lossy();
    assert!(
        s.ends_with("neural-api-family1.sock"),
        "expected socket filename: {s}"
    );
    assert_eq!(
        path,
        PathBuf::from("/run/membrane/neural-api-family1.sock"),
        "expected canonical /run/membrane fallback"
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
