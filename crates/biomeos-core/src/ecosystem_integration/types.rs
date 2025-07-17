//! # Ecosystem Integration Types
//!
//! Core data structures and enums for ecosystem integration between all Primals.
//! This module contains the fundamental types used for service registration,
//! communication, and coordination across the biomeOS ecosystem.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use uuid::Uuid;

use crate::health::{HealthIssue, HealthMetrics, HealthReport};
use crate::{HealthStatus, PrimalType};

/// Unified service registration for all Primals in the ecosystem
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemServiceRegistration {
    /// Unique service identifier: "primal-{type}-{instance}"
    pub service_id: String,
    /// Type of Primal (toadstool, songbird, nestgate, etc.)
    pub primal_type: PrimalType,
    /// Biome instance this service belongs to
    pub biome_id: String,
    /// Semantic version of the service
    pub version: String,
    /// API version (e.g., "biomeOS/v1")
    pub api_version: String,
    /// When this service was registered
    pub registration_time: DateTime<Utc>,

    /// Service endpoints
    pub endpoints: EcosystemEndpoints,
    /// Service capabilities
    pub capabilities: EcosystemCapabilities,
    /// Security configuration
    pub security: EcosystemSecurity,
    /// Resource requirements
    pub resource_requirements: ResourceRequirements,
    /// Health check configuration
    pub health_check: HealthCheckConfig,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Standardized endpoints for ecosystem services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemEndpoints {
    /// Primary API endpoint
    pub primary: String,
    /// Health check endpoint
    pub health: String,
    /// Metrics endpoint
    pub metrics: String,
    /// Admin interface (optional)
    pub admin: Option<String>,
    /// WebSocket endpoint for real-time updates (optional)
    pub websocket: Option<String>,
}

/// Capabilities provided by a service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemCapabilities {
    /// Core capabilities (always available)
    pub core: Vec<String>,
    /// Extended features (may be optional)
    pub extended: Vec<String>,
    /// Integration points with other Primals
    pub integrations: Vec<String>,
}

/// Security configuration for ecosystem services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemSecurity {
    /// Authentication method
    pub authentication_method: String,
    /// Whether TLS is enabled
    pub tls_enabled: bool,
    /// Whether mutual TLS is required
    pub mtls_required: bool,
    /// Trust domain for this service
    pub trust_domain: String,
}

/// Resource requirements for a service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    /// CPU requirement
    pub cpu: String,
    /// Memory requirement
    pub memory: String,
    /// Storage requirement
    pub storage: String,
    /// Network requirement
    pub network: String,
}

/// Health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    /// Check interval
    pub interval: Duration,
    /// Check timeout
    pub timeout: Duration,
    /// Number of retries before marking unhealthy
    pub retries: u32,
    /// Grace period for startup
    pub grace_period: Duration,
}

/// Inter-Primal communication message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemMessage {
    /// Unique message identifier
    pub message_id: Uuid,
    /// Source Primal
    pub from_primal: PrimalType,
    /// Destination Primal
    pub to_primal: PrimalType,
    /// Message type
    pub message_type: EcosystemMessageType,
    /// Message payload
    pub payload: serde_json::Value,
    /// Message timestamp
    pub timestamp: DateTime<Utc>,
    /// Correlation ID for request/response tracking
    pub correlation_id: Option<Uuid>,
}

/// Types of ecosystem messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EcosystemMessageType {
    // Service coordination
    ServiceRegistration,
    ServiceDeregistration,
    HealthCheck,

    // Resource coordination
    ResourceRequest,
    ResourceAllocation,
    ResourceRelease,

    // Workload coordination
    WorkloadRequest,
    WorkloadStatus,
    WorkloadComplete,

    // Storage coordination
    VolumeProvisionRequest,
    VolumeProvisionComplete,
    MountRequest,
    MountComplete,

    // Ecosystem events
    EcosystemStateChange,
    PrimalStatusUpdate,
    ErrorNotification,
}

/// Ecosystem health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemHealthStatus {
    pub overall_health: HealthStatus,
    pub healthy_services: usize,
    pub total_services: usize,
    pub primal_health: HashMap<String, PrimalHealthInfo>,
}

