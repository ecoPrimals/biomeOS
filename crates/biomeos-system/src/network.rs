// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Network interface information and I/O metrics.

use biomeos_types::{BiomeResult, NetworkIoMetrics};

/// Network interface information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NetworkInterface {
    /// Interface name (e.g. "eth0", "wlan0")
    pub name: String,
    /// Type of network interface
    pub interface_type: NetworkInterfaceType,
    /// Current operational status
    pub status: NetworkInterfaceStatus,
    /// IP addresses bound to this interface
    pub addresses: Vec<String>,
    /// MAC / hardware address
    pub mac_address: Option<String>,
    /// Maximum transmission unit in bytes
    pub mtu: u32,
    /// Cumulative bytes transmitted
    pub bytes_sent: u64,
    /// Cumulative bytes received
    pub bytes_received: u64,
    /// Cumulative packets transmitted
    pub packets_sent: u64,
    /// Cumulative packets received
    pub packets_received: u64,
}

/// Network interface types
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum NetworkInterfaceType {
    /// Wired Ethernet
    Ethernet,
    /// Wi-Fi / wireless
    Wireless,
    /// Loopback (lo)
    Loopback,
    /// Virtual bridge
    Bridge,
    /// VPN / tunnel interface
    Tunnel,
    /// Unknown or other interface type
    Other(String),
}

/// Network interface status
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum NetworkInterfaceStatus {
    /// Interface is up and operational
    Up,
    /// Interface is down
    Down,
    /// Status could not be determined
    Unknown,
}

/// Get network information using sysinfo for cross-platform support
pub(crate) async fn get_network_info() -> BiomeResult<Vec<NetworkInterface>> {
    use sysinfo::Networks;

    let networks = Networks::new_with_refreshed_list();
    let mut result = Vec::new();

    for (interface_name, network) in &networks {
        // Determine interface type based on name
        let interface_type = if interface_name.starts_with("lo") {
            NetworkInterfaceType::Loopback
        } else if interface_name.starts_with("eth") || interface_name.starts_with("enp") {
            NetworkInterfaceType::Ethernet
        } else if interface_name.starts_with("wlan") || interface_name.starts_with("wlp") {
            NetworkInterfaceType::Wireless
        } else if interface_name.starts_with("docker") || interface_name.starts_with("br") {
            NetworkInterfaceType::Bridge
        } else {
            NetworkInterfaceType::Other(interface_name.clone())
        };

        result.push(NetworkInterface {
            name: interface_name.clone(),
            interface_type,
            status: NetworkInterfaceStatus::Up, // sysinfo only shows active interfaces
            addresses: vec![],                  // IP addresses not directly available in sysinfo
            mac_address: Some(format!("{:?}", network.mac_address())),
            mtu: 0, // MTU not available in sysinfo
            bytes_sent: network.total_transmitted(),
            bytes_received: network.total_received(),
            packets_sent: network.total_packets_transmitted(),
            packets_received: network.total_packets_received(),
        });
    }

    // Ensure at least loopback interface for systems where detection fails
    if result.is_empty() {
        result.push(NetworkInterface {
            name: "lo".to_string(),
            interface_type: NetworkInterfaceType::Loopback,
            status: NetworkInterfaceStatus::Up,
            addresses: vec!["127.0.0.1".to_string()],
            mac_address: None,
            mtu: 65536,
            bytes_sent: 0,
            bytes_received: 0,
            packets_sent: 0,
            packets_received: 0,
        });
    }

    Ok(result)
}

