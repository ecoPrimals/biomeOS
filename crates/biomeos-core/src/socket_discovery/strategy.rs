// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Discovery Strategy Configuration
//!
//! Configures how socket discovery behaves across different platforms and use cases.

use biomeos_types::constants::ports;
use std::sync::Arc;

/// Discovery strategy configuration
#[derive(Debug, Clone)]
pub struct DiscoveryStrategy {
    /// Check environment hints first
    pub check_env_hints: bool,

    /// Use XDG runtime dir
    pub use_xdg_runtime: bool,

    /// Try abstract sockets (Linux/Android)
    pub try_abstract_sockets: bool,

    /// Use family-scoped /tmp
    pub use_family_tmp: bool,

    /// Query capability registry
    pub query_registry: bool,

    /// Enable TCP fallback (Tier 2)
    pub enable_tcp_fallback: bool,

    /// Default TCP port range start for auto-discovery
    pub tcp_port_start: u16,

    /// Default TCP host for fallback
    pub tcp_fallback_host: Arc<str>,

    /// Scan for sockets
    pub scan_sockets: bool,

    /// Cache discovered sockets
    pub enable_cache: bool,

    /// Cache TTL in seconds
    pub cache_ttl_secs: u64,
}

impl Default for DiscoveryStrategy {
    fn default() -> Self {
        Self {
            check_env_hints: true,
            use_xdg_runtime: true,
            try_abstract_sockets: cfg!(target_os = "linux"), // Only on Linux/Android
            use_family_tmp: true,
            query_registry: true,
            enable_tcp_fallback: true, // Universal IPC v3.0: always try TCP
            tcp_port_start: ports::TCP_PORT_SCAN_START, // Default port range for primals
            tcp_fallback_host: Arc::from("127.0.0.1"),
            scan_sockets: false, // Expensive, disabled by default
            enable_cache: true,
            cache_ttl_secs: 60,
        }
    }
}

impl DiscoveryStrategy {
    /// Create a strategy optimized for Android
    pub fn android() -> Self {
        Self {
            check_env_hints: true,
            use_xdg_runtime: false,     // Android doesn't use XDG
            try_abstract_sockets: true, // Prefer abstract sockets
            use_family_tmp: false,      // SELinux may block /tmp sockets
            query_registry: true,
            enable_tcp_fallback: true,
            tcp_port_start: ports::TCP_PORT_SCAN_START,
            tcp_fallback_host: Arc::from("127.0.0.1"),
            scan_sockets: false,
            enable_cache: true,
            cache_ttl_secs: 60,
        }
    }

    /// Create a strategy optimized for cross-device communication
    pub fn cross_device() -> Self {
        Self {
            check_env_hints: true,
            use_xdg_runtime: false,
            try_abstract_sockets: false, // Not cross-device
            use_family_tmp: false,       // Not cross-device
            query_registry: true,
            enable_tcp_fallback: true, // TCP is primary for cross-device
            tcp_port_start: ports::TCP_PORT_SCAN_START,
            tcp_fallback_host: Arc::from("0.0.0.0"), // Listen on all interfaces
            scan_sockets: false,
            enable_cache: true,
            cache_ttl_secs: 30, // Shorter TTL for dynamic environments
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let strategy = DiscoveryStrategy::default();

        assert!(strategy.check_env_hints);
        assert!(strategy.use_xdg_runtime);
        assert!(strategy.use_family_tmp);
        assert!(strategy.query_registry);
        assert!(strategy.enable_tcp_fallback); // Universal IPC v3.0
        assert!(!strategy.scan_sockets); // Expensive, disabled by default
        assert!(strategy.enable_cache);
    }

    #[test]
    fn test_android() {
        let strategy = DiscoveryStrategy::android();

        assert!(strategy.try_abstract_sockets);
        assert!(!strategy.use_xdg_runtime); // Android doesn't use XDG
        assert!(!strategy.use_family_tmp); // SELinux may block
        assert!(strategy.enable_tcp_fallback);
    }

    #[test]
    fn test_cross_device() {
        let strategy = DiscoveryStrategy::cross_device();

        assert!(!strategy.try_abstract_sockets); // Not cross-device
        assert!(!strategy.use_family_tmp); // Not cross-device
        assert!(strategy.enable_tcp_fallback); // Primary for cross-device
        assert_eq!(strategy.tcp_fallback_host.as_ref(), "0.0.0.0");
    }
}
