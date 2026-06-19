// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Integration-style tests for [`crate::vm_federation::VmFederationManager`] using mock
//! `PATH` executables and isolated temp directories (no libvirt/benchScale required).

#![expect(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::vm_federation::{ValidationConfig, VmFederationManager};

const CARGO_OK: &str = "#!/bin/sh\nexit 0\n";
const CARGO_FAIL: &str = "#!/bin/sh\necho benchscale failed >&2\nexit 1\n";
const CARGO_STATUS: &str = "#!/bin/sh\necho federation running\nexit 0\n";

const VIRSH_WITH_IP: &str = r#"#!/bin/sh
if [ "$1" = "list" ] && [ "$2" = "--all" ]; then
  echo " 1     fed-test-node1    running"
  exit 0
fi
if [ "$1" = "domifaddr" ]; then
  echo " Name       MAC address          Protocol     Address"
  echo " vnet0      xx:xx    ipv4         192.168.122.50/24"
  exit 0
fi
exit 1
"#;

const VIRSH_NO_192_168: &str = r#"#!/bin/sh
if [ "$1" = "list" ] && [ "$2" = "--all" ]; then
  echo " 1     fed-test-node1    running"
  exit 0
fi
if [ "$1" = "domifaddr" ]; then
  echo "ipv4         10.0.0.1/24"
  exit 0
fi
exit 1
"#;

const VIRSH_LIST_FAIL: &str = "#!/bin/sh\nexit 1\n";

const SSH_OK: &str = "#!/bin/sh\necho SSH ready\nexit 0\n";
const SSH_FAIL: &str = "#!/bin/sh\nexit 1\n";
const SSH_OK_UNTIL_VALIDATE: &str = r#"#!/bin/sh
for arg in "$@"; do
  case "$arg" in
    *hostname*) exit 1 ;;
  esac
done
exit 0
"#;

const VIRSH_MULTI_VM: &str = r#"#!/bin/sh
if [ "$1" = "list" ] && [ "$2" = "--all" ]; then
  echo " 1     fed-test-node1    running"
  echo " 2     fed-test-node2    running"
  exit 0
fi
if [ "$1" = "domifaddr" ]; then
  case "$2" in
    fed-test-node1)
      echo "ipv4         192.168.122.10/24"
      ;;
    fed-test-node2)
      echo "ipv4         192.168.122.11/24"
      ;;
  esac
  exit 0
fi
exit 1
"#;

struct MockHarness {
    root: tempfile::TempDir,
    benchscale_root: PathBuf,
    topology_path: PathBuf,
    bin_dir: PathBuf,
}

impl MockHarness {
    fn new() -> Self {
        let root = tempfile::tempdir().expect("tempdir");
        let benchscale_root = root.path().join("benchscale");
        std::fs::create_dir_all(&benchscale_root).expect("benchscale dir");
        let topology_path = root.path().join("vm-federation.yaml");
        std::fs::write(&topology_path, "name: test-topology\n").expect("topology");
        let bin_dir = root.path().join("bin");
        std::fs::create_dir_all(&bin_dir).expect("bin dir");
        Self {
            root,
            benchscale_root,
            topology_path,
            bin_dir,
        }
    }

    fn write_bin(&self, name: &str, body: &str) {
        let path = self.bin_dir.join(name);
        let mut file = std::fs::File::create(&path).expect("create mock binary");
        file.write_all(body.as_bytes()).expect("write mock binary");
        file.sync_all().expect("sync mock binary");
        drop(file);
        let mut perms = std::fs::metadata(&path)
            .expect("mock binary metadata")
            .permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&path, perms).expect("chmod mock binary");
    }

    fn path_env(&self) -> String {
        format!(
            "{}:{}",
            self.bin_dir.display(),
            std::env::var("PATH").unwrap_or_else(|_| "/usr/bin:/bin".to_string())
        )
    }

    fn manager(&self, validation_config: ValidationConfig) -> VmFederationManager {
        VmFederationManager::with_paths_for_test(
            self.benchscale_root.clone(),
            self.topology_path.clone(),
            validation_config,
        )
    }
}

fn fast_validation_config() -> ValidationConfig {
    ValidationConfig {
        cloud_init_timeout: Duration::from_secs(30),
        ssh_timeout: Duration::from_secs(5),
        ssh_retry_interval: Duration::from_millis(1),
        ssh_max_retries: 5,
    }
}

fn non_utf8_topology_path(root: &Path) -> PathBuf {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;
    root.join(OsString::from_vec(vec![
        0x74, 0x6f, 0x70, 0xFF, 0x2e, 0x79, 0x61, 0x6d, 0x6c,
    ]))
}

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
