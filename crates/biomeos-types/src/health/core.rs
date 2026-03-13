// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Core Health Status Types
//!
//! Defines the primary Health enum and its core functionality.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::issues::HealthIssue;
use super::lifecycle::{MaintenanceType, ShutdownPhase, StartupPhase};

/// Universal Health Status
///
/// This replaces PrimalHealth, HealthStatus, and other health enums
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
        last_known: Option<Box<Health>>,
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
    pub fn healthy() -> Self {
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
    pub fn critical(issues: Vec<HealthIssue>, affected_capabilities: Vec<String>) -> Self {
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
    #[must_use]
    pub fn unknown(reason: impl Into<String>) -> Self {
        Self::Unknown {
            reason: reason.into(),
            last_known: None,
        }
    }

    /// Check if the system is healthy
    #[must_use]
    pub fn is_healthy(&self) -> bool {
        matches!(self, Self::Healthy)
    }

    /// Check if the system is operational (healthy or degraded)
    #[must_use]
    pub fn is_operational(&self) -> bool {
        matches!(self, Self::Healthy | Self::Degraded { .. })
    }

    /// Check if the system is in a terminal state (unhealthy, critical)
    #[must_use]
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Unhealthy { .. } | Self::Critical { .. })
    }

    /// Check if the system is in transition (starting, stopping)
    #[must_use]
    pub fn is_transitioning(&self) -> bool {
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

impl Default for Health {
    fn default() -> Self {
        Self::Unknown {
            reason: "Not yet initialized".to_string(),
            last_known: None,
        }
    }
}

