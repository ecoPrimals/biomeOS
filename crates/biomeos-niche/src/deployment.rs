//! Niche deployment management
//!
//! Handles deploying and managing running niche instances.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::definition::NicheDefinition;
use crate::error::NicheResult;

/// A deployed niche instance
#[derive(Debug)]
pub struct NicheDeployment {
    /// Unique deployment ID
    pub id: Uuid,

    /// Niche definition
    pub definition: Arc<NicheDefinition>,

    /// Deployment status
    pub status: RwLock<DeploymentStatus>,

    /// Running organism handles
    organisms: RwLock<HashMap<String, OrganismHandle>>,

    /// Deployment directory
    pub deploy_dir: PathBuf,

    /// Created timestamp
    pub created_at: DateTime<Utc>,
}

/// Deployment status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeploymentStatus {
    /// Deployment is being prepared
    Preparing,
    /// Organisms are starting
    Starting,
    /// All organisms are running
    Running,
    /// Deployment is stopping
    Stopping,
    /// Deployment is stopped
    Stopped,
    /// Deployment failed
    Failed,
}

/// Handle to a running organism
#[derive(Debug)]
struct OrganismHandle {
    /// Organism name
    _name: String,
    /// Process ID if running
    _pid: Option<u32>,
    /// Status
    status: OrganismStatus,
}

/// Organism status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // Status variants for future use
enum OrganismStatus {
    Pending,
    Starting,
    Running,
    Stopping,
    Stopped,
    Failed,
}

impl NicheDeployment {
    /// Create a new deployment
    #[must_use]
    pub fn new(definition: Arc<NicheDefinition>, deploy_dir: PathBuf) -> Self {
        Self {
            id: Uuid::new_v4(),
            definition,
            status: RwLock::new(DeploymentStatus::Preparing),
            organisms: RwLock::new(HashMap::new()),
            deploy_dir,
            created_at: Utc::now(),
        }
    }

    /// Get current status
    pub async fn status(&self) -> DeploymentStatus {
        *self.status.read().await
    }

    /// Start the deployment
    ///
    /// # Errors
    /// Returns an error if organisms fail to start.
    pub async fn start(&self) -> NicheResult<()> {
        info!("Starting niche deployment: {}", self.definition.niche.id);

        // Update status
        *self.status.write().await = DeploymentStatus::Starting;

        // Create deploy directory
        std::fs::create_dir_all(&self.deploy_dir)?;

        // Start chimeras first
        for name in self.definition.organisms.chimeras.keys() {
            debug!("Starting chimera: {}", name);
            self.start_organism(name).await?;
        }

        // Then start primals
        for name in self.definition.organisms.primals.keys() {
            debug!("Starting primal: {}", name);
            self.start_organism(name).await?;
        }

        *self.status.write().await = DeploymentStatus::Running;
        info!("Niche deployment running: {}", self.definition.niche.id);

        Ok(())
    }

    /// Start a single organism
    async fn start_organism(&self, name: &str) -> NicheResult<()> {
        let mut organisms = self.organisms.write().await;

        organisms.insert(
            name.to_string(),
            OrganismHandle {
                _name: name.to_string(),
                _pid: None, // Would be set when actually spawning process
                status: OrganismStatus::Running,
            },
        );

        Ok(())
    }

    /// Stop the deployment
    ///
    /// # Errors
    /// Returns an error if organisms fail to stop.
    pub async fn stop(&self) -> NicheResult<()> {
        info!("Stopping niche deployment: {}", self.definition.niche.id);

        *self.status.write().await = DeploymentStatus::Stopping;

        // Stop all organisms
        let mut organisms = self.organisms.write().await;
        for (name, handle) in organisms.iter_mut() {
            debug!("Stopping organism: {}", name);
            handle.status = OrganismStatus::Stopped;
            // Would actually kill process here
        }

        *self.status.write().await = DeploymentStatus::Stopped;
        info!("Niche deployment stopped: {}", self.definition.niche.id);

        Ok(())
    }

