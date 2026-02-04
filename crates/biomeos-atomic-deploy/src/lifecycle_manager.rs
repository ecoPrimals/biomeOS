//! Primal Lifecycle Manager
//!
//! Robust lifecycle management for NUCLEUS (Tower, Node, Nest) deployments:
//! - **Germination**: Birth primal with minimal knowledge
//! - **Incubation**: Health monitoring during startup
//! - **Active**: Running and healthy
//! - **Degraded**: Running but unhealthy (will attempt resurrection)
//! - **Apoptosis**: Programmed graceful shutdown
//! - **Resurrection**: Automatic restart from deployment graph
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                   PRIMAL LIFECYCLE MANAGER                       │
//! ├─────────────────────────────────────────────────────────────────┤
//! │                                                                 │
//! │  GERMINATION → INCUBATION → ACTIVE ←→ DEGRADED → APOPTOSIS     │
//! │       ↑                        ↓                    ↓          │
//! │       └────── RESURRECTION ←───┴────────────────────┘          │
//! │                                                                 │
//! └─────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Key Features
//!
//! - **Health Monitoring**: Configurable interval JSON-RPC pings
//! - **Crash Detection**: Socket timeout or process death
//! - **Auto-Resurrection**: Restart from retained deployment graph
//! - **Dependency Awareness**: Respects primal dependency order
//! - **Graceful Apoptosis**: Coordinated shutdown with cleanup

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use crate::health_check::HealthChecker;
use crate::neural_graph::{Graph, GraphNode};
use crate::nucleation::SocketNucleation;

// ============================================================================
// LIFECYCLE STATES
// ============================================================================

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

// ============================================================================
// MANAGED PRIMAL
// ============================================================================

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

// ============================================================================
// LIFECYCLE MANAGER
// ============================================================================

/// Primal Lifecycle Manager
///
/// Manages the lifecycle of all primals in a NUCLEUS deployment:
/// - Monitors health and detects crashes
/// - Resurrects dead primals from deployment graphs
/// - Coordinates graceful shutdown (apoptosis)
pub struct LifecycleManager {
    /// Managed primals (name -> ManagedPrimal)
    primals: Arc<RwLock<HashMap<String, ManagedPrimal>>>,

    /// Family ID
    family_id: String,

    /// Socket nucleation for deterministic paths
    nucleation: Arc<RwLock<SocketNucleation>>,

    /// Health checker
    health_checker: HealthChecker,

    /// Deployment graphs (for resurrection)
    deployment_graphs: Arc<RwLock<HashMap<String, Graph>>>,

    /// Global health check interval
    health_check_interval: Duration,

    /// Shutdown flag
    shutdown: Arc<RwLock<bool>>,
}

impl LifecycleManager {
    /// Create a new lifecycle manager
    pub fn new(family_id: impl Into<String>) -> Self {
        let family_id = family_id.into();

        Self {
            primals: Arc::new(RwLock::new(HashMap::new())),
            family_id,
            nucleation: Arc::new(RwLock::new(SocketNucleation::default())),
            health_checker: HealthChecker::new(PathBuf::from("/tmp")),
            deployment_graphs: Arc::new(RwLock::new(HashMap::new())),
            health_check_interval: Duration::from_secs(10),
            shutdown: Arc::new(RwLock::new(false)),
        }
    }

    /// Create with custom configuration
    pub fn with_config(
        family_id: impl Into<String>,
        health_check_interval: Duration,
        nucleation: Arc<RwLock<SocketNucleation>>,
    ) -> Self {
        let family_id = family_id.into();

        Self {
            primals: Arc::new(RwLock::new(HashMap::new())),
            family_id,
            nucleation,
            health_checker: HealthChecker::new(PathBuf::from("/tmp")),
            deployment_graphs: Arc::new(RwLock::new(HashMap::new())),
            health_check_interval,
            shutdown: Arc::new(RwLock::new(false)),
        }
    }

    // ========================================================================
    // GERMINATION
    // ========================================================================

