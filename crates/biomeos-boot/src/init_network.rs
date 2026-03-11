//! Network Configuration for BiomeOS Init
//!
//! Configures network interfaces during boot.

use crate::init_error::Result;
use std::path::Path;
use tracing::info;

/// Network configuration manager
pub struct NetworkManager {
    configured: bool,
}

impl NetworkManager {
    /// Creates a new network manager
    pub fn new() -> Self {
        Self { configured: false }
    }

    /// Configures network interfaces
    ///
    /// Detects available interfaces and marks as configured when at least one
    /// non-loopback interface is found. Future expansion: DHCP/static IP,
    /// DNS, mDNS for service discovery.
    ///
    /// # Errors
    ///
    /// Returns an error if network configuration fails.
    pub async fn configure(&mut self) -> Result<()> {
        info!("🌐 Configuring network...");

        let interfaces = self.detect_interfaces().await?;
        self.configured = !interfaces.is_empty();
        if self.configured {
            info!(
                "✅ Network configuration complete ({} interface(s): {:?})",
                interfaces.len(),
                interfaces
            );
        } else {
            info!("⚠️ No non-loopback interfaces detected");
        }
        Ok(())
    }

    /// Checks if network is configured
    pub fn is_configured(&self) -> bool {
        self.configured
    }

    /// Detects available network interfaces
    ///
    /// Reads from `/sys/class/net/` to list interfaces, skipping loopback (lo).
    ///
    /// # Errors
    ///
    /// Returns an error if interface detection fails.
    pub async fn detect_interfaces(&self) -> Result<Vec<String>> {
        let net_dir = Path::new("/sys/class/net");
        if !net_dir.exists() {
            return Ok(vec![]);
        }

        let mut interfaces = Vec::new();
        let mut entries = tokio::fs::read_dir(net_dir)
            .await
            .map_err(|e| crate::init_error::BootError::NetworkConfig(Box::new(e)))?;

        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|e| crate::init_error::BootError::NetworkConfig(Box::new(e)))?
        {
            let name = entry
                .file_name()
                .into_string()
                .map_err(|_| crate::init_error::BootError::NetworkInterfaceDetection)?;
            if name != "lo" {
                interfaces.push(name);
            }
        }

        Ok(interfaces)
    }
}

impl Default for NetworkManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_manager_creation() {
        let mgr = NetworkManager::new();
        assert!(!mgr.is_configured());
    }

    #[tokio::test]
    async fn test_network_configuration() {
        let mut mgr = NetworkManager::new();
        assert!(mgr.configure().await.is_ok());
        // configured is true iff at least one non-loopback interface was detected
        let interfaces = mgr.detect_interfaces().await.unwrap();
        assert_eq!(mgr.is_configured(), !interfaces.is_empty());
    }

    #[tokio::test]
    async fn test_interface_detection() {
        let mgr = NetworkManager::new();
        let result = mgr.detect_interfaces().await;
        assert!(result.is_ok());
    }
}
