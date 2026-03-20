// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

#![allow(clippy::unwrap_used, clippy::expect_used)]

//! End-to-end tests for biomeos-boot CLI tools
//!
//! These tests execute the actual binaries and verify their behavior.

use anyhow::Result;
use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_biomeos_mkboot_help() -> Result<()> {
    let mut cmd = cargo_bin_cmd!("biomeos-mkboot");
    cmd.arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("BiomeOS Bootable Media Creator"))
        .stdout(predicate::str::contains("usb"))
        .stdout(predicate::str::contains("iso"))
        .stdout(predicate::str::contains("initramfs"));

    Ok(())
}

#[test]
fn test_biomeos_mkboot_usb_help() -> Result<()> {
    let mut cmd = cargo_bin_cmd!("biomeos-mkboot");
    cmd.arg("usb").arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Create bootable USB image"));

    Ok(())
}

#[test]
fn test_biomeos_mkboot_iso_help() -> Result<()> {
    let mut cmd = cargo_bin_cmd!("biomeos-mkboot");
    cmd.arg("iso").arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Create bootable ISO image"));

    Ok(())
}

#[test]
fn test_biomeos_mkboot_initramfs_help() -> Result<()> {
    let mut cmd = cargo_bin_cmd!("biomeos-mkboot");
    cmd.arg("initramfs").arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Test build initramfs only"))
        .stdout(predicate::str::contains("--output"));

    Ok(())
}

#[test]
fn test_biomeos_mkboot_invalid_command() -> Result<()> {
    let mut cmd = cargo_bin_cmd!("biomeos-mkboot");
    cmd.arg("invalid-command");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("error"));

    Ok(())
}

#[test]
fn test_biomeos_mkboot_initramfs_missing_output() -> Result<()> {
    let mut cmd = cargo_bin_cmd!("biomeos-mkboot");
    cmd.arg("initramfs");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("required"));

    Ok(())
}

#[test]
fn test_biomeos_mkboot_initramfs_build() -> Result<()> {
    let temp = TempDir::new()?;
    let output = temp.path().join("test-initramfs.img");

    // Setup minimal project structure
    let project_root = temp.path().join("project");
    fs::create_dir_all(project_root.join("target/release"))?;
    fs::write(
        project_root.join("target/release/biomeos-init"),
        "#!/bin/sh\necho init",
    )?;
    fs::write(
        project_root.join("target/release/biome"),
        "#!/bin/sh\necho cli",
    )?;

    let mut cmd = cargo_bin_cmd!("biomeos-mkboot");
    cmd.arg("--project-root")
        .arg(&project_root)
        .arg("initramfs")
        .arg("--output")
        .arg(&output);

    let result = cmd.assert();

    // Should either succeed or fail with reasonable error
    // (may fail if busybox not available, which is okay)
    let output_assert = result.get_output();
    if output_assert.status.success() {
        assert!(output.exists(), "Initramfs should be created");

        let metadata = fs::metadata(&output)?;
        assert!(metadata.len() > 0, "Initramfs should not be empty");
    } else {
        // If it failed, error should be informative
        let stderr = String::from_utf8_lossy(&output_assert.stderr);
        assert!(
            stderr.contains("Failed") || stderr.contains("Error"),
            "Should have meaningful error message"
        );
    }

    Ok(())
}

#[test]
fn test_biomeos_mkboot_custom_project_root() -> Result<()> {
    let temp = TempDir::new()?;
    let custom_root = temp.path().join("custom");
    fs::create_dir_all(&custom_root)?;

    let mut cmd = cargo_bin_cmd!("biomeos-mkboot");
    cmd.arg("--project-root").arg(&custom_root).arg("--help");

    cmd.assert().success();

    Ok(())
}

#[test]
fn test_biomeos_mkboot_nonexistent_project_root() -> Result<()> {
    let mut cmd = cargo_bin_cmd!("biomeos-mkboot");
    cmd.arg("--project-root")
        .arg("/nonexistent/path/that/should/fail")
        .arg("initramfs")
        .arg("--output")
        .arg("/tmp/test.img");

    cmd.assert().failure();

    Ok(())
}

#[test]
fn test_biomeos_init_not_pid1() -> Result<()> {
    // biomeos-init should detect it's not PID 1
    // Note: This might produce no output on some systems, which is okay
    let mut cmd = cargo_bin_cmd!("biomeos-init");

    cmd.assert().failure(); // Just check it exits with error

    Ok(())
}

#[test]
fn test_concurrent_mkboot_commands() -> Result<()> {
    use std::thread;

    let handles: Vec<_> = (0..3)
        .map(|_| {
            thread::spawn(|| -> Result<()> {
                let mut cmd = cargo_bin_cmd!("biomeos-mkboot");
                cmd.arg("--help");
                cmd.assert().success();
                Ok(())
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap()?;
    }

    Ok(())
}

#[test]
fn test_biomeos_mkboot_output_formatting() -> Result<()> {
    let mut cmd = cargo_bin_cmd!("biomeos-mkboot");
    cmd.arg("--help");

    let output = cmd.output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should have clean, formatted output
    assert!(stdout.contains("Usage:"));
    assert!(stdout.contains("Commands:"));
    assert!(stdout.contains("Options:"));

    Ok(())
}

#[test]
fn test_error_messages_are_helpful() -> Result<()> {
    let mut cmd = cargo_bin_cmd!("biomeos-mkboot");
    cmd.arg("initramfs"); // Missing required --output

    let output = cmd.output()?;
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Error should mention what's missing
    assert!(stderr.contains("output") || stderr.contains("required"));

    Ok(())
}

#[test]
fn test_biomeos_mkboot_with_env_vars() -> Result<()> {
    let _temp = TempDir::new()?;

    let mut cmd = cargo_bin_cmd!("biomeos-mkboot");
    cmd.env("RUST_LOG", "debug").arg("--help");

    cmd.assert().success();

    Ok(())
}

#[test]
fn test_biomeos_mkboot_stdin_not_required() -> Result<()> {
    let mut cmd = cargo_bin_cmd!("biomeos-mkboot");
    cmd.arg("--help").write_stdin(""); // Empty stdin

    cmd.assert().success();

    Ok(())
}

#[test]
fn test_all_subcommands_documented() -> Result<()> {
    let mut cmd = cargo_bin_cmd!("biomeos-mkboot");
    cmd.arg("--help");

    let output = cmd.output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);

    // All subcommands should be documented
    assert!(stdout.contains("usb"));
    assert!(stdout.contains("iso"));
    assert!(stdout.contains("initramfs"));

    Ok(())
}
