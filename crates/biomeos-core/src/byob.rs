//! BYOB (Bring Your Own Biome) functionality
//!
//! This module enables teams to deploy independently while leveraging shared Primal infrastructure.
//! Teams maintain sovereignty while benefiting from network effects.
//!
//! Enhanced with comprehensive health monitoring and Toadstool orchestration integration.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::RwLock;
use tracing::{info, warn};
use uuid::Uuid;

use crate::ecosystem_integration::{EcosystemCoordinator, EcosystemHealthCoordinator};
use crate::health::{ComponentType, HealthIssue, HealthMetrics, HealthMonitor, IssueSeverity};
use crate::{BiomeError, BiomeOSConfig, BiomeResult, HealthStatus};

/// Enhanced team workspace with health monitoring integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamWorkspace {
    pub team_id: String,
    pub created_at: DateTime<Utc>,
    pub resource_quota: ResourceQuota,
    pub active_deployments: Vec<String>, // Deployment IDs
    pub isolation_config: IsolationConfig,
    /// Health monitoring configuration for this team
    pub health_config: TeamHealthConfig,
    /// Current resource usage
    pub resource_usage: ResourceUsage,
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

/// Current resource usage tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    pub cpu_cores: f64,
    pub memory_bytes: u64,
    pub storage_bytes: u64,
    pub network_bandwidth_mbps: u64,
    pub active_deployments: u32,
    pub last_updated: DateTime<Utc>,
}

/// Team-specific health monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamHealthConfig {
    /// Enable health monitoring for this team
    pub enabled: bool,
    /// Health check interval in seconds
    pub check_interval: u64,
    /// Health alerting configuration
    pub alerting: TeamAlertConfig,
    /// Auto-scaling configuration
    pub auto_scaling: AutoScalingConfig,
    /// Integration with Toadstool orchestration
    pub toadstool_integration: ToadstoolIntegrationConfig,
}

/// Team alerting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamAlertConfig {
    /// Enable alerts for this team
    pub enabled: bool,
    /// Alert thresholds
    pub thresholds: AlertThresholds,
    /// Notification channels
    pub notification_channels: Vec<String>,
}

/// Alert thresholds for team resources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertThresholds {
    /// CPU usage threshold (percentage)
    pub cpu_threshold: f64,
    /// Memory usage threshold (percentage)
    pub memory_threshold: f64,
    /// Storage usage threshold (percentage)
    pub storage_threshold: f64,
    /// Network usage threshold (percentage)
    pub network_threshold: f64,
}

/// Auto-scaling configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoScalingConfig {
    /// Enable auto-scaling
    pub enabled: bool,
    /// Minimum number of instances
    pub min_instances: u32,
    /// Maximum number of instances
    pub max_instances: u32,
    /// Scale up threshold (CPU/Memory percentage)
    pub scale_up_threshold: f64,
    /// Scale down threshold (CPU/Memory percentage)
    pub scale_down_threshold: f64,
    /// Cooldown period in seconds
    pub cooldown_seconds: u64,
}

/// Toadstool integration configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToadstoolIntegrationConfig {
    /// Enable Toadstool integration
    pub enabled: bool,
    /// Toadstool endpoint
    pub endpoint: String,
    /// Integration features
    pub features: ToadstoolFeatures,
}

/// Toadstool integration features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToadstoolFeatures {
    /// Enable container orchestration
    pub container_orchestration: bool,
    /// Enable service mesh integration
    pub service_mesh: bool,
    /// Enable load balancing
    pub load_balancing: bool,
    /// Enable automatic recovery
    pub auto_recovery: bool,
}

/// Isolation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IsolationConfig {
    pub network_isolation: bool,
    pub resource_isolation: bool,
    pub secret_isolation: bool,
}

/// Enhanced deployment instance with health monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentInstance {
    pub deployment_id: String,
    pub biome_manifest: SimpleBiomeManifest,
    pub status: DeploymentStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub team_id: String,
    /// Health monitoring status
    pub health_status: DeploymentHealthStatus,
    /// Toadstool orchestration info
    pub orchestration_info: ToadstoolOrchestrationInfo,
}

