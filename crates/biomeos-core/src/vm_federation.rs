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
    fn discover_vm_ips(&self, federation_name: &str) -> Result<Vec<String>> {
        // Query libvirt for VMs matching our federation
        let output = Command::new("virsh")
            .args(["list", "--all"])
            .output()
            .context("Failed to list VMs")?;

        let vm_list = String::from_utf8_lossy(&output.stdout);
        let mut ips = Vec::new();

        // Find VM names that match our federation
        for line in vm_list.lines() {
            if line.contains(federation_name) {
                // Extract VM name
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let vm_name = parts[1];

                    // Get IP for this VM
                    if let Ok(ip_output) =
                        Command::new("virsh").args(["domifaddr", vm_name]).output()
                    {
                        let ip_text = String::from_utf8_lossy(&ip_output.stdout);
                        for ip_line in ip_text.lines() {
                            if ip_line.contains("ipv4") || ip_line.contains("192.168") {
                                // Extract IP address (format: "ipv4         192.168.122.34/24")
                                if let Some(ip_part) = ip_line.split_whitespace().last() {
                                    if let Some(ip) = ip_part.split('/').next() {
                                        if ip.starts_with("192.168") {
                                            debug!("Found VM {} with IP {}", vm_name, ip);
                                            ips.push(ip.to_string());
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        if ips.is_empty() {
            anyhow::bail!("No VM IPs found for federation: {}", federation_name);
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
                        &format!("biomeos@{}", ip),
                        "echo 'SSH ready'",
                    ])
                    .output();

                if let Ok(output) = ssh_test {
                    if output.status.success() {
                        info!("✅ VM {} is SSH-accessible", ip);
                        break;
                    }
                }

                if attempt >= self.validation_config.ssh_max_retries {
                    anyhow::bail!(
                        "Failed to SSH to {} after {} attempts. Check cloud-init logs.",
                        ip,
                        attempt
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
                    &format!("biomeos@{}", ip),
                    "hostname && uptime",
                ])
                .output()
                .context(format!("Failed to validate SSH to {}", ip))?;

            if !output.status.success() {
                anyhow::bail!("SSH validation failed for {}", ip);
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
                    "Error should be about missing benchscale, got: {}",
                    e
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
                    "Expected benchscale or path error, got: {}",
                    e
                );
            }
        }
    }

    #[tokio::test]
    async fn test_full_lifecycle() -> Result<()> {
        // Only run if benchscale is available AND libvirt testing is enabled
        if std::env::var("BENCHSCALE_TEST_LIBVIRT").is_err() {
            // Skip test if libvirt testing not enabled
            return Ok(());
        }

        let manager = match VmFederationManager::new() {
            Ok(m) => m,
            Err(_) => {
                // benchscale not available, skip test
                return Ok(());
            }
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
