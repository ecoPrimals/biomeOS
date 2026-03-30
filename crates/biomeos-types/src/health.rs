// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Unified Health Monitoring System
//!
//! This module consolidates all health-related types that were previously
//! scattered across multiple crates (`PrimalHealth`, `HealthStatus`, `SystemHealth`, etc.).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Universal Health Status
///
/// This replaces `PrimalHealth`, `HealthStatus`, and other health enums
/// with a unified, comprehensive health status system.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Health {
    /// System is fully operational
    Healthy,

    /// System is operational but with degraded performance
    Degraded {
        /// Issues affecting performance
        issues: Vec<HealthIssue>,
        /// Performance impact score (0.0-1.0)
        impact_score: Option<f64>,
    },

    /// System is experiencing critical issues but still partially functional
    Critical {
        /// Critical issues
        issues: Vec<HealthIssue>,
        /// Affected capabilities
        affected_capabilities: Vec<String>,
    },

    /// System is completely non-functional
    Unhealthy {
        /// Issues causing system failure
        issues: Vec<HealthIssue>,
        /// Time when system became unhealthy
        failed_at: DateTime<Utc>,
    },

    /// Health status cannot be determined
    Unknown {
        /// Reason why health is unknown
        reason: String,
        /// Last known health status
        last_known: Option<Box<Self>>,
    },

    /// System is starting up
    Starting {
        /// Startup phase
        phase: StartupPhase,
        /// Progress percentage (0-100)
        progress: u8,
    },

    /// System is shutting down
    Stopping {
        /// Shutdown phase
        phase: ShutdownPhase,
        /// Progress percentage (0-100)
        progress: u8,
    },

    /// System is under maintenance
    Maintenance {
        /// Maintenance type
        maintenance_type: MaintenanceType,
        /// Estimated completion time
        estimated_completion: Option<DateTime<Utc>>,
    },
}

impl Health {
    /// Create a healthy status
    #[must_use] 
    pub const fn healthy() -> Self {
        Self::Healthy
    }

    /// Create a degraded status with issues
    #[must_use] 
    pub fn degraded(issues: Vec<HealthIssue>) -> Self {
        let impact_score = Self::calculate_impact_score(&issues);
        Self::Degraded {
            issues,
            impact_score: Some(impact_score),
        }
    }

    /// Create a critical status
    #[must_use] 
    pub const fn critical(issues: Vec<HealthIssue>, affected_capabilities: Vec<String>) -> Self {
        Self::Critical {
            issues,
            affected_capabilities,
        }
    }

    /// Create an unhealthy status
    #[must_use] 
    pub fn unhealthy(issues: Vec<HealthIssue>) -> Self {
        Self::Unhealthy {
            issues,
            failed_at: Utc::now(),
        }
    }

    /// Create an unknown status
    pub fn unknown(reason: impl Into<String>) -> Self {
        Self::Unknown {
            reason: reason.into(),
            last_known: None,
        }
    }

    /// Check if the system is healthy
    #[must_use] 
    pub const fn is_healthy(&self) -> bool {
        matches!(self, Self::Healthy)
    }

    /// Check if the system is operational (healthy or degraded)
    #[must_use] 
    pub const fn is_operational(&self) -> bool {
        matches!(self, Self::Healthy | Self::Degraded { .. })
    }

    /// Check if the system is in a terminal state (unhealthy, critical)
    #[must_use] 
    pub const fn is_terminal(&self) -> bool {
        matches!(self, Self::Unhealthy { .. } | Self::Critical { .. })
    }

    /// Check if the system is in transition (starting, stopping)
    #[must_use] 
    pub const fn is_transitioning(&self) -> bool {
        matches!(self, Self::Starting { .. } | Self::Stopping { .. })
    }

    /// Get the health score (0.0 = unhealthy, 1.0 = healthy)
    #[must_use] 
    pub fn score(&self) -> f64 {
        match self {
            Self::Healthy => 1.0,
            Self::Degraded { impact_score, .. } => 1.0 - impact_score.unwrap_or(0.3),
            Self::Critical { .. } => 0.2,
            Self::Unhealthy { .. } => 0.0,
            Self::Unknown { .. } => 0.5,
            Self::Starting { progress, .. } => f64::from(*progress) / 100.0 * 0.8,
            Self::Stopping { progress, .. } => f64::from(100 - *progress) / 100.0 * 0.3,
            Self::Maintenance { .. } => 0.6,
        }
    }

