// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Topology and system metrics handlers.
//!
//! This module handles topology discovery and system metrics:
//! - `topology.get` - Get system topology
//! - `topology.primals` - List active primals
//! - `topology.proprioception` - SAME DAVE self-awareness
//! - `topology.metrics` - System metrics
//!
//! # Capability-Based Discovery
//!
//! Instead of hardcoding socket patterns for specific primals,
//! we dynamically discover active sockets and query capabilities.
//!
//! # XDG Compliance (EVOLVED Jan 27, 2026)
//!
//! Socket discovery uses SystemPaths for XDG-compliant path resolution:
//! 1. `$XDG_RUNTIME_DIR/biomeos/` (preferred)
//! 2. `/tmp/biomeos-$USER/` (fallback)
//! 3. `/tmp/` (legacy compatibility)

use crate::neural_router::NeuralRouter;
use anyhow::Result;
use biomeos_types::SystemPaths;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

/// Topology handler for system discovery and metrics.
#[derive(Clone)]
pub struct TopologyHandler {
    /// Family ID
    family_id: String,

    /// Neural Router for capability queries
    router: Arc<NeuralRouter>,

    /// Active executions count (for metrics)
    executions: Arc<RwLock<HashMap<String, super::graph::ExecutionStatus>>>,

    /// Graphs directory (for metrics)
    graphs_dir: std::path::PathBuf,
}

impl TopologyHandler {
    /// Create a new topology handler.
    pub fn new(
        family_id: impl Into<String>,
        router: Arc<NeuralRouter>,
        executions: Arc<RwLock<HashMap<String, super::graph::ExecutionStatus>>>,
        graphs_dir: impl Into<std::path::PathBuf>,
    ) -> Self {
        Self {
            family_id: family_id.into(),
            router,
            executions,
            graphs_dir: graphs_dir.into(),
        }
    }

    /// Get system topology by dynamically discovering active sockets.
    ///
    /// JSON-RPC method: `topology.get`
    ///
    /// # Dynamic Discovery
    ///
    /// Instead of hardcoding primal names, we scan for active Unix sockets
    /// matching the family pattern and query their capabilities.
    pub async fn get(&self) -> Result<Value> {
        let primals = self.discover_active_primals().await?;

        // Infer connections from capabilities
        let connections = self.infer_connections(&primals);

        Ok(json!({
            "primals": primals,
            "connections": connections,
            "timestamp": chrono::Utc::now().to_rfc3339()
        }))
    }

    /// Discover active primals by scanning socket directories.
    ///
    /// This is the capability-based approach - we don't hardcode primal names,
    /// we discover what's running.
    ///
    /// EVOLVED (Jan 27, 2026): Uses XDG-compliant socket discovery via SystemPaths
    async fn discover_active_primals(&self) -> Result<Vec<Value>> {
        let mut primals = Vec::new();

        // Get XDG-compliant socket directories (no hardcoding!)
        let socket_dirs = Self::get_socket_directories();
        debug!("Scanning socket directories: {:?}", socket_dirs);

        // Pattern: {primal}-{family_id}*.sock
        let family_pattern = format!("-{}", self.family_id);

        for socket_dir in &socket_dirs {
            if let Ok(entries) = std::fs::read_dir(socket_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

                    // Check if it's a socket for our family
                    if filename.ends_with(".sock") && filename.contains(&family_pattern) {
                        // Extract primal name from socket filename
                        // Pattern: {primal}-{family_id}[-node_id].sock
                        if let Some(primal_name) = filename.split('-').next() {
                            let socket_path = path.to_string_lossy().to_string();

                            // Query capabilities if possible
                            let capabilities = self
                                .query_primal_capabilities(&socket_path)
                                .await
                                .unwrap_or_default();

                            primals.push(json!({
                                "id": filename.trim_end_matches(".sock"),
                                "primal_type": primal_name,
                                "socket_path": socket_path,
                                "health": "healthy", // Could ping socket for real health
                                "capabilities": capabilities,
                                "resource_usage": null
                            }));
                        }
                    }
                }
            }
        }

        // Also check registered capabilities in router
        let registered = self.router.list_capabilities().await;
        for (cap, providers) in registered {
            for provider in providers {
                // Avoid duplicates
                if !primals
                    .iter()
                    .any(|p| p["socket_path"] == provider.socket_path.display().to_string())
                {
                    primals.push(json!({
                        "id": format!("{}-{}", provider.primal_name, self.family_id),
                        "primal_type": provider.primal_name,
                        "socket_path": provider.socket_path.display().to_string(),
                        "health": "healthy", // Socket exists = healthy for now
                        "capabilities": [cap],
                        "resource_usage": null
                    }));
                }
            }
        }

