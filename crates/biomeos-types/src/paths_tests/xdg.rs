use super::super::*;
use crate::primal_names;
use std::path::Path;
use tempfile::tempdir;

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
