// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Topology definition and parsing

use crate::error::{DeployError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Complete federation topology
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Topology {
    /// Topology metadata
    pub metadata: TopologyMetadata,

    /// Network configuration
    pub network: NetworkTopology,

    /// VM instances
    pub vms: Vec<VmTopology>,
}

/// Topology metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopologyMetadata {
    /// Topology name
    pub name: String,

    /// Version
    pub version: String,

    /// Description
    #[serde(default)]
    pub description: String,
}

/// Network topology configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkTopology {
    /// Bridge name
    pub bridge_name: String,

    /// Bridge IP address
    pub bridge_ip: String,

    /// Subnet configuration
    pub subnet: String,
}

/// Individual VM topology
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VmTopology {
    /// VM name
    pub name: String,

    /// Memory size (MB)
    pub memory: u32,

    /// Number of CPUs
    pub cpus: u32,

    /// Disk image path
    pub disk_image: PathBuf,

    /// IP address
    pub ip_address: String,

    /// MAC address
    pub mac_address: String,

    /// Serial log path
    pub serial_log: PathBuf,

    /// Additional options
    #[serde(default)]
    pub options: HashMap<String, String>,
}

impl Topology {
    /// Load topology from YAML file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path_ref = path.as_ref();
        let contents = std::fs::read_to_string(path_ref).map_err(|e| DeployError::FileSystem {
            message: format!("Failed to read topology file {}: {}", path_ref.display(), e),
        })?;

        let topology: Self =
            serde_yaml::from_str(&contents).map_err(|e| DeployError::TopologyParse {
                path: path_ref.to_path_buf(),
                source: e,
            })?;

        topology.validate()?;
        Ok(topology)
    }

    /// Validate topology configuration
    pub fn validate(&self) -> Result<()> {
        // Check VM names are unique
        let mut names = std::collections::HashSet::new();
        for vm in &self.vms {
            if !names.insert(&vm.name) {
                return Err(DeployError::TopologyValidation {
                    message: format!("Duplicate VM name: {}", vm.name),
                });
            }
        }

        // Check IP addresses are unique
        let mut ips = std::collections::HashSet::new();
        for vm in &self.vms {
            if !ips.insert(&vm.ip_address) {
                return Err(DeployError::TopologyValidation {
                    message: format!("Duplicate IP address: {}", vm.ip_address),
                });
            }
        }

        // Check MAC addresses are unique
        let mut macs = std::collections::HashSet::new();
        for vm in &self.vms {
            if !macs.insert(&vm.mac_address) {
                return Err(DeployError::TopologyValidation {
                    message: format!("Duplicate MAC address: {}", vm.mac_address),
                });
            }
        }

        Ok(())
    }

    /// Get VM by name
    #[must_use]
    pub fn get_vm(&self, name: &str) -> Option<&VmTopology> {
        self.vms.iter().find(|vm| vm.name == name)
    }

    /// Get number of VMs
    #[must_use]
    pub const fn vm_count(&self) -> usize {
        self.vms.len()
    }
}

