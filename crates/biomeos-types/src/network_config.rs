// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Network Configuration - Capability-Based Network Resolution
//!
//! This module provides centralized, capability-based network configuration
//! for biomeOS. All network settings (IPs, ports, STUN servers) are:
//!
//! 1. **Environment-driven** - Override via env vars
//! 2. **Capability-discoverable** - Query at runtime
//! 3. **Fallback-safe** - Sensible defaults when nothing else works
//!
//! ## Deep Debt Principle
//!
//! "Hardcoding should be evolved to agnostic and capability-based"
//!
//! BEFORE:
//! ```ignore
//! let addr = "127.0.0.1:8080";  // ❌ Hardcoded
//! ```
//!
//! AFTER:
//! ```ignore
//! let config = NetworkConfig::from_env();
//! let addr = config.bind_address();  // ✅ Environment-aware
//! ```
//!
//! ## TRUE PRIMAL Principle
//!
//! "Primals have SELF-KNOWLEDGE ONLY. They discover other primals at RUNTIME."
//!
//! Network configuration follows this principle:
//! - Primals don't hardcode other primal addresses
//! - Primals discover peers via capability.call()
//! - Defaults only used when discovery unavailable

use crate::constants::ports;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};

/// Environment variable names for network configuration
pub mod env_vars {
    /// Bind address for services
    pub const BIND_ADDRESS: &str = "BIND_ADDRESS";

    /// Bind to all interfaces flag
    pub const BIND_ALL: &str = "BIOMEOS_BIND_ALL";

    /// Custom STUN servers (comma-separated)
    pub const STUN_SERVERS: &str = "BIOMEOS_STUN_SERVERS";

    /// Self-hosted STUN server
    pub const SELF_HOSTED_STUN: &str = "BIOMEOS_STUN_SERVER";

    /// Disable public STUN fallback
    pub const NO_PUBLIC_STUN: &str = "BIOMEOS_NO_PUBLIC_STUN";

    /// HTTP port
    pub const HTTP_PORT: &str = "HTTP_PORT";

    /// HTTPS port
    pub const HTTPS_PORT: &str = "HTTPS_PORT";

    /// WebSocket port
    pub const WEBSOCKET_PORT: &str = "WEBSOCKET_PORT";

    /// Discovery port
    pub const DISCOVERY_PORT: &str = "DISCOVERY_PORT";

    /// Relay port
    pub const RELAY_PORT: &str = "RELAY_PORT";

    /// STUN port (for self-hosted)
    pub const STUN_PORT: &str = "STUN_PORT";
}

/// Network configuration with environment-aware defaults
///
/// This struct centralizes all network-related configuration and
/// provides consistent resolution across the codebase.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Bind address for services
    bind_address: IpAddr,

    /// Whether to bind to all interfaces ([::] dual-stack)
    bind_all: bool,

    /// STUN servers for NAT traversal
    stun_servers: Vec<String>,

    /// Self-hosted STUN server (if available)
    self_hosted_stun: Option<String>,

    /// Allow fallback to public STUN
    allow_public_stun: bool,

    /// Default ports
    ports: PortConfig,
}

/// Port configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortConfig {
    /// HTTP port
    pub http: u16,
    /// HTTPS port
    pub https: u16,
    /// WebSocket port
    pub websocket: u16,
    /// Discovery port
    pub discovery: u16,
    /// Relay port
    pub relay: u16,
    /// STUN port
    pub stun: u16,
}

impl Default for PortConfig {
    fn default() -> Self {
        Self {
            http: ports::HTTP_BRIDGE,
            https: ports::HTTPS_DEFAULT,
            websocket: ports::WS_DEFAULT,
            discovery: ports::WEBSOCKET,
            relay: ports::RELAY,
            stun: ports::STUN,
        }
    }
}

impl NetworkConfig {
    /// Create NetworkConfig from environment variables
    ///
    /// Resolution order for each setting:
    /// 1. Environment variable (highest priority)
    /// 2. Default value (lowest priority)
    ///
    /// For production, override defaults via environment.
    /// For development, defaults work out of the box.
    pub fn from_env() -> Self {
        let env: HashMap<String, String> = env::vars().collect();
        Self::from_env_with(&env)
    }

