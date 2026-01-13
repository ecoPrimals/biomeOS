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
                .map(|m| format!(": {}", m))
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
                format!("Directory: {}", dir),
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
                        Some(format!("Expected 0600, found {:o}", mode))
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
                if !uses_file_ref {
                    Some("Should reference BEARDOG_FAMILY_SEED_FILE".to_string())
                } else {
                    None
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
                        format!("{} executable", name),
                        is_executable,
                        if !is_executable {
                            Some("Not executable".to_string())
                        } else {
                            None
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

    #[tokio::test]
    async fn test_verify_empty_directory() {
        let temp_dir = TempDir::new().unwrap();
        let result = SporeVerification::verify(temp_dir.path()).await.unwrap();

        assert!(!result.valid); // Should fail - no structure
        assert!(!result.checks.is_empty());
    }

    #[test]
    fn test_verification_result() {
        let mut result = VerificationResult::new();
        assert!(result.valid);

        result.add_check("test1", true, None);
        assert!(result.valid);

        result.add_check("test2", false, Some("Failed".to_string()));
        assert!(!result.valid);

        assert_eq!(result.checks.len(), 2);
    }
}
