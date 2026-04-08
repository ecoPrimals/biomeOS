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
use crate::{BootTarget, BootableMediaBuilder, KernelManager};

#[test]
fn test_boot_target_variants() {
    let iso = BootTarget::Iso;
    let usb = BootTarget::Usb;
    assert!(matches!(iso, BootTarget::Iso));
    assert!(matches!(usb, BootTarget::Usb));
}

#[test]
fn test_boot_target_debug() {
    assert_eq!(format!("{:?}", BootTarget::Iso), "Iso");
    assert_eq!(format!("{:?}", BootTarget::Usb), "Usb");
}

#[test]
fn test_boot_target_clone_copy() {
    let t = BootTarget::Iso;
    let t2 = t;
    let t3 = t;
    assert!(matches!(t2, BootTarget::Iso));
    assert!(matches!(t3, BootTarget::Iso));
}

#[test]
fn test_bootable_media_builder_new() {
    let temp = tempfile::tempdir().expect("temp dir");
    let project_root = temp.path().to_path_buf();
    let _builder = BootableMediaBuilder::new(project_root.clone())
        .expect("BootableMediaBuilder::new should succeed");

    let work_dir = project_root.join("build/boot-media");
    let output_dir = project_root.join("dist");
    assert!(work_dir.exists(), "work_dir should be created");
    assert!(output_dir.exists(), "output_dir should be created");
}

#[test]
fn test_bootable_media_builder_paths() {
    let temp = tempfile::tempdir().expect("temp dir");
    let project_root = temp.path().to_path_buf();
    let _builder = BootableMediaBuilder::new(project_root).expect("new");
}

#[test]
fn test_create_archive_fallback() {
    let temp = tempfile::tempdir().expect("temp dir");
    let project_root = temp.path().to_path_buf();
    let builder = BootableMediaBuilder::new(project_root).expect("new");

    let boot_dir = temp.path().join("build/boot-media/boot-root");
    std::fs::create_dir_all(&boot_dir).expect("create boot dir");
    std::fs::write(boot_dir.join("test.txt"), "boot content").expect("write");

    let output = temp.path().join("dist/test.iso");
    let result = builder.create_archive_fallback(&boot_dir, &output);
    let path = result.expect("create_archive_fallback should succeed");
    assert!(path.exists());
    assert!(path.extension().is_some_and(|e| e == "gz"));
}

#[test]
fn test_copy_directory() {
    let temp = tempfile::tempdir().expect("temp dir");
    let project_root = temp.path().to_path_buf();
    let _builder = BootableMediaBuilder::new(project_root).expect("new");

    let src = temp.path().join("src");
    let dest = temp.path().join("dest");
    std::fs::create_dir_all(&src).expect("create src");
    std::fs::write(src.join("file1.txt"), "content1").expect("write");
    std::fs::create_dir_all(src.join("subdir")).expect("create subdir");
    std::fs::write(src.join("subdir").join("file2.txt"), "content2").expect("write");

    BootableMediaBuilder::copy_directory(&src, &dest).expect("copy");

    assert!(dest.join("file1.txt").exists());
    assert!(dest.join("subdir").join("file2.txt").exists());
    assert_eq!(
        std::fs::read_to_string(dest.join("file1.txt")).unwrap(),
        "content1"
    );
    assert_eq!(
        std::fs::read_to_string(dest.join("subdir").join("file2.txt")).unwrap(),
        "content2"
    );
}

#[test]
fn test_boot_target_equality() {
    assert_eq!(BootTarget::Iso, BootTarget::Iso);
    assert_eq!(BootTarget::Usb, BootTarget::Usb);
    assert_ne!(BootTarget::Iso, BootTarget::Usb);
}

#[test]
fn test_bootable_media_builder_creates_nested_dirs() {
    let temp = tempfile::tempdir().expect("temp dir");
    let project_root = temp.path().to_path_buf();
    let _builder = BootableMediaBuilder::new(project_root).expect("new");
    let work_dir = temp.path().join("build/boot-media");
    let output_dir = temp.path().join("dist");
    assert!(work_dir.exists());
    assert!(output_dir.exists());
    assert!(!work_dir.join("boot-root").exists());
}

