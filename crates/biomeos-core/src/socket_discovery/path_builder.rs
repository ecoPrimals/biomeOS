// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Socket path building - 5-tier resolution per PRIMAL_DEPLOYMENT_STANDARD
//!
//! Extracted from engine.rs to keep files under 1000 lines.

use std::env;
use std::path::{Path, PathBuf};

/// Build deterministic socket path for a primal with explicit overrides.
///
/// Implements 5-tier socket resolution per PRIMAL_DEPLOYMENT_STANDARD:
/// 1. Explicit override via PRIMAL_SOCKET
/// 2. XDG runtime directory
/// 3. Linux /run/user/$UID/biomeos/
/// 4. Android /data/local/tmp/biomeos/
/// 5. Fallback to /tmp/biomeos/
///
/// # Arguments
/// * `primal_name` - Name of the primal
/// * `family_id` - Family ID for namespace isolation
/// * `primal_socket` - Optional explicit socket path/dir override (Tier 1)
/// * `xdg_runtime_dir` - Optional XDG_RUNTIME_DIR override (Tier 2)
pub fn build_socket_path(
    primal_name: &str,
    family_id: &str,
    primal_socket: Option<&str>,
    xdg_runtime_dir: Option<&Path>,
) -> PathBuf {
    let socket_name = format!("{}-{}.sock", primal_name, family_id);

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
        let run_user = PathBuf::from(format!("/run/user/{uid}/biomeos"));
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
            let run_user = PathBuf::from(format!("/run/user/{uid}/biomeos"));
            if run_user.parent().is_some_and(std::path::Path::exists) {
                std::fs::create_dir_all(&run_user).ok();
                return run_user.join(&socket_name);
            }
        }
    }

    // Tier 4: Android /data/local/tmp/biomeos/
    let android_dir = PathBuf::from("/data/local/tmp/biomeos");
    if android_dir.parent().is_some_and(std::path::Path::exists) {
        std::fs::create_dir_all(&android_dir).ok();
        return android_dir.join(&socket_name);
    }

    // Tier 5: Fallback to /tmp/biomeos/
    let fallback_dir = PathBuf::from("/tmp/biomeos");
    std::fs::create_dir_all(&fallback_dir).ok();
    fallback_dir.join(&socket_name)
}

fn get_xdg_runtime_dir() -> Option<PathBuf> {
    env::var("XDG_RUNTIME_DIR")
        .ok()
        .map(PathBuf::from)
        .filter(|p| p.exists())
}