    /// Create NetworkConfig from an explicit environment map (for testing)
    ///
    /// Use this in tests to avoid env-var races. Pass a `HashMap` with the
    /// variables you need; missing vars use defaults.
    pub fn from_env_with(env: &HashMap<String, String>) -> Self {
        let bind_all = env
            .get(env_vars::BIND_ALL)
            .map(|v| v == "1" || v.to_lowercase() == "true")
            .unwrap_or(false);

        let bind_address = if bind_all {
            // Use IPv6 unspecified [::] for dual-stack (accepts both IPv4 and IPv6)
            // This is critical for sovereign beacon: Pixel connects via IPv6 direct
            IpAddr::V6(Ipv6Addr::UNSPECIFIED) // [::]
        } else {
            env.get(env_vars::BIND_ADDRESS)
                .and_then(|s| s.parse().ok())
                .unwrap_or(IpAddr::V4(Ipv4Addr::LOCALHOST)) // 127.0.0.1
        };

        let stun_servers = Self::resolve_stun_servers_with(env);
        let self_hosted_stun = env.get(env_vars::SELF_HOSTED_STUN).cloned();
        // DEEP DEBT: In sovereign mode, public STUN is disabled by default
        let is_sovereign = env
            .get("BIOMEOS_SOVEREIGN")
            .map(|v| v == "1" || v.to_lowercase() == "true")
            .unwrap_or(false);

        let allow_public_stun = if is_sovereign {
            // Sovereign mode: no external dependencies unless explicitly opted in
            env.get("BIOMEOS_ALLOW_PUBLIC_STUN")
                .map(|v| v == "1" || v.to_lowercase() == "true")
                .unwrap_or(false)
        } else {
            env.get(env_vars::NO_PUBLIC_STUN)
                .map(|v| v != "1" && v.to_lowercase() != "true")
                .unwrap_or(true)
        };

        let ports = PortConfig {
            http: Self::parse_port_with(env, env_vars::HTTP_PORT, ports::HTTP_BRIDGE),
            https: Self::parse_port_with(env, env_vars::HTTPS_PORT, ports::HTTPS_DEFAULT),
            websocket: Self::parse_port_with(env, env_vars::WEBSOCKET_PORT, ports::WS_DEFAULT),
            discovery: Self::parse_port_with(env, env_vars::DISCOVERY_PORT, ports::WEBSOCKET),
            relay: Self::parse_port_with(env, env_vars::RELAY_PORT, ports::RELAY),
            stun: Self::parse_port_with(env, env_vars::STUN_PORT, ports::STUN),
        };

        Self {
            bind_address,
            bind_all,
            stun_servers,
            self_hosted_stun,
            allow_public_stun,
            ports,
        }
    }

    /// Create with explicit bind address
    pub fn with_bind_address(bind_address: IpAddr) -> Self {
        let mut config = Self::from_env();
        config.bind_address = bind_address;
        config.bind_all = bind_address.is_unspecified();
        config
    }

    /// Create for local-only binding (127.0.0.1)
    pub fn localhost() -> Self {
        Self::with_bind_address(IpAddr::V4(Ipv4Addr::LOCALHOST))
    }

    /// Create for all-interfaces binding (`[::]` dual-stack IPv6+IPv4)
    pub fn all_interfaces() -> Self {
        Self::with_bind_address(IpAddr::V6(Ipv6Addr::UNSPECIFIED))
    }

    // =========================================================================
    // Bind Address Methods
    // =========================================================================

    /// Get the bind address for services
    pub fn bind_address(&self) -> IpAddr {
        self.bind_address
    }

    /// Get bind address as string
    pub fn bind_address_string(&self) -> String {
        self.bind_address.to_string()
    }

    /// Get socket address for a port
    pub fn socket_addr(&self, port: u16) -> SocketAddr {
        SocketAddr::new(self.bind_address, port)
    }

    /// Get HTTP socket address
    pub fn http_socket(&self) -> SocketAddr {
        self.socket_addr(self.ports.http)
    }

    /// Get HTTPS socket address
    pub fn https_socket(&self) -> SocketAddr {
        self.socket_addr(self.ports.https)
    }

    /// Get WebSocket socket address
    pub fn websocket_socket(&self) -> SocketAddr {
        self.socket_addr(self.ports.websocket)
    }

    /// Get discovery socket address
    pub fn discovery_socket(&self) -> SocketAddr {
        self.socket_addr(self.ports.discovery)
    }

