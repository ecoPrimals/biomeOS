use super::super::*;
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
