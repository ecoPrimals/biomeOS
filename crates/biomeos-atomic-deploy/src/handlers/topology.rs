// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

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
//! Socket discovery uses `SystemPaths` for XDG-compliant path resolution:
//! 1. `$XDG_RUNTIME_DIR/biomeos/` (preferred)
//! 2. `/tmp/biomeos-$USER/` (fallback)
//! 3. `/tmp/` (legacy compatibility)

use crate::neural_router::NeuralRouter;
use anyhow::Result;
use biomeos_types::SystemPaths;
use biomeos_types::defaults::DEFAULT_FAMILY_ID;
use serde_json::{Value, json};
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
    /// EVOLVED (Jan 27, 2026): Uses XDG-compliant socket discovery via `SystemPaths`
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

                            let endpoint =
                                biomeos_core::TransportEndpoint::UnixSocket { path: path.clone() };
                            let healthy =
                                crate::neural_router::NeuralRouter::check_endpoint_health(
                                    &endpoint,
                                )
                                .await;

                            primals.push(json!({
                                "id": filename.trim_end_matches(".sock"),
                                "primal_type": primal_name,
                                "socket_path": socket_path,
                                "health": if healthy { "healthy" } else { "unreachable" },
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
                    .any(|p| p["socket_path"] == provider.endpoint.display_string())
                {
                    primals.push(json!({
                        "id": format!("{}-{}", provider.primal_name, self.family_id),
                        "primal_type": provider.primal_name,
                        "socket_path": provider.endpoint.display_string(),
                        "health": "healthy",
                        "capabilities": [cap],
                        "resource_usage": null
                    }));
                }
            }
        }

        Ok(primals)
    }

    /// Query a primal for its capabilities via JSON-RPC `capabilities.list`.
    ///
    /// Connects to the primal's Unix socket, sends a `capabilities.list`
    /// request, and parses the response into a list of capability names.
    /// Falls back to an empty list on connection/parse errors so that
    /// topology discovery remains resilient.
    async fn query_primal_capabilities(&self, socket_path: &str) -> Result<Vec<String>> {
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
        use tokio::net::UnixStream;

        // Try both method names — some primals implement "capabilities.list",
        // others implement "capability.list". Both are valid per the route table.
        for method_name in ["capabilities.list", "capability.list"] {
            let stream = match tokio::time::timeout(
                std::time::Duration::from_millis(500),
                UnixStream::connect(socket_path),
            )
            .await
            {
                Ok(Ok(s)) => s,
                _ => return Ok(vec![]),
            };

            let request = serde_json::json!({
                "jsonrpc": "2.0",
                "method": method_name,
                "id": 1
            });
            let request_line = serde_json::to_string(&request)? + "\n";

            let mut reader = BufReader::new(stream);
            let stream_mut = reader.get_mut();
            if stream_mut.write_all(request_line.as_bytes()).await.is_err() {
                continue;
            }
            let _ = stream_mut.flush().await;

            let mut response_line = String::new();
            match tokio::time::timeout(
                std::time::Duration::from_millis(500),
                reader.read_line(&mut response_line),
            )
            .await
            {
                Ok(Ok(n)) if n > 0 => {}
                _ => continue,
            }

            let resp: serde_json::Value = serde_json::from_str(&response_line).unwrap_or_default();

            if resp.get("error").is_some() {
                continue;
            }

            let caps =
                biomeos_core::socket_discovery::cap_probe::extract_capabilities_from_response(
                    &resp,
                );
            if !caps.is_empty() {
                return Ok(caps);
            }
        }
        Ok(vec![])
    }

    /// Get XDG-compliant socket directories for primal discovery
    ///
    /// EVOLVED (Jan 27, 2026): No more hardcoded `/tmp` - uses `SystemPaths`
    ///
    /// # Priority Order
    /// 1. XDG runtime directory: `$XDG_RUNTIME_DIR/biomeos/`
    /// 2. User runtime fallback: `/tmp/biomeos-$USER/`
    /// 3. Legacy compatibility: `/tmp/` (for existing deployments)
    #[must_use]
    pub fn get_socket_directories() -> Vec<PathBuf> {
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

        // Priority 4: /tmp/biomeos-{FAMILY_ID} (family-scoped, Docker/NUCLEUS deployments)
        for key in ["BIOMEOS_FAMILY_ID", "FAMILY_ID"] {
            if let Ok(fid) = std::env::var(key) {
                let path = biomeos_types::constants::runtime_paths::fallback_runtime_dir(&fid);
                if !dirs.contains(&path) && path.exists() {
                    dirs.push(path);
                }
                break;
            }
        }

        // Priority 5: /tmp/biomeos-$USER (user-namespaced, via centralized constant)
        if let Ok(user) = std::env::var("USER") {
            let path = biomeos_types::constants::runtime_paths::fallback_runtime_dir(&user);
            if !dirs.contains(&path) && path.exists() {
                dirs.push(path);
            }
        }

        // Priority 6: /tmp/biomeos-default (common NUCLEUS default)
        {
            let default_path =
                biomeos_types::constants::runtime_paths::fallback_runtime_dir(DEFAULT_FAMILY_ID);
            if !dirs.contains(&default_path) && default_path.exists() {
                dirs.push(default_path);
            }
        }

        // Priority 7: /tmp/biomeos (base deploy layout without family suffix)
        // Deploy scripts often create sockets in FALLBACK_RUNTIME_BASE directly
        // (empty family_id), whereas NUCLEUS defaults family_id="default" →
        // /tmp/biomeos-default. Both must be scanned to avoid socket mismatch.
        {
            let base =
                PathBuf::from(biomeos_types::constants::runtime_paths::FALLBACK_RUNTIME_BASE);
            if !dirs.contains(&base) && base.exists() {
                dirs.push(base);
            }
        }

        // Priority 8: fallback runtime base (only if nothing else exists)
        if dirs.is_empty() {
            let base =
                PathBuf::from(biomeos_types::constants::runtime_paths::FALLBACK_RUNTIME_BASE);
            dirs.push(base);
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
            ((actual_capabilities as f64 / f64::from(expected_capabilities)) * 100.0).min(100.0);

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

    /// Get aggregated metrics (pure Rust via /proc - ecoBin v3).
    ///
    /// JSON-RPC method: `topology.metrics`
    pub async fn get_metrics(&self) -> Result<Value> {
        info!("📊 Neural API: get_metrics called");

        let cpu_percent = crate::proc_metrics::cpu_percent().await;
        let (memory_total, memory_used) = crate::proc_metrics::memory_bytes();
        let memory_total_mb = memory_total / 1024 / 1024;
        let memory_used_mb = memory_used / 1024 / 1024;
        let memory_percent = if memory_total > 0 {
            (memory_used as f64 / memory_total as f64) * 100.0
        } else {
            0.0
        };
        let uptime_seconds = crate::proc_metrics::uptime_seconds();

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
                "cpu_percent": cpu_percent,
                "memory_used_mb": memory_used_mb,
                "memory_total_mb": memory_total_mb,
                "memory_percent": memory_percent,
                "uptime_seconds": uptime_seconds
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
#[path = "topology_tests.rs"]
mod tests;