/// Health information for a specific Primal type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalHealthInfo {
    pub health: HealthStatus,
    pub healthy_count: usize,
    pub total_count: usize,
}

/// Overall ecosystem status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemStatus {
    pub health: EcosystemHealthStatus,
    pub total_services: usize,
    pub active_primals: usize,
    pub uptime: Duration,
}

/// Configuration for ecosystem health monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemHealthConfig {
    /// Health check interval for services
    pub service_check_interval: u64,
    /// Health check timeout
    pub health_check_timeout: u64,
    /// Maximum failed health checks before marking service as failed
    pub max_failed_checks: u32,
    /// Health check retry interval
    pub retry_interval: u64,
    /// Enable automatic service recovery
    pub auto_recovery_enabled: bool,
}

impl Default for EcosystemHealthConfig {
    fn default() -> Self {
        Self {
            service_check_interval: 30,
            health_check_timeout: 10,
            max_failed_checks: 3,
            retry_interval: 5,
            auto_recovery_enabled: true,
        }
    }
}

/// Enhanced ecosystem health status with comprehensive monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedEcosystemHealthStatus {
    /// Overall ecosystem health
    pub overall_health: HealthStatus,
    /// Comprehensive health report
    pub health_report: HealthReport,
    /// Service-specific health information
    pub service_health: HashMap<String, ServiceHealthInfo>,
    /// Primal health by type
    pub primal_health: HashMap<String, PrimalHealthInfo>,
    /// Performance metrics
    pub performance_metrics: EcosystemPerformanceMetrics,
    /// Health trends over time
    pub health_trends: EcosystemHealthTrends,
}

/// Service health information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceHealthInfo {
    /// Service ID
    pub service_id: String,
    /// Service health status
    pub status: HealthStatus,
    /// Health metrics
    pub metrics: HealthMetrics,
    /// Service-specific issues
    pub issues: Vec<HealthIssue>,
    /// Last successful health check
    pub last_successful_check: Option<DateTime<Utc>>,
    /// Failed health check count
    pub failed_check_count: u32,
    /// Response time history
    pub response_times: Vec<f64>,
}

/// Performance metrics for the ecosystem
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemPerformanceMetrics {
    /// Average response time across all services
    pub avg_response_time_ms: f64,
    /// Request throughput (requests per second)
    pub throughput_rps: f64,
    /// Error rate (percentage of failed requests)
    pub error_rate: f64,
    /// Resource utilization summary
    pub resource_utilization: ResourceUtilizationSummary,
    /// Network performance metrics
    pub network_metrics: NetworkPerformanceMetrics,
}

/// Resource utilization summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUtilizationSummary {
    /// Overall CPU usage
    pub cpu_usage: f64,
    /// Overall memory usage
    pub memory_usage: f64,
    /// Overall disk usage
    pub disk_usage: f64,
    /// Network bandwidth usage
    pub network_usage: f64,
}

/// Network performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkPerformanceMetrics {
    /// Average latency between services
    pub avg_latency_ms: f64,
    /// Packet loss rate
    pub packet_loss_rate: f64,
    /// Bandwidth utilization
    pub bandwidth_utilization: f64,
    /// Connection success rate
    pub connection_success_rate: f64,
}

/// Health trends over time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemHealthTrends {
    /// Overall trend direction
    pub overall_trend: HealthTrendDirection,
    /// Service-level trends
    pub service_trends: HashMap<String, HealthTrendDirection>,
    /// Performance trend
    pub performance_trend: PerformanceTrend,
    /// Trend analysis period
    pub analysis_period: chrono::Duration,
}

/// Health trend direction
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HealthTrendDirection {
    Improving,
    Declining,
    Stable,
    Unknown,
}

/// Performance trend information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceTrend {
    /// Response time trend
    pub response_time_trend: HealthTrendDirection,
    /// Throughput trend
    pub throughput_trend: HealthTrendDirection,
    /// Error rate trend
    pub error_rate_trend: HealthTrendDirection,
    /// Resource usage trend
    pub resource_usage_trend: HealthTrendDirection,
}
