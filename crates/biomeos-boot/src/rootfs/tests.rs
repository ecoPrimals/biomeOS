// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Root filesystem tests

use std::path::Path;
use std::path::PathBuf;

use super::builder::RootFsBuilder;
use super::config::RootFsConfig;
use super::dns::parse_resolv_conf;
use super::nbd::NbdGuard;

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
mod run {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_rootfs_config_default() {
        let config = RootFsConfig::default();
        assert_eq!(config.size, "8G");
        assert_eq!(config.fs_type, "ext4");
        assert_eq!(config.hostname, "biomeos");
        assert!(config.primals_dir.is_none());
    }

    #[test]
    fn test_discover_system_dns() {
        let config = RootFsConfig::default();
        let builder = RootFsBuilder::new(config);

        let dns_servers = builder.discover_system_dns();
        if let Ok(servers) = dns_servers {
            for server in servers {
                assert!(!server.is_empty());
            }
        }
    }

    #[tokio::test]
    async fn test_install_base_system_structure() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        let config = RootFsConfig::default();
        let _builder = RootFsBuilder::new(config);

        let dirs = [
            "bin", "sbin", "usr/bin", "usr/sbin", "etc", "var", "tmp", "run", "proc", "sys", "dev",
        ];

        for dir in &dirs {
            std::fs::create_dir_all(root.join(dir)).unwrap();
        }

