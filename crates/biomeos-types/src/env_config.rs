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

    // --- Transport & Binding ---

    /// Primal bind mode override. When set to `tcp_only`, all primals skip UDS
    /// `bind()` and serve via TCP only. Required for SELinux/Android substrates
    /// where `sock_file create` is denied.
    pub const PRIMAL_BIND_MODE: &str = "PRIMAL_BIND_MODE";

    // --- Operational Mode ---

    /// Bind address for HTTP/TCP listeners
    pub const BIND_ADDRESS: &str = "BIOMEOS_BIND_ADDRESS";

    /// Operational mode (nucleus, deploy, etc.)
    pub const MODE: &str = "BIOMEOS_MODE";

    /// Authentication mode (ionic, passthrough, etc.)
    pub const AUTH_MODE: &str = "BIOMEOS_AUTH_MODE";

    /// Node identifier
    pub const NODE_ID: &str = "BIOMEOS_NODE_ID";

    /// Legacy node ID (checked as fallback)
    pub const NODE_ID_LEGACY: &str = "NODE_ID";

    // --- Discovery & Registration ---

    /// Discovery provider override
    pub const DISCOVERY_PROVIDER: &str = "DISCOVERY_PROVIDER";

    /// Registry provider override
    pub const REGISTRY_PROVIDER: &str = "BIOMEOS_REGISTRY_PROVIDER";

    /// Storage provider override
    pub const STORAGE_PROVIDER: &str = "BIOMEOS_STORAGE_PROVIDER";

    /// Allow loopback discovery (dev/test)
    pub const ALLOW_LOOPBACK: &str = "BIOMEOS_ALLOW_LOOPBACK_DISCOVERY";

    /// Skip mDNS probing
    pub const SKIP_MDNS_PROBE: &str = "BIOMEOS_SKIP_MDNS_PROBE";

    // --- Security ---

    /// BTSP insecure mode (dev only — degrades security)
    pub const INSECURE: &str = "BIOMEOS_INSECURE";

    /// BTSP enforcement mode
    pub const BTSP_ENFORCE: &str = "BIOMEOS_BTSP_ENFORCE";

    /// Songbird federation (mesh relay) enabled
    pub const FEDERATION_ENABLED: &str = "SONGBIRD_FEDERATION_ENABLED";

    /// Songbird mesh port
    pub const MESH_PORT: &str = "SONGBIRD_MESH_PORT";

    /// Songbird HTTP port
    pub const HTTP_PORT: &str = "SONGBIRD_HTTP_PORT";

    // --- Manifest & Gate Identity ---

    /// Path to `ecosystem_manifest.toml` (WaterFall catalog)
    pub const ECOSYSTEM_MANIFEST_PATH: &str = "BIOMEOS_ECOSYSTEM_MANIFEST";

    /// Local gate identifier (e.g. `eastGate`, `southGate`)
    pub const GATE_ID: &str = "BIOMEOS_GATE_ID";

    // --- Runtime & Deployment ---

    /// Runtime directory override (socket/PID directory)
    pub const RUNTIME_DIR: &str = "BIOMEOS_RUNTIME_DIR";

    /// Deployment mode override
    pub const DEPLOYMENT_MODE: &str = "BIOMEOS_DEPLOYMENT_MODE";

    /// JWT secret for inter-primal authentication
    pub const JWT_SECRET: &str = "BIOMEOS_JWT_SECRET";

    /// Node family ID (alias for BIOMEOS_FAMILY_ID in some contexts)
    pub const NODE_FAMILY_ID: &str = "NODE_FAMILY_ID";

    /// Discovery socket path override
    pub const DISCOVERY_SOCKET: &str = "BIOMEOS_DISCOVERY_SOCKET";

    /// AI provider override (e.g. squirrel, claude)
    pub const AI_PROVIDER: &str = "BIOMEOS_AI_PROVIDER";

    /// Port override for the Neural API HTTP listener
    pub const PORT: &str = "BIOMEOS_PORT";

    // --- Paths & Directories ---

    /// Install directory override
    pub const INSTALL_DIR: &str = "BIOMEOS_INSTALL_DIR";

    /// Media path override
    pub const MEDIA_PATH: &str = "BIOMEOS_MEDIA_PATH";

    /// Log directory override
    pub const LOG_DIR: &str = "BIOMEOS_LOG_DIR";

    /// Persistence mode override
    pub const PERSISTENCE: &str = "BIOMEOS_PERSISTENCE";

    /// Version override
    pub const VERSION: &str = "BIOMEOS_VERSION";

    /// Isolation mode override
    pub const ISOLATION: &str = "BIOMEOS_ISOLATION";

    // --- Discovery Endpoints ---

    /// Discovery endpoint URL override
    pub const DISCOVERY_ENDPOINT: &str = "DISCOVERY_ENDPOINT";

    /// Legacy discovery endpoint (prefixed variant)
    pub const BIOMEOS_DISCOVERY_ENDPOINT: &str = "BIOMEOS_DISCOVERY_ENDPOINT";

    /// Discovery port override
    pub const DISCOVERY_PORT: &str = "BIOMEOS_DISCOVERY_PORT";

    /// Test bind address override
    pub const TEST_BIND: &str = "BIOMEOS_TEST_BIND";

    /// Test port override
    pub const TEST_PORT: &str = "BIOMEOS_TEST_PORT";

    /// Family seed for key derivation
    pub const FAMILY_SEED: &str = "BIOMEOS_FAMILY_SEED";

    // --- Realtime / Streaming ---

    /// WebSocket endpoint override
    pub const WS_ENDPOINT: &str = "BIOMEOS_WS_ENDPOINT";

    /// SSE endpoint override
    pub const SSE_ENDPOINT: &str = "BIOMEOS_SSE_ENDPOINT";

    /// API WebSocket endpoint override
    pub const API_WS: &str = "BIOMEOS_API_WS";

    /// API SSE endpoint override
    pub const API_SSE: &str = "BIOMEOS_API_SSE";

    // --- STUN ---

    /// Self-hosted STUN server address
    pub const STUN_SERVER: &str = "BIOMEOS_STUN_SERVER";

    /// Disable public STUN servers (self-hosted only)
    pub const NO_PUBLIC_STUN: &str = "BIOMEOS_NO_PUBLIC_STUN";

    /// Comma-separated list of STUN servers
    pub const STUN_SERVERS: &str = "BIOMEOS_STUN_SERVERS";

    /// Fallback STUN address
    pub const STUN_FALLBACK_ADDRESS: &str = "BIOMEOS_STUN_FALLBACK_ADDRESS";

    // --- Compute & Services ---

    /// Compute endpoint override
    pub const COMPUTE_ENDPOINT: &str = "BIOMEOS_COMPUTE_ENDPOINT";

    /// Legacy toadstool endpoint (checked as fallback for compute)
    pub const TOADSTOOL_ENDPOINT: &str = "TOADSTOOL_ENDPOINT";

    // --- Registry ---

    /// Local registry cache directory override
    pub const REGISTRY_DIR: &str = "BIOMEOS_REGISTRY_DIR";

    /// GitHub API URL override (for air-gapped or enterprise)
    pub const GITHUB_API_URL: &str = "BIOMEOS_GITHUB_API_URL";

    /// GitHub token for authenticated API access
    pub const GITHUB_TOKEN: &str = "GITHUB_TOKEN";

    // --- Boot ---

    /// Kernel image path override
    pub const KERNEL: &str = "BIOMEOS_KERNEL";

    // --- UI & User ---

    /// User identity override
    pub const USER: &str = "BIOMEOS_USER";

    // --- CLI Paths ---

    /// plasmidBin directory override (CLI)
    pub const PLASMID_DIR: &str = "BIOMEOS_PLASMID_DIR";

    /// Chimera definitions directory override
    pub const CHIMERA_DEFINITIONS_DIR: &str = "BIOMEOS_CHIMERA_DEFINITIONS_DIR";

    /// Chimera binaries directory override
    pub const BIN_CHIMERAS_DIR: &str = "BIOMEOS_BIN_CHIMERAS_DIR";

    /// Primal binaries directory override (CLI)
    pub const BIN_PRIMALS_DIR: &str = "BIOMEOS_BIN_PRIMALS_DIR";

    /// Niche templates directory override
    pub const NICHE_TEMPLATES_DIR: &str = "BIOMEOS_NICHE_TEMPLATES_DIR";

    /// Spore paths (comma-separated mount points)
    pub const SPORE_PATHS: &str = "BIOMEOS_SPORE_PATHS";

    /// CLI log root directory override
    pub const CLI_LOG_ROOT: &str = "BIOMEOS_CLI_LOG_ROOT";

    // --- Primal Process Identity ---

    /// Primal binary path (set by spawner for child primals)
    pub const PRIMAL_BINARY: &str = "PRIMAL_BINARY";

    /// Primal JSON-RPC socket path (set by spawner)
    pub const PRIMAL_SOCKET_PATH: &str = "PRIMAL_SOCKET_PATH";

    /// Primal socket path override (discovery)
    pub const PRIMAL_SOCKET: &str = "PRIMAL_SOCKET";

    /// Primal ID (set by spawner for identity)
    pub const PRIMAL_ID: &str = "PRIMAL_ID";

    /// HTTP port override for primal services (non-prefixed)
    pub const PRIMAL_HTTP_PORT: &str = "HTTP_PORT";

    // --- Discovery (Non-BIOMEOS Prefix) ---

    /// Family seed for key derivation (non-prefixed legacy)
    pub const FAMILY_SEED_LEGACY: &str = "FAMILY_SEED";

    /// mDNS-discovered endpoint
    pub const MDNS_DISCOVERED_ENDPOINT: &str = "MDNS_DISCOVERED_ENDPOINT";

    /// Broadcast-discovered endpoint
    pub const BROADCAST_DISCOVERED_ENDPOINT: &str = "BROADCAST_DISCOVERED_ENDPOINT";

    /// Multicast-discovered endpoint
    pub const MULTICAST_DISCOVERED_ENDPOINT: &str = "MULTICAST_DISCOVERED_ENDPOINT";

    /// Security endpoint override
    pub const SECURITY_ENDPOINT: &str = "SECURITY_ENDPOINT";

    /// Primal name identity (spawner-injected)
    pub const PRIMAL_NAME: &str = "PRIMAL_NAME";

    /// AI default model override
    pub const AI_DEFAULT_MODEL: &str = "AI_DEFAULT_MODEL";

    /// AI HTTP provider list (comma-separated)
    pub const AI_HTTP_PROVIDERS: &str = "AI_HTTP_PROVIDERS";

    /// Spore root directory override
    pub const SPORE_ROOT: &str = "SPORE_ROOT";

    /// Plasmodium peer list (comma-separated)
    pub const PLASMODIUM_PEERS: &str = "PLASMODIUM_PEERS";

    /// Discovery socket path override (non-prefixed legacy)
    pub const DISCOVERY_SOCKET_LEGACY: &str = "DISCOVERY_SOCKET";

    /// MCP port override
    pub const MCP_PORT: &str = "MCP_PORT";

    /// JWT secret (non-prefixed legacy fallback)
    pub const JWT_SECRET_LEGACY: &str = "JWT_SECRET";

    // --- Federation (EcoPrimal) ---

    /// EcoPrimal installation prefix
    pub const ECOPRIMAL_PREFIX: &str = "ECOPRIMAL_PREFIX";

    /// EcoPrimal config directory
    pub const ECOPRIMAL_CONFIG_DIR: &str = "ECOPRIMAL_CONFIG_DIR";

    // ── Standard system environment variables ──────────────────────────
    // Named constants to avoid string literal scatter across crates.

    /// User home directory (POSIX)
    pub const HOME: &str = "HOME";
    /// XDG data home (`~/.local/share` default)
    pub const XDG_DATA_HOME: &str = "XDG_DATA_HOME";
    /// XDG config home (`~/.config` default)
    pub const XDG_CONFIG_HOME: &str = "XDG_CONFIG_HOME";
    /// XDG cache home (`~/.cache` default)
    pub const XDG_CACHE_HOME: &str = "XDG_CACHE_HOME";
    /// XDG state home (`~/.local/state` default)
    pub const XDG_STATE_HOME: &str = "XDG_STATE_HOME";
    /// Windows Subsystem for Linux distribution name
    pub const WSL_DISTRO_NAME: &str = "WSL_DISTRO_NAME";
    /// POSIX user ID
    pub const UID: &str = "UID";
    /// OS version string (informal)
    pub const OS_VERSION: &str = "OS_VERSION";
    /// System hostname
    pub const HOSTNAME: &str = "HOSTNAME";
    /// Rust log directive
    pub const RUST_LOG: &str = "RUST_LOG";
    /// System executable search path
    pub const SYS_PATH: &str = "PATH";
    /// Current username (POSIX)
    pub const SYS_USER: &str = "USER";
    /// Current username (Windows)
    pub const SYS_USERNAME: &str = "USERNAME";
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

/// Returns `true` when `PRIMAL_BIND_MODE` is set to `tcp_only`.
///
/// When true, all server bind paths should skip UDS and serve TCP only.
/// This is the ecosystem-wide env var convention for SELinux/Android
/// substrates where `sock_file create` is denied by policy.
#[must_use]
pub fn is_tcp_only_bind_mode() -> bool {
    std::env::var(vars::PRIMAL_BIND_MODE)
        .is_ok_and(|v| v.eq_ignore_ascii_case("tcp_only"))
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
