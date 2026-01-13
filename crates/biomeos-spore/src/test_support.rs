//! Test support utilities - ONLY compiled in test mode

use crate::error::SporeResult;
use std::fs;
use std::path::{Path, PathBuf};

/// Setup mock genetic material for testing
///
/// Creates minimal mock binaries in plasmidBin/ AND bin/ for test purposes
pub fn setup_test_binaries() -> SporeResult<PathBuf> {
    let project_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();

    // Change to project root so relative paths work
    std::env::set_current_dir(project_root)?;

    // Setup plasmidBin/
    let primal_bins = project_root.join("plasmidBin");
    fs::create_dir_all(&primal_bins)?;

    // Setup bin/ (for tower orchestrator)
    let bin_dir = project_root.join("bin");
    fs::create_dir_all(&bin_dir)?;

    // Create mock tower binary (in bin/ AND target/release/)
    let tower_bin = bin_dir.join("tower");
    fs::write(&tower_bin, "#!/bin/sh\necho 'Mock tower'\n")?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&tower_bin)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&tower_bin, perms)?;
    }

    // Also create in target/release/ (for spore copy_binaries)
    let target_release = project_root.join("target/release");
    fs::create_dir_all(&target_release)?;

    let tower_release = target_release.join("tower");
    fs::write(&tower_release, "#!/bin/sh\necho 'Mock tower'\n")?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&tower_release)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&tower_release, perms)?;
    }

    // Create mock beardog-server binary
    let beardog_bin = primal_bins.join("beardog-server");
    fs::write(&beardog_bin, "#!/bin/sh\necho 'Mock beardog-server'\n")?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&beardog_bin)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&beardog_bin, perms)?;
    }

    // Create mock songbird binary
    let songbird_bin = primal_bins.join("songbird");
    fs::write(&songbird_bin, "#!/bin/sh\necho 'Mock songbird'\n")?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&songbird_bin)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&songbird_bin, perms)?;
    }

    Ok(primal_bins)
}

/// Cleanup test binaries
pub fn cleanup_test_binaries() -> SporeResult<()> {
    let project_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();

    let primal_bins = project_root.join("plasmidBin");

    // Only remove if they're mock files
    let tower_bin = primal_bins.join("tower");
    if tower_bin.exists() {
        let content = fs::read_to_string(&tower_bin).unwrap_or_default();
        if content.contains("Mock tower") {
            fs::remove_file(tower_bin)?;
        }
    }

    let beardog_bin = primal_bins.join("beardog-server");
    if beardog_bin.exists() {
        let content = fs::read_to_string(&beardog_bin).unwrap_or_default();
        if content.contains("Mock beardog-server") {
            fs::remove_file(beardog_bin)?;
        }
    }

    let songbird_bin = primal_bins.join("songbird");
    if songbird_bin.exists() {
        let content = fs::read_to_string(&songbird_bin).unwrap_or_default();
        if content.contains("Mock songbird") {
            fs::remove_file(songbird_bin)?;
        }
    }

    Ok(())
}
