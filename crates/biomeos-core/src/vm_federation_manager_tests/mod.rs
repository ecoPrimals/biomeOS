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

mod create;
mod lifecycle;

pub(super) const CARGO_OK: &str = "#!/bin/sh\nexit 0\n";
pub(super) const CARGO_FAIL: &str = "#!/bin/sh\necho benchscale failed >&2\nexit 1\n";
pub(super) const CARGO_STATUS: &str = "#!/bin/sh\necho federation running\nexit 0\n";

pub(super) const VIRSH_WITH_IP: &str = r#"#!/bin/sh
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

pub(super) const VIRSH_NO_192_168: &str = r#"#!/bin/sh
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

pub(super) const VIRSH_LIST_FAIL: &str = "#!/bin/sh\nexit 1\n";

pub(super) const SSH_OK: &str = "#!/bin/sh\necho SSH ready\nexit 0\n";
pub(super) const SSH_FAIL: &str = "#!/bin/sh\nexit 1\n";
pub(super) const SSH_OK_UNTIL_VALIDATE: &str = r#"#!/bin/sh
for arg in "$@"; do
  case "$arg" in
    *hostname*) exit 1 ;;
  esac
done
exit 0
"#;

pub(super) const VIRSH_MULTI_VM: &str = r#"#!/bin/sh
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

pub(super) struct MockHarness {
    root: tempfile::TempDir,
    benchscale_root: PathBuf,
    topology_path: PathBuf,
    bin_dir: PathBuf,
}

impl MockHarness {
    pub(super) fn new() -> Self {
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

    pub(super) fn write_bin(&self, name: &str, body: &str) {
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

    pub(super) fn path_env(&self) -> String {
        format!(
            "{}:{}",
            self.bin_dir.display(),
            std::env::var("PATH").unwrap_or_else(|_| "/usr/bin:/bin".to_string())
        )
    }

    pub(super) fn manager(&self, validation_config: ValidationConfig) -> VmFederationManager {
        VmFederationManager::with_paths_for_test(
            self.benchscale_root.clone(),
            self.topology_path.clone(),
            validation_config,
        )
    }
}

pub(super) fn fast_validation_config() -> ValidationConfig {
    ValidationConfig {
        cloud_init_timeout: Duration::from_secs(30),
        ssh_timeout: Duration::from_secs(5),
        ssh_retry_interval: Duration::from_millis(1),
        ssh_max_retries: 5,
    }
}

pub(super) fn non_utf8_topology_path(root: &Path) -> PathBuf {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;
    root.join(OsString::from_vec(vec![
        0x74, 0x6f, 0x70, 0xFF, 0x2e, 0x79, 0x61, 0x6d, 0x6c,
    ]))
}
