// crates/biomeos-cli/src/commands/verify.rs
//! Spore and nucleus verification commands

use anyhow::Result;
use biomeos_spore::manifest::BinaryManifest;
use biomeos_spore::verification::{SporeVerifier, VerificationStatus};
use clap::{Args, Subcommand};
use std::path::PathBuf;
use tracing::info;

/// Arguments for verification commands
#[derive(Args)]
pub struct VerifyArgs {
    /// Verification target
    #[command(subcommand)]
    target: VerifyTarget,
}

/// Targets for verification
#[derive(Subcommand)]
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
            println!("✅ {}", name);
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
        let (status_icon, status_text) = match binary.status {
            VerificationStatus::Fresh => ("✅", "Fresh"),
            VerificationStatus::Stale => ("⚠️ ", "Stale"),
            VerificationStatus::Missing => ("❌", "Missing"),
            VerificationStatus::Modified => ("⚠️ ", "Modified"),
            VerificationStatus::Newer => ("❓", "Newer"),
        };

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
                println!("   Actual:   v{} (SHA256: unknown)", actual_version);
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
            println!("   • {}", rec);
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
                ("✅", "Fresh")
            }
            VerificationStatus::Stale => {
                stale_count += 1;
                ("⚠️ ", "Stale")
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
    println!("  ✅ Fresh:  {}", fresh_count);
    println!("  ⚠️  Stale:  {}", stale_count);
    println!("  ❌ Issues: {}", other_count);
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
mod tests {
    #[tokio::test]
    async fn test_verify_args_parsing() {
        // Test that the command structure is valid
        // Actual verification logic is tested in biomeos-spore
    }
}
