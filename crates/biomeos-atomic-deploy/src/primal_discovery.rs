// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Primal discovery - TRUE PRIMAL approach
//!
//! Discovers running primals via Unix socket scanning (no launching!)

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::time::{Duration, sleep};
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
    /// e.g., "beardog-test_family.sock" → "beardog"
    fn extract_name(socket_name: &str) -> Option<String> {
        socket_name
            .strip_suffix(".sock")?
            .split('-')
            .next()
            .map(|s| s.to_string())
    }

    /// Extract family ID from socket filename
    /// e.g., "beardog-test_family.sock" → Some("test_family")
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
            if path.extension().is_none_or(|ext| ext != "sock") {
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
            .ok_or_else(|| anyhow::anyhow!("Primal '{name}' not found. Is it running?"))
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
                anyhow::bail!("Timeout waiting for primal '{name}' (waited {timeout:?})");
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

    /// Check if a name is a known primal (delegates to `primal_names::is_known_primal`).
    fn is_primal_name(&self, name: &str) -> bool {
        biomeos_types::primal_names::is_known_primal(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // extract_name tests
    // ========================================================================

    #[test]
    fn test_extract_name() {
        assert_eq!(
            DiscoveredPrimal::extract_name("beardog-test_family.sock"),
            Some("beardog".to_string())
        );
        assert_eq!(
            DiscoveredPrimal::extract_name("songbird-test123.sock"),
            Some("songbird".to_string())
        );
        assert_eq!(DiscoveredPrimal::extract_name("invalid"), None);
    }

    #[test]
    fn test_extract_name_all_primals() {
        assert_eq!(
            DiscoveredPrimal::extract_name("toadstool-family123.sock"),
            Some("toadstool".to_string())
        );
        assert_eq!(
            DiscoveredPrimal::extract_name("nestgate-family123.sock"),
            Some("nestgate".to_string())
        );
        assert_eq!(
            DiscoveredPrimal::extract_name("squirrel-family123.sock"),
            Some("squirrel".to_string())
        );
        assert_eq!(
            DiscoveredPrimal::extract_name("biomeos-family123.sock"),
            Some("biomeos".to_string())
        );
    }

    #[test]
    fn test_extract_name_no_suffix() {
        assert_eq!(DiscoveredPrimal::extract_name("beardog-family"), None);
        assert_eq!(DiscoveredPrimal::extract_name("beardog"), None);
    }

    #[test]
    fn test_extract_name_no_dash() {
        // Socket without family ID: "beardog.sock" → "beardog"
        assert_eq!(
            DiscoveredPrimal::extract_name("beardog.sock"),
            Some("beardog".to_string())
        );
    }

    #[test]
    fn test_extract_name_neural_api() {
        assert_eq!(
            DiscoveredPrimal::extract_name("neural-api-cf7e8729.sock"),
            Some("neural".to_string())
        );
    }

    #[test]
    fn test_extract_name_empty() {
        assert_eq!(DiscoveredPrimal::extract_name(""), None);
        assert_eq!(
            DiscoveredPrimal::extract_name(".sock"),
            Some("".to_string())
        );
    }

    // ========================================================================
    // extract_family_id tests
    // ========================================================================

    #[test]
    fn test_extract_family_id() {
        assert_eq!(
            DiscoveredPrimal::extract_family_id("beardog-test_family.sock"),
            Some("test_family".to_string())
        );
        assert_eq!(
            DiscoveredPrimal::extract_family_id("songbird-test-123.sock"),
            Some("test-123".to_string())
        );
        assert_eq!(DiscoveredPrimal::extract_family_id("beardog.sock"), None);
    }

    #[test]
    fn test_extract_family_id_hash_style() {
        assert_eq!(
            DiscoveredPrimal::extract_family_id("beardog-1894e909e454.sock"),
            Some("1894e909e454".to_string())
        );
        assert_eq!(
            DiscoveredPrimal::extract_family_id("songbird-cf7e8729.sock"),
            Some("cf7e8729".to_string())
        );
    }

    #[test]
    fn test_extract_family_id_multi_dash() {
        // Family IDs with dashes: "neural-api-cf7e8729.sock" → "api-cf7e8729"
        assert_eq!(
            DiscoveredPrimal::extract_family_id("neural-api-cf7e8729.sock"),
            Some("api-cf7e8729".to_string())
        );
    }

    #[test]
    fn test_extract_family_id_empty() {
        assert_eq!(DiscoveredPrimal::extract_family_id(""), None);
        assert_eq!(DiscoveredPrimal::extract_family_id(".sock"), None);
    }

    #[test]
    fn test_extract_family_id_no_family() {
        assert_eq!(DiscoveredPrimal::extract_family_id("beardog.sock"), None);
    }

    // ========================================================================
    // is_primal_name tests
    // ========================================================================

    #[test]
    fn test_is_primal_name_known() {
        let temp_dir = tempfile::tempdir().unwrap();
        let discovery = PrimalDiscovery::new(temp_dir.path().to_path_buf()).unwrap();

        assert!(discovery.is_primal_name("beardog"));
        assert!(discovery.is_primal_name("songbird"));
        assert!(discovery.is_primal_name("toadstool"));
        assert!(discovery.is_primal_name("nestgate"));
        assert!(discovery.is_primal_name("squirrel"));
        assert!(discovery.is_primal_name("biomeos"));
        assert!(discovery.is_primal_name("biomeos-device-management"));
    }

    #[test]
    fn test_is_primal_name_springs() {
        let temp_dir = tempfile::tempdir().unwrap();
        let discovery = PrimalDiscovery::new(temp_dir.path().to_path_buf()).unwrap();

        assert!(discovery.is_primal_name("airspring"));
        assert!(discovery.is_primal_name("wetspring"));
        assert!(discovery.is_primal_name("neuralspring"));
        assert!(discovery.is_primal_name("groundspring"));
        assert!(discovery.is_primal_name("hotspring"));
        assert!(discovery.is_primal_name("healthspring"));
        assert!(discovery.is_primal_name("ludospring"));
    }

    #[test]
    fn test_is_primal_name_unknown() {
        let temp_dir = tempfile::tempdir().unwrap();
        let discovery = PrimalDiscovery::new(temp_dir.path().to_path_buf()).unwrap();

        assert!(!discovery.is_primal_name("unknown"));
        assert!(!discovery.is_primal_name(""));
        assert!(!discovery.is_primal_name("neural-api"));
        assert!(discovery.is_primal_name("BEARDOG")); // case-insensitive via primal_names
    }

    // ========================================================================
    // PrimalDiscovery creation tests
    // ========================================================================

    #[test]
    fn test_discovery_creates_dir() {
        let temp_dir = tempfile::tempdir().unwrap();
        let new_dir = temp_dir.path().join("new_runtime_dir");
        assert!(!new_dir.exists());

        let _discovery = PrimalDiscovery::new(new_dir.clone()).unwrap();
        assert!(new_dir.exists());
    }

    #[test]
    fn test_discovery_existing_dir() {
        let temp_dir = tempfile::tempdir().unwrap();
        let discovery = PrimalDiscovery::new(temp_dir.path().to_path_buf());
        assert!(discovery.is_ok());
    }

    // ========================================================================
    // discover_all tests
    // ========================================================================

    #[tokio::test]
    async fn test_discover_all_empty() {
        let temp_dir = tempfile::tempdir().unwrap();
        let discovery = PrimalDiscovery::new(temp_dir.path().to_path_buf()).unwrap();

        let primals = discovery.discover_all().await.unwrap();
        assert_eq!(primals.len(), 0);
    }

    #[tokio::test]
    async fn test_discover_all_ignores_non_sock_files() {
        let temp_dir = tempfile::tempdir().unwrap();
        // Create non-socket files
        std::fs::write(temp_dir.path().join("beardog.log"), b"log data").unwrap();
        std::fs::write(temp_dir.path().join("songbird.pid"), b"1234").unwrap();
        std::fs::write(temp_dir.path().join("config.toml"), b"[graph]").unwrap();

        let discovery = PrimalDiscovery::new(temp_dir.path().to_path_buf()).unwrap();
        let primals = discovery.discover_all().await.unwrap();
        assert_eq!(primals.len(), 0);
    }

    #[tokio::test]
    async fn test_find_primal_not_found() {
        let temp_dir = tempfile::tempdir().unwrap();
        let discovery = PrimalDiscovery::new(temp_dir.path().to_path_buf()).unwrap();

        let result = discovery.find_primal("beardog").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[tokio::test]
    async fn test_find_by_family_empty() {
        let temp_dir = tempfile::tempdir().unwrap();
        let discovery = PrimalDiscovery::new(temp_dir.path().to_path_buf()).unwrap();

        let primals = discovery.find_by_family("test_family").await.unwrap();
        assert!(primals.is_empty());
    }

    // ========================================================================
    // DiscoveredPrimal serialization tests
    // ========================================================================

    #[test]
    fn test_discovered_primal_serialize() {
        let primal = DiscoveredPrimal {
            name: "beardog".to_string(),
            socket_path: PathBuf::from("/run/user/1000/biomeos/beardog-cf7e.sock"),
            family_id: Some("cf7e".to_string()),
            discovered_at: chrono::Utc::now(),
            responsive: true,
        };

        let json = serde_json::to_string(&primal).unwrap();
        assert!(json.contains("beardog"));
        assert!(json.contains("cf7e"));

        let deserialized: DiscoveredPrimal = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, "beardog");
        assert_eq!(deserialized.family_id, Some("cf7e".to_string()));
        assert!(deserialized.responsive);
    }
}
