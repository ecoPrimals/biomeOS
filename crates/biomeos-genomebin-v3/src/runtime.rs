// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

// biomeos-genomebin-v3/src/runtime.rs
// Runtime extraction and execution
//
// Deep Debt Principles:
// - Platform-agnostic installation paths (etcetera)
// - Runtime discovery (no hardcoding)
// - Clear error messages

use crate::{Arch, GenomeBin};
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

impl GenomeBin {
    /// Extract binary for current architecture to directory
    pub fn extract(&self, install_dir: &Path) -> Result<PathBuf> {
        let arch = Arch::detect();
        tracing::info!("Extracting {:?} binary to: {}", arch, install_dir.display());

        // Get binary for current arch
        let compressed = self
            .binaries
            .get(&arch)
            .with_context(|| format!("No binary for current architecture: {arch:?}"))?;

        // Decompress
        let decompressed = compressed
            .decompress()
            .context("Failed to decompress binary")?;

        // Create install directory
        std::fs::create_dir_all(install_dir)
            .with_context(|| format!("Failed to create directory: {}", install_dir.display()))?;

        // Write binary
        let binary_path = install_dir.join(&self.manifest.name);
        std::fs::write(&binary_path, decompressed)
            .with_context(|| format!("Failed to write binary: {}", binary_path.display()))?;

        // Make executable on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = binary_path.metadata()?.permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&binary_path, perms)?;
        }

        tracing::info!("✅ Extracted to: {}", binary_path.display());

        // If this has embedded genomes, extract them too
        for embedded in &self.embedded_genomes {
            let embedded_dir = install_dir.join(&embedded.manifest.name);
            embedded.extract(&embedded_dir).with_context(|| {
                format!(
                    "Failed to extract embedded genome: {}",
                    embedded.manifest.name
                )
            })?;
        }

        Ok(binary_path)
    }

    /// Get default install directory for current platform
    ///
    /// Deep Debt: Uses etcetera for platform-agnostic paths
    pub fn default_install_dir() -> Result<PathBuf> {
        let user = std::env::var("USER").unwrap_or_default();
        Self::default_install_dir_for_user(&user)
    }

    /// Same as [`Self::default_install_dir`], but with an explicit username (no `USER` env read).
    pub fn default_install_dir_for_user(user: &str) -> Result<PathBuf> {
        use etcetera::BaseStrategy;

        // Check for Android
        if std::path::Path::new("/system/build.prop").exists() {
            tracing::debug!("Detected Android platform");
            return Ok(PathBuf::from("/data/local/tmp"));
        }

        // Use XDG base directories (etcetera - Pure Rust)
        let strategy = etcetera::base_strategy::choose_base_strategy()
            .context("Failed to determine base directory strategy")?;

        let is_root = user == "root";

        if is_root {
            Ok(PathBuf::from("/opt"))
        } else {
            let data_dir = strategy.data_dir();
            Ok(data_dir.join("..").join(".local"))
        }
    }

    /// Run in-place without full extraction.
    ///
    /// Extracts to a temporary directory for execution. A zero-copy `mmap` path
    /// can be added when the binary format supports direct execution from the
    /// archive (requires ELF section alignment guarantees).
    pub fn run_in_place(&self, args: &[String]) -> Result<()> {
        tracing::info!("Running {} in-place", self.manifest.name);

        // Extract to temp directory, execute, cleanup
        let temp_dir = tempfile::tempdir().context("Failed to create temporary directory")?;

        let binary_path = self
            .extract(temp_dir.path())
            .context("Failed to extract to temp directory")?;

        // Execute
        let status = std::process::Command::new(&binary_path)
            .args(args)
            .status()
            .with_context(|| format!("Failed to execute: {}", binary_path.display()))?;

        if !status.success() {
            anyhow::bail!("Process exited with status: {status}");
        }

        // Temp dir automatically cleaned up on drop
        Ok(())
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
#[expect(
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
mod tests {
    use super::*;

    #[test]
    fn test_default_install_dir() {
        let dir = GenomeBin::default_install_dir().expect("default_install_dir should succeed");
        assert!(dir.is_absolute(), "install dir should be absolute path");
        // Should be /opt for root or ~/.local for non-root
        let path_str = dir.to_str().expect("path should be valid UTF-8");
        assert!(
            path_str.contains("/opt")
                || path_str.contains("/.local")
                || path_str.contains("/data/local/tmp"),
            "path should be platform-appropriate, got: {path_str}"
        );
    }

    #[test]
    fn test_extract_no_binary_for_arch() {
        // GenomeBin with no binary for current arch should fail
        let genome = GenomeBin::new("empty-genome");
        let temp = tempfile::tempdir().expect("temp dir");
        let err = genome
            .extract(temp.path())
            .expect_err("extract should fail when no binary");
        let msg = err.to_string();
        assert!(
            msg.contains("No binary") || msg.contains("architecture"),
            "error should mention missing binary/arch: {msg}"
        );
    }

    #[test]
    fn test_extract_success() {
        let mut genome = GenomeBin::new("extract-test");
        genome.add_binary_bytes(Arch::detect(), b"#!/bin/sh\necho hello");
        let temp = tempfile::tempdir().expect("temp dir");
        let path = genome
            .extract(temp.path())
            .expect("extract should succeed with binary for current arch");
        assert!(path.exists(), "extracted binary should exist");
        assert_eq!(path.file_name().unwrap(), "extract-test");
        let contents = std::fs::read(&path).expect("read binary");
        assert_eq!(contents, b"#!/bin/sh\necho hello");
    }

    #[test]
    fn test_has_current_arch() {
        let mut genome = GenomeBin::new("arch-test");
        assert!(!genome.has_current_arch());
        genome.add_binary_bytes(Arch::detect(), b"data");
        assert!(genome.has_current_arch());
    }

    #[test]
    fn test_run_in_place_success() {
        let mut genome = GenomeBin::new("run-test");
        genome.add_binary_bytes(Arch::detect(), b"#!/bin/sh\nexit 0");
        let result = genome.run_in_place(&[]);
        result.expect("run_in_place with exit 0 should succeed");
    }

    #[test]
    fn test_run_in_place_failure() {
        let mut genome = GenomeBin::new("run-fail");
        genome.add_binary_bytes(Arch::detect(), b"#!/bin/sh\nexit 1");
        let result = genome.run_in_place(&[]);
        assert!(
            result.is_err(),
            "exit 1 script should produce an error, got Ok"
        );
    }

    #[test]
    fn test_extract_with_embedded_genome() {
        let mut parent = GenomeBin::new("parent");
        parent.add_binary_bytes(Arch::detect(), b"parent-bin");
        let mut child = GenomeBin::new("child");
        child.add_binary_bytes(Arch::detect(), b"child-bin");
        parent.embed(child).expect("embed");
        let temp = tempfile::tempdir().expect("temp dir");
        let path = parent.extract(temp.path()).expect("extract");
        assert!(path.exists());
        let child_dir = temp.path().join("child");
        assert!(child_dir.exists());
        let child_bin = child_dir.join("child");
        assert!(child_bin.exists());
    }

    #[test]
    fn test_default_install_dir_root_user_uses_opt_on_non_android() {
        let dir = GenomeBin::default_install_dir_for_user("root").expect("default_install_dir");
        if std::path::Path::new("/system/build.prop").exists() {
            assert_eq!(dir, std::path::PathBuf::from("/data/local/tmp"));
        } else {
            assert_eq!(dir, std::path::PathBuf::from("/opt"));
        }
    }
}