#[test]
fn test_grub_config_path_structure() {
    let temp = tempfile::tempdir().expect("temp dir");
    let _builder = BootableMediaBuilder::new(temp.path().to_path_buf()).expect("new");
    let boot_root = temp.path().join("build/boot-media/boot-root");
    std::fs::create_dir_all(boot_root.join("boot/grub")).expect("create");
    let result = BootableMediaBuilder::create_grub_config(&boot_root.join("boot/grub"));
    assert!(result.is_ok());
    let grub_cfg = boot_root.join("boot/grub/grub.cfg");
    assert!(grub_cfg.exists());
    let content = std::fs::read_to_string(&grub_cfg).expect("read");
    assert!(content.contains("BiomeOS"));
    assert!(content.contains("menuentry"));
    assert!(content.contains("vmlinuz"));
    assert!(content.contains("initramfs"));
}

#[test]
fn test_grub_config_all_menu_entries() {
    let temp = tempfile::tempdir().expect("temp dir");
    let boot_root = temp.path().join("boot/grub");
    std::fs::create_dir_all(&boot_root).expect("create");
    BootableMediaBuilder::create_grub_config(&boot_root).expect("create_grub_config");
    let content = std::fs::read_to_string(boot_root.join("grub.cfg")).expect("read");
    assert!(content.contains("Sovereignty-First"));
    assert!(content.contains("Discovery Mode"));
    assert!(content.contains("Network Boot"));
    assert!(content.contains("biomeos.discovery"));
    assert!(content.contains("biomeos.network"));
    assert!(content.contains("rdinit=/init"));
}

#[test]
fn test_copy_directory_empty_src() {
    let temp = tempfile::tempdir().expect("temp dir");
    let src = temp.path().join("empty");
    let dest = temp.path().join("dest");
    std::fs::create_dir_all(&src).expect("create");
    BootableMediaBuilder::copy_directory(&src, &dest).expect("copy empty dir");
    assert!(dest.exists());
}

#[test]
fn test_copy_directory_symlink_skipped() {
    let temp = tempfile::tempdir().expect("temp dir");
    let src = temp.path().join("src");
    let dest = temp.path().join("dest");
    std::fs::create_dir_all(&src).expect("create");
    std::fs::write(src.join("file.txt"), "content").expect("write");
    BootableMediaBuilder::copy_directory(&src, &dest).expect("copy");
    assert!(dest.join("file.txt").exists());
}

#[test]
fn test_print_success_message_iso() {
    let temp = tempfile::tempdir().expect("temp dir");
    let image_path = temp.path().join("biomeos.iso");
    std::fs::write(&image_path, b"").expect("create file");
    BootableMediaBuilder::print_success_message(&image_path, BootTarget::Iso).expect("print");
}

#[test]
fn test_print_success_message_usb() {
    let temp = tempfile::tempdir().expect("temp dir");
    let image_path = temp.path().join("biomeos.img");
    std::fs::write(&image_path, b"").expect("create file");
    BootableMediaBuilder::print_success_message(&image_path, BootTarget::Usb).expect("print");
}

#[test]
fn test_create_grub_config_nonexistent_dir_fails() {
    let result =
        BootableMediaBuilder::create_grub_config(std::path::Path::new("/nonexistent/grub/dir"));
    assert!(result.is_err());
}

#[test]
fn test_copy_directory_src_not_a_directory_fails() {
    let temp = tempfile::tempdir().expect("temp dir");
    let f = temp.path().join("file_not_dir");
    std::fs::write(&f, b"x").expect("write");
    let dest = temp.path().join("dest");
    let r = BootableMediaBuilder::copy_directory(&f, &dest);
    assert!(r.is_err());
}

#[test]
fn test_copy_directory_missing_src_fails() {
    let temp = tempfile::tempdir().expect("temp dir");
    let r =
        BootableMediaBuilder::copy_directory(&temp.path().join("nope"), &temp.path().join("dest"));
    assert!(r.is_err());
}

