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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rootfs_config_default() {
        let config = RootFsConfig::default();
        assert_eq!(config.size, "8G");
        assert_eq!(config.output, PathBuf::from("biomeos-root.qcow2"));
        assert!(config.primals_dir.is_none());
        assert!(config.services_dir.is_none());
        assert!(config.mount_point.is_none());
        assert_eq!(config.fs_type, "ext4");
        assert!(config.dns_servers.is_none());
        assert!(config.nbd_device.is_none());
        assert_eq!(config.hostname, "biomeos");
    }

    #[test]
    fn test_rootfs_config_custom() {
        let config = RootFsConfig {
            size: "16G".to_string(),
            output: PathBuf::from("/tmp/custom.qcow2"),
            primals_dir: Some(PathBuf::from("/opt/primals")),
            services_dir: Some(PathBuf::from("/etc/systemd")),
            mount_point: Some(PathBuf::from("/mnt/build")),
            fs_type: "xfs".to_string(),
            dns_servers: Some(vec!["192.0.2.53".to_string(), "198.51.100.53".to_string()]),
            nbd_device: Some("/dev/nbd1".to_string()),
            hostname: "custom-host".to_string(),
        };
        assert_eq!(config.size, "16G");
        assert_eq!(config.fs_type, "xfs");
        assert_eq!(config.dns_servers.as_ref().map(Vec::len).unwrap_or(0), 2);
        assert_eq!(config.nbd_device.as_deref(), Some("/dev/nbd1"));
        assert_eq!(config.hostname, "custom-host");
    }

    #[test]
    fn test_rootfs_config_clone() {
        let config = RootFsConfig {
            size: "4G".to_string(),
            output: PathBuf::from("test.qcow2"),
            primals_dir: None,
            services_dir: None,
            mount_point: None,
            fs_type: "ext4".to_string(),
            dns_servers: None,
            nbd_device: None,
            hostname: "clone-test".to_string(),
        };
        let cloned = config.clone();
        assert_eq!(config.size, cloned.size);
        assert_eq!(config.hostname, cloned.hostname);
    }

    #[test]
    fn test_rootfs_config_debug() {
        let config = RootFsConfig::default();
        let debug = format!("{config:?}");
        assert!(debug.contains("RootFsConfig"));
        assert!(debug.contains("biomeos"));
    }
}
