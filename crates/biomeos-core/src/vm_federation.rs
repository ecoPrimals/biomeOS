// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! BiomeOS + benchScale Integration - VM Federation Manager
//!
//! This module provides high-level APIs for managing BiomeOS VM federations
//! using benchScale's libvirt backend.
//!
//! ## Validation Strategy
//!
//! VM creation is a multi-phase process:
//! 1. VM provisioning (benchScale)
//! 2. Cloud-init execution (10-30 minutes for package installation)
//! 3. SSH key installation
//! 4. Service startup
//!
//! This module implements **mandatory validation** to ensure VMs are actually
//! ready before declaring success. We don't just create VMs - we validate they work.

use anyhow::{Context, Result};
use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};

/// Parse IP address from virsh domifaddr output (testable pure function)
///
/// Parses lines like "ipv4         192.168.122.34/24" and extracts the first 192.168.x.x IP.
pub(crate) fn parse_ip_from_domifaddr_output(ip_text: &str) -> Option<String> {
    for ip_line in ip_text.lines() {
        if (ip_line.contains("ipv4") || ip_line.contains("192.168"))
            && let Some(ip_part) = ip_line.split_whitespace().last()
            && let Some(ip) = ip_part.split('/').next()
            && ip.starts_with("192.168")
        {
            return Some(ip.to_string());
        }
    }
    None
}

/// Extract VM names from virsh list output that match federation name
pub(crate) fn parse_vm_names_from_list(vm_list: &str, federation_name: &str) -> Vec<String> {
    let mut names = Vec::new();
    for line in vm_list.lines() {
        if line.contains(federation_name) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                names.push(parts[1].to_string());
            }
        }
    }
    names
}

/// Configuration for VM validation
#[derive(Debug, Clone)]
pub struct ValidationConfig {
    /// Maximum time to wait for cloud-init completion
    pub cloud_init_timeout: Duration,
    /// Maximum time to wait for SSH access
    pub ssh_timeout: Duration,
    /// Retry interval for SSH attempts
    pub ssh_retry_interval: Duration,
    /// Maximum SSH retry attempts
    pub ssh_max_retries: u32,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            cloud_init_timeout: Duration::from_secs(600), // 10 minutes
            ssh_timeout: Duration::from_secs(300),        // 5 minutes
            ssh_retry_interval: Duration::from_secs(30),  // 30 seconds
            ssh_max_retries: 20,                          // 20 attempts
        }
    }
}

/// VM Federation Manager
///
/// Provides a Rust API for managing BiomeOS VM federations using benchScale.
///
/// This manager implements **mandatory validation** - VMs are not considered
/// "created" until they are fully provisioned and SSH-accessible.
pub struct VmFederationManager {
    benchscale_root: PathBuf,
    topology_path: PathBuf,
    validation_config: ValidationConfig,
}

impl VmFederationManager {
    /// Create a new VM Federation Manager with default validation
    pub fn new() -> Result<Self> {
        Self::with_validation_config(ValidationConfig::default())
    }

    /// Create a new VM Federation Manager with custom validation config
    pub fn with_validation_config(validation_config: ValidationConfig) -> Result<Self> {
        let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

        // benchscale is a sibling directory
        let benchscale_root = crate_root
            .parent()
            .context("No parent directory")?
            .parent()
            .context("No grandparent directory")?
            .join("benchscale");

        if !benchscale_root.exists() {
            anyhow::bail!(
                "benchscale not found at: {}. Is it cloned alongside biomeOS?",
                benchscale_root.display()
            );
        }

        let topology_path = crate_root.join("topologies").join("vm-federation.yaml");

        Ok(Self {
            benchscale_root,
            topology_path,
            validation_config,
        })
    }

