// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! VM verification and validation
//!
//! Modern idiomatic Rust replacement for verify-primals.sh
//! Provides comprehensive VM boot verification and primal installation validation.

use crate::error::{DeployError, Result};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::fs;
use tokio::time::timeout;
use tracing::{debug, info, warn};

/// Default retry interval when waiting for log file (500ms).
pub const DEFAULT_RETRY_INTERVAL: Duration = Duration::from_millis(500);

/// Verification configuration
#[derive(Debug, Clone)]
pub struct VerifyConfig {
    /// Path to serial log file
    pub serial_log: PathBuf,

    /// Optional root filesystem directory to check
    pub rootfs_dir: Option<PathBuf>,

    /// Boot timeout (seconds)
    pub boot_timeout: u64,

    /// Expected boot message
    pub expected_boot_message: String,

    /// Interval between retries when waiting for log file (default: 500ms)
    pub retry_interval: Duration,
}

impl Default for VerifyConfig {
    fn default() -> Self {
        // Use XDG-compliant path via SystemPaths
        let serial_log = biomeos_types::SystemPaths::new_lazy()
            .runtime_dir()
            .join("verify.log");

        Self {
            serial_log,
            rootfs_dir: None,
            boot_timeout: 30,
            expected_boot_message: "BiomeOS initialization complete".to_string(),
            retry_interval: DEFAULT_RETRY_INTERVAL,
        }
    }
}

/// Verification result
#[derive(Debug, Clone)]
pub struct VerifyResult {
    /// Boot successful
    pub boot_success: bool,

    /// Boot time in milliseconds
    pub boot_time_ms: Option<u64>,

    /// Shell spawned
    pub shell_spawned: bool,

    /// Number of primals found
    pub primal_count: Option<usize>,

    /// Primal names
    pub primals: Vec<String>,

    /// Boot log excerpt
    pub log_excerpt: String,
}

impl VerifyResult {
    /// Check if verification passed
    #[must_use]
    pub const fn is_ok(&self) -> bool {
        self.boot_success
    }

    /// Get a human-readable summary
    #[must_use]
    pub fn summary(&self) -> String {
        use std::fmt::Write;
        let mut summary = String::new();

        summary.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
        summary.push_str("✅ Boot Log Analysis\n");
        summary.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n\n");

        if self.boot_success {
            summary.push_str("✅ BiomeOS booted successfully\n");
        } else {
            summary.push_str("❌ Boot failed\n");
        }

        if self.shell_spawned {
            summary.push_str("✅ Shell spawned\n");
        } else {
            summary.push_str("⚠️  Shell not detected\n");
        }

        if let Some(boot_time) = self.boot_time_ms {
            let _ = writeln!(summary, "⏱️  Boot time: {boot_time}ms");
        }

        if let Some(count) = self.primal_count {
            summary.push_str("\n📦 Primal Installation Check:\n\n");
            let _ = writeln!(summary, "✅ Found {count} primals");

            if !self.primals.is_empty() {
                summary.push_str("\nPrimals:\n");
                for primal in &self.primals {
                    let _ = writeln!(summary, "  • {primal}");
                }
            }
        }

        summary.push_str("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

        summary
    }
}

/// VM Verifier
pub struct VmVerifier {
    config: VerifyConfig,
}

impl Default for VmVerifier {
    fn default() -> Self {
        Self::new(VerifyConfig::default())
    }
}

impl VmVerifier {
    /// Create a new verifier
    #[must_use]
    pub const fn new(config: VerifyConfig) -> Self {
        Self { config }
    }

    /// Verify VM boot from serial log
    pub async fn verify_boot(&self) -> Result<VerifyResult> {
        info!(
            "Verifying VM boot from: {}",
            self.config.serial_log.display()
        );

        // Wait for log file with timeout
        let log_content = timeout(
            Duration::from_secs(self.config.boot_timeout),
            self.wait_for_log(),
        )
        .await
        .map_err(|_| DeployError::Timeout {
            operation: "waiting for boot log".to_string(),
            timeout_secs: self.config.boot_timeout,
        })??;

        // Parse boot log
        let mut result = VerifyResult {
            boot_success: false,
            boot_time_ms: None,
            shell_spawned: false,
            primal_count: None,
            primals: Vec::new(),
            log_excerpt: Self::extract_excerpt(&log_content),
        };

        // Check boot success
        result.boot_success = log_content.contains(&self.config.expected_boot_message);

        // Check shell spawned
        result.shell_spawned =
            log_content.contains("Spawning shell") || log_content.contains("shell started");

        // Extract boot time
        result.boot_time_ms = Self::extract_boot_time(&log_content);

        // Check primals if rootfs provided
        if let Some(ref rootfs_dir) = self.config.rootfs_dir {
            let (count, names) = self.check_primals(rootfs_dir).await?;
            result.primal_count = Some(count);
            result.primals = names;
        }

        Ok(result)
    }