/// Deployment health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentHealthStatus {
    /// Overall health status
    pub overall_health: HealthStatus,
    /// Service-level health information
    pub service_health: HashMap<String, ServiceHealthStatus>,
    /// Resource utilization
    pub resource_utilization: ResourceUtilization,
    /// Recent health events
    pub health_events: Vec<HealthEvent>,
    /// Last health check timestamp
    pub last_health_check: DateTime<Utc>,
}

/// Service health status within a deployment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceHealthStatus {
    /// Service name
    pub service_name: String,
    /// Health status
    pub health: HealthStatus,
    /// Response time metrics
    pub response_time_ms: f64,
    /// Error rate
    pub error_rate: f64,
    /// Instance count
    pub instance_count: u32,
    /// Healthy instance count
    pub healthy_instances: u32,
}

/// Resource utilization for a deployment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUtilization {
    /// CPU usage
    pub cpu_usage: f64,
    /// Memory usage
    pub memory_usage: f64,
    /// Storage usage
    pub storage_usage: f64,
    /// Network usage
    pub network_usage: f64,
}

/// Health event tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthEvent {
    /// Event timestamp
    pub timestamp: DateTime<Utc>,
    /// Event type
    pub event_type: HealthEventType,
    /// Component that generated the event
    pub component: String,
    /// Event message
    pub message: String,
    /// Event severity
    pub severity: IssueSeverity,
}

/// Health event types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthEventType {
    /// Health status changed
    HealthStatusChanged,
    /// Resource threshold exceeded
    ResourceThresholdExceeded,
    /// Service scaled up
    ServiceScaledUp,
    /// Service scaled down
    ServiceScaledDown,
    /// Service recovered
    ServiceRecovered,
    /// Service failed
    ServiceFailed,
}

/// Toadstool orchestration information
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ToadstoolOrchestrationInfo {
    /// Orchestration enabled
    pub enabled: bool,
    /// Container deployment ID
    pub container_deployment_id: Option<String>,
    /// Service mesh configuration
    pub service_mesh_config: Option<ServiceMeshConfig>,
    /// Load balancer configuration
    pub load_balancer_config: Option<LoadBalancerConfig>,
    /// Auto-recovery settings
    pub auto_recovery_enabled: bool,
}

/// Service mesh configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceMeshConfig {
    /// Mesh namespace
    pub namespace: String,
    /// Service discovery enabled
    pub service_discovery: bool,
    /// Traffic management enabled
    pub traffic_management: bool,
    /// Security policies enabled
    pub security_policies: bool,
}

/// Load balancer configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancerConfig {
    /// Load balancer type
    pub lb_type: LoadBalancerType,
    /// Health check configuration
    pub health_check: LoadBalancerHealthCheck,
    /// Sticky sessions enabled
    pub sticky_sessions: bool,
}

/// Load balancer types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoadBalancerType {
    RoundRobin,
    LeastConnections,
    WeightedRoundRobin,
    HealthBased,
}

/// Load balancer health check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancerHealthCheck {
    /// Health check path
    pub path: String,
    /// Check interval in seconds
    pub interval_seconds: u64,
    /// Timeout in seconds
    pub timeout_seconds: u64,
    /// Healthy threshold
    pub healthy_threshold: u32,
    /// Unhealthy threshold
    pub unhealthy_threshold: u32,
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

/// Enhanced BYOB deployment manager with health monitoring and Toadstool integration
pub struct ByobDeploymentManager {
    pub workspaces: Arc<Mutex<HashMap<String, TeamWorkspace>>>,
    pub deployments: Arc<RwLock<HashMap<String, DeploymentInstance>>>,
    pub ecosystem_coordinator: Arc<EcosystemCoordinator>,
    /// Health coordinator for ecosystem-wide health monitoring
    pub health_coordinator: Arc<EcosystemHealthCoordinator>,
    /// Health monitor for component-level monitoring
    pub health_monitor: Arc<HealthMonitor>,
    config: BiomeOSConfig,
}

