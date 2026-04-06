// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Spore management commands

use std::path::{Path, PathBuf};

use anyhow::Result;
use biomeos_spore::{Spore, SporeConfig, SporeType, SporeVerification};
use biomeos_types::primal_names::CORE_PRIMALS;
use serde_json::Value;

/// Information about a path in the spore structure.
#[derive(Debug, Clone)]
pub struct PathInfo {
    /// Relative path name
    pub name: String,
    /// Whether the path exists
    pub exists: bool,
    /// Unix permissions as octal (e.g. 0o755). None if not on Unix or not available.
    pub permissions: Option<u32>,
}

/// Report of what would be refreshed in a dry run.
#[cfg(test)]
#[derive(Debug, Clone, Default)]
pub struct RefreshReport {
    /// Spore paths that need refreshing.
    pub to_refresh: Vec<PathBuf>,
    /// Spore paths that are up to date.
    pub to_keep: Vec<PathBuf>,
}

/// Parse spore type from string (live/cold)
pub fn parse_spore_type(s: &str) -> Result<SporeType> {
    match s.to_lowercase().as_str() {
        "live" => Ok(SporeType::Live),
        "cold" => Ok(SporeType::Cold),
        _ => Err(anyhow::anyhow!(
            "Invalid spore type: '{s}'. Valid types: 'live' (deployable) or 'cold' (storage)"
        )),
    }
}

/// Gathers structure info for paths under a spore root. Returns `PathInfo` for each.
pub(crate) fn gather_spore_structure_info(path: &Path) -> Vec<PathInfo> {
    let mut rel_paths: Vec<String> = vec![
        ".family.seed".to_string(),
        "tower.toml".to_string(),
        "bin/tower".to_string(),
    ];
    for primal in CORE_PRIMALS {
        rel_paths.push(format!("primals/{primal}"));
    }

    let mut infos = Vec::new();
    for rel in &rel_paths {
        let full_path = path.join(rel);
        let exists = full_path.exists();

        let permissions = if exists {
            #[cfg(unix)]
            {
                std::fs::metadata(&full_path).ok().map(|m| {
                    use std::os::unix::fs::PermissionsExt;
                    m.permissions().mode() & 0o777
                })
            }
            #[cfg(not(unix))]
            None
        } else {
            None
        };

        infos.push(PathInfo {
            name: rel.clone(),
            exists,
            permissions,
        });
    }

    infos
}

/// Computes refresh plan from paths and parallel would_refresh flags.
#[cfg(test)]
pub(crate) fn compute_refresh_plan(paths: &[PathBuf], would_refresh: &[bool]) -> RefreshReport {
    let mut to_refresh = Vec::new();
    let mut to_keep = Vec::new();

    for (i, path) in paths.iter().enumerate() {
        if i < would_refresh.len() && would_refresh[i] {
            to_refresh.push(path.clone());
        } else {
            to_keep.push(path.clone());
        }
    }

    RefreshReport {
        to_refresh,
        to_keep,
    }
}

/// Builds display lines for spore create summary.
pub(crate) fn format_spore_create_summary(spore_info: &Value) -> Vec<String> {
    let mut lines = Vec::new();

    if let Some(location) = spore_info.get("location").and_then(|v| v.as_str()) {
        lines.push(format!("   Location: {location}"));
    }

    lines.push(String::new());
    lines.push("📋 What was created:".to_string());
    lines.push("   • Directory structure (bin/, primals/, secrets/, logs/)".to_string());
    lines.push("   • Family seed file (.family.seed)".to_string());
    lines.push("   • Tower configuration (tower.toml)".to_string());
    lines.push("   • Primal binaries (if available)".to_string());
    lines.push(String::new());
    lines.push("🔐 Security:".to_string());
    lines.push("   • Seed file permissions: 0600 (owner only)".to_string());
    lines.push("   • BearDog will handle all cryptography".to_string());
    lines.push("   • No secrets exposed in configuration".to_string());

    lines
}

/// Create a new USB spore
pub async fn handle_spore_create(
    mount: PathBuf,
    label: String,
    node_id: String,
    spore_type_str: String,
) -> Result<()> {
    let spore_type = parse_spore_type(&spore_type_str)?;

    println!("🔐 Creating {spore_type} USB spore...");
    println!("   Label: {label}");
    println!("   Node ID: {node_id}");
    println!("   Mount: {}", mount.display());
    println!("   Type: {} {}", spore_type.emoji(), spore_type);

    let family_id = std::env::var("FAMILY_ID").unwrap_or_else(|_| "default".to_string());

    let config = SporeConfig {
        label: label.clone(),
        node_id: node_id.clone(),
        family_id,
        spore_type,
        plasmid_bin_dir: None,
    };

    let spore = Spore::create(mount, config).await?;

    println!("\n✅ Spore created successfully!");
    let spore_info = serde_json::json!({
        "location": spore.root_path().display().to_string(),
    });
    let lines = format_spore_create_summary(&spore_info);
    for line in lines {
        println!("{line}");
    }

    Ok(())
}

