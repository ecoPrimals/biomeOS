// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Root filesystem configuration

use std::path::PathBuf;

/// Configuration for building a BiomeOS root filesystem
#[derive(Debug, Clone)]
pub struct RootFsConfig {
    /// Size of the root filesystem (e.g., "8G")
    pub size: String,

    /// Output path for the root filesystem image
    pub output: PathBuf,

    /// Directory containing primal binaries
    pub primals_dir: Option<PathBuf>,

    /// Directory containing systemd service files
    pub services_dir: Option<PathBuf>,

    /// Mount point for building (temporary, auto-generated if None)
    pub mount_point: Option<PathBuf>,

    /// Filesystem type (default: ext4)
    pub fs_type: String,

    /// DNS servers (discovered from system if None)
    pub dns_servers: Option<Vec<String>>,

    /// NBD device to use (auto-detect if None)
    pub nbd_device: Option<String>,

    /// Hostname for the system (default: "biomeos")
    pub hostname: String,
}

impl Default for RootFsConfig {
    fn default() -> Self {
        Self {
            size: "8G".to_string(),
            output: PathBuf::from("biomeos-root.qcow2"),
            primals_dir: None,
            services_dir: None,
            mount_point: None,
            fs_type: "ext4".to_string(),
            dns_servers: None,
            nbd_device: None,
            hostname: "biomeos".to_string(),
        }
    }
}