    /// Get all issues affecting this health status
    #[must_use] 
    pub fn issues(&self) -> Vec<&HealthIssue> {
        match self {
            Self::Degraded { issues, .. }
            | Self::Critical { issues, .. }
            | Self::Unhealthy { issues, .. } => issues.iter().collect(),
            _ => vec![],
        }
    }

    /// Calculate impact score from issues
    #[expect(
        clippy::cast_precision_loss,
        reason = "usize issue count cast to f64 for mean impact score"
    )]
    fn calculate_impact_score(issues: &[HealthIssue]) -> f64 {
        if issues.is_empty() {
            return 0.0;
        }

        let total_impact: f64 = issues
            .iter()
            .map(|issue| issue.severity.impact_score())
            .sum();

        (total_impact / issues.len() as f64).min(1.0)
    }
}

/// Health issue details
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HealthIssue {
    /// Unique issue identifier
    pub id: String,

    /// Issue category
    pub category: HealthIssueCategory,

    /// Issue severity
    pub severity: HealthIssueSeverity,

    /// Human-readable message
    pub message: String,

    /// When the issue was first detected
    pub detected_at: DateTime<Utc>,

    /// Issue-specific details
    pub details: HashMap<String, serde_json::Value>,

    /// Suggested remediation actions
    pub remediation: Vec<RemediationAction>,
}

/// Health issue categories
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum HealthIssueCategory {
    /// Resource-related issues (CPU, memory, disk, network)
    Resource,

    /// Performance issues (latency, throughput)
    Performance,

    /// Configuration problems
    Configuration,

    /// Dependency issues (external services, databases)
    Dependency,

    /// Security issues
    Security,

    /// Network connectivity issues
    Network,

    /// Authentication/authorization issues
    Authentication,

    /// Data integrity issues
    Data,

    /// Hardware issues
    Hardware,

    /// Software bugs or errors
    Software,

    /// Custom category
    Custom {
        /// Custom category name
        category: String,
    },
}

/// Health issue severity levels
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum HealthIssueSeverity {
    /// Minor issue with minimal impact
    Low,

    /// Moderate issue with noticeable impact
    Medium,

    /// Serious issue with significant impact
    High,

    /// Critical issue requiring immediate attention
    Critical,

    /// Emergency - system failure imminent or occurred
    Emergency,
}

impl HealthIssueSeverity {
    /// Get the impact score for this severity (0.0-1.0)
    #[must_use] 
    pub const fn impact_score(&self) -> f64 {
        match self {
            Self::Low => 0.1,
            Self::Medium => 0.3,
            Self::High => 0.6,
            Self::Critical => 0.8,
            Self::Emergency => 1.0,
        }
    }
}

/// Remediation action
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct RemediationAction {
    /// Action identifier
    pub id: String,

    /// Action type
    pub action_type: RemediationActionType,

    /// Human-readable description
    pub description: String,

    /// Whether this action can be automated
    pub automated: bool,

    /// Command or script to execute (if automated)
    pub command: Option<String>,

    /// Estimated time to complete (in seconds)
    pub estimated_duration_secs: Option<u32>,
}

/// Types of remediation actions
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum RemediationActionType {
    /// Restart a service or component
    Restart,

    /// Reconfigure a setting
    Reconfigure,

    /// Scale resources up or down
    Scale,

    /// Clear cache or temporary data
    Clear,

    /// Update configuration
    Update,

    /// Replace failed component
    Replace,

    /// Manual intervention required
    Manual,

    /// Custom action
    Custom {
        /// Custom action type name
        action_type: String,
    },
}

/// Startup phases
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum StartupPhase {
    /// Initial boot sequence
    Initializing,
    /// Loading configuration files and settings
    LoadingConfiguration,
    /// Connecting to required dependencies
    ConnectingDependencies,
    /// Starting internal services
    StartingServices,
    /// Executing health check probes
    RunningHealthChecks,
    /// Startup complete and ready to serve
    Ready,
}

