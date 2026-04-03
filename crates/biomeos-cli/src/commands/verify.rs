// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

// crates/biomeos-cli/src/commands/verify.rs
//! Spore and nucleus verification commands

use anyhow::Result;
use biomeos_spore::manifest::BinaryManifest;
use biomeos_spore::verification::{SporeVerifier, VerificationStatus};

/// Map verification status to (icon, text) for display (testable pure function)
#[must_use]
pub const fn verification_status_display(
    status: &VerificationStatus,
) -> (&'static str, &'static str) {
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

/// Resolve plasmidBin directory from optional `BIOMEOS_PLASMID_DIR`-style value (testable).
fn plasmid_bin_dir_for_verify(plasmid_dir_env: Option<&str>) -> PathBuf {
    match plasmid_dir_env {
        Some(p) if !p.is_empty() => PathBuf::from(p),
        _ => PathBuf::from("plasmidBin"),
    }
}

/// `plasmidBin` path for spore verification: `BIOMEOS_PLASMID_DIR` if set, else cwd-relative `plasmidBin`.
fn nucleus_path_for_spore_verify() -> PathBuf {
    plasmid_bin_dir_for_verify(std::env::var("BIOMEOS_PLASMID_DIR").ok().as_deref())
}

/// Parse `BIOMEOS_SPORE_PATHS`-style comma-separated list (testable; used by [`discover_spore_mounts`]).
fn parse_biomeos_spore_paths_list(paths: &str) -> Vec<(String, String)> {
    paths
        .split(',')
        .filter(|p| !p.trim().is_empty())
        .enumerate()
        .map(|(i, p)| {
            let p = p.trim().to_string();
            let node_id = PathBuf::from(&p)
                .file_name()
                .map_or_else(|| format!("node-{i}"), |n| n.to_string_lossy().to_string());
            (p, node_id)
        })
        .collect()
}

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
            verify_nucleus(&path)?;
        }
        VerifyTarget::Spore { mount_point } => {
            verify_single_spore(&mount_point)?;
        }
        VerifyTarget::All { verbose } => {
            verify_all_spores(verbose)?;
        }
    }

    Ok(())
}

