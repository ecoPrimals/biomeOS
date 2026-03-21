// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Spore verification and integrity checking

use std::path::Path;
use tokio::fs;
use tracing::info;

use crate::error::SporeResult;

/// Result of spore verification
#[derive(Debug, Clone)]
pub struct VerificationResult {
    /// Overall status
    pub valid: bool,

    /// Individual checks
    pub checks: Vec<VerificationCheck>,
}

/// Individual verification check
#[derive(Debug, Clone)]
pub struct VerificationCheck {
    /// Name of the check
    pub name: String,

    /// Whether it passed
    pub passed: bool,

    /// Optional message
    pub message: Option<String>,
}

impl VerificationResult {
    /// Create a new empty result
    fn new() -> Self {
        Self {
            valid: true,
            checks: Vec::new(),
        }
    }

    /// Add a check result
    fn add_check(&mut self, name: impl Into<String>, passed: bool, message: Option<String>) {
        if !passed {
            self.valid = false;
        }
        self.checks.push(VerificationCheck {
            name: name.into(),
            passed,
            message,
        });
    }

    /// Print a summary
    pub fn print_summary(&self) {
        if self.valid {
            println!("✅ Spore verification PASSED");
        } else {
            println!("❌ Spore verification FAILED");
        }

        for check in &self.checks {
            let icon = if check.passed { "✅" } else { "❌" };
            let msg = check
                .message
                .as_ref()
                .map(|m| format!(": {m}"))
                .unwrap_or_default();
            println!("  {} {}{}", icon, check.name, msg);
        }
    }
}

/// Spore verification utility
pub struct SporeVerification;

impl SporeVerification {
    /// Verify a spore's integrity
    ///
    /// Checks:
    /// - Directory structure exists
    /// - Family seed file present and correct size
    /// - Configuration file present
    /// - Binaries present
    /// - Permissions correct
    pub async fn verify(spore_path: &Path) -> SporeResult<VerificationResult> {
        info!("Verifying spore at: {}", spore_path.display());

        let mut result = VerificationResult::new();

        // Check root directory exists
        result.add_check(
            "Root directory",
            spore_path.exists(),
            if spore_path.exists() {
                None
            } else {
                Some(format!("Not found: {}", spore_path.display()))
            },
        );

        if !spore_path.exists() {
            return Ok(result);
        }

        // Check directory structure
        Self::check_directory_structure(spore_path, &mut result).await;

        // Check family seed
        Self::check_family_seed(spore_path, &mut result).await;

        // Check configuration
        Self::check_configuration(spore_path, &mut result).await;

        // Check binaries
        Self::check_binaries(spore_path, &mut result).await;

        info!(
            "Verification complete: {}",
            if result.valid { "PASSED" } else { "FAILED" }
        );

        Ok(result)
    }

    async fn check_directory_structure(spore_path: &Path, result: &mut VerificationResult) {
        let required_dirs = ["bin", "primals", "secrets", "logs"];

        for dir in &required_dirs {
            let path = spore_path.join(dir);
            result.add_check(
                format!("Directory: {dir}"),
                path.exists() && path.is_dir(),
                if !path.exists() {
                    Some("Missing".to_string())
                } else if !path.is_dir() {
                    Some("Not a directory".to_string())
                } else {
                    None
                },
            );
        }
    }

