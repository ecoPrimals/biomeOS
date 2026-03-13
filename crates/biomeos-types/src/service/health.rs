// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Service Health Configurations
//!
//! This module contains health-related types including health checks,
//! probes, and monitoring configurations.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::health::HealthCheckConfig;

/// Service health configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceHealth {
    /// Health check configurations
    pub health_checks: Vec<HealthCheckConfig>,

    /// Liveness probe
    pub liveness_probe: Option<ProbeConfig>,

    /// Readiness probe
    pub readiness_probe: Option<ProbeConfig>,

    /// Startup probe
    pub startup_probe: Option<ProbeConfig>,

    /// Health reporting configuration
    pub reporting: HealthReporting,
}

/// Probe configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeConfig {
    /// Probe handler
    pub handler: ProbeHandler,

    /// Initial delay before starting probes (seconds)
    pub initial_delay_seconds: u32,

    /// Probe interval (seconds)
    pub period_seconds: u32,

    /// Probe timeout (seconds)
    pub timeout_seconds: u32,

    /// Success threshold
    pub success_threshold: u32,

    /// Failure threshold
    pub failure_threshold: u32,

    /// Termination grace period after failure (seconds)
    pub termination_grace_period_seconds: Option<u32>,
}

/// Probe handler
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProbeHandler {
    /// HTTP GET probe
    HttpGet {
        /// Path to request
        path: String,
        /// Port to probe
        port: ProbePort,
        /// Host header
        host: Option<String>,
        /// HTTP scheme
        scheme: HttpScheme,
        /// HTTP headers
        http_headers: Vec<HttpHeader>,
    },

    /// TCP socket probe
    TcpSocket {
        /// Port to probe
        port: ProbePort,
        /// Host to probe
        host: Option<String>,
    },

    /// gRPC probe
    Grpc {
        /// Port to probe
        port: ProbePort,
        /// Service name
        service: Option<String>,
    },

    /// Exec probe
    Exec {
        /// Command to execute
        command: Vec<String>,
    },
}

/// Probe port
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProbePort {
    /// Port number
    Number(u16),
    /// Port name
    Name(String),
}

/// HTTP schemes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HttpScheme {
    /// Plain HTTP
    Http,
    /// HTTPS (TLS)
    Https,
}

/// HTTP header
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpHeader {
    /// Header name
    pub name: String,
    /// Header value
    pub value: String,
}

/// Health reporting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthReporting {
    /// Health reporting enabled
    pub enabled: bool,

    /// Reporting interval (seconds)
    pub interval: u32,

    /// Reporting endpoint
    pub endpoint: Option<String>,

    /// Report format
    pub format: HealthReportFormat,
}

/// Health report formats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthReportFormat {
    /// JSON format
    Json,
    /// Prometheus exposition format
    Prometheus,
    /// Custom format
    Custom(String),
}

/// Health status aggregation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthAggregation {
    /// Aggregation strategy
    pub strategy: AggregationStrategy,

    /// Component weights
    pub weights: HashMap<String, f64>,

    /// Required components
    pub required_components: Vec<String>,

    /// Optional components
    pub optional_components: Vec<String>,
}

/// Aggregation strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AggregationStrategy {
    /// All components must be healthy
    All,

    /// Any component can be healthy
    Any,

    /// Majority of components must be healthy
    Majority,

    /// Weighted average
    Weighted,

    /// Custom aggregation logic
    Custom(String),
}

/// Health check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult {
    /// Check name
    pub name: String,

    /// Check status
    pub status: HealthStatus,

    /// Check message
    pub message: Option<String>,

    /// Check details
    pub details: HashMap<String, serde_json::Value>,

    /// Check timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,

    /// Check duration (milliseconds)
    pub duration_ms: u64,
}

/// Health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthStatus {
    /// Service is healthy
    Healthy,
    /// Service is unhealthy
    Unhealthy,
    /// Service is degraded
    Degraded,
    /// Health status is unknown
    Unknown,
}