    /// Get organism count
    pub async fn organism_count(&self) -> usize {
        self.organisms.read().await.len()
    }

    /// Check if all organisms are healthy
    pub async fn is_healthy(&self) -> bool {
        let organisms = self.organisms.read().await;
        organisms
            .values()
            .all(|h| h.status == OrganismStatus::Running)
    }
}

/// Deployment manager for multiple niches
#[derive(Debug, Default)]
pub struct DeploymentManager {
    /// Active deployments
    deployments: RwLock<HashMap<Uuid, Arc<NicheDeployment>>>,
}

impl DeploymentManager {
    /// Create a new deployment manager
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Deploy a niche
    ///
    /// # Errors
    /// Returns an error if deployment fails.
    pub async fn deploy(
        &self,
        definition: NicheDefinition,
        deploy_dir: PathBuf,
    ) -> NicheResult<Arc<NicheDeployment>> {
        let deployment = Arc::new(NicheDeployment::new(Arc::new(definition), deploy_dir));

        deployment.start().await?;

        self.deployments
            .write()
            .await
            .insert(deployment.id, Arc::clone(&deployment));

        Ok(deployment)
    }

    /// Get a deployment by ID
    pub async fn get(&self, id: Uuid) -> Option<Arc<NicheDeployment>> {
        self.deployments.read().await.get(&id).cloned()
    }

    /// Stop and remove a deployment
    ///
    /// # Errors
    /// Returns an error if the deployment fails to stop.
    pub async fn undeploy(&self, id: Uuid) -> NicheResult<()> {
        if let Some(deployment) = self.deployments.write().await.remove(&id) {
            deployment.stop().await?;
        }
        Ok(())
    }

    /// List all deployments
    pub async fn list(&self) -> Vec<(Uuid, String, DeploymentStatus)> {
        let deployments = self.deployments.read().await;
        let mut result = Vec::new();

        for (id, deployment) in deployments.iter() {
            result.push((
                *id,
                deployment.definition.niche.id.clone(),
                deployment.status().await,
            ));
        }

        result
    }

    /// Stop all deployments
    ///
    /// # Errors
    /// Returns an error if any deployment fails to stop.
    pub async fn stop_all(&self) -> NicheResult<()> {
        let deployments = self.deployments.read().await;
        for deployment in deployments.values() {
            if let Err(e) = deployment.stop().await {
                warn!("Failed to stop deployment {}: {}", deployment.id, e);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::definition::{NicheMetadata, NicheNetworking, NicheResources, NicheSecurity};
    use crate::organism::{ChimeraOrganism, OrganismSpec};
    use tempfile::TempDir;

    fn create_test_niche() -> NicheDefinition {
        NicheDefinition {
            niche: NicheMetadata {
                id: "test-niche".into(),
                name: "Test Niche".into(),
                version: "1.0.0".into(),
                description: "Test".into(),
                category: String::new(),
                difficulty: String::new(),
                author: String::new(),
                features: Vec::new(),
            },
            organisms: OrganismSpec::new().with_chimera("mesh", ChimeraOrganism::new("p2p-secure")),
            interactions: Vec::new(),
            customization: Vec::new(),
            resources: NicheResources::default(),
            networking: NicheNetworking::default(),
            security: NicheSecurity::default(),
        }
    }

    #[tokio::test]
    async fn test_deployment_lifecycle() {
        let temp_dir = TempDir::new().unwrap();
        let definition = create_test_niche();

        let deployment = NicheDeployment::new(Arc::new(definition), temp_dir.path().to_path_buf());

        assert_eq!(deployment.status().await, DeploymentStatus::Preparing);

        deployment.start().await.unwrap();
        assert_eq!(deployment.status().await, DeploymentStatus::Running);
        assert_eq!(deployment.organism_count().await, 1);

        deployment.stop().await.unwrap();
        assert_eq!(deployment.status().await, DeploymentStatus::Stopped);
    }
}
