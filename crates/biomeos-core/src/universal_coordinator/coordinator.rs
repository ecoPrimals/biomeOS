//! Main coordinator implementation for the universal biome coordinator
//!
//! This module contains the main UniversalBiomeCoordinator implementation
//! that orchestrates the entire biome ecosystem deployment process.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use uuid::Uuid;
use chrono::Utc;

use crate::universal_manifest::{
    UniversalBiomeManifest, ValidationSeverity,
};
use crate::universal_primal::{
    CapabilityRequirement, DefaultDiscoveryService, DiscoveredPrimal,
};
use crate::{BiomeError, BiomeResult};
use super::types::{
    UniversalBiomeCoordinator, CoordinatorConfig, EcosystemInstance, EcosystemStatus,
    DeploymentPlan, PrimalAssignment, ResourcePlan, DeployedPrimal, PrimalStatus,
    ResourceAllocation, CapabilityRouter, RequirementMatcher,
};

impl UniversalBiomeCoordinator {
    /// Create new coordinator with default configuration
    pub fn new() -> Self {
        Self::with_config(CoordinatorConfig::default())
    }

    /// Create coordinator with custom configuration
    pub fn with_config(config: CoordinatorConfig) -> Self {
        let discovery_service = Arc::new(DefaultDiscoveryService::new());
        let capability_router = Arc::new(CapabilityRouter::new());
        let requirement_matcher = Arc::new(RequirementMatcher::new());

        Self {
            discovery_service,
            capability_router,
            requirement_matcher,
            active_deployments: Arc::new(RwLock::new(HashMap::new())),
            primal_clients: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    /// Bootstrap a new ecosystem from a manifest
    pub async fn bootstrap_ecosystem(
        &self,
        manifest: UniversalBiomeManifest,
    ) -> BiomeResult<EcosystemInstance> {
        info!(
            "Starting ecosystem bootstrap for biome: {}",
            manifest.metadata.name
        );

        // Step 1: Validate manifest
        let validation_results = manifest.validate()?;
        if validation_results
            .iter()
            .any(|e| matches!(e.severity, ValidationSeverity::Error))
        {
            return Err(BiomeError::ValidationError(
                "Manifest validation failed with errors".to_string(),
            ));
        }

        // Step 2: Discover available primals
        let available_primals = self.discover_primals().await?;
        info!("Discovered {} primals", available_primals.len());

        // Step 3: Create deployment plan
        let deployment_plan = self
            .create_deployment_plan(&manifest, &available_primals)
            .await?;
        info!(
            "Created deployment plan with {} primal assignments",
            deployment_plan.primal_assignments.len()
        );

        // Step 4: Validate deployment plan
        self.validate_deployment_plan(&deployment_plan).await?;

        // Step 5: Execute deployment
        let instance = self.execute_deployment(&manifest, deployment_plan).await?;
        info!("Successfully bootstrapped ecosystem: {}", instance.id);

        // Step 6: Store instance
        {
            let mut deployments = self.active_deployments.write().await;
            deployments.insert(instance.id.clone(), instance.clone());
        }

        Ok(instance)
    }

    /// Discover available primals
    async fn discover_primals(&self) -> BiomeResult<Vec<DiscoveredPrimal>> {
        debug!("Starting primal discovery");

        // Use discovery service to find primals
        let primals = self.discovery_service.auto_discover().await?;

        // Update capability router
        self.capability_router.update_primals(&primals).await?;

        Ok(primals)
    }

    /// Create deployment plan
    async fn create_deployment_plan(
        &self,
        manifest: &UniversalBiomeManifest,
        available_primals: &[DiscoveredPrimal],
    ) -> BiomeResult<DeploymentPlan> {
        debug!("Creating deployment plan");

        // Get all required capabilities
        let required_capabilities = manifest.get_all_required_capabilities();

        // Convert to capability requirements
        let capability_requirements: Vec<CapabilityRequirement> = required_capabilities
            .into_iter()
            .map(|cap| CapabilityRequirement {
                capability: cap,
                min_version: "1.0.0".to_string(),
                max_version: None,
                optional: false,
                constraints: vec![],
                fallback: None,
            })
            .collect();

        // Match requirements to primals
        let matches = self
            .requirement_matcher
            .match_requirements(&capability_requirements, available_primals)
            .await?;

        // Create primal assignments
        let mut assignments = Vec::new();
        for (i, match_result) in matches.iter().enumerate() {
            let assignment = PrimalAssignment {
                id: format!("assignment-{}", i),
                required_capabilities: match_result.details.capability_matches.keys().cloned().collect(),
                assigned_primal: match_result.primal.clone(),
                score: match_result.score,
                justification: format!(
                    "Matched {} capabilities with score {:.2}",
                    match_result.details.capability_matches.len(),
                    match_result.score
                ),
            };
            assignments.push(assignment);
        }

        // Create resource plan
        let resource_plan = self.create_resource_plan(manifest, &assignments).await?;

        // Create deployment order
        let deployment_order = self.create_deployment_order(&assignments).await?;

        let plan = DeploymentPlan {
            id: Uuid::new_v4().to_string(),
            strategy: manifest.deployment.strategy.clone(),
            primal_assignments: assignments,
            resource_plan,
            deployment_order,
            validation_results: vec![],
        };

        Ok(plan)
    }

    /// Create resource allocation plan
    async fn create_resource_plan(
        &self,
        manifest: &UniversalBiomeManifest,
        _assignments: &[PrimalAssignment],
    ) -> BiomeResult<ResourcePlan> {
        let resource_summary = manifest.get_resource_summary();

        Ok(ResourcePlan {
            cpu: resource_summary.total_cpu.to_string(),
            memory: format!("{}MB", resource_summary.total_memory_mb),
            storage: format!("{}MB", resource_summary.total_storage_mb),
            network: None,
            gpu: None,
            pools: vec![],
        })
    }

    /// Create deployment order
    async fn create_deployment_order(
        &self,
        assignments: &[PrimalAssignment],
    ) -> BiomeResult<Vec<String>> {
        // Simple ordering for now - deploy in assignment order
        let order = assignments.iter().map(|a| a.id.clone()).collect();
        Ok(order)
    }

    /// Validate deployment plan
    async fn validate_deployment_plan(&self, plan: &DeploymentPlan) -> BiomeResult<()> {
        debug!("Validating deployment plan");

        // Check if all assignments are valid
        if plan.primal_assignments.is_empty() {
            return Err(BiomeError::ValidationError(
                "No primal assignments in deployment plan".to_string(),
            ));
        }

        // Check if all assignments have valid scores
        for assignment in &plan.primal_assignments {
            if assignment.score < 0.5 {
                warn!(
                    "Assignment {} has low score: {:.2}",
                    assignment.id, assignment.score
                );
            }
        }

        Ok(())
    }

    /// Execute deployment
    async fn execute_deployment(
        &self,
        manifest: &UniversalBiomeManifest,
        deployment_plan: DeploymentPlan,
    ) -> BiomeResult<EcosystemInstance> {
        debug!("Starting deployment execution");

        let instance_id = Uuid::new_v4().to_string();
        let now = Utc::now();

        // Create initial instance
        let mut instance = EcosystemInstance {
            id: instance_id.clone(),
            name: manifest.metadata.name.clone(),
            manifest: manifest.clone(),
            primals: HashMap::new(),
            status: EcosystemStatus::Deploying,
            created_at: now,
            updated_at: now,
            deployment_plan: deployment_plan.clone(),
            resource_allocation: ResourceAllocation {
                cpu: deployment_plan.resource_plan.cpu.clone(),
                memory: deployment_plan.resource_plan.memory.clone(),
                storage: deployment_plan.resource_plan.storage.clone(),
                network: deployment_plan.resource_plan.network.clone(),
                gpu: deployment_plan.resource_plan.gpu.clone(),
            },
        };

        // Deploy primals in order
        for assignment_id in &deployment_plan.deployment_order {
            let assignment = deployment_plan
                .primal_assignments
                .iter()
                .find(|a| a.id == *assignment_id)
                .ok_or_else(|| {
                    BiomeError::RuntimeError(format!("Assignment not found: {}", assignment_id))
                })?;

            let deployed_primal = self.deploy_primal(assignment).await?;
            instance.primals.insert(assignment.id.clone(), deployed_primal);
        }

        // Update instance status
        instance.status = EcosystemStatus::Running;
        instance.updated_at = Utc::now();

        info!("Deployment execution completed for instance: {}", instance.id);
        Ok(instance)
    }

    /// Deploy a single primal
    async fn deploy_primal(&self, assignment: &PrimalAssignment) -> BiomeResult<DeployedPrimal> {
        debug!("Deploying primal: {}", assignment.assigned_primal.id);

        // Create deployed primal
        let deployed_primal = DeployedPrimal {
            id: assignment.assigned_primal.id.clone(),
            primal_type: assignment.assigned_primal.primal_type.clone(),
            endpoints: assignment.assigned_primal.endpoints.clone(),
            status: PrimalStatus::Running,
            capabilities: assignment.assigned_primal.capabilities.clone(),
            resources: ResourceAllocation {
                cpu: "1".to_string(),
                memory: "512MB".to_string(),
                storage: "1GB".to_string(),
                network: None,
                gpu: None,
            },
            health: crate::HealthStatus::Healthy,
            last_health_check: Utc::now(),
        };

        Ok(deployed_primal)
    }

    /// Get ecosystem instance
    pub async fn get_ecosystem(&self, id: &str) -> BiomeResult<Option<EcosystemInstance>> {
        let deployments = self.active_deployments.read().await;
        Ok(deployments.get(id).cloned())
    }

    /// List all ecosystems
    pub async fn list_ecosystems(&self) -> BiomeResult<Vec<EcosystemInstance>> {
        let deployments = self.active_deployments.read().await;
        Ok(deployments.values().cloned().collect())
    }

    /// Stop ecosystem
    pub async fn stop_ecosystem(&self, id: &str) -> BiomeResult<()> {
        let mut deployments = self.active_deployments.write().await;
        if let Some(instance) = deployments.get_mut(id) {
            instance.status = EcosystemStatus::Stopped;
            instance.updated_at = Utc::now();
            info!("Stopped ecosystem: {}", id);
        }
        Ok(())
    }

    /// Remove ecosystem
    pub async fn remove_ecosystem(&self, id: &str) -> BiomeResult<()> {
        let mut deployments = self.active_deployments.write().await;
        if deployments.remove(id).is_some() {
            info!("Removed ecosystem: {}", id);
        }
        Ok(())
    }

    /// Get coordinator statistics
    pub async fn get_stats(&self) -> BiomeResult<CoordinatorStats> {
        let deployments = self.active_deployments.read().await;
        let total_ecosystems = deployments.len();

        let mut running_count = 0;
        let mut stopped_count = 0;
        let mut failed_count = 0;

        for instance in deployments.values() {
            match instance.status {
                EcosystemStatus::Running => running_count += 1,
                EcosystemStatus::Stopped => stopped_count += 1,
                EcosystemStatus::Failed(_) => failed_count += 1,
                _ => {}
            }
        }

        Ok(CoordinatorStats {
            total_ecosystems,
            running_ecosystems: running_count,
            stopped_ecosystems: stopped_count,
            failed_ecosystems: failed_count,
        })
    }
}

impl Default for UniversalBiomeCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

/// Coordinator statistics
#[derive(Debug, Clone)]
pub struct CoordinatorStats {
    /// Total number of ecosystems
    pub total_ecosystems: usize,
    /// Number of running ecosystems
    pub running_ecosystems: usize,
    /// Number of stopped ecosystems
    pub stopped_ecosystems: usize,
    /// Number of failed ecosystems
    pub failed_ecosystems: usize,
} 