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
        started_at: chrono::DateTime<chrono::Utc>,
        timeout_ms: u64,
    },

    /// Running and healthy
    Active {
        since: chrono::DateTime<chrono::Utc>,
        last_health_check: chrono::DateTime<chrono::Utc>,
    },

    /// Running but unhealthy - will attempt resurrection
    Degraded {
        since: chrono::DateTime<chrono::Utc>,
        reason: String,
        resurrection_attempts: u32,
    },

    /// Programmed graceful shutdown
    Apoptosis {
        reason: ApoptosisReason,
        started_at: chrono::DateTime<chrono::Utc>,
    },

    /// Dead - process terminated
    Dead {
        since: chrono::DateTime<chrono::Utc>,
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
