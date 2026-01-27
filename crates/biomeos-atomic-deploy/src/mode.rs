// biomeOS Mode Detection
//
// Determines whether biomeOS should operate in:
// - Bootstrap Mode (genesis - no ecosystem exists)
// - Coordinated Mode (participant - ecosystem exists)

use std::path::PathBuf;
use tokio::net::UnixStream;
use tokio::time::{timeout, Duration};
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
    /// 1. BIOMEOS_MODE environment variable (explicit override)
    /// 2. Auto-detect based on Tower Atomic presence
    ///
    /// Returns:
    /// - Bootstrap if Tower Atomic does not exist (or BIOMEOS_MODE=bootstrap)
    /// - Coordinated if Tower Atomic exists (or BIOMEOS_MODE=coordinated)
    pub async fn detect(family_id: &str) -> Self {
        info!("🔍 Detecting biomeOS operating mode...");

        // Priority 1: Check for explicit mode override
        if let Ok(mode_override) = std::env::var("BIOMEOS_MODE") {
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
        // Tower Atomic consists of BearDog + Songbird
        // If either is reachable, Tower Atomic exists

        let beardog_socket = format!("/tmp/beardog-{}.sock", family_id);
        let songbird_socket = format!("/tmp/songbird-{}.sock", family_id);

        // Check BearDog
        if Self::primal_reachable(&beardog_socket).await {
            debug!("✅ BearDog reachable at {}", beardog_socket);
            return true;
        }

        // Check Songbird
        if Self::primal_reachable(&songbird_socket).await {
            debug!("✅ Songbird reachable at {}", songbird_socket);
            return true;
        }

        debug!(
            "❌ Tower Atomic not found (checked {} and {})",
            beardog_socket, songbird_socket
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
    use super::*;
    use tempfile::tempdir;
    use tokio::net::UnixListener;

    #[tokio::test]
    async fn test_detect_bootstrap_mode() {
        // No Tower Atomic exists
        let mode = BiomeOsMode::detect("test-bootstrap").await;
        assert_eq!(mode, BiomeOsMode::Bootstrap);
    }

    #[tokio::test]
    async fn test_detect_coordinated_mode() {
        // Create mock BearDog socket
        let dir = tempdir().unwrap();
        let socket_path = dir.path().join("beardog-test-coord.sock");
        let _listener = UnixListener::bind(&socket_path).unwrap();

        // Should detect Tower Atomic exists (but socket path doesn't match family_id pattern)
        // This test needs adjustment for actual family_id-based paths
    }

    #[tokio::test]
    async fn test_primal_reachable_nonexistent() {
        let reachable = BiomeOsMode::primal_reachable("/tmp/nonexistent-socket.sock").await;
        assert!(!reachable);
    }
}
