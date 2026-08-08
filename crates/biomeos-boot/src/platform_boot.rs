// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! G68 Platform Boot Abstractions.
//!
//! Provides platform-gated wrappers for Linux-specific boot operations:
//! - [`platform_link`] — cross-platform symlink creation
//! - [`query_access`] — permission mode query (read-only, no mutation)
//! - [`platform_mount`] — filesystem mount abstraction
//! - [`platform_mknod`] — device node creation
//!
//! These exist because `biomeos-boot` is inherently Linux-targeted (PID 1 init),
//! but G68 requires all platform-specific calls to flow through named abstractions
//! for auditability and future portability.

use crate::init_error::{BootError, Result};
use std::path::Path;

/// Create a symbolic link (L1 abstraction).
///
/// On Unix: delegates to `std::os::unix::fs::symlink`.
/// On other platforms: returns an error (symlinks are platform-specific in boot context).
pub fn platform_link(target: impl AsRef<Path>, link_path: impl AsRef<Path>) -> Result<()> {
    let target = target.as_ref();
    let link_path = link_path.as_ref();

    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(target, link_path).map_err(|e| BootError::DeviceCreation {
            device: link_path.to_string_lossy().to_string(),
            error: format!(
                "symlink {} -> {}: {e}",
                link_path.display(),
                target.display()
            ),
        })?;
    }

    #[cfg(not(unix))]
    {
        return Err(BootError::DeviceCreation {
            device: link_path.to_string_lossy().to_string(),
            error: "symlinks not supported on this platform for boot context".to_string(),
        });
    }

    Ok(())
}

/// Query the Unix permission mode of a file (L2 read abstraction).
///
/// Returns the raw `u32` mode bits on Unix, or `0o755` as a default on
/// other platforms (all files assumed executable in non-Unix boot context).
#[must_use]
pub fn query_access(path: &Path) -> u32 {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::metadata(path)
            .map(|m| m.permissions().mode() & 0o777)
            .unwrap_or(0)
    }

    #[cfg(not(unix))]
    {
        let _ = path;
        0o755
    }
}

/// Check whether a file is executable (L2 read abstraction).
#[must_use]
pub fn is_executable(path: &Path) -> bool {
    query_access(path) & 0o111 != 0
}

/// Platform-gated filesystem mount (L3 abstraction).
///
/// On Linux: delegates to `rustix::mount::mount`.
/// On other platforms: no-op (returns Ok).
pub fn platform_mount(source: &str, target: &Path, fstype: &str, flags: u32) -> Result<()> {
    #[cfg(target_os = "linux")]
    {
        use rustix::mount::{MountFlags, mount};

        let mount_flags = MountFlags::from_bits_retain(flags);

        mount(source, target, fstype, mount_flags, None::<&std::ffi::CStr>).map_err(|e| {
            BootError::MountFailed {
                target: target.to_string_lossy().to_string(),
                fs_source: source.to_string(),
                errno: e,
            }
        })?;
    }

    #[cfg(not(target_os = "linux"))]
    {
        let _ = (source, target, fstype, flags);
    }

    Ok(())
}

/// Platform-gated device node creation (L3 abstraction).
pub fn platform_mknod(path: &str, major: u32, minor: u32) -> Result<()> {
    #[cfg(target_os = "linux")]
    {
        use rustix::fs::{CWD, FileType, Mode, makedev, mknodat};

        let mode = Mode::from_raw_mode(0o666);
        mknodat(
            CWD,
            path,
            FileType::CharacterDevice,
            mode,
            makedev(major, minor),
        )
        .map_err(|e| BootError::DeviceCreation {
            device: path.to_string(),
            error: format!("mknod({path}, {major}:{minor}): {e}"),
        })?;
    }

    #[cfg(not(target_os = "linux"))]
    {
        let _ = (path, major, minor);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn query_access_nonexistent_returns_zero() {
        assert_eq!(query_access(Path::new("/nonexistent/path/xyz")), 0);
    }

    #[test]
    fn is_executable_checks_mode_bits() {
        let dir = tempfile::tempdir().expect("tempdir");
        let file = dir.path().join("test.sh");
        std::fs::write(&file, "#!/bin/sh").expect("write");

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o644)).expect("chmod");
            assert!(!is_executable(&file));

            std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o755)).expect("chmod");
            assert!(is_executable(&file));
        }
    }

    #[cfg(unix)]
    #[test]
    fn platform_link_creates_symlink() {
        let dir = tempfile::tempdir().expect("tempdir");
        let target = dir.path().join("target.txt");
        let link = dir.path().join("link.txt");
        std::fs::write(&target, "hello").expect("write");

        platform_link(&target, &link).expect("symlink");
        assert!(link.is_symlink());
    }
}
