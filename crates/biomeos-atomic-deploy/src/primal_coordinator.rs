//! Primal coordinator - TRUE PRIMAL approach
//!
//! Coordinates primal interactions (doesn't launch them!)

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info};

use crate::primal_discovery::{DiscoveredPrimal, PrimalDiscovery};

/// Primal coordination status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CoordinationStatus {
    /// All primals discovered and ready
    Ready,
    
    /// Missing required primals
    MissingPrimals(Vec<String>),
    
    /// Primals found but not responsive
    Unresponsive(Vec<String>),
}

/// Deployment guide for users
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentGuide {
    /// Atomic type being deployed
    pub atomic_name: String,
    
    /// Required primals
    pub required_primals: Vec<String>,
    
    /// Commands to start missing primals
    pub start_commands: Vec<String>,
    
    /// Verification command
    pub verification: String,
    
    /// Expected Unix sockets
    pub expected_sockets: Vec<String>,
}

/// Primal coordinator - guides deployment, doesn't launch!
#[derive(Debug)]
pub struct PrimalCoordinator {
    discovery: PrimalDiscovery,
}

impl PrimalCoordinator {
    /// Create new coordinator
    pub fn new(discovery: PrimalDiscovery) -> Self {
        Self { discovery }
    }
    
    /// Get access to the discovery service
    pub fn discovery(&self) -> &PrimalDiscovery {
        &self.discovery
    }
    
    /// Verify required primals are running
    pub async fn verify_primals(&self, required: &[&str]) -> Result<CoordinationStatus> {
        let discovered = self.discovery.discover_all().await?;
        
        let discovered_names: HashMap<String, &DiscoveredPrimal> = discovered
            .iter()
            .map(|p| (p.name.clone(), p))
            .collect();
        
        // Check for missing primals
        let mut missing = Vec::new();
        let mut unresponsive = Vec::new();
        
        for &primal_name in required {
            match discovered_names.get(primal_name) {
                Some(primal) => {
                    if !primal.responsive {
                        unresponsive.push(primal_name.to_string());
                    }
                }
                None => {
                    missing.push(primal_name.to_string());
                }
            }
        }
        
        if !missing.is_empty() {
            Ok(CoordinationStatus::MissingPrimals(missing))
        } else if !unresponsive.is_empty() {
            Ok(CoordinationStatus::Unresponsive(unresponsive))
        } else {
            Ok(CoordinationStatus::Ready)
        }
    }
    
    /// Generate deployment guide for missing primals
    pub fn generate_guide(
        &self,
        atomic_name: &str,
        required_primals: &[&str],
        family_id: &str,
    ) -> DeploymentGuide {
        let start_commands = required_primals
            .iter()
            .map(|&primal| {
                format!(
                    "FAMILY_ID={} NODE_ID={}-{} ./{} &",
                    family_id,
                    atomic_name,
                    primal,
                    primal
                )
            })
            .collect();
        
        let expected_sockets = required_primals
            .iter()
            .map(|&primal| format!("{}-{}.sock", primal, family_id))
            .collect();
        
        DeploymentGuide {
            atomic_name: atomic_name.to_string(),
            required_primals: required_primals.iter().map(|s| s.to_string()).collect(),
            start_commands,
            verification: format!(
                "ls /run/user/$(id -u)/*{}*.sock",
                family_id
            ),
            expected_sockets,
        }
    }
    
    /// Coordinate primal introductions (facilitate discovery)
    pub async fn coordinate_introductions(
        &self,
        primals: &[DiscoveredPrimal],
    ) -> Result<()> {
        info!("Coordinating introductions for {} primals", primals.len());
        
        // In TRUE PRIMAL approach, primals discover each other
        // The coordinator just facilitates by ensuring they're all running
        // and can communicate
        
        for primal in primals {
            debug!(
                "Primal {} available at {}",
                primal.name,
                primal.socket_path.display()
            );
        }
        
        // Future: Could implement capability sharing, trust establishment, etc.
        // For now, just verify they're all responsive
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    
    #[test]
    fn test_generate_guide() {
        let temp_dir = tempfile::tempdir().unwrap();
        let discovery = PrimalDiscovery::new(temp_dir.path().to_path_buf()).unwrap();
        let coordinator = PrimalCoordinator::new(discovery);
        
        let guide = coordinator.generate_guide(
            "tower",
            &["beardog", "songbird"],
            "nat0",
        );
        
        assert_eq!(guide.atomic_name, "tower");
        assert_eq!(guide.required_primals, vec!["beardog", "songbird"]);
        assert_eq!(guide.start_commands.len(), 2);
        assert!(guide.start_commands[0].contains("FAMILY_ID=nat0"));
        assert!(guide.start_commands[0].contains("beardog"));
    }
}

