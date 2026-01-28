//! Verify lineage mode - Validate genetic lineage
//!
//! EVOLVED (Jan 27, 2026): Complete lineage verification implementation
//!
//! # Deep Debt Principles
//! - Capability-based: Discovers BearDog for cryptographic verification
//! - No hardcoding: Socket paths via SystemPaths
//! - Graceful degradation: Basic checks work without BearDog

use anyhow::{Context, Result};
use std::path::PathBuf;
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
async fn verify_lineage(path: &PathBuf, detailed: bool) -> Result<LineageVerification> {
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
    path: &PathBuf,
    verification: &LineageVerification,
) -> Result<Vec<String>> {
    use biomeos_core::atomic_client::AtomicClient;

    let mut details = Vec::new();

    // Discover BearDog for cryptographic operations
    let beardog = AtomicClient::discover("beardog")
        .await
        .context("BearDog not available for cryptographic verification")?;

    debug!("BearDog discovered, performing cryptographic verification");

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
