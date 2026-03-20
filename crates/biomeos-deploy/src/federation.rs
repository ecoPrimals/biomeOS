// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Federation deployment and management

use crate::error::{DeployError, Result};
use crate::health::{HealthCheck, VmHealth};
use crate::network::{BridgeConfig, NetworkBridge};
use crate::qemu::{QemuConfig, QemuInstance};
use crate::topology::{Topology, VmTopology};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{info, warn};

/// Federation configuration
#[derive(Debug, Clone)]
pub struct FederationConfig {
    /// Topology definition
    pub topology: Topology,

    /// Enable KVM acceleration
    pub enable_kvm: bool,

    /// Health check timeout (seconds)
    pub health_check_timeout: u64,

    /// Wait for VMs to be healthy before returning
    pub wait_for_healthy: bool,
}

/// Federation deployment manager
pub struct Federation {
    config: FederationConfig,
    network: Option<NetworkBridge>,
    vms: HashMap<String, QemuInstance>,
}

impl Federation {
    /// Create a new federation manager
    pub fn new(config: FederationConfig) -> Self {
        Self {
            config,
            network: None,
            vms: HashMap::new(),
        }
    }

    /// Deploy the complete federation
    pub async fn deploy(&mut self) -> Result<()> {
        info!(
            "🚀 Deploying federation: {}",
            self.config.topology.metadata.name
        );

        // Step 1: Setup network bridge
        self.setup_network().await?;

        // Step 2: Prepare disk images (backing images)
        self.prepare_disks().await?;

        // Step 3: Start all VMs
        self.start_vms().await?;

        // Step 4: Health checks
        if self.config.wait_for_healthy {
            self.wait_for_all_healthy().await?;
        }

        info!(
            "✅ Federation {} deployed successfully with {} VMs",
            self.config.topology.metadata.name,
            self.vms.len()
        );

        Ok(())
    }

    /// Setup network bridge
    async fn setup_network(&mut self) -> Result<()> {
        let bridge_config = BridgeConfig {
            name: self.config.topology.network.bridge_name.clone(),
            ip_address: self.config.topology.network.bridge_ip.clone(),
            subnet: self.config.topology.network.subnet.clone(),
        };

        let mut bridge = NetworkBridge::new(bridge_config);
        bridge.create().await?;
        self.network = Some(bridge);
        Ok(())
    }

    /// Prepare disk images (create backing images)
    async fn prepare_disks(&self) -> Result<()> {
        info!("📦 Preparing disk images...");

        for vm in &self.config.topology.vms {
            // Check if disk image already exists
            if vm.disk_image.exists() {
                info!("  ✓ Disk {} exists", vm.disk_image.display());
                continue;
            }

            // If it doesn't exist, assume it needs a backing image
            // For now, we'll just error - in production we'd create backing images
            return Err(DeployError::FileSystem {
                message: format!(
                    "Disk image not found: {}. Please create it first.",
                    vm.disk_image.display()
                ),
            });
        }

        Ok(())
    }

    /// Start all VMs
    async fn start_vms(&mut self) -> Result<()> {
        info!("🚀 Starting {} VMs...", self.config.topology.vms.len());

        for vm_topology in &self.config.topology.vms {
            let qemu_config = self.vm_topology_to_qemu_config(vm_topology)?;
            let mut vm = QemuInstance::new(qemu_config);
            vm.start().await?;
            self.vms.insert(vm_topology.name.clone(), vm);
        }

        Ok(())
    }

    /// Convert VM topology to QEMU config (requires network)
    pub(crate) fn vm_topology_to_qemu_config(&self, vm: &VmTopology) -> Result<QemuConfig> {
        let bridge_name = self
            .network
            .as_ref()
            .ok_or_else(|| DeployError::QemuConfig {
                message: "Network bridge not initialized".to_string(),
            })?
            .name()
            .to_string();

        Ok(QemuConfig {
            name: vm.name.clone(),
            memory: vm.memory,
            cpus: vm.cpus,
            disk_image: vm.disk_image.clone(),
            bridge_name,
            mac_address: vm.mac_address.clone(),
            serial_log: vm.serial_log.clone(),
            enable_kvm: self.config.enable_kvm,
            extra_args: vec![],
        })
    }

    /// Wait for all VMs to be healthy
    async fn wait_for_all_healthy(&self) -> Result<()> {
        info!("🔍 Waiting for all VMs to become healthy...");

        let timeout = std::time::Duration::from_secs(self.config.health_check_timeout);

        for (name, vm) in &self.vms {
            HealthCheck::wait_for_healthy(name, vm.serial_log_path(), timeout).await?;
        }

        Ok(())
    }

