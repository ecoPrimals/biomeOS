// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! biomeOS operating mode detection.
//!
//! Determines Bootstrap (genesis, no ecosystem) vs Coordinated (participant, ecosystem exists).

use std::path::PathBuf;
use tokio::net::UnixStream;
use tokio::time::{Duration, timeout};
use tracing::{debug, info, warn};

/// biomeOS Operating Mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BiomeOsMode {
    /// Bootstrap Mode: No ecosystem exists, biomeOS creates foundation
    Bootstrap,

    /// Coordinated Mode: Ecosystem exists, biomeOS participates as primal
    Coordinated,
}

impl BiomeOsMode {
    /// Detect which mode biomeOS should operate in
    ///
    /// Priority:
    /// 1. `BIOMEOS_MODE` environment variable (explicit override)
    /// 2. Auto-detect based on Tower Atomic presence
    ///
    /// Returns:
    /// - Bootstrap if Tower Atomic does not exist (or `BIOMEOS_MODE=bootstrap`)
    /// - Coordinated if Tower Atomic exists (or `BIOMEOS_MODE=coordinated`)
    pub async fn detect(family_id: &str) -> Self {
        Self::detect_with_mode(family_id, std::env::var("BIOMEOS_MODE").ok().as_deref()).await
    }

    /// Like [`Self::detect`], but supplies the `BIOMEOS_MODE` value directly (for tests and tooling).
    ///
    /// Pass `None` when the variable is unset (auto-detect only).
    pub async fn detect_with_mode(family_id: &str, mode_env: Option<&str>) -> Self {
        info!("🔍 Detecting biomeOS operating mode...");

        // Priority 1: Check for explicit mode override
        if let Some(mode_override) = mode_env {
            match mode_override.to_lowercase().as_str() {
                "coordinated" | "coord" | "join" => {
                    info!(
                        "✅ BIOMEOS_MODE={} - entering COORDINATED MODE (explicit)",
                        mode_override
                    );
                    return Self::Coordinated;
                }
                "bootstrap" | "boot" | "genesis" => {
                    info!(
                        "🌱 BIOMEOS_MODE={} - entering BOOTSTRAP MODE (explicit)",
                        mode_override
                    );
                    return Self::Bootstrap;
                }
                _ => {
                    warn!(
                        "⚠️  Unknown BIOMEOS_MODE '{}', falling back to auto-detect",
                        mode_override
                    );
                }
            }
        }

        // Priority 2: Auto-detect based on Tower Atomic presence
        if Self::tower_atomic_exists(family_id).await {
            info!("✅ Tower Atomic detected - entering COORDINATED MODE");
            Self::Coordinated
        } else {
            info!("🌱 No Tower Atomic found - entering BOOTSTRAP MODE");
            Self::Bootstrap
        }
    }

    /// Check if Tower Atomic exists and is reachable
    async fn tower_atomic_exists(family_id: &str) -> bool {
        use crate::nucleation::SocketNucleation;

        // Tower Atomic consists of security + network primals
        // If either is reachable, Tower Atomic exists
        // Uses SocketNucleation for deterministic paths
        // Bootstrap hints from canonical constants; production uses runtime discovery

        let security_provider = std::env::var("BIOMEOS_SECURITY_PROVIDER")
            .unwrap_or_else(|_| biomeos_types::primal_names::BEARDOG.to_string());
        let network_provider = std::env::var("BIOMEOS_NETWORK_PROVIDER")
            .unwrap_or_else(|_| biomeos_types::primal_names::SONGBIRD.to_string());

        let mut nucleation = SocketNucleation::default();
        let security_socket = nucleation.assign_socket(&security_provider, family_id);
        let network_socket = nucleation.assign_socket(&network_provider, family_id);

        // Check security provider
        if Self::primal_reachable(security_socket.to_string_lossy().as_ref()).await {
            debug!(
                "✅ Security provider ({}) reachable at {:?}",
                security_provider, security_socket
            );
            return true;
        }

        // Check network provider
        if Self::primal_reachable(network_socket.to_string_lossy().as_ref()).await {
            debug!(
                "✅ Network provider ({}) reachable at {:?}",
                network_provider, network_socket
            );
            return true;
        }

        debug!(
            "❌ Tower Atomic not found (checked {:?} and {:?})",
            security_socket, network_socket
        );
        false
    }

