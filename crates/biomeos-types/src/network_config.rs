// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

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
//! - Primals discover peers via `capability.call()`
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
    pub(crate) bind_all: bool,

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
    /// Create `NetworkConfig` from environment variables
    ///
    /// Resolution order for each setting:
    /// 1. Environment variable (highest priority)
    /// 2. Default value (lowest priority)
    ///
    /// For production, override defaults via environment.
    /// For development, defaults work out of the box.
    #[must_use]
    pub fn from_env() -> Self {
        let env: HashMap<String, String> = env::vars().collect();
        Self::from_env_with(&env)
    }

    /// Create `NetworkConfig` from an explicit environment map (for testing)
    ///
    /// Use this in tests to avoid env-var races. Pass a `HashMap` with the
    /// variables you need; missing vars use defaults.
    #[must_use]
    pub fn from_env_with(env: &HashMap<String, String>) -> Self {
        let bind_all = env
            .get(env_vars::BIND_ALL)
            .is_some_and(|v| v == "1" || v.to_lowercase() == "true");

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
        // In sovereign mode, public STUN is disabled by default
        let is_sovereign = env
            .get("BIOMEOS_SOVEREIGN")
            .is_some_and(|v| v == "1" || v.to_lowercase() == "true");

        let allow_public_stun = if is_sovereign {
            // Sovereign mode: no external dependencies unless explicitly opted in
            env.get("BIOMEOS_ALLOW_PUBLIC_STUN")
                .is_some_and(|v| v == "1" || v.to_lowercase() == "true")
        } else {
            env.get(env_vars::NO_PUBLIC_STUN)
                .is_none_or(|v| v != "1" && v.to_lowercase() != "true")
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
    #[must_use]
    pub fn with_bind_address(bind_address: IpAddr) -> Self {
        let mut config = Self::from_env();
        config.bind_address = bind_address;
        config.bind_all = bind_address.is_unspecified();
        config
    }

    /// Create for local-only binding (127.0.0.1)
    #[must_use]
    pub fn localhost() -> Self {
        Self::with_bind_address(IpAddr::V4(Ipv4Addr::LOCALHOST))
    }

    /// Create for all-interfaces binding (`[::]` dual-stack IPv6+IPv4)
    #[must_use]
    pub fn all_interfaces() -> Self {
        Self::with_bind_address(IpAddr::V6(Ipv6Addr::UNSPECIFIED))
    }

    // =========================================================================
    // Bind Address Methods
    // =========================================================================

    /// Get the bind address for services
    #[must_use]
    pub const fn bind_address(&self) -> IpAddr {
        self.bind_address
    }

    /// Get bind address as string
    #[must_use]
    pub fn bind_address_string(&self) -> String {
        self.bind_address.to_string()
    }

    /// Get socket address for a port
    #[must_use]
    pub const fn socket_addr(&self, port: u16) -> SocketAddr {
        SocketAddr::new(self.bind_address, port)
    }

    /// Get HTTP socket address
    #[must_use]
    pub const fn http_socket(&self) -> SocketAddr {
        self.socket_addr(self.ports.http)
    }

    /// Get HTTPS socket address
    #[must_use]
    pub const fn https_socket(&self) -> SocketAddr {
        self.socket_addr(self.ports.https)
    }

    /// Get WebSocket socket address
    #[must_use]
    pub const fn websocket_socket(&self) -> SocketAddr {
        self.socket_addr(self.ports.websocket)
    }

    /// Get discovery socket address
    #[must_use]
    pub const fn discovery_socket(&self) -> SocketAddr {
        self.socket_addr(self.ports.discovery)
    }

    /// Get relay socket address
    #[must_use]
    pub const fn relay_socket(&self) -> SocketAddr {
        self.socket_addr(self.ports.relay)
    }

    // =========================================================================
    // Port Methods
    // =========================================================================

    /// Get port configuration
    #[must_use]
    pub const fn ports(&self) -> &PortConfig {
        &self.ports
    }

    /// Get HTTP port
    #[must_use]
    pub const fn http_port(&self) -> u16 {
        self.ports.http
    }

    /// Get HTTPS port
    #[must_use]
    pub const fn https_port(&self) -> u16 {
        self.ports.https
    }

    /// Get WebSocket port
    #[must_use]
    pub const fn websocket_port(&self) -> u16 {
        self.ports.websocket
    }

    /// Get discovery port
    #[must_use]
    pub const fn discovery_port(&self) -> u16 {
        self.ports.discovery
    }

    /// Get relay port
    #[must_use]
    pub const fn relay_port(&self) -> u16 {
        self.ports.relay
    }

    /// Get STUN port
    #[must_use]
    pub const fn stun_port(&self) -> u16 {
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
    #[must_use]
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
    #[must_use]
    pub fn self_hosted_stun(&self) -> Option<&str> {
        self.self_hosted_stun.as_deref()
    }

    /// Check if public STUN fallback is allowed
    #[must_use]
    pub const fn allows_public_stun(&self) -> bool {
        self.allow_public_stun
    }

    // =========================================================================
    // Internal Helpers
    // =========================================================================

    fn parse_port_with(env: &HashMap<String, String>, var: &str, default: u16) -> u16 {
        env.get(var).and_then(|s| s.parse().ok()).unwrap_or(default)
    }

    fn resolve_stun_servers_with(env: &HashMap<String, String>) -> Vec<String> {
        env.get(env_vars::STUN_SERVERS)
            .map_or_else(Vec::new, |servers| {
                servers
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect()
            })
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
/// Use this when you just need the bind address without full `NetworkConfig`.
#[must_use]
pub fn bind_address() -> IpAddr {
    NetworkConfig::from_env().bind_address()
}

/// Get bind address as string (convenience function)
#[must_use]
pub fn bind_address_string() -> String {
    NetworkConfig::from_env().bind_address_string()
}

/// Get socket address for a port (convenience function)
#[must_use]
pub fn socket_addr(port: u16) -> SocketAddr {
    NetworkConfig::from_env().socket_addr(port)
}

/// Get STUN servers in priority order (convenience function)
#[must_use]
pub fn stun_servers() -> Vec<String> {
    NetworkConfig::from_env().stun_servers()
}
