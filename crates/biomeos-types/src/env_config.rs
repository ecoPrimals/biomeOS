// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Centralized environment variable configuration
//!
//! Single source of truth for all environment variable names and typed accessors.
//! All env var access in the codebase should go through this module rather than
//! calling `std::env::var` with scattered string literals.
//!
//! # Usage
//!
//! ```ignore
//! use biomeos_types::env_config;
//!
//! let family = env_config::family_id();
//! let security = env_config::security_provider();
//! let socket_dir = env_config::socket_dir();
//! ```

use std::env;
use std::path::PathBuf;

/// Environment variable name constants
pub mod vars {
    // --- Identity & Family ---

    /// Family ID (preferred)
    pub const FAMILY_ID: &str = "BIOMEOS_FAMILY_ID";

    /// Family ID (legacy alias, checked as fallback)
    pub const FAMILY_ID_LEGACY: &str = "FAMILY_ID";

    // --- Provider Resolution ---

    /// Security provider primal name override (default: resolved via CapabilityTaxonomy)
    pub const SECURITY_PROVIDER: &str = "BIOMEOS_SECURITY_PROVIDER";

    /// Network/discovery provider primal name override
    pub const NETWORK_PROVIDER: &str = "BIOMEOS_NETWORK_PROVIDER";

    /// When set, disables fallback provider resolution (strict discovery only)
    pub const STRICT_DISCOVERY: &str = "BIOMEOS_STRICT_DISCOVERY";

    // --- Paths & Sockets ---

    /// Override for the socket directory
    pub const SOCKET_DIR: &str = "BIOMEOS_SOCKET_DIR";

    /// XDG runtime directory (standard)
    pub const XDG_RUNTIME_DIR: &str = "XDG_RUNTIME_DIR";

    /// Ecosystem-level primal/spring binary directory (`ecoPrimals/plasmidBin/`).
    /// Springs use this same path to spin up primals for local niche deployments.
    pub const PLASMID_BIN: &str = "ECOPRIMALS_PLASMID_BIN";

    /// biomeOS-local binary directory (fallback when ecosystem root isn't set)
    pub const PLASMID_BIN_DIR: &str = "BIOMEOS_PLASMID_BIN_DIR";

    /// GenomeBin path for distribution
    pub const GENOMEBIN_PATH: &str = "GENOMEBIN_PATH";

    // --- Per-Primal Sockets ---

    /// Neural API socket
    pub const NEURAL_API_SOCKET: &str = "NEURAL_API_SOCKET";

    /// BearDog socket
    pub const BEARDOG_SOCKET: &str = "BEARDOG_SOCKET";

    /// Songbird socket
    pub const SONGBIRD_SOCKET: &str = "SONGBIRD_SOCKET";
}

/// Get the family ID from environment (checks both `BIOMEOS_FAMILY_ID` and `FAMILY_ID`)
pub fn family_id() -> Option<String> {
    env::var(vars::FAMILY_ID)
        .ok()
        .or_else(|| env::var(vars::FAMILY_ID_LEGACY).ok())
}

/// Get the security provider name override, or `None` for taxonomy-based resolution
pub fn security_provider() -> Option<String> {
    env::var(vars::SECURITY_PROVIDER).ok()
}

/// Get the network provider name override, or `None` for taxonomy-based resolution
pub fn network_provider() -> Option<String> {
    env::var(vars::NETWORK_PROVIDER).ok()
}

/// Returns `true` if strict discovery mode is enabled (no fallback providers)
pub fn strict_discovery() -> bool {
    env::var(vars::STRICT_DISCOVERY).is_ok()
}

/// Get the socket directory override, or `None` for XDG-resolved default
pub fn socket_dir() -> Option<PathBuf> {
    env::var(vars::SOCKET_DIR).ok().map(PathBuf::from)
}

/// Get the XDG runtime directory
pub fn xdg_runtime_dir() -> Option<PathBuf> {
    env::var(vars::XDG_RUNTIME_DIR).ok().map(PathBuf::from)
}

