use super::*;

#[test]
fn with_validation_config_errors_when_benchscale_missing() {
    let err = VmFederationManager::with_validation_config(ValidationConfig::default())
        .err()
        .expect("benchscale should be missing in CI");
    let msg = err.to_string();
    assert!(
        msg.contains("benchscale not found") || msg.contains("parent"),
        "unexpected error: {msg}"
    );
}

#[tokio::test]
async fn manager_create_succeeds_with_mock_commands() {
    let harness = MockHarness::new();
    harness.write_bin("cargo", CARGO_OK);
    harness.write_bin("virsh", VIRSH_WITH_IP);
    harness.write_bin("ssh", SSH_OK);
    let path = harness.path_env();
    let manager = harness.manager(fast_validation_config());

    temp_env::async_with_vars([("PATH", Some(path.as_str()))], async move {
        manager
            .create("fed-test")
            .await
            .expect("create should succeed with mocks");
    })
    .await;
}

#[tokio::test]
async fn manager_create_fails_when_benchscale_create_fails() {
    let harness = MockHarness::new();
    harness.write_bin("cargo", CARGO_FAIL);
    let path = harness.path_env();
    let manager = harness.manager(fast_validation_config());

    temp_env::async_with_vars([("PATH", Some(path.as_str()))], async move {
        let err = manager
            .create("fed-test")
            .await
            .expect_err("create should fail");
        let msg = err.to_string();
        assert!(
            msg.contains("benchscale create failed") || msg.contains("benchscale failed"),
            "unexpected: {msg}"
        );
    })
    .await;
}

#[tokio::test]
async fn manager_create_fails_when_no_vm_ips_discovered() {
    let harness = MockHarness::new();
    harness.write_bin("cargo", CARGO_OK);
    harness.write_bin("virsh", VIRSH_NO_192_168);
    let path = harness.path_env();
    let manager = harness.manager(fast_validation_config());

    temp_env::async_with_vars([("PATH", Some(path.as_str()))], async move {
        let err = manager.create("fed-test").await.expect_err("no IPs");
        assert!(
            err.to_string().contains("No VM IPs found"),
            "unexpected: {err}"
        );
    })
    .await;
}

#[tokio::test]
async fn manager_create_fails_when_topology_path_not_utf8() {
    let harness = MockHarness::new();
    harness.write_bin("cargo", CARGO_OK);
    let bad_topology = non_utf8_topology_path(harness.root.path());
    std::fs::write(&bad_topology, b"name: bad\n").expect("write bad topology");
    let path = harness.path_env();
    let manager = VmFederationManager::with_paths_for_test(
        harness.benchscale_root.clone(),
        bad_topology,
        fast_validation_config(),
    );

    temp_env::async_with_vars([("PATH", Some(path.as_str()))], async move {
        let err = manager.create("fed-test").await.expect_err("utf8 topology");
        assert!(
            err.to_string().contains("UTF-8") || err.to_string().contains("utf-8"),
            "unexpected: {err}"
        );
    })
    .await;
}

#[tokio::test]
async fn manager_create_fails_when_ssh_never_becomes_ready() {
    let harness = MockHarness::new();
    harness.write_bin("cargo", CARGO_OK);
    harness.write_bin("virsh", VIRSH_WITH_IP);
    harness.write_bin("ssh", SSH_FAIL);
    let path = harness.path_env();
    let manager = harness.manager(ValidationConfig {
        cloud_init_timeout: Duration::from_secs(60),
        ssh_timeout: Duration::from_secs(5),
        ssh_retry_interval: Duration::from_millis(1),
        ssh_max_retries: 2,
    });

    temp_env::async_with_vars([("PATH", Some(path.as_str()))], async move {
        let err = manager
            .create("fed-test")
            .await
            .expect_err("ssh should fail");
        assert!(
            err.to_string().contains("Failed to SSH"),
            "unexpected: {err}"
        );
    })
    .await;
}

