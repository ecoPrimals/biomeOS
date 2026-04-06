// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

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
//! 2. **XDG Compliance**: Respects `XDG_RUNTIME_DIR` when available
//! 3. **Family-Based Isolation**: Sockets namespaced by `family_id`
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
//! 2. Capability-first sockets (e.g., `security.sock`, `crypto.sock`)
//! 3. `XDG_RUNTIME_DIR` (e.g., `/run/user/1000/biomeos/beardog-1894e909e454.sock`)
//! 4. Abstract socket (Android: `@biomeos_beardog_1894e909e454`)
//! 5. Family-scoped /tmp (e.g., `/tmp/beardog-1894e909e454.sock`)
//! 6. Socket registry (`$XDG_RUNTIME_DIR/biomeos/socket-registry.json`)
//! 7. Capability registry query via Neural API
//! 8. TCP fallback (e.g., `127.0.0.1:9100`)
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

mod cap_probe;
mod capability_sockets;
mod engine;
mod engine_probes;
mod neural_api;
mod path_builder;
mod registry_queries;
mod result;
mod strategy;
mod transport;

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
mod engine_tests;
#[cfg(test)]
mod engine_tests2;
#[cfg(test)]
mod engine_tests3;

// Re-export primary types
pub use cap_probe::probe_unix_socket_capabilities_list;
pub use engine::SocketDiscovery;
pub use result::{DiscoveredSocket, DiscoveryMethod, PrimalManifest};
pub use strategy::DiscoveryStrategy;
pub use transport::TransportEndpoint;

use std::env;
use std::path::{Path, PathBuf};

// ============================================================================
// CONVENIENCE FUNCTIONS (for quick migrations from hardcoded paths)
// ============================================================================

/// Build deterministic socket path with optional tier-1 / tier-2 overrides (see `path_builder::build_socket_path`).
#[must_use]
pub fn build_socket_path_with_overrides(
    primal_name: &str,
    family_id: &str,
    primal_socket: Option<&str>,
    xdg_runtime_dir: Option<&Path>,
) -> PathBuf {
    path_builder::build_socket_path(primal_name, family_id, primal_socket, xdg_runtime_dir)
}

/// Discover socket for a primal (convenience function)
///
/// Uses default `family_id` from `FAMILY_ID` or `BIOMEOS_FAMILY_ID` environment.
pub async fn discover_socket(primal_name: &str) -> Option<PathBuf> {
    discover_socket_with_family(primal_name, None).await
}

/// Like [`discover_socket`], with an explicit `family_id` (skips `FAMILY_ID` / `BIOMEOS_FAMILY_ID` env).
pub async fn discover_socket_with_family(
    primal_name: &str,
    family_id: Option<&str>,
) -> Option<PathBuf> {
    let family_id = family_id
        .map(String::from)
        .or_else(|| env::var("FAMILY_ID").ok())
        .or_else(|| env::var("BIOMEOS_FAMILY_ID").ok())
        .unwrap_or_else(|| "default".to_string());

    let discovery = SocketDiscovery::new(family_id);
    discovery.get_socket_path(primal_name).await
}

/// Build socket path for a primal (convenience function)
///
/// Deterministic path building for primals to register their own sockets.
#[must_use]
pub fn build_socket(primal_name: &str, family_id: &str) -> PathBuf {
    let discovery = SocketDiscovery::new(family_id);
    discovery.build_socket_path(primal_name)
}

/// Discover transport endpoint with automatic fallback (convenience function)
///
/// **Universal IPC Standard v3.0**: Use this for cross-platform discovery.
pub async fn discover_endpoint(primal_name: &str) -> Option<TransportEndpoint> {
    discover_endpoint_with_family(primal_name, None).await
}

/// Like [`discover_endpoint`], with an explicit `family_id`.
pub async fn discover_endpoint_with_family(
    primal_name: &str,
    family_id: Option<&str>,
) -> Option<TransportEndpoint> {
    let family_id = family_id
        .map(String::from)
        .or_else(|| env::var("FAMILY_ID").ok())
        .or_else(|| env::var("BIOMEOS_FAMILY_ID").ok())
        .unwrap_or_else(|| "default".to_string());

    let discovery = SocketDiscovery::new(family_id);
    discovery.discover_with_fallback(primal_name).await
}

/// Discover transport endpoint by capability (convenience function)
///
/// **`WateringHole` standard**: No hardcoded primal names. Use capability constants
/// from `biomeos_types::constants::capability` (e.g., `capability::CRYPTO`).
pub async fn discover_endpoint_by_capability(capability: &str) -> Option<TransportEndpoint> {
    let family_id = env::var("FAMILY_ID")
        .or_else(|_| env::var("BIOMEOS_FAMILY_ID"))
        .unwrap_or_else(|_| "default".to_string());

    let discovery = SocketDiscovery::new(family_id);
    discovery
        .discover_capability(capability)
        .await
        .map(|s| s.endpoint)
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let _result = discover_socket_with_family("beardog", Some("env-family")).await;
        // Just verify it doesn't panic - result may be None if socket doesn't exist
    }

    #[tokio::test]
    async fn test_discover_socket_family_id_from_env() {
        // Verify family_id flows through to SocketDiscovery (no env mutation needed)
        let family_id = "env-family-id";
        let discovery = SocketDiscovery::new(family_id);
        assert_eq!(discovery.family_id.as_str(), "env-family-id");
    }

    #[tokio::test]
    async fn test_discover_endpoint_convenience() {
        let _result =
            discover_endpoint_with_family("nonexistent-primal", Some("test-default")).await;
        // Just verify it doesn't panic
    }

    #[tokio::test]
    async fn test_discover_socket_default_family() {
        let _result = discover_socket_with_family("beardog", None).await;
        // Should use "default" when no env vars set
    }
}
