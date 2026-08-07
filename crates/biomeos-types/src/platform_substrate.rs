// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! G68 Platform Substrate Abstraction — Layer 2 (Permissions).
//!
//! Provides [`PlatformAccess`] for cross-platform file permission semantics.
//! On Unix: maps to `chmod` mode bits via `PermissionsExt`.
//! On Windows: maps to readonly attribute (ACL support is future G68 Phase 2).
//!
//! # Usage
//! ```rust,no_run
//! use biomeos_types::platform_substrate::PlatformAccess;
//! PlatformAccess::Executable.apply("/path/to/binary").ok();
//! PlatformAccess::SecretFile.apply("/path/to/key").ok();
//! ```

use std::io;
use std::path::Path;

/// Platform-agnostic file access semantics.
///
/// Each variant encodes *intent* (what the file IS), not raw mode bits.
/// The platform layer translates intent to the appropriate mechanism.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlatformAccess {
    /// Executable binary or script: owner rwx, group rx, other rx (0o755)
    Executable,
    /// Private directory: owner rwx only (0o700)
    PrivateDir,
    /// Read-only data (immutable config, checksums): everyone read (0o444)
    ReadOnly,
    /// Secret file (keys, tokens): owner rw only (0o600)
    SecretFile,
    /// Socket directory: owner+group rwx (0o770)
    SocketDir,
    /// Custom mode for edge cases during migration. Avoid in new code.
    Custom(u32),
}

impl PlatformAccess {
    /// Apply this access level to a filesystem path.
    ///
    /// On Unix: sets mode bits via `std::fs::set_permissions`.
    /// On Windows: sets/clears the readonly attribute where semantically appropriate.
    pub fn apply(self, path: impl AsRef<Path>) -> io::Result<()> {
        self.apply_inner(path.as_ref())
    }

    #[cfg(unix)]
    fn apply_inner(self, path: &Path) -> io::Result<()> {
        use std::os::unix::fs::PermissionsExt;

        let mode = self.unix_mode();
        let perms = std::fs::Permissions::from_mode(mode);
        std::fs::set_permissions(path, perms)
    }

    #[cfg(not(unix))]
    fn apply_inner(self, path: &Path) -> io::Result<()> {
        let readonly = matches!(self, Self::ReadOnly | Self::SecretFile);
        let mut perms = std::fs::metadata(path)?.permissions();
        perms.set_readonly(readonly);
        std::fs::set_permissions(path, perms)
    }

    /// Returns the Unix mode bits for this access level.
    #[cfg(unix)]
    #[must_use]
    pub const fn unix_mode(self) -> u32 {
        match self {
            Self::Executable => 0o755,
            Self::PrivateDir => 0o700,
            Self::ReadOnly => 0o444,
            Self::SecretFile => 0o600,
            Self::SocketDir => 0o770,
            Self::Custom(m) => m,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn access_levels_have_correct_modes() {
        #[cfg(unix)]
        {
            assert_eq!(PlatformAccess::Executable.unix_mode(), 0o755);
            assert_eq!(PlatformAccess::PrivateDir.unix_mode(), 0o700);
            assert_eq!(PlatformAccess::ReadOnly.unix_mode(), 0o444);
            assert_eq!(PlatformAccess::SecretFile.unix_mode(), 0o600);
            assert_eq!(PlatformAccess::SocketDir.unix_mode(), 0o770);
            assert_eq!(PlatformAccess::Custom(0o644).unix_mode(), 0o644);
        }
    }

    #[test]
    fn apply_to_tempfile() {
        let dir = tempfile::tempdir().expect("tempdir");
        let file = dir.path().join("test.bin");
        std::fs::write(&file, b"test").expect("write");

        PlatformAccess::Executable.apply(&file).expect("apply");

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let meta = std::fs::metadata(&file).expect("metadata");
            assert_eq!(meta.permissions().mode() & 0o777, 0o755);
        }
    }

    #[test]
    fn apply_secret_file() {
        let dir = tempfile::tempdir().expect("tempdir");
        let file = dir.path().join("secret.key");
        std::fs::write(&file, b"secret").expect("write");

        PlatformAccess::SecretFile.apply(&file).expect("apply");

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let meta = std::fs::metadata(&file).expect("metadata");
            assert_eq!(meta.permissions().mode() & 0o777, 0o600);
        }
    }
}
