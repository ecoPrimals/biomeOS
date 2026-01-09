//! Primal Health Monitoring
//!
//! This module provides continuous health monitoring for primals in the ecosystem,
//! with automatic recovery, alerting, and metrics export.
//!
//! ## Health Check Strategy
//!
//! 1. **Periodic Checks**: Poll primal health endpoints at configurable intervals
//! 2. **State Tracking**: Monitor health state transitions (Healthy → Degraded → Unhealthy)
//! 3. **Threshold-Based**: Only alert after sustained failures (not transient blips)
//! 4. **Auto-Recovery**: Attempt automatic recovery before escalating
//!
//! ## Usage
//!
//! ```ignore
//! use biomeos_core::primal_health::*;
//!
//! let monitor = PrimalHealthMonitor::builder()
//!     .check_interval(Duration::from_secs(30))
//!     .unhealthy_threshold(3)
//!     .recovery_strategy(RecoveryStrategy::Automatic)
//!     .build();
//!
//! // Start monitoring
//! let handle = monitor.start(discovered_primals).await;
//!
//! // Check health status
//! let status = monitor.get_status(&primal_id).await;
//! ```

use crate::adaptive_client::BirdSongError;
use biomeos_types::identifiers::{Endpoint, PrimalId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// Health status of a primal
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum HealthStatus {
    /// Primal is healthy and responsive
    Healthy {
        last_check: u64, // Unix timestamp
        consecutive_successes: usize,
    },

    /// Primal is degraded (responding but with issues)
    Degraded {
        reason: String,
        since: u64, // Unix timestamp
        consecutive_failures: usize,
    },

    /// Primal is unhealthy (not responding or failing)
    Unhealthy {
        reason: String,
        since: u64, // Unix timestamp
        consecutive_failures: usize,
        recovery_attempts: usize,
    },

    /// Health status unknown (never checked)
    Unknown,
}

impl HealthStatus {
    /// Check if primal is considered healthy
    pub fn is_healthy(&self) -> bool {
        matches!(self, HealthStatus::Healthy { .. })
    }

    /// Check if primal is degraded
    pub fn is_degraded(&self) -> bool {
        matches!(self, HealthStatus::Degraded { .. })
    }

    /// Check if primal is unhealthy
    pub fn is_unhealthy(&self) -> bool {
        matches!(self, HealthStatus::Unhealthy { .. })
    }

    /// Get consecutive failure count
    pub fn consecutive_failures(&self) -> usize {
        match self {
            HealthStatus::Degraded { consecutive_failures, .. } => *consecutive_failures,
            HealthStatus::Unhealthy { consecutive_failures, .. } => *consecutive_failures,
            _ => 0,
        }
    }
}

/// Recovery strategy when primal becomes unhealthy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecoveryStrategy {
    /// No automatic recovery, just monitor and alert
    None,

    /// Attempt automatic recovery (restart, reconnect, etc.)
    Automatic,

    /// Manual recovery required, but track attempts
    Manual,
}

/// Configuration for health monitoring
#[derive(Debug, Clone)]
pub struct HealthMonitorConfig {
    /// Interval between health checks
    pub check_interval: Duration,

    /// Number of consecutive failures before marking as unhealthy
    pub unhealthy_threshold: usize,

    /// Number of consecutive failures before marking as degraded
    pub degraded_threshold: usize,

    /// Timeout for health check requests
    pub check_timeout: Duration,

    /// Recovery strategy to use
    pub recovery_strategy: RecoveryStrategy,

    /// Maximum recovery attempts before giving up
    pub max_recovery_attempts: usize,
}

impl Default for HealthMonitorConfig {
    fn default() -> Self {
        Self {
            check_interval: Duration::from_secs(30),
            unhealthy_threshold: 3,
            degraded_threshold: 2,
            check_timeout: Duration::from_secs(5),
            recovery_strategy: RecoveryStrategy::Automatic,
            max_recovery_attempts: 3,
        }
    }
}

/// Primal health monitor
pub struct PrimalHealthMonitor {
    config: HealthMonitorConfig,
    health_states: Arc<RwLock<HashMap<PrimalId, HealthState>>>,
    http_client: reqwest::Client,
}

/// Internal health state tracking
#[derive(Debug, Clone)]
struct HealthState {
    status: HealthStatus,
    endpoint: Endpoint,
    last_check: Instant,
    consecutive_successes: usize,
    consecutive_failures: usize,
    recovery_attempts: usize,
}

impl PrimalHealthMonitor {
    /// Create a new health monitor with default configuration
    pub fn new() -> Self {
        Self::with_config(HealthMonitorConfig::default())
    }

    /// Create a new health monitor with custom configuration
    pub fn with_config(config: HealthMonitorConfig) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(config.check_timeout)
            .build()
            .expect("Failed to create HTTP client");

