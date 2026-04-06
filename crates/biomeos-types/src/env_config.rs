// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

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

use std::collections::HashMap;
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

    /// Security provider primal name override (default: resolved via `CapabilityTaxonomy`)
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

    /// `GenomeBin` path for distribution
    pub const GENOMEBIN_PATH: &str = "GENOMEBIN_PATH";

    // --- Per-Primal Sockets ---

    /// Neural API socket
    pub const NEURAL_API_SOCKET: &str = "NEURAL_API_SOCKET";

    /// `BearDog` socket
    pub const BEARDOG_SOCKET: &str = "BEARDOG_SOCKET";

    /// Songbird socket
    pub const SONGBIRD_SOCKET: &str = "SONGBIRD_SOCKET";
}

/// Get the family ID from environment (checks both `BIOMEOS_FAMILY_ID` and `FAMILY_ID`)
#[must_use]
pub fn family_id() -> Option<String> {
    family_id_with(&env::vars().collect::<HashMap<_, _>>())
}

/// Get the family ID from an explicit environment map (for testing / DI)
fn family_id_with(env: &HashMap<String, String>) -> Option<String> {
    env.get(vars::FAMILY_ID)
        .cloned()
        .or_else(|| env.get(vars::FAMILY_ID_LEGACY).cloned())
}

/// Get the security provider name override, or `None` for taxonomy-based resolution
#[must_use]
pub fn security_provider() -> Option<String> {
    security_provider_with(&env::vars().collect::<HashMap<_, _>>())
}

/// Get the security provider from an explicit environment map (for testing / DI)
fn security_provider_with(env: &HashMap<String, String>) -> Option<String> {
    env.get(vars::SECURITY_PROVIDER).cloned()
}

/// Get the network provider name override, or `None` for taxonomy-based resolution
#[must_use]
pub fn network_provider() -> Option<String> {
    network_provider_with(&env::vars().collect::<HashMap<_, _>>())
}

/// Get the network provider from an explicit environment map (for testing / DI)
fn network_provider_with(env: &HashMap<String, String>) -> Option<String> {
    env.get(vars::NETWORK_PROVIDER).cloned()
}

/// Returns `true` if strict discovery mode is enabled (no fallback providers)
#[must_use]
pub fn strict_discovery() -> bool {
    strict_discovery_with(&env::vars().collect::<HashMap<_, _>>())
}

/// Check strict discovery from an explicit environment map (for testing / DI)
fn strict_discovery_with(env: &HashMap<String, String>) -> bool {
    env.contains_key(vars::STRICT_DISCOVERY)
}

/// Get the socket directory override, or `None` for XDG-resolved default
#[must_use]
pub fn socket_dir() -> Option<PathBuf> {
    socket_dir_with(&env::vars().collect::<HashMap<_, _>>())
}

/// Get the socket directory from an explicit environment map (for testing / DI)
fn socket_dir_with(env: &HashMap<String, String>) -> Option<PathBuf> {
    env.get(vars::SOCKET_DIR).map(|s| PathBuf::from(s.as_str()))
}

/// Get the XDG runtime directory
#[must_use]
pub fn xdg_runtime_dir() -> Option<PathBuf> {
    xdg_runtime_dir_with(&env::vars().collect::<HashMap<_, _>>())
}

/// Get the XDG runtime directory from an explicit environment map (for testing / DI)
fn xdg_runtime_dir_with(env: &HashMap<String, String>) -> Option<PathBuf> {
    env.get(vars::XDG_RUNTIME_DIR)
        .map(|s| PathBuf::from(s.as_str()))
}

/// Get the primal binary directory (tries ecosystem-level, then biomeOS-local)
#[must_use]
pub fn plasmid_bin_dir() -> Option<PathBuf> {
    plasmid_bin_dir_with(&env::vars().collect::<HashMap<_, _>>())
}

