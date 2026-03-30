// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Primal coordinator - TRUE PRIMAL approach
//!
//! Coordinates primal interactions (doesn't launch them!)

use anyhow::Result;
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
    #[must_use] 
    pub const fn new(discovery: PrimalDiscovery) -> Self {
        Self { discovery }
    }

    /// Get access to the discovery service
    #[must_use] 
    pub const fn discovery(&self) -> &PrimalDiscovery {
        &self.discovery
    }

    /// Verify required primals are running
    pub async fn verify_primals(&self, required: &[&str]) -> Result<CoordinationStatus> {
        let discovered = self.discovery.discover_all().await?;

        let discovered_names: HashMap<String, &DiscoveredPrimal> =
            discovered.iter().map(|p| (p.name.clone(), p)).collect();

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
    #[must_use] 
    pub fn generate_guide(
        &self,
        atomic_name: &str,
        required_primals: &[&str],
        family_id: &str,
    ) -> DeploymentGuide {
        let start_commands = required_primals
            .iter()
            .map(|&primal| {
                format!("FAMILY_ID={family_id} NODE_ID={atomic_name}-{primal} ./{primal} &")
            })
            .collect();

        let expected_sockets = required_primals
            .iter()
            .map(|&primal| format!("{primal}-{family_id}.sock"))
            .collect();

        DeploymentGuide {
            atomic_name: atomic_name.to_string(),
            required_primals: required_primals.iter().map(|s| s.to_string()).collect(),
            start_commands,
            verification: format!("ls /run/user/$(id -u)/*{family_id}*.sock"),
            expected_sockets,
        }
    }

    /// Coordinate primal introductions (facilitate discovery)
    pub async fn coordinate_introductions(&self, primals: &[DiscoveredPrimal]) -> Result<()> {
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

        // Capability sharing and trust establishment happen via the Neural API
        // when primals register with `lifecycle.register` + `capability.register`.

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coordination_status_ready() {
        let status = CoordinationStatus::Ready;
        let json = serde_json::to_string(&status).expect("serialize");
        assert!(json.contains("Ready"));
    }

    #[test]
    fn test_coordination_status_missing_primals() {
        let status = CoordinationStatus::MissingPrimals(vec!["beardog".to_string()]);
        let json = serde_json::to_string(&status).expect("serialize");
        assert!(json.contains("MissingPrimals"));
        assert!(json.contains("beardog"));
    }

    #[test]
    fn test_coordination_status_unresponsive() {
        let status = CoordinationStatus::Unresponsive(vec!["songbird".to_string()]);
        let json = serde_json::to_string(&status).expect("serialize");
        assert!(json.contains("Unresponsive"));
        assert!(json.contains("songbird"));
    }

    #[test]
    fn test_deployment_guide_serialization() {
        let guide = DeploymentGuide {
            atomic_name: "tower".to_string(),
            required_primals: vec!["beardog".to_string(), "songbird".to_string()],
            start_commands: vec!["./beardog &".to_string()],
            verification: "ls /run/user/1000/*.sock".to_string(),
            expected_sockets: vec!["beardog.sock".to_string()],
        };

        let json = serde_json::to_string(&guide).expect("serialize");
        assert!(json.contains("tower"));
        assert!(json.contains("beardog"));
        assert!(json.contains("verification"));
    }

    #[test]
    fn test_deployment_guide_deserialization() {
        let json = r#"{
            "atomic_name": "nucleus",
            "required_primals": ["beardog", "songbird", "toadstool"],
            "start_commands": ["cmd1", "cmd2"],
            "verification": "verify",
            "expected_sockets": ["a.sock", "b.sock"]
        }"#;

        let guide: DeploymentGuide = serde_json::from_str(json).expect("deserialize");
        assert_eq!(guide.atomic_name, "nucleus");
        assert_eq!(guide.required_primals.len(), 3);
        assert_eq!(guide.start_commands.len(), 2);
    }

    #[test]
    fn test_generate_guide() {
        let temp_dir = tempfile::tempdir().unwrap();
        let discovery = PrimalDiscovery::new(temp_dir.path().to_path_buf()).unwrap();
        let coordinator = PrimalCoordinator::new(discovery);

        let guide = coordinator.generate_guide("tower", &["beardog", "songbird"], "1894e909e454");

        assert_eq!(guide.atomic_name, "tower");
        assert_eq!(guide.required_primals, vec!["beardog", "songbird"]);
        assert_eq!(guide.start_commands.len(), 2);
        assert!(guide.start_commands[0].contains("FAMILY_ID=1894e909e454"));
        assert!(guide.start_commands[0].contains("beardog"));
    }

    #[test]
    fn test_generate_guide_expected_sockets() {
        let temp_dir = tempfile::tempdir().unwrap();
        let discovery = PrimalDiscovery::new(temp_dir.path().to_path_buf()).unwrap();
        let coordinator = PrimalCoordinator::new(discovery);

        let guide = coordinator.generate_guide("node", &["toadstool"], "test-fam");

        assert_eq!(guide.expected_sockets.len(), 1);
        assert!(guide.expected_sockets[0].contains("toadstool"));
        assert!(guide.expected_sockets[0].contains("test-fam"));
    }

    #[test]
    fn test_generate_guide_verification_command() {
        let temp_dir = tempfile::tempdir().unwrap();
        let discovery = PrimalDiscovery::new(temp_dir.path().to_path_buf()).unwrap();
        let coordinator = PrimalCoordinator::new(discovery);

        let guide = coordinator.generate_guide("nest", &["nestgate"], "family123");

        assert!(guide.verification.contains("family123"));
        assert!(guide.verification.contains("ls"));
        assert!(guide.verification.contains("sock"));
    }

    #[test]
    fn test_primal_coordinator_debug() {
        let temp_dir = tempfile::tempdir().unwrap();
        let discovery = PrimalDiscovery::new(temp_dir.path().to_path_buf()).unwrap();
        let coordinator = PrimalCoordinator::new(discovery);

        let debug_str = format!("{coordinator:?}");
        assert!(debug_str.contains("PrimalCoordinator"));
    }

    #[test]
    fn test_coordinator_discovery_access() {
        let temp_dir = tempfile::tempdir().unwrap();
        let discovery = PrimalDiscovery::new(temp_dir.path().to_path_buf()).unwrap();
        let coordinator = PrimalCoordinator::new(discovery);

        // Should be able to access discovery through coordinator
        let _discovery_ref = coordinator.discovery();
    }
}
