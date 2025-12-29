//! mDNS/UDP discovery validation
//!
//! Validate that VMs discover each other via mDNS.

use crate::deployment::DeployedVm;
use anyhow::{Context, Result};

/// mDNS validation manager
#[derive(Debug)]
pub struct MdnsValidator {
    pub expected_peers: usize,
}

impl MdnsValidator {
    /// Create a new mDNS validator
    #[must_use]
    pub fn new(expected_peers: usize) -> Self {
        Self { expected_peers }
    }

    /// Validate mDNS discovery on a VM
    pub async fn validate(&self, vm: &DeployedVm) -> Result<ValidationResult> {
        println!("  🔍 Validating mDNS on {}...", vm.name);

        // Check if avahi is installed
        let avahi_check = vm
            .ssh_exec("command -v avahi-browse > /dev/null && echo 'installed'")
            .unwrap_or_default();

        if !avahi_check.contains("installed") {
            println!("    ⚠️  avahi-daemon not installed, skipping mDNS validation");
            return Ok(ValidationResult {
                discovered_services: Vec::new(),
                peer_count: 0,
                validation_skipped: true,
            });
        }

        // Query mDNS services (with timeout)
        let output = vm
            .ssh_exec("timeout 5 avahi-browse -a -t -p 2>/dev/null || echo ''")
            .context("Failed to query mDNS services")?;

        let services = self.parse_avahi_output(&output);

        println!("    Found {} services", services.len());
        for service in &services {
            println!("      • {} ({})", service.name, service.service_type);
        }

        // Count unique peers
        let peer_count = services
            .iter()
            .filter(|s| s.hostname != vm.name)
            .map(|s| &s.hostname)
            .collect::<std::collections::HashSet<_>>()
            .len();

        println!("    Discovered {} peers", peer_count);

        Ok(ValidationResult {
            discovered_services: services,
            peer_count,
            validation_skipped: false,
        })
    }

    /// Parse avahi-browse output
    fn parse_avahi_output(&self, output: &str) -> Vec<DiscoveredService> {
        let mut services = Vec::new();

        for line in output.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // avahi-browse -p format:
            // +;eth0;IPv4;servicename;_service._tcp;local
            let parts: Vec<&str> = line.split(';').collect();
            if parts.len() >= 5 && parts[0] == "+" {
                services.push(DiscoveredService {
                    interface: parts[1].to_string(),
                    protocol: parts[2].to_string(),
                    name: parts[3].to_string(),
                    service_type: parts[4].to_string(),
                    hostname: parts.get(6).unwrap_or(&"unknown").to_string(),
                });
            }
        }

        services
    }

    /// Wait for mDNS discovery (with retries)
    pub async fn wait_for_discovery(
        &self,
        vm: &DeployedVm,
        timeout_secs: u64,
    ) -> Result<ValidationResult> {
        println!("  ⏳ Waiting for mDNS discovery (timeout: {}s)...", timeout_secs);

        let start = std::time::Instant::now();
        let mut last_result = None;

        while start.elapsed().as_secs() < timeout_secs {
            let result = self.validate(vm).await?;

            if result.validation_skipped {
                return Ok(result);
            }

            if result.peer_count >= self.expected_peers {
                println!("    ✅ Discovery complete! ({} peers)", result.peer_count);
                return Ok(result);
            }

            last_result = Some(result);
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        }

        let result = last_result.unwrap_or(ValidationResult {
            discovered_services: Vec::new(),
            peer_count: 0,
            validation_skipped: false,
        });

        println!(
            "    ⚠️  Timeout: expected {} peers, found {}",
            self.expected_peers, result.peer_count
        );

        Ok(result)
    }
}

/// Result of mDNS validation
#[derive(Debug)]
pub struct ValidationResult {
    pub discovered_services: Vec<DiscoveredService>,
    pub peer_count: usize,
    pub validation_skipped: bool,
}

/// A discovered mDNS service
#[derive(Debug, Clone)]
pub struct DiscoveredService {
    pub interface: String,
    pub protocol: String,
    pub name: String,
    pub service_type: String,
    pub hostname: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_avahi_output() {
        let validator = MdnsValidator::new(1);
        let output = "+;eth0;IPv4;test-service;_http._tcp;local;host1.local;192.168.1.1;80";

        let services = validator.parse_avahi_output(output);
        assert_eq!(services.len(), 1);
        assert_eq!(services[0].name, "test-service");
        assert_eq!(services[0].service_type, "_http._tcp");
    }

    #[test]
    fn test_validation_result() {
        let result = ValidationResult {
            discovered_services: Vec::new(),
            peer_count: 2,
            validation_skipped: false,
        };

        assert_eq!(result.peer_count, 2);
        assert!(!result.validation_skipped);
    }
}

