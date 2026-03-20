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
use std::path::{Path, PathBuf};
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

/// Collect 192.168.x.x IPs from virsh `domifaddr` outputs (testable; used by [`VmFederationManager::discover_vm_ips`]).
/// BenchScale `cargo run --release -- create …` argv (testable without invoking `cargo`).
pub(crate) fn benchscale_create_argv<'a>(name: &'a str, topology: &'a str) -> [&'a str; 9] {
    [
        "run",
        "--release",
        "--",
        "create",
        name,
        "--topology",
        topology,
        "--backend",
        "libvirt",
    ]
}

/// BenchScale `cargo run --release -- <subcommand> <name>` argv (start/stop/destroy/test).
pub(crate) fn benchscale_subcommand_argv<'a>(subcommand: &'a str, name: &'a str) -> [&'a str; 5] {
    ["run", "--release", "--", subcommand, name]
}

/// Return topology path as UTF-8 for CLI args, or error if the path is not valid Unicode.
pub(crate) fn topology_path_for_cli(path: &Path) -> Result<&str> {
    path.to_str()
        .ok_or_else(|| anyhow::anyhow!("Topology path contains invalid UTF-8"))
}

pub(crate) fn collect_ips_for_vm_names(
    vm_names: Vec<String>,
    mut domifaddr_output: impl FnMut(&str) -> std::io::Result<std::process::Output>,
) -> Vec<String> {
    let mut ips = Vec::new();
    for vm_name in vm_names {
        if let Ok(ip_output) = domifaddr_output(&vm_name) {
            let ip_text = String::from_utf8_lossy(&ip_output.stdout);
            if let Some(ip) = parse_ip_from_domifaddr_output(&ip_text) {
                debug!("Found VM {} with IP {}", vm_name, ip);
                ips.push(ip);
            }
        }
    }
    ips
}

/// Wait until SSH to `ip` succeeds or limits are hit (testable via `try_ssh`).
pub(crate) async fn wait_for_vm_ssh_ready(
    ip: &str,
    validation_config: &ValidationConfig,
    start: Instant,
    mut try_ssh: impl FnMut() -> std::io::Result<std::process::Output>,
) -> Result<()> {
    let timeout = validation_config.cloud_init_timeout;
    let mut attempt = 0u32;
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
            attempt, validation_config.ssh_max_retries, ip
        );

        if let Ok(output) = try_ssh()
            && output.status.success()
        {
            info!("✅ VM {} is SSH-accessible", ip);
            return Ok(());
        }

        if attempt >= validation_config.ssh_max_retries {
            anyhow::bail!("Failed to SSH to {ip} after {attempt} attempts. Check cloud-init logs.");
        }

        let wait_time = validation_config.ssh_retry_interval * attempt;
        debug!("Waiting {}s before retry", wait_time.as_secs());
        tokio::time::sleep(wait_time).await;
    }
}

/// Validate a single SSH probe [`std::process::Output`] (testable).
pub(crate) fn validate_ssh_probe_output(ip: &str, output: &std::process::Output) -> Result<()> {
    if !output.status.success() {
        anyhow::bail!("SSH validation failed for {ip}");
    }
    Ok(())
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
        let topology = topology_path_for_cli(&self.topology_path)?;
        let output = Command::new("cargo")
            .current_dir(&self.benchscale_root)
            .args(benchscale_create_argv(name, topology))
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
    #[expect(
        clippy::unused_self,
        reason = "instance method for API symmetry with other VM federation helpers"
    )]
    fn discover_vm_ips(&self, federation_name: &str) -> Result<Vec<String>> {
        // Query libvirt for VMs matching our federation
        let output = Command::new("virsh")
            .args(["list", "--all"])
            .output()
            .context("Failed to list VMs")?;

        let vm_list = String::from_utf8_lossy(&output.stdout);
        let vm_names = parse_vm_names_from_list(&vm_list, federation_name);
        let ips = collect_ips_for_vm_names(vm_names, |name| {
            Command::new("virsh").args(["domifaddr", name]).output()
        });

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

        for (idx, ip) in vm_ips.iter().enumerate() {
            info!(
                "Waiting for VM {}/{} ({}) to be ready",
                idx + 1,
                vm_ips.len(),
                ip
            );

            let ip = ip.clone();
            wait_for_vm_ssh_ready(&ip, &self.validation_config, start, || {
                Command::new("ssh")
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
                    .output()
            })
            .await?;
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

            validate_ssh_probe_output(ip, &output)?;

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
            .args(benchscale_subcommand_argv("start", name))
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
            .args(benchscale_subcommand_argv("test", name))
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
            .args(benchscale_subcommand_argv("stop", name))
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
            .args(benchscale_subcommand_argv("destroy", name))
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
            .args(benchscale_subcommand_argv("status", name))
            .output()
            .context("Failed to execute benchscale status")?;

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}

#[cfg(test)]
mod tests;
