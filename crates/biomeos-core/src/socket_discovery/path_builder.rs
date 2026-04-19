// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Socket path building - 5-tier resolution per `PRIMAL_DEPLOYMENT_STANDARD`
//!
//! Extracted from engine.rs to keep files under 1000 lines.

use biomeos_types::constants::runtime_paths;
use std::env;
use std::path::{Path, PathBuf};

/// Build deterministic socket path for a primal with explicit overrides.
///
/// Implements 5-tier socket resolution per `PRIMAL_DEPLOYMENT_STANDARD`:
/// 1. Explicit override via `PRIMAL_SOCKET`
/// 2. XDG runtime directory
/// 3. Linux /run/user/$UID/biomeos/
/// 4. Android /data/local/tmp/biomeos/
/// 5. Fallback to /tmp/biomeos/
///
/// # Arguments
/// * `primal_name` - Name of the primal
/// * `family_id` - Family ID for namespace isolation
/// * `primal_socket` - Optional explicit socket path/dir override (Tier 1)
/// * `xdg_runtime_dir` - Optional `XDG_RUNTIME_DIR` override (Tier 2)
pub fn build_socket_path(
    primal_name: &str,
    family_id: &str,
    primal_socket: Option<&str>,
    xdg_runtime_dir: Option<&Path>,
) -> PathBuf {
    const BIOMEOS: &str = runtime_paths::BIOMEOS_SUBDIR;
    let socket_name = format!("{primal_name}-{family_id}.sock");

    // Tier 1: Explicit override via PRIMAL_SOCKET
    let primal_socket_val = primal_socket
        .map(String::from)
        .or_else(|| env::var("PRIMAL_SOCKET").ok());
    if let Some(primal_socket) = primal_socket_val {
        let path = PathBuf::from(&primal_socket);
        if path.is_dir() || !path.exists() {
            return path.join(&socket_name);
        }
        return path;
    }

    // Tier 2: XDG runtime directory
    let runtime_dir = xdg_runtime_dir
        .map(PathBuf::from)
        .or_else(get_xdg_runtime_dir);
    if let Some(runtime_dir) = runtime_dir {
        let biomeos_dir = runtime_dir.join("biomeos");
        std::fs::create_dir_all(&biomeos_dir).ok();
        return biomeos_dir.join(&socket_name);
    }

    // Tier 3: Linux /run/user/$UID/biomeos/
    if let Ok(uid) = env::var("UID") {
        let run_user = PathBuf::from(format!(
            "{}/{uid}/{BIOMEOS}",
            runtime_paths::LINUX_RUNTIME_DIR_PREFIX,
        ));
        if run_user.parent().is_some_and(std::path::Path::exists) {
            std::fs::create_dir_all(&run_user).ok();
            return run_user.join(&socket_name);
        }
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if let Ok(meta) = std::fs::metadata("/proc/self") {
            let uid = meta.uid();
            let run_user = PathBuf::from(format!(
                "{}/{uid}/{BIOMEOS}",
                runtime_paths::LINUX_RUNTIME_DIR_PREFIX,
            ));
            if run_user.parent().is_some_and(std::path::Path::exists) {
                std::fs::create_dir_all(&run_user).ok();
                return run_user.join(&socket_name);
            }
        }
    }

    // Tier 4: Android /data/local/tmp/biomeos/
    let android_dir = PathBuf::from(runtime_paths::ANDROID_RUNTIME_BASE);
    if android_dir.parent().is_some_and(std::path::Path::exists) {
        std::fs::create_dir_all(&android_dir).ok();
        return android_dir.join(&socket_name);
    }

    // Tier 5: Fallback to /tmp/biomeos/
    let fallback_dir = PathBuf::from(runtime_paths::FALLBACK_RUNTIME_BASE);
    std::fs::create_dir_all(&fallback_dir).ok();
    fallback_dir.join(&socket_name)
}