/// Shutdown phases
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum ShutdownPhase {
    /// Shutdown has been initiated
    Initiated,
    /// Dropping active connections
    DroppingConnections,
    /// Flushing buffered data to storage
    FlushingData,
    /// Stopping running services
    StoppingServices,
    /// Cleaning up temporary resources
    Cleanup,
    /// Shutdown complete
    Stopped,
}

/// Maintenance types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum MaintenanceType {
    /// Planned maintenance window
    Planned,

    /// Emergency maintenance
    Emergency,

    /// Security updates
    Security,

    /// Performance optimization
    Performance,

    /// Configuration updates
    Configuration,

    /// Custom maintenance
    Custom {
        /// Custom maintenance type name
        maintenance_type: String,
    },
}

/// Comprehensive health report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthReport {
    /// Report identifier
    pub id: Uuid,

    /// Subject of the health report (service, primal, system)
    pub subject: HealthSubject,

    /// Overall health status
    pub health: Health,

    /// Individual component health
    pub components: HashMap<String, ComponentHealth>,

    /// Health metrics
    pub metrics: HealthMetrics,

    /// Health history (recent changes)
    pub history: Vec<HealthEvent>,

    /// Report generation timestamp
    pub generated_at: DateTime<Utc>,

    /// Next health check scheduled time
    pub next_check_at: Option<DateTime<Utc>>,
}

/// Subject of health monitoring
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HealthSubject {
    /// Subject identifier
    pub id: String,

    /// Subject type
    pub subject_type: HealthSubjectType,

    /// Subject name
    pub name: String,

    /// Subject version
    pub version: String,
}

/// Types of health subjects
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthSubjectType {
    /// Individual primal
    Primal,

    /// Service instance
    Service,

    /// Entire system/cluster
    System,

    /// Component within a service
    Component,

    /// Custom subject type
    Custom {
        /// Custom subject type name
        subject_type: String,
    },
}

/// Component health information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    /// Component name
    pub name: String,

    /// Component health status
    pub health: Health,

    /// Component metrics
    pub metrics: HashMap<String, serde_json::Value>,

    /// Last health check time
    pub last_check: DateTime<Utc>,
}

/// Health metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthMetrics {
    /// Response time metrics (milliseconds)
    pub response_time: Option<ResponseTimeMetrics>,

    /// Resource utilization metrics
    pub resources: Option<ResourceMetrics>,

    /// Error rate metrics
    pub errors: Option<ErrorMetrics>,

    /// Availability metrics
    pub availability: Option<AvailabilityMetrics>,

    /// Custom metrics
    pub custom: HashMap<String, serde_json::Value>,
}

/// Response time metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseTimeMetrics {
    /// Average response time in milliseconds
    pub average_ms: f64,
    /// 50th percentile (median) response time in milliseconds
    pub p50_ms: f64,
    /// 95th percentile response time in milliseconds
    pub p95_ms: f64,
    /// 99th percentile response time in milliseconds
    pub p99_ms: f64,
    /// Maximum response time in milliseconds
    pub max_ms: f64,
}

/// Resource utilization metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceMetrics {
    /// CPU utilization (0.0-1.0)
    pub cpu_usage: Option<f64>,

    /// Memory utilization (0.0-1.0)
    pub memory_usage: Option<f64>,

    /// Disk utilization (0.0-1.0)
    pub disk_usage: Option<f64>,

    /// Network utilization (bytes/sec)
    pub network_io: Option<NetworkIoMetrics>,
}

/// Network I/O metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkIoMetrics {
    /// Inbound bytes per second
    pub bytes_in_per_sec: f64,
    /// Outbound bytes per second
    pub bytes_out_per_sec: f64,
    /// Inbound packets per second
    pub packets_in_per_sec: f64,
    /// Outbound packets per second
    pub packets_out_per_sec: f64,
}

/// Error rate metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorMetrics {
    /// Total error rate (errors/second)
    pub error_rate: f64,

    /// Error rate by category
    pub errors_by_category: HashMap<String, f64>,

    /// Recent error count
    pub recent_errors: u64,
}

/// Availability metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvailabilityMetrics {
    /// Uptime percentage (0.0-1.0)
    pub uptime_percentage: f64,

    /// Total uptime in seconds
    pub uptime_seconds: u64,

    /// Total downtime in seconds
    pub downtime_seconds: u64,

    /// Number of outages
    pub outage_count: u64,

    /// Mean time to recovery (MTTR) in seconds
    pub mttr_seconds: Option<f64>,
}

