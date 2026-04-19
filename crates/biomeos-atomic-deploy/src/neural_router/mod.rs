// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Neural API Routing Layer
//!
//! **Universal IPC v3.0 + tarpc**: Uses `AtomicClient` for multi-transport routing
//! with protocol escalation to tarpc for hot-paths.
//!
//! Pure Rust implementation of capability-based primal routing.

#![forbid(unsafe_code)]

mod discovery;
mod discovery_composite;
mod discovery_primal;
mod discovery_registry;
mod forwarding;
#[cfg(test)]
mod forwarding_routing_tests;
#[cfg(test)]
mod forwarding_tests;
mod types;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::RwLock;
use tokio::time::Duration;
use tracing::{debug, info, warn};

use crate::living_graph::LivingGraph;
use biomeos_core::TransportEndpoint;
use biomeos_types::tarpc_types::ProtocolPreference;

pub use types::{
    AtomicType, DiscoveredAtomic, DiscoveredPrimal, RegisteredCapability, RoutingMetrics,
};

/// Neural Router - Capability-based request routing
pub struct NeuralRouter {
    /// Family ID for socket discovery
    pub(crate) family_id: String,

    /// Discovered primals cache (runtime discovery)
    discovered_primals: Arc<RwLock<HashMap<String, DiscoveredPrimal>>>,

    /// Capability Registry (dynamic registration)
    capability_registry: Arc<RwLock<HashMap<String, Vec<RegisteredCapability>>>>,

    /// Metrics collection
    metrics: Arc<RwLock<Vec<RoutingMetrics>>>,

    /// Request timeout
    pub(crate) request_timeout: Duration,

    /// Living graph for protocol state tracking
    pub(crate) living_graph: Option<Arc<LivingGraph>>,

    /// Protocol preference from environment
    pub(crate) protocol_preference: ProtocolPreference,

    /// Whether a lazy rescan has already been attempted this session.
    /// Prevents repeated rescans on every miss in a tight loop.
    pub(crate) lazy_rescan_attempted: AtomicBool,

    /// Neural API's own socket path, excluded from auto-discovery to prevent
    /// self-registration pollution (GAP-MATRIX-08).
    self_socket_path: RwLock<Option<PathBuf>>,
}

impl NeuralRouter {
    /// Create a new Neural Router
    pub fn new(family_id: impl Into<String>) -> Self {
        Self {
            family_id: family_id.into(),
            discovered_primals: Arc::new(RwLock::new(HashMap::new())),
            capability_registry: Arc::new(RwLock::new(HashMap::new())),
            metrics: Arc::new(RwLock::new(Vec::new())),
            request_timeout: Duration::from_secs(30),
            living_graph: None,
            protocol_preference: biomeos_types::tarpc_types::protocol_from_env(),
            lazy_rescan_attempted: AtomicBool::new(false),
            self_socket_path: RwLock::new(None),
        }
    }

    /// Attach a living graph for protocol-aware routing
    pub fn with_living_graph(mut self, graph: Arc<LivingGraph>) -> Self {
        self.living_graph = Some(graph);
        self
    }

    /// Set protocol preference override
    #[must_use]
    pub const fn with_protocol_preference(mut self, preference: ProtocolPreference) -> Self {
        self.protocol_preference = preference;
        self
    }

    /// Set the Neural API's own socket path so auto-discovery excludes it.
    pub async fn set_self_socket_path(&self, path: PathBuf) {
        *self.self_socket_path.write().await = Some(path);
    }

    /// Register a capability with a transport endpoint
    pub async fn register_capability(
        &self,
        capability: impl Into<String>,
        primal_name: impl Into<String>,
        endpoint: TransportEndpoint,
        source: impl Into<String>,
    ) -> anyhow::Result<()> {
        let capability = capability.into();
        let primal_name = primal_name.into();

        info!(
            "✅ Registered capability: {} → {} @ {}",
            capability,
            primal_name,
            endpoint.display_string()
        );

        let registration = RegisteredCapability {
            capability: Arc::from(capability.as_str()),
            primal_name: Arc::from(primal_name.as_str()),
            endpoint,
            registered_at: chrono::Utc::now(),
            source: Arc::from(source.into().as_str()),
        };

        let mut registry = self.capability_registry.write().await;
        registry
            .entry(capability.to_string())
            .or_default()
            .push(registration);

        Ok(())
    }

    /// Convenience: register a capability bound to a Unix socket path
    pub async fn register_capability_unix(
        &self,
        capability: impl Into<String>,
        primal_name: impl Into<String>,
        socket_path: impl Into<PathBuf>,
        source: impl Into<String>,
    ) -> anyhow::Result<()> {
        let endpoint = TransportEndpoint::UnixSocket {
            path: socket_path.into(),
        };
        self.register_capability(capability, primal_name, endpoint, source)
            .await
    }

    /// List all registered capabilities
    pub async fn list_capabilities(&self) -> HashMap<String, Vec<RegisteredCapability>> {
        self.capability_registry.read().await.clone()
    }

    /// Get providers for a specific capability
    pub async fn get_capability_providers(
        &self,
        capability: &str,
    ) -> Option<Vec<RegisteredCapability>> {
        self.capability_registry
            .read()
            .await
            .get(capability)
            .cloned()
    }

    /// Log routing metrics for learning
    pub async fn log_metric(&self, metric: RoutingMetrics) {
        debug!(
            "📊 Metric logged: {} - {}ms",
            metric.method, metric.latency_ms
        );

        let mut metrics = self.metrics.write().await;
        metrics.push(metric);
    }