#[tokio::test]
async fn manager_create_fails_when_virsh_list_fails() {
    let harness = MockHarness::new();
    harness.write_bin("cargo", CARGO_OK);
    harness.write_bin("virsh", VIRSH_LIST_FAIL);
    let path = harness.path_env();
    let manager = harness.manager(fast_validation_config());

    temp_env::async_with_vars([("PATH", Some(path.as_str()))], async move {
        let err = manager
            .create("fed-test")
            .await
            .expect_err("virsh list fail");
        assert!(
            err.to_string().contains("Failed to list VMs")
                || err.to_string().contains("No VM IPs found"),
            "unexpected: {err}"
        );
    })
    .await;
}
#[tokio::test]
async fn manager_create_fails_when_final_ssh_validation_fails() {
    let harness = MockHarness::new();
    harness.write_bin("cargo", CARGO_OK);
    harness.write_bin("virsh", VIRSH_WITH_IP);
    harness.write_bin("ssh", SSH_OK_UNTIL_VALIDATE);
    let path = harness.path_env();
    let manager = harness.manager(fast_validation_config());

    temp_env::async_with_vars([("PATH", Some(path.as_str()))], async move {
        let err = manager
            .create("fed-test")
            .await
            .expect_err("final ssh validation should fail");
        assert!(
            err.to_string().contains("SSH validation failed"),
            "unexpected: {err}"
        );
    })
    .await;
}

#[tokio::test]
async fn manager_create_times_out_waiting_for_cloud_init() {
    let harness = MockHarness::new();
    harness.write_bin("cargo", CARGO_OK);
    harness.write_bin("virsh", VIRSH_WITH_IP);
    harness.write_bin("ssh", SSH_FAIL);
    let path = harness.path_env();
    let manager = harness.manager(ValidationConfig {
        cloud_init_timeout: Duration::from_millis(50),
        ssh_timeout: Duration::from_secs(5),
        ssh_retry_interval: Duration::from_millis(5),
        ssh_max_retries: 10_000,
    });

    temp_env::async_with_vars([("PATH", Some(path.as_str()))], async move {
        let err = manager
            .create("fed-test")
            .await
            .expect_err("cloud-init timeout");
        assert!(
            err.to_string().contains("Timeout waiting for VM"),
            "unexpected: {err}"
        );
    })
    .await;
}

#[tokio::test]
async fn manager_create_discovers_multiple_vm_ips() {
    let harness = MockHarness::new();
    harness.write_bin("cargo", CARGO_OK);
    harness.write_bin("virsh", VIRSH_MULTI_VM);
    harness.write_bin("ssh", SSH_OK);
    let path = harness.path_env();
    let manager = harness.manager(fast_validation_config());

    temp_env::async_with_vars([("PATH", Some(path.as_str()))], async move {
        manager
            .create("fed-test")
            .await
            .expect("multi-vm create should succeed");
    })
    .await;
}

#[test]
fn manager_start_errors_when_cargo_missing_from_path() {
    let harness = MockHarness::new();
    let manager = harness.manager(ValidationConfig::default());

    temp_env::with_var("PATH", Some("/usr/bin:/bin"), || {
        let err = manager.start("fed-test").unwrap_err();
        assert!(
            err.to_string()
                .contains("Failed to execute benchscale start"),
            "unexpected: {err}"
        );
    });
}

#[test]
fn manager_status_errors_when_cargo_missing_from_path() {
    let harness = MockHarness::new();
    let manager = harness.manager(ValidationConfig::default());

    temp_env::with_var("PATH", Some("/usr/bin:/bin"), || {
        let err = manager.status("fed-test").unwrap_err();
        assert!(
            err.to_string()
                .contains("Failed to execute benchscale status"),
            "unexpected: {err}"
        );
    });
}

#[test]
fn manager_create_errors_when_cargo_missing_from_path() {
    let harness = MockHarness::new();
    let path = "/usr/bin:/bin".to_string();
    let manager = harness.manager(fast_validation_config());

    temp_env::with_var("PATH", Some(path.as_str()), || {
        let err = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("runtime")
            .block_on(manager.create("fed-test"))
            .unwrap_err();
        assert!(
            err.to_string()
                .contains("Failed to execute benchscale create"),
            "unexpected: {err}"
        );
    });
}