    /// Get relay socket address
    pub fn relay_socket(&self) -> SocketAddr {
        self.socket_addr(self.ports.relay)
    }

    // =========================================================================
    // Port Methods
    // =========================================================================

    /// Get port configuration
    pub fn ports(&self) -> &PortConfig {
        &self.ports
    }

    /// Get HTTP port
    pub fn http_port(&self) -> u16 {
        self.ports.http
    }

    /// Get HTTPS port
    pub fn https_port(&self) -> u16 {
        self.ports.https
    }

    /// Get WebSocket port
    pub fn websocket_port(&self) -> u16 {
        self.ports.websocket
    }

    /// Get discovery port
    pub fn discovery_port(&self) -> u16 {
        self.ports.discovery
    }

    /// Get relay port
    pub fn relay_port(&self) -> u16 {
        self.ports.relay
    }

    /// Get STUN port
    pub fn stun_port(&self) -> u16 {
        self.ports.stun
    }

    // =========================================================================
    // STUN Configuration
    // =========================================================================

    /// Get STUN servers (self-hosted first if available, then public)
    ///
    /// Returns servers in priority order:
    /// 1. Self-hosted STUN (if configured)
    /// 2. Custom STUN servers (from env)
    /// 3. Public STUN servers (if allowed)
    pub fn stun_servers(&self) -> Vec<String> {
        let mut servers = Vec::new();

        // Self-hosted first (highest priority, maximum sovereignty)
        if let Some(ref self_hosted) = self.self_hosted_stun {
            servers.push(self_hosted.clone());
        }

        // Custom configured servers
        servers.extend(self.stun_servers.clone());

        // Public fallback (if allowed)
        if self.allow_public_stun && servers.is_empty() {
            servers.extend(Self::default_public_stun_servers());
        }

        servers
    }

    /// Get self-hosted STUN server (if configured)
    pub fn self_hosted_stun(&self) -> Option<&str> {
        self.self_hosted_stun.as_deref()
    }

    /// Check if public STUN fallback is allowed
    pub fn allows_public_stun(&self) -> bool {
        self.allow_public_stun
    }

    // =========================================================================
    // Internal Helpers
    // =========================================================================

    fn parse_port_with(env: &HashMap<String, String>, var: &str, default: u16) -> u16 {
        env.get(var).and_then(|s| s.parse().ok()).unwrap_or(default)
    }