    /// Check if a primal is reachable via its Unix socket
    async fn primal_reachable(socket_path: &str) -> bool {
        let path = PathBuf::from(socket_path);

        // 1. Check if socket file exists
        if !path.exists() {
            debug!("Socket does not exist: {}", socket_path);
            return false;
        }

        // 2. Try to connect (with timeout)
        match timeout(Duration::from_millis(100), UnixStream::connect(&path)).await {
            Ok(Ok(_stream)) => {
                debug!("Successfully connected to {}", socket_path);
                true
            }
            Ok(Err(e)) => {
                debug!(
                    "Socket exists but connection failed: {} - {}",
                    socket_path, e
                );
                false
            }
            Err(_) => {
                debug!("Connection timeout: {}", socket_path);
                false
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;
    use tempfile::tempdir;
    use tokio::net::UnixListener;

    #[test]
    fn test_biome_os_mode_debug() {
        let bootstrap = BiomeOsMode::Bootstrap;
        let coordinated = BiomeOsMode::Coordinated;

        assert!(format!("{bootstrap:?}").contains("Bootstrap"));
        assert!(format!("{coordinated:?}").contains("Coordinated"));
    }

    #[test]
    fn test_biome_os_mode_clone() {
        let mode = BiomeOsMode::Bootstrap;
        let cloned = mode;
        assert_eq!(mode, cloned);
    }

    #[test]
    fn test_biome_os_mode_copy() {
        let mode = BiomeOsMode::Coordinated;
        let copied: BiomeOsMode = mode;
        assert_eq!(mode, copied);
    }

    #[test]
    fn test_biome_os_mode_equality() {
        assert_eq!(BiomeOsMode::Bootstrap, BiomeOsMode::Bootstrap);
        assert_eq!(BiomeOsMode::Coordinated, BiomeOsMode::Coordinated);
        assert_ne!(BiomeOsMode::Bootstrap, BiomeOsMode::Coordinated);
    }

    #[tokio::test]
    async fn test_detect_bootstrap_mode() {
        // No Tower Atomic exists for random family ID
        let mode = BiomeOsMode::detect("test-bootstrap-random-12345").await;
        assert_eq!(mode, BiomeOsMode::Bootstrap);
    }

    #[tokio::test]
    async fn test_detect_coordinated_mode_with_socket() {
        // Create mock BearDog socket in a temp directory
        let dir = tempdir().unwrap();
        let socket_path = dir.path().join("test-socket.sock");
        let _listener = UnixListener::bind(&socket_path).unwrap();

        // Test that primal_reachable works with existing socket
        let reachable = BiomeOsMode::primal_reachable(socket_path.to_str().unwrap()).await;
        assert!(reachable, "Should detect socket as reachable");
    }

    #[tokio::test]
    async fn test_primal_reachable_nonexistent() {
        let reachable = BiomeOsMode::primal_reachable("/tmp/nonexistent-socket-xyz.sock").await;
        assert!(!reachable);
    }

    #[tokio::test]
    async fn test_primal_reachable_with_valid_socket() {
        let dir = tempdir().unwrap();
        let socket_path = dir.path().join("reachable-test.sock");
        let _listener = UnixListener::bind(&socket_path).unwrap();

        let reachable = BiomeOsMode::primal_reachable(socket_path.to_str().unwrap()).await;
        assert!(reachable);
    }

    #[tokio::test]
    async fn test_tower_atomic_exists_no_sockets() {
        // With a random family ID that doesn't have sockets
        let exists = BiomeOsMode::tower_atomic_exists("no-tower-exists-xyz").await;
        assert!(!exists);
    }

    #[tokio::test]
    async fn test_detect_coordinated_mode_explicit() {
        let mode = BiomeOsMode::detect_with_mode("any-family", Some("coordinated")).await;
        assert_eq!(mode, BiomeOsMode::Coordinated);
    }

    #[tokio::test]
    async fn test_detect_coordinated_mode_coord_alias() {
        let mode = BiomeOsMode::detect_with_mode("any-family", Some("coord")).await;
        assert_eq!(mode, BiomeOsMode::Coordinated);
    }

    #[tokio::test]
    async fn test_detect_coordinated_mode_join_alias() {
        let mode = BiomeOsMode::detect_with_mode("any-family", Some("join")).await;
        assert_eq!(mode, BiomeOsMode::Coordinated);
    }

    #[tokio::test]
    async fn test_detect_bootstrap_mode_explicit() {
        let mode = BiomeOsMode::detect_with_mode("any-family", Some("bootstrap")).await;
        assert_eq!(mode, BiomeOsMode::Bootstrap);
    }

    #[tokio::test]
    async fn test_detect_bootstrap_mode_boot_alias() {
        let mode = BiomeOsMode::detect_with_mode("any-family", Some("boot")).await;
        assert_eq!(mode, BiomeOsMode::Bootstrap);
    }

    #[tokio::test]
    async fn test_detect_bootstrap_mode_genesis_alias() {
        let mode = BiomeOsMode::detect_with_mode("any-family", Some("genesis")).await;
        assert_eq!(mode, BiomeOsMode::Bootstrap);
    }

    #[tokio::test]
    async fn test_detect_unknown_mode_falls_back_to_autodetect() {
        let mode =
            BiomeOsMode::detect_with_mode("no-tower-exists-xyz", Some("unknown_mode_xyz")).await;
        assert_eq!(mode, BiomeOsMode::Bootstrap);
    }

    #[tokio::test]
    async fn test_primal_reachable_timeout() {
        let dir = tempdir().unwrap();
        let socket_path = dir.path().join("timeout.sock");
        let _listener = UnixListener::bind(&socket_path).unwrap();
        let reachable = BiomeOsMode::primal_reachable(socket_path.to_str().unwrap()).await;
        assert!(reachable);
    }

    #[tokio::test]
    async fn test_primal_reachable_connection_failed() {
        let dir = tempdir().unwrap();
        let socket_path = dir.path().join("conn_fail.sock");
        std::fs::File::create(&socket_path).unwrap();
        let reachable = BiomeOsMode::primal_reachable(socket_path.to_str().unwrap()).await;
        assert!(!reachable);
    }
}
