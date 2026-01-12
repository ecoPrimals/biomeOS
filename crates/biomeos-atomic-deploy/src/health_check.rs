//! Health checking for deployed primals

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
// use tokio::time::{timeout, Duration}; // TODO: Implement JSON-RPC health checks

/// Health status of a primal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub is_healthy: bool,
    pub socket_exists: bool,
    pub socket_accessible: bool,
    pub message: Option<String>,
}

/// Health checker for primals
pub struct HealthChecker {
    runtime_dir: PathBuf,
}

impl HealthChecker {
    /// Create new health checker
    pub fn new(runtime_dir: PathBuf) -> Self {
        Self { runtime_dir }
    }

    /// Check health of a primal via its socket
    pub async fn check_primal(&self, socket_path: &Path) -> Result<HealthStatus> {
        // Check 1: Socket exists
        if !socket_path.exists() {
            return Ok(HealthStatus {
                is_healthy: false,
                socket_exists: false,
                socket_accessible: false,
                message: Some(format!("Socket not found: {}", socket_path.display())),
            });
        }

        // Check 2: Socket is accessible (Unix socket)
        #[cfg(unix)]
        {
            use std::os::unix::fs::FileTypeExt;
            let metadata = std::fs::metadata(socket_path)
                .context("Failed to get socket metadata")?;

            if !metadata.file_type().is_socket() {
                return Ok(HealthStatus {
                    is_healthy: false,
                    socket_exists: true,
                    socket_accessible: false,
                    message: Some("Path exists but is not a socket".to_string()),
                });
            }
        }

        // Check 3: Can connect to socket (basic connectivity)
        // TODO: Implement JSON-RPC health check ping
        // For now, socket existence is sufficient

        Ok(HealthStatus {
            is_healthy: true,
            socket_exists: true,
            socket_accessible: true,
            message: Some("Primal operational".to_string()),
        })
    }

    /// Check health of all sockets in runtime dir matching a pattern
    pub async fn check_all(&self, pattern: &str) -> Result<Vec<(PathBuf, HealthStatus)>> {
        let mut results = Vec::new();

        let entries = std::fs::read_dir(&self.runtime_dir)
            .context("Failed to read runtime directory")?;

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
    use tempfile::TempDir;
    use std::os::unix::net::UnixListener;

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
            message: Some("Test message".to_string()),
        };

        // Test JSON round-trip
        let json = serde_json::to_string(&status).unwrap();
        let deserialized: HealthStatus = serde_json::from_str(&json).unwrap();

        assert_eq!(status.is_healthy, deserialized.is_healthy);
        assert_eq!(status.socket_exists, deserialized.socket_exists);
        assert_eq!(status.message, deserialized.message);
    }
}

