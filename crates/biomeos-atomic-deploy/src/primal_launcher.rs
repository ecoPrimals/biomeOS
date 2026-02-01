//! Primal process launcher
//!
//! **TRUE ecoBin v2.0:** Isomorphic IPC support with automatic endpoint discovery.
//!
//! Modern Rust replacement for bash process management with:
//! - Automatic detection of Unix sockets OR TCP discovery files
//! - Platform-agnostic primal launching
//! - No hardcoded transport assumptions

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::process::Command;
use tokio::time::{sleep, Duration};
use tracing::{debug, info};

/// Primal instance information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalInstance {
    pub primal_name: String,
    pub pid: u32,
    pub socket_path: PathBuf,
    pub started_at: chrono::DateTime<chrono::Utc>,
}

impl PrimalInstance {
    /// Calculate uptime
    pub fn uptime(&self) -> chrono::Duration {
        chrono::Utc::now() - self.started_at
    }

    /// Check if process is still running
    ///
    /// Uses signal 0 (null signal) to test process existence without affecting it.
    /// This is safe and idiomatic using the nix crate's signal handling.
    pub fn is_running(&self) -> bool {
        use nix::sys::signal::kill;
        use nix::unistd::Pid;

        // Signal 0 checks process existence without sending an actual signal
        // Returns Ok if process exists and we have permission to signal it
        kill(Pid::from_raw(self.pid as i32), None).is_ok()
    }
}

/// Primal launcher - manages primal lifecycle
#[derive(Debug)]
pub struct PrimalLauncher {
    binary_dir: PathBuf,
    _runtime_dir: PathBuf,
}

impl PrimalLauncher {
    /// Create new launcher
    pub fn new(binary_dir: PathBuf, runtime_dir: PathBuf) -> Result<Self> {
        if !binary_dir.exists() {
            anyhow::bail!("Binary directory not found: {}", binary_dir.display());
        }

        std::fs::create_dir_all(&runtime_dir).context("Failed to create runtime directory")?;

        Ok(Self {
            binary_dir,
            _runtime_dir: runtime_dir,
        })
    }

    /// Launch a primal with environment configuration
    pub async fn launch(
        &mut self,
        primal_name: &str,
        env: HashMap<String, String>,
    ) -> Result<PrimalInstance> {
        let binary_path = self.find_binary(primal_name)?;

        debug!("Launching {} from {}", primal_name, binary_path.display());

        // Extract socket path from env
        let socket_env_key = self.socket_env_key(primal_name);
        let socket_path = env
            .get(socket_env_key)
            .ok_or_else(|| anyhow::anyhow!("Socket path not provided for {}", primal_name))?
            .clone();

        // Clean up old socket if it exists (Unix sockets only)
        let socket_path_buf = PathBuf::from(&socket_path);
        if socket_path_buf.exists() {
            std::fs::remove_file(&socket_path_buf).context("Failed to remove old socket")?;
        }
        
        // Note: TCP discovery files are automatically cleaned up by the primal itself
        // when it starts (part of isomorphic IPC pattern)

        // Build command
        let mut cmd = Command::new(&binary_path);

        // Set environment
        for (key, value) in env {
            cmd.env(key, value);
        }

        // Redirect stdio (don't block on output)
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        // Spawn process
        let child = cmd
            .spawn()
            .context(format!("Failed to spawn {}", primal_name))?;

        let pid = child
            .id()
            .ok_or_else(|| anyhow::anyhow!("Failed to get PID"))?;

        info!("   Launched {} (PID: {})", primal_name, pid);

        // Wait for socket to appear
        self.wait_for_socket(&socket_path_buf, Duration::from_secs(5))
            .await?;

        // Detach child (let it run independently)
        std::mem::forget(child);

        Ok(PrimalInstance {
            primal_name: primal_name.to_string(),
            pid,
            socket_path: socket_path_buf,
            started_at: chrono::Utc::now(),
        })
    }

    /// Find binary for a primal
    fn find_binary(&self, primal_name: &str) -> Result<PathBuf> {
        // Try primals subdirectory first
        let primals_path = self.binary_dir.join("primals").join(primal_name);
        if primals_path.exists() {
            return Ok(primals_path);
        }

        // Try root binary directory
        let root_path = self.binary_dir.join(primal_name);
        if root_path.exists() {
            return Ok(root_path);
        }

        anyhow::bail!(
            "Binary not found: {} (searched in {} and {})",
            primal_name,
            primals_path.display(),
            root_path.display()
        )
    }

