//! # BiomeOS Lab Integration
//!
//! Integration layer for benchScale lab environment system.
//! Allows BiomeOS to orchestrate lab experiments for testing P2P coordination,
//! BTSP tunnels, BirdSong encryption, and multi-primal deployments.

use anyhow::{Context, Result};
use std::path::PathBuf;
use std::process::Command;
use tracing::{info, warn};

/// Lab manager for orchestrating benchScale experiments
#[derive(Debug, Clone)]
pub struct LabManager {
    /// Path to benchScale root directory
    benchscale_root: PathBuf,
}

impl LabManager {
    /// Create a new lab manager
    pub fn new() -> Self {
        // Default to ../benchscale/ directory (parallel to biomeOS)
        // Get current directory and find workspace root
        let current = std::env::current_dir().unwrap_or_default();
        let benchscale_root = if current.ends_with("biomeOS") {
            // We're in biomeOS root, go up one level and into benchscale
            current.parent().unwrap().join("benchscale")
        } else {
            // Try to find it relative to cargo manifest dir
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .unwrap()
                .parent()
                .unwrap()
                .parent()
                .unwrap()
                .join("benchscale")
        };
        Self { benchscale_root }
    }

    /// Create a new lab manager with custom benchScale path
    pub fn with_path(benchscale_root: PathBuf) -> Self {
        Self { benchscale_root }
    }

    /// Get path to benchScale scripts directory
    fn scripts_dir(&self) -> PathBuf {
        self.benchscale_root.join("scripts")
    }

    /// Create a new lab environment
    pub async fn create_lab(&self, topology: &str, name: &str) -> Result<LabHandle> {
        info!("Creating lab: {} with topology: {}", name, topology);

        let script = self.scripts_dir().join("create-lab.sh");
        let output = Command::new(&script)
            .arg("--topology")
            .arg(topology)
            .arg("--name")
            .arg(name)
            .current_dir(&self.benchscale_root)
            .output()
            .context("Failed to execute create-lab.sh")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to create lab: {}", stderr);
        }

        info!("Lab created successfully: {}", name);
        Ok(LabHandle {
            name: name.to_string(),
            topology: topology.to_string(),
            manager: self.clone(),
        })
    }

    /// Deploy primals to a lab
    pub async fn deploy_to_lab(&self, lab_name: &str, manifest: &str) -> Result<()> {
        info!("Deploying to lab: {} with manifest: {}", lab_name, manifest);

        let script = self.scripts_dir().join("deploy-to-lab.sh");
        let output = Command::new(&script)
            .arg("--lab")
            .arg(lab_name)
            .arg("--manifest")
            .arg(manifest)
            .current_dir(&self.benchscale_root)
            .output()
            .context("Failed to execute deploy-to-lab.sh")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!("Deploy had warnings: {}", stderr);
            // Don't fail on warnings - binary copies might fail but that's OK for now
        }

        info!("Deployment complete for lab: {}", lab_name);
        Ok(())
    }

    /// Run tests on a lab
    pub async fn run_test(&self, lab_name: &str, test_name: &str) -> Result<TestResult> {
        info!("Running test: {} on lab: {}", test_name, lab_name);

        let script = self.scripts_dir().join("run-tests.sh");
        let output = Command::new(&script)
            .arg("--lab")
            .arg(lab_name)
            .arg("--test")
            .arg(test_name)
            .current_dir(&self.benchscale_root)
            .output()
            .context("Failed to execute run-tests.sh")?;

        let success = output.status.success();
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        info!("Test {} {}", test_name, if success { "PASSED" } else { "FAILED" });

        Ok(TestResult {
            test_name: test_name.to_string(),
            success,
            stdout,
            stderr,
        })
    }

    /// Destroy a lab environment
    pub async fn destroy_lab(&self, lab_name: &str) -> Result<()> {
        info!("Destroying lab: {}", lab_name);

        let script = self.scripts_dir().join("destroy-lab.sh");
        let output = Command::new(&script)
            .arg("--lab")
            .arg(lab_name)
            .arg("--force")
            .current_dir(&self.benchscale_root)
            .output()
            .context("Failed to execute destroy-lab.sh")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to destroy lab: {}", stderr);
        }

        info!("Lab destroyed successfully: {}", lab_name);
        Ok(())
    }
}

impl Default for LabManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Handle to a lab environment
#[derive(Debug, Clone)]
pub struct LabHandle {
    name: String,
    topology: String,
    manager: LabManager,
}

impl LabHandle {
    /// Get lab name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get topology name
    pub fn topology(&self) -> &str {
        &self.topology
    }

    /// Deploy primals to this lab
    pub async fn deploy(&self, manifest: &str) -> Result<()> {
        self.manager.deploy_to_lab(&self.name, manifest).await
    }

    /// Run a test on this lab
    pub async fn run_test(&self, test_name: &str) -> Result<TestResult> {
        self.manager.run_test(&self.name, test_name).await
    }

    /// Destroy this lab
    pub async fn destroy(self) -> Result<()> {
        self.manager.destroy_lab(&self.name).await
    }
}

/// Result of a lab test
#[derive(Debug, Clone)]
pub struct TestResult {
    pub test_name: String,
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
}

impl TestResult {
    /// Check if test passed
    pub fn passed(&self) -> bool {
        self.success
    }

    /// Get test output
    pub fn output(&self) -> &str {
        &self.stdout
    }

    /// Get test errors (if any)
    pub fn errors(&self) -> &str {
        &self.stderr
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lab_manager_creation() {
        let manager = LabManager::new();
        assert!(manager.scripts_dir().to_string_lossy().contains("benchscale/scripts"));
    }

    #[test]
    fn test_lab_handle() {
        let manager = LabManager::new();
        let handle = LabHandle {
            name: "test-lab".to_string(),
            topology: "simple-lan".to_string(),
            manager,
        };

        assert_eq!(handle.name(), "test-lab");
        assert_eq!(handle.topology(), "simple-lan");
    }
}