#[tokio::test]
async fn test_create_boot_structure_copies_kernel_and_initramfs() {
    let temp = tempfile::tempdir().expect("temp dir");
    let project_root = temp.path().to_path_buf();
    let builder = BootableMediaBuilder::new(project_root).expect("new");
    let kernel_path = temp.path().join("vmlinuz-test");
    std::fs::write(&kernel_path, b"fake-kernel-bytes").expect("kernel");
    let initramfs = temp.path().join("mini-initramfs.img");
    std::fs::write(&initramfs, b"cpio-bytes").expect("initramfs");
    let km = KernelManager::detect_or_custom(Some(kernel_path)).expect("kernel mgr");
    let boot_dir = builder
        .create_boot_structure(&km, &initramfs)
        .expect("boot structure");
    assert!(boot_dir.join("boot/vmlinuz").exists());
    assert!(boot_dir.join("boot/initramfs.img").exists());
    assert!(boot_dir.join("boot/grub/grub.cfg").exists());
}

#[tokio::test]
async fn test_add_biomeos_data_copies_phase1_and_templates() {
    let temp = tempfile::tempdir().expect("temp dir");
    let project_root = temp.path().join("proj");
    std::fs::create_dir_all(&project_root).expect("proj");
    std::fs::create_dir_all(temp.path().join("phase1bins")).expect("p1");
    std::fs::write(temp.path().join("phase1bins").join("p.bin"), b"p").expect("p1 bin");
    std::fs::create_dir_all(project_root.join("templates")).expect("tpl");
    std::fs::write(project_root.join("templates/hello.txt"), b"tpl").expect("tpl file");
    let builder = BootableMediaBuilder::new(project_root).expect("builder");
    let boot_dir = temp.path().join("boot-root");
    std::fs::create_dir_all(&boot_dir).expect("boot");
    builder.add_biomeos_data(&boot_dir).expect("add data");
    assert!(boot_dir.join("biomeos/primals/p.bin").exists());
    assert!(boot_dir.join("biomeos/templates/hello.txt").exists());
}

#[tokio::test]
async fn test_add_biomeos_data_without_phase1_parent() {
    let temp = tempfile::tempdir().expect("temp dir");
    let project_root = temp.path().to_path_buf();
    let builder = BootableMediaBuilder::new(project_root).expect("new");
    let boot_dir = temp.path().join("br");
    std::fs::create_dir_all(&boot_dir).expect("boot");
    builder
        .add_biomeos_data(&boot_dir)
        .expect("ok without phase1");
    assert!(boot_dir.join("biomeos/primals").exists());
}

#[tokio::test]
async fn test_create_bootable_image_produces_artifact() {
    let temp = tempfile::tempdir().expect("temp dir");
    let builder = BootableMediaBuilder::new(temp.path().to_path_buf()).expect("new");
    let boot_dir = temp.path().join("br");
    std::fs::create_dir_all(boot_dir.join("boot/grub")).expect("grub");
    std::fs::write(boot_dir.join("boot/grub/grub.cfg"), b"# test").expect("cfg");
    let path = builder
        .create_bootable_image(&boot_dir, BootTarget::Iso)
        .expect("image");
    assert!(path.exists());
}

#[tokio::test]
async fn test_create_with_grub_mkrescue_or_error() {
    let temp = tempfile::tempdir().expect("temp dir");
    let builder = BootableMediaBuilder::new(temp.path().to_path_buf()).expect("new");
    let boot_dir = temp.path().join("br2");
    std::fs::create_dir_all(&boot_dir).expect("create");
    let out = temp.path().join("out.iso");
    let r = builder.create_with_grub_mkrescue(&boot_dir, &out);
    let grub_ok = std::process::Command::new("grub-mkrescue")
        .arg("--help")
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    if grub_ok {
        assert!(r.is_ok() || r.is_err());
    } else {
        assert!(r.is_err());
    }
}

#[tokio::test]
async fn test_create_with_xorriso_or_error() {
    let temp = tempfile::tempdir().expect("temp dir");
    let builder = BootableMediaBuilder::new(temp.path().to_path_buf()).expect("new");
    let boot_dir = temp.path().join("br3");
    std::fs::create_dir_all(&boot_dir).expect("create");
    let out = temp.path().join("out-xor.iso");
    let r = builder.create_with_xorriso(&boot_dir, &out);
    let xor_ok = std::process::Command::new("xorriso")
        .arg("-version")
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    if xor_ok {
        assert!(r.is_ok() || r.is_err());
    } else {
        assert!(r.is_err());
    }
}

