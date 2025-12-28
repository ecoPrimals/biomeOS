//! BiomeOS + benchScale Integration - VM Federation Manager
//!
//! This module provides high-level APIs for managing BiomeOS VM federations
//! using benchScale's libvirt backend.

use anyhow::{Context, Result};
use std::path::PathBuf;
use std::process::Command;
use tracing::{info, warn};

/// VM Federation Manager
///
/// Provides a Rust API for managing BiomeOS VM federations using benchScale.
pub struct VmFederationManager {
    benchscale_root: PathBuf,
    topology_path: PathBuf,
}

impl VmFederationManager {
    /// Create a new VM Federation Manager
    pub fn new() -> Result<Self> {
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
        })
    }

    /// Create the VM federation
    pub async fn create(&self, name: &str) -> Result<()> {
        info!("Creating VM federation: {}", name);

        let output = Command::new("cargo")
            .current_dir(&self.benchscale_root)
            .args([
                "run",
                "--release",
                "--",
                "create",
                name,
                "--topology",
                self.topology_path.to_str().unwrap(),
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

        info!("VM federation created: {}", name);
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
    fn test_manager_creation() {
        let manager = VmFederationManager::new();
        assert!(manager.is_ok(), "Should create VM federation manager");
    }

    #[tokio::test]
    async fn test_full_lifecycle() {
        let manager = VmFederationManager::new().expect("Create manager");
        let name = "test-federation";

        // This would actually create VMs if libvirt is available
        // For CI, we'd mock this or skip if libvirt isn't present
        if std::env::var("BENCHSCALE_TEST_LIBVIRT").is_ok() {
            manager.create(name).await.expect("Create federation");
            manager.start(name).await.expect("Start federation");
            manager.test(name).await.expect("Test federation");
            manager.stop(name).await.expect("Stop federation");
            manager.destroy(name).await.expect("Destroy federation");
        }
    }
}