        Self {
            config,
            health_states: Arc::new(RwLock::new(HashMap::new())),
            http_client,
        }
    }

    /// Builder for custom configuration
    pub fn builder() -> HealthMonitorBuilder {
        HealthMonitorBuilder::default()
    }

    /// Register a primal for monitoring
    pub async fn register(&self, primal_id: PrimalId, endpoint: Endpoint) {
        let mut states = self.health_states.write().await;
        states.insert(
            primal_id.clone(),
            HealthState {
                status: HealthStatus::Unknown,
                endpoint,
                last_check: Instant::now(),
                consecutive_successes: 0,
                consecutive_failures: 0,
                recovery_attempts: 0,
            },
        );
        info!("📊 Registered primal for health monitoring: {}", primal_id);
    }

    /// Unregister a primal from monitoring
    pub async fn unregister(&self, primal_id: &PrimalId) {
        let mut states = self.health_states.write().await;
        states.remove(primal_id);
        info!("📊 Unregistered primal from health monitoring: {}", primal_id);
    }

    /// Check health of a specific primal
    pub async fn check_health(&self, primal_id: &PrimalId) -> Result<HealthStatus, BirdSongError> {
        let states = self.health_states.read().await;
        let state = states.get(primal_id).ok_or_else(|| {
            BirdSongError::Integration(format!("Primal not registered: {}", primal_id))
        })?;

        let endpoint = state.endpoint.clone();
        drop(states); // Release read lock before making HTTP call

        // Attempt health check
        let health_url = format!("{}/health", endpoint.as_str());
        let result = self.http_client.get(&health_url).send().await;

        let is_healthy = result.as_ref()
            .map(|response| response.status().is_success())
            .unwrap_or(false);

        // Update state
        let mut states = self.health_states.write().await;
        if let Some(state) = states.get_mut(primal_id) {
            state.last_check = Instant::now();

            if is_healthy {
                state.consecutive_successes += 1;
                state.consecutive_failures = 0;

                // Transition to healthy if was degraded/unhealthy
                if !state.status.is_healthy() {
                    info!("✅ Primal recovered: {} (after {} failures)", 
                          primal_id, state.consecutive_failures);
                    state.recovery_attempts = 0;
                }

                state.status = HealthStatus::Healthy {
                    last_check: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or(std::time::Duration::from_secs(0))
                        .as_secs(),
                    consecutive_successes: state.consecutive_successes,
                };

                Ok(state.status.clone())
            } else {
                state.consecutive_failures += 1;
                state.consecutive_successes = 0;

                let reason = result
                    .as_ref()
                    .err()
                    .map(|e| e.to_string())
                    .unwrap_or_else(|| "HTTP error".to_string());

                // Determine new status based on failure count
                let new_status = if state.consecutive_failures >= self.config.unhealthy_threshold {
                    warn!("❌ Primal unhealthy: {} ({} consecutive failures)", 
                          primal_id, state.consecutive_failures);

                    HealthStatus::Unhealthy {
                        reason: reason.clone(),
                        since: std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or(std::time::Duration::from_secs(0))
                            .as_secs(),
                        consecutive_failures: state.consecutive_failures,
                        recovery_attempts: state.recovery_attempts,
                    }
                } else if state.consecutive_failures >= self.config.degraded_threshold {
                    warn!("⚠️  Primal degraded: {} ({} consecutive failures)", 
                          primal_id, state.consecutive_failures);

                    HealthStatus::Degraded {
                        reason: reason.clone(),
                        since: std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or(std::time::Duration::from_secs(0))
                            .as_secs(),
                        consecutive_failures: state.consecutive_failures,
                    }
                } else {
                    // Still healthy, but noted failure
                    debug!("Health check failed for {} (attempt {}/{})", 
                           primal_id, state.consecutive_failures, self.config.degraded_threshold);
                    state.status.clone()
                };

                state.status = new_status.clone();

                // Attempt recovery if configured
                if state.status.is_unhealthy() && self.config.recovery_strategy == RecoveryStrategy::Automatic {
                    if state.recovery_attempts < self.config.max_recovery_attempts {
                        state.recovery_attempts += 1;
                        info!("🔧 Attempting automatic recovery for {} (attempt {}/{})",
                              primal_id, state.recovery_attempts, self.config.max_recovery_attempts);
                        
                        // Recovery is handled by external orchestrator via callbacks
                        // The orchestrator monitors health status changes and triggers restarts
                        // This keeps health monitoring decoupled from orchestration logic
                        warn!("Recovery requires orchestrator intervention for {}", primal_id);
                    }
                }

                Ok(new_status)
            }
        } else {
            Err(BirdSongError::Integration(format!(
                "Primal disappeared during check: {}",
                primal_id
            )))
        }
    }

    /// Get current health status for a primal
    pub async fn get_status(&self, primal_id: &PrimalId) -> Option<HealthStatus> {
        let states = self.health_states.read().await;
        states.get(primal_id).map(|s| s.status.clone())
    }

    /// Get all health statuses
    pub async fn get_all_statuses(&self) -> HashMap<PrimalId, HealthStatus> {
        let states = self.health_states.read().await;
        states
            .iter()
            .map(|(id, state)| (id.clone(), state.status.clone()))
            .collect()
    }

    /// Start continuous health monitoring (returns task handle)
    pub fn start_monitoring(self: Arc<Self>) -> tokio::task::JoinHandle<()> {
        let monitor = self.clone();
        tokio::spawn(async move {
            info!("📊 Starting continuous health monitoring (interval: {:?})", 
                  monitor.config.check_interval);

            let mut interval = tokio::time::interval(monitor.config.check_interval);
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            loop {
                interval.tick().await;

                // Get list of primals to check
                let primal_ids: Vec<PrimalId> = {
                    let states = monitor.health_states.read().await;
                    states.keys().cloned().collect()
                };

                // Check each primal concurrently
                let checks: Vec<_> = primal_ids
                    .iter()
                    .map(|id| {
                        let monitor = monitor.clone();
                        let primal_id = id.clone();
                        async move {
                            if let Err(e) = monitor.check_health(&primal_id).await {
                                error!("Health check error for {}: {}", primal_id, e);
                            }
                        }
                    })
                    .collect();

                futures::future::join_all(checks).await;
            }
        })
    }
}

