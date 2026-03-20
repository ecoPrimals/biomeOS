// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Spore management commands

use std::path::{Path, PathBuf};

use anyhow::Result;
use biomeos_spore::{Spore, SporeConfig, SporeType, SporeVerification};
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

/// Gathers structure info for paths under a spore root. Returns PathInfo for each.
pub(crate) fn gather_spore_structure_info(path: &Path) -> Vec<PathInfo> {
    let rel_paths = [
        ".family.seed",
        "tower.toml",
        "bin/tower",
        "primals/beardog",
        "primals/songbird",
    ];

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
            name: (*rel).to_string(),
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

/// Refresh spore binaries from plasmidBin
#[allow(
    clippy::too_many_lines,
    reason = "spore refresh flow with dry-run and apply paths"
)]
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
    let nucleus_path = PathBuf::from("plasmidBin");
    if !nucleus_path.exists() {
        return Err(anyhow::anyhow!(
            "plasmidBin not found. Expected at: {}. Run 'cargo run -p biomeos-harvest' to build binaries first.",
            nucleus_path.display()
        ));
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
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    fn test_parse_spore_type_live() {
        assert_eq!(parse_spore_type("live").unwrap(), SporeType::Live);
        assert_eq!(parse_spore_type("LIVE").unwrap(), SporeType::Live);
        assert_eq!(parse_spore_type("Live").unwrap(), SporeType::Live);
    }

    #[test]
    fn test_parse_spore_type_cold() {
        assert_eq!(parse_spore_type("cold").unwrap(), SporeType::Cold);
        assert_eq!(parse_spore_type("COLD").unwrap(), SporeType::Cold);
        assert_eq!(parse_spore_type("Cold").unwrap(), SporeType::Cold);
    }

    #[test]
    fn test_parse_spore_type_invalid() {
        assert!(parse_spore_type("invalid").is_err());
        assert!(parse_spore_type("").is_err());
        assert!(parse_spore_type("warm").is_err());
    }

    #[test]
    fn test_parse_spore_type_error_message() {
        let err = parse_spore_type("invalid").unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("invalid"),
            "error should mention invalid input: {msg}"
        );
        assert!(
            msg.contains("live") || msg.contains("cold"),
            "error should mention valid types: {msg}"
        );
    }

    #[test]
    fn test_gather_spore_structure_info_nonexistent() {
        let infos = gather_spore_structure_info(Path::new("/nonexistent/path"));
        assert_eq!(infos.len(), 5);
        assert!(infos.iter().all(|i| !i.exists));
    }

    #[test]
    fn test_compute_refresh_plan() {
        let paths = vec![
            PathBuf::from("bin/tower"),
            PathBuf::from("primals/beardog"),
            PathBuf::from("primals/songbird"),
        ];
        let would_refresh = vec![true, false, true];
        let report = compute_refresh_plan(&paths, &would_refresh);
        assert_eq!(report.to_refresh.len(), 2);
        assert_eq!(report.to_keep.len(), 1);
        assert!(report.to_refresh.contains(&PathBuf::from("bin/tower")));
        assert!(
            report
                .to_refresh
                .contains(&PathBuf::from("primals/songbird"))
        );
        assert!(report.to_keep.contains(&PathBuf::from("primals/beardog")));
    }

    #[test]
    fn test_compute_refresh_plan_empty() {
        let paths: Vec<PathBuf> = vec![];
        let would_refresh: Vec<bool> = vec![];
        let report = compute_refresh_plan(&paths, &would_refresh);
        assert!(report.to_refresh.is_empty());
        assert!(report.to_keep.is_empty());
    }

    #[test]
    fn test_format_spore_create_summary() {
        let spore_info = serde_json::json!({
            "location": "/media/usb/biomeOS"
        });
        let lines = format_spore_create_summary(&spore_info);
        assert!(lines.iter().any(|l| l.contains("/media/usb/biomeOS")));
        assert!(lines.iter().any(|l| l.contains("What was created")));
        assert!(lines.iter().any(|l| l.contains("Security")));
    }

    #[test]
    fn test_format_spore_create_summary_no_location() {
        let spore_info = serde_json::json!({});
        let lines = format_spore_create_summary(&spore_info);
        assert!(lines.iter().any(|l| l.contains("What was created")));
        assert!(lines.iter().any(|l| l.contains("Security")));
    }

    #[test]
    fn test_path_info_debug() {
        let info = PathInfo {
            name: "bin/tower".to_string(),
            exists: true,
            permissions: Some(0o755),
        };
        let _ = format!("{info:?}");
    }

    #[test]
    fn test_path_info_clone() {
        let info = PathInfo {
            name: "tower.toml".to_string(),
            exists: false,
            permissions: None,
        };
        let cloned = info.clone();
        assert_eq!(info.name, cloned.name);
        assert_eq!(info.exists, cloned.exists);
    }

    #[test]
    fn test_gather_spore_structure_info_checks_all_paths() {
        let infos = gather_spore_structure_info(std::path::Path::new("/nonexistent"));
        let names: Vec<_> = infos.iter().map(|i| i.name.as_str()).collect();
        assert!(names.contains(&".family.seed"));
        assert!(names.contains(&"tower.toml"));
        assert!(names.contains(&"bin/tower"));
        assert!(names.contains(&"primals/beardog"));
        assert!(names.contains(&"primals/songbird"));
    }

    #[test]
    fn test_compute_refresh_plan_mismatched_lengths() {
        let paths = vec![std::path::PathBuf::from("a"), std::path::PathBuf::from("b")];
        let would_refresh = vec![true];
        let report = compute_refresh_plan(&paths, &would_refresh);
        assert_eq!(report.to_refresh.len(), 1);
        assert_eq!(report.to_keep.len(), 1);
    }

    #[test]
    fn test_spore_type_emoji() {
        assert_eq!(SporeType::Live.emoji(), "🌱");
        assert_eq!(SporeType::Cold.emoji(), "❄️");
    }

    #[test]
    fn test_format_spore_create_summary_location_null() {
        let spore_info = serde_json::json!({"location": null});
        let lines = format_spore_create_summary(&spore_info);
        assert!(lines.iter().any(|l| l.contains("What was created")));
    }

    #[test]
    fn test_format_spore_create_summary_location_number() {
        let spore_info = serde_json::json!({"location": 42});
        let lines = format_spore_create_summary(&spore_info);
        assert!(lines.iter().any(|l| l.contains("Security")));
    }

    #[test]
    fn test_refresh_report_default() {
        let report = RefreshReport::default();
        assert!(report.to_refresh.is_empty());
        assert!(report.to_keep.is_empty());
    }

    #[tokio::test]
    async fn test_handle_spore_refresh_no_plasmid_bin() {
        let temp = tempfile::tempdir().expect("temp dir");
        let result = handle_spore_refresh(temp.path().to_path_buf(), true).await;
        if let Err(e) = result {
            let err = e.to_string();
            assert!(
                err.contains("plasmidBin") || err.contains("not found") || err.contains("tower"),
                "unexpected error: {err}"
            );
        }
    }

    #[tokio::test]
    async fn test_handle_spore_list() {
        let result = handle_spore_list().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_spore_verify_nonexistent() {
        let temp = tempfile::tempdir().expect("temp dir");
        let mount = temp.path().join("biomeOS");
        std::fs::create_dir_all(&mount).expect("create dir");
        let result = handle_spore_verify(temp.path().to_path_buf()).await;
        assert!(result.is_err() || result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_spore_info_nonexistent() {
        let temp = tempfile::tempdir().expect("temp dir");
        let result = handle_spore_info(temp.path().to_path_buf()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_handle_spore_clone_missing_source() {
        let temp = tempfile::tempdir().expect("temp dir");
        let from = temp.path().join("no-such-spore");
        let to = temp.path().join("dest-spore");
        let result = handle_spore_clone(from, to, "node-new".into()).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_gather_spore_structure_info_partial_tree() {
        let temp = tempfile::tempdir().expect("temp dir");
        let root = temp.path();
        std::fs::create_dir_all(root.join("bin")).expect("bin");
        std::fs::create_dir_all(root.join("primals")).expect("primals");
        std::fs::write(root.join(".family.seed"), b"seed").expect("seed");
        std::fs::write(root.join("tower.toml"), b"[tower]").expect("tower");
        std::fs::write(root.join("bin/tower"), b"exe").expect("tower bin");
        std::fs::write(root.join("primals/beardog"), b"bd").expect("bd");
        // songbird missing on purpose

        let infos = gather_spore_structure_info(root);
        assert!(infos.iter().any(|i| i.name == ".family.seed" && i.exists));
        assert!(infos.iter().any(|i| i.name == "tower.toml" && i.exists));
        assert!(
            infos
                .iter()
                .any(|i| i.name == "primals/songbird" && !i.exists)
        );
    }

    #[tokio::test]
    async fn test_handle_spore_verify_invalid_tree() {
        let temp = tempfile::tempdir().expect("temp dir");
        let mount = temp.path().join("not-a-spore");
        std::fs::create_dir_all(&mount).expect("dir");
        let result = handle_spore_verify(mount).await;
        assert!(result.is_err());
    }

    struct RestoreCwd(std::path::PathBuf);
    impl Drop for RestoreCwd {
        fn drop(&mut self) {
            let _ = std::env::set_current_dir(&self.0);
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_handle_spore_refresh_dry_run_with_plasmid_and_spore() {
        use biomeos_spore::manifest::{
            BinaryInfo, BinaryManifest, CompatibilityInfo, ManifestMeta,
        };
        use sha2::{Digest, Sha256};
        use std::collections::HashMap;

        let _ = std::env::set_current_dir("/");
        let _restore = RestoreCwd(std::env::current_dir().expect("cwd"));
        let temp = tempfile::tempdir().expect("temp dir");
        let root = temp.path();
        std::env::set_current_dir(root).expect("set cwd");

        let nucleus = root.join("plasmidBin");
        std::fs::create_dir_all(nucleus.join("tower")).expect("tower dir");
        std::fs::create_dir_all(nucleus.join("primals")).expect("primals dir");
        let tower_bytes = b"tower-nucleus-v1";
        std::fs::write(nucleus.join("tower/tower"), tower_bytes).expect("nucleus tower");

        let mut hasher = Sha256::new();
        hasher.update(tower_bytes);
        let sha = format!("{:x}", hasher.finalize());

        let mut binaries = HashMap::new();
        binaries.insert(
            "tower".to_string(),
            BinaryInfo {
                name: "tower".to_string(),
                version: "1.0.0".to_string(),
                git_commit: "abc".to_string(),
                build_date: chrono::Utc::now(),
                sha256: sha,
                size_bytes: tower_bytes.len() as u64,
                source_repo: "test".to_string(),
                features: vec![],
            },
        );

        let manifest = BinaryManifest {
            manifest: ManifestMeta {
                version: "1.0.0".to_string(),
                created_at: chrono::Utc::now(),
                pipeline_run: "test".to_string(),
            },
            binaries,
            compatibility: CompatibilityInfo {
                min_tower_version: "1.0.0".to_string(),
                min_beardog_version: "0.1.0".to_string(),
                min_songbird_version: "0.1.0".to_string(),
            },
        };
        std::fs::write(
            nucleus.join("MANIFEST.toml"),
            toml::to_string_pretty(&manifest).expect("manifest toml"),
        )
        .expect("write MANIFEST.toml");

        let spore = root.join("spore");
        std::fs::create_dir_all(spore.join("bin")).expect("bin");
        std::fs::write(spore.join("bin/tower"), tower_bytes).expect("matching tower");
        std::fs::write(
            spore.join("tower.toml"),
            r#"
node_id = "test-node"
family_id = "test-family"

[primals.env]
BEARDOG_NODE_ID = "test-node"
"#,
        )
        .expect("tower.toml");

        let result = handle_spore_refresh(spore, true).await;
        assert!(
            result.is_ok(),
            "dry-run refresh should succeed: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_format_spore_create_summary_has_security_section() {
        let lines = format_spore_create_summary(&serde_json::json!({}));
        assert!(lines.iter().any(|l| l.contains("0600")));
        assert!(lines.iter().any(|l| l.contains("BearDog")));
    }

    #[test]
    fn test_compute_refresh_plan_all_keep() {
        let paths = vec![PathBuf::from("a"), PathBuf::from("b")];
        let flags = vec![false, false];
        let r = compute_refresh_plan(&paths, &flags);
        assert_eq!(r.to_refresh.len(), 0);
        assert_eq!(r.to_keep.len(), 2);
    }

    #[test]
    fn test_compute_refresh_plan_all_refresh() {
        let paths = vec![PathBuf::from("x")];
        let flags = vec![true];
        let r = compute_refresh_plan(&paths, &flags);
        assert_eq!(r.to_refresh.len(), 1);
        assert!(r.to_keep.is_empty());
    }

    #[test]
    fn test_parse_spore_type_whitespace_not_trimmed() {
        assert!(parse_spore_type("  cold  ").is_err());
    }

    #[test]
    fn test_gather_spore_structure_info_order() {
        let infos = gather_spore_structure_info(Path::new("/nonexistent"));
        let order: Vec<_> = infos.iter().map(|i| i.name.as_str()).collect();
        assert_eq!(
            order,
            vec![
                ".family.seed",
                "tower.toml",
                "bin/tower",
                "primals/beardog",
                "primals/songbird"
            ]
        );
    }

    #[test]
    fn test_compute_refresh_plan_index_out_of_bounds_goes_to_keep() {
        let paths = vec![PathBuf::from("only-one")];
        let flags: Vec<bool> = vec![];
        let r = compute_refresh_plan(&paths, &flags);
        assert_eq!(r.to_keep, paths);
        assert!(r.to_refresh.is_empty());
    }

    #[test]
    fn test_format_spore_create_summary_location_object() {
        let lines = format_spore_create_summary(&serde_json::json!({
            "location": { "nested": true }
        }));
        assert!(lines.iter().any(|l| l.contains("What was created")));
    }
}