    fn resolve_stun_servers_with(env: &HashMap<String, String>) -> Vec<String> {
        if let Some(servers) = env.get(env_vars::STUN_SERVERS) {
            servers
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Default public STUN servers (community-run, FOSS-aligned)
    ///
    /// Sovereign-first: community-run servers only. Corporate STUN endpoints
    /// (Google, Cloudflare) are excluded — core NAT traversal must not depend
    /// on corporate infrastructure. Override via `BIOMEOS_STUN_SERVERS` env var.
    fn default_public_stun_servers() -> Vec<String> {
        vec![
            format!("stun.nextcloud.com:{}", ports::STUN),
            format!("stun.sip.us:{}", ports::STUN),
            format!("stun.stunprotocol.org:{}", ports::STUN),
        ]
    }
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self::from_env()
    }
}

// =============================================================================
// Convenience Functions
// =============================================================================

/// Get bind address from environment (convenience function)
///
/// Use this when you just need the bind address without full NetworkConfig.
pub fn bind_address() -> IpAddr {
    NetworkConfig::from_env().bind_address()
}

/// Get bind address as string (convenience function)
pub fn bind_address_string() -> String {
    NetworkConfig::from_env().bind_address_string()
}

/// Get socket address for a port (convenience function)
pub fn socket_addr(port: u16) -> SocketAddr {
    NetworkConfig::from_env().socket_addr(port)
}

/// Get STUN servers in priority order (convenience function)
pub fn stun_servers() -> Vec<String> {
    NetworkConfig::from_env().stun_servers()
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
mod tests {
    use super::*;

    fn env_map(entries: &[(&str, &str)]) -> HashMap<String, String> {
        entries
            .iter()
            .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
            .collect()
    }

    #[test]
    fn test_default_config() {
        let env = HashMap::new();
        let config = NetworkConfig::from_env_with(&env);
        assert_eq!(config.bind_address(), IpAddr::V4(Ipv4Addr::LOCALHOST));
        assert!(!config.bind_all);
    }

    #[test]
    fn test_bind_all() {
        let env = env_map(&[(env_vars::BIND_ALL, "true")]);
        let config = NetworkConfig::from_env_with(&env);
        // DEEP DEBT: bind_all uses IPv6 [::] for dual-stack (accepts IPv4 + IPv6)
        assert_eq!(config.bind_address(), IpAddr::V6(Ipv6Addr::UNSPECIFIED));
        assert!(config.bind_all);
    }

    #[test]
    fn test_custom_bind_address() {
        let env = env_map(&[(env_vars::BIND_ADDRESS, "192.168.1.100")]);
        let config = NetworkConfig::from_env_with(&env);
        assert!(config.bind_address().is_ipv4() || config.bind_address().is_ipv6());
    }

    #[test]
    fn test_socket_addr() {
        let config = NetworkConfig::localhost();
        let addr = config.socket_addr(8080);

        assert_eq!(addr.port(), 8080);
        assert_eq!(addr.ip(), IpAddr::V4(Ipv4Addr::LOCALHOST));
    }

    #[test]
    fn test_port_defaults() {
        let config = NetworkConfig::from_env();
        let ports = config.ports();

        assert!(ports.http > 0);
        assert!(ports.https > 0);
        assert!(ports.websocket > 0);
        assert!(ports.discovery > 0);
    }

    #[test]
    fn test_custom_port() {
        let env = env_map(&[(env_vars::HTTP_PORT, "9999")]);
        let config = NetworkConfig::from_env_with(&env);
        assert_eq!(config.http_port(), 9999);
    }

    #[test]
    fn test_stun_servers_default() {
        let env = HashMap::new();
        let config = NetworkConfig::from_env_with(&env);
        let servers = config.stun_servers();

        // Should have public fallback servers
        assert!(!servers.is_empty());
        assert!(servers[0].contains(':'));
    }

    #[test]
    fn test_custom_stun_servers() {
        let env = env_map(&[(
            env_vars::STUN_SERVERS,
            "stun.example.com:3478,stun2.example.com:3478",
        )]);
        let config = NetworkConfig::from_env_with(&env);
        let servers = config.stun_servers();

        assert!(servers.contains(&"stun.example.com:3478".to_string()));
        assert!(servers.contains(&"stun2.example.com:3478".to_string()));
    }

    #[test]
    fn test_self_hosted_stun_priority() {
        let env = env_map(&[(env_vars::SELF_HOSTED_STUN, "my-stun.local:3478")]);
        let config = NetworkConfig::from_env_with(&env);
        let servers = config.stun_servers();

        // Self-hosted should be first
        assert_eq!(servers[0], "my-stun.local:3478");
    }

    #[test]
    fn test_no_public_stun() {
        let env = env_map(&[(env_vars::NO_PUBLIC_STUN, "true")]);
        let config = NetworkConfig::from_env_with(&env);
        let servers = config.stun_servers();

        // No servers when public disabled and no custom configured
        assert!(servers.is_empty());
    }

    #[test]
    fn test_convenience_functions() {
        let addr = bind_address();
        assert!(addr.is_ipv4() || addr.is_ipv6());

        let addr_str = bind_address_string();
        assert!(!addr_str.is_empty());

        let socket = socket_addr(8080);
        assert_eq!(socket.port(), 8080);
    }

    #[test]
    fn test_localhost_factory() {
        let config = NetworkConfig::localhost();
        assert_eq!(config.bind_address(), IpAddr::V4(Ipv4Addr::LOCALHOST));
    }

    #[test]
    fn test_all_interfaces_factory() {
        let config = NetworkConfig::all_interfaces();
        // Uses IPv6 [::] for dual-stack binding
        assert_eq!(config.bind_address(), IpAddr::V6(Ipv6Addr::UNSPECIFIED));
    }

    #[test]
    fn test_with_bind_address() {
        let config = NetworkConfig::with_bind_address(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)));
        assert_eq!(
            config.bind_address(),
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1))
        );
    }

    #[test]
    fn test_bind_address_string() {
        let config = NetworkConfig::localhost();
        let s = config.bind_address_string();
        assert_eq!(s, "127.0.0.1");
    }

    #[test]
    fn test_socket_methods() {
        let config = NetworkConfig::localhost();
        assert_eq!(config.http_socket().port(), config.http_port());
        assert_eq!(config.https_socket().port(), config.https_port());
        assert_eq!(config.websocket_socket().port(), config.websocket_port());
        assert_eq!(config.discovery_socket().port(), config.discovery_port());
        assert_eq!(config.relay_socket().port(), config.relay_port());
    }

    #[test]
    fn test_port_config_default() {
        let ports = PortConfig::default();
        assert_eq!(ports.http, 8080);
        assert_eq!(ports.https, 8443);
        assert_eq!(ports.websocket, 8081);
        assert_eq!(ports.discovery, 8001);
        assert_eq!(ports.relay, 3490);
        assert_eq!(ports.stun, 3478);
    }

    #[test]
    fn test_ports_accessor() {
        let config = NetworkConfig::localhost();
        let ports = config.ports();
        assert!(ports.http > 0);
        assert!(ports.stun > 0);
    }

    #[test]
    fn test_stun_port_accessor() {
        let config = NetworkConfig::localhost();
        assert_eq!(config.stun_port(), 3478);
    }

    #[test]
    fn test_allows_public_stun_accessor() {
        let config = NetworkConfig::localhost();
        let _ = config.allows_public_stun();
    }

    #[test]
    fn test_self_hosted_stun_none() {
        let config = NetworkConfig::localhost();
        assert!(config.self_hosted_stun().is_none());
    }

    #[test]
    fn test_stun_servers_convenience() {
        let servers = stun_servers();
        assert!(servers.is_empty() || servers.iter().all(|s| s.contains(':')));
    }

    // ── Additional env-var tests (using from_env_with) ──────────────────────

    #[test]
    fn test_from_env_bind_all_one() {
        let env = env_map(&[(env_vars::BIND_ALL, "1")]);
        let config = NetworkConfig::from_env_with(&env);
        assert_eq!(config.bind_address(), IpAddr::V6(Ipv6Addr::UNSPECIFIED));
    }

    #[test]
    fn test_from_env_bind_address_ipv6() {
        let env = env_map(&[(env_vars::BIND_ADDRESS, "::1")]);
        let config = NetworkConfig::from_env_with(&env);
        assert!(config.bind_address().is_ipv6());
    }

    #[test]
    fn test_from_env_invalid_bind_address_fallback() {
        let env = env_map(&[(env_vars::BIND_ADDRESS, "not-an-ip")]);
        let config = NetworkConfig::from_env_with(&env);
        // Should fall back to localhost when parse fails
        assert_eq!(config.bind_address(), IpAddr::V4(Ipv4Addr::LOCALHOST));
    }

    #[test]
    fn test_stun_servers_self_hosted_first() {
        let env = env_map(&[
            (env_vars::SELF_HOSTED_STUN, "stun.self.local:3478"),
            (env_vars::STUN_SERVERS, "stun.custom.com:3478"),
        ]);
        let config = NetworkConfig::from_env_with(&env);
        let servers = config.stun_servers();

        assert!(!servers.is_empty());
        assert_eq!(servers[0], "stun.self.local:3478");
    }

    #[test]
    fn test_stun_servers_custom_only_no_public() {
        let env = env_map(&[
            (env_vars::STUN_SERVERS, "stun.a.com:3478,stun.b.com:3478"),
            (env_vars::NO_PUBLIC_STUN, "1"),
        ]);
        let config = NetworkConfig::from_env_with(&env);
        let servers = config.stun_servers();

        assert_eq!(servers.len(), 2);
        assert!(servers.contains(&"stun.a.com:3478".to_string()));
        assert!(servers.contains(&"stun.b.com:3478".to_string()));
    }

    #[test]
    fn test_stun_servers_sovereign_mode_no_public() {
        let env = env_map(&[("BIOMEOS_SOVEREIGN", "true")]);
        let config = NetworkConfig::from_env_with(&env);
        let servers = config.stun_servers();

        assert!(servers.is_empty());
        assert!(!config.allows_public_stun());
    }

    #[test]
    fn test_stun_servers_sovereign_with_opt_in() {
        let env = env_map(&[
            ("BIOMEOS_SOVEREIGN", "1"),
            ("BIOMEOS_ALLOW_PUBLIC_STUN", "true"),
        ]);
        let config = NetworkConfig::from_env_with(&env);
        let servers = config.stun_servers();

        assert!(!servers.is_empty());
        assert!(config.allows_public_stun());
    }

    #[test]
    fn test_bind_address_string_ipv6() {
        let config = NetworkConfig::all_interfaces();
        let s = config.bind_address_string();
        assert!(s.contains("::") || s == "::");
    }

    #[test]
    fn test_port_config_defaults_full() {
        let ports = PortConfig::default();
        assert_eq!(ports.http, 8080);
        assert_eq!(ports.https, 8443);
        assert_eq!(ports.websocket, 8081);
        assert_eq!(ports.discovery, 8001);
        assert_eq!(ports.relay, 3490);
        assert_eq!(ports.stun, 3478);
    }

    #[test]
    fn test_with_bind_address_sets_bind_all_for_unspecified() {
        let config = NetworkConfig::with_bind_address(IpAddr::V6(Ipv6Addr::UNSPECIFIED));
        assert!(config.bind_all);
    }

    #[test]
    fn test_network_config_serialization() {
        let config = NetworkConfig::localhost();
        let json = serde_json::to_string(&config).expect("serialize");
        let _parsed: NetworkConfig = serde_json::from_str(&json).expect("deserialize");
    }

    #[test]
    fn test_port_config_serialization() {
        let ports = PortConfig::default();
        let json = serde_json::to_string(&ports).expect("serialize");
        let parsed: PortConfig = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed.http, ports.http);
        assert_eq!(parsed.stun, ports.stun);
    }

    #[test]
    fn test_stun_servers_whitespace_trimmed() {
        let env = env_map(&[
            (
                env_vars::STUN_SERVERS,
                " stun.a.com:3478 , stun.b.com:3478 ,  ",
            ),
            (env_vars::NO_PUBLIC_STUN, "1"),
        ]);
        let config = NetworkConfig::from_env_with(&env);
        let servers = config.stun_servers();

        assert!(servers.contains(&"stun.a.com:3478".to_string()));
        assert!(servers.contains(&"stun.b.com:3478".to_string()));
    }

    #[test]
    fn test_stun_servers_empty_entries_filtered() {
        let env = env_map(&[
            (env_vars::STUN_SERVERS, "stun.a.com:3478,,,stun.b.com:3478"),
            (env_vars::NO_PUBLIC_STUN, "1"),
        ]);
        let config = NetworkConfig::from_env_with(&env);
        let servers = config.stun_servers();

        assert_eq!(servers.len(), 2);
        assert!(servers.contains(&"stun.a.com:3478".to_string()));
        assert!(servers.contains(&"stun.b.com:3478".to_string()));
    }

    #[test]
    fn test_network_config_default_impl() {
        let config = NetworkConfig::default();
        assert!(config.bind_address().is_ipv4() || config.bind_address().is_ipv6());
    }

    #[test]
    fn test_network_config_clone() {
        let config = NetworkConfig::localhost();
        let cloned = config.clone();
        assert_eq!(config.bind_address(), cloned.bind_address());
        assert_eq!(config.http_port(), cloned.http_port());
    }

    #[test]
    fn test_network_config_serialization_roundtrip() {
        let config = NetworkConfig::localhost();
        let json = serde_json::to_string(&config).expect("serialize");
        let parsed: NetworkConfig = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(config.bind_address(), parsed.bind_address());
        assert_eq!(config.ports().http, parsed.ports().http);
    }

    #[test]
    fn test_no_public_stun_false_allows_public() {
        let env = env_map(&[(env_vars::NO_PUBLIC_STUN, "false")]);
        let config = NetworkConfig::from_env_with(&env);
        let servers = config.stun_servers();

        assert!(!servers.is_empty());
        assert!(config.allows_public_stun());
    }

    #[test]
    fn test_env_vars_constants() {
        assert_eq!(env_vars::BIND_ADDRESS, "BIND_ADDRESS");
        assert_eq!(env_vars::BIND_ALL, "BIOMEOS_BIND_ALL");
        assert_eq!(env_vars::STUN_SERVERS, "BIOMEOS_STUN_SERVERS");
        assert_eq!(env_vars::SELF_HOSTED_STUN, "BIOMEOS_STUN_SERVER");
        assert_eq!(env_vars::NO_PUBLIC_STUN, "BIOMEOS_NO_PUBLIC_STUN");
    }
}