impl ByobDeploymentManager {
    /// Create new BYOB deployment manager with health monitoring
    pub fn new(config: BiomeOSConfig) -> Self {
        let ecosystem_coordinator = Arc::new(EcosystemCoordinator::new());
        let health_coordinator = EcosystemHealthCoordinator::new(Arc::new(
            ecosystem_coordinator.service_registry.clone(),
        ));
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
        let issues = Self::collect_deployment_issues(deployment).await?;

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
        for (service_name, service_spec) in &manifest.services {
            let service_id = format!("{}-{}-{}", team_id, deployment_id, service_name);

            // Register with ecosystem
            let service_registration = crate::ecosystem_integration::EcosystemServiceRegistration {
                service_id: service_id.clone(),
                primal_type: service_spec.primal.clone(),
                biome_id: manifest.metadata.name.clone(),
                version: manifest.metadata.version.clone(),
                api_version: manifest.api_version.clone(),
                registration_time: Utc::now(),
                endpoints: crate::ecosystem_integration::EcosystemEndpoints {
                    primary: format!("http://localhost:8080/{}", service_name),
                    health: format!("http://localhost:8080/{}/health", service_name),
                    metrics: format!("http://localhost:8080/{}/metrics", service_name),
                    admin: Some(format!("http://localhost:8080/{}/admin", service_name)),
                    websocket: None,
                },
                capabilities: crate::ecosystem_integration::EcosystemCapabilities {
                    core: vec!["biomeOS/v1".to_string()],
                    extended: vec!["deployment".to_string()],
                    integrations: vec!["http".to_string()],
                },
                security: crate::ecosystem_integration::EcosystemSecurity {
                    authentication_method: "jwt".to_string(),
                    tls_enabled: true,
                    mtls_required: false,
                    trust_domain: format!("team-{}", team_id),
                },
                resource_requirements: crate::ecosystem_integration::ResourceRequirements {
                    cpu: format!("{}", service_spec.resources.cpu),
                    memory: format!("{}B", service_spec.resources.memory),
                    storage: format!("{}B", service_spec.resources.storage.unwrap_or(0)),
                    network: "100Mbps".to_string(),
                },
                health_check: crate::ecosystem_integration::HealthCheckConfig {
                    interval: std::time::Duration::from_secs(30),
                    timeout: std::time::Duration::from_secs(10),
                    retries: 3,
                    grace_period: std::time::Duration::from_secs(60),
                },
                metadata: std::collections::HashMap::new(),
            };

            // Register service with ecosystem
            if let Err(e) = self
                .ecosystem_coordinator
                .service_registry
                .register_service(service_registration)
                .await
            {
                self.mark_deployment_failed(
                    &deployment_id,
                    &format!("Service registration failed: {}", e),
                )
                .await;
                return Err(e);
            }

            // Register service with health monitoring
            if let Err(e) = self
                .health_monitor
                .register_component(service_id.clone(), ComponentType::Primal, Some(30))
                .await
            {
                warn!(
                    "Failed to register service {} with health monitoring: {}",
                    service_id, e
                );
            }
        }

        // Integrate with Toadstool if enabled
        if workspace.health_config.toadstool_integration.enabled {
            self.integrate_with_toadstool(&deployment_id, manifest, &workspace)
                .await?;
        }

        // Update deployment status to running
        {
            let mut deployments = self.deployments.write().await;
            if let Some(deployment) = deployments.get_mut(&deployment_id) {
                deployment.status = DeploymentStatus::Running;
                deployment.updated_at = Utc::now();
            }
        }

        // Update workspace resource usage
        self.update_workspace_resource_usage(team_id, manifest, true)
            .await?;

        info!("Biome deployed successfully: {}", deployment_id);
        Ok(deployment_id)
    }

    /// Check deployment against resource quotas
    fn check_deployment_quotas(
        &self,
        workspace: &TeamWorkspace,
        manifest: &SimpleBiomeManifest,
    ) -> BiomeResult<()> {
        let quota = &workspace.resource_quota;
        let usage = &workspace.resource_usage;

        // Calculate resource requirements for new deployment
        let mut required_cpu = 0.0;
        let mut required_memory = 0u64;
        let mut required_storage = 0u64;

        for service_spec in manifest.services.values() {
            required_cpu += service_spec.resources.cpu;
            required_memory += service_spec.resources.memory;
            required_storage += service_spec.resources.storage.unwrap_or(0);
        }

        // Check quotas
        if usage.cpu_cores + required_cpu > quota.max_cpu_cores {
            return Err(BiomeError::ResourceError(format!(
                "CPU quota exceeded: {} + {} > {}",
                usage.cpu_cores, required_cpu, quota.max_cpu_cores
            )));
        }

        if usage.memory_bytes + required_memory > quota.max_memory_bytes {
            return Err(BiomeError::ResourceError(format!(
                "Memory quota exceeded: {} + {} > {}",
                usage.memory_bytes, required_memory, quota.max_memory_bytes
            )));
        }

        if usage.storage_bytes + required_storage > quota.max_storage_bytes {
            return Err(BiomeError::ResourceError(format!(
                "Storage quota exceeded: {} + {} > {}",
                usage.storage_bytes, required_storage, quota.max_storage_bytes
            )));
        }

        if usage.active_deployments + 1 > quota.max_deployments {
            return Err(BiomeError::ResourceError(format!(
                "Deployment quota exceeded: {} + 1 > {}",
                usage.active_deployments, quota.max_deployments
            )));
        }

        Ok(())
    }