    /// Get socket environment variable key for a primal
    fn socket_env_key(&self, primal_name: &str) -> &'static str {
        match primal_name {
            "beardog-server" => "BEARDOG_SOCKET",
            "songbird-orchestrator" => "SONGBIRD_SOCKET",
            "toadstool" => "TOADSTOOL_SOCKET",
            "nestgate" => "NESTGATE_SOCKET",
            _ => "PRIMAL_SOCKET",
        }
    }

    /// Wait for socket or discovery file to appear
    ///
    /// **TRUE ecoBin v2.0:** Supports isomorphic IPC detection.
    ///
    /// This checks for:
    /// 1. Unix socket at specified path (optimal)
    /// 2. TCP discovery file if Unix socket doesn't appear (Android fallback)
    async fn wait_for_socket(&self, socket_path: &Path, timeout: Duration) -> Result<()> {
        use biomeos_core::ipc::detect_best_transport;
        
        let start = std::time::Instant::now();
        let service_name = socket_path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| anyhow::anyhow!("Invalid socket path: no filename"))?;

        while start.elapsed() < timeout {
            // Try Unix socket first
            if socket_path.exists() {
                debug!("   ✅ Unix socket appeared: {}", socket_path.display());
                return Ok(());
            }

            // Try TCP discovery file
            if detect_best_transport(service_name).is_ok() {
                debug!("   ✅ TCP discovery file appeared for: {}", service_name);
                return Ok(());
            }

            sleep(Duration::from_millis(100)).await;
        }

        anyhow::bail!(
            "Timeout waiting for socket or discovery file: {} ({}s)",
            socket_path.display(),
            timeout.as_secs()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_socket_env_key() {
        let launcher = PrimalLauncher {
            binary_dir: PathBuf::from("/tmp"),
            _runtime_dir: PathBuf::from("/tmp"),
        };

        assert_eq!(launcher.socket_env_key("beardog-server"), "BEARDOG_SOCKET");
        assert_eq!(
            launcher.socket_env_key("songbird-orchestrator"),
            "SONGBIRD_SOCKET"
        );
        assert_eq!(launcher.socket_env_key("toadstool"), "TOADSTOOL_SOCKET");
        assert_eq!(launcher.socket_env_key("nestgate"), "NESTGATE_SOCKET");
        assert_eq!(launcher.socket_env_key("unknown"), "PRIMAL_SOCKET");
    }

    #[test]
    fn test_primal_launcher_creation() {
        let temp_dir = TempDir::new().unwrap();
        let binary_dir = temp_dir.path().join("bin");
        let runtime_dir = temp_dir.path().join("runtime");

        // Create binary directory
        std::fs::create_dir(&binary_dir).unwrap();

        let launcher = PrimalLauncher::new(binary_dir.clone(), runtime_dir.clone());
        assert!(launcher.is_ok());

        // Verify runtime dir was created
        assert!(runtime_dir.exists());
    }

    #[test]
    fn test_primal_launcher_missing_binary_dir() {
        let temp_dir = TempDir::new().unwrap();
        let binary_dir = temp_dir.path().join("nonexistent");
        let runtime_dir = temp_dir.path().join("runtime");

        let result = PrimalLauncher::new(binary_dir, runtime_dir);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Binary directory not found"));
    }

    #[test]
    fn test_find_binary() {
        let temp_dir = TempDir::new().unwrap();
        let binary_dir = temp_dir.path().join("bin");
        let runtime_dir = temp_dir.path().join("runtime");

        std::fs::create_dir(&binary_dir).unwrap();

        // Create a test binary in primals subdirectory
        let primals_dir = binary_dir.join("primals");
        std::fs::create_dir(&primals_dir).unwrap();
        let test_binary = primals_dir.join("test-primal");
        std::fs::write(&test_binary, "#!/bin/bash\necho test").unwrap();

        let launcher = PrimalLauncher::new(binary_dir.clone(), runtime_dir).unwrap();
        let found = launcher.find_binary("test-primal").unwrap();
        assert_eq!(found, test_binary);

        // Test binary not found
        let result = launcher.find_binary("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_primal_instance_serialization() {
        let instance = PrimalInstance {
            primal_name: "beardog-server".to_string(),
            pid: 12345,
            socket_path: PathBuf::from("/tmp/test.sock"),
            started_at: chrono::Utc::now(),
        };

        // Test JSON round-trip
        let json = serde_json::to_string(&instance).unwrap();
        let deserialized: PrimalInstance = serde_json::from_str(&json).unwrap();

        assert_eq!(instance.primal_name, deserialized.primal_name);
        assert_eq!(instance.pid, deserialized.pid);
        assert_eq!(instance.socket_path, deserialized.socket_path);
    }

    #[test]
    fn test_primal_instance_uptime() {
        let instance = PrimalInstance {
            primal_name: "test".to_string(),
            pid: 1,
            socket_path: PathBuf::from("/tmp/test.sock"),
            started_at: chrono::Utc::now() - chrono::Duration::seconds(10),
        };

        let uptime = instance.uptime();
        assert!(uptime.num_seconds() >= 10);
        assert!(uptime.num_seconds() < 11); // Should be ~10 seconds
    }

    #[tokio::test]
    async fn test_wait_for_socket_timeout() {
        let temp_dir = TempDir::new().unwrap();
        let binary_dir = temp_dir.path().join("bin");
        let runtime_dir = temp_dir.path().join("runtime");

        std::fs::create_dir(&binary_dir).unwrap();

        let launcher = PrimalLauncher::new(binary_dir, runtime_dir).unwrap();

        let socket_path = temp_dir.path().join("nonexistent.sock");
        let result = launcher
            .wait_for_socket(&socket_path, Duration::from_millis(100))
            .await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Timeout"));
    }

    #[tokio::test]
    async fn test_wait_for_socket_success() {
        let temp_dir = TempDir::new().unwrap();
        let binary_dir = temp_dir.path().join("bin");
        let runtime_dir = temp_dir.path().join("runtime");

        std::fs::create_dir(&binary_dir).unwrap();

        let launcher = PrimalLauncher::new(binary_dir, runtime_dir).unwrap();

        let socket_path = temp_dir.path().join("test.sock");

        // Create socket in background after 100ms
        let socket_path_clone = socket_path.clone();
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(100)).await;
            std::fs::write(&socket_path_clone, "").unwrap();
        });

        let result = launcher
            .wait_for_socket(&socket_path, Duration::from_secs(1))
            .await;
        assert!(result.is_ok());
    }
}