    /// Get all collected metrics (for analysis)
    pub async fn get_metrics(&self) -> Vec<RoutingMetrics> {
        self.metrics.read().await.clone()
    }

    /// Clear metrics cache
    pub async fn clear_metrics(&self) {
        self.metrics.write().await.clear();
    }

    /// Probe a newly-spawned primal and register its capabilities.
    ///
    /// Call after `wait_for_socket` / `wait_for_tcp_port` succeeds so the
    /// primal is guaranteed to be listening. This bridges the gap where
    /// primals spawned after boot discovery (or between lazy rescans) would
    /// otherwise remain invisible to `capability.call` routing.
    pub async fn register_spawned_primal(
        &self,
        primal_name: &str,
        socket_path: Option<&std::path::Path>,
        tcp_port: Option<u16>,
    ) -> usize {
        use biomeos_core::socket_discovery::cap_probe::probe_unix_socket_capabilities_list;

        let capabilities: Vec<String> = if let Some(port) = tcp_port {
            let addr = format!("{}:{}", biomeos_types::constants::DEFAULT_LOCALHOST, port);
            crate::neural_api_server::discovery_init::probe_tcp_capabilities_public(&addr).await
        } else if let Some(path) = socket_path {
            probe_unix_socket_capabilities_list(path).await
        } else {
            return 0;
        };

        if capabilities.is_empty() {
            debug!(
                "Post-spawn probe for {} returned no capabilities",
                primal_name
            );
            return 0;
        }

        let mut registered = 0;
        for cap in &capabilities {
            let result = if let Some(port) = tcp_port {
                let endpoint = biomeos_core::TransportEndpoint::TcpSocket {
                    host: Arc::from(biomeos_types::constants::DEFAULT_LOCALHOST),
                    port,
                };
                self.register_capability(cap, primal_name, endpoint, "post-spawn")
                    .await
            } else if let Some(path) = socket_path {
                self.register_capability_unix(cap, primal_name, path, "post-spawn")
                    .await
            } else {
                continue;
            };

            if let Err(e) = result {
                warn!(
                    "Failed to register {}.{} post-spawn: {}",
                    primal_name, cap, e
                );
            } else {
                registered += 1;
            }
        }

        if registered > 0 {
            info!(
                "✅ Post-spawn registered {} capabilities for {}",
                registered, primal_name
            );
        }
        registered
    }

    /// Invalidate discovery cache (force rediscovery)
    pub async fn invalidate_cache(&self) {
        self.discovered_primals.write().await.clear();
        self.lazy_rescan_attempted.store(false, Ordering::Relaxed);
        info!("🔄 Discovery cache invalidated");
    }

    /// Rescan socket directories for newly-appeared primals.
    ///
    /// Called lazily on the first `capability.call` miss (BM-04 fix). Only runs
    /// once per session — subsequent misses fast-fail. Reset via
    /// `invalidate_cache()` or `topology.rescan`.
    pub(crate) async fn lazy_rescan_sockets(&self) -> usize {
        if self.lazy_rescan_attempted.swap(true, Ordering::Relaxed) {
            return 0;
        }

        info!("🔄 Lazy rescan: capability miss triggered socket re-discovery");
        let socket_dirs = crate::handlers::TopologyHandler::get_socket_directories();
        let self_socket = self.self_socket_path.read().await.clone();
        let mut registered = 0usize;

        for socket_dir in &socket_dirs {
            let entries = match std::fs::read_dir(socket_dir) {
                Ok(e) => e,
                Err(_) => continue,
            };

            for entry in entries.flatten() {
                let path = entry.path();
                let filename = match path.file_name().and_then(|n| n.to_str()) {
                    Some(f) => f.to_string(),
                    None => continue,
                };

                if !filename.ends_with(".sock") {
                    continue;
                }

                if self_socket.as_ref().is_some_and(|s| *s == path) {
                    continue;
                }

                let primal_name = match filename.strip_suffix(".sock") {
                    Some(base) => base.split('-').next().unwrap_or(base).to_string(),
                    None => continue,
                };

                let socket_str = path.to_string_lossy().to_string();
                let capabilities = probe_primal_capabilities_standalone(socket_str.as_str()).await;

                if capabilities.is_empty() {
                    debug!("   {} — no capabilities during lazy rescan", primal_name);
                    continue;
                }

                for cap in &capabilities {
                    if let Err(e) = self
                        .register_capability_unix(cap, &primal_name, &path, "lazy-rescan")
                        .await
                    {
                        warn!("   Failed to register {}.{}: {}", primal_name, cap, e);
                    }
                }

                info!(
                    "   🔍 Lazy rescan discovered {} — {} capabilities",
                    primal_name,
                    capabilities.len(),
                );
                registered += capabilities.len();
            }
        }

        if registered > 0 {
            info!("✅ Lazy rescan registered {} new capabilities", registered);
        }

        registered
    }

    /// Reset the lazy-rescan gate so the next miss triggers a fresh scan.
    pub fn reset_lazy_rescan(&self) {
        self.lazy_rescan_attempted.store(false, Ordering::Relaxed);
    }
}

/// Probe a primal socket for capabilities (standalone, no `NeuralApiServer` dependency).
///
/// Delegates to [`biomeos_core::socket_discovery::probe_unix_socket_capabilities_list`].
pub(crate) async fn probe_primal_capabilities_standalone(socket_path: &str) -> Vec<String> {
    biomeos_core::socket_discovery::probe_unix_socket_capabilities_list(std::path::Path::new(
        socket_path,
    ))
    .await
}