        Ok(primals)
    }

    /// Query a primal for its capabilities via JSON-RPC.
    async fn query_primal_capabilities(&self, _socket_path: &str) -> Result<Vec<String>> {
        // Future: Actually call health.capabilities on the socket
        // For now, return empty - we'll use registered capabilities
        Ok(vec![])
    }

    /// Get XDG-compliant socket directories for primal discovery
    ///
    /// EVOLVED (Jan 27, 2026): No more hardcoded `/tmp` - uses SystemPaths
    ///
    /// # Priority Order
    /// 1. XDG runtime directory: `$XDG_RUNTIME_DIR/biomeos/`
    /// 2. User runtime fallback: `/tmp/biomeos-$USER/`
    /// 3. Legacy compatibility: `/tmp/` (for existing deployments)
    fn get_socket_directories() -> Vec<PathBuf> {
        let mut dirs = Vec::new();

        // Priority 1: SystemPaths XDG-compliant runtime directory
        if let Ok(paths) = SystemPaths::new() {
            dirs.push(paths.runtime_dir().to_path_buf());
        }

        // Priority 2: Explicit BIOMEOS_SOCKET_DIR environment variable
        if let Ok(socket_dir) = std::env::var("BIOMEOS_SOCKET_DIR") {
            let path = PathBuf::from(&socket_dir);
            if !dirs.contains(&path) && path.exists() {
                dirs.push(path);
            }
        }

        // Priority 3: XDG_RUNTIME_DIR/biomeos (direct check)
        if let Ok(xdg_runtime) = std::env::var("XDG_RUNTIME_DIR") {
            let path = PathBuf::from(xdg_runtime).join("biomeos");
            if !dirs.contains(&path) && path.exists() {
                dirs.push(path);
            }
        }

        // Priority 4: /tmp/biomeos-$USER (user-namespaced)
        if let Ok(user) = std::env::var("USER") {
            let path = PathBuf::from(format!("/tmp/biomeos-{}", user));
            if !dirs.contains(&path) && path.exists() {
                dirs.push(path);
            }
        }

        // Priority 5: Legacy compatibility - /tmp (only if nothing else exists)
        // This ensures backwards compatibility during migration
        if dirs.is_empty() {
            dirs.push(PathBuf::from("/tmp"));
        }

        dirs
    }

    /// Infer connections from capability relationships.
    fn infer_connections(&self, primals: &[Value]) -> Vec<Value> {
        let mut connections = Vec::new();

        // Security provider relationship: anything with security capability → security provider
        let security_providers: Vec<_> = primals
            .iter()
            .filter(|p| {
                p["capabilities"]
                    .as_array()
                    .map(|caps| caps.iter().any(|c| c.as_str() == Some("security")))
                    .unwrap_or(false)
            })
            .collect();

        let discovery_providers: Vec<_> = primals
            .iter()
            .filter(|p| {
                p["capabilities"]
                    .as_array()
                    .map(|caps| caps.iter().any(|c| c.as_str() == Some("discovery")))
                    .unwrap_or(false)
            })
            .collect();

        // Connect discovery → security (standard pattern)
        for disco in &discovery_providers {
            for sec in &security_providers {
                if disco["id"] != sec["id"] {
                    connections.push(json!({
                        "from": disco["id"],
                        "to": sec["id"],
                        "connection_type": "security-provider",
                        "latency_ms": null
                    }));
                }
            }
        }

        connections
    }

    /// Get active primals.
    ///
    /// JSON-RPC method: `topology.primals`
    pub async fn get_primals(&self) -> Result<Value> {
        info!("📊 Neural API: get_primals called");

        let primals = self.discover_active_primals().await?;

        Ok(json!({
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "family_id": self.family_id,
            "primals": primals,
            "count": primals.len()
        }))
    }

    /// Get proprioception - SAME DAVE self-awareness.
    ///
    /// JSON-RPC method: `topology.proprioception`
    ///
    /// # Capability-Based Health
    ///
    /// Instead of checking for specific primals like "beardog",
    /// we check for CAPABILITIES like "security", "discovery", "compute".
    pub async fn get_proprioception(&self) -> Result<Value> {
        info!("📊 Neural API: get_proprioception called");

        let primals = self.discover_active_primals().await?;
        let primal_count = primals.len();

        // Check capabilities via taxonomy (not hardcoded primal names)
        let has_capability = |primals: &[serde_json::Value], cap: &str| -> bool {
            primals.iter().any(|p| {
                // Primary: check declared capabilities
                p["capabilities"]
                    .as_array()
                    .map(|caps| caps.iter().any(|c| c.as_str() == Some(cap)))
                    .unwrap_or(false)
            }) || primals.iter().any(|p| {
                // Fallback: resolve primal_type through taxonomy
                p["primal_type"]
                    .as_str()
                    .map(|name| {
                        biomeos_types::capability_taxonomy::capabilities_for_primal(name)
                            .iter()
                            .any(|c| c == cap)
                    })
                    .unwrap_or(false)
            })
        };

        let has_security = has_capability(&primals, "security");
        let has_discovery = has_capability(&primals, "discovery");
        let has_compute = has_capability(&primals, "compute");

        let expected_capabilities = 3;
        let actual_capabilities = [has_security, has_discovery, has_compute]
            .iter()
            .filter(|&&x| x)
            .count();

        let health_percentage =
            ((actual_capabilities as f64 / expected_capabilities as f64) * 100.0).min(100.0);

        let confidence = if has_security && has_discovery && has_compute {
            100.0
        } else {
            health_percentage * 0.5
        };

        Ok(json!({
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "family_id": self.family_id,
            "health": {
                "percentage": health_percentage,
                "status": if health_percentage >= 80.0 { "healthy" }
                          else if health_percentage >= 50.0 { "degraded" }
                          else { "critical" }
            },
            "confidence": confidence,
            "self_awareness": {
                "knows_about": primal_count,
                "can_coordinate": primal_count > 0,
                "has_security": has_security,
                "has_discovery": has_discovery,
                "has_compute": has_compute
            },
            "motor": {
                "can_deploy": true,
                "can_execute_graphs": true,
                "can_coordinate_primals": primal_count >= 2
            },
            "sensory": {
                "active_sockets": primal_count,
                "last_scan": chrono::Utc::now().to_rfc3339()
            }
        }))
    }

    /// Get aggregated metrics.
    ///
    /// JSON-RPC method: `topology.metrics`
    pub async fn get_metrics(&self) -> Result<Value> {
        info!("📊 Neural API: get_metrics called");

        let mut sys = sysinfo::System::new_all();
        sys.refresh_all();

        let primals = self.discover_active_primals().await?;
        let primal_count = primals.len();

        let graphs_count = std::fs::read_dir(&self.graphs_dir)
            .map(|entries| {
                entries
                    .filter_map(|e| e.ok())
                    .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("toml"))
                    .count()
            })
            .unwrap_or(0);

        Ok(json!({
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "system": {
                "cpu_percent": sys.global_cpu_usage(),
                "memory_used_mb": sys.used_memory() / 1024 / 1024,
                "memory_total_mb": sys.total_memory() / 1024 / 1024,
                "memory_percent": (sys.used_memory() as f64 / sys.total_memory() as f64) * 100.0,
                "uptime_seconds": sysinfo::System::uptime()
            },
            "neural_api": {
                "family_id": self.family_id,
                "active_primals": primal_count,
                "graphs_available": graphs_count,
                "active_executions": self.executions.read().await.len()
            }
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::handlers::graph::ExecutionStatus;
    use std::path::PathBuf;

    fn make_handler(
        family_id: &str,
        router: Arc<NeuralRouter>,
        graphs_dir: impl Into<PathBuf>,
    ) -> TopologyHandler {
        let executions = Arc::new(RwLock::new(HashMap::new()));
        TopologyHandler::new(family_id, router, executions, graphs_dir)
    }

    #[tokio::test]
    async fn test_topology_handler_creation() {
        let router = Arc::new(NeuralRouter::new("test-family"));
        let handler = make_handler("test-family", router, "/tmp");

        let result = handler
            .get_proprioception()
            .await
            .expect("get_proprioception");
        assert_eq!(result["family_id"], "test-family");
    }

    // =========================================================================
    // Topology node/edge types and serialization
    // =========================================================================

    #[tokio::test]
    async fn test_topology_get_response_structure() {
        let router = Arc::new(NeuralRouter::new("test-family"));
        let temp = tempfile::tempdir().expect("temp dir");
        let handler = make_handler("test-family", router, temp.path().to_path_buf());

        let result = handler.get().await.expect("topology.get");

        assert!(
            result.get("primals").is_some(),
            "Response must have primals"
        );
        assert!(
            result.get("connections").is_some(),
            "Response must have connections"
        );
        assert!(
            result.get("timestamp").is_some(),
            "Response must have timestamp"
        );

        let primals = result["primals"].as_array().expect("primals is array");
        let connections = result["connections"]
            .as_array()
            .expect("connections is array");

        for p in primals {
            assert!(p.get("id").is_some(), "Primal must have id");
            assert!(
                p.get("primal_type").is_some(),
                "Primal must have primal_type"
            );
            assert!(
                p.get("socket_path").is_some(),
                "Primal must have socket_path"
            );
            assert!(p.get("health").is_some(), "Primal must have health");
            assert!(
                p.get("capabilities").is_some(),
                "Primal must have capabilities"
            );
        }

        for c in connections {
            assert!(c.get("from").is_some(), "Connection must have from");
            assert!(c.get("to").is_some(), "Connection must have to");
            assert_eq!(
                c["connection_type"].as_str(),
                Some("security-provider"),
                "Connection type"
            );
        }
    }

    #[tokio::test]
    async fn test_topology_get_with_registered_capabilities() {
        let router = Arc::new(NeuralRouter::new("test-family"));
        router
            .register_capability(
                "security",
                "beardog",
                "/tmp/beardog-test-family.sock",
                "test",
            )
            .await
            .expect("register security");
        router
            .register_capability(
                "discovery",
                "songbird",
                "/tmp/songbird-test-family.sock",
                "test",
            )
            .await
            .expect("register discovery");

        let temp = tempfile::tempdir().expect("temp dir");
        let handler = make_handler("test-family", router, temp.path().to_path_buf());

        let result = handler.get().await.expect("topology.get");
        let primals = result["primals"].as_array().expect("primals");
        let connections = result["connections"].as_array().expect("connections");

        assert!(
            primals.len() >= 2,
            "Should discover beardog and songbird from registry, got {}",
            primals.len()
        );

        let primal_ids: Vec<&str> = primals
            .iter()
            .map(|p| p["id"].as_str().unwrap_or(""))
            .collect();
        assert!(
            primal_ids.contains(&"beardog-test-family"),
            "Should have beardog, got {:?}",
            primal_ids
        );
        assert!(
            primal_ids.contains(&"songbird-test-family"),
            "Should have songbird, got {:?}",
            primal_ids
        );

        if !connections.is_empty() {
            let conn = &connections[0];
            assert_eq!(conn["connection_type"].as_str(), Some("security-provider"));
            assert!(conn["from"].as_str().is_some());
            assert!(conn["to"].as_str().is_some());
        }
    }

    // =========================================================================
    // Topology construction and query logic
    // =========================================================================

    #[tokio::test]
    async fn test_get_primals_response_format() {
        let router = Arc::new(NeuralRouter::new("test-family"));
        let temp = tempfile::tempdir().expect("temp dir");
        let handler = make_handler("test-family", router, temp.path().to_path_buf());

        let result = handler.get_primals().await.expect("get_primals");

        assert_eq!(result["family_id"], "test-family");
        assert!(result.get("timestamp").is_some());
        assert!(result.get("primals").is_some());
        assert!(result.get("count").is_some());

        let count = result["count"].as_u64().expect("count is number");
        let primals = result["primals"].as_array().expect("primals is array");
        assert_eq!(count as usize, primals.len());
    }

    #[tokio::test]
    async fn test_get_proprioception_health_levels() {
        let router = Arc::new(NeuralRouter::new("test-family"));
        let temp = tempfile::tempdir().expect("temp dir");
        let handler = make_handler("test-family", router, temp.path().to_path_buf());

        let result = handler
            .get_proprioception()
            .await
            .expect("get_proprioception");

        assert!(result.get("health").is_some());
        let health = &result["health"];
        assert!(health.get("percentage").is_some());
        assert!(health.get("status").is_some());

        let status = health["status"].as_str().expect("status is string");
        assert!(
            ["healthy", "degraded", "critical"].contains(&status),
            "status must be healthy/degraded/critical, got {}",
            status
        );

        assert!(result.get("self_awareness").is_some());
        assert!(result.get("motor").is_some());
        assert!(result.get("sensory").is_some());
    }

    #[tokio::test]
    async fn test_get_proprioception_with_full_capabilities() {
        let router = Arc::new(NeuralRouter::new("test-family"));
        router
            .register_capability("security", "beardog", "/tmp/beardog-test.sock", "test")
            .await
            .expect("register");
        router
            .register_capability("discovery", "songbird", "/tmp/songbird-test.sock", "test")
            .await
            .expect("register");
        router
            .register_capability("compute", "toadstool", "/tmp/toadstool-test.sock", "test")
            .await
            .expect("register");

        let temp = tempfile::tempdir().expect("temp dir");
        let handler = make_handler("test-family", router, temp.path().to_path_buf());

        let result = handler
            .get_proprioception()
            .await
            .expect("get_proprioception");

        let sa = &result["self_awareness"];
        assert_eq!(sa["has_security"], true);
        assert_eq!(sa["has_discovery"], true);
        assert_eq!(sa["has_compute"], true);

        assert_eq!(result["health"]["percentage"], 100.0);
        assert_eq!(result["health"]["status"], "healthy");
        assert_eq!(result["confidence"], 100.0);
    }

    #[tokio::test]
    async fn test_get_proprioception_capability_via_primal_type() {
        let router = Arc::new(NeuralRouter::new("test-family"));
        router
            .register_capability("other", "beardog", "/tmp/beardog-test.sock", "test")
            .await
            .expect("register");

        let temp = tempfile::tempdir().expect("temp dir");
        let handler = make_handler("test-family", router, temp.path().to_path_buf());

        let result = handler
            .get_proprioception()
            .await
            .expect("get_proprioception");

        let sa = &result["self_awareness"];
        assert_eq!(
            sa["has_security"], true,
            "beardog provides security via taxonomy"
        );
    }

    #[tokio::test]
    async fn test_get_metrics_response_structure() {
        let router = Arc::new(NeuralRouter::new("test-family"));
        let temp = tempfile::tempdir().expect("temp dir");
        let handler = make_handler("test-family", router, temp.path().to_path_buf());

        let result = handler.get_metrics().await.expect("get_metrics");

        assert!(result.get("timestamp").is_some());
        assert!(result.get("system").is_some());
        assert!(result.get("neural_api").is_some());

        let system = &result["system"];
        assert!(system.get("cpu_percent").is_some());
        assert!(system.get("memory_used_mb").is_some());
        assert!(system.get("memory_total_mb").is_some());
        assert!(system.get("memory_percent").is_some());
        assert!(system.get("uptime_seconds").is_some());

        let neural = &result["neural_api"];
        assert_eq!(neural["family_id"], "test-family");
        assert!(neural.get("active_primals").is_some());
        assert!(neural.get("graphs_available").is_some());
        assert!(neural.get("active_executions").is_some());
    }

    #[tokio::test]
    async fn test_get_metrics_with_graphs_dir() {
        let router = Arc::new(NeuralRouter::new("test-family"));
        let temp = tempfile::tempdir().expect("temp dir");
        let graph_file = temp.path().join("test.toml");
        std::fs::write(
            &graph_file,
            r#"
[graph]
id = "test"
version = "1.0"
description = "Test"

[[nodes]]
id = "node1"
"#,
        )
        .expect("write graph");
        let handler = make_handler("test-family", router, temp.path().to_path_buf());

        let result = handler.get_metrics().await.expect("get_metrics");
        assert_eq!(result["neural_api"]["graphs_available"], 1);
    }

    #[tokio::test]
    async fn test_get_metrics_with_nonexistent_graphs_dir() {
        let router = Arc::new(NeuralRouter::new("test-family"));
        let handler = make_handler("test-family", router, "/nonexistent/path/12345");

        let result = handler.get_metrics().await.expect("get_metrics");
        assert_eq!(result["neural_api"]["graphs_available"], 0);
    }

    #[tokio::test]
    async fn test_get_metrics_active_executions_count() {
        let router = Arc::new(NeuralRouter::new("test-family"));
        let temp = tempfile::tempdir().expect("temp dir");
        let executions = Arc::new(RwLock::new(HashMap::from([(
            "exec-1".to_string(),
            ExecutionStatus {
                execution_id: "exec-1".to_string(),
                state: "running".to_string(),
                current_phase: Some(1),
                total_phases: 2,
                completed_nodes: vec![],
                failed_nodes: vec![],
                duration_ms: 100,
                error: None,
            },
        )])));
        let handler = TopologyHandler::new("test-family", router, executions, temp.path());

        let result = handler.get_metrics().await.expect("get_metrics");
        assert_eq!(result["neural_api"]["active_executions"], 1);
    }
}
