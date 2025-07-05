//! Health monitoring for biomeOS

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Health status enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HealthStatus {
    /// System is healthy and functioning normally
    Healthy,
    /// System has minor issues but is still functional
    Warning,
    /// System has serious issues affecting functionality
    Critical,
    /// System is not responding or has failed
    Failed,
    /// Health status is unknown
    Unknown,
}

/// Detailed health information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthInfo {
    /// Overall health status
    pub status: HealthStatus,
    /// Health check timestamp
    pub timestamp: DateTime<Utc>,
    /// Health score (0.0 = failed, 1.0 = perfect health)
    pub score: f64,
    /// Detailed health metrics
    pub metrics: HealthMetrics,
    /// Health issues if any
    pub issues: Vec<HealthIssue>,
}

/// Health metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthMetrics {
    /// CPU usage percentage (0.0-1.0)
    pub cpu_usage: f64,
    /// Memory usage percentage (0.0-1.0)
    pub memory_usage: f64,
    /// Disk usage percentage (0.0-1.0)
    pub disk_usage: f64,
    /// Network latency in milliseconds
    pub network_latency_ms: f64,
    /// Request success rate (0.0-1.0)
    pub success_rate: f64,
    /// Average response time in milliseconds
    pub avg_response_time_ms: f64,
}

/// Health issue description
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthIssue {
    /// Issue severity
    pub severity: IssueSeverity,
    /// Component affected
    pub component: String,
    /// Issue description
    pub description: String,
    /// When the issue was first detected
    pub first_detected: DateTime<Utc>,
    /// Suggested resolution
    pub resolution: Option<String>,
}

/// Issue severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum IssueSeverity {
    /// Informational message
    Info,
    /// Warning that should be addressed
    Warning,
    /// Error affecting functionality
    Error,
    /// Critical issue requiring immediate attention
    Critical,
}

impl Default for HealthStatus {
    fn default() -> Self {
        HealthStatus::Unknown
    }
}

impl Default for HealthInfo {
    fn default() -> Self {
        Self {
            status: HealthStatus::Unknown,
            timestamp: Utc::now(),
            score: 0.0,
            metrics: HealthMetrics::default(),
            issues: Vec::new(),
        }
    }
}

impl Default for HealthMetrics {
    fn default() -> Self {
        Self {
            cpu_usage: 0.0,
            memory_usage: 0.0,
            disk_usage: 0.0,
            network_latency_ms: 0.0,
            success_rate: 1.0,
            avg_response_time_ms: 0.0,
        }
    }
}
