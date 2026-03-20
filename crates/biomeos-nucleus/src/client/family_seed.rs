// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Family seed resolution from environment and runtime paths (trust lineage).

use bytes::Bytes;
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
pub(crate) fn load_family_seed_from_storage() -> Bytes {
    use base64::{Engine, engine::general_purpose::STANDARD};

    // Priority 1: Environment variable (for bootstrap/testing)
    if let Ok(seed_b64) = std::env::var("BIOMEOS_FAMILY_SEED") {
        if let Ok(seed) = STANDARD.decode(&seed_b64) {
            debug!("Family seed loaded from BIOMEOS_FAMILY_SEED environment");
            return Bytes::from(seed);
        }
        warn!("BIOMEOS_FAMILY_SEED set but invalid base64, ignoring");
    }

    // Priority 2: XDG runtime directory (secure runtime storage)
    if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
        let seed_path = std::path::Path::new(&runtime_dir)
            .join("biomeos")
            .join("family.seed");
        if let Ok(seed) = std::fs::read(&seed_path) {
            debug!(
                "Family seed loaded from XDG runtime dir: {}",
                seed_path.display()
            );
            return Bytes::from(seed);
        }
    }

    // Priority 3: User-specific runtime dir (fallback)
    if let Ok(uid) = std::env::var("UID").or_else(|_| {
        // Try to get UID from /proc on Linux
        std::fs::read_to_string("/proc/self/loginuid")
            .map(|s| s.trim().to_string())
            .or_else(|_| Ok::<_, std::io::Error>("1000".to_string()))
    }) {
        let seed_path = std::path::PathBuf::from(format!("/run/user/{uid}/biomeos/family.seed"));
        if let Ok(seed) = std::fs::read(&seed_path) {
            debug!(
                "Family seed loaded from user runtime dir: {}",
                seed_path.display()
            );
            return Bytes::from(seed);
        }
    }

    // Graceful degradation: no seed available, trust evaluation will use Known level
    debug!("No family seed available - trust evaluation will use Known level");
    Bytes::new()
}