/// Clone an existing spore to create a sibling
pub async fn handle_spore_clone(from: PathBuf, to: PathBuf, node_id: String) -> Result<()> {
    println!("🔄 Cloning spore to create sibling...");
    println!("   Source: {}", from.display());
    println!("   Target: {}", to.display());
    println!("   New Node ID: {node_id}");

    // Load source spore
    let source = Spore::from_path(&from)?;
    println!("   Source label: {}", source.config().label);

    // Clone to create sibling (same family, different node_id)
    let sibling = source.clone_sibling(to, &node_id).await?;

    println!("\n✅ Sibling spore created!");
    println!("   Location: {}", sibling.root_path().display());
    println!("\n🧬 Genetic Lineage:");
    println!("   • Same family seed (siblings!)");
    println!("   • BearDog will recognize as family");
    println!("   • Cryptographic trust enabled");

    Ok(())
}

/// Verify spore integrity
pub async fn handle_spore_verify(mount: PathBuf) -> Result<()> {
    println!("🔍 Verifying spore...");
    println!("   Path: {}", mount.display());

    let spore_path = mount.join("biomeOS");
    let result = SporeVerification::verify(&spore_path).await?;

    println!();
    result.print_summary();

    if !result.valid {
        return Err(anyhow::anyhow!(
            "Some verification checks failed. Review the details above."
        ));
    }

    Ok(())
}

/// Show spore information
pub async fn handle_spore_info(mount: PathBuf) -> Result<()> {
    println!("📊 Spore Information");
    println!("   Path: {}", mount.display());

    let spore = Spore::from_path(&mount)?;

    println!("\n📝 Configuration:");
    println!("   Label: {}", spore.config().label);
    println!("   Node ID: {}", spore.config().node_id);
    println!("   Root: {}", spore.root_path().display());

    println!("\n📁 Structure:");
    let infos = gather_spore_structure_info(spore.root_path());
    for info in &infos {
        let icon = if info.exists { "✅" } else { "❌" };
        println!("   {} {}", icon, info.name);

        if info.exists {
            if let Ok(metadata) = tokio::fs::metadata(spore.root_path().join(&info.name)).await {
                println!("      Size: {} bytes", metadata.len());

                #[cfg(unix)]
                if let Some(mode) = info.permissions {
                    println!("      Permissions: {mode:o}");
                }
            }
        }
    }

    println!("\n🏗️  Architecture:");
    println!("   • biomeOS: Orchestration layer (this spore)");
    println!("   • BearDog: Security layer (handles crypto)");
    println!("   • Songbird: Discovery layer (UDP multicast)");

    Ok(())
}

/// List available USB devices
pub async fn handle_spore_list() -> Result<()> {
    use biomeos_spore::usb;

    println!("🔍 Discovering USB devices...\n");

    let devices = usb::discover_usb_devices().await?;

    if devices.is_empty() {
        println!("No USB devices found.");
        println!("\nCheck that your USB drive is:");
        println!("  • Properly inserted");
        println!("  • Mounted (e.g., /media/usb)");
        println!("  • Accessible to your user");
        return Ok(());
    }

    println!("Found {} device(s):\n", devices.len());

    for device in &devices {
        println!("📱 Device:");
        println!("   Mount: {}", device.mount_point.display());
        if let Some(ref label) = device.label {
            println!("   Label: {label}");
        }
        println!(
            "   Space: {:.2} GB available / {:.2} GB total ({:.1}% used)",
            device.available_space as f64 / 1_000_000_000.0,
            device.total_space as f64 / 1_000_000_000.0,
            device.utilization_percent()
        );

        // Check if it has a spore
        let spore_path = device.mount_point.join("biomeOS");
        if spore_path.exists() {
            println!("   🎯 Contains spore!");
        }
        println!();
    }

    Ok(())
}