    async fn check_family_seed(spore_path: &Path, result: &mut VerificationResult) {
        let seed_path = spore_path.join(".family.seed");

        // Check exists
        if !seed_path.exists() {
            result.add_check("Family seed", false, Some("File not found".to_string()));
            return;
        }

        // Check size
        if let Ok(metadata) = fs::metadata(&seed_path).await {
            let correct_size = metadata.len() == 32;
            result.add_check(
                "Family seed size",
                correct_size,
                if correct_size {
                    None
                } else {
                    Some(format!("Expected 32 bytes, found {}", metadata.len()))
                },
            );

            // Check permissions (Unix only)
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let perms = metadata.permissions();
                let mode = perms.mode() & 0o777;
                let secure = mode == 0o600;

                result.add_check(
                    "Family seed permissions",
                    secure,
                    if secure {
                        None
                    } else {
                        Some(format!("Expected 0600, found {mode:o}"))
                    },
                );
            }
        } else {
            result.add_check(
                "Family seed metadata",
                false,
                Some("Cannot read metadata".to_string()),
            );
        }
    }

    async fn check_configuration(spore_path: &Path, result: &mut VerificationResult) {
        let config_path = spore_path.join("tower.toml");

        if !config_path.exists() {
            result.add_check(
                "Configuration",
                false,
                Some("tower.toml not found".to_string()),
            );
            return;
        }

        // Check content
        if let Ok(content) = fs::read_to_string(&config_path).await {
            // Verify it references BEARDOG_FAMILY_SEED_FILE (not raw seed)
            let uses_file_ref = content.contains("BEARDOG_FAMILY_SEED_FILE");
            let has_raw_seed = content.contains("BEARDOG_FAMILY_SEED =")
                && !content.contains("BEARDOG_FAMILY_SEED_FILE");

            result.add_check(
                "Config uses seed file",
                uses_file_ref,
                if uses_file_ref {
                    None
                } else {
                    Some("Should reference BEARDOG_FAMILY_SEED_FILE".to_string())
                },
            );

            result.add_check(
                "Config not exposing raw seed",
                !has_raw_seed,
                if has_raw_seed {
                    Some("Should use BEARDOG_FAMILY_SEED_FILE, not raw seed".to_string())
                } else {
                    None
                },
            );
        } else {
            result.add_check(
                "Configuration readable",
                false,
                Some("Cannot read tower.toml".to_string()),
            );
        }
    }

    async fn check_binaries(spore_path: &Path, result: &mut VerificationResult) {
        let binaries = [
            ("primals/beardog", "BearDog binary"),
            ("primals/songbird", "Songbird binary"),
            ("bin/tower", "Tower binary"),
        ];

        for (path, name) in &binaries {
            let full_path = spore_path.join(path);

            if !full_path.exists() {
                result.add_check(*name, false, Some("Not found".to_string()));
                continue;
            }

            // Check executable (Unix only)
            #[cfg(unix)]
            {
                if let Ok(metadata) = fs::metadata(&full_path).await {
                    use std::os::unix::fs::PermissionsExt;
                    let perms = metadata.permissions();
                    let is_executable = perms.mode() & 0o111 != 0;

                    result.add_check(
                        format!("{name} executable"),
                        is_executable,
                        if is_executable {
                            None
                        } else {
                            Some("Not executable".to_string())
                        },
                    );
                }
            }

            #[cfg(not(unix))]
            result.add_check(name, true, None);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // ========== VerificationResult Tests ==========

    #[test]
    fn test_verification_result_new() {
        let result = VerificationResult::new();
        assert!(result.valid);
        assert!(result.checks.is_empty());
    }

    #[test]
    fn test_verification_result_passing_checks() {
        let mut result = VerificationResult::new();

        result.add_check("test1", true, None);
        result.add_check("test2", true, Some("All good".to_string()));

        assert!(result.valid);
        assert_eq!(result.checks.len(), 2);
        assert!(result.checks[0].passed);
        assert!(result.checks[1].passed);
        assert!(result.checks[0].message.is_none());
        assert_eq!(result.checks[1].message, Some("All good".to_string()));
    }

    #[test]
    fn test_verification_result_failing_check_invalidates() {
        let mut result = VerificationResult::new();
        assert!(result.valid);

        result.add_check("pass", true, None);
        assert!(result.valid);

        result.add_check("fail", false, Some("Failed".to_string()));
        assert!(!result.valid);

        // Adding another pass doesn't restore validity
        result.add_check("pass2", true, None);
        assert!(!result.valid);

        assert_eq!(result.checks.len(), 3);
    }

    #[test]
    fn test_verification_result_clone() {
        let mut result = VerificationResult::new();
        result.add_check("test", true, None);

        let cloned = result.clone();
        assert_eq!(cloned.valid, result.valid);
        assert_eq!(cloned.checks.len(), result.checks.len());
    }

    #[test]
    fn test_verification_check_clone() {
        let check = VerificationCheck {
            name: "Family seed".to_string(),
            passed: true,
            message: Some("32 bytes OK".to_string()),
        };
        let cloned = check.clone();
        assert_eq!(cloned.name, check.name);
        assert_eq!(cloned.passed, check.passed);
        assert_eq!(cloned.message, check.message);
    }

    // ========== SporeVerification Tests ==========

    #[tokio::test]
    async fn test_verify_nonexistent_path() {
        let result = SporeVerification::verify(Path::new("/nonexistent/path/abc123"))
            .await
            .expect("verify should succeed even for missing path");

        assert!(!result.valid);
        assert_eq!(result.checks.len(), 1);
        assert_eq!(result.checks[0].name, "Root directory");
        assert!(!result.checks[0].passed);
    }

    #[tokio::test]
    async fn test_verify_empty_directory() {
        let temp_dir = TempDir::new().expect("create temp dir");
        let result = SporeVerification::verify(temp_dir.path())
            .await
            .expect("verify");

        assert!(!result.valid);
        // Root exists, but dirs/seed/config/binaries missing
        assert!(result.checks.len() > 1);

        // Root directory check should pass
        let root_check = result.checks.iter().find(|c| c.name == "Root directory");
        assert!(root_check.expect("root check").passed);
    }

    #[tokio::test]
    async fn test_verify_directory_structure_checks() {
        let temp_dir = TempDir::new().expect("create temp dir");
        let spore = temp_dir.path();

        // Create partial structure
        std::fs::create_dir_all(spore.join("bin")).expect("create bin");
        std::fs::create_dir_all(spore.join("primals")).expect("create primals");
        // Missing: secrets, logs

        let result = SporeVerification::verify(spore).await.expect("verify");

        // bin and primals should pass, secrets and logs should fail
        let bin_check = result.checks.iter().find(|c| c.name == "Directory: bin");
        assert!(bin_check.expect("bin check").passed);

        let secrets_check = result
            .checks
            .iter()
            .find(|c| c.name == "Directory: secrets");
        assert!(!secrets_check.expect("secrets check").passed);
    }

    #[tokio::test]
    async fn test_verify_family_seed_missing() {
        let temp_dir = TempDir::new().expect("create temp dir");
        let spore = temp_dir.path();

        // Create all required dirs but no seed
        for dir in &["bin", "primals", "secrets", "logs"] {
            std::fs::create_dir_all(spore.join(dir)).expect("create dir");
        }

        let result = SporeVerification::verify(spore).await.expect("verify");

        let seed_check = result.checks.iter().find(|c| c.name == "Family seed");
        assert!(!seed_check.expect("seed check").passed);
    }

    #[tokio::test]
    async fn test_verify_family_seed_correct_size() {
        let temp_dir = TempDir::new().expect("create temp dir");
        let spore = temp_dir.path();

        for dir in &["bin", "primals", "secrets", "logs"] {
            std::fs::create_dir_all(spore.join(dir)).expect("create dir");
        }

        // Write correct 32-byte seed
        std::fs::write(spore.join(".family.seed"), [0u8; 32]).expect("write seed");

        let result = SporeVerification::verify(spore).await.expect("verify");

        let size_check = result.checks.iter().find(|c| c.name == "Family seed size");
        assert!(size_check.expect("seed size check").passed);
    }

    #[tokio::test]
    async fn test_verify_family_seed_wrong_size() {
        let temp_dir = TempDir::new().expect("create temp dir");
        let spore = temp_dir.path();

        for dir in &["bin", "primals", "secrets", "logs"] {
            std::fs::create_dir_all(spore.join(dir)).expect("create dir");
        }

        // Write wrong-size seed
        std::fs::write(spore.join(".family.seed"), [0u8; 16]).expect("write seed");

        let result = SporeVerification::verify(spore).await.expect("verify");

        let size_check = result.checks.iter().find(|c| c.name == "Family seed size");
        assert!(!size_check.expect("seed size check").passed);
        assert!(
            size_check
                .expect("seed size check message")
                .message
                .as_ref()
                .expect("message")
                .contains("16")
        );
    }

    #[tokio::test]
    async fn test_verify_config_missing() {
        let temp_dir = TempDir::new().expect("create temp dir");
        let spore = temp_dir.path();

        for dir in &["bin", "primals", "secrets", "logs"] {
            std::fs::create_dir_all(spore.join(dir)).expect("create dir");
        }
        std::fs::write(spore.join(".family.seed"), [0u8; 32]).expect("write seed");

        let result = SporeVerification::verify(spore).await.expect("verify");

        let config_check = result.checks.iter().find(|c| c.name == "Configuration");
        assert!(!config_check.expect("config check").passed);
    }

    #[tokio::test]
    async fn test_verify_config_uses_seed_file() {
        let temp_dir = TempDir::new().expect("create temp dir");
        let spore = temp_dir.path();

        for dir in &["bin", "primals", "secrets", "logs"] {
            std::fs::create_dir_all(spore.join(dir)).expect("create dir");
        }
        std::fs::write(spore.join(".family.seed"), [0u8; 32]).expect("write seed");

        // Write tower.toml that references BEARDOG_FAMILY_SEED_FILE
        let config_content = r#"
[tower]
name = "test"

[environment]
BEARDOG_FAMILY_SEED_FILE = "/biomeOS/secrets/.family.seed"
"#;
        std::fs::write(spore.join("tower.toml"), config_content).expect("write config");

        let result = SporeVerification::verify(spore).await.expect("verify");

        let seed_ref_check = result
            .checks
            .iter()
            .find(|c| c.name == "Config uses seed file");
        assert!(seed_ref_check.expect("seed ref check").passed);

        let raw_seed_check = result
            .checks
            .iter()
            .find(|c| c.name == "Config not exposing raw seed");
        assert!(raw_seed_check.expect("raw seed check").passed);
    }

    #[tokio::test]
    async fn test_verify_binaries_missing() {
        let temp_dir = TempDir::new().expect("create temp dir");
        let spore = temp_dir.path();

        for dir in &["bin", "primals", "secrets", "logs"] {
            std::fs::create_dir_all(spore.join(dir)).expect("create dir");
        }
        std::fs::write(spore.join(".family.seed"), [0u8; 32]).expect("write seed");
        std::fs::write(
            spore.join("tower.toml"),
            "BEARDOG_FAMILY_SEED_FILE = \"/secrets/.family.seed\"\n",
        )
        .expect("write config");

        let result = SporeVerification::verify(spore).await.expect("verify");

        // All binary checks should fail
        let binary_checks: Vec<_> = result
            .checks
            .iter()
            .filter(|c| {
                c.name == "BearDog binary"
                    || c.name == "Songbird binary"
                    || c.name == "Tower binary"
            })
            .collect();

        for check in &binary_checks {
            assert!(!check.passed, "Binary {} should be missing", check.name);
        }
    }

    #[tokio::test]
    async fn test_verify_complete_spore() {
        let temp_dir = TempDir::new().expect("create temp dir");
        let spore = temp_dir.path();

        // Create complete structure
        for dir in &["bin", "primals", "secrets", "logs"] {
            std::fs::create_dir_all(spore.join(dir)).expect("create dir");
        }
        std::fs::write(spore.join(".family.seed"), [0u8; 32]).expect("write seed");
        std::fs::write(
            spore.join("tower.toml"),
            "BEARDOG_FAMILY_SEED_FILE = \"/secrets/.family.seed\"\n",
        )
        .expect("write config");

        // Create executable binaries
        std::fs::write(spore.join("bin/tower"), b"tower binary").expect("write tower");
        std::fs::write(spore.join("primals/beardog"), b"beardog binary").expect("write beardog");
        std::fs::write(spore.join("primals/songbird"), b"songbird binary").expect("write songbird");

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            for path in &["bin/tower", "primals/beardog", "primals/songbird"] {
                let mut perms = std::fs::metadata(spore.join(path))
                    .expect("metadata")
                    .permissions();
                perms.set_mode(0o755);
                std::fs::set_permissions(spore.join(path), perms).expect("set perms");
            }
        }

        let result = SporeVerification::verify(spore).await.expect("verify");

        // All directory and seed checks should pass
        let dir_checks: Vec<_> = result
            .checks
            .iter()
            .filter(|c| c.name.starts_with("Directory:"))
            .collect();
        for check in &dir_checks {
            assert!(check.passed, "Directory check {} should pass", check.name);
        }
    }

    // ========== print_summary tests ==========

    #[test]
    fn test_print_summary_passing() {
        let mut result = VerificationResult::new();
        result.add_check("test1", true, None);
        result.add_check("test2", true, Some("All good".to_string()));

        // Just ensure it doesn't panic
        result.print_summary();
    }

    #[test]
    fn test_print_summary_failing() {
        let mut result = VerificationResult::new();
        result.add_check("pass", true, None);
        result.add_check("fail", false, Some("Missing".to_string()));

        // Just ensure it doesn't panic
        result.print_summary();
    }
}
