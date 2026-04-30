// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Family seed resolution from environment and runtime paths (trust lineage).

use bytes::Bytes;
use std::path::Path;
use tracing::{debug, warn};

/// Load family seed from secure storage for trust evaluation.
///
/// EVOLVED (Jan 27, 2026): Capability-based secure storage access
///
/// # Priority Sources
/// 1. `BIOMEOS_FAMILY_SEED` environment variable (base64-encoded)
/// 2. XDG runtime dir: `$XDG_RUNTIME_DIR/biomeos/family.seed`
/// 3. Empty (graceful degradation - results in Known trust level)
///
/// # Deep Debt Principle
/// Family seed is NOT hardcoded. It's discovered from the environment
/// or secure runtime storage. Missing seed results in reduced trust
/// rather than failure.
pub fn load_family_seed_from_storage() -> Bytes {
    let seed_b64 = std::env::var("BIOMEOS_FAMILY_SEED").ok();
    let runtime_dir = std::env::var("XDG_RUNTIME_DIR")
        .ok()
        .map(std::path::PathBuf::from);
    load_family_seed_from_storage_with(seed_b64.as_deref(), runtime_dir.as_deref(), true)
}

/// Same resolution as [`load_family_seed_from_storage`], but with injectable sources for tests.
///
/// `seed_b64` is the raw value of `BIOMEOS_FAMILY_SEED` (base64), not read from the environment.
/// `runtime_dir` is the XDG runtime directory root (file: `{runtime_dir}/biomeos/family.seed`).
/// When `include_uid_fallback` is false, the `/run/user/{uid}/biomeos/family.seed` tier is skipped.
pub fn load_family_seed_from_storage_with(
    seed_b64: Option<&str>,
    runtime_dir: Option<&Path>,
    include_uid_fallback: bool,
) -> Bytes {
    use base64::{Engine, engine::general_purpose::STANDARD};

    if let Some(seed_b64) = seed_b64 {
        if let Ok(seed) = STANDARD.decode(seed_b64) {
            debug!("Family seed loaded from BIOMEOS_FAMILY_SEED value");
            return Bytes::from(seed);
        }
        warn!("BIOMEOS_FAMILY_SEED set but invalid base64, ignoring");
    }

    if let Some(runtime_dir) = runtime_dir {
        let seed_path = runtime_dir.join("biomeos").join("family.seed");
        if let Ok(seed) = std::fs::read(&seed_path) {
            debug!(
                "Family seed loaded from XDG runtime dir: {}",
                seed_path.display()
            );
            return Bytes::from(seed);
        }
    }

    if include_uid_fallback {
        let uid_opt = std::env::var("UID")
            .or_else(|_| std::env::var("EUID"))
            .or_else(|_| {
                std::fs::read_to_string("/proc/self/loginuid").map(|s| s.trim().to_string())
            })
            .ok()
            .filter(|s| !s.is_empty());

        if let Some(uid) = uid_opt {
            let seed_path = std::path::PathBuf::from(format!(
                "{}/{uid}/biomeos/family.seed",
                biomeos_types::runtime_paths::LINUX_RUNTIME_DIR_PREFIX
            ));
            if let Ok(seed) = std::fs::read(&seed_path) {
                debug!(
                    "Family seed loaded from user runtime dir: {}",
                    seed_path.display()
                );
                return Bytes::from(seed);
            }
        }
    }

    debug!("No family seed available - trust evaluation will use Known level");
    Bytes::new()
}
