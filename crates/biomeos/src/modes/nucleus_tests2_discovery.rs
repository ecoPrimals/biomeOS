// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::*;
use std::path::PathBuf;

#[test]
fn test_discover_binaries_empty_path_env() {
    let map =
        discover_binaries_with(&["beardog"], None, &[], None).expect("discover should not panic");
    // May or may not find beardog depending on relative paths (plasmidBin, target/release)
    // Just verify it doesn't panic with empty PATH
    let _ = map;
}

#[test]
fn test_discover_binaries_with_missing_primal_returns_partial_ok() {
    let map = discover_binaries_with(&["missing_primal_abc"], None, &[], None).expect("ok");
    assert!(map.is_empty());
}

#[tokio::test]
async fn test_discover_binaries_with_livespore_usb_arch_primals() {
    let temp = tempfile::tempdir().expect("tempdir");
    let arch = std::env::consts::ARCH;
    let primal_dir = temp.path().join("livespore-usb").join(arch).join("primals");
    std::fs::create_dir_all(&primal_dir).expect("mkdir");
    let name = "biomeos_unique_primal_livespore_usb_xyz";
    let binary_path = primal_dir.join(name);
    std::fs::write(&binary_path, b"#! /bin/sh\nexit 0\n").expect("write bin");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&binary_path).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&binary_path, perms).unwrap();
    }
    let map = discover_binaries_with(&[name], None, &[], Some(temp.path())).expect("discover");
    assert!(
        map.contains_key(name),
        "expected {name} under livespore-usb/{arch}/primals, got {map:?}"
    );
}

#[tokio::test]
async fn test_discover_binaries_with_livespore_usb_primals_flat_layout() {
    let _guard = crate::CWD_TEST_LOCK.lock().await;
    let temp = tempfile::tempdir().expect("tempdir");
    let primal_dir = temp.path().join("livespore-usb").join("primals");
    std::fs::create_dir_all(&primal_dir).expect("mkdir");
    let name = "biomeos_unique_primal_livespore_flat_xyz";
    let binary_path = primal_dir.join(name);
    std::fs::write(&binary_path, b"#! /bin/sh\nexit 0\n").expect("write bin");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&binary_path).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&binary_path, perms).unwrap();
    }
    let old = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp.path()).unwrap();
    let map = discover_binaries_with(&[name], None, &[], None);
    std::env::set_current_dir(old).unwrap();
    let map = map.expect("discover");
    assert!(
        map.contains_key(name),
        "expected {name} under livespore-usb/primals, got {map:?}"
    );
}

#[tokio::test]
async fn test_discover_binaries_with_plasmidbin_optimized_arch() {
    let _guard = crate::CWD_TEST_LOCK.lock().await;
    let temp = tempfile::tempdir().expect("tempdir");
    let arch = std::env::consts::ARCH;
    let primal_dir = temp.path().join("plasmidBin").join("optimized").join(arch);
    std::fs::create_dir_all(&primal_dir).expect("mkdir");
    let name = "biomeos_unique_primal_plasmid_opt_xyz";
    let binary_path = primal_dir.join(name);
    std::fs::write(&binary_path, b"#! /bin/sh\nexit 0\n").expect("write bin");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&binary_path).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&binary_path, perms).unwrap();
    }
    let old = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp.path()).unwrap();
    let map = discover_binaries_with(&[name], None, &[], None);
    std::env::set_current_dir(old).unwrap();
    let map = map.expect("discover");
    assert!(
        map.contains_key(name),
        "expected {name} under plasmidBin/optimized/{arch}, got {map:?}"
    );
}

#[tokio::test]
async fn test_discover_binaries_with_plasmidbin_relative() {
    let _guard = crate::CWD_TEST_LOCK.lock().await;
    let temp = tempfile::tempdir().expect("tempdir");
    let bin_dir = temp.path().join("plasmidBin");
    std::fs::create_dir_all(&bin_dir).expect("mkdir");
    let name = "biomeos_unique_primal_plasmidbin_rel_xyz";
    let binary_path = bin_dir.join(name);
    std::fs::write(&binary_path, b"#! /bin/sh\nexit 0\n").expect("write bin");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&binary_path).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&binary_path, perms).unwrap();
    }
    let old = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp.path()).unwrap();
    let map = discover_binaries_with(&[name], None, &[], None);
    std::env::set_current_dir(old).unwrap();
    let map = map.expect("discover");
    assert!(
        map.contains_key(name),
        "expected {name} under plasmidBin, got {map:?}"
    );
}

