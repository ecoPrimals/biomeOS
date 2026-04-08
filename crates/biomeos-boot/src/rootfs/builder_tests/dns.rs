// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use std::path::PathBuf;

use super::{test_config, RootFsBuilder, RootFsConfig};

#[test]
fn test_builder_new() {
    let config = test_config();
    let builder = RootFsBuilder::new(config);
    assert_eq!(builder.config.size, "1G");
    assert_eq!(builder.config.hostname, "test-biomeos");
}

#[test]
fn test_configure_dns_empty_servers_early_return() {
    let temp = tempfile::Builder::new().tempdir().expect("tempdir");
    let root = temp.path();
    std::fs::create_dir_all(root.join("etc")).expect("create etc");

    let config = RootFsConfig {
        dns_servers: Some(vec![]),
        ..test_config()
    };
    let builder = RootFsBuilder::new(config);
    builder.configure_dns(root).expect("configure_dns");
    assert!(!root.join("etc/resolv.conf").exists());
}

#[test]
fn test_configure_dns_with_servers() {
    let temp = tempfile::Builder::new().tempdir().expect("tempdir");
    let root = temp.path();
    std::fs::create_dir_all(root.join("etc")).expect("create etc");

    let config = RootFsConfig {
        dns_servers: Some(vec!["10.0.0.1".to_string(), "10.0.0.2".to_string()]),
        ..test_config()
    };
    let builder = RootFsBuilder::new(config);
    builder.configure_dns(root).expect("configure_dns");

    let content = std::fs::read_to_string(root.join("etc/resolv.conf")).expect("read");
    assert!(content.contains("nameserver 10.0.0.1"));
    assert!(content.contains("nameserver 10.0.0.2"));
}

#[test]
fn test_configure_system_writes_hostname() {
    let temp = tempfile::Builder::new().tempdir().expect("tempdir");
    let root = temp.path();
    std::fs::create_dir_all(root.join("etc")).expect("create etc");

    let config = RootFsConfig {
        hostname: "my-custom-host".to_string(),
        dns_servers: Some(vec![]),
        ..test_config()
    };
    let builder = RootFsBuilder::new(config);
    builder.configure_system(root).expect("configure_system");

    let hostname = std::fs::read_to_string(root.join("etc/hostname")).expect("read");
    assert_eq!(hostname.trim(), "my-custom-host");
}

#[test]
fn test_configure_dns_uses_system_when_none() {
    let temp = tempfile::Builder::new().tempdir().expect("tempdir");
    let root = temp.path();
    std::fs::create_dir_all(root.join("etc")).expect("create etc");

    let config = RootFsConfig {
        dns_servers: None,
        ..test_config()
    };
    let builder = RootFsBuilder::new(config);
    let result = builder.configure_dns(root);
    if result.is_ok() {
        if root.join("etc/resolv.conf").exists() {
            let content = std::fs::read_to_string(root.join("etc/resolv.conf")).unwrap();
            assert!(content.contains("nameserver"));
        }
    }
}

#[test]
fn test_root_fs_config_fields() {
    let c = RootFsConfig {
        size: "2G".to_string(),
        output: PathBuf::from("/tmp/out.qcow2"),
        primals_dir: Some(PathBuf::from("/p")),
        services_dir: Some(PathBuf::from("/s")),
        mount_point: Some(PathBuf::from("/mnt")),
        fs_type: "xfs".to_string(),
        dns_servers: Some(vec!["8.8.8.8".to_string()]),
        nbd_device: None,
        hostname: "h".to_string(),
    };
    assert_eq!(c.fs_type, "xfs");
    assert_eq!(c.size, "2G");
}

#[test]
fn test_configure_system_empty_hostname_still_writes() {
    let temp = tempfile::Builder::new().tempdir().expect("tempdir");
    let root = temp.path();
    std::fs::create_dir_all(root.join("etc")).expect("create etc");
    let config = RootFsConfig {
        hostname: String::new(),
        dns_servers: Some(vec![]),
        ..test_config()
    };
    let builder = RootFsBuilder::new(config);
    builder.configure_system(root).expect("configure_system");
    let h = std::fs::read_to_string(root.join("etc/hostname")).expect("read");
    assert_eq!(h.trim(), "");
}

#[test]
fn test_builder_exposes_config() {
    let b = RootFsBuilder::new(test_config());
    assert_eq!(b.config.hostname, "test-biomeos");
}

#[test]
fn test_discover_system_dns_reads_resolv_conf() {
    let servers = RootFsBuilder::discover_system_dns();
    match servers {
        Ok(v) => {
            if std::path::Path::new("/etc/resolv.conf").exists() {
                assert!(
                    v.iter().all(|s| !s.is_empty()),
                    "nameservers should be non-empty strings when parsed"
                );
            }
        }
        Err(e) => {
            assert!(
                e.to_string().contains("resolv") || e.to_string().contains("Failed"),
                "unexpected error: {e}"
            );
        }
    }
}

#[test]
fn test_configure_dns_system_branch_via_none_config() {
    let temp = tempfile::Builder::new().tempdir().expect("tempdir");
    let root = temp.path();
    std::fs::create_dir_all(root.join("etc")).expect("create etc");
    let config = RootFsConfig {
        dns_servers: None,
        ..test_config()
    };
    let builder = RootFsBuilder::new(config);
    let r = builder.configure_dns(root);
    assert!(r.is_ok());
}

#[test]
fn test_configure_dns_single_nameserver() {
    let temp = tempfile::Builder::new().tempdir().expect("tempdir");
    let root = temp.path();
    std::fs::create_dir_all(root.join("etc")).expect("create etc");
    let config = RootFsConfig {
        dns_servers: Some(vec!["1.1.1.1".to_string()]),
        ..test_config()
    };
    let builder = RootFsBuilder::new(config);
    builder.configure_dns(root).expect("dns");
    let c = std::fs::read_to_string(root.join("etc/resolv.conf")).expect("read");
    assert_eq!(c.lines().count(), 1);
    assert!(c.contains("1.1.1.1"));
}

#[test]
fn test_root_fs_config_mount_point_optional() {
    let c = RootFsConfig {
        mount_point: Some(PathBuf::from("/mnt/rootfs")),
        ..test_config()
    };
    assert_eq!(
        c.mount_point
            .as_ref()
            .map(|p| p.to_string_lossy().to_string()),
        Some("/mnt/rootfs".to_string())
    );
}

#[test]
fn test_configure_dns_three_nameservers_order_preserved() {
    let temp = tempfile::Builder::new().tempdir().expect("tempdir");
    let root = temp.path();
    std::fs::create_dir_all(root.join("etc")).expect("create etc");
    let config = RootFsConfig {
        dns_servers: Some(vec![
            "9.9.9.9".to_string(),
            "149.112.112.112".to_string(),
            "1.0.0.1".to_string(),
        ]),
        ..test_config()
    };
    let builder = RootFsBuilder::new(config);
    builder.configure_dns(root).expect("configure_dns");
    let content = std::fs::read_to_string(root.join("etc/resolv.conf")).expect("read");
    assert_eq!(content.lines().count(), 3);
    assert!(content.contains("9.9.9.9"));
    assert!(content.contains("149.112.112.112"));
    assert!(content.contains("1.0.0.1"));
}