    /// Wait for log file to be created and contain data
    async fn wait_for_log(&self) -> Result<String> {
        let max_attempts = self.config.boot_timeout * 2; // Check every 500ms

        for attempt in 0..max_attempts {
            if self.config.serial_log.exists() {
                match fs::read_to_string(&self.config.serial_log).await {
                    Ok(content) if !content.is_empty() => {
                        debug!("Log file ready after {} attempts", attempt);
                        return Ok(content);
                    }
                    Ok(_) => {
                        debug!("Log file empty, waiting...");
                    }
                    Err(e) => {
                        debug!("Error reading log: {}", e);
                    }
                }
            }

            tokio::time::sleep(self.config.retry_interval).await;
        }

        Err(DeployError::FileNotFound {
            path: self.config.serial_log.clone(),
        })
    }

    /// Extract boot time from log
    fn extract_boot_time(log: &str) -> Option<u64> {
        // Look for patterns like "BootLogger stats: 145ms" or "Boot time: 145ms"
        for line in log.lines() {
            if line.contains("BootLogger stats") || line.contains("Boot time") {
                // Extract number followed by "ms"
                if let Some(pos) = line.find("ms") {
                    let before = &line[..pos];
                    if let Some(num_start) = before.rfind(|c: char| !c.is_ascii_digit()) {
                        if let Ok(ms) = before[num_start + 1..].parse::<u64>() {
                            return Some(ms);
                        }
                    }
                }
            }
        }
        None
    }

    /// Extract relevant excerpt from log
    fn extract_excerpt(log: &str) -> String {
        // Get last 20 lines or up to 1000 chars
        let lines: Vec<&str> = log.lines().collect();
        let start = lines.len().saturating_sub(20);
        lines[start..].join("\n")
    }

    /// Check primals in root filesystem
    async fn check_primals(&self, rootfs_dir: &Path) -> Result<(usize, Vec<String>)> {
        let primal_dir = rootfs_dir.join("usr/local/bin");

        if !primal_dir.exists() {
            warn!("Primal directory not found: {}", primal_dir.display());
            return Ok((0, Vec::new()));
        }

        let mut primals = Vec::new();
        let mut entries =
            fs::read_dir(&primal_dir)
                .await
                .map_err(|_e| DeployError::FileNotFound {
                    path: primal_dir.clone(),
                })?;

        while let Some(entry) =
            entries
                .next_entry()
                .await
                .map_err(|e| DeployError::ConfigError {
                    message: format!("Failed to read directory entry: {e}"),
                })?
        {
            let file_name = entry.file_name();
            let name = file_name.to_string_lossy().to_string();

            // Check if it's an executable
            if let Ok(metadata) = entry.metadata().await {
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    if metadata.is_file() && (metadata.permissions().mode() & 0o111) != 0 {
                        primals.push(name);
                    }
                }

                #[cfg(not(unix))]
                {
                    if metadata.is_file() {
                        primals.push(name);
                    }
                }
            }
        }

        primals.sort();
        let count = primals.len();

        Ok((count, primals))
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
    use crate::error::DeployError;
    use std::io::Write;
    use tempfile::{NamedTempFile, TempDir};

    #[tokio::test]
    async fn test_extract_boot_time() {
        let log = "Some boot output\nBootLogger stats: 145ms\nMore output";
        assert_eq!(VmVerifier::extract_boot_time(log), Some(145));

        let log2 = "Boot time: 250ms";
        assert_eq!(VmVerifier::extract_boot_time(log2), Some(250));
    }

    #[test]
    fn test_extract_boot_time_no_match() {
        assert_eq!(VmVerifier::extract_boot_time("no timing here\n"), None);
    }

    #[test]
    fn test_extract_excerpt_truncates_long_log() {
        let mut log = String::new();
        for i in 0..40 {
            std::fmt::Write::write_fmt(&mut log, format_args!("line {i}\n")).unwrap();
        }
        let excerpt = VmVerifier::extract_excerpt(&log);
        assert!(excerpt.contains("line 39"));
        assert!(!excerpt.contains("line 0"));
        assert!(excerpt.lines().count() <= 20);
    }

    #[test]
    fn test_verify_result_is_ok_and_summary_branches() {
        let ok = VerifyResult {
            boot_success: true,
            boot_time_ms: Some(10),
            shell_spawned: true,
            primal_count: Some(2),
            primals: vec!["a".into(), "b".into()],
            log_excerpt: String::new(),
        };
        assert!(ok.is_ok());
        let s = ok.summary();
        assert!(s.contains("BiomeOS booted successfully"));
        assert!(s.contains("Shell spawned"));
        assert!(s.contains("Boot time"));

        let bad = VerifyResult {
            boot_success: false,
            boot_time_ms: None,
            shell_spawned: false,
            primal_count: None,
            primals: vec![],
            log_excerpt: String::new(),
        };
        assert!(!bad.is_ok());
        let s2 = bad.summary();
        assert!(s2.contains("Boot failed"));
        assert!(s2.contains("Shell not detected"));
    }

