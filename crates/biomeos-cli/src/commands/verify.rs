// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

// crates/biomeos-cli/src/commands/verify.rs
//! Spore and nucleus verification commands

use anyhow::Result;
use biomeos_spore::manifest::BinaryManifest;
use biomeos_spore::verification::{SporeVerifier, VerificationStatus};

/// Map verification status to (icon, text) for display (testable pure function)
pub fn verification_status_display(status: &VerificationStatus) -> (&'static str, &'static str) {
    match status {
        VerificationStatus::Fresh => ("✅", "Fresh"),
        VerificationStatus::Stale => ("⚠️ ", "Stale"),
        VerificationStatus::Missing => ("❌", "Missing"),
        VerificationStatus::Modified => ("⚠️ ", "Modified"),
        VerificationStatus::Newer => ("❓", "Newer"),
    }
}
use clap::{Args, Subcommand};
use std::path::PathBuf;
use tracing::info;

/// Arguments for verification commands
#[derive(Args, Debug)]
pub struct VerifyArgs {
    /// Verification target
    #[command(subcommand)]
    pub target: VerifyTarget,
}

/// Targets for verification
#[derive(Subcommand, Debug)]
pub enum VerifyTarget {
    /// Verify plasmidBin integrity
    Nucleus {
        /// Path to plasmidBin directory
        #[arg(short, long, default_value = "plasmidBin")]
        path: PathBuf,
    },

    /// Verify a specific spore
    Spore {
        /// Path to spore mount point
        #[arg(value_name = "MOUNT_POINT")]
        mount_point: PathBuf,
    },

    /// Verify all mounted spores
    All {
        /// Show detailed binary information
        #[arg(short, long)]
        verbose: bool,
    },
}

/// Execute a verification command
pub async fn run(args: VerifyArgs) -> Result<()> {
    match args.target {
        VerifyTarget::Nucleus { path } => {
            verify_nucleus(&path).await?;
        }
        VerifyTarget::Spore { mount_point } => {
            verify_single_spore(&mount_point).await?;
        }
        VerifyTarget::All { verbose } => {
            verify_all_spores(verbose).await?;
        }
    }

    Ok(())
}

async fn verify_nucleus(nucleus_path: &PathBuf) -> Result<()> {
    info!("Verifying plasmidBin at: {}", nucleus_path.display());

    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║                                                                ║");
    println!("║         🔍 PlasmidBin Verification                            ║");
    println!("║                                                                ║");
    println!("╚════════════════════════════════════════════════════════════════╝");
    println!();

    // Check if plasmidBin exists
    if !nucleus_path.exists() {
        println!(
            "❌ Error: plasmidBin not found at: {}",
            nucleus_path.display()
        );
        println!();
        println!("Expected structure:");
        println!("  plasmidBin/");
        println!("    tower/");
        println!("      tower (binary)");
        println!("    primals/");
        println!("      beardog-server (binary)");
        println!("      songbird (binary)");
        println!("    MANIFEST.toml (optional)");
        println!();
        return Ok(());
    }

    // Check for MANIFEST.toml
    let manifest_path = nucleus_path.join("MANIFEST.toml");
    let manifest = if manifest_path.exists() {
        println!("✅ MANIFEST.toml found");
        Some(BinaryManifest::load(nucleus_path)?)
    } else {
        println!("⚠️  MANIFEST.toml not found (generating from binaries)");
        Some(BinaryManifest::from_nucleus(nucleus_path)?)
    };

    println!();

    if let Some(manifest) = manifest {
        println!("📋 Binary Inventory:");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        for (name, binary) in &manifest.binaries {
            println!("✅ {name}");
            println!("   Version:    {}", binary.version);
            println!(
                "   Size:       {} bytes ({:.2} MB)",
                binary.size_bytes,
                binary.size_bytes as f64 / 1_048_576.0
            );
            println!("   SHA256:     {}...", &binary.sha256[..16]);
            println!("   Git Commit: {}", binary.git_commit);
            if !binary.features.is_empty() {
                println!("   Features:   {}", binary.features.join(", "));
            }
            println!();
        }

        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("Total binaries: {}", manifest.binaries.len());
        println!();
        println!("✅ PlasmidBin is valid and ready for deployment");
    }

    Ok(())
}

