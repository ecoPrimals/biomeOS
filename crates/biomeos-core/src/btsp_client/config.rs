// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use biomeos_types::defaults::DEFAULT_FAMILY_ID;
use biomeos_types::primal_names;
use std::path::Path;
use tracing::{info, warn};

use super::types::{BtspHandshakeError, SecurityMode};

/// Determine the security mode from environment.
#[must_use]
pub fn security_mode() -> SecurityMode {
    let has_family = has_family_id();

    if has_family {
        let btsp_available = security_provider_socket_path().is_some();
        SecurityMode::Production { btsp_available }
    } else {
        SecurityMode::Development
    }
}

/// Whether `FAMILY_ID` (or `BIOMEOS_FAMILY_ID`) is set to a non-default value.
#[must_use]
pub fn has_family_id() -> bool {
    std::env::var(biomeos_types::env_config::vars::FAMILY_ID_LEGACY)
        .or_else(|_| std::env::var(biomeos_types::env_config::vars::FAMILY_ID))
        .is_ok_and(|v| !v.is_empty() && v != DEFAULT_FAMILY_ID)
}

/// Read the family ID string from environment.
#[must_use]
pub fn family_id() -> Option<String> {
    std::env::var(biomeos_types::env_config::vars::FAMILY_ID_LEGACY)
        .or_else(|_| std::env::var(biomeos_types::env_config::vars::FAMILY_ID))
        .ok()
        .filter(|v| !v.is_empty() && v != DEFAULT_FAMILY_ID)
}

/// Whether BTSP enforcement is active. When `true`, connections from
/// clients that do not complete a BTSP handshake are rejected. When
/// `false`, unauthenticated connections log a warning but proceed.
///
/// Default: `true` when `FAMILY_ID` is set, `false` otherwise.
/// Override: `BIOMEOS_BTSP_ENFORCE=0` disables enforcement during rollout.
#[must_use]
pub fn btsp_enforce() -> bool {
    if !has_family_id() {
        return false;
    }
    std::env::var(biomeos_types::env_config::vars::BTSP_ENFORCE)
        .map_or(true, |v| v != "0" && v != "false")
}

/// Locate the security provider socket for BTSP delegation.
///
/// The security provider is resolved via `BIOMEOS_SECURITY_PROVIDER` (defaulting
/// to the canonical security provider constant from `primal_names`). This function
/// does not hardcode which primal provides security — it discovers the socket by
/// capability-based provider name.
///
/// Resolution order:
/// 1. `BIOMEOS_SECURITY_SOCKET` environment variable (explicit path)
/// 2. `SECURITY_PROVIDER_SOCKET` (capability-based, preferred)
/// 3. Family-scoped socket `{provider}-{fid}.sock` in socket dir
/// 4. Development socket `{provider}.sock` in socket dir
#[must_use]
pub fn security_provider_socket_path() -> Option<std::path::PathBuf> {
    for env_key in ["BIOMEOS_SECURITY_SOCKET", "SECURITY_PROVIDER_SOCKET"] {
        if let Ok(p) = std::env::var(env_key) {
            let path = std::path::PathBuf::from(&p);
            if path.exists() {
                return Some(path);
            }
        }
    }

    let provider = std::env::var(biomeos_types::env_config::vars::SECURITY_PROVIDER)
        .ok()
        .or_else(|| {
            biomeos_types::capability_taxonomy::CapabilityTaxonomy::resolve_to_primal("security")
                .map(String::from)
        })
        .unwrap_or_else(|| primal_names::BEARDOG.to_string());

    let socket_dir = socket_dir()?;
    if let Some(fid) = family_id() {
        let family_path = socket_dir.join(format!("{provider}-{fid}.sock"));
        if family_path.exists() {
            return Some(family_path);
        }
    }
    let dev_path = socket_dir.join(format!("{provider}.sock"));
    if dev_path.exists() {
        return Some(dev_path);
    }
    None
}

fn socket_dir() -> Option<std::path::PathBuf> {
    if let Ok(dir) = std::env::var(biomeos_types::env_config::vars::SOCKET_DIR) {
        return Some(std::path::PathBuf::from(dir));
    }
    if let Ok(runtime) = std::env::var(biomeos_types::env_config::vars::XDG_RUNTIME_DIR) {
        let dir = std::path::PathBuf::from(runtime)
            .join(biomeos_types::constants::runtime_paths::BIOMEOS_SUBDIR);
        if dir.is_dir() {
            return Some(dir);
        }
    }
    None
}

/// Check that `FAMILY_ID` and `BIOMEOS_INSECURE` are not both set.
///
/// biomeOS MUST call this at startup before binding any sockets or connecting
/// to primals.
///
/// # Errors
///
/// Returns a human-readable error message when both are set.
pub fn validate_insecure_guard() -> Result<(), BtspHandshakeError> {
    let has_family = std::env::var(biomeos_types::env_config::vars::FAMILY_ID_LEGACY)
        .or_else(|_| std::env::var(biomeos_types::env_config::vars::FAMILY_ID))
        .is_ok_and(|v| !v.is_empty() && v != DEFAULT_FAMILY_ID);
    let insecure = std::env::var(biomeos_types::env_config::vars::INSECURE)
        .is_ok_and(|v| v == "1" || v == "true");

    if has_family && insecure {
        return Err(BtspHandshakeError::InsecureGuard);
    }
    Ok(())
}

/// Detect whether a socket path is family-scoped (requires BTSP handshake).
///
/// Family-scoped sockets match the pattern `{primal}-{family_id}.sock`.
/// Non-family sockets are `{primal}.sock` (development mode).
#[must_use]
pub fn is_family_scoped_socket(path: &Path) -> bool {
    let Some(filename) = path.file_name().and_then(|f| f.to_str()) else {
        return false;
    };
    let Some(stem) = filename.strip_suffix(".sock") else {
        return false;
    };
    // Family-scoped: `{canonical_primal_id}-{family_id}.sock` (see `primal_names`).
    stem.contains('-') && stem.split('-').count() >= 2
}

/// Extract the family ID from a family-scoped socket path.
///
/// Returns `None` if the socket is not family-scoped.
#[must_use]
pub fn extract_family_id(path: &Path) -> Option<String> {
    let filename = path.file_name()?.to_str()?;
    let stem = filename.strip_suffix(".sock")?;
    let dash_pos = stem.find('-')?;
    Some(stem[dash_pos + 1..].to_owned())
}

/// Log the security posture at startup.
pub fn log_security_posture() {
    match security_mode() {
        SecurityMode::Production { .. } => {
            let fid = std::env::var(biomeos_types::env_config::vars::FAMILY_ID_LEGACY)
                .or_else(|_| std::env::var(biomeos_types::env_config::vars::FAMILY_ID))
                .unwrap_or_else(|_| "unknown".to_owned());
            info!(
                family_id = %fid,
                mode = "production",
                "Secure Socket Architecture: BTSP authentication required for all primal connections"
            );
        }
        SecurityMode::Development => {
            let insecure = std::env::var(biomeos_types::env_config::vars::INSECURE)
                .is_ok_and(|v| v == "1" || v == "true");
            if insecure {
                warn!("INSECURE MODE — no BTSP authentication. Development only.");
            } else {
                info!(
                    mode = "standalone",
                    "No FAMILY_ID set — running in development/standalone mode"
                );
            }
        }
    }
}
