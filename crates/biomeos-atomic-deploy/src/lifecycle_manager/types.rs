// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Lifecycle types: states, configs, and managed primal structures

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;

use crate::neural_graph::GraphNode;

/// Lifecycle state of a primal
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum LifecycleState {
    /// Birth - primal being spawned with minimal knowledge
    Germinating,

    /// Startup monitoring - waiting for socket and health
    Incubating {
        /// When incubation started
        started_at: chrono::DateTime<chrono::Utc>,
        /// Timeout in ms before marking failed
        timeout_ms: u64,
    },

    /// Running and healthy
    Active {
        /// When primal became active
        since: chrono::DateTime<chrono::Utc>,
        /// Last successful health check
        last_health_check: chrono::DateTime<chrono::Utc>,
    },

    /// Running but unhealthy - will attempt resurrection
    Degraded {
        /// When degradation was detected
        since: chrono::DateTime<chrono::Utc>,
        /// Failure reason
        reason: String,
        /// Resurrection attempts so far
        resurrection_attempts: u32,
    },

    /// Programmed graceful shutdown
    Apoptosis {
        /// Why apoptosis was triggered
        reason: ApoptosisReason,
        /// When shutdown started
        started_at: chrono::DateTime<chrono::Utc>,
    },

    /// Dead - process terminated
    Dead {
        /// When death was detected
        since: chrono::DateTime<chrono::Utc>,
        /// Death reason
        reason: String,
    },
}

/// Reason for programmed death (apoptosis)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ApoptosisReason {
    /// User requested shutdown
    UserRequest,
    /// Ecosystem health requires it
    EcosystemHealth,
    /// Resource pressure (memory, CPU)
    ResourcePressure,
    /// Dependency died
    DependencyDeath(String),
    /// Too many resurrection failures
    ResurrectionExhausted,
    /// System shutdown
    SystemShutdown,
}

/// A primal being managed by the lifecycle manager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManagedPrimal {
    /// Primal name (e.g., "beardog", "songbird")
    pub name: String,

    /// Family ID
    pub family_id: String,

    /// Socket path
    pub socket_path: PathBuf,

    /// Process ID (if running)
    pub pid: Option<u32>,

    /// Current lifecycle state
    pub state: LifecycleState,

    /// Deployment graph node (for resurrection)
    pub deployment_node: Option<GraphNode>,

    /// Dependencies (primal names this depends on)
    pub depends_on: Vec<String>,

    /// Dependents (primal names that depend on this)
    pub depended_by: Vec<String>,

    /// Health check configuration
    pub health_config: HealthConfig,

    /// Resurrection configuration
    pub resurrection_config: ResurrectionConfig,

    /// Metrics
    pub metrics: PrimalMetrics,
}

/// Health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthConfig {
    /// Health check interval
    pub check_interval: Duration,

    /// Timeout for health check response
    pub timeout: Duration,

    /// Number of consecutive failures before marking degraded
    pub failure_threshold: u32,

    /// JSON-RPC method to use for health check
    pub health_method: String,
}

impl Default for HealthConfig {
    fn default() -> Self {
        Self {
            check_interval: Duration::from_secs(30),
            timeout: Duration::from_secs(5),
            failure_threshold: 3,
            health_method: "health".to_string(),
        }
    }
}

/// Resurrection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResurrectionConfig {
    /// Whether auto-resurrection is enabled
    pub enabled: bool,

    /// Maximum resurrection attempts
    pub max_attempts: u32,

    /// Base delay between attempts (exponential backoff)
    pub base_delay: Duration,

    /// Maximum delay between attempts
    pub max_delay: Duration,
}

impl Default for ResurrectionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_attempts: 5,
            base_delay: Duration::from_secs(2),
            max_delay: Duration::from_secs(60),
        }
    }
}

/// Primal metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PrimalMetrics {
    /// Total uptime (excluding dead time)
    pub total_uptime_secs: u64,

    /// Number of resurrections
    pub resurrection_count: u32,

    /// Number of health check failures
    pub health_failures: u32,

    /// Last health check latency (ms)
    pub last_health_latency_ms: u64,

    /// Requests served (if available)
    pub requests_served: u64,
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_health_config_default() {
        let config = HealthConfig::default();
        assert_eq!(config.check_interval, Duration::from_secs(30));
        assert_eq!(config.timeout, Duration::from_secs(5));
        assert_eq!(config.failure_threshold, 3);
        assert_eq!(config.health_method, "health");
    }

    #[test]
    fn test_resurrection_config_default() {
        let config = ResurrectionConfig::default();
        assert!(config.enabled);
        assert_eq!(config.max_attempts, 5);
        assert_eq!(config.base_delay, Duration::from_secs(2));
        assert_eq!(config.max_delay, Duration::from_secs(60));
    }

    #[test]
    fn test_primal_metrics_default() {
        let metrics = PrimalMetrics::default();
        assert_eq!(metrics.total_uptime_secs, 0);
        assert_eq!(metrics.resurrection_count, 0);
        assert_eq!(metrics.health_failures, 0);
    }

    #[test]
    fn test_apoptosis_reason_serde_roundtrip() {
        for reason in [
            ApoptosisReason::UserRequest,
            ApoptosisReason::EcosystemHealth,
            ApoptosisReason::ResourcePressure,
            ApoptosisReason::DependencyDeath("beardog".to_string()),
            ApoptosisReason::ResurrectionExhausted,
            ApoptosisReason::SystemShutdown,
        ] {
            let json = serde_json::to_string(&reason).unwrap();
            let parsed: ApoptosisReason = serde_json::from_str(&json).unwrap();
            assert_eq!(reason, parsed);
        }
    }

    #[test]
    fn test_lifecycle_state_germinating_serde() {
        let state = LifecycleState::Germinating;
        let json = serde_json::to_string(&state).unwrap();
        let parsed: LifecycleState = serde_json::from_str(&json).unwrap();
        assert_eq!(state, parsed);
    }

    #[test]
    fn test_lifecycle_state_incubating_serde() {
        let now = chrono::Utc::now();
        let state = LifecycleState::Incubating {
            started_at: now,
            timeout_ms: 5000,
        };
        let json = serde_json::to_string(&state).unwrap();
        let parsed: LifecycleState = serde_json::from_str(&json).unwrap();
        assert_eq!(state, parsed);
    }

    #[test]
    fn test_lifecycle_state_apoptosis_serde() {
        let now = chrono::Utc::now();
        let state = LifecycleState::Apoptosis {
            reason: ApoptosisReason::UserRequest,
            started_at: now,
        };
        let json = serde_json::to_string(&state).unwrap();
        let parsed: LifecycleState = serde_json::from_str(&json).unwrap();
        assert_eq!(state, parsed);
    }

    #[test]
    fn test_managed_primal_serde_roundtrip() {
        let primal = ManagedPrimal {
            name: "beardog".to_string(),
            family_id: "fam-1".to_string(),
            socket_path: PathBuf::from("/tmp/beardog.sock"),
            pid: Some(1234),
            state: LifecycleState::Germinating,
            deployment_node: None,
            depends_on: vec![],
            depended_by: vec!["songbird".to_string()],
            health_config: HealthConfig::default(),
            resurrection_config: ResurrectionConfig::default(),
            metrics: PrimalMetrics::default(),
        };
        let json = serde_json::to_string(&primal).unwrap();
        let parsed: ManagedPrimal = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name, primal.name);
        assert_eq!(parsed.pid, primal.pid);
    }
}
