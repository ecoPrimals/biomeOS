use super::*;

#[test]
fn manager_start_succeeds_with_mock_cargo() {
    let harness = MockHarness::new();
    harness.write_bin("cargo", CARGO_OK);
    let path = harness.path_env();
    let manager = harness.manager(ValidationConfig::default());

    temp_env::with_var("PATH", Some(path.as_str()), || {
        manager.start("fed-test").expect("start");
    });
}

#[test]
fn manager_start_fails_when_mock_cargo_errors() {
    let harness = MockHarness::new();
    harness.write_bin("cargo", CARGO_FAIL);
    let path = harness.path_env();
    let manager = harness.manager(ValidationConfig::default());

    temp_env::with_var("PATH", Some(path.as_str()), || {
        let err = manager.start("fed-test").expect_err("start fail");
        assert!(
            err.to_string().contains("benchscale start failed")
                || err.to_string().contains("benchscale failed"),
            "unexpected: {err}"
        );
    });
}

#[test]
fn manager_status_returns_mock_stdout() {
    let harness = MockHarness::new();
    harness.write_bin("cargo", CARGO_STATUS);
    let path = harness.path_env();
    let manager = harness.manager(ValidationConfig::default());

    temp_env::with_var("PATH", Some(path.as_str()), || {
        let status = manager.status("fed-test").expect("status");
        assert!(status.contains("federation running"), "status={status}");
    });
}

#[test]
fn manager_test_completes_despite_mock_failure() {
    let harness = MockHarness::new();
    harness.write_bin("cargo", CARGO_FAIL);
    let path = harness.path_env();
    let manager = harness.manager(ValidationConfig::default());

    temp_env::with_var("PATH", Some(path.as_str()), || {
        manager.test("fed-test").expect("test returns Ok on stderr");
    });
}

#[test]
fn manager_stop_completes_despite_mock_failure() {
    let harness = MockHarness::new();
    harness.write_bin("cargo", CARGO_FAIL);
    let path = harness.path_env();
    let manager = harness.manager(ValidationConfig::default());

    temp_env::with_var("PATH", Some(path.as_str()), || {
        manager.stop("fed-test").expect("stop returns Ok on stderr");
    });
}

#[test]
fn manager_destroy_completes_despite_mock_failure() {
    let harness = MockHarness::new();
    harness.write_bin("cargo", CARGO_FAIL);
    let path = harness.path_env();
    let manager = harness.manager(ValidationConfig::default());

    temp_env::with_var("PATH", Some(path.as_str()), || {
        manager
            .destroy("fed-test")
            .expect("destroy returns Ok on stderr");
    });
}
#[test]
fn with_validation_config_succeeds_when_benchscale_dir_exists() {
    let benchscale_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("workspace root")
        .join("benchscale");
    let created = !benchscale_root.exists();
    if created {
        std::fs::create_dir_all(&benchscale_root).expect("create benchscale fixture");
    }

    let result = VmFederationManager::with_validation_config(ValidationConfig::default());
    if created {
        let _ = std::fs::remove_dir(&benchscale_root);
    }

    if benchscale_root.exists() && !created {
        result.expect("benchscale already present in workspace");
    } else if created {
        result.expect("benchscale fixture should satisfy constructor");
    }
}

#[test]
fn manager_test_stop_destroy_succeed_with_mock_cargo() {
    let harness = MockHarness::new();
    harness.write_bin("cargo", CARGO_OK);
    let path = harness.path_env();
    let manager = harness.manager(ValidationConfig::default());

    temp_env::with_var("PATH", Some(path.as_str()), || {
        manager.test("fed-test").expect("test");
        manager.stop("fed-test").expect("stop");
        manager.destroy("fed-test").expect("destroy");
    });
}

#[test]
fn manager_new_or_with_config_resolves_paths() {
    match VmFederationManager::new() {
        Ok(_manager) => {
            // benchscale present in this environment; path resolution succeeded.
        }
        Err(e) => {
            assert!(
                e.to_string().contains("benchscale not found"),
                "unexpected new() error: {e}"
            );
        }
    }
}