    /// Check health of all VMs
    pub async fn health_check(&self) -> Result<Vec<VmHealth>> {
        let mut results = Vec::new();

        for (name, vm) in &self.vms {
            let health = HealthCheck::check_vm(name, vm.serial_log_path()).await?;
            results.push(health);
        }

        Ok(results)
    }

    /// Gracefully shutdown the federation
    pub async fn shutdown(&mut self) -> Result<()> {
        info!(
            "🛑 Shutting down federation: {}",
            self.config.topology.metadata.name
        );

        // Step 1: Stop all VMs
        for (name, vm) in &mut self.vms {
            info!("Stopping VM: {}", name);
            if let Err(e) = vm.stop().await {
                warn!("Failed to stop VM {}: {}", name, e);
            }
        }
        self.vms.clear();

        // Step 2: Destroy network (only if we created it)
        if let Some(ref mut network) = self.network {
            network.destroy().await?;
        }
        self.network = None;

        info!(
            "✅ Federation {} shutdown complete",
            self.config.topology.metadata.name
        );

        Ok(())
    }

    /// Get number of running VMs
    pub fn vm_count(&mut self) -> usize {
        let mut count = 0;
        for vm in self.vms.values_mut() {
            if vm.is_running() {
                count += 1;
            }
        }
        count
    }

    /// Get VM by name
    pub fn get_vm(&self, name: &str) -> Option<&QemuInstance> {
        self.vms.get(name)
    }

    #[cfg(test)]
    pub(crate) fn set_network_for_test(&mut self, network: NetworkBridge) {
        self.network = Some(network);
    }
}

/// Federation status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationStatus {
    /// Federation name
    pub name: String,
    /// Total number of VMs in the federation
    pub total_vms: usize,
    /// Number of currently running VMs
    pub running_vms: usize,
    /// Health status of each VM
    pub vm_health: Vec<VmHealth>,
}

#[expect(
    clippy::unwrap_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
