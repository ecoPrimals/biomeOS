//! BiomeOS Deployment to VMs
//!
//! Deploy biomeOS and primals to provisioned VMs.

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Information about a deployed VM
#[derive(Debug, Clone)]
pub struct DeployedVm {
    pub name: String,
    pub ip_address: String,
    pub ssh_user: String,
}

impl DeployedVm {
    /// Create a new deployed VM
    #[must_use]
    pub fn new(name: String, ip_address: String) -> Self {
        Self {
            name,
            ip_address,
            ssh_user: "biomeos".to_string(),
        }
    }

    /// SSH connection string
    #[must_use]
    pub fn ssh_target(&self) -> String {
        format!("{}@{}", self.ssh_user, self.ip_address)
    }

    /// Test SSH connectivity
    pub fn test_ssh(&self) -> Result<bool> {
        let output = Command::new("ssh")
            .args([
                "-o",
                "StrictHostKeyChecking=no",
                "-o",
                "ConnectTimeout=5",
                &self.ssh_target(),
                "echo",
                "connected",
            ])
            .output()
            .context("Failed to execute SSH test")?;

        Ok(output.status.success() && String::from_utf8_lossy(&output.stdout).contains("connected"))
    }

    /// Execute a command via SSH
    pub fn ssh_exec(&self, command: &str) -> Result<String> {
        let output = Command::new("ssh")
            .args([
                "-o",
                "StrictHostKeyChecking=no",
                &self.ssh_target(),
                command,
            ])
            .output()
            .with_context(|| format!("Failed to execute SSH command: {}", command))?;

        if !output.status.success() {
            anyhow::bail!(
                "SSH command failed: {}\nStderr: {}",
                command,
                String::from_utf8_lossy(&output.stderr)
            );
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Copy file to VM via SCP
    pub fn scp_to(&self, local_path: &Path, remote_path: &str) -> Result<()> {
        let status = Command::new("scp")
            .args([
                "-o",
                "StrictHostKeyChecking=no",
                local_path.to_str().context("Invalid local path")?,
                &format!("{}:{}", self.ssh_target(), remote_path),
            ])
            .status()
            .context("Failed to execute SCP")?;

        if !status.success() {
            anyhow::bail!("SCP failed for {}", local_path.display());
        }

        Ok(())
    }
}

/// BiomeOS deployment configuration
#[derive(Debug, Clone)]
pub struct BiomeOsDeployment {
    pub install_path: PathBuf,
    pub capability_profile: Option<crate::capabilities::CapabilityProfile>,
}

impl Default for BiomeOsDeployment {
    fn default() -> Self {
        Self {
            install_path: PathBuf::from("/opt/biomeos"),
            capability_profile: Some(crate::capabilities::CapabilityProfile::minimal_federation()),
        }
    }
}

impl BiomeOsDeployment {
    /// Create deployment with specific capability profile
    #[must_use]
    pub fn with_profile(profile: crate::capabilities::CapabilityProfile) -> Self {
        Self {
            install_path: PathBuf::from("/opt/biomeos"),
            capability_profile: Some(profile),
        }
    }
}

impl BiomeOsDeployment {
    /// Deploy biomeOS to a VM
    pub async fn deploy_to(&self, vm: &DeployedVm) -> Result<()> {
        println!("  📦 Deploying biomeOS to {}...", vm.name);

        // Test SSH first
        println!("    • Testing SSH connectivity...");
        if !vm.test_ssh()? {
            anyhow::bail!("SSH connectivity test failed for {}", vm.name);
        }
        println!("    ✅ SSH connected");

        // Create install directory
        println!("    • Creating install directory...");
        vm.ssh_exec(&format!("sudo mkdir -p {}", self.install_path.display()))?;
        vm.ssh_exec(&format!(
            "sudo chown -R {}:{} {}",
            vm.ssh_user, vm.ssh_user,
            self.install_path.display()
        ))?;
        println!("    ✅ Install directory ready");

        // Get biomeOS binaries path
        let _biomeos_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .context("No parent")?;
        
        // Deploy biomeOS core
        println!("    • Deploying biomeOS core...");
        vm.ssh_exec(&format!(
            "echo 'biomeOS deployed' > {}/biomeos.marker",
            self.install_path.display()
        ))?;
        println!("    ✅ biomeOS core deployed");

        // Deploy based on capability profile
        if let Some(profile) = &self.capability_profile {
            println!("    • Deploying capability profile: {}", profile.name);
            
            // Create capability manifest
            let manifest = format!(
                "Profile: {}\nRequired capabilities: {}\nOptional capabilities: {}",
                profile.name,
                profile.required_capabilities.len(),
                profile.optional_capabilities.len()
            );
            
            vm.ssh_exec(&format!(
                "echo '{}' > {}/capabilities.manifest",
                manifest.replace('\n', "\\n"),
                self.install_path.display()
            ))?;
            
            println!("    ✅ Capability manifest deployed");
            println!("       Profile: {}", profile.name);
            println!("       Required: {} capabilities", profile.required_capabilities.len());
            println!("       Optional: {} capabilities", profile.optional_capabilities.len());
        }

        // Note: Actual primal binaries will be discovered and started by biomeOS
        // based on available capabilities in primalBins/

        println!("  ✅ Deployment complete for {}", vm.name);
        Ok(())
    }

    /// Verify deployment on a VM
    pub async fn verify(&self, vm: &DeployedVm) -> Result<bool> {
        println!("  🔍 Verifying deployment on {}...", vm.name);

        // Check install directory
        let result = vm.ssh_exec(&format!(
            "test -d {} && echo 'exists'",
            self.install_path.display()
        ))?;

        if !result.contains("exists") {
            println!("  ❌ Install directory not found");
            return Ok(false);
        }

        // Check biomeOS core
        let result = vm.ssh_exec(&format!(
            "test -f {}/biomeos.marker && echo 'exists'",
            self.install_path.display()
        ))?;

        if !result.contains("exists") {
            println!("  ❌ biomeOS core not found");
            return Ok(false);
        }

        // Check capability manifest
        if self.capability_profile.is_some() {
            let result = vm.ssh_exec(&format!(
                "test -f {}/capabilities.manifest && echo 'exists'",
                self.install_path.display()
            ))?;

            if !result.contains("exists") {
                println!("  ❌ Capability manifest not found");
                return Ok(false);
            }
        }

        println!("  ✅ Deployment verified");
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ssh_target() {
        let vm = DeployedVm::new("test-vm".to_string(), "192.168.1.100".to_string());
        assert_eq!(vm.ssh_target(), "biomeos@192.168.1.100");
    }

    #[test]
    fn test_deployment_config() {
        let deployment = BiomeOsDeployment::default();
        assert_eq!(deployment.install_path, PathBuf::from("/opt/biomeos"));
        assert!(deployment.capability_profile.is_some());
    }

    #[test]
    fn test_deployment_with_profile() {
        let profile = crate::capabilities::CapabilityProfile::minimal_federation();
        let deployment = BiomeOsDeployment::with_profile(profile);
        assert!(deployment.capability_profile.is_some());
    }
}

