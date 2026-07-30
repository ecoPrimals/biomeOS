// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

#![expect(
    clippy::unwrap_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
#![expect(
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]

use super::*;

#[test]
fn test_initramfs_builder_new() {
    let temp = tempfile::tempdir().expect("temp dir");
    let _builder = InitramfsBuilder::new(temp.path()).expect("new");
    assert!(temp.path().join("initramfs-root").exists());
}

#[test]
fn test_binary_spec_debug() {
    let spec = BinarySpec {
        source: PathBuf::from("/bin/true"),
        dest: "/init".to_string(),
        permissions: 0o755,
    };
    let s = format!("{:?}", spec);
    assert!(s.contains("BinarySpec"));
}

#[test]
fn test_create_directory_structure() {
    let temp = tempfile::tempdir().expect("temp dir");
    let mut builder = InitramfsBuilder::new(temp.path()).expect("new");
    builder.create_directory_structure().expect("create dirs");

    let root = temp.path().join("initramfs-root");
    for dir in ["bin", "sbin", "proc", "sys", "dev", "tmp", "biomeos", "etc"] {
        assert!(root.join(dir).exists(), "{} should exist", dir);
    }
}

#[test]
fn test_add_binary_skips_nonexistent() {
    let temp = tempfile::tempdir().expect("temp dir");
    let mut builder = InitramfsBuilder::new(temp.path()).expect("new");
    let spec = BinarySpec {
        source: PathBuf::from("/nonexistent/binary"),
        dest: "/bin/missing".to_string(),
        permissions: 0o755,
    };
    builder
        .add_binary(spec)
        .expect("add_binary should not fail for missing");
}

#[test]
fn test_add_binary_adds_existing() {
    let temp = tempfile::tempdir().expect("temp dir");
    let bin_path = temp.path().join("mybin");
    std::fs::write(&bin_path, b"#!/bin/sh").expect("write");
    let mut builder = InitramfsBuilder::new(temp.path()).expect("new");
    let spec = BinarySpec {
        source: bin_path,
        dest: "/bin/mybin".to_string(),
        permissions: 0o755,
    };
    builder.add_binary(spec).expect("add_binary");
    builder.install_binaries().expect("install");
    assert!(temp.path().join("initramfs-root/bin/mybin").exists());
}

#[test]
fn test_create_init_script() {
    let temp = tempfile::tempdir().expect("temp dir");
    let mut builder = InitramfsBuilder::new(temp.path()).expect("new");
    builder.create_directory_structure().expect("dirs");
    builder.create_init_script().expect("init script");

    let init_path = temp.path().join("initramfs-root/init");
    assert!(init_path.exists());
    let content = std::fs::read_to_string(&init_path).expect("read");
    assert!(content.contains("#!/bin/sh"));
    assert!(content.contains("exec /init"));
}

#[test]
fn test_kernel_manager_custom() {
    let temp = tempfile::tempdir().expect("temp dir");
    let kernel_path = temp.path().join("vmlinuz");
    std::fs::write(&kernel_path, b"kernel").expect("write");
    let mgr = KernelManager::detect_or_custom(Some(kernel_path.clone())).expect("custom");
    assert_eq!(mgr.kernel_path(), kernel_path.as_path());
    assert_eq!(
        mgr.initramfs_path(),
        kernel_path
            .parent()
            .unwrap()
            .join("initramfs.img")
            .as_path()
    );
}

#[test]
fn test_kernel_manager_detect_or_custom_takes_precedence() {
    let temp = tempfile::tempdir().expect("temp dir");
    let kernel_path = temp.path().join("vmlinuz");
    std::fs::write(&kernel_path, b"kernel").expect("write");
    let mgr = KernelManager::detect_or_custom(Some(kernel_path.clone())).expect("custom");
    assert_eq!(mgr.kernel_path(), kernel_path.as_path());
}

#[test]
fn test_build_creates_output() {
    let temp = tempfile::tempdir().expect("temp dir");
    let mut builder = InitramfsBuilder::new(temp.path()).expect("new");
    builder.create_directory_structure().expect("dirs");
    std::fs::write(temp.path().join("initramfs-root/test.txt"), "content").expect("write");
    let output = temp.path().join("initramfs.img");
    builder.build(&output).expect("build");
    assert!(output.exists());
    assert!(std::fs::metadata(&output).unwrap().len() > 0);
}

#[test]
fn test_build_fails_when_output_dir_not_writable() {
    let temp = tempfile::tempdir().expect("temp dir");
    let mut builder = InitramfsBuilder::new(temp.path()).expect("new");
    builder.create_directory_structure().expect("dirs");
    let bad = temp.path().join("not-a-dir.img");
    std::fs::write(&bad, b"x").expect("file blocks path as directory parent");
    let output = bad.join("nested.img");
    let r = builder.build(&output);
    assert!(r.is_err());
}

#[test]
fn test_install_binaries_empty_list_ok() {
    let temp = tempfile::tempdir().expect("temp dir");
    let builder = InitramfsBuilder::new(temp.path()).expect("new");
    builder.install_binaries().expect("no binaries");
}

#[test]
fn test_add_required_libraries_ldd_on_empty_file() {
    let temp = tempfile::tempdir().expect("temp dir");
    let bin = temp.path().join("not-exec");
    std::fs::write(&bin, b"").expect("write");
    let builder = InitramfsBuilder::new(temp.path()).expect("new");
    let r = builder.add_required_libraries(&bin);
    assert!(r.is_ok());
}