/// Get the plasmid bin directory from an explicit environment map (for testing / DI)
fn plasmid_bin_dir_with(env: &HashMap<String, String>) -> Option<PathBuf> {
    env.get(vars::PLASMID_BIN)
        .or_else(|| env.get(vars::PLASMID_BIN_DIR))
        .map(|s| PathBuf::from(s.as_str()))
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn test_family_id_not_set() {
        let _ = family_id();
    }

    #[test]
    fn test_strict_discovery_default() {
        let env: HashMap<String, String> = HashMap::new();
        assert!(!strict_discovery_with(&env));
    }

    #[test]
    fn test_vars_constants_are_consistent() {
        assert!(vars::FAMILY_ID.starts_with("BIOMEOS_"));
        assert!(vars::SECURITY_PROVIDER.starts_with("BIOMEOS_"));
        assert!(vars::NETWORK_PROVIDER.starts_with("BIOMEOS_"));
        assert!(vars::SOCKET_DIR.starts_with("BIOMEOS_"));
    }

    #[test]
    fn test_family_id_biomeos_precedence() {
        let mut env = HashMap::new();
        env.insert(vars::FAMILY_ID.to_string(), "biomeos-family".to_string());
        env.insert(
            vars::FAMILY_ID_LEGACY.to_string(),
            "legacy-family".to_string(),
        );
        assert_eq!(family_id_with(&env), Some("biomeos-family".to_string()));
    }

    #[test]
    fn test_family_id_legacy_fallback() {
        let mut env = HashMap::new();
        env.insert(
            vars::FAMILY_ID_LEGACY.to_string(),
            "legacy-only".to_string(),
        );
        assert_eq!(family_id_with(&env), Some("legacy-only".to_string()));
    }

    #[test]
    fn test_security_provider() {
        let mut env = HashMap::new();
        env.insert(
            vars::SECURITY_PROVIDER.to_string(),
            "custom-security".to_string(),
        );
        assert_eq!(
            security_provider_with(&env),
            Some("custom-security".to_string())
        );
    }

    #[test]
    fn test_network_provider() {
        let mut env = HashMap::new();
        env.insert(
            vars::NETWORK_PROVIDER.to_string(),
            "custom-network".to_string(),
        );
        assert_eq!(
            network_provider_with(&env),
            Some("custom-network".to_string())
        );
    }

    #[test]
    fn test_strict_discovery_enabled() {
        let mut env = HashMap::new();
        env.insert(vars::STRICT_DISCOVERY.to_string(), "1".to_string());
        assert!(strict_discovery_with(&env));
    }

    #[test]
    fn test_socket_dir() {
        let mut env = HashMap::new();
        env.insert(vars::SOCKET_DIR.to_string(), "/run/biomeos".to_string());
        assert_eq!(socket_dir_with(&env), Some(PathBuf::from("/run/biomeos")));
    }

    #[test]
    fn test_xdg_runtime_dir() {
        let mut env = HashMap::new();
        env.insert(
            vars::XDG_RUNTIME_DIR.to_string(),
            "/tmp/xdg-test".to_string(),
        );
        assert_eq!(
            xdg_runtime_dir_with(&env),
            Some(PathBuf::from("/tmp/xdg-test"))
        );
    }

    #[test]
    fn test_plasmid_bin_dir_ecoprimals() {
        let mut env = HashMap::new();
        env.insert(vars::PLASMID_BIN.to_string(), "/eco/plasmid".to_string());
        assert_eq!(
            plasmid_bin_dir_with(&env),
            Some(PathBuf::from("/eco/plasmid"))
        );
    }

    #[test]
    fn test_plasmid_bin_dir_biomeos_fallback() {
        let mut env = HashMap::new();
        env.insert(
            vars::PLASMID_BIN_DIR.to_string(),
            "/biomeos/bin".to_string(),
        );
        assert_eq!(
            plasmid_bin_dir_with(&env),
            Some(PathBuf::from("/biomeos/bin"))
        );
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
