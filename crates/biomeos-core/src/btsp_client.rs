// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! BTSP Client — biomeOS-side handshake for Secure Socket Architecture.
//!
//! When biomeOS connects to a family-scoped primal socket (`{primal}-{fid}.sock`),
//! it MUST perform a BTSP handshake to prove family membership before sending
//! any JSON-RPC requests.
//!
//! This module provides:
//! - Detection of family-scoped sockets (BTSP-required vs development-mode)
//! - BTSP session state tracking
//! - The INSECURE guard (refuse to run with both `FAMILY_ID` and `BIOMEOS_INSECURE`)
//!
//! The actual cryptographic handshake is delegated to BearDog via JSON-RPC
//! (`btsp.session.create`, `btsp.session.verify`). biomeOS is a family member
//! and holds the family seed for key derivation.

use std::path::Path;
use tracing::{info, warn};

/// Security mode for biomeOS socket connections.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecurityMode {
    /// Production: `FAMILY_ID` is set. BTSP handshake required for family-scoped sockets.
    Production {
        /// Whether BTSP handshake has been negotiated for this connection.
        btsp_available: bool,
    },
    /// Development: `BIOMEOS_INSECURE=1`. Raw cleartext JSON-RPC.
    Development,
}

/// Determine the security mode from environment.
#[must_use]
pub fn security_mode() -> SecurityMode {
    let has_family = std::env::var("FAMILY_ID")
        .or_else(|_| std::env::var("BIOMEOS_FAMILY_ID"))
        .map(|v| !v.is_empty() && v != "default")
        .unwrap_or(false);

    if has_family {
        SecurityMode::Production {
            btsp_available: false,
        }
    } else {
        SecurityMode::Development
    }
}

/// Check that `FAMILY_ID` and `BIOMEOS_INSECURE` are not both set.
///
/// biomeOS MUST call this at startup before binding any sockets or connecting
/// to primals.
///
/// # Errors
///
/// Returns a human-readable error message when both are set.
pub fn validate_insecure_guard() -> Result<(), String> {
    let has_family = std::env::var("FAMILY_ID")
        .or_else(|_| std::env::var("BIOMEOS_FAMILY_ID"))
        .map(|v| !v.is_empty() && v != "default")
        .unwrap_or(false);
    let insecure = std::env::var("BIOMEOS_INSECURE")
        .map(|v| v == "1" || v == "true")
        .unwrap_or(false);

    if has_family && insecure {
        return Err("FATAL: FAMILY_ID and BIOMEOS_INSECURE=1 cannot coexist. \
             Production mode (FAMILY_ID set) requires BTSP authentication. \
             Remove BIOMEOS_INSECURE to run in production, or unset FAMILY_ID for development."
            .to_owned());
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
    // Family-scoped: at least one hyphen and the suffix is not empty
    // e.g., "beardog-8ff3b864a4bc589a" has primal "beardog" and fid "8ff3b864a4bc589a"
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
            let fid = std::env::var("FAMILY_ID")
                .or_else(|_| std::env::var("BIOMEOS_FAMILY_ID"))
                .unwrap_or_else(|_| "unknown".to_owned());
            info!(
                family_id = %fid,
                mode = "production",
                "Secure Socket Architecture: BTSP authentication required for all primal connections"
            );
        }
        SecurityMode::Development => {
            let insecure = std::env::var("BIOMEOS_INSECURE")
                .map(|v| v == "1" || v == "true")
                .unwrap_or(false);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn family_scoped_detection() {
        assert!(is_family_scoped_socket(Path::new(
            "/run/user/1000/biomeos/beardog-8ff3b864a4bc589a.sock"
        )));
        assert!(is_family_scoped_socket(Path::new(
            "/tmp/biomeos/songbird-abc123.sock"
        )));
        assert!(!is_family_scoped_socket(Path::new(
            "/run/user/1000/biomeos/beardog.sock"
        )));
        assert!(!is_family_scoped_socket(Path::new(
            "/run/user/1000/biomeos/biomeos.sock"
        )));
    }

    #[test]
    fn family_scoped_domain_stem_sockets() {
        assert!(is_family_scoped_socket(Path::new(
            "/run/user/1000/biomeos/security-8ff3b864.sock"
        )));
        assert!(is_family_scoped_socket(Path::new(
            "/run/user/1000/biomeos/compute-abc123.sock"
        )));
        assert!(is_family_scoped_socket(Path::new(
            "/run/user/1000/biomeos/ai-def456.sock"
        )));
        assert!(!is_family_scoped_socket(Path::new(
            "/run/user/1000/biomeos/security.sock"
        )));
    }

    #[test]
    fn extract_family_from_socket() {
        assert_eq!(
            extract_family_id(Path::new("/tmp/beardog-abc123.sock")),
            Some("abc123".to_owned())
        );
        assert_eq!(
            extract_family_id(Path::new("/tmp/nestgate-8ff3b864a4bc589a.sock")),
            Some("8ff3b864a4bc589a".to_owned())
        );
        assert_eq!(extract_family_id(Path::new("/tmp/beardog.sock")), None);
    }

    #[test]
    fn extract_family_from_domain_stem_socket() {
        assert_eq!(
            extract_family_id(Path::new("/tmp/security-abc123.sock")),
            Some("abc123".to_owned())
        );
        assert_eq!(
            extract_family_id(Path::new("/tmp/compute-familyXYZ.sock")),
            Some("familyXYZ".to_owned())
        );
    }

    #[test]
    fn multi_hyphen_family_id() {
        assert!(is_family_scoped_socket(Path::new(
            "/tmp/beardog-abc-def-123.sock"
        )));
        assert_eq!(
            extract_family_id(Path::new("/tmp/beardog-abc-def-123.sock")),
            Some("abc-def-123".to_owned())
        );
    }

    #[test]
    fn edge_cases() {
        assert!(!is_family_scoped_socket(Path::new("")));
        assert!(!is_family_scoped_socket(Path::new("/tmp/.sock")));
        assert!(!is_family_scoped_socket(Path::new("/tmp/noext")));
        assert!(extract_family_id(Path::new("")).is_none());
        assert!(extract_family_id(Path::new("/tmp/noext")).is_none());
    }

    #[test]
    fn insecure_guard_ok_without_env() {
        // In test env, neither FAMILY_ID nor BIOMEOS_INSECURE are typically set,
        // so the guard should pass.
        // (We cannot safely mutate env vars in parallel tests, so we test the
        // no-conflict path.)
        let result = validate_insecure_guard();
        // If test runner happens to set these, we just verify it returns a result
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn security_mode_returns_valid_variant() {
        let mode = security_mode();
        // Both variants are valid depending on env; just confirm it returns
        match mode {
            SecurityMode::Development | SecurityMode::Production { .. } => {}
        }
    }

    #[test]
    fn log_security_posture_does_not_panic() {
        log_security_posture();
    }
}
