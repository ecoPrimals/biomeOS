//! BYOB (Bring Your Own Biome) deployment manager
//!
//! This module contains the main implementation of the ByobDeploymentManager,
//! responsible for managing team workspaces, deployments, health monitoring,
//! and Toadstool orchestration integration.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::RwLock;
use tracing::{info, warn};
use uuid::Uuid;


use crate::health::{ComponentType, HealthIssue, HealthMetrics, HealthMonitor, IssueSeverity};
use crate::{BiomeError, BiomeOSConfig, BiomeResult, HealthStatus};

use super::types::*;

impl ByobDeploymentManager {
    /// Create new BYOB deployment manager with health monitoring
    pub fn new(config: BiomeOSConfig) -> Self {
        // Ecosystem coordinator removed - using universal primal manager
        // Health coordinator removed - using universal primal manager
        let health_monitor = Arc::new(HealthMonitor::new());

        Self {
            workspaces: Arc::new(Mutex::new(HashMap::new())),
            deployments: Arc::new(RwLock::new(HashMap::new())),
            ecosystem_coordinator,
            health_coordinator: Arc::new(health_coordinator),
            health_monitor,
            config,
        }
    }

    /// Initialize the BYOB system with health monitoring
    pub async fn initialize(&self) -> BiomeResult<()> {
        info!("Initializing BYOB system with health monitoring");

        // Initialize health monitoring
        self.health_coordinator.initialize().await?;

        // Start health monitoring background tasks
        self.start_health_monitoring().await?;

        info!("BYOB system initialized successfully");
        Ok(())
    }

