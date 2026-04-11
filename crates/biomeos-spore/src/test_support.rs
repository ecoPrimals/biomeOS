// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Test support utilities - ONLY compiled in test mode
//!
//! Creates the expected plasmidBin structure for tests.
//!
//! # Structure
//! ```text
//! plasmidBin/
//! ├── tower/
//! │   └── tower       # Tower orchestrator
//! └── primals/
//!     ├── beardog     # UniBin compliant primal binary
//!     └── songbird    # UniBin compliant primal binary
//! ```
//!
//! # Usage
//!
//! For isolated tests (recommended):
//! ```ignore
//! let temp_dir = TempDir::new().unwrap();
//! setup_test_binaries_at(temp_dir.path()).unwrap();
//! // Tests run with temp_dir as working directory
//! ```
//!
//! For tests using project root (checks for real binaries):
//! ```ignore
//! setup_test_binaries().unwrap();
//! // Uses existing plasmidBin if available
//! ```

use crate::error::SporeResult;
use std::fs;
use std::path::{Path, PathBuf};

/// Setup test binaries using project root's plasmidBin
///
/// This function:
/// 1. Changes to project root
/// 2. Verifies plasmidBin/tower/tower exists (real or mock)
/// 3. Verifies at least one primal exists in plasmidBin/primals/
/// 4. Creates minimal mocks only if NOTHING exists
///
/// For isolated testing, prefer `setup_test_binaries_at()` instead.
#[expect(
    clippy::unwrap_used,
    reason = "test setup: known path structure from CARGO_MANIFEST_DIR"
)]
pub fn setup_test_binaries() -> SporeResult<PathBuf> {
    let project_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();

    // Change to project root so relative paths work
    std::env::set_current_dir(project_root)?;

    let plasmid_bin = project_root.join("plasmidBin");
    let tower_dir = plasmid_bin.join("tower");
    let primals_dir = plasmid_bin.join("primals");

    // Ensure directories exist
    fs::create_dir_all(&tower_dir)?;
    fs::create_dir_all(&primals_dir)?;

    // Check tower - create mock only if it's missing or empty
    let tower_bin = tower_dir.join("tower");
    if !tower_bin.exists() {
        fs::write(&tower_bin, "#!/bin/sh\necho 'Mock tower'\n")?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&tower_bin)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&tower_bin, perms)?;
        }
    }

    // Check primals - we need at least one primal binary (file, not directory)
    let has_primal_binary = primals_dir
        .read_dir()?
        .filter_map(std::result::Result::ok)
        .any(|entry| {
            let path = entry.path();
            // Count files that are executable binaries (not directories, not dotfiles)
            path.is_file() && !entry.file_name().to_string_lossy().starts_with('.')
        });

    if !has_primal_binary {
        // No primal binaries found - create minimal mocks
        for primal in ["beardog", "songbird"] {
            let primal_path = primals_dir.join(primal);
            // Only create if it doesn't exist (might be a directory with versions)
            if !primal_path.exists() {
                fs::write(&primal_path, format!("#!/bin/sh\necho 'Mock {primal}'\n"))?;
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let mut perms = fs::metadata(&primal_path)?.permissions();
                    perms.set_mode(0o755);
                    fs::set_permissions(&primal_path, perms)?;
                }
            }
        }
    }

    Ok(plasmid_bin)
}

