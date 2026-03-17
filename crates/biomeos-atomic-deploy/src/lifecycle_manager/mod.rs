// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

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

mod apoptosis;
mod germination;
mod helpers;
mod monitoring;
mod resurrection;
mod types;

pub use types::*;

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

use crate::health_check::HealthChecker;
use crate::neural_graph::Graph;
use crate::nucleation::SocketNucleation;

/// Primal Lifecycle Manager
///
/// Manages the lifecycle of all primals in a NUCLEUS deployment:
/// - Monitors health and detects crashes
/// - Resurrects dead primals from deployment graphs
/// - Coordinates graceful shutdown (apoptosis)
pub struct LifecycleManager {
    /// Managed primals (name -> ManagedPrimal)
    pub(crate) primals: Arc<RwLock<HashMap<String, ManagedPrimal>>>,

    /// Family ID
    pub(crate) family_id: String,

    /// Socket nucleation for deterministic paths
    pub(crate) nucleation: Arc<RwLock<SocketNucleation>>,

    /// Health checker
    pub(crate) health_checker: HealthChecker,

    /// Deployment graphs (for resurrection)
    pub(crate) deployment_graphs: Arc<RwLock<HashMap<String, Graph>>>,

    /// Global health check interval
    pub(crate) health_check_interval: Duration,

    /// Shutdown flag
    pub(crate) shutdown: Arc<RwLock<bool>>,
}

