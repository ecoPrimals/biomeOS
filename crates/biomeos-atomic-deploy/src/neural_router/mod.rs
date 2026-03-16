// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Neural API Routing Layer
//!
//! **Universal IPC v3.0 + tarpc**: Uses AtomicClient for multi-transport routing
//! with protocol escalation to tarpc for hot-paths.
//!
//! Pure Rust implementation of capability-based primal routing.

#![deny(unsafe_code)]

mod discovery;
mod forwarding;
mod types;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::Duration;
use tracing::{debug, info};

use crate::living_graph::LivingGraph;
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
        }
    }

    /// Attach a living graph for protocol-aware routing
    pub fn with_living_graph(mut self, graph: Arc<LivingGraph>) -> Self {
        self.living_graph = Some(graph);
        self
    }

    /// Set protocol preference override
    pub fn with_protocol_preference(mut self, preference: ProtocolPreference) -> Self {
        self.protocol_preference = preference;
        self
    }

    /// Register a capability
    pub async fn register_capability(
        &self,
        capability: impl Into<String>,
        primal_name: impl Into<String>,
        socket_path: impl Into<PathBuf>,
        source: impl Into<String>,
    ) -> anyhow::Result<()> {
        let capability = capability.into();
        let primal_name = primal_name.into();
        let socket_path = socket_path.into();

        let registration = RegisteredCapability {
            capability: Arc::from(capability.as_str()),
            primal_name: Arc::from(primal_name.as_str()),
            socket_path,
            registered_at: chrono::Utc::now(),
            source: Arc::from(source.into().as_str()),
        };

        let mut registry = self.capability_registry.write().await;
        registry
            .entry(capability.to_string())
            .or_default()
            .push(registration);

        info!("✅ Registered capability: {} → {}", capability, primal_name);

        Ok(())
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

    /// Invalidate discovery cache (force rediscovery)
    pub async fn invalidate_cache(&self) {
        self.discovered_primals.write().await.clear();
        info!("🔄 Discovery cache invalidated");
    }
}
