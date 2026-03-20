// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Network interface information and I/O metrics (pure Rust via /proc/net/dev - ecoBin v3).

use std::fs;

use biomeos_types::{BiomeResult, NetworkIoMetrics};

/// Parsed line from /proc/net/dev: interface name and rx/tx bytes + packets
fn parse_net_dev(content: &str) -> Vec<(String, u64, u64, u64, u64)> {
    let mut out = Vec::new();
    for line in content.lines().skip(2) {
        // Format: "  eth0: bytes packets ..."
        let Some(colon) = line.find(':') else {
            continue;
        };
        let name = line[..colon].trim().to_string();
        let rest = line[colon + 1..].trim();
        let nums: Vec<u64> = rest
            .split_whitespace()
            .filter_map(|s| s.parse().ok())
            .collect();
        // Receive: bytes, packets, ... Transmit: bytes, packets, ...
        if nums.len() >= 8 {
            let rx_bytes = nums[0];
            let rx_packets = nums[1];
            let tx_bytes = nums[8];
            let tx_packets = nums[9];
            out.push((name, rx_bytes, rx_packets, tx_bytes, tx_packets));
        }
    }
    out
}

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

/// Get network information via /proc/net/dev + /sys/class/net (pure Rust).
#[cfg(target_os = "linux")]
pub(crate) async fn get_network_info() -> BiomeResult<Vec<NetworkInterface>> {
    let content = fs::read_to_string("/proc/net/dev").unwrap_or_default();
    let parsed = parse_net_dev(&content);
    let mut result = Vec::new();

    for (name, rx_bytes, rx_packets, tx_bytes, tx_packets) in parsed {
        let interface_type = if name.starts_with("lo") {
            NetworkInterfaceType::Loopback
        } else if name.starts_with("eth") || name.starts_with("enp") {
            NetworkInterfaceType::Ethernet
        } else if name.starts_with("wlan") || name.starts_with("wlp") {
            NetworkInterfaceType::Wireless
        } else if name.starts_with("docker") || name.starts_with("br") {
            NetworkInterfaceType::Bridge
        } else {
            NetworkInterfaceType::Other(name.clone())
        };

        let status = fs::read_to_string(format!("/sys/class/net/{name}/operstate"))
            .ok()
            .map_or(NetworkInterfaceStatus::Unknown, |s| match s.trim() {
                "up" => NetworkInterfaceStatus::Up,
                "down" => NetworkInterfaceStatus::Down,
                _ => NetworkInterfaceStatus::Unknown,
            });

        let mac_address = fs::read_to_string(format!("/sys/class/net/{name}/address"))
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());

        let mtu = fs::read_to_string(format!("/sys/class/net/{name}/mtu"))
            .ok()
            .and_then(|s| s.trim().parse().ok())
            .unwrap_or(0);

        result.push(NetworkInterface {
            name,
            interface_type,
            status,
            addresses: vec![], // Would require /proc/net or ip parsing
            mac_address,
            mtu,
            bytes_sent: tx_bytes,
            bytes_received: rx_bytes,
            packets_sent: tx_packets,
            packets_received: rx_packets,
        });
    }

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

/// Non-Linux fallback.
#[cfg(not(target_os = "linux"))]
pub(crate) async fn get_network_info() -> BiomeResult<Vec<NetworkInterface>> {
    Ok(vec![NetworkInterface {
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
    }])
}

/// Default sample interval for network I/O (1s).
const DEFAULT_NETWORK_SAMPLE_INTERVAL: std::time::Duration = std::time::Duration::from_secs(1);

/// Get current network I/O via /proc/net/dev (pure Rust).
#[cfg(target_os = "linux")]
pub(crate) async fn get_network_io() -> BiomeResult<NetworkIoMetrics> {
    get_network_io_with_interval(DEFAULT_NETWORK_SAMPLE_INTERVAL).await
}

/// Get current network I/O with configurable sample interval.
#[cfg(target_os = "linux")]
pub(crate) async fn get_network_io_with_interval(
    sample_interval: std::time::Duration,
) -> BiomeResult<NetworkIoMetrics> {
    let content1 = fs::read_to_string("/proc/net/dev").unwrap_or_default();
    let parsed1 = parse_net_dev(&content1);
    let (init_rx, init_tx, init_rxp, init_txp) = parsed1.iter().fold(
        (0u64, 0u64, 0u64, 0u64),
        |(rx, tx, rxp, txp), (_, a, b, c, d)| (rx + a, tx + c, rxp + b, txp + d),
    );

    tokio::time::sleep(sample_interval).await;

    let content2 = fs::read_to_string("/proc/net/dev").unwrap_or_default();
    let parsed2 = parse_net_dev(&content2);
    let (final_rx, final_tx, final_rxp, final_txp) = parsed2.iter().fold(
        (0u64, 0u64, 0u64, 0u64),
        |(rx, tx, rxp, txp), (_, a, b, c, d)| (rx + a, tx + c, rxp + b, txp + d),
    );

    // u64->f64: precision loss acceptable for I/O metrics
    Ok(NetworkIoMetrics {
        bytes_in_per_sec: (final_rx.saturating_sub(init_rx)) as f64,
        bytes_out_per_sec: (final_tx.saturating_sub(init_tx)) as f64,
        packets_in_per_sec: (final_rxp.saturating_sub(init_rxp)) as f64,
        packets_out_per_sec: (final_txp.saturating_sub(init_txp)) as f64,
    })
}

/// Non-Linux fallback.
#[cfg(not(target_os = "linux"))]
pub(crate) async fn get_network_io() -> BiomeResult<NetworkIoMetrics> {
    Ok(NetworkIoMetrics {
        bytes_in_per_sec: 0.0,
        bytes_out_per_sec: 0.0,
        packets_in_per_sec: 0.0,
        packets_out_per_sec: 0.0,
    })
}

#[expect(
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
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
        let cloned_status = status;
        assert!(matches!(cloned_status, NetworkInterfaceStatus::Up));
    }
}
