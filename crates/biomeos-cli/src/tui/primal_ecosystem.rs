// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Primal API shapes, deployment lifecycle types, and aggregated ecosystem health for the TUI.

#![forbid(unsafe_code)]

use biomeos_types::{Health, PrimalCapability, PrimalType};
use std::time::{Duration, Instant};

/// State of a primal obtained from its headless API
#[derive(Debug, Clone)]
pub struct PrimalApiState {
    /// Unique identifier for the primal
    pub primal_id: String,
    /// Type classification of the primal
    pub primal_type: PrimalType,
    /// API endpoint URL
    pub endpoint: String,
    /// Current health status
    pub health: Health,
    /// Capabilities provided by the primal
    pub capabilities: Vec<PrimalCapability>,
    /// Primal metadata (name, version, etc.)
    pub metadata: PrimalMetadata,
    /// Services managed by this primal
    pub services: Vec<PrimalServiceInfo>,
    /// Performance metrics from the primal
    pub metrics: PrimalMetrics,
    /// Timestamp of the last API data update
    pub last_updated: Instant,
    /// API version reported by the primal
    pub api_version: String,
}

/// Metadata from primal API
#[derive(Debug, Clone)]
pub struct PrimalMetadata {
    /// Primal display name
    pub name: String,
    /// Primal version string
    pub version: String,
    /// Human-readable description
    pub description: String,
    /// How long the primal has been running
    pub uptime: Duration,
    /// Current resource utilization
    pub resource_usage: ResourceUsage,
}

/// Service information from primal
#[derive(Debug, Clone)]
pub struct PrimalServiceInfo {
    /// Unique service identifier
    pub service_id: String,
    /// Service display name
    pub name: String,
    /// Current service status
    pub status: ServiceStatus,
    /// Number of running replicas (if applicable)
    pub replicas: Option<u32>,
    /// Current resource utilization
    pub resource_usage: ResourceUsage,
}

/// Resource usage metrics
#[derive(Debug, Clone)]
pub struct ResourceUsage {
    /// CPU utilization percentage (0.0–100.0)
    pub cpu_percent: f64,
    /// Memory usage in megabytes
    pub memory_mb: f64,
    /// Disk usage in gigabytes
    pub disk_gb: f64,
    /// Network throughput in megabits per second
    pub network_mbps: f64,
}

/// Service status
#[derive(Debug, Clone)]
pub enum ServiceStatus {
    /// Service is running normally
    Running,
    /// Service is starting up
    Starting,
    /// Service is shutting down
    Stopping,
    /// Service has failed
    Failed,
    /// Service is scaling up or down
    Scaling,
}

/// Metrics from primal APIs
#[derive(Debug, Clone)]
pub struct PrimalMetrics {
    /// Number of requests handled per second
    pub requests_per_second: f64,
    /// Average response time across recent requests
    pub average_response_time: Duration,
    /// Fraction of requests that resulted in errors (0.0–1.0)
    pub error_rate: f64,
    /// Data throughput rate
    pub throughput: f64,
}

/// Overall ecosystem health aggregated from all primals
#[derive(Debug, Clone)]
pub struct EcosystemHealth {
    /// Aggregated health status across all primals
    pub overall_status: Health,
    /// Total number of primals in the ecosystem
    pub primal_count: usize,
    /// Number of primals reporting healthy status
    pub healthy_primals: usize,
    /// Total number of services across all primals
    pub total_services: usize,
    /// Number of services reporting healthy status
    pub healthy_services: usize,
    /// Number of currently active deployments
    pub active_deployments: usize,
    /// List of critical issues requiring attention
    pub critical_issues: Vec<String>,
}

impl Default for EcosystemHealth {
    fn default() -> Self {
        Self {
            overall_status: Health::Unknown {
                reason: "No data available".to_string(),
                last_known: None,
            },
            primal_count: 0,
            healthy_primals: 0,
            total_services: 0,
            healthy_services: 0,
            active_deployments: 0,
            critical_issues: Vec::new(),
        }
    }
}

/// Deployment status for orchestration
#[derive(Debug, Clone)]
pub struct DeploymentStatus {
    /// Unique deployment identifier
    pub deployment_id: String,
    /// Name of the biome being deployed
    pub biome_name: String,
    /// Current deployment phase
    pub status: DeploymentPhase,
    /// Target environment for deployment
    pub target_environment: String,
    /// Deployment progress percentage (0–100)
    pub progress: u8,
    /// When the deployment started
    pub started_at: Instant,
    /// Estimated completion time (if available)
    pub estimated_completion: Option<Instant>,
    /// Services that have been successfully deployed
    pub deployed_services: Vec<String>,
    /// Services that failed during deployment
    pub failed_services: Vec<String>,
}

/// Deployment phases
#[derive(Debug, Clone)]
pub enum DeploymentPhase {
    /// Validating deployment configuration
    Validating,
    /// Deploying services
    Deploying,
    /// Scaling services to target replicas
    Scaling,
    /// Applying configuration to deployed services
    Configuring,
    /// Running health checks on deployed services
    HealthChecking,
    /// Deployment completed successfully
    Complete,
    /// Deployment failed
    Failed {
        /// Reason for the deployment failure
        reason: String,
    },
    /// Rolling back a failed deployment
    RollingBack,
}

/// Deployment events for history
#[derive(Debug, Clone)]
pub struct DeploymentEvent {
    /// When the event occurred
    pub timestamp: Instant,
    /// Associated deployment identifier
    pub deployment_id: String,
    /// Type of deployment event
    pub event_type: DeploymentEventType,
    /// Human-readable event message
    pub message: String,
}

/// Types of deployment events
#[derive(Debug, Clone)]
pub enum DeploymentEventType {
    /// Deployment started
    Started,
    /// A service was successfully deployed
    ServiceDeployed,
    /// A service deployment failed
    ServiceFailed,
    /// Deployment completed successfully
    Completed,
    /// Deployment failed overall
    Failed,
    /// Deployment was rolled back
    RolledBack,
}