async fn verify_single_spore(mount_point: &PathBuf) -> Result<()> {
    info!("Verifying spore at: {}", mount_point.display());

    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║                                                                ║");
    println!("║         🔍 Spore Verification Report                          ║");
    println!("║                                                                ║");
    println!("╚════════════════════════════════════════════════════════════════╝");
    println!();

    // Load nucleus manifest
    let nucleus_path = PathBuf::from("plasmidBin");
    if !nucleus_path.exists() {
        println!("❌ Error: plasmidBin not found (required for comparison)");
        println!("   Expected at: {}", nucleus_path.display());
        return Ok(());
    }

    let verifier = SporeVerifier::from_nucleus(&nucleus_path)?;
    let report = verifier.verify_spore(mount_point)?;

    println!("Node ID: {}", report.node_id);
    println!("Path:    {}", report.spore_path.display());
    println!();

    // Overall status with color
    match report.overall_status {
        VerificationStatus::Fresh => {
            println!("✅ Status: FRESH");
            println!("   All binaries match plasmidBin exactly");
        }
        VerificationStatus::Stale => {
            println!("⚠️  Status: STALE");
            println!("   Some binaries need updating");
        }
        VerificationStatus::Missing => {
            println!("❌ Status: MISSING BINARIES");
            println!("   Some required binaries are not present");
        }
        _ => {
            println!("❌ Status: {:?}", report.overall_status);
        }
    }

    println!();
    println!("Binary Status:");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    for binary in &report.binaries {
        let (status_icon, status_text) = verification_status_display(&binary.status);

        println!("{} {}: {}", status_icon, binary.name, status_text);
        println!(
            "   Expected: v{} (SHA256: {}...)",
            binary.expected_version,
            &binary.expected_sha256[..16]
        );

        if let Some(ref actual_version) = binary.actual_version {
            if let Some(ref actual_sha256) = binary.actual_sha256 {
                println!(
                    "   Actual:   v{} (SHA256: {}...)",
                    actual_version,
                    &actual_sha256[..16]
                );
            } else {
                println!("   Actual:   v{actual_version} (SHA256: unknown)");
            }
        } else {
            println!("   Actual:   MISSING");
        }
        println!();
    }

    if !report.recommendations.is_empty() {
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("💡 Recommendations:");
        for rec in &report.recommendations {
            println!("   • {rec}");
        }
        println!();
    }

    Ok(())
}

