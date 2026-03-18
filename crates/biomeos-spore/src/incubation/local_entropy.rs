// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Local entropy gathering for spore incubation.
//!
//! Collects system-specific information (hostname, machine-id, MAC, etc.)
//! to mix with the spore seed for unique node identity derivation.

use anyhow::Context;
use bytes::Bytes;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tracing::{debug, info, warn};

use crate::error::SporeResult;

/// Local entropy gathered from the computer during incubation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalEntropy {
    /// System hostname
    pub hostname: String,

    /// Machine ID (from /etc/machine-id or generated)
    pub machine_id: String,

    /// Timestamp of incubation
    pub timestamp: DateTime<Utc>,

    /// Primary network interface MAC address
    pub mac_address: Option<String>,

    /// Random nonce for additional entropy
    pub random_nonce: Bytes,

    /// CPU info hash (optional)
    pub cpu_hash: Option<String>,

    /// Disk serial (optional, for additional uniqueness)
    pub disk_serial: Option<String>,
}

impl LocalEntropy {
    /// Generate local entropy from the current computer
    ///
    /// This gathers system-specific information to create unique entropy
    /// that will be mixed with the spore seed.
    pub fn generate(computer_name: Option<&str>) -> SporeResult<Self> {
        info!("Generating local entropy for incubation");

        // 1. Hostname
        let hostname = computer_name
            .map(std::string::ToString::to_string)
            .or_else(|| std::env::var("HOSTNAME").ok())
            .or_else(|| std::env::var("COMPUTERNAME").ok())
            .unwrap_or_else(|| {
                warn!("Could not determine hostname, using default");
                "unknown-host".to_string()
            });

        // 2. Machine ID
        let machine_id = Self::get_machine_id()?;

        // 3. Timestamp
        let timestamp = Utc::now();

        // 4. MAC address (optional, best effort)
        let mac_address = Self::get_primary_mac_address().ok();

        // 5. Random nonce (32 bytes)
        let mut random_nonce = vec![0u8; 32];
        getrandom::getrandom(&mut random_nonce).context("Failed to generate random nonce")?;
        let random_nonce = Bytes::from(random_nonce);

        // 6. CPU hash (optional)
        let cpu_hash = Self::get_cpu_hash().ok();

        // 7. Disk serial (optional)
        let disk_serial = Self::get_disk_serial().ok();

        debug!("Generated entropy for hostname: {}", hostname);

        Ok(Self {
            hostname,
            machine_id,
            timestamp,
            mac_address,
            random_nonce,
            cpu_hash,
            disk_serial,
        })
    }

    /// Calculate SHA256 hash of all entropy components
    pub fn hash(&self) -> String {
        let mut hasher = Sha256::new();

        hasher.update(self.hostname.as_bytes());
        hasher.update(self.machine_id.as_bytes());
        hasher.update(self.timestamp.to_rfc3339().as_bytes());

        if let Some(ref mac) = self.mac_address {
            hasher.update(mac.as_bytes());
        }

        hasher.update(&self.random_nonce[..]);

        if let Some(ref cpu) = self.cpu_hash {
            hasher.update(cpu.as_bytes());
        }

        if let Some(ref disk) = self.disk_serial {
            hasher.update(disk.as_bytes());
        }

        format!("{:x}", hasher.finalize())
    }

    /// Get machine ID from system
    fn get_machine_id() -> SporeResult<String> {
        // Try /etc/machine-id (Linux)
        if let Ok(id) = std::fs::read_to_string("/etc/machine-id") {
            return Ok(id.trim().to_string());
        }

        // Try /var/lib/dbus/machine-id (Linux fallback)
        if let Ok(id) = std::fs::read_to_string("/var/lib/dbus/machine-id") {
            return Ok(id.trim().to_string());
        }

        // Generate a stable UUID based on hostname + user
        warn!("Could not read machine-id, generating stable fallback");
        let hostname = std::env::var("HOSTNAME").unwrap_or_else(|_| "unknown".to_string());
        let user = std::env::var("USER").unwrap_or_else(|_| "unknown".to_string());

        let mut hasher = Sha256::new();
        hasher.update(hostname.as_bytes());
        hasher.update(user.as_bytes());
        hasher.update(b"biomeos-machine-id");

        Ok(format!("{:x}", hasher.finalize()))
    }

    /// Get primary network interface MAC address
    ///
    /// On Linux: reads from `/sys/class/net/<interface>/address` for the first
    /// non-loopback interface. Falls back to placeholder if no interfaces found.
    fn get_primary_mac_address() -> Result<String, anyhow::Error> {
        #[cfg(target_os = "linux")]
        {
            let net_dir = "/sys/class/net";
            if let Ok(entries) = std::fs::read_dir(net_dir) {
                let mut ifaces: Vec<std::path::PathBuf> = entries
                    .filter_map(std::result::Result::ok)
                    .map(|e| e.path())
                    .filter(|p| {
                        let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
                        !name.starts_with("lo") // Skip loopback only
                    })
                    .collect();
                ifaces.sort(); // Deterministic: prefer eth0, enp0s3, etc. over wlan0

                for path in ifaces {
                    let iface_str = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    let mac_path = format!("/sys/class/net/{iface_str}/address");
                    if let Ok(mac) = std::fs::read_to_string(&mac_path) {
                        let mac = mac.trim().to_string();
                        if !mac.is_empty() {
                            return Ok(mac);
                        }
                    }
                }
            }
        }

        // Fallback: no suitable interface found (non-Linux or no interfaces)
        Ok("00:00:00:00:00:00".to_string())
    }

    /// Get CPU info hash
    fn get_cpu_hash() -> Result<String, anyhow::Error> {
        #[cfg(target_os = "linux")]
        {
            if let Ok(cpuinfo) = std::fs::read_to_string("/proc/cpuinfo") {
                let mut hasher = Sha256::new();
                hasher.update(cpuinfo.as_bytes());
                return Ok(format!("{:x}", hasher.finalize()));
            }
        }

        Ok("unknown-cpu".to_string())
    }

    /// Get disk serial
    fn get_disk_serial() -> Result<String, anyhow::Error> {
        // This is platform-specific and best-effort
        // In production, might use a crate or system calls
        Ok("unknown-disk".to_string())
    }
}