/// Setup plasmidBin at a custom directory for isolated tests
///
/// Creates a complete mock plasmidBin structure at the specified base directory.
/// This is the **preferred method** for testing as it:
/// - Doesn't modify the project root
/// - Creates a clean, predictable environment
/// - Avoids conflicts with existing directories
///
/// The working directory is changed to `base_dir` after setup.
///
/// # Example
/// ```ignore
/// let temp_dir = TempDir::new().unwrap();
/// setup_test_binaries_at(temp_dir.path()).unwrap();
/// // Now Spore::create can find plasmidBin/
/// let result = Spore::create(mount_point, config).await;
/// ```
pub fn setup_test_binaries_at(base_dir: &Path) -> SporeResult<PathBuf> {
    let plasmid_bin = base_dir.join("plasmidBin");
    let tower_dir = plasmid_bin.join("tower");
    let primals_dir = plasmid_bin.join("primals");

    fs::create_dir_all(&tower_dir)?;
    fs::create_dir_all(&primals_dir)?;

    // Create mock tower binary
    let tower_bin = tower_dir.join("tower");
    fs::write(&tower_bin, "#!/bin/sh\necho 'Mock tower'\n")?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&tower_bin)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&tower_bin, perms)?;
    }

    // Create mock primal binaries (UniBin compliant names)
    for primal in ["beardog", "songbird"] {
        let primal_bin = primals_dir.join(primal);
        fs::write(&primal_bin, format!("#!/bin/sh\necho 'Mock {primal}'\n"))?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&primal_bin)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&primal_bin, perms)?;
        }
    }

    Ok(plasmid_bin)
}

/// Cleanup mock test binaries from project root
///
/// Only removes files that contain "Mock" to avoid deleting real binaries.
/// Safe to call even if no mock binaries exist.
#[expect(
    clippy::unwrap_used,
    reason = "test setup: known path structure from CARGO_MANIFEST_DIR"
)]
pub fn cleanup_test_binaries() -> SporeResult<()> {
    let project_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();

    let plasmid_bin = project_root.join("plasmidBin");

    // Only remove if they're mock files
    let tower_bin = plasmid_bin.join("tower/tower");
    if tower_bin.exists() && tower_bin.is_file() {
        let content = fs::read_to_string(&tower_bin).unwrap_or_default();
        if content.contains("Mock tower") {
            fs::remove_file(tower_bin)?;
        }
    }

    let primals_dir = plasmid_bin.join("primals");
    for primal in ["beardog", "songbird"] {
        let primal_bin = primals_dir.join(primal);
        // Only remove if it's a file (not a directory) and contains "Mock"
        if primal_bin.exists() && primal_bin.is_file() {
            let content = fs::read_to_string(&primal_bin).unwrap_or_default();
            if content.contains("Mock") {
                fs::remove_file(primal_bin)?;
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    #![expect(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::*;
    use tempfile::TempDir;

    #[test]
    fn setup_test_binaries_at_creates_structure() {
        let temp = TempDir::new().unwrap();
        let plasmid_bin = setup_test_binaries_at(temp.path()).unwrap();

        assert!(plasmid_bin.join("tower/tower").exists());
        assert!(plasmid_bin.join("primals/beardog").exists());
        assert!(plasmid_bin.join("primals/songbird").exists());

        let tower_content = fs::read_to_string(plasmid_bin.join("tower/tower")).unwrap();
        assert!(tower_content.contains("Mock tower"));

        let beardog_content = fs::read_to_string(plasmid_bin.join("primals/beardog")).unwrap();
        assert!(beardog_content.contains("Mock beardog"));
    }

    #[test]
    fn setup_test_binaries_at_returns_correct_path() {
        let temp = TempDir::new().unwrap();
        let plasmid_bin = setup_test_binaries_at(temp.path()).unwrap();

        assert_eq!(plasmid_bin, temp.path().join("plasmidBin"));
    }

    #[test]
    fn setup_test_binaries_at_idempotent() {
        let temp = TempDir::new().unwrap();
        let first = setup_test_binaries_at(temp.path()).unwrap();
        let second = setup_test_binaries_at(temp.path()).unwrap();

        assert_eq!(first, second);
        assert!(first.join("tower/tower").exists());
    }

    #[test]
    fn cleanup_test_binaries_safe_when_no_mocks() {
        let result = cleanup_test_binaries();
        assert!(result.is_ok());
    }

    #[test]
    fn setup_test_binaries_succeeds() {
        let original_cwd = std::env::current_dir().unwrap();
        let result = setup_test_binaries();
        assert!(result.is_ok());
        let plasmid_bin = result.unwrap();
        assert!(plasmid_bin.join("tower").exists());
        assert!(plasmid_bin.join("primals").exists());
        let _ = std::env::set_current_dir(&original_cwd);
    }
}