/// Health metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthMetrics {
    /// Total health checks
    pub total_checks: u64,

    /// Successful checks
    pub successful_checks: u64,

    /// Failed checks
    pub failed_checks: u64,

    /// Average response time (milliseconds)
    pub avg_response_time_ms: f64,

    /// Uptime percentage
    pub uptime_percentage: f64,

    /// Last failure timestamp
    pub last_failure: Option<chrono::DateTime<chrono::Utc>>,

    /// Last success timestamp
    pub last_success: Option<chrono::DateTime<chrono::Utc>>,
}

/// Health alert configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthAlert {
    /// Alert name
    pub name: String,

    /// Alert condition
    pub condition: AlertCondition,

    /// Alert severity
    pub severity: AlertSeverity,

    /// Alert actions
    pub actions: Vec<AlertAction>,

    /// Alert cooldown (seconds)
    pub cooldown: u32,

    /// Alert enabled
    pub enabled: bool,
}

/// Alert conditions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertCondition {
    /// Health status equals
    StatusEquals(HealthStatus),

    /// Health status not equals
    StatusNotEquals(HealthStatus),

    /// Failure count exceeds threshold
    FailureCountExceeds(u32),

    /// Success rate below threshold
    SuccessRateBelowPercent(f64),

    /// Response time exceeds threshold
    ResponseTimeExceedsMs(u64),

    /// Custom condition
    Custom(String),
}

/// Alert severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    /// Informational alert
    Info,
    /// Warning-level alert
    Warning,
    /// Error-level alert
    Error,
    /// Critical alert
    Critical,
}

/// Alert actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertAction {
    /// Log alert
    Log,

    /// Send notification
    Notify {
        /// Notification channel
        channel: String,
        /// Message template
        template: Option<String>,
    },

    /// Execute command
    Exec {
        /// Command to execute
        command: Vec<String>,
    },

    /// Call webhook
    Webhook {
        /// Webhook URL
        url: String,
        /// HTTP method
        method: String,
        /// Request headers
        headers: HashMap<String, String>,
        /// Request body template
        body_template: Option<String>,
    },

    /// Custom action
    Custom {
        /// Action type
        action_type: String,
        /// Action configuration
        config: HashMap<String, serde_json::Value>,
    },
}

/// Health dashboard configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthDashboard {
    /// Dashboard enabled
    pub enabled: bool,

    /// Dashboard port
    pub port: u16,

    /// Dashboard path
    pub path: String,

    /// Dashboard authentication
    pub auth: Option<DashboardAuth>,

    /// Dashboard theme
    pub theme: DashboardTheme,

    /// Refresh interval (seconds)
    pub refresh_interval: u32,
}

/// Dashboard authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardAuth {
    /// Authentication type
    pub auth_type: DashboardAuthType,

    /// Username (for basic auth)
    pub username: Option<String>,

    /// Password (for basic auth)
    pub password: Option<String>,

    /// API key (for API key auth)
    pub api_key: Option<String>,
}

/// Dashboard authentication types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DashboardAuthType {
    /// No authentication
    None,
    /// Basic (username/password) authentication
    Basic,
    /// API key authentication
    ApiKey,
    /// JWT token authentication
    Jwt,
}

/// Dashboard themes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DashboardTheme {
    /// Light theme
    Light,
    /// Dark theme
    Dark,
    /// Auto-detect from system
    Auto,
    /// Custom theme
    Custom(String),
}

/// Default implementation for ServiceHealth
impl Default for ServiceHealth {
    fn default() -> Self {
        Self {
            health_checks: vec![],
            liveness_probe: None,
            readiness_probe: None,
            startup_probe: None,
            reporting: HealthReporting {
                enabled: true,
                interval: 30,
                endpoint: None,
                format: HealthReportFormat::Json,
            },
        }
    }
}
