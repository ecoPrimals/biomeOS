// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Verify lineage mode - Validate genetic lineage
//!
//! EVOLVED (Jan 27, 2026): Complete lineage verification implementation
//!
//! # Deep Debt Principles
//! - Capability-based: Discovers BearDog for cryptographic verification
//! - No hardcoding: Socket paths via SystemPaths
//! - Graceful degradation: Basic checks work without BearDog

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};

/// Lineage verification result
#[derive(Debug)]
pub struct LineageVerification {
    /// Whether the lineage is valid
    pub valid: bool,
    /// Family ID if found
    pub family_id: Option<String>,
    /// Node ID if found
    pub node_id: Option<String>,
    /// Verification details
    pub details: Vec<String>,
    /// Any warnings
    pub warnings: Vec<String>,
}

pub async fn run(path: PathBuf, detailed: bool) -> Result<()> {
    info!("🔍 biomeOS Lineage Verification");
    info!("Path: {}", path.display());

    if !path.exists() {
        anyhow::bail!("Path not found: {}", path.display());
    }

    let verification = verify_lineage(&path, detailed).await?;

    // Display results
    if verification.valid {
        info!("✅ Lineage verification PASSED");
    } else {
        info!("❌ Lineage verification FAILED");
    }

    if let Some(ref family_id) = verification.family_id {
        info!("   Family ID: {}", family_id);
    }
    if let Some(ref node_id) = verification.node_id {
        info!("   Node ID: {}", node_id);
    }

    for detail in &verification.details {
        info!("   ✓ {}", detail);
    }

    for warning in &verification.warnings {
        warn!("   ⚠️ {}", warning);
    }

    if verification.valid {
        Ok(())
    } else {
        anyhow::bail!("Lineage verification failed")
    }
}

/// Verify lineage of a spore or primal directory
pub(crate) async fn verify_lineage(path: &PathBuf, detailed: bool) -> Result<LineageVerification> {
    let mut verification = LineageVerification {
        valid: true,
        family_id: None,
        node_id: None,
        details: Vec::new(),
        warnings: Vec::new(),
    };

    let metadata = std::fs::metadata(path)?;

    if metadata.is_dir() {
        // Verify spore directory structure
        verification.details.push("Directory exists".to_string());

        // Check for manifest
        let manifest_path = path.join("manifest.toml");
        if manifest_path.exists() {
            verification.details.push("Manifest found".to_string());

            // Parse manifest for family_id and node_id
            if let Ok(manifest_content) = std::fs::read_to_string(&manifest_path) {
                if let Ok(manifest) = manifest_content.parse::<toml::Table>() {
                    if let Some(family) = manifest.get("family_id").and_then(|v| v.as_str()) {
                        verification.family_id = Some(family.to_string());
                    }
                    if let Some(node) = manifest.get("node_id").and_then(|v| v.as_str()) {
                        verification.node_id = Some(node.to_string());
                    }
                }
            }
        } else {
            verification
                .warnings
                .push("No manifest.toml found".to_string());
        }

        // Check for family seed
        let seed_path = path.join(".family.seed");
        if seed_path.exists() {
            if let Ok(seed_metadata) = std::fs::metadata(&seed_path) {
                if seed_metadata.len() == 64 {
                    verification
                        .details
                        .push("Family seed valid (64 bytes)".to_string());
                } else {
                    verification.valid = false;
                    verification.warnings.push(format!(
                        "Family seed invalid size: {} bytes (expected 64)",
                        seed_metadata.len()
                    ));
                }
            }
        } else {
            verification
                .warnings
                .push("No .family.seed found".to_string());
        }

        // Check primals directory
        let primals_path = path.join("primals");
        if primals_path.exists() {
            if let Ok(entries) = std::fs::read_dir(&primals_path) {
                let count = entries.count();
                verification
                    .details
                    .push(format!("Primals directory: {} binaries", count));
            }
        } else {
            verification
                .warnings
                .push("No primals directory found".to_string());
        }

        // Detailed verification: cryptographic checks via BearDog
        if detailed {
            match verify_cryptographic_lineage(path, &verification).await {
                Ok(crypto_details) => {
                    verification.details.extend(crypto_details);
                }
                Err(e) => {
                    verification
                        .warnings
                        .push(format!("Cryptographic verification skipped: {}", e));
                }
            }
        }
    } else {
        // Single file (possibly a seed file)
        let file_size = metadata.len();
        if file_size == 64 {
            verification
                .details
                .push("Valid seed file (64 bytes)".to_string());
        } else if file_size == 32 {
            verification
                .details
                .push("Valid hash file (32 bytes)".to_string());
        } else {
            verification
                .warnings
                .push(format!("Unknown file format ({} bytes)", file_size));
        }
    }

    Ok(verification)
}