/// Get the primal binary directory (tries ecosystem-level, then biomeOS-local)
pub fn plasmid_bin_dir() -> Option<PathBuf> {
    env::var(vars::PLASMID_BIN)
        .ok()
        .or_else(|| env::var(vars::PLASMID_BIN_DIR).ok())
        .map(PathBuf::from)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn test_family_id_not_set() {
        let _ = family_id();
    }

    #[test]
    fn test_strict_discovery_default() {
        env::remove_var(vars::STRICT_DISCOVERY);
        assert!(!strict_discovery());
    }

    #[test]
    fn test_vars_constants_are_consistent() {
        assert!(vars::FAMILY_ID.starts_with("BIOMEOS_"));
        assert!(vars::SECURITY_PROVIDER.starts_with("BIOMEOS_"));
        assert!(vars::NETWORK_PROVIDER.starts_with("BIOMEOS_"));
        assert!(vars::SOCKET_DIR.starts_with("BIOMEOS_"));
    }

    #[test]
    #[ignore = "env-var test is thread-unsafe; run with --test-threads=1"]
    fn test_family_id_biomeos_precedence() {
        env::set_var(vars::FAMILY_ID, "biomeos-family");
        env::set_var(vars::FAMILY_ID_LEGACY, "legacy-family");
        let id = family_id();
        env::remove_var(vars::FAMILY_ID);
        env::remove_var(vars::FAMILY_ID_LEGACY);
        assert_eq!(id, Some("biomeos-family".to_string()));
    }

    #[test]
    #[ignore = "env-var test is thread-unsafe; run with --test-threads=1"]
    fn test_family_id_legacy_fallback() {
        env::remove_var(vars::FAMILY_ID);
        env::set_var(vars::FAMILY_ID_LEGACY, "legacy-only");
        let id = family_id();
        env::remove_var(vars::FAMILY_ID_LEGACY);
        assert_eq!(id, Some("legacy-only".to_string()));
    }

    #[test]
    #[ignore = "env-var test is thread-unsafe; run with --test-threads=1"]
    fn test_security_provider() {
        env::set_var(vars::SECURITY_PROVIDER, "custom-security");
        let provider = security_provider();
        env::remove_var(vars::SECURITY_PROVIDER);
        assert_eq!(provider, Some("custom-security".to_string()));
    }

    #[test]
    #[ignore = "env-var test is thread-unsafe; run with --test-threads=1"]
    fn test_network_provider() {
        env::set_var(vars::NETWORK_PROVIDER, "custom-network");
        let provider = network_provider();
        env::remove_var(vars::NETWORK_PROVIDER);
        assert_eq!(provider, Some("custom-network".to_string()));
    }

    #[test]
    #[ignore = "env-var test is thread-unsafe; run with --test-threads=1"]
    fn test_strict_discovery_enabled() {
        env::set_var(vars::STRICT_DISCOVERY, "1");
        assert!(strict_discovery());
        env::remove_var(vars::STRICT_DISCOVERY);
    }

    #[test]
    #[ignore = "env-var test is thread-unsafe; run with --test-threads=1"]
    fn test_socket_dir() {
        env::set_var(vars::SOCKET_DIR, "/run/biomeos");
        let dir = socket_dir();
        env::remove_var(vars::SOCKET_DIR);
        assert_eq!(dir, Some(PathBuf::from("/run/biomeos")));
    }

    #[test]
    #[ignore = "env-var test is thread-unsafe; run with --test-threads=1"]
    fn test_xdg_runtime_dir() {
        env::set_var(vars::XDG_RUNTIME_DIR, "/tmp/xdg-test");
        let dir = xdg_runtime_dir();
        env::remove_var(vars::XDG_RUNTIME_DIR);
        assert_eq!(dir, Some(PathBuf::from("/tmp/xdg-test")));
    }

    #[test]
    #[ignore = "env-var test is thread-unsafe; run with --test-threads=1"]
    fn test_plasmid_bin_dir_ecoprimals() {
        env::set_var(vars::PLASMID_BIN, "/eco/plasmid");
        env::remove_var(vars::PLASMID_BIN_DIR);
        let dir = plasmid_bin_dir();
        env::remove_var(vars::PLASMID_BIN);
        assert_eq!(dir, Some(PathBuf::from("/eco/plasmid")));
    }

    #[test]
    #[ignore = "env-var test is thread-unsafe; run with --test-threads=1"]
    fn test_plasmid_bin_dir_biomeos_fallback() {
        env::remove_var(vars::PLASMID_BIN);
        env::set_var(vars::PLASMID_BIN_DIR, "/biomeos/bin");
        let dir = plasmid_bin_dir();
        env::remove_var(vars::PLASMID_BIN_DIR);
        assert_eq!(dir, Some(PathBuf::from("/biomeos/bin")));
    }

    #[test]
    fn test_vars_all_constants() {
        assert_eq!(vars::FAMILY_ID, "BIOMEOS_FAMILY_ID");
        assert_eq!(vars::FAMILY_ID_LEGACY, "FAMILY_ID");
        assert_eq!(vars::NEURAL_API_SOCKET, "NEURAL_API_SOCKET");
        assert_eq!(vars::BEARDOG_SOCKET, "BEARDOG_SOCKET");
        assert_eq!(vars::SONGBIRD_SOCKET, "SONGBIRD_SOCKET");
    }
}
