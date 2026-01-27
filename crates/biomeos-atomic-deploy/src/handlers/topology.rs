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

use crate::neural_router::NeuralRouter;
use anyhow::Result;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

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

    /// Discover active primals by scanning socket directory.
    ///
    /// This is the capability-based approach - we don't hardcode primal names,
    /// we discover what's running.
    async fn discover_active_primals(&self) -> Result<Vec<Value>> {
        let mut primals = Vec::new();
        let socket_dir = Path::new("/tmp");

        // Pattern: {primal}-{family_id}*.sock
        let family_pattern = format!("-{}", self.family_id);

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

        // Check capabilities, not primal names
        let has_security = primals.iter().any(|p| {
            p["capabilities"]
                .as_array()
                .map(|caps| caps.iter().any(|c| c.as_str() == Some("security")))
                .unwrap_or(false)
        }) || primals.iter().any(|p| p["primal_type"] == "beardog"); // Fallback

        let has_discovery = primals.iter().any(|p| {
            p["capabilities"]
                .as_array()
                .map(|caps| caps.iter().any(|c| c.as_str() == Some("discovery")))
                .unwrap_or(false)
        }) || primals.iter().any(|p| p["primal_type"] == "songbird");

        let has_compute = primals.iter().any(|p| {
            p["capabilities"]
                .as_array()
                .map(|caps| caps.iter().any(|c| c.as_str() == Some("compute")))
                .unwrap_or(false)
        }) || primals.iter().any(|p| p["primal_type"] == "toadstool");

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

    #[tokio::test]
    async fn test_topology_handler_creation() {
        let router = Arc::new(NeuralRouter::new("test-family"));
        let executions = Arc::new(RwLock::new(HashMap::new()));

        let handler = TopologyHandler::new("test-family", router, executions, "/tmp");

        // Should work even with no primals
        let result = handler.get_proprioception().await.unwrap();
        assert_eq!(result["family_id"], "test-family");
    }
}
