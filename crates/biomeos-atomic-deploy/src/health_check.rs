//! Health checking for deployed primals
//!
//! **Universal IPC v3.0**: Uses AtomicClient for multi-transport health checks.
//!
//! Provides multi-level health checking:
//! - Level 1: Socket existence check (fast)
//! - Level 2: Socket type validation (fast)
//! - Level 3: JSON-RPC ping via AtomicClient (deep, validates primal is responding)

use anyhow::{Context, Result};
use biomeos_core::atomic_client::AtomicClient;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tracing::{debug, warn};

/// Health status of a primal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub is_healthy: bool,
    pub socket_exists: bool,
    pub socket_accessible: bool,
    /// JSON-RPC ping succeeded (if attempted)
    pub rpc_responsive: Option<bool>,
    /// Response latency in milliseconds (if RPC ping attempted)
    pub latency_ms: Option<u64>,
    pub message: Option<String>,
}

/// Health checker for primals
pub struct HealthChecker {
    runtime_dir: PathBuf,
    /// Timeout for JSON-RPC pings
    rpc_timeout: Duration,
}

impl HealthChecker {
    /// Create new health checker
    pub fn new(runtime_dir: PathBuf) -> Self {
        Self {
            runtime_dir,
            rpc_timeout: Duration::from_secs(5),
        }
    }

    /// Create with custom timeout
    pub fn with_timeout(runtime_dir: PathBuf, rpc_timeout: Duration) -> Self {
        Self {
            runtime_dir,
            rpc_timeout,
        }
    }

    /// Check health of a primal via its socket (socket existence only)
    pub async fn check_primal(&self, socket_path: &Path) -> Result<HealthStatus> {
        // Check 1: Socket exists
        if !socket_path.exists() {
            return Ok(HealthStatus {
                is_healthy: false,
                socket_exists: false,
                socket_accessible: false,
                rpc_responsive: None,
                latency_ms: None,
                message: Some(format!("Socket not found: {}", socket_path.display())),
            });
        }

        // Check 2: Socket is accessible (Unix socket)
        #[cfg(unix)]
        {
            use std::os::unix::fs::FileTypeExt;
            let metadata =
                std::fs::metadata(socket_path).context("Failed to get socket metadata")?;

            if !metadata.file_type().is_socket() {
                return Ok(HealthStatus {
                    is_healthy: false,
                    socket_exists: true,
                    socket_accessible: false,
                    rpc_responsive: None,
                    latency_ms: None,
                    message: Some("Path exists but is not a socket".to_string()),
                });
            }
        }

        Ok(HealthStatus {
            is_healthy: true,
            socket_exists: true,
            socket_accessible: true,
            rpc_responsive: None,
            latency_ms: None,
            message: Some("Socket operational".to_string()),
        })
    }

    /// Deep health check with JSON-RPC ping
    ///
    /// This validates that the primal is actually responding to requests,
    /// not just that its socket exists.
    pub async fn check_primal_deep(
        &self,
        socket_path: &Path,
        health_method: &str,
    ) -> Result<HealthStatus> {
        // First do basic socket check
        let basic = self.check_primal(socket_path).await?;
        if !basic.is_healthy {
            return Ok(basic);
        }

        // Now do JSON-RPC ping
        let start = std::time::Instant::now();

        match self.rpc_ping(socket_path, health_method).await {
            Ok(response) => {
                let latency_ms = start.elapsed().as_millis() as u64;
                debug!(
                    "💚 RPC ping succeeded: {} ({}ms)",
                    socket_path.display(),
                    latency_ms
                );

                Ok(HealthStatus {
                    is_healthy: true,
                    socket_exists: true,
                    socket_accessible: true,
                    rpc_responsive: Some(true),
                    latency_ms: Some(latency_ms),
                    message: response
                        .get("message")
                        .and_then(|m| m.as_str())
                        .map(String::from),
                })
            }
            Err(e) => {
                let latency_ms = start.elapsed().as_millis() as u64;
                warn!(
                    "🔴 RPC ping failed: {} - {} ({}ms)",
                    socket_path.display(),
                    e,
                    latency_ms
                );

                Ok(HealthStatus {
                    is_healthy: false,
                    socket_exists: true,
                    socket_accessible: true,
                    rpc_responsive: Some(false),
                    latency_ms: Some(latency_ms),
                    message: Some(format!("RPC ping failed: {}", e)),
                })
            }
        }
    }