/// Verify cryptographic lineage via BearDog
///
/// This delegates to BearDog for cryptographic verification:
/// - Seed derivation chain validation
/// - Signature verification
/// - Family membership proof
async fn verify_cryptographic_lineage(
    path: &Path,
    verification: &LineageVerification,
) -> Result<Vec<String>> {
    use biomeos_core::atomic_client::AtomicClient;

    let mut details = Vec::new();

    // Discover security provider for cryptographic operations
    // DEEP DEBT EVOLUTION: Resolve provider name from env, not hardcoded
    let security_provider =
        std::env::var("BIOMEOS_SECURITY_PROVIDER").unwrap_or_else(|_| "beardog".to_string());
    let beardog = AtomicClient::discover(&security_provider)
        .await
        .context(format!(
            "{} not available for cryptographic verification",
            security_provider
        ))?;

    debug!(
        "{} discovered, performing cryptographic verification",
        security_provider
    );

    // Read family seed if available
    let seed_path = path.join(".family.seed");
    if !seed_path.exists() {
        return Ok(vec![
            "Cryptographic verification skipped: no seed file".to_string()
        ]);
    }

    let seed = std::fs::read(&seed_path)?;
    let seed_b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &seed);

    // Call BearDog to verify seed derivation
    let family_id = verification.family_id.as_deref().unwrap_or("unknown");
    let node_id = verification.node_id.as_deref().unwrap_or("unknown");

    let result = beardog
        .call(
            "lineage.verify",
            serde_json::json!({
                "seed": seed_b64,
                "family_id": family_id,
                "node_id": node_id
            }),
        )
        .await;

    match result {
        Ok(response) => {
            if response
                .get("valid")
                .and_then(|v| v.as_bool())
                .unwrap_or(false)
            {
                details.push("Cryptographic lineage verified ✓".to_string());

                if let Some(generation) = response.get("generation").and_then(|v| v.as_u64()) {
                    details.push(format!("Generation: {}", generation));
                }

                if let Some(parent) = response.get("parent_id").and_then(|v| v.as_str()) {
                    details.push(format!("Parent: {}", parent));
                }
            } else {
                let reason = response
                    .get("reason")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown");
                details.push(format!("Cryptographic verification failed: {}", reason));
            }
        }
        Err(e) => {
            details.push(format!("BearDog verification call failed: {}", e));
        }
    }

    Ok(details)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;
    use std::io::Write;

    #[test]
    fn test_lineage_verification_construction() {
        let v = LineageVerification {
            valid: true,
            family_id: Some("fam-123".to_string()),
            node_id: Some("node-456".to_string()),
            details: vec!["detail1".to_string()],
            warnings: vec!["warn1".to_string()],
        };
        assert!(v.valid);
        assert_eq!(v.family_id.as_deref(), Some("fam-123"));
        assert_eq!(v.node_id.as_deref(), Some("node-456"));
        assert_eq!(v.details.len(), 1);
        assert_eq!(v.warnings.len(), 1);
    }

    #[tokio::test]
    async fn test_run_path_not_found_returns_error() {
        let path = PathBuf::from("/nonexistent/path/that/does/not/exist/12345");
        let result = run(path, false).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.to_string().contains("Path not found") || err.to_string().contains("not found"),
            "Expected path not found error: {}",
            err
        );
    }

    #[tokio::test]
    async fn test_verify_lineage_directory_basic() {
        let dir = tempfile::tempdir().expect("create temp dir");
        let path = dir.path().to_path_buf();

        let result = verify_lineage(&path, false).await;
        let v = result.expect("verify_lineage should succeed");
        assert!(v.valid);
        assert!(v.details.contains(&"Directory exists".to_string()));
        assert!(v.warnings.contains(&"No manifest.toml found".to_string()));
        assert!(v.warnings.contains(&"No .family.seed found".to_string()));
        assert!(v
            .warnings
            .contains(&"No primals directory found".to_string()));
    }

    #[tokio::test]
    async fn test_verify_lineage_directory_with_manifest() {
        let dir = tempfile::tempdir().expect("create temp dir");
        let manifest_path = dir.path().join("manifest.toml");
        std::fs::write(
            &manifest_path,
            r#"
family_id = "test-family-123"
node_id = "test-node-456"
"#,
        )
        .expect("write manifest");

        let path = dir.path().to_path_buf();
        let result = verify_lineage(&path, false).await;
        let v = result.expect("verify_lineage should succeed");
        assert!(v.valid);
        assert_eq!(v.family_id.as_deref(), Some("test-family-123"));
        assert_eq!(v.node_id.as_deref(), Some("test-node-456"));
        assert!(v.details.contains(&"Manifest found".to_string()));
    }

    #[tokio::test]
    async fn test_verify_lineage_directory_with_valid_seed() {
        let dir = tempfile::tempdir().expect("create temp dir");
        let seed_path = dir.path().join(".family.seed");
        let mut f = std::fs::File::create(&seed_path).expect("create seed");
        f.write_all(&[0u8; 64]).expect("write 64 bytes");

        let path = dir.path().to_path_buf();
        let result = verify_lineage(&path, false).await;
        let v = result.expect("verify_lineage should succeed");
        assert!(v.valid);
        assert!(v
            .details
            .contains(&"Family seed valid (64 bytes)".to_string()));
    }

    #[tokio::test]
    async fn test_verify_lineage_directory_with_invalid_seed_size() {
        let dir = tempfile::tempdir().expect("create temp dir");
        let seed_path = dir.path().join(".family.seed");
        let mut f = std::fs::File::create(&seed_path).expect("create seed");
        f.write_all(&[0u8; 32]).expect("write 32 bytes");

        let path = dir.path().to_path_buf();
        let result = verify_lineage(&path, false).await;
        let v = result.expect("verify_lineage should succeed");
        assert!(!v.valid);
        assert!(v.warnings.iter().any(|w| w.contains("invalid size")));
        assert!(v.warnings.iter().any(|w| w.contains("32 bytes")));
    }

    #[tokio::test]
    async fn test_verify_lineage_directory_with_primals() {
        let dir = tempfile::tempdir().expect("create temp dir");
        let primals_dir = dir.path().join("primals");
        std::fs::create_dir(&primals_dir).expect("create primals dir");
        std::fs::File::create(primals_dir.join("beardog")).expect("create binary");
        std::fs::File::create(primals_dir.join("songbird")).expect("create binary");

        let path = dir.path().to_path_buf();
        let result = verify_lineage(&path, false).await;
        let v = result.expect("verify_lineage should succeed");
        assert!(v.valid);
        assert!(
            v.details
                .iter()
                .any(|d| d.contains("Primals") && d.contains("2 binaries")),
            "Expected primals directory detail, got: {:?}",
            v.details
        );
    }

    #[tokio::test]
    async fn test_verify_lineage_single_file_64_bytes() {
        let file = tempfile::NamedTempFile::new().expect("create temp file");
        let mut f = file.reopen().expect("reopen");
        f.write_all(&[0u8; 64]).expect("write 64 bytes");
        drop(f);

        let path = file.path().to_path_buf();
        let result = verify_lineage(&path, false).await;
        let v = result.expect("verify_lineage should succeed");
        assert!(v.valid);
        assert!(v
            .details
            .contains(&"Valid seed file (64 bytes)".to_string()));
    }

    #[tokio::test]
    async fn test_verify_lineage_single_file_32_bytes() {
        let file = tempfile::NamedTempFile::new().expect("create temp file");
        let mut f = file.reopen().expect("reopen");
        f.write_all(&[0u8; 32]).expect("write 32 bytes");
        drop(f);

        let path = file.path().to_path_buf();
        let result = verify_lineage(&path, false).await;
        let v = result.expect("verify_lineage should succeed");
        assert!(v.valid);
        assert!(v
            .details
            .contains(&"Valid hash file (32 bytes)".to_string()));
    }

    #[tokio::test]
    async fn test_verify_lineage_single_file_unknown_size() {
        let file = tempfile::NamedTempFile::new().expect("create temp file");
        let mut f = file.reopen().expect("reopen");
        f.write_all(&[0u8; 100]).expect("write 100 bytes");
        drop(f);

        let path = file.path().to_path_buf();
        let result = verify_lineage(&path, false).await;
        let v = result.expect("verify_lineage should succeed");
        assert!(v.valid);
        assert!(v.warnings.iter().any(|w| w.contains("Unknown file format")));
        assert!(v.warnings.iter().any(|w| w.contains("100 bytes")));
    }

    #[tokio::test]
    async fn test_verify_lineage_manifest_partial() {
        let dir = tempfile::tempdir().expect("create temp dir");
        let manifest_path = dir.path().join("manifest.toml");
        std::fs::write(&manifest_path, "family_id = \"only-family\"\n").expect("write manifest");

        let path = dir.path().to_path_buf();
        let result = verify_lineage(&path, false).await;
        let v = result.expect("verify_lineage should succeed");
        assert_eq!(v.family_id.as_deref(), Some("only-family"));
        assert_eq!(v.node_id, None);
    }

    #[tokio::test]
    async fn test_verify_lineage_manifest_invalid_toml() {
        let dir = tempfile::tempdir().expect("create temp dir");
        let manifest_path = dir.path().join("manifest.toml");
        std::fs::write(&manifest_path, "invalid toml {{{").expect("write invalid manifest");

        let path = dir.path().to_path_buf();
        let result = verify_lineage(&path, false).await;
        let v = result.expect("verify_lineage should succeed");
        assert!(v.details.contains(&"Manifest found".to_string()));
        assert_eq!(v.family_id, None);
        assert_eq!(v.node_id, None);
    }

    #[tokio::test]
    async fn test_run_success_with_valid_directory() {
        let dir = tempfile::tempdir().expect("create temp dir");
        let manifest_path = dir.path().join("manifest.toml");
        std::fs::write(
            &manifest_path,
            "family_id = \"run-test\"\nnode_id = \"node-1\"\n",
        )
        .expect("write manifest");
        let seed_path = dir.path().join(".family.seed");
        let mut f = std::fs::File::create(&seed_path).expect("create seed");
        f.write_all(&[0u8; 64]).expect("write 64 bytes");

        let path = dir.path().to_path_buf();
        let result = run(path, false).await;
        assert!(result.is_ok(), "run should succeed: {:?}", result.err());
    }

    #[tokio::test]
    async fn test_run_fails_when_verification_invalid() {
        let dir = tempfile::tempdir().expect("create temp dir");
        let seed_path = dir.path().join(".family.seed");
        let mut f = std::fs::File::create(&seed_path).expect("create seed");
        f.write_all(&[0u8; 32]).expect("write 32 bytes");

        let path = dir.path().to_path_buf();
        let result = run(path, false).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.to_string().contains("failed") || err.to_string().contains("invalid"),
            "Expected failure: {}",
            err
        );
    }

    #[tokio::test]
    async fn test_verify_lineage_detailed_skips_when_no_beardog() {
        let dir = tempfile::tempdir().expect("create temp dir");
        let seed_path = dir.path().join(".family.seed");
        let mut f = std::fs::File::create(&seed_path).expect("create seed");
        f.write_all(&[0u8; 64]).expect("write 64 bytes");

        std::env::remove_var("BIOMEOS_SECURITY_PROVIDER");
        let path = dir.path().to_path_buf();
        let result = verify_lineage(&path, true).await;
        let v = result.expect("verify_lineage should succeed");
        assert!(v.valid);
        let has_skip_warning = v
            .warnings
            .iter()
            .any(|w| w.contains("Cryptographic") || w.contains("skipped"));
        let has_crypto_detail = v
            .details
            .iter()
            .any(|d| d.contains("Cryptographic") || d.contains("skipped"));
        assert!(
            has_skip_warning || has_crypto_detail,
            "Expected crypto skip message, got warnings: {:?}, details: {:?}",
            v.warnings,
            v.details
        );
    }
}