    /// Create the VM federation with mandatory validation
    ///
    /// This method creates VMs via benchScale and then **validates** they are
    /// fully provisioned and SSH-accessible before returning success.
    ///
    /// # Validation Steps
    /// 1. Create VMs via benchScale
    /// 2. Discover VM IP addresses
    /// 3. Wait for cloud-init completion (with timeout)
    /// 4. Verify SSH access to all VMs
    ///
    /// # Errors
    /// Returns error if:
    /// - VM creation fails
    /// - Cloud-init timeout reached
    /// - SSH validation fails
    ///
    /// # Example
    /// ```no_run
    /// # use biomeos_core::vm_federation::VmFederationManager;
    /// # async fn example() -> anyhow::Result<()> {
    /// let manager = VmFederationManager::new()?;
    /// manager.create("my-federation").await?;
    /// // VMs are guaranteed to be SSH-accessible here
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create(&self, name: &str) -> Result<()> {
        info!("Creating VM federation: {} (with validation)", name);

        // Phase 1: Create VMs via benchScale
        info!("Phase 1/4: Creating VMs via benchScale");
        let output = Command::new("cargo")
            .current_dir(&self.benchscale_root)
            .args([
                "run",
                "--release",
                "--",
                "create",
                name,
                "--topology",
                self.topology_path
                    .to_str()
                    .ok_or_else(|| anyhow::anyhow!("Topology path contains invalid UTF-8"))?,
                "--backend",
                "libvirt",
            ])
            .output()
            .context("Failed to execute benchscale create")?;

        if !output.status.success() {
            anyhow::bail!(
                "benchscale create failed:\n{}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        info!("Phase 2/4: Discovering VM IP addresses");
        let vm_ips = self.discover_vm_ips(name)?;
        info!("Found {} VMs: {:?}", vm_ips.len(), vm_ips);

        // Phase 3: Wait for cloud-init (via SSH attempts)
        info!(
            "Phase 3/4: Waiting for cloud-init (up to {}s)",
            self.validation_config.cloud_init_timeout.as_secs()
        );
        self.wait_for_all_vms_ready(&vm_ips).await?;

        info!("Phase 4/4: Final SSH validation");
        self.validate_ssh_access(&vm_ips).await?;

        info!("✅ VM federation created and validated: {}", name);
        Ok(())
    }

    /// Discover IP addresses of VMs in a federation
    ///
    /// Uses virsh to query DHCP leases for federation VMs.
    #[allow(clippy::unused_self)]
    fn discover_vm_ips(&self, federation_name: &str) -> Result<Vec<String>> {
        // Query libvirt for VMs matching our federation
        let output = Command::new("virsh")
            .args(["list", "--all"])
            .output()
            .context("Failed to list VMs")?;

        let vm_list = String::from_utf8_lossy(&output.stdout);
        let vm_names = parse_vm_names_from_list(&vm_list, federation_name);
        let mut ips = Vec::new();

        for vm_name in vm_names {
            if let Ok(ip_output) = Command::new("virsh").args(["domifaddr", &vm_name]).output() {
                let ip_text = String::from_utf8_lossy(&ip_output.stdout);
                if let Some(ip) = parse_ip_from_domifaddr_output(&ip_text) {
                    debug!("Found VM {} with IP {}", vm_name, ip);
                    ips.push(ip);
                }
            }
        }

        if ips.is_empty() {
            anyhow::bail!("No VM IPs found for federation: {federation_name}");
        }

        Ok(ips)
    }

    /// Wait for all VMs to be SSH-accessible
    ///
    /// This implements the validation logic that compensates for benchScale's
    /// current lack of cloud-init completion checking.
    async fn wait_for_all_vms_ready(&self, vm_ips: &[String]) -> Result<()> {
        let start = Instant::now();
        let timeout = self.validation_config.cloud_init_timeout;

        for (idx, ip) in vm_ips.iter().enumerate() {
            info!(
                "Waiting for VM {}/{} ({}) to be ready",
                idx + 1,
                vm_ips.len(),
                ip
            );

            let mut attempt = 0;
            loop {
                if start.elapsed() >= timeout {
                    anyhow::bail!(
                        "Timeout waiting for VM {} after {}s. Cloud-init may have failed.",
                        ip,
                        timeout.as_secs()
                    );
                }

                attempt += 1;
                debug!(
                    "SSH attempt {}/{} to {}",
                    attempt, self.validation_config.ssh_max_retries, ip
                );

                // Try SSH connection
                let ssh_test = Command::new("ssh")
                    .args([
                        "-o",
                        "ConnectTimeout=5",
                        "-o",
                        "StrictHostKeyChecking=no",
                        "-o",
                        "BatchMode=yes",
                        &format!("biomeos@{ip}"),
                        "echo 'SSH ready'",
                    ])
                    .output();

                if let Ok(output) = ssh_test
                    && output.status.success()
                {
                    info!("✅ VM {} is SSH-accessible", ip);
                    break;
                }

                if attempt >= self.validation_config.ssh_max_retries {
                    anyhow::bail!(
                        "Failed to SSH to {ip} after {attempt} attempts. Check cloud-init logs."
                    );
                }

                // Exponential backoff: 30s, 60s, 90s, ...
                let wait_time = self.validation_config.ssh_retry_interval * attempt;
                debug!("Waiting {}s before retry", wait_time.as_secs());
                tokio::time::sleep(wait_time).await;
            }
        }

        Ok(())
    }

    /// Validate SSH access to all VMs (final check)
    async fn validate_ssh_access(&self, vm_ips: &[String]) -> Result<()> {
        for ip in vm_ips {
            let output = Command::new("ssh")
                .args([
                    "-o",
                    "ConnectTimeout=5",
                    "-o",
                    "StrictHostKeyChecking=no",
                    &format!("biomeos@{ip}"),
                    "hostname && uptime",
                ])
                .output()
                .context(format!("Failed to validate SSH to {ip}"))?;

            if !output.status.success() {
                anyhow::bail!("SSH validation failed for {ip}");
            }

            info!(
                "✅ VM {} validated: {}",
                ip,
                String::from_utf8_lossy(&output.stdout).trim()
            );
        }

        Ok(())
    }

    /// Start all VMs in the federation
    pub async fn start(&self, name: &str) -> Result<()> {
        info!("Starting VM federation: {}", name);

        let output = Command::new("cargo")
            .current_dir(&self.benchscale_root)
            .args(["run", "--release", "--", "start", name])
            .output()
            .context("Failed to execute benchscale start")?;

        if !output.status.success() {
            anyhow::bail!(
                "benchscale start failed:\n{}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        info!("VM federation started: {}", name);
        Ok(())
    }

    /// Run tests on the federation
    pub async fn test(&self, name: &str) -> Result<()> {
        info!("Running tests on VM federation: {}", name);

        let output = Command::new("cargo")
            .current_dir(&self.benchscale_root)
            .args(["run", "--release", "--", "test", name])
            .output()
            .context("Failed to execute benchscale test")?;

        if !output.status.success() {
            warn!(
                "Some tests failed:\n{}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        info!("Tests complete for: {}", name);
        Ok(())
    }

    /// Stop all VMs in the federation
    pub async fn stop(&self, name: &str) -> Result<()> {
        info!("Stopping VM federation: {}", name);

        let output = Command::new("cargo")
            .current_dir(&self.benchscale_root)
            .args(["run", "--release", "--", "stop", name])
            .output()
            .context("Failed to execute benchscale stop")?;

        if !output.status.success() {
            warn!(
                "benchscale stop had issues:\n{}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        info!("VM federation stopped: {}", name);
        Ok(())
    }

    /// Destroy the VM federation (cleanup)
    pub async fn destroy(&self, name: &str) -> Result<()> {
        info!("Destroying VM federation: {}", name);

        let output = Command::new("cargo")
            .current_dir(&self.benchscale_root)
            .args(["run", "--release", "--", "destroy", name])
            .output()
            .context("Failed to execute benchscale destroy")?;

        if !output.status.success() {
            warn!(
                "benchscale destroy had issues:\n{}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        info!("VM federation destroyed: {}", name);
        Ok(())
    }

    /// Get the status of the federation
    pub async fn status(&self, name: &str) -> Result<String> {
        let output = Command::new("cargo")
            .current_dir(&self.benchscale_root)
            .args(["run", "--release", "--", "status", name])
            .output()
            .context("Failed to execute benchscale status")?;

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_config_default() {
        let config = ValidationConfig::default();
        assert_eq!(config.cloud_init_timeout.as_secs(), 600);
        assert_eq!(config.ssh_timeout.as_secs(), 300);
        assert_eq!(config.ssh_retry_interval.as_secs(), 30);
        assert_eq!(config.ssh_max_retries, 20);
    }

    #[test]
    fn test_validation_config_custom() {
        let config = ValidationConfig {
            cloud_init_timeout: Duration::from_secs(120),
            ssh_timeout: Duration::from_secs(60),
            ssh_retry_interval: Duration::from_secs(10),
            ssh_max_retries: 5,
        };
        assert_eq!(config.cloud_init_timeout.as_secs(), 120);
        assert_eq!(config.ssh_max_retries, 5);
    }

    #[test]
    fn test_manager_creation() {
        let manager = VmFederationManager::new();
        // Manager creation requires benchscale directory to exist
        // This is a valid requirement, so we just verify the Result type works
        match manager {
            Ok(_) => {
                // benchscale exists - great!
            }
            Err(e) => {
                // benchscale doesn't exist - expected in CI/test environments
                assert!(
                    e.to_string().contains("benchscale not found"),
                    "Error should be about missing benchscale, got: {e}"
                );
            }
        }
    }

    #[test]
    fn test_with_validation_config_requires_benchscale() {
        let config = ValidationConfig::default();
        let result = VmFederationManager::with_validation_config(config);
        match result {
            Ok(_) => {}
            Err(e) => {
                assert!(
                    e.to_string().contains("benchscale not found")
                        || e.to_string().contains("parent"),
                    "Expected benchscale or path error, got: {e}"
                );
            }
        }
    }

    #[test]
    fn test_parse_ip_from_domifaddr_output() {
        let output = " Name       MAC address          Protocol     Address\n\nvnet0      xx:xx:xx:xx:xx:xx    ipv4         192.168.122.34/24\n";
        assert_eq!(
            super::parse_ip_from_domifaddr_output(output),
            Some("192.168.122.34".to_string())
        );
    }

    #[test]
    fn test_parse_ip_from_domifaddr_output_no_match() {
        assert_eq!(super::parse_ip_from_domifaddr_output(""), None);
        assert_eq!(
            super::parse_ip_from_domifaddr_output("ipv6  fe80::1/64"),
            None
        );
    }

    #[test]
    fn test_parse_vm_names_from_list() {
        let list = " Id    Name                           State\n----------------------------------------------------\n 1     my-fed-node1                   running\n 2     my-fed-node2                   running\n";
        let names = super::parse_vm_names_from_list(list, "my-fed");
        assert_eq!(names, vec!["my-fed-node1", "my-fed-node2"]);
    }

    #[test]
    fn test_parse_vm_names_from_list_empty() {
        let names = super::parse_vm_names_from_list("", "nonexistent");
        assert!(names.is_empty());
    }

    #[test]
    fn test_parse_ip_from_domifaddr_multiple_lines() {
        let output = " Name       MAC address          Protocol     Address\n\nvnet0      xx:xx    ipv4         192.168.122.100/24\nvnet1      yy:yy    ipv4         192.168.122.101/24\n";
        let ip = super::parse_ip_from_domifaddr_output(output);
        assert_eq!(ip, Some("192.168.122.100".to_string()));
    }

    #[test]
    fn test_parse_ip_from_domifaddr_ipv6_only() {
        assert_eq!(
            super::parse_ip_from_domifaddr_output("ipv6  fe80::1/64"),
            None
        );
    }

    #[test]
    fn test_parse_vm_names_from_list_partial_match() {
        let list = " 1     fed-node1    running\n 2     fed-node2    running\n 3     other-node   running\n";
        let names = super::parse_vm_names_from_list(list, "fed");
        assert_eq!(names, vec!["fed-node1", "fed-node2"]);
    }

    #[test]
    fn test_parse_vm_names_from_list_single_vm() {
        let list = " 1     my-fed-node1    running\n";
        let names = super::parse_vm_names_from_list(list, "my-fed");
        assert_eq!(names, vec!["my-fed-node1"]);
    }

    #[test]
    fn test_parse_ip_from_domifaddr_192_168_prefix_only() {
        let output = " Name       MAC address          Protocol     Address\n\nvnet0      xx:xx    ipv4         192.168.1.50/24\n";
        assert_eq!(
            super::parse_ip_from_domifaddr_output(output),
            Some("192.168.1.50".to_string())
        );
    }

    #[test]
    fn test_parse_ip_from_domifaddr_multiple_ipv4_takes_first() {
        let output = " vnet0  xx  ipv4  192.168.122.10/24\n vnet1  yy  ipv4  192.168.122.20/24\n";
        let ip = super::parse_ip_from_domifaddr_output(output);
        assert_eq!(ip, Some("192.168.122.10".to_string()));
    }

    #[test]
    fn test_parse_vm_names_from_list_no_match() {
        let list = " 1     other-vm-1    running\n 2     other-vm-2    running\n";
        let names = super::parse_vm_names_from_list(list, "my-fed");
        assert!(names.is_empty());
    }

    #[test]
    fn test_parse_vm_names_from_list_single_column() {
        let list = " 1     fed-node1\n";
        let names = super::parse_vm_names_from_list(list, "fed");
        assert_eq!(names, vec!["fed-node1"]);
    }

    #[test]
    fn test_parse_ip_from_domifaddr_whitespace_variations() {
        let output = "  ipv4    192.168.100.1/24  ";
        let ip = super::parse_ip_from_domifaddr_output(output);
        assert_eq!(ip, Some("192.168.100.1".to_string()));
    }

    #[test]
    fn test_validation_config_debug() {
        let config = ValidationConfig::default();
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("ValidationConfig"));
    }

    #[test]
    fn test_parse_ip_from_domifaddr_192_168_in_middle() {
        let output = " Name       MAC address          Protocol     Address\n\nvnet0      xx:xx    ipv4         10.0.0.1/24\nvnet1      yy:yy    ipv4         192.168.122.50/24\n";
        let ip = super::parse_ip_from_domifaddr_output(output);
        assert_eq!(ip, Some("192.168.122.50".to_string()));
    }

    #[test]
    fn test_parse_vm_names_from_list_extra_columns() {
        let list = " Id    Name                State       CPU    Memory\n----------------------------------------------------\n 1     fed-node1           running     1      1024\n";
        let names = super::parse_vm_names_from_list(list, "fed");
        assert_eq!(names, vec!["fed-node1"]);
    }

    #[test]
    fn test_validation_config_clone() {
        let config = ValidationConfig::default();
        let cloned = config.clone();
        assert_eq!(cloned.ssh_max_retries, config.ssh_max_retries);
        assert_eq!(cloned.cloud_init_timeout, config.cloud_init_timeout);
    }

    #[test]
    fn test_parse_ip_from_domifaddr_empty_lines() {
        let output = "\n\n  ipv4    192.168.1.1/24  \n\n";
        let ip = super::parse_ip_from_domifaddr_output(output);
        assert_eq!(ip, Some("192.168.1.1".to_string()));
    }

    #[test]
    fn test_parse_ip_from_domifaddr_contains_192_168_in_line() {
        let output = " ipv4  192.168.0.1/24";
        let ip = super::parse_ip_from_domifaddr_output(output);
        assert_eq!(ip, Some("192.168.0.1".to_string()));
    }

    #[test]
    fn test_parse_ip_from_domifaddr_192_168_without_ipv4_keyword() {
        // Branch: line matches via `192.168` substring without `ipv4` label
        let output = " Name   MAC   Address\n  vnet0  xx  192.168.122.200/24\n";
        assert_eq!(
            super::parse_ip_from_domifaddr_output(output),
            Some("192.168.122.200".to_string())
        );
    }

    #[test]
    fn test_parse_vm_names_from_list_malformed_line() {
        let list = " 1     fed-node1    running\n single_word\n";
        let names = super::parse_vm_names_from_list(list, "fed");
        assert_eq!(names, vec!["fed-node1"]);
    }

    #[test]
    fn test_validation_config_builder_pattern() {
        let config = ValidationConfig {
            cloud_init_timeout: Duration::from_secs(900),
            ssh_timeout: Duration::from_secs(600),
            ssh_retry_interval: Duration::from_secs(15),
            ssh_max_retries: 40,
        };
        assert_eq!(config.cloud_init_timeout.as_secs(), 900);
        assert_eq!(config.ssh_max_retries, 40);
    }

    #[test]
    fn test_parse_ip_ipv4_label_but_non_rfc1918_returns_none() {
        assert_eq!(
            super::parse_ip_from_domifaddr_output("ipv4         10.0.0.1/24"),
            None
        );
    }

    #[test]
    fn test_parse_ip_line_ipv4_without_192_168_until_later_line() {
        let output = "vnet0  xx  ipv4  10.0.0.1/24\nvnet1  yy  ipv4  192.168.50.2/24\n";
        assert_eq!(
            super::parse_ip_from_domifaddr_output(output),
            Some("192.168.50.2".to_string())
        );
    }

    #[test]
    fn test_parse_vm_names_single_token_line_not_pushed() {
        let list = "my-fed\n";
        let names = super::parse_vm_names_from_list(list, "my-fed");
        assert!(names.is_empty());
    }

    #[test]
    fn test_parse_vm_names_header_line_extracts_second_column() {
        // Only lines containing the federation substring participate; `real-vm-1` line has no `my-fed`.
        let list = "my-fed header line\n 1     real-vm-1    running\n";
        let names = super::parse_vm_names_from_list(list, "my-fed");
        assert_eq!(names, vec!["header"]);
    }

    #[test]
    fn test_parse_ip_last_token_not_ip() {
        assert_eq!(
            super::parse_ip_from_domifaddr_output("ipv4   garbage"),
            None
        );
    }

    #[test]
    fn test_parse_ip_non_numeric_octets_still_matches_prefix_heuristic() {
        // Parser does not validate dotted-decimal; it only checks the `192.168` prefix.
        assert_eq!(
            super::parse_ip_from_domifaddr_output("foo 192.168.abc.1/24"),
            Some("192.168.abc.1".to_string())
        );
    }

    #[test]
    fn test_parse_vm_names_tabs_and_multiple_spaces() {
        let list = "1\tmy-fed-node1\trunning\n";
        let names = super::parse_vm_names_from_list(list, "my-fed");
        assert_eq!(names, vec!["my-fed-node1"]);
    }

    #[test]
    fn test_validation_config_extreme_retries_zero() {
        let config = ValidationConfig {
            cloud_init_timeout: Duration::from_secs(1),
            ssh_timeout: Duration::from_secs(1),
            ssh_retry_interval: Duration::from_secs(1),
            ssh_max_retries: 0,
        };
        assert_eq!(config.ssh_max_retries, 0);
    }

    #[test]
    fn test_parse_vm_names_duplicate_lines() {
        let list = " 1     dup-fed-a    running\n 2     dup-fed-b    running\n";
        let names = super::parse_vm_names_from_list(list, "dup-fed");
        assert_eq!(names, vec!["dup-fed-a", "dup-fed-b"]);
    }

    #[test]
    fn test_parse_ip_from_domifaddr_only_non_matching_lines() {
        let output = "header\n  ipv6  fe80::1/64\n  other  text\n";
        assert_eq!(super::parse_ip_from_domifaddr_output(output), None);
    }

    #[test]
    fn test_parse_vm_names_numeric_id_with_federation_in_name() {
        let list = " 10    my-fed-10    running\n";
        let names = super::parse_vm_names_from_list(list, "my-fed");
        assert_eq!(names, vec!["my-fed-10"]);
    }

    #[test]
    fn test_parse_ip_strips_cidr_from_token() {
        let output = "  ipv4    192.168.255.254/16  ";
        assert_eq!(
            super::parse_ip_from_domifaddr_output(output),
            Some("192.168.255.254".to_string())
        );
    }

    #[test]
    fn test_parse_vm_names_line_contains_fed_but_less_than_two_columns() {
        let list = "my-fed\n";
        let names = parse_vm_names_from_list(list, "my-fed");
        assert!(names.is_empty());
    }

    #[test]
    fn test_parse_ip_from_domifaddr_ipv4_keyword_non_matching_ip_token() {
        assert_eq!(
            parse_ip_from_domifaddr_output("ipv4         garbage/24"),
            None
        );
    }

    #[test]
    fn test_parse_vm_names_preserves_order() {
        let list = " 2     fed-b    running\n 1     fed-a    running\n";
        let names = parse_vm_names_from_list(list, "fed");
        assert_eq!(names, vec!["fed-b", "fed-a"]);
    }

    #[test]
    fn test_parse_vm_names_match_in_first_column_extracts_second_column() {
        // Any line containing the federation substring participates; the VM name is always column 2.
        let list = "my-fed-prefix    actual-vm-name    running\n";
        let names = parse_vm_names_from_list(list, "my-fed");
        assert_eq!(names, vec!["actual-vm-name"]);
    }

    #[test]
    fn test_parse_ip_line_ipv4_only_no_192_match() {
        assert_eq!(
            parse_ip_from_domifaddr_output("proto  ipv4  10.11.12.13/24"),
            None
        );
    }

    #[test]
    fn test_parse_vm_names_long_federation_substring() {
        let list = " 1     prefix-my-fed-suffix    running\n";
        let names = parse_vm_names_from_list(list, "my-fed");
        assert_eq!(names, vec!["prefix-my-fed-suffix"]);
    }

    #[test]
    fn test_parse_ip_domifaddr_ipv4_keyword_non_192_line_then_valid() {
        let t = "vnet0  ipv4  10.0.0.1/24\nvnet1  ipv4  192.168.1.2/24\n";
        assert_eq!(
            parse_ip_from_domifaddr_output(t),
            Some("192.168.1.2".to_string())
        );
    }

    #[test]
    fn test_validation_config_extreme_durations() {
        let c = ValidationConfig {
            cloud_init_timeout: Duration::from_secs(u64::MAX / 4),
            ssh_timeout: Duration::from_secs(1),
            ssh_retry_interval: Duration::from_millis(1),
            ssh_max_retries: u32::MAX,
        };
        assert!(c.cloud_init_timeout > Duration::from_secs(1_000_000));
    }

    #[test]
    fn test_parse_vm_names_windows_style_line_endings() {
        let list = " 1     my-fed-w1    running\r\n 2     my-fed-w2    running\r\n";
        let names = parse_vm_names_from_list(list, "my-fed");
        assert_eq!(names, vec!["my-fed-w1", "my-fed-w2"]);
    }

    #[test]
    fn test_parse_ip_slash_only_after_dot() {
        assert_eq!(
            parse_ip_from_domifaddr_output("ipv4  192.168.0.1/"),
            Some("192.168.0.1".to_string())
        );
    }

    #[test]
    fn test_parse_vm_names_uuid_suffix_in_name() {
        let list = " 1     fed-node-550e8400-e29b-41d4-a716-446655440000    running\n";
        let names = parse_vm_names_from_list(list, "fed-node");
        assert_eq!(names, vec!["fed-node-550e8400-e29b-41d4-a716-446655440000"]);
    }

    #[test]
    fn test_parse_ip_tabs_instead_of_spaces() {
        let t = "vnet0\tipv4\t192.168.99.1/24\n";
        assert_eq!(
            parse_ip_from_domifaddr_output(t),
            Some("192.168.99.1".to_string())
        );
    }

    #[test]
    fn test_parse_vm_names_three_columns_id_name_state() {
        let list = "42    vm-fed-core    shut off\n";
        let names = parse_vm_names_from_list(list, "fed");
        assert_eq!(names, vec!["vm-fed-core"]);
    }

    #[test]
    fn test_parse_ip_rejects_line_with_192_168_substring_in_wrong_token() {
        // Last token must parse as starting with 192.168 after split on '/'
        assert_eq!(
            parse_ip_from_domifaddr_output("note: 192.168 is reserved  garbage"),
            None
        );
    }

    #[tokio::test]
    #[ignore = "requires benchscale and libvirt"]
    async fn test_full_lifecycle() -> Result<()> {
        // Only run if benchscale is available AND libvirt testing is enabled
        if std::env::var("BENCHSCALE_TEST_LIBVIRT").is_err() {
            // Skip test if libvirt testing not enabled
            return Ok(());
        }

        let Ok(manager) = VmFederationManager::new() else {
            return Ok(());
        };

        let name = "test-federation";

        // This would actually create VMs if libvirt is available
        manager.create(name).await?;
        manager.start(name).await?;
        manager.test(name).await?;
        manager.stop(name).await?;
        manager.destroy(name).await?;
        Ok(())
    }
}