    /// Register a primal for lifecycle management
    ///
    /// Called after germination to track the primal
    pub async fn register_primal(
        &self,
        name: impl Into<String>,
        socket_path: PathBuf,
        pid: Option<u32>,
        deployment_node: Option<GraphNode>,
    ) -> Result<()> {
        let name = name.into();
        let depends_on = deployment_node
            .as_ref()
            .map(|n| n.depends_on.clone())
            .unwrap_or_default();

        let primal = ManagedPrimal {
            name: name.clone(),
            family_id: self.family_id.clone(),
            socket_path,
            pid,
            state: LifecycleState::Incubating {
                started_at: chrono::Utc::now(),
                timeout_ms: 30000,
            },
            deployment_node,
            depends_on,
            depended_by: Vec::new(),
            health_config: HealthConfig::default(),
            resurrection_config: ResurrectionConfig::default(),
            metrics: PrimalMetrics::default(),
        };

        {
            let mut primals = self.primals.write().await;
            primals.insert(name.clone(), primal);
            info!("🌱 Registered primal: {} (incubating)", name);
        } // Release write lock before calling update_dependency_graph

        // Update dependency graph (requires its own write lock)
        self.update_dependency_graph().await;

        Ok(())
    }

    /// Store deployment graph for resurrection
    pub async fn store_deployment_graph(&self, graph_id: impl Into<String>, graph: Graph) {
        let graph_id = graph_id.into();
        let mut graphs = self.deployment_graphs.write().await;
        graphs.insert(graph_id.clone(), graph);
        info!(
            "📋 Stored deployment graph: {} (for resurrection)",
            graph_id
        );
    }

    // ========================================================================
    // MONITORING LOOP
    // ========================================================================

    /// Start the health monitoring loop
    ///
    /// This runs continuously, checking primal health and triggering
    /// resurrection when needed.
    pub async fn start_monitoring(&self) -> Result<()> {
        info!(
            "🔍 Starting lifecycle monitoring (interval: {:?})",
            self.health_check_interval
        );

        let primals = self.primals.clone();
        let shutdown = self.shutdown.clone();
        let _health_checker = self.health_checker.clone();
        let interval = self.health_check_interval;
        let manager = self.clone_for_task();

        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);

