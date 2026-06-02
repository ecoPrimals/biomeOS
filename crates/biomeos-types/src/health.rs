// SPDX-License-Identifier: AGPL-3.0-or-later
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

pub use super::health_check::*;
pub use super::health_metrics::*;

#[cfg(test)]
#[path = "health_tests.rs"]
mod tests;
