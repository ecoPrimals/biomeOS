//! Spore management commands

use std::path::PathBuf;

use anyhow::Result;
use biomeos_spore::{Spore, SporeConfig, SporeType, SporeVerification};

/// Create a new USB spore
pub async fn handle_spore_create(
    mount: PathBuf,
    label: String,
    node_id: String,
    spore_type_str: String,
) -> Result<()> {
    let spore_type = match spore_type_str.to_lowercase().as_str() {
        "live" => SporeType::Live,
        "cold" => SporeType::Cold,
        _ => {
            eprintln!("❌ Invalid spore type: {}", spore_type_str);
            eprintln!("   Valid types: 'live' (deployable) or 'cold' (storage)");
            std::process::exit(1);
        }
    };

    println!("🔐 Creating {} USB spore...", spore_type);
    println!("   Label: {}", label);
    println!("   Node ID: {}", node_id);
    println!("   Mount: {}", mount.display());
    println!("   Type: {} {}", spore_type.emoji(), spore_type);

    let config = SporeConfig {
        label: label.clone(),
        node_id: node_id.clone(),
        spore_type,
    };

    let spore = Spore::create(mount, config).await?;

    println!("\n✅ Spore created successfully!");
    println!("   Location: {}", spore.root_path().display());
    println!("\n📋 What was created:");
    println!("   • Directory structure (bin/, primals/, secrets/, logs/)");
    println!("   • Family seed file (.family.seed)");
    println!("   • Tower configuration (tower.toml)");
    println!("   • Primal binaries (if available)");
    println!("\n🔐 Security:");
    println!("   • Seed file permissions: 0600 (owner only)");
    println!("   • BearDog will handle all cryptography");
    println!("   • No secrets exposed in configuration");

    Ok(())
}

/// Clone an existing spore to create a sibling
pub async fn handle_spore_clone(
    from: PathBuf,
    to: PathBuf,
    node_id: String,
) -> Result<()> {
    println!("🔄 Cloning spore to create sibling...");
    println!("   Source: {}", from.display());
    println!("   Target: {}", to.display());
    println!("   New Node ID: {}", node_id);

    // Load source spore
    let source = Spore::from_path(from)?;
    println!("   Source label: {}", source.config().label);

    // Clone to create sibling
    let new_config = SporeConfig {
        label: format!("biomeOS-{}", node_id),
        node_id: node_id.clone(),
        spore_type: SporeType::default(), // Inherit from parent in clone_sibling
    };

    let sibling = source.clone_sibling(to, new_config).await?;

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
        println!("\n⚠️  Some checks failed. Review the details above.");
        std::process::exit(1);
    }

    Ok(())
}

/// Show spore information
pub async fn handle_spore_info(mount: PathBuf) -> Result<()> {
    println!("📊 Spore Information");
    println!("   Path: {}", mount.display());

    let spore = Spore::from_path(mount)?;

    println!("\n📝 Configuration:");
    println!("   Label: {}", spore.config().label);
    println!("   Node ID: {}", spore.config().node_id);
    println!("   Root: {}", spore.root_path().display());

    println!("\n📁 Structure:");
    let paths = [
        ".family.seed",
        "tower.toml",
        "bin/tower",
        "primals/beardog",
        "primals/songbird",
    ];

    for path in &paths {
        let full_path = spore.root_path().join(path);
        let exists = full_path.exists();
        let icon = if exists { "✅" } else { "❌" };
        println!("   {} {}", icon, path);

        if exists {
            if let Ok(metadata) = tokio::fs::metadata(&full_path).await {
                println!("      Size: {} bytes", metadata.len());

                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let perms = metadata.permissions();
                    let mode = perms.mode() & 0o777;
                    println!("      Permissions: {:o}", mode);
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
            println!("   Label: {}", label);
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


/// Refresh spore binaries from nucleusBin
pub async fn handle_spore_refresh(mount: PathBuf, dry_run: bool) -> Result<()> {
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
    
    // Load nucleus
    let nucleus_path = PathBuf::from("nucleusBin");
    if !nucleus_path.exists() {
        eprintln!("❌ Error: nucleusBin not found");
        eprintln!("   Expected at: {}", nucleus_path.display());
        eprintln!();
        eprintln!("💡 Run './scripts/harvest-primals.sh' to build binaries first");
        std::process::exit(1);
    }
    
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
                println!("   Current: v{}", binary.actual_version.as_ref().unwrap_or(&"unknown".to_string()));
                println!("   New:     v{}", binary.expected_version);
                println!();
            }
        }
        
        if !would_refresh {
            println!("✅ No binaries need refreshing - spore is already fresh!");
        } else {
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!();
            println!("💡 Run without --dry-run to apply these updates");
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
                    println!("   SHA256: {}...", &binary.new_sha256[..16]);
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
                println!("🎊 Spore refresh complete! {} binaries updated.", report.refreshed_binaries.len());
            } else {
                println!("⚠️  Spore refresh partially complete. {} succeeded, {} failed.",
                    report.refreshed_binaries.len(),
                    report.failed_binaries.len()
                );
            }
        }
    }
    
    println!();
    Ok(())
}