            loop {
                interval_timer.tick().await;

                // Check shutdown flag
                if *shutdown.read().await {
                    info!("🛑 Monitoring loop shutting down");
                    break;
                }

                // Check health of all primals
                let primal_names: Vec<String> = {
                    let primals = primals.read().await;
                    primals.keys().cloned().collect()
                };

                for name in primal_names {
                    if let Err(e) = manager.check_primal_health(&name).await {
                        warn!("Health check failed for {}: {}", name, e);
                    }
                }
            }
        });

        Ok(())
    }

    /// Check health of a single primal
    async fn check_primal_health(&self, name: &str) -> Result<()> {
        let mut primals = self.primals.write().await;
        let primal = primals
            .get_mut(name)
            .ok_or_else(|| anyhow::anyhow!("Primal not found: {}", name))?;

        // Skip if not in active or incubating state
        match &primal.state {
            LifecycleState::Active { .. } | LifecycleState::Incubating { .. } => {}
            LifecycleState::Degraded { .. } => {
                // Already degraded, resurrection handler will manage
                return Ok(());
            }
            _ => return Ok(()),
        }

        let start = Instant::now();

        // Check socket health
        let health_result = self
            .health_checker
            .check_primal(&primal.socket_path)
            .await?;

        primal.metrics.last_health_latency_ms = start.elapsed().as_millis() as u64;

        if health_result.is_healthy {
            // Transition to active if incubating
            match &primal.state {
                LifecycleState::Incubating { .. } => {
                    info!("✅ {} transitioned to ACTIVE", name);
                    primal.state = LifecycleState::Active {
                        since: chrono::Utc::now(),
                        last_health_check: chrono::Utc::now(),
                    };
                }
                LifecycleState::Active { since, .. } => {
                    primal.state = LifecycleState::Active {
                        since: *since,
                        last_health_check: chrono::Utc::now(),
                    };
                }
                _ => {}
            }

            debug!(
                "💚 {} healthy ({}ms)",
                name, primal.metrics.last_health_latency_ms
            );
        } else {
            // Health check failed
            primal.metrics.health_failures += 1;

            if primal.metrics.health_failures >= primal.health_config.failure_threshold {
                warn!(
                    "🔴 {} DEGRADED after {} failures: {:?}",
                    name, primal.metrics.health_failures, health_result.message
                );

                primal.state = LifecycleState::Degraded {
                    since: chrono::Utc::now(),
                    reason: health_result
                        .message
                        .unwrap_or_else(|| "Health check failed".to_string()),
                    resurrection_attempts: 0,
                };

                // Drop lock before spawning resurrection
                let name_clone = name.to_string();
                let manager = self.clone_for_task();
                drop(primals);

                // Trigger resurrection
                tokio::spawn(async move {
                    if let Err(e) = manager.attempt_resurrection(&name_clone).await {
                        error!("Resurrection failed for {}: {}", name_clone, e);
                    }
                });
            }
        }

        Ok(())
    }

    // ========================================================================
    // RESURRECTION
    // ========================================================================

    /// Attempt to resurrect a degraded/dead primal
    async fn attempt_resurrection(&self, name: &str) -> Result<()> {
        let mut primals = self.primals.write().await;
        let primal = primals
            .get_mut(name)
            .ok_or_else(|| anyhow::anyhow!("Primal not found: {}", name))?;

        // Check if resurrection is enabled
        if !primal.resurrection_config.enabled {
            warn!("⚠️ Resurrection disabled for {}", name);
            return Ok(());
        }

        // Get resurrection attempt count
        let attempts = match &primal.state {
            LifecycleState::Degraded {
                resurrection_attempts,
                ..
            } => *resurrection_attempts,
            LifecycleState::Dead { .. } => 0,
            _ => return Ok(()), // Not in a state that needs resurrection
        };

        // Check max attempts
        if attempts >= primal.resurrection_config.max_attempts {
            error!("💀 {} resurrection exhausted ({} attempts)", name, attempts);
            primal.state = LifecycleState::Apoptosis {
                reason: ApoptosisReason::ResurrectionExhausted,
                started_at: chrono::Utc::now(),
            };
            return Ok(());
        }

        // Calculate backoff delay
        let delay = std::cmp::min(
            primal.resurrection_config.base_delay * 2u32.pow(attempts),
            primal.resurrection_config.max_delay,
        );

        info!(
            "🔄 Resurrecting {} (attempt {}/{}, delay: {:?})",
            name,
            attempts + 1,
            primal.resurrection_config.max_attempts,
            delay
        );

        // Get deployment node for resurrection
        let deployment_node = primal.deployment_node.clone();
        let socket_path = primal.socket_path.clone();

        // Update state
        if let LifecycleState::Degraded {
            resurrection_attempts,
            ..
        } = &mut primal.state
        {
            *resurrection_attempts = attempts + 1;
        }

        primal.metrics.resurrection_count += 1;

        // Drop lock before async operations
        drop(primals);

        // Wait backoff delay
        tokio::time::sleep(delay).await;

        // Kill old process if still running
        self.kill_primal_process(name).await?;

        // Clean up old socket
        if socket_path.exists() {
            tokio::fs::remove_file(&socket_path).await.ok();
        }

        // Respawn from deployment node
        if let Some(node) = deployment_node {
            self.respawn_primal(name, &node).await?;
        } else {
            warn!("⚠️ No deployment node for {} - cannot resurrect", name);
        }

        Ok(())
    }

    /// Kill primal process
    async fn kill_primal_process(&self, name: &str) -> Result<()> {
        let primals = self.primals.read().await;
        if let Some(primal) = primals.get(name) {
            if let Some(pid) = primal.pid {
                info!("🔪 Killing {} (PID: {})", name, pid);

                // Send SIGTERM first
                #[cfg(unix)]
                {
                    use nix::sys::signal::{kill, Signal};
                    use nix::unistd::Pid;

                    let pid = Pid::from_raw(pid as i32);

                    // Try graceful SIGTERM
                    if kill(pid, Signal::SIGTERM).is_ok() {
                        // Wait up to 5 seconds for graceful shutdown
                        for _ in 0..50 {
                            tokio::time::sleep(Duration::from_millis(100)).await;
                            if kill(pid, None).is_err() {
                                return Ok(()); // Process dead
                            }
                        }

                        // Force SIGKILL if still running
                        warn!("⚠️ {} didn't terminate gracefully, sending SIGKILL", name);
                        kill(pid, Signal::SIGKILL).ok();
                    }
                }
            }
        }
        Ok(())
    }

    /// Respawn a primal from its deployment node
    async fn respawn_primal(&self, name: &str, node: &GraphNode) -> Result<()> {
        use crate::executor::context::ExecutionContext;
        use crate::executor::primal_spawner;

        info!("🌱 Respawning {} from deployment node", name);

        // Create execution context
        let mut env = HashMap::new();
        env.insert("FAMILY_ID".to_string(), self.family_id.clone());

        let context = ExecutionContext::new(env).with_nucleation(self.nucleation.clone());

        // Get operation mode
        let mode = node
            .operation
            .as_ref()
            .and_then(|op| op.params.get("mode"))
            .and_then(|v| v.as_str())
            .unwrap_or("server");

        // Spawn primal
        let child = primal_spawner::spawn_primal_process(name, mode, &context, node).await?;

        let pid = child.id();

        // Update primal state
        let mut primals = self.primals.write().await;
        if let Some(primal) = primals.get_mut(name) {
            primal.pid = pid;
            primal.state = LifecycleState::Incubating {
                started_at: chrono::Utc::now(),
                timeout_ms: 30000,
            };
            primal.metrics.health_failures = 0;
        }

        // Relay output streams
        primal_spawner::relay_output_streams(child, name.to_string());

        info!("✅ {} respawned (PID: {:?})", name, pid);

        Ok(())
    }

    // ========================================================================
    // APOPTOSIS (Graceful Shutdown)
    // ========================================================================

    /// Initiate apoptosis (graceful shutdown) for a primal
    ///
    /// Uses iterative approach instead of recursion to avoid async boxing complexity.
    pub async fn apoptosis(&self, name: &str, _reason: ApoptosisReason) -> Result<()> {
        // Collect all primals to shut down (transitive dependents)
        let shutdown_list = self.collect_shutdown_order(name).await;

        for (primal_name, shutdown_reason) in shutdown_list {
            self.apoptosis_single(&primal_name, shutdown_reason).await?;
        }

        Ok(())
    }

    /// Collect primals to shut down in order (dependents first)
    async fn collect_shutdown_order(&self, name: &str) -> Vec<(String, ApoptosisReason)> {
        let mut result = Vec::new();
        let mut to_process = vec![(name.to_string(), ApoptosisReason::UserRequest)];
        let mut processed = std::collections::HashSet::new();

        while let Some((current, reason)) = to_process.pop() {
            if processed.contains(&current) {
                continue;
            }
            processed.insert(current.clone());

            // Get dependents
            let primals = self.primals.read().await;
            if let Some(primal) = primals.get(&current) {
                // Add dependents first (they get shut down before this primal)
                for dependent in &primal.depended_by {
                    if !processed.contains(dependent) {
                        to_process.push((
                            dependent.clone(),
                            ApoptosisReason::DependencyDeath(current.clone()),
                        ));
                    }
                }
            }
            drop(primals);

            // Add this primal to result
            result.push((current, reason));
        }

        // Reverse so dependents are first
        result.reverse();
        result
    }

    /// Shutdown a single primal (no recursion)
    async fn apoptosis_single(&self, name: &str, reason: ApoptosisReason) -> Result<()> {
        info!("💀 Initiating apoptosis for {}: {:?}", name, reason);

        let mut primals = self.primals.write().await;
        let primal = match primals.get_mut(name) {
            Some(p) => p,
            None => {
                warn!("⚠️ Primal not found for apoptosis: {}", name);
                return Ok(());
            }
        };

        primal.state = LifecycleState::Apoptosis {
            reason: reason.clone(),
            started_at: chrono::Utc::now(),
        };

        let _pid = primal.pid;
        drop(primals);

        // Kill the process
        self.kill_primal_process(name).await?;

        // Update state to dead
        let mut primals = self.primals.write().await;
        if let Some(primal) = primals.get_mut(name) {
            primal.state = LifecycleState::Dead {
                since: chrono::Utc::now(),
                reason: format!("{:?}", reason),
            };
            primal.pid = None;
        }

        info!("💀 {} apoptosis complete", name);

        Ok(())
    }

    /// Initiate system-wide shutdown
    pub async fn shutdown_all(&self) -> Result<()> {
        info!("🛑 Initiating system-wide shutdown");

        // Set shutdown flag
        *self.shutdown.write().await = true;

        // Get all primals in dependency order (reverse topological)
        let primals = self.primals.read().await;
        let mut shutdown_order: Vec<String> = primals.keys().cloned().collect();

        // Sort by number of dependents (most dependents first = shutdown last)
        shutdown_order.sort_by_key(|name| {
            primals
                .get(name)
                .map(|p| std::cmp::Reverse(p.depended_by.len()))
                .unwrap_or(std::cmp::Reverse(0))
        });

        drop(primals);

        // Shutdown in order
        for name in shutdown_order {
            self.apoptosis(&name, ApoptosisReason::SystemShutdown)
                .await
                .ok();
        }

        info!("✅ System shutdown complete");

        Ok(())
    }

    // ========================================================================
    // HELPERS
    // ========================================================================

    /// Update dependency graph (depended_by relationships)
    async fn update_dependency_graph(&self) {
        let mut primals = self.primals.write().await;

        // Clear all depended_by lists
        for primal in primals.values_mut() {
            primal.depended_by.clear();
        }

        // Build dependency graph
        let depends_on_map: HashMap<String, Vec<String>> = primals
            .iter()
            .map(|(name, p)| (name.clone(), p.depends_on.clone()))
            .collect();

        // Set depended_by relationships
        for (name, deps) in &depends_on_map {
            for dep in deps {
                if let Some(dep_primal) = primals.get_mut(dep) {
                    dep_primal.depended_by.push(name.clone());
                }
            }
        }
    }

    /// Get status of all managed primals
    pub async fn get_status(&self) -> HashMap<String, LifecycleState> {
        let primals = self.primals.read().await;
        primals
            .iter()
            .map(|(name, p)| (name.clone(), p.state.clone()))
            .collect()
    }

    /// Get detailed info for a primal
    pub async fn get_primal_info(&self, name: &str) -> Option<ManagedPrimal> {
        let primals = self.primals.read().await;
        primals.get(name).cloned()
    }

    /// Clone for spawning async tasks
    fn clone_for_task(&self) -> Self {
        Self {
            primals: self.primals.clone(),
            family_id: self.family_id.clone(),
            nucleation: self.nucleation.clone(),
            health_checker: self.health_checker.clone(),
            deployment_graphs: self.deployment_graphs.clone(),
            health_check_interval: self.health_check_interval,
            shutdown: self.shutdown.clone(),
        }
    }
}