impl LifecycleManager {
    /// Create a new lifecycle manager
    pub fn new(family_id: impl Into<String>) -> Self {
        let family_id = family_id.into();

        Self {
            primals: Arc::new(RwLock::new(HashMap::new())),
            family_id,
            nucleation: Arc::new(RwLock::new(SocketNucleation::default())),
            health_checker: HealthChecker::new_default(),
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
            health_checker: HealthChecker::new_default(),
            deployment_graphs: Arc::new(RwLock::new(HashMap::new())),
            health_check_interval,
            shutdown: Arc::new(RwLock::new(false)),
        }
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::neural_graph::GraphConfig;
    use std::path::PathBuf;

    // ========================================================================
    // Config Defaults
    // ========================================================================

    #[test]
    fn test_health_config_defaults() {
        let config = HealthConfig::default();
        assert_eq!(config.check_interval, Duration::from_secs(30));
        assert_eq!(config.timeout, Duration::from_secs(5));
        assert_eq!(config.failure_threshold, 3);
        assert_eq!(config.health_method, "health");
    }

    #[test]
    fn test_resurrection_config_defaults() {
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
        assert_eq!(metrics.last_health_latency_ms, 0);
        assert_eq!(metrics.requests_served, 0);
    }

    // ========================================================================
    // Serialization
    // ========================================================================

    #[test]
    fn test_apoptosis_reason_serialization() {
        let reason = ApoptosisReason::DependencyDeath("beardog".to_string());
        let json = serde_json::to_string(&reason).expect("serialize apoptosis reason");
        let deserialized: ApoptosisReason =
            serde_json::from_str(&json).expect("deserialize apoptosis reason");
        assert_eq!(reason, deserialized);
    }

    #[test]
    fn test_all_apoptosis_reasons_serialize() {
        let reasons = vec![
            ApoptosisReason::UserRequest,
            ApoptosisReason::EcosystemHealth,
            ApoptosisReason::ResourcePressure,
            ApoptosisReason::DependencyDeath("songbird".to_string()),
            ApoptosisReason::ResurrectionExhausted,
            ApoptosisReason::SystemShutdown,
        ];
        for reason in reasons {
            let json = serde_json::to_string(&reason).expect("serialize reason");
            let parsed: ApoptosisReason = serde_json::from_str(&json).expect("parse reason");
            assert_eq!(reason, parsed);
        }
    }

    #[test]
    fn test_lifecycle_state_germinating_serialization() {
        let state = LifecycleState::Germinating;
        let json = serde_json::to_string(&state).expect("serialize");
        let parsed: LifecycleState = serde_json::from_str(&json).expect("parse");
        assert_eq!(state, parsed);
    }

    #[test]
    fn test_lifecycle_state_active_serialization() {
        let now = chrono::Utc::now();
        let state = LifecycleState::Active {
            since: now,
            last_health_check: now,
        };
        let json = serde_json::to_string(&state).expect("serialize");
        let parsed: LifecycleState = serde_json::from_str(&json).expect("parse");
        assert_eq!(state, parsed);
    }

    #[test]
    fn test_lifecycle_state_degraded_serialization() {
        let state = LifecycleState::Degraded {
            since: chrono::Utc::now(),
            reason: "health check failed".to_string(),
            resurrection_attempts: 2,
        };
        let json = serde_json::to_string(&state).expect("serialize");
        let parsed: LifecycleState = serde_json::from_str(&json).expect("parse");
        assert_eq!(state, parsed);
    }

    #[test]
    fn test_lifecycle_state_dead_serialization() {
        let state = LifecycleState::Dead {
            since: chrono::Utc::now(),
            reason: "SystemShutdown".to_string(),
        };
        let json = serde_json::to_string(&state).expect("serialize");
        let parsed: LifecycleState = serde_json::from_str(&json).expect("parse");
        assert_eq!(state, parsed);
    }

    #[test]
    fn test_health_config_serialization() {
        let config = HealthConfig::default();
        let json = serde_json::to_string(&config).expect("serialize health config");
        let parsed: HealthConfig = serde_json::from_str(&json).expect("parse health config");
        assert_eq!(parsed.failure_threshold, config.failure_threshold);
        assert_eq!(parsed.health_method, config.health_method);
    }

    #[test]
    fn test_resurrection_config_serialization() {
        let config = ResurrectionConfig {
            enabled: false,
            max_attempts: 10,
            base_delay: Duration::from_secs(5),
            max_delay: Duration::from_secs(120),
        };
        let json = serde_json::to_string(&config).expect("serialize");
        let parsed: ResurrectionConfig = serde_json::from_str(&json).expect("parse");
        assert!(!parsed.enabled);
        assert_eq!(parsed.max_attempts, 10);
    }

    #[test]
    fn test_primal_metrics_serialization() {
        let metrics = PrimalMetrics {
            total_uptime_secs: 3600,
            resurrection_count: 2,
            health_failures: 5,
            last_health_latency_ms: 12,
            requests_served: 1000,
        };
        let json = serde_json::to_string(&metrics).expect("serialize");
        let parsed: PrimalMetrics = serde_json::from_str(&json).expect("parse");
        assert_eq!(parsed.total_uptime_secs, 3600);
        assert_eq!(parsed.resurrection_count, 2);
        assert_eq!(parsed.requests_served, 1000);
    }

    // ========================================================================
    // Lifecycle Manager Core
    // ========================================================================

    #[tokio::test]
    async fn test_lifecycle_manager_creation() {
        let manager = LifecycleManager::new("test-family");
        let status = manager.get_status().await;
        assert!(status.is_empty());
    }

    #[tokio::test]
    async fn test_lifecycle_manager_with_config() {
        let nucleation = Arc::new(RwLock::new(SocketNucleation::default()));
        let manager =
            LifecycleManager::with_config("custom-family", Duration::from_secs(5), nucleation);
        let status = manager.get_status().await;
        assert!(status.is_empty());
    }

    #[tokio::test]
    async fn test_register_primal_incubating() {
        let manager = LifecycleManager::new("test-family");

        manager
            .register_primal(
                "test-primal",
                PathBuf::from("/tmp/test-primal.sock"),
                Some(12345),
                None,
            )
            .await
            .expect("register primal");

        let status = manager.get_status().await;
        assert!(matches!(
            status.get("test-primal"),
            Some(LifecycleState::Incubating { .. })
        ));
    }

    #[tokio::test]
    async fn test_register_multiple_primals() {
        let manager = LifecycleManager::new("test-family");

        for name in &["beardog", "songbird", "nestgate"] {
            manager
                .register_primal(
                    *name,
                    PathBuf::from(format!("/tmp/{name}.sock")),
                    Some(100),
                    None,
                )
                .await
                .expect("register primal");
        }

        let status = manager.get_status().await;
        assert_eq!(status.len(), 3);
        assert!(status.contains_key("beardog"));
        assert!(status.contains_key("songbird"));
        assert!(status.contains_key("nestgate"));
    }

    #[tokio::test]
    async fn test_get_primal_info() {
        let manager = LifecycleManager::new("test-family");

        manager
            .register_primal(
                "beardog",
                PathBuf::from("/tmp/beardog.sock"),
                Some(42),
                None,
            )
            .await
            .expect("register");

        let info = manager.get_primal_info("beardog").await;
        assert!(info.is_some());
        let info = info.expect("primal info");
        assert_eq!(info.name, "beardog");
        assert_eq!(info.family_id, "test-family");
        assert_eq!(info.pid, Some(42));
        assert_eq!(info.socket_path, PathBuf::from("/tmp/beardog.sock"));
    }

    #[tokio::test]
    async fn test_get_primal_info_not_found() {
        let manager = LifecycleManager::new("test-family");
        let info = manager.get_primal_info("nonexistent").await;
        assert!(info.is_none());
    }

    /// Helper: create a minimal GraphNode for tests
    fn test_graph_node(id: &str, depends_on: Vec<String>) -> crate::neural_graph::GraphNode {
        crate::neural_graph::GraphNode {
            id: id.to_string(),
            depends_on,
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn test_register_with_dependencies() {
        let manager = LifecycleManager::new("test-family");

        // Register beardog first (no deps)
        manager
            .register_primal(
                "beardog",
                PathBuf::from("/tmp/beardog.sock"),
                Some(100),
                Some(test_graph_node("beardog", vec![])),
            )
            .await
            .expect("register beardog");

        // Register songbird depending on beardog
        manager
            .register_primal(
                "songbird",
                PathBuf::from("/tmp/songbird.sock"),
                Some(101),
                Some(test_graph_node("songbird", vec!["beardog".to_string()])),
            )
            .await
            .expect("register songbird");

        // Verify dependency tracking
        let beardog = manager.get_primal_info("beardog").await.expect("beardog");
        assert!(beardog.depended_by.contains(&"songbird".to_string()));

        let songbird = manager.get_primal_info("songbird").await.expect("songbird");
        assert!(songbird.depends_on.contains(&"beardog".to_string()));
    }

    #[tokio::test]
    async fn test_store_deployment_graph() {
        let manager = LifecycleManager::new("test-family");

        let graph = crate::neural_graph::Graph {
            id: "tower".to_string(),
            version: "1.0.0".to_string(),
            description: "Test graph".to_string(),
            nodes: vec![test_graph_node("beardog", vec![])],
            config: GraphConfig::default(),
            coordination: None,
        };

        manager.store_deployment_graph("tower-graph", graph).await;

        // Verify it was stored (no panic)
    }

    // ========================================================================
    // Apoptosis
    // ========================================================================

    #[tokio::test]
    async fn test_apoptosis_nonexistent_primal() {
        let manager = LifecycleManager::new("test-family");
        // Should not error on nonexistent primal
        let result = manager
            .apoptosis("nonexistent", ApoptosisReason::UserRequest)
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_apoptosis_updates_state() {
        let manager = LifecycleManager::new("test-family");

        manager
            .register_primal(
                "test-primal",
                PathBuf::from("/tmp/test.sock"),
                None, // No real PID
                None,
            )
            .await
            .expect("register");

        manager
            .apoptosis("test-primal", ApoptosisReason::UserRequest)
            .await
            .expect("apoptosis");

        let status = manager.get_status().await;
        assert!(matches!(
            status.get("test-primal"),
            Some(LifecycleState::Dead { .. })
        ));
    }

    #[tokio::test]
    async fn test_shutdown_all() {
        let manager = LifecycleManager::new("test-family");

        for name in &["beardog", "songbird"] {
            manager
                .register_primal(
                    *name,
                    PathBuf::from(format!("/tmp/{name}.sock")),
                    None,
                    None,
                )
                .await
                .expect("register");
        }

        manager.shutdown_all().await.expect("shutdown");

        let status = manager.get_status().await;
        for state in status.values() {
            assert!(
                matches!(state, LifecycleState::Dead { .. }),
                "Expected Dead state, got: {state:?}"
            );
        }
    }

    #[tokio::test]
    async fn test_collect_shutdown_order() {
        let manager = LifecycleManager::new("test-family");

        // beardog has no deps
        manager
            .register_primal(
                "beardog",
                PathBuf::from("/tmp/beardog.sock"),
                None,
                Some(test_graph_node("beardog", vec![])),
            )
            .await
            .expect("register beardog");

        // songbird depends on beardog
        manager
            .register_primal(
                "songbird",
                PathBuf::from("/tmp/songbird.sock"),
                None,
                Some(test_graph_node("songbird", vec!["beardog".to_string()])),
            )
            .await
            .expect("register songbird");

        let order = manager.collect_shutdown_order("beardog").await;

        // Songbird (dependent) should be shut down before beardog
        assert_eq!(order.len(), 2);
        let names: Vec<&str> = order.iter().map(|(n, _)| n.as_str()).collect();
        let songbird_pos = names
            .iter()
            .position(|&n| n == "songbird")
            .expect("songbird");
        let beardog_pos = names.iter().position(|&n| n == "beardog").expect("beardog");
        assert!(
            songbird_pos < beardog_pos,
            "Songbird should shut down before beardog"
        );
    }

    // ========================================================================
    // Clone for task
    // ========================================================================

    #[tokio::test]
    async fn test_clone_for_task() {
        let manager = LifecycleManager::new("test-family");
        manager
            .register_primal("test", PathBuf::from("/tmp/test.sock"), None, None)
            .await
            .expect("register");

        let cloned = manager.clone_for_task();
        let status = cloned.get_status().await;
        assert_eq!(status.len(), 1);
        assert!(status.contains_key("test"));
    }

    // ========================================================================
    // ManagedPrimal
    // ========================================================================

    #[test]
    fn test_managed_primal_serialization() {
        let primal = ManagedPrimal {
            name: "beardog".to_string(),
            family_id: "test-family".to_string(),
            socket_path: PathBuf::from("/tmp/beardog.sock"),
            pid: Some(1234),
            state: LifecycleState::Germinating,
            deployment_node: None,
            depends_on: vec!["base".to_string()],
            depended_by: vec!["songbird".to_string()],
            health_config: HealthConfig::default(),
            resurrection_config: ResurrectionConfig::default(),
            metrics: PrimalMetrics::default(),
        };

        let json = serde_json::to_string(&primal).expect("serialize managed primal");
        let parsed: ManagedPrimal = serde_json::from_str(&json).expect("parse managed primal");
        assert_eq!(parsed.name, "beardog");
        assert_eq!(parsed.family_id, "test-family");
        assert_eq!(parsed.pid, Some(1234));
        assert_eq!(parsed.depends_on, vec!["base"]);
        assert_eq!(parsed.depended_by, vec!["songbird"]);
    }
}