    /// Integrate deployment with Toadstool orchestration
    async fn integrate_with_toadstool(
        &self,
        deployment_id: &str,
        manifest: &SimpleBiomeManifest,
        workspace: &TeamWorkspace,
    ) -> BiomeResult<()> {
        info!("Integrating deployment {} with Toadstool", deployment_id);

        let toadstool_config = &workspace.health_config.toadstool_integration;

        // Create Toadstool deployment request
        let _toadstool_request = ToadstoolDeploymentRequest {
            deployment_id: deployment_id.to_string(),
            team_id: workspace.team_id.clone(),
            biome_manifest: manifest.clone(),
            features: toadstool_config.features.clone(),
            health_monitoring: true,
        };

        // Send request to Toadstool (this would be an actual HTTP call)
        // For now, we'll simulate the integration
        let container_deployment_id = format!("toadstool-{}", deployment_id);

        // Update deployment with Toadstool info
        {
            let mut deployments = self.deployments.write().await;
            if let Some(deployment) = deployments.get_mut(deployment_id) {
                deployment.orchestration_info = ToadstoolOrchestrationInfo {
                    enabled: true,
                    container_deployment_id: Some(container_deployment_id),
                    service_mesh_config: if toadstool_config.features.service_mesh {
                        Some(ServiceMeshConfig::default())
                    } else {
                        None
                    },
                    load_balancer_config: if toadstool_config.features.load_balancing {
                        Some(LoadBalancerConfig::default())
                    } else {
                        None
                    },
                    auto_recovery_enabled: toadstool_config.features.auto_recovery,
                };
            }
        }

        info!(
            "Toadstool integration completed for deployment: {}",
            deployment_id
        );
        Ok(())
    }

    /// Update workspace resource usage
    async fn update_workspace_resource_usage(
        &self,
        team_id: &str,
        manifest: &SimpleBiomeManifest,
        is_addition: bool,
    ) -> BiomeResult<()> {
        let mut workspaces = self.workspaces.lock().unwrap();

        if let Some(workspace) = workspaces.get_mut(team_id) {
            let multiplier = if is_addition { 1.0 } else { -1.0 };

            for service_spec in manifest.services.values() {
                workspace.resource_usage.cpu_cores += service_spec.resources.cpu * multiplier;
                workspace.resource_usage.memory_bytes = (workspace.resource_usage.memory_bytes
                    as f64
                    + service_spec.resources.memory as f64 * multiplier)
                    as u64;
                workspace.resource_usage.storage_bytes = (workspace.resource_usage.storage_bytes
                    as f64
                    + service_spec.resources.storage.unwrap_or(0) as f64 * multiplier)
                    as u64;
            }

            if is_addition {
                workspace.resource_usage.active_deployments += 1;
            } else {
                workspace.resource_usage.active_deployments = workspace
                    .resource_usage
                    .active_deployments
                    .saturating_sub(1);
            }

            workspace.resource_usage.last_updated = Utc::now();
        }

        Ok(())
    }

    /// Mark deployment as failed
    async fn mark_deployment_failed(&self, deployment_id: &str, error_message: &str) {
        let mut deployments = self.deployments.write().await;
        if let Some(deployment) = deployments.get_mut(deployment_id) {
            deployment.status = DeploymentStatus::Failed(error_message.to_string());
            deployment.updated_at = Utc::now();
        }
    }