fn get_xdg_runtime_dir() -> Option<PathBuf> {
    env::var("XDG_RUNTIME_DIR")
        .ok()
        .map(PathBuf::from)
        .filter(|p| p.exists())
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test")]
mod tests {
    use super::build_socket_path;
    use std::fs::File;

    #[test]
    fn explicit_override_directory_appends_namespaced_socket() {
        let tmp = tempfile::tempdir().unwrap();
        let path = build_socket_path("beardog", "fam-1", Some(tmp.path().to_str().unwrap()), None);
        assert_eq!(
            path,
            tmp.path().join("beardog-fam-1.sock"),
            "directory override should append `<primal>-<family>.sock`"
        );
    }

    #[test]
    fn explicit_override_existing_regular_file_returns_that_path() {
        let tmp = tempfile::tempdir().unwrap();
        let sock = tmp.path().join("my.sock");
        File::create(&sock).unwrap();
        let path = build_socket_path("x", "y", Some(sock.to_str().unwrap()), None);
        assert_eq!(path, sock);
    }

    #[test]
    fn explicit_override_missing_path_treats_as_directory_and_joins_socket_name() {
        let tmp = tempfile::tempdir().unwrap();
        let missing = tmp.path().join("no_such_dir");
        let path = build_socket_path("p", "f", Some(missing.to_str().unwrap()), None);
        assert_eq!(path, missing.join("p-f.sock"));
    }

    #[test]
    fn xdg_runtime_override_skips_later_tiers() {
        let tmp = tempfile::tempdir().unwrap();
        let path = build_socket_path("songbird", "fam", None, Some(tmp.path()));
        assert_eq!(path, tmp.path().join("biomeos").join("songbird-fam.sock"));
    }

    #[test]
    fn primal_socket_param_wins_over_xdg_runtime_dir() {
        let xdg = tempfile::tempdir().unwrap();
        let tier1 = tempfile::tempdir().unwrap();
        let got = build_socket_path(
            "a",
            "b",
            Some(tier1.path().to_str().unwrap()),
            Some(xdg.path()),
        );
        assert_eq!(got, tier1.path().join("a-b.sock"));
    }

    #[test]
    fn empty_primal_name_still_produces_deterministic_suffix() {
        let tmp = tempfile::tempdir().unwrap();
        let p = build_socket_path("", "only-family", Some(tmp.path().to_str().unwrap()), None);
        assert_eq!(p, tmp.path().join("-only-family.sock"));
    }

    /// Tier-1 directory detection follows [`Path::is_dir`] (symlink to dir counts as dir).
    #[cfg(unix)]
    #[test]
    fn explicit_override_symlink_to_dir_behaves_like_directory() {
        use std::os::unix::fs::symlink;

        let tmp = tempfile::tempdir().unwrap();
        let real = tempfile::tempdir().unwrap();
        let link = tmp.path().join("link_dir");
        symlink(real.path(), &link).unwrap();
        let path = build_socket_path("p", "f", Some(link.to_str().unwrap()), None);
        assert_eq!(path, link.join("p-f.sock"));
    }

    /// Symlink to a regular file is not a directory — return the symlink path as the socket.
    #[cfg(unix)]
    #[test]
    fn explicit_override_symlink_to_file_returns_link_path() {
        use std::os::unix::fs::symlink;

        let tmp = tempfile::tempdir().unwrap();
        let real = tmp.path().join("real.sock");
        File::create(&real).unwrap();
        let link = tmp.path().join("link.sock");
        symlink(&real, &link).unwrap();
        let path = build_socket_path("p", "f", Some(link.to_str().unwrap()), None);
        assert_eq!(path, link);
    }

    #[test]
    fn socket_name_joins_primal_and_family_with_hyphen() {
        let tmp = tempfile::tempdir().unwrap();
        let got = build_socket_path(
            "alpha",
            "beta-gamma",
            Some(tmp.path().to_str().unwrap()),
            None,
        );
        assert_eq!(got, tmp.path().join("alpha-beta-gamma.sock"));
    }

    #[test]
    fn xdg_runtime_override_uses_biomeos_subdir_even_when_empty() {
        let tmp = tempfile::tempdir().unwrap();
        let path = build_socket_path("only", "fam", None, Some(tmp.path()));
        assert!(path.starts_with(tmp.path().join("biomeos")));
        assert_eq!(path.file_name().unwrap(), "only-fam.sock");
    }

    #[test]
    fn explicit_override_relative_file_path_returns_that_path() {
        let tmp = tempfile::tempdir().unwrap();
        let rel = tmp.path().join("rel.sock");
        File::create(&rel).unwrap();
        let path = build_socket_path("ignored", "ignored", Some(rel.to_str().unwrap()), None);
        assert_eq!(path, rel);
    }
}