/// Get current network I/O using sysinfo
pub(crate) async fn get_network_io() -> BiomeResult<NetworkIoMetrics> {
    use sysinfo::Networks;

    let mut networks = Networks::new_with_refreshed_list();

    // First measurement
    let initial_rx: u64 = networks.values().map(|data| data.total_received()).sum();
    let initial_tx: u64 = networks.values().map(|data| data.total_transmitted()).sum();
    let initial_rx_packets: u64 = networks
        .values()
        .map(|data| data.total_packets_received())
        .sum();
    let initial_tx_packets: u64 = networks
        .values()
        .map(|data| data.total_packets_transmitted())
        .sum();

    // Wait 1 second
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    // Second measurement
    networks.refresh();
    let final_rx: u64 = networks.values().map(|data| data.total_received()).sum();
    let final_tx: u64 = networks.values().map(|data| data.total_transmitted()).sum();
    let final_rx_packets: u64 = networks
        .values()
        .map(|data| data.total_packets_received())
        .sum();
    let final_tx_packets: u64 = networks
        .values()
        .map(|data| data.total_packets_transmitted())
        .sum();

    Ok(NetworkIoMetrics {
        bytes_in_per_sec: (final_rx.saturating_sub(initial_rx)) as f64,
        bytes_out_per_sec: (final_tx.saturating_sub(initial_tx)) as f64,
        packets_in_per_sec: (final_rx_packets.saturating_sub(initial_rx_packets)) as f64,
        packets_out_per_sec: (final_tx_packets.saturating_sub(initial_tx_packets)) as f64,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_network_info() {
        let network_info = get_network_info()
            .await
            .expect("get_network_info should succeed");

        assert!(
            !network_info.is_empty(),
            "should have at least one interface"
        );
        for iface in &network_info {
            assert!(!iface.name.is_empty(), "interface name should not be empty");
        }
    }

    #[test]
    fn test_network_interface_type_serialization() {
        let variants = [
            NetworkInterfaceType::Ethernet,
            NetworkInterfaceType::Wireless,
            NetworkInterfaceType::Loopback,
            NetworkInterfaceType::Bridge,
            NetworkInterfaceType::Tunnel,
            NetworkInterfaceType::Other("veth0".to_string()),
        ];
        for variant in &variants {
            let json = serde_json::to_string(variant).expect("serialization should succeed");
            let deserialized: NetworkInterfaceType =
                serde_json::from_str(&json).expect("deserialization should succeed");
            assert_eq!(
                std::mem::discriminant(variant),
                std::mem::discriminant(&deserialized)
            );
        }
    }

    #[test]
    fn test_network_interface_status_serialization() {
        let variants = [
            NetworkInterfaceStatus::Up,
            NetworkInterfaceStatus::Down,
            NetworkInterfaceStatus::Unknown,
        ];
        for variant in &variants {
            let json = serde_json::to_string(variant).expect("serialization should succeed");
            let deserialized: NetworkInterfaceStatus =
                serde_json::from_str(&json).expect("deserialization should succeed");
            assert_eq!(
                std::mem::discriminant(variant),
                std::mem::discriminant(&deserialized)
            );
        }
    }

    #[test]
    fn test_network_interface_serialization_roundtrip() {
        let info = NetworkInterface {
            name: "wlan0".to_string(),
            interface_type: NetworkInterfaceType::Wireless,
            status: NetworkInterfaceStatus::Up,
            addresses: vec!["192.168.1.100".to_string()],
            mac_address: Some("aa:bb:cc:dd:ee:ff".to_string()),
            mtu: 1500,
            bytes_sent: 1_000_000,
            bytes_received: 2_000_000,
            packets_sent: 5000,
            packets_received: 10000,
        };
        let json = serde_json::to_string(&info).expect("serialization should succeed");
        let deserialized: NetworkInterface =
            serde_json::from_str(&json).expect("deserialization should succeed");
        assert_eq!(info.name, deserialized.name);
        assert_eq!(info.addresses, deserialized.addresses);
        assert_eq!(info.bytes_sent, deserialized.bytes_sent);
    }

    #[test]
    fn test_network_interface_other_type_with_custom_string() {
        let info = NetworkInterface {
            name: "veth12345".to_string(),
            interface_type: NetworkInterfaceType::Other("custom".to_string()),
            status: NetworkInterfaceStatus::Unknown,
            addresses: vec![],
            mac_address: None,
            mtu: 0,
            bytes_sent: 0,
            bytes_received: 0,
            packets_sent: 0,
            packets_received: 0,
        };
        let json = serde_json::to_string(&info).expect("serialization should succeed");
        let deserialized: NetworkInterface =
            serde_json::from_str(&json).expect("deserialization should succeed");
        if let NetworkInterfaceType::Other(s) = &deserialized.interface_type {
            assert_eq!(s, "custom");
        } else {
            panic!("Expected Other variant");
        }
    }

    #[test]
    fn test_clone_network_interface_status() {
        let status = NetworkInterfaceStatus::Up;
        let cloned_status = status.clone();
        assert!(matches!(cloned_status, NetworkInterfaceStatus::Up));
    }
}