    /// Get deployment health status
    pub async fn get_deployment_health(
        &self,
        deployment_id: &str,
    ) -> BiomeResult<DeploymentHealthStatus> {
        let deployments = self.deployments.read().await;

        if let Some(deployment) = deployments.get(deployment_id) {
            Ok(deployment.health_status.clone())
        } else {
            Err(BiomeError::ResourceError(format!(
                "Deployment not found: {}",
                deployment_id
            )))
        }
    }

    /// Get comprehensive team health report
    pub async fn get_team_health_report(&self, team_id: &str) -> BiomeResult<TeamHealthReport> {
        let workspace = self.get_team_workspace(team_id)?;
        let deployments = self.deployments.read().await;

        // Get all deployments for this team
        let team_deployments: Vec<_> = deployments
            .values()
            .filter(|d| d.team_id == team_id)
            .collect();

        // Generate health report
        let mut healthy_deployments = 0;
        let total_deployments = team_deployments.len();
        let mut deployment_health = HashMap::new();

        for deployment in &team_deployments {
            let health_status = match deployment.status {
                DeploymentStatus::Running => HealthStatus::Healthy,
                DeploymentStatus::Scaling => HealthStatus::Warning,
                DeploymentStatus::Failed(_) => HealthStatus::Critical,
                _ => HealthStatus::Unknown,
            };

            if health_status == HealthStatus::Healthy {
                healthy_deployments += 1;
            }

            deployment_health.insert(deployment.deployment_id.clone(), health_status);
        }

        let overall_health = if healthy_deployments == total_deployments {
            HealthStatus::Healthy
        } else if healthy_deployments >= (total_deployments * 2 / 3) {
            HealthStatus::Warning
        } else {
            HealthStatus::Critical
        };

        Ok(TeamHealthReport {
            team_id: team_id.to_string(),
            overall_health,
            healthy_deployments,
            total_deployments,
            deployment_health,
            resource_usage: workspace.resource_usage.clone(),
            resource_quota: workspace.resource_quota.clone(),
            generated_at: Utc::now(),
        })
    }

    /// Remove a deployment
    pub async fn remove_deployment(&self, deployment_id: &str) -> BiomeResult<()> {
        info!("Removing deployment: {}", deployment_id);

        // Get deployment info before removal
        let deployment = {
            let deployments = self.deployments.read().await;
            deployments.get(deployment_id).cloned()
        };

        if let Some(deployment) = deployment {
            // Unregister services from ecosystem
            for service_name in deployment.biome_manifest.services.keys() {
                let service_id =
                    format!("{}-{}-{}", deployment.team_id, deployment_id, service_name);
                if let Err(e) = self
                    .ecosystem_coordinator
                    .service_registry
                    .unregister_service(&service_id)
                    .await
                {
                    warn!("Failed to unregister service {}: {}", service_id, e);
                }
            }

            // Update workspace resource usage
            self.update_workspace_resource_usage(
                &deployment.team_id,
                &deployment.biome_manifest,
                false,
            )
            .await?;

            // Remove deployment
            {
                let mut deployments = self.deployments.write().await;
                deployments.remove(deployment_id);
            }

            // Note: Health monitoring cleanup would be handled by the health monitor
            // when it detects the component is no longer responding

            info!("Deployment removed successfully: {}", deployment_id);
            Ok(())
        } else {
            Err(BiomeError::ResourceError(format!(
                "Deployment not found: {}",
                deployment_id
            )))
        }
    }

    /// List all deployments for a team
    pub async fn list_team_deployments(
        &self,
        team_id: &str,
    ) -> BiomeResult<Vec<DeploymentInstance>> {
        let deployments = self.deployments.read().await;
        let team_deployments: Vec<_> = deployments
            .values()
            .filter(|d| d.team_id == team_id)
            .cloned()
            .collect();

        Ok(team_deployments)
    }

    /// Get deployment status
    pub async fn get_deployment_status(
        &self,
        deployment_id: &str,
    ) -> BiomeResult<DeploymentStatus> {
        let deployments = self.deployments.read().await;

        if let Some(deployment) = deployments.get(deployment_id) {
            Ok(deployment.status.clone())
        } else {
            Err(BiomeError::ResourceError(format!(
                "Deployment not found: {}",
                deployment_id
            )))
        }
    }
}

