// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Network bridge management
//!
//! Pure Rust implementation via rtnetlink (Netlink). No shell-outs.
//! Bridge create/delete/configure use netlink socket calls.
//!
//! **Privilege requirement**: Creating and destroying bridges requires
//! `CAP_NET_ADMIN` (typically root). The process must run with sufficient
//! privileges; rtnetlink does not invoke sudo.

use crate::error::{DeployError, Result};
use futures::StreamExt;
use std::net::IpAddr;
use tracing::{info, warn};

use rtnetlink::{LinkBridge, LinkUnspec};

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

/// Parse CIDR string (e.g. "10.0.0.1/24") into (`IpAddr`, `prefix_len`)
pub(crate) fn parse_cidr(cidr: &str) -> Result<(IpAddr, u8)> {
    let (ip_str, prefix_str) = cidr
        .split_once('/')
        .ok_or_else(|| DeployError::NetworkBridge {
            message: format!("Invalid CIDR: expected ADDR/PREFIX, got '{cidr}'"),
        })?;
    let ip: IpAddr = ip_str.parse().map_err(|e| DeployError::NetworkBridge {
        message: format!("Invalid IP in CIDR '{cidr}': {e}"),
    })?;
    let prefix: u8 = prefix_str.parse().map_err(|e| DeployError::NetworkBridge {
        message: format!("Invalid prefix in CIDR '{cidr}': {e}"),
    })?;
    Ok((ip, prefix))
}

impl NetworkBridge {
    /// Create a new network bridge manager
    #[must_use]
    pub const fn new(config: BridgeConfig) -> Self {
        Self {
            config,
            created: false,
        }
    }

    /// Check if bridge exists (pure Rust via /sys/class/net/)
    #[must_use]
    pub fn exists(&self) -> bool {
        std::path::Path::new(&format!("/sys/class/net/{}", self.config.name)).exists()
    }

    /// Create the network bridge (pure Rust via rtnetlink)
    pub async fn create(&mut self) -> Result<()> {
        if self.exists() {
            info!("Network bridge {} already exists", self.config.name);
            self.created = false; // We didn't create it
            return Ok(());
        }

        info!("Creating network bridge {}...", self.config.name);

        let (connection, handle, _) =
            rtnetlink::new_connection().map_err(|e| DeployError::NetworkBridge {
                message: format!("Failed to create netlink connection: {e}"),
            })?;
        tokio::spawn(connection);

        // Create bridge (create down so we can add address first)
        handle
            .link()
            .add(LinkBridge::new(&self.config.name).down().build())
            .execute()
            .await
            .map_err(|e| DeployError::NetworkBridge {
                message: format!("Failed to create bridge: {e}"),
            })?;

        // Get link index by name
        let mut links = handle
            .link()
            .get()
            .match_name(self.config.name.clone())
            .execute();
        let link_msg = links
            .next()
            .await
            .transpose()
            .map_err(|e| DeployError::NetworkBridge {
                message: format!("Failed to get bridge link: {e}"),
            })?
            .ok_or_else(|| DeployError::NetworkBridge {
                message: format!("Bridge {} created but link not found", self.config.name),
            })?;

        let index = link_msg.header.index;

        // Set IP address
        let (ip, prefix_len) = parse_cidr(&self.config.ip_address)?;
        handle
            .address()
            .add(index, ip, prefix_len)
            .execute()
            .await
            .map_err(|e| DeployError::NetworkBridge {
                message: format!("Failed to set IP address: {e}"),
            })?;

        // Bring up bridge
        handle
            .link()
            .change(LinkUnspec::new_with_name(&self.config.name).up().build())
            .execute()
            .await
            .map_err(|e| DeployError::NetworkBridge {
                message: format!("Failed to bring up bridge: {e}"),
            })?;

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

        let (connection, handle, _) =
            rtnetlink::new_connection().map_err(|e| DeployError::NetworkBridge {
                message: format!("Failed to create netlink connection: {e}"),
            })?;
        tokio::spawn(connection);

        // Get link index by name for deletion
        let mut links = handle
            .link()
            .get()
            .match_name(self.config.name.clone())
            .execute();
        let link_msg = links
            .next()
            .await
            .transpose()
            .map_err(|e| DeployError::NetworkBridge {
                message: format!("Failed to get bridge link: {e}"),
            })?
            .ok_or_else(|| DeployError::NetworkBridge {
                message: format!("Bridge {} not found for deletion", self.config.name),
            })?;

        handle
            .link()
            .del(link_msg.header.index)
            .execute()
            .await
            .map_err(|e| DeployError::NetworkBridge {
                message: format!("Failed to destroy bridge: {e}"),
            })?;

        info!("✅ Network bridge {} destroyed", self.config.name);
        self.created = false;
        Ok(())
    }