pub(super) fn discover_plasmid_dir_with_override(
    plasmid_override: Option<&Path>,
) -> Result<PathBuf> {
    if let Some(path) = plasmid_override {
        if path.exists() {
            return Ok(path.to_path_buf());
        }
        return Err(anyhow::anyhow!(
            "plasmidBin not found at {}",
            path.display()
        ));
    }

    if let Ok(p) = std::env::var("BIOMEOS_PLASMID_DIR") {
        let path = PathBuf::from(p);
        if path.exists() {
            return Ok(path);
        }
    }

    let cwd_relative = PathBuf::from("plasmidBin");
    if cwd_relative.exists() {
        return Ok(cwd_relative);
    }

    Err(anyhow::anyhow!(
        "plasmidBin not found. Set BIOMEOS_PLASMID_DIR or run from the workspace root. \
         Run 'cargo run -p biomeos-harvest' to build binaries first."
    ))
}

/// Refresh spore binaries from plasmidBin
pub async fn handle_spore_refresh(mount: PathBuf, dry_run: bool) -> Result<()> {
    handle_spore_refresh_impl(mount, dry_run, None).await
}

#[cfg(test)]
/// Test helper: `handle_spore_refresh` with an explicit plasmid directory.
pub async fn handle_spore_refresh_with_plasmid_dir(
    mount: PathBuf,
    dry_run: bool,
    plasmid_dir: PathBuf,
) -> Result<()> {
    handle_spore_refresh_impl(mount, dry_run, Some(plasmid_dir)).await
}

async fn handle_spore_refresh_impl(
    mount: PathBuf,
    dry_run: bool,
    plasmid_override: Option<PathBuf>,
) -> Result<()> {
    use biomeos_spore::refresh::SporeRefresher;
    use biomeos_spore::verification::{SporeVerifier, VerificationStatus};

    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║                                                                ║");
    println!("║         🔄 Spore Refresh                                       ║");
    println!("║                                                                ║");
    println!("╚════════════════════════════════════════════════════════════════╝");
    println!();

    if dry_run {
        println!("🔍 DRY RUN MODE - No changes will be made");
        println!();
    }

    println!("Spore: {}", mount.display());
    println!();

    let nucleus_path = discover_plasmid_dir_with_override(plasmid_override.as_deref())?;

    let refresher = SporeRefresher::from_nucleus(&nucleus_path)?;

    if dry_run {
        // Dry run: just verify what would be updated
        let verifier = SporeVerifier::from_nucleus(&nucleus_path)?;
        let report = verifier.verify_spore(&mount)?;

        println!("📋 Binaries that would be refreshed:");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        let mut would_refresh = false;
        for binary in &report.binaries {
            if !matches!(binary.status, VerificationStatus::Fresh) {
                would_refresh = true;
                println!("🔄 {}", binary.name);
                println!(
                    "   Current: v{}",
                    binary.actual_version.as_deref().unwrap_or("unknown")
                );
                println!("   New:     v{}", binary.expected_version);
                println!();
            }
        }

        if would_refresh {
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!();
            println!("💡 Run without --dry-run to apply these updates");
        } else {
            println!("✅ No binaries need refreshing - spore is already fresh!");
        }
    } else {
        // Actual refresh
        println!("🔄 Refreshing binaries...");
        println!();

        let report = refresher.refresh_spore(&mount)?;

        if report.refreshed_binaries.is_empty() && report.failed_binaries.is_empty() {
            println!("✅ Spore is already fresh - no updates needed!");
        } else {
            if !report.refreshed_binaries.is_empty() {
                println!("✅ Successfully Refreshed:");
                println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                for binary in &report.refreshed_binaries {
                    println!("✅ {}", binary.name);
                    if let Some(ref old_ver) = binary.old_version {
                        println!("   {} → v{}", old_ver, binary.new_version);
                    } else {
                        println!("   v{} (newly installed)", binary.new_version);
                    }
                    println!(
                        "   SHA256: {}...",
                        &binary.new_sha256[..16.min(binary.new_sha256.len())]
                    );
                    println!();
                }
                println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            }

            if !report.failed_binaries.is_empty() {
                println!();
                println!("❌ Failed to Refresh:");
                println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                for failed in &report.failed_binaries {
                    println!("❌ {}: {}", failed.name, failed.error);
                }
                println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            }

            println!();
            if report.is_success() {
                println!(
                    "🎊 Spore refresh complete! {} binaries updated.",
                    report.refreshed_binaries.len()
                );
            } else {
                println!(
                    "⚠️  Spore refresh partially complete. {} succeeded, {} failed.",
                    report.refreshed_binaries.len(),
                    report.failed_binaries.len()
                );
            }
        }
    }

    println!();
    Ok(())
}

#[cfg(test)]
#[path = "spore_tests.rs"]
mod tests;
