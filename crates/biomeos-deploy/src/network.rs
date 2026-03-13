// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Network bridge management
//!
//! ## Shell-outs (accepted - privileged network configuration)
//!
//! The `create()` and `destroy()` methods call `sudo ip link/addr` to manage
//! Linux network bridges. These require root privileges and cannot be replaced
//! with pure Rust without a netlink library (`netlink-rs` or `rtnetlink`).
//!
//! The `exists()` method has been evolved to pure Rust via `/sys/class/net/`.
//!
//! **Evolution path**: When the `rtnetlink` crate stabilizes for our use case,
//! bridge create/delete/configure operations can be replaced with Netlink
//! socket calls, eliminating the `sudo ip` dependency entirely.

use crate::error::{DeployError, Result};
use std::process::Command;
use tracing::{info, warn};

/// Network bridge configuration
#[derive(Debug, Clone)]
pub struct BridgeConfig {
    /// Bridge name
    pub name: String,

    /// Bridge IP address (CIDR notation)
    pub ip_address: String,

    /// Subnet (CIDR notation)
    pub subnet: String,
}

/// Network bridge manager
pub struct NetworkBridge {
    config: BridgeConfig,
    created: bool,
}

impl NetworkBridge {
    /// Create a new network bridge manager
    pub fn new(config: BridgeConfig) -> Self {
        Self {
            config,
            created: false,
        }
    }

    /// Check if bridge exists (pure Rust via /sys/class/net/)
    pub fn exists(&self) -> bool {
        std::path::Path::new(&format!("/sys/class/net/{}", self.config.name)).exists()
    }

    /// Create the network bridge
    pub async fn create(&mut self) -> Result<()> {
        if self.exists() {
            info!("Network bridge {} already exists", self.config.name);
            self.created = false; // We didn't create it
            return Ok(());
        }

        info!("Creating network bridge {}...", self.config.name);

        // Create bridge
        let output = Command::new("sudo")
            .args(["ip", "link", "add", &self.config.name, "type", "bridge"])
            .output()
            .map_err(|e| DeployError::NetworkBridge {
                message: format!("Failed to create bridge: {}", e),
            })?;

        if !output.status.success() {
            return Err(DeployError::NetworkBridge {
                message: format!(
                    "Failed to create bridge: {}",
                    String::from_utf8_lossy(&output.stderr)
                ),
            });
        }

        // Set IP address
        let output = Command::new("sudo")
            .args([
                "ip",
                "addr",
                "add",
                &self.config.ip_address,
                "dev",
                &self.config.name,
            ])
            .output()
            .map_err(|e| DeployError::NetworkBridge {
                message: format!("Failed to set IP address: {}", e),
            })?;

        if !output.status.success() {
            return Err(DeployError::NetworkBridge {
                message: format!(
                    "Failed to set IP address: {}",
                    String::from_utf8_lossy(&output.stderr)
                ),
            });
        }

        // Bring up bridge
        let output = Command::new("sudo")
            .args(["ip", "link", "set", &self.config.name, "up"])
            .output()
            .map_err(|e| DeployError::NetworkBridge {
                message: format!("Failed to bring up bridge: {}", e),
            })?;

        if !output.status.success() {
            return Err(DeployError::NetworkBridge {
                message: format!(
                    "Failed to bring up bridge: {}",
                    String::from_utf8_lossy(&output.stderr)
                ),
            });
        }

        info!("✅ Network bridge {} created", self.config.name);
        self.created = true;
        Ok(())
    }

    /// Destroy the network bridge (only if we created it)
    pub async fn destroy(&mut self) -> Result<()> {
        if !self.created {
            info!(
                "Skipping destruction of bridge {} (not created by us)",
                self.config.name
            );
            return Ok(());
        }

        if !self.exists() {
            warn!("Bridge {} already removed", self.config.name);
            return Ok(());
        }

        info!("Destroying network bridge {}...", self.config.name);

        let output = Command::new("sudo")
            .args(["ip", "link", "delete", &self.config.name])
            .output()
            .map_err(|e| DeployError::NetworkBridge {
                message: format!("Failed to destroy bridge: {}", e),
            })?;

        if !output.status.success() {
            return Err(DeployError::NetworkBridge {
                message: format!(
                    "Failed to destroy bridge: {}",
                    String::from_utf8_lossy(&output.stderr)
                ),
            });
        }

        info!("✅ Network bridge {} destroyed", self.config.name);
        self.created = false;
        Ok(())
    }

    /// Get bridge name
    pub fn name(&self) -> &str {
        &self.config.name
    }
}

impl Drop for NetworkBridge {
    fn drop(&mut self) {
        if self.created {
            warn!(
                "NetworkBridge {} dropped without explicit destroy() call",
                self.config.name
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bridge_config_construction() {
        let config = BridgeConfig {
            name: "test-br0".to_string(),
            ip_address: "10.0.0.1/24".to_string(),
            subnet: "10.0.0.0/24".to_string(),
        };
        assert_eq!(config.name, "test-br0");
        assert_eq!(config.ip_address, "10.0.0.1/24");
    }

    #[test]
    fn test_network_bridge_new() {
        let config = BridgeConfig {
            name: "biomeos-test-bridge-xyz".to_string(),
            ip_address: "192.168.100.1/24".to_string(),
            subnet: "192.168.100.0/24".to_string(),
        };
        let bridge = NetworkBridge::new(config);
        assert_eq!(bridge.name(), "biomeos-test-bridge-xyz");
    }

    #[test]
    fn test_network_bridge_exists_nonexistent() {
        let config = BridgeConfig {
            name: "nonexistent-bridge-987654321".to_string(),
            ip_address: "10.0.0.1/24".to_string(),
            subnet: "10.0.0.0/24".to_string(),
        };
        let bridge = NetworkBridge::new(config);
        assert!(!bridge.exists());
    }

    #[test]
    fn test_network_bridge_exists_loopback() {
        let config = BridgeConfig {
            name: "lo".to_string(),
            ip_address: "127.0.0.1/8".to_string(),
            subnet: "127.0.0.0/8".to_string(),
        };
        let bridge = NetworkBridge::new(config);
        assert!(bridge.exists());
    }
}