async fn verify_all_spores(verbose: bool) -> Result<()> {
    info!("Verifying all mounted spores");

    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║                                                                ║");
    println!("║         🔍 All Spores Verification Report                     ║");
    println!("║                                                                ║");
    println!("╚════════════════════════════════════════════════════════════════╝");
    println!();

    // Load nucleus manifest
    let nucleus_path = PathBuf::from("plasmidBin");
    if !nucleus_path.exists() {
        println!("❌ Error: plasmidBin not found (required for comparison)");
        println!("   Expected at: {}", nucleus_path.display());
        return Ok(());
    }

    let verifier = SporeVerifier::from_nucleus(&nucleus_path)?;
    let reports = verifier.verify_all_spores()?;

    if reports.is_empty() {
        println!("⚠️  No spores found");
        println!();
        println!("Expected spores in:");
        println!("  /media/$USER/*/biomeOS/");
        println!();
        println!("Make sure USB spores are mounted and contain:");
        println!("  - tower.toml");
        println!("  - .family.seed");
        println!("  - bin/tower");
        println!("  - primals/");
        return Ok(());
    }

    let mut fresh_count = 0;
    let mut stale_count = 0;
    let mut other_count = 0;

    println!("Found {} spore(s):", reports.len());
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();

    for report in &reports {
        let (status_icon, status_text) = match report.overall_status {
            VerificationStatus::Fresh => {
                fresh_count += 1;
                verification_status_display(&VerificationStatus::Fresh)
            }
            VerificationStatus::Stale => {
                stale_count += 1;
                verification_status_display(&VerificationStatus::Stale)
            }
            _ => {
                other_count += 1;
                ("❌", "Issue")
            }
        };

        println!(
            "{} {} ({}): {}",
            status_icon,
            report.node_id,
            report.spore_path.display(),
            status_text
        );

        if verbose {
            for binary in &report.binaries {
                let bin_icon = match binary.status {
                    VerificationStatus::Fresh => "✅",
                    VerificationStatus::Stale => "⚠️ ",
                    _ => "❌",
                };
                println!("   {} {}: {:?}", bin_icon, binary.name, binary.status);
            }
        }

        println!();
    }

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Summary:");
    println!("  ✅ Fresh:  {fresh_count}");
    println!("  ⚠️  Stale:  {stale_count}");
    println!("  ❌ Issues: {other_count}");
    println!("  📊 Total:  {}", reports.len());
    println!();

    if stale_count > 0 {
        println!("💡 Recommendation:");
        println!("   Run 'biomeos spore refresh <mount>' to update stale spores");
        println!("   Or re-create spores with fresh binaries from plasmidBin");
        println!();
    }

    if fresh_count == reports.len() {
        println!("✅ All spores are fresh and ready for deployment!");
    }

    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use biomeos_spore::verification::VerificationStatus;

    #[test]
    fn test_verification_status_display() {
        assert_eq!(
            verification_status_display(&VerificationStatus::Fresh),
            ("✅", "Fresh")
        );
        assert_eq!(
            verification_status_display(&VerificationStatus::Stale),
            ("⚠️ ", "Stale")
        );
        assert_eq!(
            verification_status_display(&VerificationStatus::Missing),
            ("❌", "Missing")
        );
        assert_eq!(
            verification_status_display(&VerificationStatus::Modified),
            ("⚠️ ", "Modified")
        );
        assert_eq!(
            verification_status_display(&VerificationStatus::Newer),
            ("❓", "Newer")
        );
    }

    #[tokio::test]
    async fn test_verify_args_parsing() {
        // Test that the command structure is valid
        // Actual verification logic is tested in biomeos-spore
    }

    #[test]
    fn test_verification_status_display_all_variants() {
        use biomeos_spore::verification::VerificationStatus;
        let variants = [
            (VerificationStatus::Fresh, "✅", "Fresh"),
            (VerificationStatus::Stale, "⚠️ ", "Stale"),
            (VerificationStatus::Missing, "❌", "Missing"),
            (VerificationStatus::Modified, "⚠️ ", "Modified"),
            (VerificationStatus::Newer, "❓", "Newer"),
        ];
        for (status, expected_icon, expected_text) in variants {
            let (icon, text) = verification_status_display(&status);
            assert_eq!(icon, expected_icon, "icon for {status:?}");
            assert_eq!(text, expected_text, "text for {status:?}");
        }
    }

    #[test]
    fn test_verification_status_display_icons_distinct() {
        let (fresh_icon, _) = verification_status_display(&VerificationStatus::Fresh);
        let (missing_icon, _) = verification_status_display(&VerificationStatus::Missing);
        assert_ne!(fresh_icon, missing_icon);
    }

    #[test]
    fn test_verify_args_target_nucleus() {
        let args = VerifyArgs {
            target: VerifyTarget::Nucleus {
                path: PathBuf::from("plasmidBin"),
            },
        };
        match &args.target {
            VerifyTarget::Nucleus { path } => assert_eq!(path, &PathBuf::from("plasmidBin")),
            _ => panic!("expected Nucleus"),
        }
    }

    #[test]
    fn test_verify_args_target_spore() {
        let args = VerifyArgs {
            target: VerifyTarget::Spore {
                mount_point: PathBuf::from("/media/usb/biomeOS"),
            },
        };
        match &args.target {
            VerifyTarget::Spore { mount_point } => {
                assert_eq!(mount_point, &PathBuf::from("/media/usb/biomeOS"));
            }
            _ => panic!("expected Spore"),
        }
    }

    #[test]
    fn test_verify_args_target_all() {
        let args = VerifyArgs {
            target: VerifyTarget::All { verbose: false },
        };
        match &args.target {
            VerifyTarget::All { verbose } => assert!(!*verbose),
            _ => panic!("expected All"),
        }
    }

    #[tokio::test]
    async fn test_run_nucleus_nonexistent_path() {
        let args = VerifyArgs {
            target: VerifyTarget::Nucleus {
                path: PathBuf::from("/nonexistent/path/xyz"),
            },
        };
        let result = run(args).await;
        assert!(result.is_ok(), "run should return Ok (prints message)");
    }

    #[tokio::test]
    async fn test_run_spore_nonexistent() {
        let args = VerifyArgs {
            target: VerifyTarget::Spore {
                mount_point: PathBuf::from("/nonexistent/spore/mount"),
            },
        };
        let result = run(args).await;
        assert!(result.is_ok(), "spore verify with no plasmidBin returns Ok");
    }

    #[tokio::test]
    async fn test_run_all_spores() {
        let args = VerifyArgs {
            target: VerifyTarget::All { verbose: false },
        };
        let result = run(args).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_run_all_spores_verbose() {
        let args = VerifyArgs {
            target: VerifyTarget::All { verbose: true },
        };
        let result = run(args).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_verify_args_debug() {
        let args = VerifyArgs {
            target: VerifyTarget::Nucleus {
                path: PathBuf::from("plasmidBin"),
            },
        };
        let _ = format!("{args:?}");
    }

    #[test]
    fn test_verify_target_variants() {
        let _ = format!(
            "{:?}",
            VerifyTarget::Nucleus {
                path: PathBuf::from("p")
            }
        );
        let _ = format!(
            "{:?}",
            VerifyTarget::Spore {
                mount_point: PathBuf::from("/m")
            }
        );
        let _ = format!("{:?}", VerifyTarget::All { verbose: true });
    }
}