// Implement Clone for health_checker (it's just config)
impl Clone for HealthChecker {
    fn clone(&self) -> Self {
        Self::new(PathBuf::from("/tmp"))
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_lifecycle_state_transitions() {
        let manager = LifecycleManager::new("test-family");

        // Register a primal
        manager
            .register_primal(
                "test-primal",
                PathBuf::from("/tmp/test-primal.sock"),
                Some(12345),
                None,
            )
            .await
            .unwrap();

        // Check initial state
        let status = manager.get_status().await;
        assert!(matches!(
            status.get("test-primal"),
            Some(LifecycleState::Incubating { .. })
        ));
    }

    #[test]
    fn test_apoptosis_reason_serialization() {
        let reason = ApoptosisReason::DependencyDeath("beardog".to_string());
        let json = serde_json::to_string(&reason).unwrap();
        let deserialized: ApoptosisReason = serde_json::from_str(&json).unwrap();
        assert_eq!(reason, deserialized);
    }

    #[test]
    fn test_health_config_defaults() {
        let config = HealthConfig::default();
        assert_eq!(config.check_interval, Duration::from_secs(30));
        assert_eq!(config.failure_threshold, 3);
    }

    #[test]
    fn test_resurrection_config_defaults() {
        let config = ResurrectionConfig::default();
        assert!(config.enabled);
        assert_eq!(config.max_attempts, 5);
    }
}