fn verify_nucleus(nucleus_path: &PathBuf) -> Result<()> {
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

fn verify_single_spore(mount_point: &PathBuf) -> Result<()> {
    verify_single_spore_at(mount_point, nucleus_path_for_spore_verify())
}

/// Verify a spore against a specific plasmidBin path (env-free; tests and tooling).
fn verify_single_spore_at(
    mount_point: &PathBuf,
    nucleus_path: impl AsRef<std::path::Path>,
) -> Result<()> {
    info!("Verifying spore at: {}", mount_point.display());

    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║                                                                ║");
    println!("║         🔍 Spore Verification Report                          ║");
    println!("║                                                                ║");
    println!("╚════════════════════════════════════════════════════════════════╝");
    println!();

    // Load nucleus manifest
    let nucleus_path = nucleus_path.as_ref();
    if !nucleus_path.exists() {
        println!("❌ Error: plasmidBin not found (required for comparison)");
        println!("   Expected at: {}", nucleus_path.display());
        return Ok(());
    }

    let verifier = SporeVerifier::from_nucleus(nucleus_path)?;
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

fn verify_all_spores(verbose: bool) -> Result<()> {
    info!("Verifying all mounted spores");

    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║                                                                ║");
    println!("║         🔍 All Spores Verification Report                     ║");
    println!("║                                                                ║");
    println!("╚════════════════════════════════════════════════════════════════╝");
    println!();

    // Load nucleus manifest
    let nucleus_path = nucleus_path_for_spore_verify();
    if !nucleus_path.exists() {
        println!("❌ Error: plasmidBin not found (required for comparison)");
        println!("   Expected at: {}", nucleus_path.display());
        return Ok(());
    }

    let verifier = SporeVerifier::from_nucleus(nucleus_path)?;
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

/// Run the genetic lineage verification workflow.
///
/// Discovers spore mount points from the environment (`BIOMEOS_SPORE_PATHS`,
/// or scans `/media/$USER` by default), then verifies genetic relationships
/// between spores using the security provider's HKDF-SHA256 lineage system.
pub async fn run_verify_lineage() -> Result<()> {
    use biomeos_federation::security_client::SecurityProviderClient;

    println!("\n  Genetic Lineage Verifier (security provider)");
    println!("===============================================\n");

    println!("Discovering security provider...");
    let client = SecurityProviderClient::from_discovery()
        .await
        .map_err(|e| {
            anyhow::anyhow!(
                "Failed to discover security provider: {e}. Is the security primal running?"
            )
        })?;
    println!("Security provider found!\n");

    let spore_paths = discover_spore_mounts();
    if spore_paths.is_empty() {
        println!("No spore seeds found. Set BIOMEOS_SPORE_PATHS or mount USB spores.");
        return Ok(());
    }

    let mut spores = Vec::new();
    println!("Loading spore seeds...");
    for (path, node_id) in &spore_paths {
        let seed_path = PathBuf::from(path).join(".family.seed");
        if !seed_path.exists() {
            println!(
                "  Skipping {node_id}: seed not found at {}",
                seed_path.display()
            );
            continue;
        }
        match std::fs::read(&seed_path) {
            Ok(seed_bytes) => {
                use sha2::{Digest, Sha256};
                let seed_hash = format!("{:x}", Sha256::digest(&seed_bytes));
                println!("  {node_id}: {}...", &seed_hash[..16]);
                spores.push((node_id.clone(), seed_bytes, seed_hash));
            }
            Err(e) => println!("  Failed to read {node_id}: {e}"),
        }
    }

    if spores.len() < 2 {
        println!("Need at least 2 spores to compare (found {})", spores.len());
        return Ok(());
    }

    println!("\nVerifying Genetic Relationships\n");

    let mut all_siblings = true;
    let mut any_siblings = false;

    for i in 0..spores.len() {
        for j in (i + 1)..spores.len() {
            let (node_a, _, _) = &spores[i];
            let (node_b, _, hash_b) = &spores[j];

            println!("Testing: {node_a} <-> {node_b}");

            match client.verify_same_family("nat0", hash_b, node_b).await {
                Ok(response) => {
                    if response.is_family_member {
                        println!("  RELATED: {}", response.relationship);
                        any_siblings = true;
                    } else {
                        println!("  UNRELATED: Different genetic families");
                        all_siblings = false;
                    }
                }
                Err(e) => {
                    println!("  Verification failed: {e}");
                    all_siblings = false;
                }
            }
            println!();
        }
    }

    println!("Summary: {} spores tested", spores.len());
    if all_siblings {
        println!("  All spores are SIBLINGS (same parent)");
    } else if any_siblings {
        println!("  MIXED relationships detected");
    } else {
        println!("  Spores are UNRELATED (different parents)");
    }
    println!();

    Ok(())
}

/// Discover spore mount points from `BIOMEOS_SPORE_PATHS` or by scanning
/// `/media/$USER` for directories containing `.family.seed`.
fn discover_spore_mounts() -> Vec<(String, String)> {
    if let Ok(paths) = std::env::var("BIOMEOS_SPORE_PATHS") {
        return parse_biomeos_spore_paths_list(&paths);
    }

    let user = std::env::var("USER").unwrap_or_else(|_| "root".to_string());
    let media_dir = PathBuf::from(format!("/media/{user}"));
    if !media_dir.exists() {
        return Vec::new();
    }

    let mut mounts = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&media_dir) {
        for entry in entries.flatten() {
            let biome_path = entry.path().join("biomeOS");
            if biome_path.join(".family.seed").exists() {
                let node_id = entry.file_name().to_string_lossy().to_string();
                mounts.push((biome_path.to_string_lossy().to_string(), node_id));
            }
        }
    }
    mounts
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test")]
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
        let _result = run(args).await;
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

    fn minimal_plasmid_bin(temp: &std::path::Path) -> std::path::PathBuf {
        let pb = temp.join("plasmidBin");
        std::fs::create_dir_all(pb.join("tower")).unwrap();
        std::fs::write(pb.join("tower").join("tower"), b"tower-bytes").unwrap();
        std::fs::create_dir_all(pb.join("primals")).unwrap();
        std::fs::write(pb.join("primals").join("beardog-server"), b"bd").unwrap();
        std::fs::write(pb.join("primals").join("songbird"), b"sb").unwrap();
        pb
    }

    #[tokio::test]
    async fn test_run_nucleus_with_minimal_plasmid_bin() {
        let temp = tempfile::tempdir().unwrap();
        let pb = minimal_plasmid_bin(temp.path());
        let args = VerifyArgs {
            target: VerifyTarget::Nucleus { path: pb },
        };
        assert!(run(args).await.is_ok());
    }

    #[tokio::test]
    async fn test_run_nucleus_with_manifest_toml() {
        let temp = tempfile::tempdir().unwrap();
        let pb = minimal_plasmid_bin(temp.path());
        let manifest = biomeos_spore::manifest::BinaryManifest::from_nucleus(&pb).unwrap();
        manifest.save(pb.join("MANIFEST.toml")).unwrap();

        let args = VerifyArgs {
            target: VerifyTarget::Nucleus { path: pb },
        };
        assert!(run(args).await.is_ok());
    }

    #[tokio::test]
    async fn test_run_nucleus_with_manifest_features_prints_features_line() {
        use biomeos_spore::manifest::{
            BinaryInfo, BinaryManifest, CompatibilityInfo, ManifestMeta,
        };
        use chrono::Utc;
        use sha2::{Digest, Sha256};
        use std::collections::HashMap;

        let temp = tempfile::tempdir().unwrap();
        let pb = minimal_plasmid_bin(temp.path());
        let tower_path = pb.join("tower/tower");
        let bytes = std::fs::read(&tower_path).unwrap();
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        let sha = format!("{:x}", hasher.finalize());

        let mut binaries = HashMap::new();
        binaries.insert(
            "tower".to_string(),
            BinaryInfo {
                name: "tower".to_string(),
                version: "9.9.9".to_string(),
                git_commit: "abc".to_string(),
                build_date: Utc::now(),
                sha256: sha,
                size_bytes: bytes.len() as u64,
                source_repo: "test".to_string(),
                features: vec!["feat-a".to_string(), "feat-b".to_string()],
            },
        );

        let manifest = BinaryManifest {
            manifest: ManifestMeta {
                version: "9.9.9".to_string(),
                created_at: Utc::now(),
                pipeline_run: "test".to_string(),
            },
            binaries,
            compatibility: CompatibilityInfo {
                min_tower_version: "1.0.0".to_string(),
                min_beardog_version: "0.1.0".to_string(),
                min_songbird_version: "0.1.0".to_string(),
            },
        };
        manifest.save(pb.join("MANIFEST.toml")).unwrap();

        let args = VerifyArgs {
            target: VerifyTarget::Nucleus { path: pb },
        };
        assert!(run(args).await.is_ok());
    }

    #[tokio::test]
    async fn test_run_spore_verify_with_matching_plasmid_and_spore() {
        let temp = tempfile::tempdir().unwrap();
        let pb = minimal_plasmid_bin(temp.path());
        let spore = temp.path().join("spore-mount");
        std::fs::create_dir_all(spore.join("bin")).unwrap();
        std::fs::create_dir_all(spore.join("primals")).unwrap();
        std::fs::write(spore.join("bin").join("tower"), b"tower-bytes").unwrap();
        std::fs::write(spore.join("primals").join("beardog-server"), b"bd").unwrap();
        std::fs::write(spore.join("primals").join("songbird"), b"sb").unwrap();
        std::fs::write(spore.join(".family.seed"), b"seed").unwrap();
        std::fs::write(
            spore.join("tower.toml"),
            r#"
[tower]
NODE_ID = "node-test-123"
"#,
        )
        .unwrap();

        assert!(verify_single_spore_at(&spore, &pb).is_ok());
    }

    #[tokio::test]
    async fn test_run_spore_verify_stale_binary() {
        let temp = tempfile::tempdir().unwrap();
        let pb = minimal_plasmid_bin(temp.path());
        let spore = temp.path().join("spore-stale");
        std::fs::create_dir_all(spore.join("bin")).unwrap();
        std::fs::create_dir_all(spore.join("primals")).unwrap();
        std::fs::write(spore.join("bin").join("tower"), b"wrong-tower").unwrap();
        std::fs::write(spore.join("primals").join("beardog-server"), b"bd").unwrap();
        std::fs::write(spore.join("primals").join("songbird"), b"sb").unwrap();
        std::fs::write(spore.join(".family.seed"), b"seed").unwrap();
        std::fs::write(
            spore.join("tower.toml"),
            r#"
[tower]
NODE_ID = "node-stale"
"#,
        )
        .unwrap();

        assert!(verify_single_spore_at(&spore, &pb).is_ok());
    }

    #[tokio::test]
    async fn test_verify_single_spore_at_missing_binary_branch() {
        let temp = tempfile::tempdir().unwrap();
        let pb = minimal_plasmid_bin(temp.path());
        let spore = temp.path().join("spore-missing-bin");
        std::fs::create_dir_all(spore.join("bin")).unwrap();
        std::fs::create_dir_all(spore.join("primals")).unwrap();
        std::fs::write(spore.join("bin").join("tower"), b"tower-bytes").unwrap();
        std::fs::write(spore.join("primals").join("beardog-server"), b"bd").unwrap();
        // songbird intentionally absent — exercises Missing per-binary path
        std::fs::write(spore.join(".family.seed"), b"seed").unwrap();
        std::fs::write(
            spore.join("tower.toml"),
            r#"
[tower]
NODE_ID = "node-missing"
"#,
        )
        .unwrap();

        assert!(verify_single_spore_at(&spore, &pb).is_ok());
    }

    #[test]
    fn test_parse_biomeos_spore_paths_list_two_paths() {
        let mounts = super::parse_biomeos_spore_paths_list(
            "/tmp/biome-paths/spore-a,/tmp/biome-paths/spore-b",
        );
        assert_eq!(mounts.len(), 2);
        assert_eq!(mounts[0].0, "/tmp/biome-paths/spore-a");
        assert_eq!(mounts[0].1, "spore-a");
        assert_eq!(mounts[1].1, "spore-b");
    }

    #[test]
    fn test_parse_biomeos_spore_paths_list_skips_empty_segments() {
        let mounts = super::parse_biomeos_spore_paths_list("/x/only-one,,/y/two");
        assert_eq!(mounts.len(), 2);
        assert_eq!(mounts[0].1, "only-one");
        assert_eq!(mounts[1].1, "two");
    }

    #[test]
    fn test_plasmid_bin_dir_for_verify_custom() {
        assert_eq!(
            super::plasmid_bin_dir_for_verify(Some("/opt/custom/plasmidBin")),
            PathBuf::from("/opt/custom/plasmidBin")
        );
    }

    #[test]
    fn test_plasmid_bin_dir_for_verify_defaults_when_none_or_empty() {
        assert_eq!(
            super::plasmid_bin_dir_for_verify(None),
            PathBuf::from("plasmidBin")
        );
        assert_eq!(
            super::plasmid_bin_dir_for_verify(Some("")),
            PathBuf::from("plasmidBin")
        );
    }

    #[tokio::test]
    async fn test_verify_single_spore_at_nucleus_missing_returns_ok() {
        let temp = tempfile::tempdir().unwrap();
        let spore = temp.path().join("any-spore");
        std::fs::create_dir_all(&spore).unwrap();
        let missing_nucleus = temp.path().join("no-plasmid-here");
        assert!(verify_single_spore_at(&spore, &missing_nucleus).is_ok());
    }
}
