// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use std::path::{Path, PathBuf};

use super::{test_config, RootFsBuilder, RootFsConfig};

#[tokio::test]
async fn test_install_primals_with_files() {
    let temp = tempfile::Builder::new().tempdir().expect("tempdir");
    let root = temp.path();
    let primals_dir = temp.path().join("primals");
    std::fs::create_dir_all(&primals_dir).expect("create primals");
    std::fs::write(primals_dir.join("beardog"), b"#!/bin/sh\necho beardog").expect("write");
    std::fs::write(primals_dir.join("songbird"), b"#!/bin/sh\necho songbird").expect("write");

    let config = RootFsConfig {
        primals_dir: Some(primals_dir.clone()),
        dns_servers: Some(vec![]),
        ..test_config()
    };
    let builder = RootFsBuilder::new(config);
    builder
        .install_primals(root, &primals_dir)
        .expect("install_primals");

    let target = root.join("usr/local/bin");
    assert!(target.join("beardog").exists());
    assert!(target.join("songbird").exists());
}

#[tokio::test]
async fn test_install_primals_nonexistent_dir_returns_ok() {
    let temp = tempfile::Builder::new().tempdir().expect("tempdir");
    let root = temp.path();
    std::fs::create_dir_all(root.join("usr/local/bin")).expect("create target");

    let config = RootFsConfig {
        primals_dir: Some(PathBuf::from("/nonexistent/path")),
        dns_servers: Some(vec![]),
        ..test_config()
    };
    let builder = RootFsBuilder::new(config);
    builder
        .install_primals(root, Path::new("/nonexistent/path"))
        .expect("install_primals");
}

#[test]
fn test_install_services_nonexistent_dir_returns_ok() {
    let temp = tempfile::Builder::new().tempdir().expect("tempdir");
    let root = temp.path();
    std::fs::create_dir_all(root.join("etc/systemd/system")).expect("create systemd");

    let services_dir = Path::new("/nonexistent/services");
    RootFsBuilder::install_services(root, services_dir).expect("install_services");
}

#[test]
#[cfg(unix)]
fn test_install_services_with_service_file() {
    let temp = tempfile::Builder::new().tempdir().expect("tempdir");
    let services_dir = temp.path().join("services");
    std::fs::create_dir_all(&services_dir).expect("create services");
    std::fs::write(
        services_dir.join("test.service"),
        "[Unit]\nDescription=Test\n[Service]\nExecStart=/bin/true\n",
    )
    .expect("write");

    let root = temp.path().join("root");
    std::fs::create_dir_all(root.join("etc/systemd/system")).expect("create systemd");

    RootFsBuilder::install_services(&root, &services_dir).expect("install_services");

    assert!(root.join("etc/systemd/system/test.service").exists());
    assert!(root
        .join("etc/systemd/system/multi-user.target.wants/test.service")
        .exists());
}

#[test]
fn test_install_services_skips_non_service_files() {
    let temp = tempfile::Builder::new().tempdir().expect("tempdir");
    let services_dir = temp.path().join("services");
    std::fs::create_dir_all(&services_dir).expect("create services");
    std::fs::write(services_dir.join("not-a-service.txt"), "content").expect("write");
    std::fs::write(services_dir.join("config.conf"), "config").expect("write");

    let root = temp.path().join("root");
    std::fs::create_dir_all(root.join("etc/systemd/system")).expect("create systemd");

    RootFsBuilder::install_services(&root, &services_dir).expect("install_services");

    assert!(!root.join("etc/systemd/system/not-a-service.txt").exists());
    assert!(!root.join("etc/systemd/system/config.conf").exists());
}

#[test]
#[cfg(unix)]
fn test_install_services_with_existing_symlink() {
    let temp = tempfile::Builder::new().tempdir().expect("tempdir");
    let services_dir = temp.path().join("services");
    std::fs::create_dir_all(&services_dir).expect("create services");
    std::fs::write(
        services_dir.join("dup.service"),
        "[Unit]\nDescription=Dup\n[Service]\nExecStart=/bin/true\n",
    )
    .expect("write");

    let root = temp.path().join("root");
    let systemd_dir = root.join("etc/systemd/system");
    let wants_dir = systemd_dir.join("multi-user.target.wants");
    std::fs::create_dir_all(&wants_dir).expect("create");
    std::fs::write(systemd_dir.join("dup.service"), "old").expect("write");
    std::os::unix::fs::symlink(
        systemd_dir.join("dup.service"),
        wants_dir.join("dup.service"),
    )
    .expect("symlink");

    RootFsBuilder::install_services(&root, &services_dir).expect("install_services");
    assert!(wants_dir.join("dup.service").exists());
}

#[test]
#[cfg(unix)]
fn test_install_base_system_creates_dirs() {
    let temp = tempfile::Builder::new().tempdir().expect("tempdir");
    let root = temp.path();
    if RootFsBuilder::install_base_system(root).is_ok() {
        for dir in [
            "bin", "sbin", "usr/bin", "etc", "var/log", "proc", "sys", "dev", "tmp", "run",
        ] {
            assert!(root.join(dir).exists(), "{} should exist", dir);
        }
    }
}

#[test]
fn test_install_primals_skips_subdirectories() {
    let temp = tempfile::Builder::new().tempdir().expect("tempdir");
    let root = temp.path();
    let primals_dir = temp.path().join("primals");
    std::fs::create_dir_all(&primals_dir).expect("create primals");
    std::fs::create_dir_all(primals_dir.join("subdir")).expect("subdir");
    std::fs::write(primals_dir.join("exec-only"), b"x").expect("file");

    let config = RootFsConfig {
        primals_dir: Some(primals_dir.clone()),
        dns_servers: Some(vec![]),
        ..test_config()
    };
    let builder = RootFsBuilder::new(config);
    builder
        .install_primals(root, &primals_dir)
        .expect("install_primals");
    assert!(root.join("usr/local/bin/exec-only").exists());
}

#[test]
#[cfg(unix)]
fn test_install_services_replaces_stale_symlink_target() {
    let temp = tempfile::Builder::new().tempdir().expect("tempdir");
    let services_dir = temp.path().join("services");
    std::fs::create_dir_all(&services_dir).expect("create services");
    std::fs::write(
        services_dir.join("replace.service"),
        "[Unit]\nDescription=R\n[Service]\nExecStart=/bin/true\n",
    )
    .expect("write");

    let root = temp.path().join("root");
    let systemd_dir = root.join("etc/systemd/system");
    let wants = systemd_dir.join("multi-user.target.wants");
    std::fs::create_dir_all(&wants).expect("create");
    let unit = systemd_dir.join("replace.service");
    std::fs::write(&unit, "old content").expect("old unit");
    let wrong = wants.join("replace.service");
    std::os::unix::fs::symlink("/dev/null", &wrong).expect("wrong link");

    RootFsBuilder::install_services(&root, &services_dir).expect("install_services");
    assert!(wrong.exists());
    let new_content = std::fs::read_to_string(&unit).expect("read unit");
    assert!(new_content.contains("Description=R"));
}

#[test]
fn test_install_services_path_is_file_errors() {
    let temp = tempfile::Builder::new().tempdir().expect("tempdir");
    let not_a_dir = temp.path().join("file-not-dir");
    std::fs::write(&not_a_dir, b"x").expect("write");
    let root = temp.path().join("root");
    std::fs::create_dir_all(&root).expect("root");
    let result = RootFsBuilder::install_services(&root, &not_a_dir);
    assert!(result.is_err());
}