/// Health event for tracking health changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthEvent {
    /// Event identifier
    pub id: Uuid,

    /// Event timestamp
    pub timestamp: DateTime<Utc>,

    /// Previous health status
    pub previous_health: Option<Health>,

    /// New health status
    pub new_health: Health,

    /// Event trigger
    pub trigger: HealthEventTrigger,

    /// Additional event context
    pub context: HashMap<String, serde_json::Value>,
}

/// Health event triggers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthEventTrigger {
    /// Scheduled health check
    ScheduledCheck,

    /// Manual health check
    ManualCheck,

    /// Metric threshold exceeded
    MetricThreshold {
        /// Name of the metric
        metric: String,
        /// Threshold value that was exceeded
        threshold: f64,
    },

    /// External event
    ExternalEvent {
        /// Source of the external event
        source: String,
    },

    /// System startup
    Startup,

    /// System shutdown
    Shutdown,

    /// Configuration change
    ConfigurationChange,

    /// Custom trigger
    Custom {
        /// Custom trigger identifier
        trigger: String,
    },
}

/// Health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    /// Check interval in seconds
    pub interval_secs: u32,

    /// Check timeout in seconds
    pub timeout_secs: u32,

    /// Number of consecutive failures before marking unhealthy
    pub failure_threshold: u32,

    /// Number of consecutive successes before marking healthy
    pub success_threshold: u32,

    /// Health check endpoint or command
    pub check_target: HealthCheckTarget,

    /// Expected response for healthy status
    pub expected_response: Option<String>,

    /// Metric thresholds
    pub metric_thresholds: HashMap<String, MetricThreshold>,
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        use crate::constants::timeouts::{
            DEFAULT_HEALTH_CHECK_INTERVAL, DEFAULT_HEALTH_CHECK_TIMEOUT,
        };

        Self {
            interval_secs: u32::try_from(DEFAULT_HEALTH_CHECK_INTERVAL.as_secs())
                .unwrap_or(u32::MAX),
            timeout_secs: u32::try_from(DEFAULT_HEALTH_CHECK_TIMEOUT.as_secs()).unwrap_or(u32::MAX),
            failure_threshold: 3,
            success_threshold: 1,
            check_target: HealthCheckTarget::Http {
                url: "/health".to_string(),
                method: "GET".to_string(),
            },
            expected_response: Some("OK".to_string()),
            metric_thresholds: HashMap::new(),
        }
    }
}

/// Health check targets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthCheckTarget {
    /// HTTP endpoint
    Http {
        /// URL path to check
        url: String,
        /// HTTP method to use
        method: String,
    },

    /// TCP port check
    Tcp {
        /// Hostname or IP to connect to
        host: String,
        /// Port number to check
        port: u16,
    },

    /// Command execution
    Command {
        /// Command to execute
        command: String,
        /// Command arguments
        args: Vec<String>,
    },

    /// Function call
    Function {
        /// Function name to invoke
        function: String,
    },

    /// Custom check
    Custom {
        /// Custom check target identifier
        target: String,
    },
}

/// Metric threshold configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricThreshold {
    /// Threshold value
    pub value: f64,

    /// Comparison operator
    pub operator: ThresholdOperator,

    /// Action to take when threshold is exceeded
    pub action: ThresholdAction,
}

/// Threshold comparison operators
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThresholdOperator {
    /// Value is greater than threshold
    GreaterThan,
    /// Value is less than threshold
    LessThan,
    /// Value equals threshold
    Equal,
    /// Value does not equal threshold
    NotEqual,
}

/// Actions to take when threshold is exceeded
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThresholdAction {
    /// Mark as degraded
    MarkDegraded,

    /// Mark as critical
    MarkCritical,

    /// Mark as unhealthy
    MarkUnhealthy,

    /// Trigger alert
    TriggerAlert {
        /// Type of alert to trigger
        alert_type: String,
    },

    /// Execute remediation
    ExecuteRemediation {
        /// ID of the remediation action to execute
        action_id: String,
    },
}

#[cfg(test)]
#[path = "health_tests.rs"]
mod tests;
