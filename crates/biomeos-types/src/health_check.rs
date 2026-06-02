// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Health check configuration and threshold types
//!
//! Extracted from `health.rs` for cohesion. Re-exported via `health` module.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::health::Health;

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

/// Health event for tracking health changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthEvent {
    /// Event identifier
    pub id: uuid::Uuid,

    /// Event timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,

    /// Previous health status
    pub previous_health: Option<Health>,

    /// New health status
    pub new_health: Health,

    /// Event trigger
    pub trigger: HealthEventTrigger,

    /// Additional event context
    pub context: HashMap<String, serde_json::Value>,
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