    /// Start health monitoring background tasks
    async fn start_health_monitoring(&self) -> BiomeResult<()> {
        let health_monitor = self.health_monitor.clone();
        let deployments = self.deployments.clone();
        let workspaces = self.workspaces.clone();

        // Start deployment health monitoring
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));

            loop {
                interval.tick().await;

                // Check health of all deployments
                let deployments_guard = deployments.read().await;
                for (deployment_id, deployment) in deployments_guard.iter() {
                    if let Err(e) =
                        Self::check_deployment_health(&health_monitor, deployment_id, deployment)
                            .await
                    {
                        warn!(
                            "Health check failed for deployment {}: {}",
                            deployment_id, e
                        );
                    }
                }

                // Check resource usage against quotas
                let workspaces_snapshot = {
                    let workspaces_guard = workspaces.lock().unwrap();
                    workspaces_guard.clone()
                };
                for (team_id, workspace) in workspaces_snapshot.iter() {
                    if let Err(e) = Self::check_resource_usage(team_id, workspace).await {
                        warn!("Resource usage check failed for team {}: {}", team_id, e);
                    }
                }
            }
        });

        Ok(())
    }

    /// Check health of a specific deployment
    async fn check_deployment_health(
        health_monitor: &HealthMonitor,
        deployment_id: &str,
        deployment: &DeploymentInstance,
    ) -> BiomeResult<()> {
        // Register deployment as a component if not already registered
        if (health_monitor
            .register_component(deployment_id.to_string(), ComponentType::Biome, Some(30))
            .await)
            .is_err()
        {
            // Component might already be registered, which is fine
        }

        // Collect health metrics for the deployment
        let metrics = Self::collect_deployment_metrics(deployment).await?;

        // Collect health issues
        let issues = vec![]; // TODO: collect_deployment_issues implementation

        // Determine overall health status
        let health_status = Self::determine_deployment_health(&metrics, &issues);

        // Update component health
        health_monitor
            .update_component_health(deployment_id, health_status, metrics, issues)
            .await?;

        Ok(())
    }

    /// Collect metrics for a deployment
    async fn collect_deployment_metrics(
        deployment: &DeploymentInstance,
    ) -> BiomeResult<HealthMetrics> {
        // This would collect actual metrics from the deployment
        // For now, we'll simulate based on deployment status
        let metrics = match deployment.status {
            DeploymentStatus::Running => HealthMetrics {
                cpu_usage: 0.4,
                memory_usage: 0.6,
                disk_usage: 0.3,
                network_latency_ms: 50.0,
                success_rate: 0.98,
                avg_response_time_ms: 150.0,
            },
            DeploymentStatus::Scaling => HealthMetrics {
                cpu_usage: 0.8,
                memory_usage: 0.7,
                disk_usage: 0.3,
                network_latency_ms: 80.0,
                success_rate: 0.95,
                avg_response_time_ms: 200.0,
            },
            DeploymentStatus::Failed(_) => HealthMetrics {
                cpu_usage: 0.0,
                memory_usage: 0.0,
                disk_usage: 0.3,
                network_latency_ms: 0.0,
                success_rate: 0.0,
                avg_response_time_ms: 0.0,
            },
            _ => HealthMetrics::default(),
        };

        Ok(metrics)
    }

    /// Collect health issues for a deployment
    async fn collect_deployment_issues(
        deployment: &DeploymentInstance,
    ) -> BiomeResult<Vec<HealthIssue>> {
        let mut issues = Vec::new();

        // Check deployment status
        match &deployment.status {
            DeploymentStatus::Failed(error) => {
                issues.push(HealthIssue {
                    severity: IssueSeverity::Critical,
                    component: deployment.deployment_id.clone(),
                    description: format!("Deployment failed: {}", error),
                    first_detected: deployment.updated_at,
                    resolution: Some("Check deployment logs and redeploy".to_string()),
                });
            }
            DeploymentStatus::Scaling => {
                issues.push(HealthIssue {
                    severity: IssueSeverity::Warning,
                    component: deployment.deployment_id.clone(),
                    description: "Deployment is currently scaling".to_string(),
                    first_detected: deployment.updated_at,
                    resolution: Some("Monitor scaling progress".to_string()),
                });
            }
            _ => {}
        }

        // Check for resource issues based on deployment age
        let deployment_age = Utc::now() - deployment.created_at;
        if deployment_age > chrono::Duration::hours(24) {
            issues.push(HealthIssue {
                severity: IssueSeverity::Info,
                component: deployment.deployment_id.clone(),
                description: "Long-running deployment detected".to_string(),
                first_detected: deployment.created_at,
                resolution: Some("Consider scheduled maintenance".to_string()),
            });
        }

        Ok(issues)
    }

    /// Determine overall deployment health
    fn determine_deployment_health(
        metrics: &HealthMetrics,
        issues: &[HealthIssue],
    ) -> HealthStatus {
        // Check for critical issues
        if issues.iter().any(|i| i.severity == IssueSeverity::Critical) {
            return HealthStatus::Critical;
        }

        // Check metrics
        if metrics.cpu_usage > 0.9 || metrics.memory_usage > 0.9 || metrics.success_rate < 0.5 {
            return HealthStatus::Critical;
        }

        if metrics.cpu_usage > 0.7 || metrics.memory_usage > 0.7 || metrics.success_rate < 0.9 {
            return HealthStatus::Warning;
        }

        // Check for error issues
        if issues.iter().any(|i| i.severity == IssueSeverity::Error) {
            return HealthStatus::Warning;
        }

        HealthStatus::Healthy
    }

    /// Check resource usage against quotas
    async fn check_resource_usage(team_id: &str, workspace: &TeamWorkspace) -> BiomeResult<()> {
        let usage = &workspace.resource_usage;
        let quota = &workspace.resource_quota;

        // Check CPU usage
        if usage.cpu_cores > quota.max_cpu_cores {
            warn!(
                "Team {} exceeds CPU quota: {} > {}",
                team_id, usage.cpu_cores, quota.max_cpu_cores
            );
        }

        // Check memory usage
        if usage.memory_bytes > quota.max_memory_bytes {
            warn!(
                "Team {} exceeds memory quota: {} > {}",
                team_id, usage.memory_bytes, quota.max_memory_bytes
            );
        }

        // Check storage usage
        if usage.storage_bytes > quota.max_storage_bytes {
            warn!(
                "Team {} exceeds storage quota: {} > {}",
                team_id, usage.storage_bytes, quota.max_storage_bytes
            );
        }

        // Check deployment count
        if usage.active_deployments > quota.max_deployments {
            warn!(
                "Team {} exceeds deployment quota: {} > {}",
                team_id, usage.active_deployments, quota.max_deployments
            );
        }

        Ok(())
    }

    /// Create team workspace with health monitoring
    pub fn create_team_workspace(&self, team_id: &str) -> BiomeResult<TeamWorkspace> {
        let workspace = TeamWorkspace {
            team_id: team_id.to_string(),
            created_at: Utc::now(),
            resource_quota: ResourceQuota::default(),
            active_deployments: Vec::new(),
            isolation_config: IsolationConfig::default(),
            health_config: TeamHealthConfig::default(),
            resource_usage: ResourceUsage::default(),
        };

        let mut workspaces = self.workspaces.lock().unwrap();
        if workspaces.contains_key(team_id) {
            return Err(BiomeError::ResourceError(format!(
                "Team workspace already exists: {}",
                team_id
            )));
        }

        workspaces.insert(team_id.to_string(), workspace.clone());
        info!(
            "Created team workspace with health monitoring for: {}",
            team_id
        );
        Ok(workspace)
    }

    /// Get team workspace
    pub fn get_team_workspace(&self, team_id: &str) -> BiomeResult<TeamWorkspace> {
        let workspaces = self.workspaces.lock().unwrap();
        workspaces.get(team_id).cloned().ok_or_else(|| {
            BiomeError::ResourceError(format!("Team workspace not found: {}", team_id))
        })
    }

    /// Deploy a biome with health monitoring and Toadstool integration
    pub async fn deploy_biome(
        &self,
        manifest: &SimpleBiomeManifest,
        team_id: &str,
    ) -> BiomeResult<String> {
        info!(
            "Deploying biome for team {} with health monitoring",
            team_id
        );

        // Validate team workspace
        let workspace = self.get_team_workspace(team_id)?;

        // Check resource quotas
        self.check_deployment_quotas(&workspace, manifest)?;

        // Generate deployment ID
        let deployment_id = format!("deploy-{}", Uuid::new_v4());

        // Create deployment instance with health monitoring
        let deployment = DeploymentInstance {
            deployment_id: deployment_id.clone(),
            biome_manifest: manifest.clone(),
            status: DeploymentStatus::Deploying,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            team_id: team_id.to_string(),
            health_status: DeploymentHealthStatus::default(),
            orchestration_info: ToadstoolOrchestrationInfo::default(),
        };

        // Store deployment
        {
            let mut deployments = self.deployments.write().await;
            deployments.insert(deployment_id.clone(), deployment);
        }

        // Register services with ecosystem and health monitoring
        for service in &manifest.services {
            let service_id = format!("{}-{}-{}", team_id, deployment_id, service.name);

            // Register with ecosystem
            // Service registration removed - using universal primal manager

            // TODO: Replace with universal primal manager service registration
        }

        Ok(())
    }
}

