// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Tests for [`crate::GenomeDeployer`] and deployment flows.

use super::helpers::write_genome_bin_with_arch_dir;
use crate::{Architecture, GenomeDeployer, Platform};
use std::fs::File;
use std::io::Write;
use tempfile::TempDir;

// ============================================================================
// GenomeDeployer Tests
// ============================================================================

#[test]
fn test_genome_deployer_new_file_not_found() {
    let result = GenomeDeployer::new("/nonexistent/path/to/genome.genome");
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("not found"));
}

#[test]
fn test_genome_deployer_new_with_existing_file() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let genome_path = temp_dir.path().join("test.genome");

    // Create a dummy genome file
    let mut file = File::create(&genome_path).expect("Failed to create file");
    file.write_all(b"dummy content").expect("Failed to write");

    let result = GenomeDeployer::new(&genome_path);
    assert!(result.is_ok(), "Should create deployer for existing file");
}

#[test]
fn test_genome_deployer_with_install_dir() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let genome_path = temp_dir.path().join("test.genome");

    let mut file = File::create(&genome_path).expect("Failed to create file");
    file.write_all(b"dummy content").expect("Failed to write");

    let install_path = temp_dir.path().join("custom_install");
    std::fs::create_dir_all(&install_path).expect("create");

    let deployer = GenomeDeployer::new(&genome_path)
        .expect("Should create deployer")
        .with_install_dir(&install_path);

    let result = deployer.deploy();
    assert!(result.is_err());
}

#[test]
fn test_genome_deployer_deploy_with_custom_dir_fails_without_archive() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let genome_path = temp_dir.path().join("test.genome");

    let mut file = File::create(&genome_path).expect("Failed to create file");
    file.write_all(b"dummy content").expect("Failed to write");

    let deployer = GenomeDeployer::new(&genome_path).expect("Should create deployer");

    let custom_dir = temp_dir.path().join("install");
    std::fs::create_dir_all(&custom_dir).expect("create");
    let deployer_with_dir = deployer.with_install_dir(&custom_dir);
    let result = deployer_with_dir.deploy();
    assert!(result.is_err());
}

#[test]
fn test_genome_deployer_architecture_field() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let genome_path = temp_dir.path().join("test.genome");

    let mut file = File::create(&genome_path).expect("Failed to create file");
    file.write_all(b"dummy content").expect("Failed to write");

    let deployer = GenomeDeployer::new(&genome_path).expect("Should create deployer");

    // Verify architecture was detected
    let arch_str = deployer.architecture.as_str();
    assert!(
        ["x86_64", "aarch64", "armv7", "riscv64"].contains(&arch_str),
        "Should have valid architecture"
    );
}

#[test]
fn test_genome_deployer_platform_field() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let genome_path = temp_dir.path().join("test.genome");

    let mut file = File::create(&genome_path).expect("Failed to create file");
    file.write_all(b"dummy content").expect("Failed to write");

    let deployer = GenomeDeployer::new(&genome_path).expect("Should create deployer");

    // Verify platform was detected
    let platform_name = deployer.platform.name();
    assert!(
        ["Linux", "Android", "macOS", "Windows"].contains(&platform_name),
        "Should have valid platform"
    );
}

// ============================================================================
// Integration-style Tests (without actual deployment)
// ============================================================================

#[test]
fn test_deploy_fails_without_archive_marker() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let genome_path = temp_dir.path().join("invalid.genome");

    // Create file without archive marker
    let mut file = File::create(&genome_path).expect("Failed to create file");
    file.write_all(b"not a valid genome file")
        .expect("Failed to write");

    let deployer = GenomeDeployer::new(&genome_path).expect("Should create deployer");
    let result = deployer.deploy();

    // Should fail because no archive marker
    assert!(result.is_err(), "Should fail without archive marker");
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("marker") || err.contains("Archive"),
        "Error should mention missing marker"
    );
}

#[test]
fn test_all_platform_install_dirs_are_unique() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let genome_path = temp_dir.path().join("test.genome");

    let mut file = File::create(&genome_path).expect("Failed to create file");
    file.write_all(b"dummy content").expect("Failed to write");

    let deployer = GenomeDeployer::new(&genome_path).expect("Should create deployer");

    // Generate paths for different primals
    let path1 = deployer.default_install_dir("primal1");
    let path2 = deployer.default_install_dir("primal2");

    // They should be different
    assert_ne!(
        path1, path2,
        "Different primals should have different install dirs"
    );
}

#[test]
fn test_platform_abstract_socket_consistency() {
    // Platforms that support abstract sockets should be Linux-family
    let linux_supports = Platform::Linux.supports_abstract_sockets();
    let android_supports = Platform::Android.supports_abstract_sockets();
    let macos_supports = Platform::MacOS.supports_abstract_sockets();
    let windows_supports = Platform::Windows.supports_abstract_sockets();

    // Linux and Android are Linux-family, should support abstract sockets
    assert!(linux_supports);
    assert!(android_supports);

    // macOS and Windows should not
    assert!(!macos_supports);
    assert!(!windows_supports);
}

#[test]
fn test_default_install_dir_linux_user_uses_dot_local() {
    let temp = TempDir::new().expect("temp dir");
    let genome_path = temp.path().join("primal.genome");
    let mut f = File::create(&genome_path).expect("create");
    f.write_all(b"x").expect("write");
    let deployer = GenomeDeployer::new(&genome_path).expect("deployer");
    if deployer.platform != Platform::Linux {
        return;
    }
    let p = deployer.default_install_dir("myprimal");
    let s = p.to_string_lossy();
    assert!(
        s.contains(".local/myprimal") || s.contains("/opt/myprimal"),
        "unexpected install dir: {s}"
    );
}

#[test]
fn test_extract_archive_wrong_arch_fails() {
    let temp = TempDir::new().expect("temp dir");
    let genome_path = temp.path().join("wrongarch.genome");
    write_genome_bin_with_arch_dir(&genome_path, "definitely_not_a_real_arch_triple");
    let install = temp.path().join("out");
    std::fs::create_dir_all(&install).expect("install");
    let deployer = GenomeDeployer::new(&genome_path)
        .expect("deployer")
        .with_install_dir(&install);
    let err = deployer
        .deploy()
        .expect_err("deploy should fail when arch dir missing");
    let msg = err.to_string();
    assert!(
        msg.contains("No binaries") || msg.contains("architecture"),
        "unexpected: {msg}"
    );
}

#[test]
fn test_deploy_full_success_with_stub_binary() {
    let temp = TempDir::new().expect("temp dir");
    let genome_path = temp.path().join("stub.genome");
    let arch = Architecture::detect().expect("arch").as_str();
    write_genome_bin_with_arch_dir(&genome_path, arch);
    let install = temp.path().join("install");
    std::fs::create_dir_all(&install).expect("install");
    let deployer = GenomeDeployer::new(&genome_path)
        .expect("deployer")
        .with_install_dir(&install);
    deployer.deploy().expect("deploy");
    let binary = install.join("stub");
    assert!(binary.exists(), "stub binary should be installed");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(&binary)
            .expect("meta")
            .permissions()
            .mode();
        assert!(mode & 0o111 != 0, "binary should be executable");
    }
}

#[test]
fn test_verify_installation_skips_version_when_binary_nonzero() {
    let temp = TempDir::new().expect("temp dir");
    let install = temp.path().join("verify");
    std::fs::create_dir_all(&install).expect("dir");
    let primal = install.join("verifybin");
    std::fs::write(&primal, b"not executable").expect("file");
    let r = GenomeDeployer::verify_installation(&install, "verifybin");
    assert!(r.is_ok());
}
