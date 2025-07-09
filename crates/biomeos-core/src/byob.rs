//! BYOB (Bring Your Own Biome) functionality
//! 
//! This module enables teams to deploy independently while leveraging shared Primal infrastructure.
//! Teams maintain sovereignty while benefiting from network effects.

use std::collections::HashMap;
use std::sync::Arc;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{BiomeResult, BiomeError, BiomeManifest, PrimalType, HealthStatus, BiomeOSConfig};

/// Team workspace with resource isolation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamWorkspace {
    pub team_id: String,
    pub created_at: DateTime<Utc>,
    pub resource_quota: ResourceQuota,
    pub active_deployments: Vec<String>,  // Deployment IDs
    pub isolation_config: IsolationConfig,
}

/// Resource quota for team isolation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceQuota {
    pub max_cpu_cores: f64,
    pub max_memory_bytes: u64,
    pub max_storage_bytes: u64,
    pub max_network_bandwidth_mbps: u64,
    pub max_deployments: u32,
}

/// Isolation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IsolationConfig {
    pub network_isolation: bool,
    pub resource_isolation: bool,
    pub secret_isolation: bool,
}

/// Deployment instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentInstance {
    pub deployment_id: String,
    pub biome_manifest: BiomeManifest,
    pub status: DeploymentStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub team_id: String,
    pub primal_assignments: HashMap<String, PrimalType>,
}

/// Deployment status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeploymentStatus {
    Pending,
    Deploying,
    Running,
    Scaling,
    Stopping,
    Stopped,
    Failed(String),
}

/// BYOB deployment manager
#[derive(Debug)]
pub struct ByobDeploymentManager {
    workspaces: Arc<std::sync::Mutex<HashMap<String, TeamWorkspace>>>,
    deployments: Arc<std::sync::Mutex<HashMap<String, DeploymentInstance>>>,
    config: BiomeOSConfig,
}

impl ByobDeploymentManager {
    /// Create new BYOB deployment manager
    pub fn new(config: BiomeOSConfig) -> Self {
        Self {
            workspaces: Arc::new(std::sync::Mutex::new(HashMap::new())),
            deployments: Arc::new(std::sync::Mutex::new(HashMap::new())),
            config,
        }
    }

    /// Create team workspace
    pub fn create_team_workspace(&self, team_id: &str) -> BiomeResult<TeamWorkspace> {
        let workspace = TeamWorkspace {
            team_id: team_id.to_string(),
            created_at: Utc::now(),
            resource_quota: ResourceQuota::default(),
            active_deployments: Vec::new(),
            isolation_config: IsolationConfig::default(),
        };

        let mut workspaces = self.workspaces.lock().unwrap();
        workspaces.insert(team_id.to_string(), workspace.clone());

        Ok(workspace)
    }

    /// Get team workspace
    pub fn get_team_workspace(&self, team_id: &str) -> BiomeResult<TeamWorkspace> {
        let workspaces = self.workspaces.lock().unwrap();
        workspaces.get(team_id)
            .cloned()
            .ok_or_else(|| BiomeError::Generic { 
                message: format!("Team workspace not found: {}", team_id) 
            })
    }

    /// Deploy biome for team
    pub fn deploy_biome(&self, team_id: &str, manifest: BiomeManifest) -> BiomeResult<String> {
        let deployment_id = Uuid::new_v4().to_string();
        
        // Get or create team workspace
        let _workspace = match self.get_team_workspace(team_id) {
            Ok(ws) => ws,
            Err(_) => self.create_team_workspace(team_id)?,
        };

        // Create deployment
        let deployment = DeploymentInstance {
            deployment_id: deployment_id.clone(),
            biome_manifest: manifest,
            status: DeploymentStatus::Pending,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            team_id: team_id.to_string(),
            primal_assignments: HashMap::new(),
        };

        // Store deployment
        let mut deployments = self.deployments.lock().unwrap();
        deployments.insert(deployment_id.clone(), deployment);

        Ok(deployment_id)
    }

    /// List team deployments
    pub fn list_team_deployments(&self, team_id: &str) -> BiomeResult<Vec<DeploymentInstance>> {
        let deployments = self.deployments.lock().unwrap();
        let team_deployments: Vec<DeploymentInstance> = deployments
            .values()
            .filter(|d| d.team_id == team_id)
            .cloned()
            .collect();

        Ok(team_deployments)
    }

    /// Get deployment status
    pub fn get_deployment_status(&self, deployment_id: &str) -> BiomeResult<DeploymentStatus> {
        let deployments = self.deployments.lock().unwrap();
        deployments.get(deployment_id)
            .map(|d| d.status.clone())
            .ok_or_else(|| BiomeError::Generic { 
                message: format!("Deployment not found: {}", deployment_id) 
            })
    }

    /// Remove deployment
    pub fn remove_deployment(&self, deployment_id: &str) -> BiomeResult<()> {
        let mut deployments = self.deployments.lock().unwrap();
        deployments.remove(deployment_id)
            .ok_or_else(|| BiomeError::Generic { 
                message: format!("Deployment not found: {}", deployment_id) 
            })?;

        Ok(())
    }
}

impl Default for ResourceQuota {
    fn default() -> Self {
        Self {
            max_cpu_cores: 16.0,
            max_memory_bytes: 68719476736, // 64GB
            max_storage_bytes: 549755813888, // 512GB
            max_network_bandwidth_mbps: 1000,
            max_deployments: 5,
        }
    }
}

impl Default for IsolationConfig {
    fn default() -> Self {
        Self {
            network_isolation: true,
            resource_isolation: true,
            secret_isolation: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_byob_deployment_manager_creation() {
        let config = BiomeOSConfig::default();
        let manager = ByobDeploymentManager::new(config);
        
        let workspace = manager.create_team_workspace("test-team").unwrap();
        assert_eq!(workspace.team_id, "test-team");
        assert_eq!(workspace.resource_quota.max_cpu_cores, 16.0);
    }

    #[test]
    fn test_team_workspace_isolation() {
        let config = BiomeOSConfig::default();
        let manager = ByobDeploymentManager::new(config);
        
        let workspace1 = manager.create_team_workspace("team1").unwrap();
        let workspace2 = manager.create_team_workspace("team2").unwrap();
        
        assert_ne!(workspace1.team_id, workspace2.team_id);
        assert!(workspace1.isolation_config.network_isolation);
        assert!(workspace2.isolation_config.resource_isolation);
    }
} 