#[tokio::test]
async fn test_discover_binaries_with_relative_plasmidbin_primals() {
    let _guard = crate::CWD_TEST_LOCK.lock().await;
    let temp = tempfile::tempdir().expect("tempdir");
    let nested = temp.path().join("work").join("a").join("b");
    std::fs::create_dir_all(&nested).expect("mkdir nested");
    // From cwd `work/a/b`, `../../plasmidBin/primals` resolves to `work/plasmidBin/primals`.
    let plasmid_primals = temp.path().join("work").join("plasmidBin").join("primals");
    std::fs::create_dir_all(&plasmid_primals).expect("mkdir plasmidBin/primals");
    let name = "biomeos_unique_primal_rel_plasmid_xyz";
    let binary_path = plasmid_primals.join(name);
    std::fs::write(&binary_path, b"#! /bin/sh\nexit 0\n").expect("write bin");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&binary_path).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&binary_path, perms).unwrap();
    }
    let old = std::env::current_dir().unwrap();
    std::env::set_current_dir(&nested).unwrap();
    let map = discover_binaries_with(&[name], None, &[], None);
    std::env::set_current_dir(old).unwrap();
    let map = map.expect("discover");
    assert!(
        map.contains_key(name),
        "expected {name} via ../../plasmidBin/primals, got {map:?}"
    );
}

#[tokio::test]
async fn test_discover_binaries_with_plasmidbin_direct_binary() {
    let _guard = crate::CWD_TEST_LOCK.lock().await;
    let temp = tempfile::tempdir().expect("tempdir");
    let plasmid = temp.path().join("plasmidBin");
    std::fs::create_dir_all(&plasmid).expect("mkdir plasmidBin");
    let name = "biomeos_unique_primal_plasmidbin_root_xyz";
    let binary_path = plasmid.join(name);
    std::fs::write(&binary_path, b"#! /bin/sh\nexit 0\n").expect("write bin");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&binary_path).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&binary_path, perms).unwrap();
    }
    let old = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp.path()).unwrap();
    let map = discover_binaries_with(&[name], None, &[], None);
    std::env::set_current_dir(old).unwrap();
    let map = map.expect("discover");
    assert!(
        map.contains_key(name),
        "expected {name} under plasmidBin/, got {map:?}"
    );
}

#[test]
fn test_discover_search_path_with_cwd() {
    let cwd = std::path::Path::new("/home/user/project");
    let rel = PathBuf::from("plasmidBin");
    let result = discover_search_path(rel, Some(cwd));
    assert_eq!(result, PathBuf::from("/home/user/project/plasmidBin"));
}

#[test]
fn test_discover_search_path_without_cwd() {
    let rel = PathBuf::from("plasmidBin");
    let result = discover_search_path(rel, None);
    assert_eq!(result, PathBuf::from("plasmidBin"));
}

#[test]
fn test_discover_binaries_with_cwd_finds_in_plasmidbin() {
    let temp = tempfile::tempdir().expect("tempdir");
    let bin_dir = temp.path().join("plasmidBin");
    std::fs::create_dir_all(&bin_dir).expect("create plasmidBin");
    let name = "biomeos_cwd_test_binary";
    let binary = bin_dir.join(name);
    std::fs::write(&binary, b"#!/bin/sh\nexit 0").expect("write");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&binary).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&binary, perms).unwrap();
    }
    let map = discover_binaries_with(&[name], None, &[], Some(temp.path())).expect("discover");
    assert!(
        map.contains_key(name),
        "should find {name} in plasmidBin with cwd, got {map:?}"
    );
}

#[test]
fn test_discover_depot_takes_priority_over_livespore_usb() {
    let temp = tempfile::tempdir().expect("tempdir");
    let name = "biomeos_depot_priority_test_primal";

    // Create binary in livespore-usb/primals (low priority)
    let usb_dir = temp.path().join("livespore-usb/primals");
    std::fs::create_dir_all(&usb_dir).expect("mkdir livespore-usb/primals");
    let usb_binary = usb_dir.join(name);
    std::fs::write(&usb_binary, b"#!/bin/sh\necho stale").expect("write usb bin");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&usb_binary).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&usb_binary, perms).unwrap();
    }

    // Create binary in depot (high priority)
    let depot_dir = temp.path().join("depot/primals");
    std::fs::create_dir_all(&depot_dir).expect("mkdir depot/primals");
    let depot_binary = depot_dir.join(name);
    std::fs::write(&depot_binary, b"#!/bin/sh\necho depot").expect("write depot bin");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&depot_binary).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&depot_binary, perms).unwrap();
    }

    let depot_path = temp.path().join("depot");
    let map = discover_binaries_with(&[name], Some(depot_path.as_path()), &[], Some(temp.path()))
        .expect("discover");

    assert!(map.contains_key(name), "primal should be found");
    let resolved = &map[name];
    assert!(
        resolved.starts_with(&depot_dir),
        "depot should win over livespore-usb: resolved={}, expected prefix={}",
        resolved.display(),
        depot_dir.display()
    );
}
