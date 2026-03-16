// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Socket Discovery - Capability-Based Runtime Discovery
//!
//! **Deep Debt Solution**: Replaces hardcoded `/tmp/{primal}.sock` paths with
//! capability-based discovery that respects system conventions and primal self-knowledge.
//!
//! ## Universal IPC Standard v3.0 Compliance
//!
//! This module implements the Universal IPC Standard v3.0 for biomeOS:
//! - Multi-transport support (Unix, Abstract, TCP)
//! - Tier 1 → Tier 2 graceful fallback
//! - Platform-agnostic transport selection
//! - Runtime discovery (no hardcoded primal knowledge)
//!
//! ## Principles
//!
//! 1. **No Hardcoding**: Socket paths discovered at runtime
//! 2. **XDG Compliance**: Respects XDG_RUNTIME_DIR when available
//! 3. **Family-Based Isolation**: Sockets namespaced by family_id
//! 4. **Capability Discovery**: Find primals by what they do, not where they are
//! 5. **Platform Agnostic**: Works across Linux, macOS, Android, and other systems
//! 6. **Graceful Fallback**: Tier 1 (Unix/Abstract) → Tier 2 (TCP) automatically
//!
//! ## Transport Tiers
//!
//! - **Tier 1 (Native)**: Unix sockets, Abstract sockets (Linux/Android)
//! - **Tier 2 (Universal)**: TCP sockets (cross-device, WASM, restricted environments)
//!
//! ## Discovery Order
//!
//! 1. Environment variable hint (e.g., `BEARDOG_SOCKET`, `BEARDOG_TCP`)
//! 2. XDG_RUNTIME_DIR (e.g., `/run/user/1000/biomeos/beardog-1894e909e454.sock`)
//! 3. Abstract socket (Android: `@biomeos_beardog_1894e909e454`)
//! 4. Family-scoped /tmp (e.g., `/tmp/beardog-1894e909e454.sock`)
//! 5. Capability registry query via Neural API
//! 6. TCP fallback (e.g., `127.0.0.1:9100`)
//!
//! ## Usage
//!
//! ```ignore
//! use biomeos_core::socket_discovery::{SocketDiscovery, TransportEndpoint};
//! use biomeos_core::family_discovery::get_family_id;
//!
//! // Use dynamic family discovery instead of hardcoded family ID
//! let family_id = get_family_id(); // Discovers from .family.seed or env
//! let discovery = SocketDiscovery::new(&family_id);
//!
//! // Discover primal by capability, not name
//! let endpoint = discovery.discover_with_fallback("security").await?;
//! match endpoint {
//!     TransportEndpoint::UnixSocket { path } => { /* connect via Unix */ }
//!     TransportEndpoint::AbstractSocket { name } => { /* connect via abstract */ }
//!     TransportEndpoint::TcpSocket { host, port } => { /* connect via TCP */ }
//! }
//! ```

mod engine;
mod result;
mod strategy;
mod transport;

#[cfg(test)]
mod engine_tests;

// Re-export primary types
pub use engine::SocketDiscovery;
pub use result::{DiscoveredSocket, DiscoveryMethod};
pub use strategy::DiscoveryStrategy;
pub use transport::TransportEndpoint;

use std::env;
use std::path::PathBuf;

// ============================================================================
// CONVENIENCE FUNCTIONS (for quick migrations from hardcoded paths)
// ============================================================================

/// Discover socket for a primal (convenience function)
///
/// Uses default family_id from FAMILY_ID or BIOMEOS_FAMILY_ID environment.
pub async fn discover_socket(primal_name: &str) -> Option<PathBuf> {
    let family_id = env::var("FAMILY_ID")
        .or_else(|_| env::var("BIOMEOS_FAMILY_ID"))
        .unwrap_or_else(|_| "default".to_string());

    let discovery = SocketDiscovery::new(family_id);
    discovery.get_socket_path(primal_name).await
}

/// Build socket path for a primal (convenience function)
///
/// Deterministic path building for primals to register their own sockets.
pub fn build_socket(primal_name: &str, family_id: &str) -> PathBuf {
    let discovery = SocketDiscovery::new(family_id);
    discovery.build_socket_path(primal_name)
}

/// Discover transport endpoint with automatic fallback (convenience function)
///
/// **Universal IPC Standard v3.0**: Use this for cross-platform discovery.
pub async fn discover_endpoint(primal_name: &str) -> Option<TransportEndpoint> {
    let family_id = env::var("FAMILY_ID")
        .or_else(|_| env::var("BIOMEOS_FAMILY_ID"))
        .unwrap_or_else(|_| "default".to_string());

    let discovery = SocketDiscovery::new(family_id);
    discovery.discover_with_fallback(primal_name).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_build_socket_convenience() {
        let path = build_socket("beardog", "test-family");
        assert!(path.to_string_lossy().contains("beardog"));
        assert!(path.to_string_lossy().contains("test-family"));
    }

    #[test]
    fn test_build_socket_deterministic() {
        let path1 = build_socket("songbird", "family-1");
        let path2 = build_socket("songbird", "family-1");
        assert_eq!(path1, path2);
    }

    #[tokio::test]
    async fn test_discover_socket_uses_family_id_env() {
        env::set_var("FAMILY_ID", "env-family");
        let _result = discover_socket("beardog").await;
        env::remove_var("FAMILY_ID");
        // Just verify it doesn't panic - result may be None if socket doesn't exist
    }

    #[tokio::test]
    #[ignore = "env-var test is thread-unsafe; run with --test-threads=1"]
    async fn test_discover_socket_family_id_from_env() {
        env::set_var("BIOMEOS_FAMILY_ID", "env-family-id");
        let family_id = env::var("FAMILY_ID")
            .or_else(|_| env::var("BIOMEOS_FAMILY_ID"))
            .unwrap_or_else(|_| "default".to_string());
        let discovery = SocketDiscovery::new(&family_id);
        assert_eq!(discovery.family_id.as_str(), "env-family-id");
        env::remove_var("BIOMEOS_FAMILY_ID");
    }

    #[tokio::test]
    async fn test_discover_endpoint_convenience() {
        env::set_var("BIOMEOS_FAMILY_ID", "test-default");
        let _result = discover_endpoint("nonexistent-primal").await;
        env::remove_var("BIOMEOS_FAMILY_ID");
        // Just verify it doesn't panic
    }

    #[tokio::test]
    async fn test_discover_socket_default_family() {
        env::remove_var("FAMILY_ID");
        env::remove_var("BIOMEOS_FAMILY_ID");
        let _result = discover_socket("beardog").await;
        // Should use "default" when no env vars set
    }
}