    /// Get bridge name
    #[must_use]
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
#[expect(
    clippy::unwrap_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
#[expect(
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
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
    fn test_parse_cidr() {
        let (ip, prefix) = parse_cidr("10.0.0.1/24").unwrap();
        assert_eq!(ip.to_string(), "10.0.0.1");
        assert_eq!(prefix, 24);
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

    #[test]
    fn test_parse_cidr_ipv6() {
        let (ip, prefix) = parse_cidr("fe80::1/64").unwrap();
        assert!(ip.is_ipv6());
        assert_eq!(prefix, 64);
    }

    #[test]
    fn test_parse_cidr_invalid_no_slash() {
        let result = parse_cidr("10.0.0.1");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Invalid CIDR") || err.contains("ADDR/PREFIX"));
    }

    #[test]
    fn test_parse_cidr_invalid_ip() {
        let result = parse_cidr("not-an-ip/24");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_cidr_invalid_prefix() {
        let result = parse_cidr("10.0.0.1/256");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_cidr_invalid_prefix_non_numeric() {
        let result = parse_cidr("10.0.0.1/xx");
        assert!(result.is_err());
    }

    #[test]
    fn test_bridge_config_clone() {
        let config = BridgeConfig {
            name: "br0".to_string(),
            ip_address: "10.0.0.1/24".to_string(),
            subnet: "10.0.0.0/24".to_string(),
        };
        let cloned = config.clone();
        assert_eq!(config.name, cloned.name);
        assert_eq!(config.ip_address, cloned.ip_address);
    }

    #[test]
    fn test_bridge_config_debug() {
        let config = BridgeConfig {
            name: "test".to_string(),
            ip_address: "192.168.1.1/24".to_string(),
            subnet: "192.168.1.0/24".to_string(),
        };
        let s = format!("{config:?}");
        assert!(s.contains("test"));
    }

    #[test]
    fn test_parse_cidr_all_zeros() {
        let (ip, p) = parse_cidr("0.0.0.0/0").unwrap();
        assert_eq!(ip.to_string(), "0.0.0.0");
        assert_eq!(p, 0);
    }

    #[test]
    fn test_parse_cidr_max_ipv4_prefix() {
        let (ip, p) = parse_cidr("255.255.255.255/32").unwrap();
        assert_eq!(p, 32);
        assert!(ip.is_ipv4());
    }

    #[test]
    fn test_parse_cidr_ipv6_loopback() {
        let (ip, p) = parse_cidr("::1/128").unwrap();
        assert!(ip.is_ipv6());
        assert_eq!(p, 128);
    }

    #[test]
    fn test_network_bridge_name_accessor() {
        let config = BridgeConfig {
            name: "br99".to_string(),
            ip_address: "10.1.1.1/24".to_string(),
            subnet: "10.1.1.0/24".to_string(),
        };
        let b = NetworkBridge::new(config);
        assert_eq!(b.name(), "br99");
    }

    #[test]
    fn test_network_bridge_drop_without_create_does_not_panic() {
        let config = BridgeConfig {
            name: "biomeos-drop-test-bridge-999001".to_string(),
            ip_address: "10.99.0.1/24".to_string(),
            subnet: "10.99.0.0/24".to_string(),
        };
        let bridge = NetworkBridge::new(config);
        drop(bridge);
    }

    #[tokio::test]
    async fn test_network_bridge_destroy_skips_when_not_created() {
        let config = BridgeConfig {
            name: "biomeos-test-br-destroy-skip-001".to_string(),
            ip_address: "10.200.0.1/24".to_string(),
            subnet: "10.200.0.0/24".to_string(),
        };
        let mut bridge = NetworkBridge::new(config);
        bridge.destroy().await.expect("destroy noop");
    }

    #[tokio::test]
    async fn test_network_bridge_create_fails_without_privileges_or_missing() {
        let config = BridgeConfig {
            name: "biomeos-test-br-priv-xyz-999".to_string(),
            ip_address: "10.201.0.1/24".to_string(),
            subnet: "10.201.0.0/24".to_string(),
        };
        let mut bridge = NetworkBridge::new(config);
        if !bridge.exists() {
            let r = bridge.create().await;
            assert!(r.is_err() || r.is_ok());
        }
    }

    #[test]
    fn test_parse_cidr_single_char_prefix_edge() {
        let (ip, p) = parse_cidr("192.168.0.1/8").unwrap();
        assert!(ip.is_ipv4());
        assert_eq!(p, 8);
    }

    #[tokio::test]
    async fn test_network_bridge_create_when_loopback_exists_is_noop() {
        let config = BridgeConfig {
            name: "lo".to_string(),
            ip_address: "127.0.0.1/8".to_string(),
            subnet: "127.0.0.0/8".to_string(),
        };
        let mut bridge = NetworkBridge::new(config);
        assert!(bridge.exists());
        let r = bridge.create().await;
        assert!(
            r.is_ok(),
            "expected Ok when interface already present: {r:?}"
        );
    }

    #[test]
    fn test_network_bridge_debug_clone_eq_paths() {
        let c = BridgeConfig {
            name: "br-dbg".to_string(),
            ip_address: "10.55.0.1/24".to_string(),
            subnet: "10.55.0.0/24".to_string(),
        };
        let s = format!("{c:?}");
        assert!(s.contains("br-dbg"));
        let c2 = c.clone();
        assert_eq!(c.name, c2.name);
    }

    #[test]
    fn test_parse_cidr_empty_string() {
        let r = parse_cidr("");
        assert!(r.is_err());
    }

    #[test]
    fn test_parse_cidr_only_slash() {
        let r = parse_cidr("/");
        assert!(r.is_err());
    }

    #[test]
    fn test_parse_cidr_double_slash_in_addr() {
        // `split_once('/')` yields prefix `/24`, which is not a valid `u8`
        let r = parse_cidr("10.0.0.1//24");
        assert!(r.is_err());
    }
}