impl Default for PrimalHealthMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for PrimalHealthMonitor
#[derive(Debug, Default)]
pub struct HealthMonitorBuilder {
    config: HealthMonitorConfig,
}

impl HealthMonitorBuilder {
    pub fn check_interval(mut self, interval: Duration) -> Self {
        self.config.check_interval = interval;
        self
    }

    pub fn unhealthy_threshold(mut self, threshold: usize) -> Self {
        self.config.unhealthy_threshold = threshold;
        self
    }

    pub fn degraded_threshold(mut self, threshold: usize) -> Self {
        self.config.degraded_threshold = threshold;
        self
    }

    pub fn check_timeout(mut self, timeout: Duration) -> Self {
        self.config.check_timeout = timeout;
        self
    }

    pub fn recovery_strategy(mut self, strategy: RecoveryStrategy) -> Self {
        self.config.recovery_strategy = strategy;
        self
    }

    pub fn max_recovery_attempts(mut self, max: usize) -> Self {
        self.config.max_recovery_attempts = max;
        self
    }

    pub fn build(self) -> PrimalHealthMonitor {
        PrimalHealthMonitor::with_config(self.config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_status_is_healthy() {
        let healthy = HealthStatus::Healthy {
            last_check: 1234567890,
            consecutive_successes: 5,
        };
        assert!(healthy.is_healthy());
        assert!(!healthy.is_degraded());
        assert!(!healthy.is_unhealthy());
    }

    #[test]
    fn test_health_status_is_degraded() {
        let degraded = HealthStatus::Degraded {
            reason: "slow response".to_string(),
            since: 1234567890,
            consecutive_failures: 2,
        };
        assert!(!degraded.is_healthy());
        assert!(degraded.is_degraded());
        assert!(!degraded.is_unhealthy());
    }

    #[test]
    fn test_health_status_consecutive_failures() {
        let unhealthy = HealthStatus::Unhealthy {
            reason: "not responding".to_string(),
            since: 1234567890,
            consecutive_failures: 5,
            recovery_attempts: 2,
        };
        assert_eq!(unhealthy.consecutive_failures(), 5);
    }

    #[tokio::test]
    async fn test_health_monitor_registration() {
        let monitor = PrimalHealthMonitor::new();
        let primal_id = PrimalId::new("test-primal").unwrap();
        let endpoint = Endpoint::new("http://localhost:9000".to_string()).unwrap();

        monitor.register(primal_id.clone(), endpoint).await;

        let status = monitor.get_status(&primal_id).await;
        assert!(status.is_some());
        assert!(matches!(status.unwrap(), HealthStatus::Unknown));
    }

    #[tokio::test]
    async fn test_health_monitor_unregister() {
        let monitor = PrimalHealthMonitor::new();
        let primal_id = PrimalId::new("test-primal").unwrap();
        let endpoint = Endpoint::new("http://localhost:9000".to_string()).unwrap();

        monitor.register(primal_id.clone(), endpoint).await;
        monitor.unregister(&primal_id).await;

        let status = monitor.get_status(&primal_id).await;
        assert!(status.is_none());
    }

    #[test]
    fn test_builder_configuration() {
        let monitor = PrimalHealthMonitor::builder()
            .check_interval(Duration::from_secs(60))
            .unhealthy_threshold(5)
            .degraded_threshold(3)
            .recovery_strategy(RecoveryStrategy::Manual)
            .build();

        assert_eq!(monitor.config.check_interval, Duration::from_secs(60));
        assert_eq!(monitor.config.unhealthy_threshold, 5);
        assert_eq!(monitor.config.degraded_threshold, 3);
        assert_eq!(monitor.config.recovery_strategy, RecoveryStrategy::Manual);
    }
}