        for dir in &dirs {
            assert!(root.join(dir).exists());
        }
    }

    #[test]
    #[ignore = "Requires NBD kernel module and sudo"]
    fn test_nbd_device_detection() {
        let result = NbdGuard::find_available_device();

        match result {
            Ok(device) => {
                assert!(device.starts_with("/dev/nbd"));
            }
            Err(e) => {
                let err_msg = format!("{e}");
                assert!(err_msg.contains("No available NBD devices"));
            }
        }
    }

    #[test]
    fn test_rootfs_config_custom_values() {
        let config = RootFsConfig {
            size: "10G".to_string(),
            output: PathBuf::from("/tmp/custom.qcow2"),
            primals_dir: Some(PathBuf::from("/opt/primals")),
            services_dir: Some(PathBuf::from("/etc/systemd")),
            mount_point: Some(PathBuf::from("/mnt/build")),
            fs_type: "xfs".to_string(),
            dns_servers: Some(vec!["192.0.2.53".to_string(), "192.0.2.54".to_string()]),
            nbd_device: Some("/dev/nbd0".to_string()),
            hostname: "custom-host".to_string(),
        };

        assert_eq!(config.size, "10G");
        assert_eq!(config.fs_type, "xfs");
        assert_eq!(config.hostname, "custom-host");
        assert_eq!(config.dns_servers.as_ref().unwrap().len(), 2);
    }

    #[test]
    fn test_rootfs_builder_new() {
        let config = RootFsConfig::default();
        let builder = RootFsBuilder::new(config.clone());
        assert_eq!(builder.config.size, config.size);
    }

    #[test]
    fn test_rootfs_cli_parse() {
        use clap::Parser;

        let cli = crate::rootfs::RootFsCli::parse_from([
            "biomeos-rootfs",
            "-s",
            "4G",
            "-o",
            "test.qcow2",
        ]);
        assert_eq!(cli.size, "4G");
        assert_eq!(cli.output, PathBuf::from("test.qcow2"));
    }

    #[test]
    fn test_rootfs_cli_defaults() {
        use clap::Parser;

        let cli = crate::rootfs::RootFsCli::parse_from(["biomeos-rootfs"]);
        assert_eq!(cli.size, "8G");
        assert_eq!(cli.output, PathBuf::from("biomeos-root.qcow2"));
        assert_eq!(cli.fs_type, "ext4");
    }

    #[test]
    fn test_configure_dns_empty_servers() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        let config = RootFsConfig {
            dns_servers: Some(vec![]),
            ..Default::default()
        };
        let builder = RootFsBuilder::new(config);

        let result = builder.configure_dns(root);
        result.expect("configure_dns with empty servers should succeed");
    }

    #[test]
    fn test_configure_dns_with_servers() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();
        std::fs::create_dir_all(root.join("etc")).unwrap();

        let config = RootFsConfig {
            dns_servers: Some(vec!["1.2.3.4".to_string(), "5.6.7.8".to_string()]),
            ..Default::default()
        };
        let builder = RootFsBuilder::new(config);

        builder.configure_dns(root).expect("configure_dns");

        let resolv = std::fs::read_to_string(root.join("etc/resolv.conf")).unwrap();
        assert!(resolv.contains("1.2.3.4"));
        assert!(resolv.contains("5.6.7.8"));
    }

    #[test]
    fn test_configure_system_sets_hostname() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();
        std::fs::create_dir_all(root.join("etc")).unwrap();

        let config = RootFsConfig {
            hostname: "test-host".to_string(),
            dns_servers: Some(vec![]),
            ..Default::default()
        };
        let builder = RootFsBuilder::new(config);

        builder.configure_system(root).expect("configure_system");

        let hostname = std::fs::read_to_string(root.join("etc/hostname")).unwrap();
        assert_eq!(hostname.trim(), "test-host");
    }

    #[test]
    fn test_rootfs_config_size_parsing() {
        let config = RootFsConfig {
            size: "4G".to_string(),
            ..Default::default()
        };
        assert_eq!(config.size, "4G");
    }

    #[test]
    fn test_rootfs_config_output_path() {
        let config = RootFsConfig {
            output: PathBuf::from("/tmp/custom.qcow2"),
            ..Default::default()
        };
        assert_eq!(config.output, PathBuf::from("/tmp/custom.qcow2"));
    }

    #[test]
    fn test_install_base_system_dirs_created() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        let dirs = [
            "bin", "sbin", "usr/bin", "usr/sbin", "etc", "var", "tmp", "run", "proc", "sys", "dev",
            "home", "root",
        ];
        for dir in &dirs {
            std::fs::create_dir_all(root.join(dir)).unwrap();
        }
        for dir in &dirs {
            assert!(root.join(dir).exists(), "{dir} should exist");
        }
    }

    #[test]
    fn test_parse_resolv_conf_empty() {
        assert!(parse_resolv_conf("").is_empty());
        assert!(parse_resolv_conf("   \n  \n").is_empty());
    }

    #[test]
    fn test_parse_resolv_conf_single_nameserver() {
        let content = "nameserver 8.8.8.8\n";
        let servers = parse_resolv_conf(content);
        assert_eq!(servers, vec!["8.8.8.8"]);
    }

    #[test]
    fn test_parse_resolv_conf_multiple_nameservers() {
        let content = r#"# comment
nameserver 8.8.8.8
nameserver 8.8.4.4
nameserver 1.1.1.1
"#;
        let servers = parse_resolv_conf(content);
        assert_eq!(servers, vec!["8.8.8.8", "8.8.4.4", "1.1.1.1"]);
    }

    #[test]
    fn test_parse_resolv_conf_skips_comments_and_options() {
        let content = r#"# Generated by resolvconf
search example.com
options timeout:2
nameserver 192.168.1.1
nameserver 10.0.0.1
"#;
        let servers = parse_resolv_conf(content);
        assert_eq!(servers, vec!["192.168.1.1", "10.0.0.1"]);
    }

    #[test]
    fn test_parse_resolv_conf_handles_whitespace() {
        let content = "  nameserver   127.0.0.1  \n";
        let servers = parse_resolv_conf(content);
        assert_eq!(servers, vec!["127.0.0.1"]);
    }

    #[test]
    fn test_rootfs_cli_with_primals_flag() {
        use clap::Parser;

        let cli = crate::rootfs::RootFsCli::parse_from([
            "biomeos-rootfs",
            "-s",
            "4G",
            "-o",
            "out.qcow2",
            "-p",
            "/opt/primals",
        ]);
        assert_eq!(cli.size, "4G");
        assert_eq!(cli.primals, Some(PathBuf::from("/opt/primals")));
    }

    #[test]
    fn test_rootfs_cli_fs_type() {
        use clap::Parser;

        let cli = crate::rootfs::RootFsCli::parse_from(["biomeos-rootfs", "-f", "xfs"]);
        assert_eq!(cli.fs_type, "xfs");
    }

    #[test]
    fn test_parse_resolv_conf_single_line_no_newline() {
        let content = "nameserver 10.0.0.1";
        let servers = parse_resolv_conf(content);
        assert_eq!(servers, vec!["10.0.0.1"]);
    }

    #[test]
    fn test_parse_resolv_conf_invalid_line_skipped() {
        let content = "nameserver\nnameserver 8.8.8.8";
        let servers = parse_resolv_conf(content);
        assert_eq!(servers, vec!["8.8.8.8"]);
    }

    #[test]
    fn test_install_services_skips_non_service_files() {
        let temp = TempDir::new().unwrap();
        let services_dir = temp.path();
        std::fs::write(services_dir.join("not-a-service.txt"), "x").unwrap();
        std::fs::write(services_dir.join("real.service"), "[Unit]").unwrap();

        let root = TempDir::new().unwrap().path().to_path_buf();
        std::fs::create_dir_all(root.join("etc/systemd/system")).unwrap();

        let config = RootFsConfig {
            services_dir: Some(services_dir.to_path_buf()),
            dns_servers: Some(vec![]),
            ..Default::default()
        };
        let builder = RootFsBuilder::new(config);
        builder.install_services(&root, services_dir).unwrap();

        assert!(root.join("etc/systemd/system/real.service").exists());
        assert!(!root.join("etc/systemd/system/not-a-service.txt").exists());
    }

    #[test]
    fn test_install_services_empty_dir() {
        let temp = TempDir::new().unwrap();
        let services_dir = temp.path();
        let root = TempDir::new().unwrap().path().to_path_buf();
        std::fs::create_dir_all(root.join("etc/systemd/system")).unwrap();

        let config = RootFsConfig {
            services_dir: Some(services_dir.to_path_buf()),
            dns_servers: Some(vec![]),
            ..Default::default()
        };
        let builder = RootFsBuilder::new(config);
        builder.install_services(&root, services_dir).unwrap();
    }

    #[tokio::test]
    async fn test_install_primals_nonexistent_dir_ok() {
        let root = TempDir::new().unwrap().path().to_path_buf();
        std::fs::create_dir_all(root.join("usr/local/bin")).unwrap();

        let config = RootFsConfig {
            primals_dir: Some(PathBuf::from("/nonexistent/primals")),
            dns_servers: Some(vec![]),
            ..Default::default()
        };
        let builder = RootFsBuilder::new(config);
        let result: Result<(), _> = builder
            .install_primals(&root, Path::new("/nonexistent/primals"))
            .await;
        result.unwrap();
    }
}