#[expect(
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_topology_validation() {
        let topology = Topology {
            metadata: TopologyMetadata {
                name: "test".to_string(),
                version: "1.0".to_string(),
                description: "Test topology".to_string(),
            },
            network: NetworkTopology {
                bridge_name: "biomeos-br0".to_string(),
                bridge_ip: "10.0.0.1/24".to_string(),
                subnet: "10.0.0.0/24".to_string(),
            },
            vms: vec![
                VmTopology {
                    name: "vm1".to_string(),
                    memory: 2048,
                    cpus: 2,
                    disk_image: PathBuf::from("vm1.qcow2"),
                    ip_address: "10.0.0.10".to_string(),
                    mac_address: "52:54:00:00:00:01".to_string(),
                    serial_log: PathBuf::from("vm1.log"),
                    options: HashMap::new(),
                },
                VmTopology {
                    name: "vm2".to_string(),
                    memory: 2048,
                    cpus: 2,
                    disk_image: PathBuf::from("vm2.qcow2"),
                    ip_address: "10.0.0.11".to_string(),
                    mac_address: "52:54:00:00:00:02".to_string(),
                    serial_log: PathBuf::from("vm2.log"),
                    options: HashMap::new(),
                },
            ],
        };

        assert!(topology.validate().is_ok());
        assert_eq!(topology.vm_count(), 2);
        assert!(topology.get_vm("vm1").is_some());
    }

    #[test]
    fn test_topology_validation_duplicate_vm_name() {
        let topology = Topology {
            metadata: TopologyMetadata {
                name: "test".to_string(),
                version: "1.0".to_string(),
                description: String::new(),
            },
            network: NetworkTopology {
                bridge_name: "br0".to_string(),
                bridge_ip: "10.0.0.1/24".to_string(),
                subnet: "10.0.0.0/24".to_string(),
            },
            vms: vec![
                VmTopology {
                    name: "vm1".to_string(),
                    memory: 1024,
                    cpus: 1,
                    disk_image: PathBuf::from("a.qcow2"),
                    ip_address: "10.0.0.10".to_string(),
                    mac_address: "52:54:00:00:00:01".to_string(),
                    serial_log: PathBuf::from("a.log"),
                    options: HashMap::new(),
                },
                VmTopology {
                    name: "vm1".to_string(),
                    memory: 1024,
                    cpus: 1,
                    disk_image: PathBuf::from("b.qcow2"),
                    ip_address: "10.0.0.11".to_string(),
                    mac_address: "52:54:00:00:00:02".to_string(),
                    serial_log: PathBuf::from("b.log"),
                    options: HashMap::new(),
                },
            ],
        };
        let err = topology.validate().expect_err("duplicate name should fail");
        assert!(err.to_string().contains("Duplicate VM name"));
    }

    #[test]
    fn test_topology_validation_duplicate_ip() {
        let topology = Topology {
            metadata: TopologyMetadata {
                name: "test".to_string(),
                version: "1.0".to_string(),
                description: String::new(),
            },
            network: NetworkTopology {
                bridge_name: "br0".to_string(),
                bridge_ip: "10.0.0.1/24".to_string(),
                subnet: "10.0.0.0/24".to_string(),
            },
            vms: vec![
                VmTopology {
                    name: "vm1".to_string(),
                    memory: 1024,
                    cpus: 1,
                    disk_image: PathBuf::from("a.qcow2"),
                    ip_address: "10.0.0.10".to_string(),
                    mac_address: "52:54:00:00:00:01".to_string(),
                    serial_log: PathBuf::from("a.log"),
                    options: HashMap::new(),
                },
                VmTopology {
                    name: "vm2".to_string(),
                    memory: 1024,
                    cpus: 1,
                    disk_image: PathBuf::from("b.qcow2"),
                    ip_address: "10.0.0.10".to_string(),
                    mac_address: "52:54:00:00:00:02".to_string(),
                    serial_log: PathBuf::from("b.log"),
                    options: HashMap::new(),
                },
            ],
        };
        let err = topology.validate().expect_err("duplicate IP should fail");
        assert!(err.to_string().contains("Duplicate IP address"));
    }

    #[test]
    fn test_topology_validation_duplicate_mac() {
        let topology = Topology {
            metadata: TopologyMetadata {
                name: "test".to_string(),
                version: "1.0".to_string(),
                description: String::new(),
            },
            network: NetworkTopology {
                bridge_name: "br0".to_string(),
                bridge_ip: "10.0.0.1/24".to_string(),
                subnet: "10.0.0.0/24".to_string(),
            },
            vms: vec![
                VmTopology {
                    name: "vm1".to_string(),
                    memory: 1024,
                    cpus: 1,
                    disk_image: PathBuf::from("a.qcow2"),
                    ip_address: "10.0.0.10".to_string(),
                    mac_address: "52:54:00:00:00:01".to_string(),
                    serial_log: PathBuf::from("a.log"),
                    options: HashMap::new(),
                },
                VmTopology {
                    name: "vm2".to_string(),
                    memory: 1024,
                    cpus: 1,
                    disk_image: PathBuf::from("b.qcow2"),
                    ip_address: "10.0.0.11".to_string(),
                    mac_address: "52:54:00:00:00:01".to_string(),
                    serial_log: PathBuf::from("b.log"),
                    options: HashMap::new(),
                },
            ],
        };
        let err = topology.validate().expect_err("duplicate MAC should fail");
        assert!(err.to_string().contains("Duplicate MAC address"));
    }

    #[test]
    fn test_topology_get_vm_missing() {
        let topology = Topology {
            metadata: TopologyMetadata {
                name: "test".to_string(),
                version: "1.0".to_string(),
                description: String::new(),
            },
            network: NetworkTopology {
                bridge_name: "br0".to_string(),
                bridge_ip: "10.0.0.1/24".to_string(),
                subnet: "10.0.0.0/24".to_string(),
            },
            vms: vec![],
        };
        assert!(topology.get_vm("nonexistent").is_none());
        assert_eq!(topology.vm_count(), 0);
    }

    #[test]
    fn test_topology_from_file() {
        let temp = tempfile::tempdir().expect("temp dir");
        let yaml = r#"
metadata:
  name: file-test
  version: "1.0"
  description: From file
network:
  bridge_name: br0
  bridge_ip: 10.0.0.1/24
  subnet: 10.0.0.0/24
vms:
  - name: vm1
    memory: 2048
    cpus: 2
    disk_image: vm1.qcow2
    ip_address: 10.0.0.10
    mac_address: 52:54:00:00:00:01
    serial_log: vm1.log
"#;
        let path = temp.path().join("topology.yaml");
        std::fs::write(&path, yaml).expect("write yaml");
        let topology = Topology::from_file(&path).expect("load topology");
        assert_eq!(topology.metadata.name, "file-test");
        assert_eq!(topology.vm_count(), 1);
        assert!(topology.get_vm("vm1").is_some());
    }

    #[test]
    fn test_topology_metadata_default_description() {
        let json = r#"{"metadata":{"name":"x","version":"1.0"},"network":{"bridge_name":"br0","bridge_ip":"10.0.0.1/24","subnet":"10.0.0.0/24"},"vms":[]}"#;
        let topology: Topology = serde_json::from_str(json).expect("parse");
        assert_eq!(topology.metadata.description, "");
    }
}
