// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

#![expect(clippy::unwrap_used, reason = "test assertions")]
#![expect(clippy::expect_used, reason = "test assertions")]

mod dns;
mod install;

pub(crate) use crate::rootfs::builder::RootFsBuilder;
pub(crate) use crate::rootfs::config::RootFsConfig;

pub(crate) fn test_config() -> RootFsConfig {
    use std::path::PathBuf;
    RootFsConfig {
        size: "1G".to_string(),
        output: PathBuf::from("/tmp/test-rootfs.qcow2"),
        primals_dir: None,
        services_dir: None,
        mount_point: None,
        fs_type: "ext4".to_string(),
        dns_servers: Some(vec![]),
        nbd_device: None,
        hostname: "test-biomeos".to_string(),
    }
}
