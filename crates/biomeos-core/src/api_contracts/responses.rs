//! API response types and structures

use super::types::*;
use crate::{HealthStatus, PerformanceMetrics, PrimalType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Primal information response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalInfoResponse {
    /// Primal ID
    pub primal_id: String,
    /// Primal type
    pub primal_type: PrimalType,
    /// Primal version
    pub version: String,
    /// Primal name
    pub name: String,
    /// Primal description
    pub description: String,
    /// Primal maintainer
    pub maintainer: String,
    /// Primal endpoints
    pub endpoints: Vec<EndpointInfo>,
    /// Primal dependencies
    pub dependencies: Vec<DependencyInfo>,
    /// Primal metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Health check response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResponse {
    /// Overall health status
    pub status: HealthStatus,
    /// Health score (0.0-1.0)
    pub health_score: f64,
    /// Component health
    pub components: HashMap<String, ComponentHealth>,
    /// Health checks
    pub checks: Vec<HealthCheckResult>,
    /// System information
    pub system_info: SystemInfo,
}

/// Capabilities response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilitiesResponse {
    /// Available capabilities
    pub capabilities: Vec<CapabilityInfo>,
    /// Capability metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Resource status response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceStatusResponse {
    /// Resource information
    pub resources: Vec<ResourceInfo>,
    /// Overall resource health
    pub overall_health: f64,
    /// Resource metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Performance metrics response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetricsResponse {
    /// Performance metrics
    pub metrics: PerformanceMetrics,
    /// Metric collection timestamp
    pub timestamp: u64,
    /// Metrics metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Operation response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationResponse {
    /// Request ID
    pub request_id: String,
    /// Operation result
    pub result: serde_json::Value,
    /// Operation metadata
    pub metadata: HashMap<String, serde_json::Value>,
    /// Operation warnings
    pub warnings: Vec<String>,
}

/// Inter-primal response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterPrimalResponse {
    /// Message ID
    pub message_id: String,
    /// Response payload
    pub payload: serde_json::Value,
    /// Response metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Configuration response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigurationResponse {
    /// Configuration values
    pub configuration: HashMap<String, serde_json::Value>,
    /// Configuration schema
    pub schema: Option<serde_json::Value>,
    /// Configuration metadata
    pub metadata: ConfigurationMetadata,
}

/// Log response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogResponse {
    /// Log entries
    pub entries: Vec<LogEntry>,
    /// Total count
    pub total_count: u64,
    /// Has more entries
    pub has_more: bool,
}

/// Metrics response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsResponse {
    /// Metrics data
    pub metrics: Vec<MetricData>,
    /// Metrics metadata
    pub metadata: MetricsMetadata,
}