#[expect(
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::health::{HealthStatus, VmHealth};
    use crate::topology::{NetworkTopology, TopologyMetadata};
    use std::collections::HashMap;
    use std::path::PathBuf;

    fn sample_topology() -> Topology {
        Topology {
            metadata: TopologyMetadata {
                name: "test-federation".to_string(),
                version: "1.0".to_string(),
                description: "Test topology".to_string(),
            },
            network: NetworkTopology {
                bridge_name: "biomeos-br0".to_string(),
                bridge_ip: "10.0.0.1/24".to_string(),
                subnet: "10.0.0.0/24".to_string(),
            },
            vms: vec![VmTopology {
                name: "vm1".to_string(),
                memory: 2048,
                cpus: 2,
                disk_image: PathBuf::from("/tmp/vm1.qcow2"),
                ip_address: "10.0.0.10".to_string(),
                mac_address: "52:54:00:00:00:01".to_string(),
                serial_log: PathBuf::from("/tmp/vm1.log"),
                options: HashMap::new(),
            }],
        }
    }

    #[test]
    fn test_federation_config_construction() {
        let topology = sample_topology();
        let config = FederationConfig {
            topology,
            enable_kvm: true,
            health_check_timeout: 60,
            wait_for_healthy: false,
        };
        assert_eq!(config.topology.metadata.name, "test-federation");
        assert!(config.enable_kvm);
        assert_eq!(config.health_check_timeout, 60);
    }

    #[test]
    fn test_federation_new() {
        let config = FederationConfig {
            topology: sample_topology(),
            enable_kvm: false,
            health_check_timeout: 30,
            wait_for_healthy: true,
        };
        let federation = Federation::new(config);
        assert!(federation.get_vm("vm1").is_none());
    }

    #[test]
    fn test_federation_get_vm_empty() {
        let config = FederationConfig {
            topology: sample_topology(),
            enable_kvm: false,
            health_check_timeout: 30,
            wait_for_healthy: false,
        };
        let federation = Federation::new(config);
        assert!(federation.get_vm("nonexistent").is_none());
        assert!(federation.get_vm("vm1").is_none());
    }

    #[test]
    fn test_federation_vm_count_empty() {
        let config = FederationConfig {
            topology: sample_topology(),
            enable_kvm: false,
            health_check_timeout: 30,
            wait_for_healthy: false,
        };
        let mut federation = Federation::new(config);
        assert_eq!(federation.vm_count(), 0);
    }

    #[test]
    fn test_federation_status_serialization() {
        let status = FederationStatus {
            name: "test".to_string(),
            total_vms: 2,
            running_vms: 1,
            vm_health: vec![VmHealth {
                vm_name: "vm1".to_string(),
                status: HealthStatus::Healthy,
                last_message_time: None,
                boot_completed: true,
                error: None,
            }],
        };
        let json = serde_json::to_string(&status).expect("serialization should succeed");
        assert!(json.contains("\"name\":\"test\""));
        assert!(json.contains("\"total_vms\":2"));
        let deserialized: FederationStatus =
            serde_json::from_str(&json).expect("deserialization should succeed");
        assert_eq!(deserialized.name, status.name);
        assert_eq!(deserialized.total_vms, status.total_vms);
    }

    #[test]
    fn test_vm_topology_to_qemu_config_error_no_network() {
        let topology = sample_topology();
        let vm = topology.vms[0].clone();
        let config = FederationConfig {
            topology,
            enable_kvm: false,
            health_check_timeout: 30,
            wait_for_healthy: false,
        };
        let federation = Federation::new(config);
        let result = federation.vm_topology_to_qemu_config(&vm);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, DeployError::QemuConfig { .. }));
        assert!(err.to_string().contains("Network bridge not initialized"));
    }

    #[tokio::test]
    async fn test_federation_health_check_empty() {
        let config = FederationConfig {
            topology: sample_topology(),
            enable_kvm: false,
            health_check_timeout: 30,
            wait_for_healthy: false,
        };
        let federation = Federation::new(config);
        let results = federation.get_vm("vm1");
        assert!(results.is_none());
        let health_results = federation
            .health_check()
            .await
            .expect("health_check should succeed");
        assert!(health_results.is_empty());
    }

    #[tokio::test]
    async fn test_federation_shutdown_empty() {
        let config = FederationConfig {
            topology: sample_topology(),
            enable_kvm: false,
            health_check_timeout: 30,
            wait_for_healthy: false,
        };
        let mut federation = Federation::new(config);
        federation
            .shutdown()
            .await
            .expect("shutdown should succeed");
    }

    #[test]
    fn test_federation_config_clone() {
        let config = FederationConfig {
            topology: sample_topology(),
            enable_kvm: true,
            health_check_timeout: 90,
            wait_for_healthy: true,
        };
        let cloned = config.clone();
        assert_eq!(cloned.enable_kvm, config.enable_kvm);
        assert_eq!(cloned.health_check_timeout, 90);
    }

    #[test]
    fn test_federation_status_deserialize() {
        let json = r#"{"name":"test","total_vms":2,"running_vms":1,"vm_health":[]}"#;
        let status: FederationStatus = serde_json::from_str(json).expect("parse");
        assert_eq!(status.name, "test");
        assert_eq!(status.total_vms, 2);
        assert_eq!(status.running_vms, 1);
    }

    #[tokio::test]
    async fn test_federation_deploy_fails_without_disk_or_network() {
        let mut topology = sample_topology();
        topology.vms[0].disk_image = PathBuf::from("/nonexistent/vm1.qcow2");
        let config = FederationConfig {
            topology,
            enable_kvm: false,
            health_check_timeout: 5,
            wait_for_healthy: false,
        };
        let mut federation = Federation::new(config);
        let result = federation.deploy().await;
        assert!(
            result.is_err(),
            "deploy should fail without disk or without root for bridge"
        );
    }

    #[test]
    fn test_vm_topology_to_qemu_config_with_network() {
        let topology = sample_topology();
        let bridge_config = BridgeConfig {
            name: topology.network.bridge_name.clone(),
            ip_address: topology.network.bridge_ip.clone(),
            subnet: topology.network.subnet.clone(),
        };
        let mut federation = Federation::new(FederationConfig {
            topology: topology.clone(),
            enable_kvm: false,
            health_check_timeout: 30,
            wait_for_healthy: false,
        });
        federation.set_network_for_test(NetworkBridge::new(bridge_config));
        let vm = &topology.vms[0];
        let qemu_config = federation.vm_topology_to_qemu_config(vm).expect("config");
        assert_eq!(qemu_config.name, vm.name);
        assert_eq!(qemu_config.memory, vm.memory);
        assert_eq!(qemu_config.cpus, vm.cpus);
        assert_eq!(qemu_config.disk_image, vm.disk_image);
        assert_eq!(qemu_config.mac_address, vm.mac_address);
    }

    #[test]
    fn test_federation_status_clone_and_debug() {
        let s = FederationStatus {
            name: "fed".to_string(),
            total_vms: 1,
            running_vms: 0,
            vm_health: vec![],
        };
        let c = s.clone();
        assert_eq!(c.name, "fed");
        let d = format!("{s:?}");
        assert!(d.contains("FederationStatus"));
    }

    #[test]
    fn test_federation_config_wait_for_healthy_flag() {
        let topology = sample_topology();
        let cfg = FederationConfig {
            topology,
            enable_kvm: true,
            health_check_timeout: 120,
            wait_for_healthy: true,
        };
        assert!(cfg.wait_for_healthy);
        assert_eq!(cfg.health_check_timeout, 120);
    }
}