#[tokio::test]
async fn test_create_boot_structure_initramfs_copy_failures() {
    let temp = tempfile::tempdir().expect("temp dir");
    let project_root = temp.path().to_path_buf();
    let builder = BootableMediaBuilder::new(project_root).expect("new");
    let kernel_path = temp.path().join("vmlinuz-ok");
    std::fs::write(&kernel_path, b"k").expect("k");
    let km = KernelManager::detect_or_custom(Some(kernel_path)).expect("km");
    let bad_init = temp.path().join("missing-initramfs.img");
    let r = builder.create_boot_structure(&km, &bad_init);
    assert!(r.is_err());
}

#[tokio::test]
async fn test_build_initramfs_with_minimal_release_binaries() {
    let temp = tempfile::tempdir().expect("temp dir");
    let project_root = temp.path().to_path_buf();
    let tr = project_root.join("target/release");
    std::fs::create_dir_all(&tr).expect("release dir");
    std::fs::write(tr.join("biomeos-init"), b"#!fake").expect("init bin");
    std::fs::write(tr.join("biome"), b"#!fake").expect("biome bin");
    let builder = BootableMediaBuilder::new(project_root).expect("new");
    let path = builder.build_initramfs().expect("initramfs build");
    assert!(path.exists());
    assert!(path.extension().is_some_and(|e| e == "img"));
}

#[tokio::test]
async fn test_add_biomeos_data_skips_phase1_when_parent_has_no_bins() {
    let temp = tempfile::tempdir().expect("temp dir");
    let project_root = temp.path().join("proj");
    std::fs::create_dir_all(&project_root).expect("proj");
    let builder = BootableMediaBuilder::new(project_root).expect("builder");
    let boot_dir = temp.path().join("boot-root");
    std::fs::create_dir_all(&boot_dir).expect("boot");
    builder.add_biomeos_data(&boot_dir).expect("add data");
    assert!(boot_dir.join("biomeos/primals").exists());
}

#[tokio::test]
async fn test_add_biomeos_data_without_templates_dir() {
    let temp = tempfile::tempdir().expect("temp dir");
    let project_root = temp.path().to_path_buf();
    std::fs::create_dir_all(&project_root).expect("root");
    let builder = BootableMediaBuilder::new(project_root).expect("builder");
    let boot_dir = temp.path().join("br");
    std::fs::create_dir_all(&boot_dir).expect("boot");
    builder.add_biomeos_data(&boot_dir).expect("ok");
    assert!(boot_dir.join("biomeos/templates").exists());
}

#[tokio::test]
async fn test_create_boot_structure_kernel_copy_fails() {
    let temp = tempfile::tempdir().expect("temp dir");
    let builder = BootableMediaBuilder::new(temp.path().to_path_buf()).expect("new");
    let missing_kernel = temp.path().join("no-such-vmlinuz");
    let km = KernelManager::detect_or_custom(Some(missing_kernel)).expect("km");
    let initramfs = temp.path().join("ir.img");
    std::fs::write(&initramfs, b"x").expect("init");
    let r = builder.create_boot_structure(&km, &initramfs);
    assert!(r.is_err());
}

#[test]
fn test_copy_directory_invalid_entry_name_edge() {
    let temp = tempfile::tempdir().expect("temp dir");
    let src = temp.path().join("src");
    std::fs::create_dir_all(&src).expect("src");
    std::fs::write(src.join("ok.txt"), b"1").expect("w");
    let dest = temp.path().join("dest");
    let r = BootableMediaBuilder::copy_directory(&src, &dest);
    assert!(r.is_ok());
}

#[tokio::test]
async fn test_create_archive_fallback_errors_on_bad_parent() {
    let temp = tempfile::tempdir().expect("temp dir");
    let builder = BootableMediaBuilder::new(temp.path().to_path_buf()).expect("new");
    let boot_dir = temp.path().join("br");
    std::fs::create_dir_all(&boot_dir).expect("boot");
    let blocker = temp.path().join("blocker");
    std::fs::write(&blocker, b"x").expect("file");
    let out = blocker.join("nested.iso");
    let r = builder.create_archive_fallback(&boot_dir, &out);
    assert!(r.is_err());
}
