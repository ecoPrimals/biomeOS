//! VM Type Configurations
//!
//! Different VM types for different validation scenarios.

use anyhow::Result;
use std::path::PathBuf;

/// VM configuration for different use cases
#[derive(Debug, Clone)]
pub struct VmConfig {
    pub name: String,
    pub vm_type: VmType,
    pub memory_mb: u32,
    pub vcpus: u32,
    pub disk_size_gb: u32,
}

/// Different types of VMs we can provision
#[derive(Debug, Clone, Copy)]
pub enum VmType {
    /// Desktop VM with RustDesk (agentReagents template)
    Desktop,
    /// Minimal server VM (Ubuntu base)
    Server,
    /// Federation node (biomeOS + Songbird)
    FederationNode,
    /// Compute node (biomeOS + Toadstool)
    ComputeNode,
}

impl VmConfig {
    /// Create a desktop VM configuration
    #[must_use]
    pub fn desktop(name: &str) -> Self {
        Self {
            name: name.to_string(),
            vm_type: VmType::Desktop,
            memory_mb: 2048,  // 2GB
            vcpus: 2,
            disk_size_gb: 25,
        }
    }

    /// Create a server VM configuration
    #[must_use]
    pub fn server(name: &str) -> Self {
        Self {
            name: name.to_string(),
            vm_type: VmType::Server,
            memory_mb: 1024,  // 1GB
            vcpus: 1,
            disk_size_gb: 20,
        }
    }

    /// Create a federation node configuration
    #[must_use]
    pub fn federation_node(name: &str) -> Self {
        Self {
            name: name.to_string(),
            vm_type: VmType::FederationNode,
            memory_mb: 3072,  // 3GB (needs more for biomeOS + Songbird)
            vcpus: 2,
            disk_size_gb: 30,
        }
    }

    /// Create a compute node configuration
    #[must_use]
    pub fn compute_node(name: &str) -> Self {
        Self {
            name: name.to_string(),
            vm_type: VmType::ComputeNode,
            memory_mb: 4096,  // 4GB (compute-heavy)
            vcpus: 4,
            disk_size_gb: 40,
        }
    }

    /// Get the appropriate template path for this VM type
    /// 
    /// Uses benchScale's template discovery system with graceful fallback to agentReagents.
    /// Set BENCHSCALE_TEMPLATE_PATH environment variable for custom locations.
    pub fn template_path(&self) -> Result<PathBuf> {
        // Let benchScale handle template discovery
        // It will auto-discover from agentReagents if available
        let template_name = match self.vm_type {
            VmType::Desktop | VmType::FederationNode | VmType::ComputeNode => {
                "rustdesk-ubuntu-22.04-template.qcow2"
            }
            VmType::Server => {
                "rustdesk-ubuntu-22.04-template.qcow2"
            }
        };

        // Try environment variable first (benchScale standard)
        if let Ok(template_dir) = std::env::var("BENCHSCALE_TEMPLATE_PATH") {
            let path = PathBuf::from(template_dir).join(template_name);
            if path.exists() {
                return Ok(path);
            }
        }

        // Fallback: check agentReagents standard location
        let agentreagents_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("validation/ should have parent")
            .parent()
            .expect("biomeOS should have parent")
            .parent()
            .expect("phase2 should have parent")
            .join("primalTools")
            .join("agentReagents")
            .join("images")
            .join("templates")
            .join(template_name);

        if agentreagents_path.exists() {
            return Ok(agentreagents_path);
        }

        anyhow::bail!(
            "Template '{}' not found. Please either:\n\
             1. Set BENCHSCALE_TEMPLATE_PATH environment variable, or\n\
             2. Ensure agentReagents templates are built (see AGENTREAGENTS_INTEGRATION.md)\n\
             \n\
             benchScale will auto-discover templates from agentReagents when available.",
            template_name
        )
    }

    /// Human-readable description
    #[must_use]
    pub fn description(&self) -> String {
        match self.vm_type {
            VmType::Desktop => format!("Desktop VM ({}MB RAM, {} CPUs)", self.memory_mb, self.vcpus),
            VmType::Server => format!("Server VM ({}MB RAM, {} CPUs)", self.memory_mb, self.vcpus),
            VmType::FederationNode => {
                format!("Federation Node ({}MB RAM, {} CPUs)", self.memory_mb, self.vcpus)
            }
            VmType::ComputeNode => {
                format!("Compute Node ({}MB RAM, {} CPUs)", self.memory_mb, self.vcpus)
            }
        }
    }
}

/// Topology configuration for multi-VM setups
#[derive(Debug, Clone)]
pub struct Topology {
    pub name: String,
    pub vms: Vec<VmConfig>,
}

impl Topology {
    /// Create a new topology
    #[must_use]
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            vms: Vec::new(),
        }
    }

    /// Add a VM to the topology
    pub fn add_vm(&mut self, vm: VmConfig) -> &mut Self {
        self.vms.push(vm);
        self
    }

    /// Create a 2-node federation topology
    #[must_use]
    pub fn federation_2_node() -> Self {
        let mut topology = Self::new("federation-2node");
        topology
            .add_vm(VmConfig::federation_node("federation-vm1"))
            .add_vm(VmConfig::federation_node("federation-vm2"));
        topology
    }

    /// Create a 3-node federation topology
    #[must_use]
    pub fn federation_3_node() -> Self {
        let mut topology = Self::new("federation-3node");
        topology
            .add_vm(VmConfig::federation_node("federation-vm1"))
            .add_vm(VmConfig::federation_node("federation-vm2"))
            .add_vm(VmConfig::federation_node("federation-vm3"));
        topology
    }

    /// Create a mixed topology (federation + compute)
    #[must_use]
    pub fn mixed_ecosystem() -> Self {
        let mut topology = Self::new("mixed-ecosystem");
        topology
            .add_vm(VmConfig::federation_node("federation-vm1"))
            .add_vm(VmConfig::federation_node("federation-vm2"))
            .add_vm(VmConfig::compute_node("compute-vm1"));
        topology
    }

    /// Create a simple test topology
    #[must_use]
    pub fn simple_test() -> Self {
        let mut topology = Self::new("simple-test");
        topology
            .add_vm(VmConfig::desktop("test-vm1"))
            .add_vm(VmConfig::desktop("test-vm2"));
        topology
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vm_config_creation() {
        let desktop = VmConfig::desktop("test-desktop");
        assert_eq!(desktop.memory_mb, 2048);
        assert_eq!(desktop.vcpus, 2);

        let server = VmConfig::server("test-server");
        assert_eq!(server.memory_mb, 1024);
        assert_eq!(server.vcpus, 1);
    }

    #[test]
    fn test_topology_creation() {
        let topology = Topology::federation_2_node();
        assert_eq!(topology.vms.len(), 2);
        assert_eq!(topology.name, "federation-2node");
    }

    #[test]
    fn test_mixed_topology() {
        let topology = Topology::mixed_ecosystem();
        assert_eq!(topology.vms.len(), 3);
    }
}