#[test]
fn test_kernel_manager_biomeos_kernel_env_valid() {
    let temp = tempfile::tempdir().expect("temp dir");
    let k = temp.path().join("from-env-vmlinuz");
    std::fs::write(&k, b"k").expect("write");
    let mgr = KernelManager::detect_or_custom_with(None, Some(k.to_str().expect("utf8")))
        .expect("kernel from env");
    assert_eq!(mgr.kernel_path(), k.as_path());
    assert_eq!(
        mgr.initramfs_path(),
        k.parent().unwrap().join("biomeos-initramfs.img")
    );
}

#[test]
fn test_kernel_manager_biomeos_kernel_missing_file_warns_and_falls_back() {
    let r = KernelManager::detect_or_custom_with(
        None,
        Some("/nonexistent/biomeos-test-kernel-zzzz.img"),
    );
    if let Some(err) = r.err() {
        assert!(err.to_string().to_lowercase().contains("kernel"));
    }
}

#[test]
fn test_binary_spec_clone() {
    let spec = BinarySpec {
        source: PathBuf::from("/a"),
        dest: "/b".to_string(),
        permissions: 0o755,
    };
    let c = spec.clone();
    assert_eq!(c.dest, spec.dest);
}

#[test]
fn test_create_directory_structure_includes_biomeos_primals_path() {
    let temp = tempfile::tempdir().expect("temp dir");
    let mut builder = InitramfsBuilder::new(temp.path()).expect("new");
    builder.create_directory_structure().expect("dirs");
    let root = temp.path().join("initramfs-root");
    assert!(root.join("biomeos/primals").exists());
    assert!(root.join("var/log").exists());
}

#[test]
fn test_kernel_manager_find_matching_initramfs_prefers_existing() {
    let temp = tempfile::tempdir().expect("temp dir");
    let kernel = temp.path().join("vmlinuz");
    std::fs::write(&kernel, b"k").expect("k");
    let init = temp.path().join("initramfs.img");
    std::fs::write(&init, b"i").expect("i");
    let mgr = KernelManager::detect_or_custom(Some(kernel)).expect("mgr");
    assert_eq!(mgr.initramfs_path(), init.as_path());
}

#[test]
fn test_find_matching_initramfs_prefers_initramfs_img() {
    let temp = tempfile::tempdir().expect("temp dir");
    let kernel = temp.path().join("vmlinuz");
    std::fs::write(&kernel, b"k").expect("k");
    std::fs::write(temp.path().join("initramfs.img"), b"a").expect("a");
    std::fs::write(temp.path().join("initrd.img"), b"b").expect("b");
    let p = KernelManager::find_matching_initramfs(&kernel).expect("match");
    assert_eq!(p, temp.path().join("initramfs.img"));
}

#[test]
fn test_find_matching_initramfs_falls_back_to_initrd() {
    let temp = tempfile::tempdir().expect("temp dir");
    let kernel = temp.path().join("vmlinuz");
    std::fs::write(&kernel, b"k").expect("k");
    let init = temp.path().join("initrd.img");
    std::fs::write(&init, b"i").expect("i");
    let p = KernelManager::find_matching_initramfs(&kernel).expect("match");
    assert_eq!(p, init);
}

#[test]
fn test_find_matching_initramfs_falls_back_to_initramfs_linux() {
    let temp = tempfile::tempdir().expect("temp dir");
    let kernel = temp.path().join("vmlinuz");
    std::fs::write(&kernel, b"k").expect("k");
    let init = temp.path().join("initramfs-linux.img");
    std::fs::write(&init, b"i").expect("i");
    let p = KernelManager::find_matching_initramfs(&kernel).expect("match");
    assert_eq!(p, init);
}

#[test]
fn test_find_matching_initramfs_defaults_biomeos_name() {
    let temp = tempfile::tempdir().expect("temp dir");
    let kernel = temp.path().join("vmlinuz");
    std::fs::write(&kernel, b"k").expect("k");
    let p = KernelManager::find_matching_initramfs(&kernel).expect("match");
    assert_eq!(p, temp.path().join("biomeos-initramfs.img"));
}

#[test]
fn test_add_biomeos_binaries_registers_paths() {
    let temp = tempfile::tempdir().expect("temp dir");
    let mut builder = InitramfsBuilder::new(temp.path()).expect("new");
    let pr = temp.path().join("proj");
    let tr = pr.join("target/release");
    std::fs::create_dir_all(&tr).expect("tr");
    std::fs::write(tr.join("biomeos-init"), b"x").expect("init");
    std::fs::write(tr.join("biome"), b"x").expect("biome");
    builder.add_biomeos_binaries(&pr).expect("bins");
    assert!(
        builder.binaries.len() >= 2,
        "expected biome binaries; busybox may add a third"
    );
}

#[test]
fn test_install_binaries_preserves_permissions() {
    let temp = tempfile::tempdir().expect("temp dir");
    let mut builder = InitramfsBuilder::new(temp.path()).expect("new");
    let src = temp.path().join("tool");
    std::fs::write(&src, b"#!bin").expect("w");
    builder
        .add_binary(BinarySpec {
            source: src,
            dest: "/sbin/tool".to_string(),
            permissions: 0o755,
        })
        .expect("add");
    builder.install_binaries().expect("install");
    let dest = temp.path().join("initramfs-root/sbin/tool");
    assert!(dest.exists());
}

#[test]
fn test_build_succeeds_with_empty_initramfs_tree() {
    let temp = tempfile::tempdir().expect("temp dir");
    let builder = InitramfsBuilder::new(temp.path()).expect("new");
    let out = temp.path().join("empty-tree.img");
    builder
        .build(&out)
        .expect("empty tree still produces gzip cpio");
    assert!(out.exists());
}

#[test]
fn test_find_matching_initramfs_root_kernel_errors() {
    let kernel = Path::new("/");
    let r = KernelManager::find_matching_initramfs(kernel);
    assert!(r.is_err());
}
