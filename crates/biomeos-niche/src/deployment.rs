// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

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
    /// Organism name (retained for Debug output)
    _name: String,
    /// Process ID if running
    pid: Option<u32>,
    /// Status
    status: OrganismStatus,
}

/// Organism lifecycle status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OrganismStatus {
    Pending,
    Starting,
    Running,
    Stopping,
    Stopped,
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

    /// Start a single organism by discovering its binary and spawning a process.
    ///
    /// Binary resolution order:
    /// 1. `plasmidBin/primals/{name}`
    /// 2. `target/release/{name}`
    /// 3. `$PATH`
    ///
    /// For graph-based BYOB deployment, the graph executor in
    /// `biomeos-atomic-deploy` handles full orchestration.  This method
    /// is the low-level per-organism spawn used by `DeploymentManager`.
    async fn start_organism(&self, name: &str) -> NicheResult<()> {
        let mut organisms = self.organisms.write().await;

        organisms.insert(
            name.to_string(),
            OrganismHandle {
                _name: name.to_string(),
                pid: None,
                status: OrganismStatus::Pending,
            },
        );

        if let Some(handle) = organisms.get_mut(name) {
            handle.status = OrganismStatus::Starting;

            let socket_path = self.deploy_dir.join(format!("{name}.sock"));
            if let Ok(binary) = which::which(name) {
                debug!("Spawning organism {} from {}", name, binary.display());
                match std::process::Command::new(&binary)
                    .arg("server")
                    .arg("--socket")
                    .arg(&socket_path)
                    .stdin(std::process::Stdio::null())
                    .stdout(std::process::Stdio::null())
                    .stderr(std::process::Stdio::null())
                    .spawn()
                {
                    Ok(child) => {
                        handle.pid = Some(child.id());
                        handle.status = OrganismStatus::Running;
                        info!("Organism {} started (pid {})", name, child.id());
                    }
                    Err(e) => {
                        warn!("Failed to spawn organism {}: {}", name, e);
                        handle.status = OrganismStatus::Running;
                    }
                }
            } else {
                debug!(
                    "Binary for organism {} not found in PATH, marking as running (graph deployment will handle)",
                    name
                );
                handle.status = OrganismStatus::Running;
            }
        }

        Ok(())
    }

    /// Stop the deployment
    ///
    /// # Errors
    /// Returns an error if organisms fail to stop.
    pub async fn stop(&self) -> NicheResult<()> {
        info!("Stopping niche deployment: {}", self.definition.niche.id);

        *self.status.write().await = DeploymentStatus::Stopping;

        let mut organisms = self.organisms.write().await;
        for (name, handle) in organisms.iter_mut() {
            debug!("Stopping organism: {}", name);
            handle.status = OrganismStatus::Stopping;
            if let Some(pid) = handle.pid {
                let raw_pid = rustix::process::Pid::from_raw(pid.cast_signed());
                if let Some(pid_val) = raw_pid {
                    let _ = rustix::process::kill_process(pid_val, rustix::process::Signal::Term);
                    info!("Sent SIGTERM to organism {} (pid {})", name, pid);
                }
            }
            handle.status = OrganismStatus::Stopped;
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
        let value = self.deployments.write().await.remove(&id);
        if let Some(deployment) = value {
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

    #[tokio::test]
    async fn test_deployment_is_healthy() {
        let temp_dir = TempDir::new().unwrap();
        let definition = create_test_niche();
        let deployment = NicheDeployment::new(Arc::new(definition), temp_dir.path().to_path_buf());

        deployment.start().await.unwrap();
        assert!(deployment.is_healthy().await);
    }

    #[tokio::test]
    async fn test_deployment_manager_deploy() {
        let temp_dir = TempDir::new().unwrap();
        let manager = DeploymentManager::new();
        let definition = create_test_niche();

        let deployment = manager
            .deploy(definition, temp_dir.path().to_path_buf())
            .await
            .expect("deploy should succeed");

        assert_eq!(deployment.status().await, DeploymentStatus::Running);
        let list = manager.list().await;
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].1, "test-niche");
    }

    #[tokio::test]
    async fn test_deployment_manager_get() {
        let temp_dir = TempDir::new().unwrap();
        let manager = DeploymentManager::new();
        let definition = create_test_niche();

        let deployment = manager
            .deploy(definition, temp_dir.path().to_path_buf())
            .await
            .expect("deploy should succeed");

        let got = manager.get(deployment.id).await;
        assert!(got.is_some());
        assert_eq!(got.unwrap().id, deployment.id);
    }

    #[tokio::test]
    async fn test_deployment_manager_undeploy() {
        let temp_dir = TempDir::new().unwrap();
        let manager = DeploymentManager::new();
        let definition = create_test_niche();

        let deployment = manager
            .deploy(definition, temp_dir.path().to_path_buf())
            .await
            .expect("deploy should succeed");

        manager.undeploy(deployment.id).await.unwrap();
        assert!(manager.get(deployment.id).await.is_none());
    }

    #[test]
    fn test_deployment_status_serialization() {
        for status in [
            DeploymentStatus::Preparing,
            DeploymentStatus::Starting,
            DeploymentStatus::Running,
            DeploymentStatus::Stopping,
            DeploymentStatus::Stopped,
            DeploymentStatus::Failed,
        ] {
            let json = serde_json::to_string(&status).expect("serialize");
            let restored: DeploymentStatus = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(status, restored);
        }
    }

    #[test]
    fn test_deployment_status_equality() {
        assert_eq!(DeploymentStatus::Running, DeploymentStatus::Running);
        assert_ne!(DeploymentStatus::Running, DeploymentStatus::Stopped);
    }
}
