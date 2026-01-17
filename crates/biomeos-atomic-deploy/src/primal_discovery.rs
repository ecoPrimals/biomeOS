//! Primal discovery - TRUE PRIMAL approach
//!
//! Discovers running primals via Unix socket scanning (no launching!)

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::time::{sleep, Duration};
use tracing::{debug, info, warn};

/// Discovered primal information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredPrimal {
    /// Primal name (e.g., "beardog", "songbird")
    pub name: String,

    /// Unix socket path
    pub socket_path: PathBuf,

    /// Family ID (extracted from socket name)
    pub family_id: Option<String>,

    /// Discovered at timestamp
    pub discovered_at: chrono::DateTime<chrono::Utc>,

    /// Is responsive (can we connect?)
    pub responsive: bool,
}

impl DiscoveredPrimal {
    /// Extract primal name from socket filename
    /// e.g., "beardog-nat0.sock" → "beardog"
    fn extract_name(socket_name: &str) -> Option<String> {
        socket_name
            .strip_suffix(".sock")?
            .split('-')
            .next()
            .map(|s| s.to_string())
    }

    /// Extract family ID from socket filename
    /// e.g., "beardog-nat0.sock" → Some("nat0")
    fn extract_family_id(socket_name: &str) -> Option<String> {
        let without_sock = socket_name.strip_suffix(".sock")?;
        let parts: Vec<&str> = without_sock.split('-').collect();

        if parts.len() >= 2 {
            Some(parts[1..].join("-"))
        } else {
            None
        }
    }
}

/// Primal discovery service
#[derive(Debug)]
pub struct PrimalDiscovery {
    /// Runtime socket directory
    runtime_dir: PathBuf,
}

impl PrimalDiscovery {
    /// Create new discovery service
    pub fn new(runtime_dir: PathBuf) -> Result<Self> {
        if !runtime_dir.exists() {
            std::fs::create_dir_all(&runtime_dir).with_context(|| {
                format!(
                    "Failed to create runtime directory: {}",
                    runtime_dir.display()
                )
            })?;
        }

        Ok(Self { runtime_dir })
    }

    /// Discover all primals in the runtime directory
    pub async fn discover_all(&self) -> Result<Vec<DiscoveredPrimal>> {
        info!("Discovering primals in {}", self.runtime_dir.display());

        let mut primals = Vec::new();

        let mut entries = tokio::fs::read_dir(&self.runtime_dir)
            .await
            .with_context(|| {
                format!(
                    "Failed to read runtime directory: {}",
                    self.runtime_dir.display()
                )
            })?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();

            // Only consider .sock files
            if !path.extension().map_or(false, |ext| ext == "sock") {
                continue;
            }

            // Check if it's a Unix socket
            let metadata = match tokio::fs::metadata(&path).await {
                Ok(m) => m,
                Err(e) => {
                    warn!("Failed to get metadata for {}: {}", path.display(), e);
                    continue;
                }
            };

            #[cfg(unix)]
            {
                use std::os::unix::fs::FileTypeExt;
                if !metadata.file_type().is_socket() {
                    continue;
                }
            }

            // Extract primal info from socket name
            let socket_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

            if let Some(name) = DiscoveredPrimal::extract_name(socket_name) {
                // Filter to known primal names
                if !self.is_primal_name(&name) {
                    continue;
                }

                let family_id = DiscoveredPrimal::extract_family_id(socket_name);

                // Test responsiveness (try to connect)
                let responsive = self.test_socket(&path).await;

                let primal = DiscoveredPrimal {
                    name: name.clone(),
                    socket_path: path,
                    family_id,
                    discovered_at: chrono::Utc::now(),
                    responsive,
                };

                debug!("Discovered primal: {} at {:?}", name, primal.socket_path);
                primals.push(primal);
            }
        }

        info!("Discovered {} primal(s)", primals.len());
        Ok(primals)
    }

    /// Find a specific primal by name
    pub async fn find_primal(&self, name: &str) -> Result<DiscoveredPrimal> {
        let all_primals = self.discover_all().await?;

        all_primals
            .into_iter()
            .find(|p| p.name == name)
            .ok_or_else(|| anyhow::anyhow!("Primal '{}' not found. Is it running?", name))
    }

    /// Find primals by family ID
    pub async fn find_by_family(&self, family_id: &str) -> Result<Vec<DiscoveredPrimal>> {
        let all_primals = self.discover_all().await?;

        Ok(all_primals
            .into_iter()
            .filter(|p| p.family_id.as_deref() == Some(family_id))
            .collect())
    }

    /// Wait for a primal to appear
    pub async fn wait_for_primal(&self, name: &str, timeout: Duration) -> Result<DiscoveredPrimal> {
        let start = std::time::Instant::now();

        loop {
            if let Ok(primal) = self.find_primal(name).await {
                return Ok(primal);
            }

            if start.elapsed() > timeout {
                anyhow::bail!(
                    "Timeout waiting for primal '{}' (waited {:?})",
                    name,
                    timeout
                );
            }

            sleep(Duration::from_millis(500)).await;
        }
    }

    /// Test if a socket is responsive
    async fn test_socket(&self, socket_path: &Path) -> bool {
        #[cfg(unix)]
        {
            use tokio::net::UnixStream;

            // Try to connect (don't send data, just test connectivity)
            match tokio::time::timeout(Duration::from_secs(1), UnixStream::connect(socket_path))
                .await
            {
                Ok(Ok(_stream)) => true,
                Ok(Err(e)) => {
                    debug!("Socket {} not responsive: {}", socket_path.display(), e);
                    false
                }
                Err(_) => {
                    debug!("Socket {} connection timeout", socket_path.display());
                    false
                }
            }
        }

        #[cfg(not(unix))]
        {
            // On non-Unix systems, assume socket exists = responsive
            socket_path.exists()
        }
    }

    /// Check if a name is a known primal
    fn is_primal_name(&self, name: &str) -> bool {
        matches!(
            name,
            "beardog"
                | "songbird"
                | "toadstool"
                | "nestgate"
                | "squirrel"
                | "biomeos"
                | "biomeos-device-management"
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_name() {
        assert_eq!(
            DiscoveredPrimal::extract_name("beardog-nat0.sock"),
            Some("beardog".to_string())
        );
        assert_eq!(
            DiscoveredPrimal::extract_name("songbird-test123.sock"),
            Some("songbird".to_string())
        );
        assert_eq!(DiscoveredPrimal::extract_name("invalid"), None);
    }

    #[test]
    fn test_extract_family_id() {
        assert_eq!(
            DiscoveredPrimal::extract_family_id("beardog-nat0.sock"),
            Some("nat0".to_string())
        );
        assert_eq!(
            DiscoveredPrimal::extract_family_id("songbird-test-123.sock"),
            Some("test-123".to_string())
        );
        assert_eq!(DiscoveredPrimal::extract_family_id("beardog.sock"), None);
    }

    #[tokio::test]
    async fn test_discover_all() {
        let temp_dir = tempfile::tempdir().unwrap();
        let discovery = PrimalDiscovery::new(temp_dir.path().to_path_buf()).unwrap();

        // Should discover empty (no sockets yet)
        let primals = discovery.discover_all().await.unwrap();
        assert_eq!(primals.len(), 0);
    }
}