    #[tokio::test]
    async fn test_verify_boot_success() {
        let mut log_file = NamedTempFile::new().unwrap();
        writeln!(log_file, "BiomeOS booting...").unwrap();
        writeln!(log_file, "BiomeOS initialization complete").unwrap();
        writeln!(log_file, "Spawning shell").unwrap();
        writeln!(log_file, "BootLogger stats: 123ms").unwrap();
        log_file.flush().unwrap();

        let config = VerifyConfig {
            serial_log: log_file.path().to_path_buf(),
            rootfs_dir: None,
            boot_timeout: 5,
            expected_boot_message: "BiomeOS initialization complete".to_string(),
            retry_interval: Duration::ZERO,
        };

        let verifier = VmVerifier::new(config);
        let result = verifier.verify_boot().await.unwrap();

        assert!(result.boot_success);
        assert!(result.shell_spawned);
        assert_eq!(result.boot_time_ms, Some(123));
    }

    #[tokio::test]
    async fn test_verify_boot_detects_shell_started_variant() {
        let mut log_file = NamedTempFile::new().unwrap();
        writeln!(log_file, "BiomeOS initialization complete").unwrap();
        writeln!(log_file, "shell started").unwrap();
        log_file.flush().unwrap();

        let config = VerifyConfig {
            serial_log: log_file.path().to_path_buf(),
            rootfs_dir: None,
            boot_timeout: 5,
            expected_boot_message: "BiomeOS initialization complete".to_string(),
            retry_interval: Duration::ZERO,
        };

        let verifier = VmVerifier::new(config);
        let result = verifier.verify_boot().await.unwrap();
        assert!(result.shell_spawned);
    }

    #[tokio::test]
    async fn test_wait_for_log_file_not_found() {
        let missing = std::path::PathBuf::from("/nonexistent/verify-log-xyz-12345.log");
        let config = VerifyConfig {
            serial_log: missing.clone(),
            rootfs_dir: None,
            boot_timeout: 1,
            expected_boot_message: "x".to_string(),
            retry_interval: Duration::from_millis(10),
        };
        let verifier = VmVerifier::new(config);
        let err = verifier
            .wait_for_log()
            .await
            .expect_err("expected file not found");
        assert!(
            matches!(err, DeployError::FileNotFound { ref path } if path == &missing),
            "got {err:?}"
        );
    }

    #[tokio::test]
    async fn test_verify_boot_propagates_file_not_found_when_log_never_appears() {
        let missing = std::path::PathBuf::from("/nonexistent/verify-boot-missing-xyz.log");
        let config = VerifyConfig {
            serial_log: missing.clone(),
            rootfs_dir: None,
            boot_timeout: 1,
            expected_boot_message: "never appears".to_string(),
            retry_interval: Duration::from_millis(50),
        };
        let verifier = VmVerifier::new(config);
        let err = verifier.verify_boot().await.expect_err("expected error");
        assert!(
            matches!(err, DeployError::FileNotFound { ref path } if path == &missing),
            "got {err:?}"
        );
    }

    #[tokio::test]
    async fn test_check_primals_missing_bin_dir_returns_empty() {
        let tmp = TempDir::new().unwrap();
        let config = VerifyConfig {
            serial_log: tmp.path().join("unused.log"),
            rootfs_dir: Some(tmp.path().to_path_buf()),
            boot_timeout: 5,
            expected_boot_message: String::new(),
            retry_interval: Duration::ZERO,
        };
        let verifier = VmVerifier::new(config);
        let (count, names) = verifier
            .check_primals(tmp.path())
            .await
            .expect("check_primals");
        assert_eq!(count, 0);
        assert!(names.is_empty());
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn test_check_primals_lists_executable_files() {
        use std::os::unix::fs::PermissionsExt;

        let tmp = TempDir::new().unwrap();
        let bin_dir = tmp.path().join("usr/local/bin");
        std::fs::create_dir_all(&bin_dir).unwrap();
        let p = bin_dir.join("test-primal-verify");
        std::fs::write(&p, b"#! /bin/sh\nexit 0\n").unwrap();
        let mut perms = std::fs::metadata(&p).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&p, perms).unwrap();

        let config = VerifyConfig {
            serial_log: tmp.path().join("unused.log"),
            rootfs_dir: Some(tmp.path().to_path_buf()),
            boot_timeout: 5,
            expected_boot_message: String::new(),
            retry_interval: Duration::ZERO,
        };
        let verifier = VmVerifier::new(config);
        let (count, names) = verifier.check_primals(tmp.path()).await.unwrap();
        assert_eq!(count, 1);
        assert_eq!(names, vec!["test-primal-verify".to_string()]);
    }
}
