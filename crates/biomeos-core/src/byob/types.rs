

//! BYOB (Bring Your Own Biome) types and data structures
//!
//! This module contains all the type definitions for the BYOB functionality,
//! including team workspaces, deployment instances, health monitoring, and
//! Toadstool orchestration integration.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::BiomeResult;
use std::sync::{Arc, Mutex};
use tokio::sync::RwLock;


use crate::health::{HealthMonitor, IssueSeverity};
use crate::{BiomeOSConfig, HealthStatus};

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
    pub config: BiomeOSConfig,
}

/// Simple biome manifest for BYOB deployments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleBiomeManifest {
    pub metadata: SimpleBiomeMetadata,
    pub services: Vec<SimpleBiomeService>,
    pub resources: SimpleBiomeResources,
}

/// Simple biome metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleBiomeMetadata {
    pub name: String,
    pub version: String,
}

/// Simple biome service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleBiomeService {
    pub name: String,
    pub image: String,
    pub port: u16,
}

/// Simple biome resources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleBiomeResources {
    pub cpu_cores: f64,
    pub memory_mb: u64,
    pub storage_gb: u64,
} 
/// BiomeManifest definition for BYOB
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeManifest {
    pub name: String,
    pub version: String,
    pub services: Vec<Service>,
    pub dependencies: Vec<String>,
}

/// Service definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Service {
    pub name: String,
    pub image: String,
    pub ports: Vec<u16>,
}

/// Ecosystem coordinator placeholder
#[derive(Debug, Clone)]
pub struct EcosystemCoordinator {
    pub coordinator_id: String,
}

/// Ecosystem health coordinator placeholder
#[derive(Debug, Clone)]
pub struct EcosystemHealthCoordinator {
    pub coordinator_id: String,
}


impl EcosystemHealthCoordinator {
    pub fn new() -> Self {
        Self {
            coordinator_id: "health-coordinator".to_string(),
        }
    }

    pub async fn initialize(&self) -> BiomeResult<()> {
        // TODO: Implement health coordinator initialization
        Ok(())
    }
}