    /// Send a JSON-RPC ping to a primal via AtomicClient (Universal IPC v3.0)
    ///
    /// Uses `AtomicClient` with configurable timeout for health checks.
    async fn rpc_ping(&self, socket_path: &Path, method: &str) -> Result<serde_json::Value> {
        // Create AtomicClient with configured timeout
        let client = AtomicClient::unix(socket_path).with_timeout(self.rpc_timeout);

        // Call the health method
        client
            .call(method, json!({}))
            .await
            .context("RPC ping failed")
    }

    /// Check health of all sockets in runtime dir matching a pattern
    pub async fn check_all(&self, pattern: &str) -> Result<Vec<(PathBuf, HealthStatus)>> {
        let mut results = Vec::new();

        let entries =
            std::fs::read_dir(&self.runtime_dir).context("Failed to read runtime directory")?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            if let Some(name) = path.file_name() {
                if name.to_string_lossy().contains(pattern) {
                    let status = self.check_primal(&path).await?;
                    results.push((path, status));
                }
            }
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::net::UnixListener;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_health_check_nonexistent_socket() {
        let temp_dir = TempDir::new().unwrap();
        let checker = HealthChecker::new(temp_dir.path().to_path_buf());

        let socket_path = temp_dir.path().join("nonexistent.sock");
        let status = checker.check_primal(&socket_path).await.unwrap();

        assert!(!status.is_healthy);
        assert!(!status.socket_exists);
        assert!(!status.socket_accessible);
        assert!(status.message.unwrap().contains("Socket not found"));
    }

    #[tokio::test]
    async fn test_health_check_valid_socket() {
        let temp_dir = TempDir::new().unwrap();
        let checker = HealthChecker::new(temp_dir.path().to_path_buf());

        let socket_path = temp_dir.path().join("test.sock");

        // Create a real Unix socket
        let _listener = UnixListener::bind(&socket_path).unwrap();

        let status = checker.check_primal(&socket_path).await.unwrap();

        assert!(status.is_healthy);
        assert!(status.socket_exists);
        assert!(status.socket_accessible);
    }

    #[tokio::test]
    async fn test_health_check_not_a_socket() {
        let temp_dir = TempDir::new().unwrap();
        let checker = HealthChecker::new(temp_dir.path().to_path_buf());

        let socket_path = temp_dir.path().join("not_a_socket.txt");

        // Create a regular file (not a socket)
        std::fs::write(&socket_path, "test").unwrap();

        let status = checker.check_primal(&socket_path).await.unwrap();

        assert!(!status.is_healthy);
        assert!(status.socket_exists);
        assert!(!status.socket_accessible);
        assert!(status.message.unwrap().contains("not a socket"));
    }

    #[tokio::test]
    async fn test_check_all() {
        let temp_dir = TempDir::new().unwrap();
        let checker = HealthChecker::new(temp_dir.path().to_path_buf());

        // Create multiple sockets
        let _socket1 = UnixListener::bind(temp_dir.path().join("beardog-tower.sock")).unwrap();
        let _socket2 = UnixListener::bind(temp_dir.path().join("beardog-node.sock")).unwrap();
        let _socket3 = UnixListener::bind(temp_dir.path().join("songbird-tower.sock")).unwrap();

        // Check all beardog sockets
        let results = checker.check_all("beardog").await.unwrap();

        assert_eq!(results.len(), 2);
        for (_, status) in &results {
            assert!(status.is_healthy);
        }
    }

    #[test]
    fn test_health_status_serialization() {
        let status = HealthStatus {
            is_healthy: true,
            socket_exists: true,
            socket_accessible: true,
            rpc_responsive: Some(true),
            latency_ms: Some(42),
            message: Some("Test message".to_string()),
        };

        // Test JSON round-trip
        let json = serde_json::to_string(&status).unwrap();
        let deserialized: HealthStatus = serde_json::from_str(&json).unwrap();

        assert_eq!(status.is_healthy, deserialized.is_healthy);
        assert_eq!(status.socket_exists, deserialized.socket_exists);
        assert_eq!(status.rpc_responsive, deserialized.rpc_responsive);
        assert_eq!(status.latency_ms, deserialized.latency_ms);
        assert_eq!(status.message, deserialized.message);
    }

    #[test]
    fn test_health_checker_with_timeout() {
        let checker = HealthChecker::with_timeout(PathBuf::from("/tmp"), Duration::from_secs(10));
        assert_eq!(checker.rpc_timeout, Duration::from_secs(10));
    }
}