/// Toadstool deployment request structure
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ToadstoolDeploymentRequest {
    deployment_id: String,
    team_id: String,
    biome_manifest: SimpleBiomeManifest,
    features: ToadstoolFeatures,
    health_monitoring: bool,
}

/// Team health report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamHealthReport {
    pub team_id: String,
    pub overall_health: HealthStatus,
    pub healthy_deployments: usize,
    pub total_deployments: usize,
    pub deployment_health: HashMap<String, HealthStatus>,
    pub resource_usage: ResourceUsage,
    pub resource_quota: ResourceQuota,
    pub generated_at: DateTime<Utc>,
}

// Default implementations for various configuration structs

impl Default for ResourceQuota {
    fn default() -> Self {
        Self {
            max_cpu_cores: 8.0,
            max_memory_bytes: 16 * 1024 * 1024 * 1024, // 16GB
            max_storage_bytes: 100 * 1024 * 1024 * 1024, // 100GB
            max_network_bandwidth_mbps: 1000,
            max_deployments: 10,
        }
    }
}

impl Default for ResourceUsage {
    fn default() -> Self {
        Self {
            cpu_cores: 0.0,
            memory_bytes: 0,
            storage_bytes: 0,
            network_bandwidth_mbps: 0,
            active_deployments: 0,
            last_updated: Utc::now(),
        }
    }
}

impl Default for TeamHealthConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            check_interval: 30,
            alerting: TeamAlertConfig::default(),
            auto_scaling: AutoScalingConfig::default(),
            toadstool_integration: ToadstoolIntegrationConfig::default(),
        }
    }
}

impl Default for TeamAlertConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            thresholds: AlertThresholds::default(),
            notification_channels: vec!["email".to_string()],
        }
    }
}

impl Default for AlertThresholds {
    fn default() -> Self {
        Self {
            cpu_threshold: 0.8,
            memory_threshold: 0.8,
            storage_threshold: 0.9,
            network_threshold: 0.8,
        }
    }
}

impl Default for AutoScalingConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            min_instances: 1,
            max_instances: 10,
            scale_up_threshold: 0.8,
            scale_down_threshold: 0.3,
            cooldown_seconds: 300,
        }
    }
}

impl Default for ToadstoolIntegrationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            endpoint: "http://localhost:8090".to_string(),
            features: ToadstoolFeatures::default(),
        }
    }
}

impl Default for ToadstoolFeatures {
    fn default() -> Self {
        Self {
            container_orchestration: true,
            service_mesh: false,
            load_balancing: true,
            auto_recovery: true,
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

impl Default for DeploymentHealthStatus {
    fn default() -> Self {
        Self {
            overall_health: HealthStatus::Unknown,
            service_health: HashMap::new(),
            resource_utilization: ResourceUtilization::default(),
            health_events: Vec::new(),
            last_health_check: Utc::now(),
        }
    }
}

impl Default for ResourceUtilization {
    fn default() -> Self {
        Self {
            cpu_usage: 0.0,
            memory_usage: 0.0,
            storage_usage: 0.0,
            network_usage: 0.0,
        }
    }
}

impl Default for ServiceMeshConfig {
    fn default() -> Self {
        Self {
            namespace: "default".to_string(),
            service_discovery: true,
            traffic_management: true,
            security_policies: true,
        }
    }
}

impl Default for LoadBalancerConfig {
    fn default() -> Self {
        Self {
            lb_type: LoadBalancerType::HealthBased,
            health_check: LoadBalancerHealthCheck::default(),
            sticky_sessions: false,
        }
    }
}

impl Default for LoadBalancerHealthCheck {
    fn default() -> Self {
        Self {
            path: "/health".to_string(),
            interval_seconds: 30,
            timeout_seconds: 10,
            healthy_threshold: 2,
            unhealthy_threshold: 3,
        }
    }
}

// Temporary struct definitions (these would be imported from manifest module)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleBiomeManifest {
    pub metadata: SimpleBiomeMetadata,
    pub api_version: String,
    pub services: HashMap<String, SimpleBiomeService>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleBiomeMetadata {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleBiomeService {
    pub primal: String,
    pub resources: SimpleBiomeResources,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleBiomeResources {
    pub cpu: f64,
    pub memory: u64,
    pub storage: Option<u64>,
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
        assert_eq!(workspace.resource_quota.max_cpu_cores, 8.0);
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
