// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Neural API Routing Layer
//!
//! **Universal IPC v3.0 + tarpc**: Uses `AtomicClient` for multi-transport routing
//! with protocol escalation to tarpc for hot-paths.
//!
//! Pure Rust implementation of capability-based primal routing.

#![forbid(unsafe_code)]

pub mod composition;
mod discovery;
mod discovery_composite;
mod discovery_primal;
mod discovery_registry;
mod forwarding;
mod registry;
#[cfg(test)]
mod forwarding_routing_tests;
#[cfg(test)]
mod forwarding_tests;
pub mod perceptron;
mod types;
pub mod weights;

use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use tokio::sync::RwLock;
use tokio::time::Duration;
use tracing::{debug, info};

use crate::living_graph::LivingGraph;
use biomeos_types::tarpc_types::ProtocolPreference;

pub use composition::{
    CompositionPattern, CompositionPatternRegistry, CompositionTier, TierCompositionPlan,
};
pub use perceptron::{
    DispatchTrainingRow, PerceptronDispatcher, PerceptronPhase, PerceptronWeights,
};
pub use types::{
    AtomicType, DiscoveredAtomic, DiscoveredPrimal, RegisteredCapability, RoutingMetrics,
};
pub use weights::{
    CapabilityUtilizationTracker, MethodUtilization, ProviderWeight, RoutingWeightTable,
    UtilizationSummary, WeightTableSummary,
};

/// Neural Router - Capability-based request routing with adaptive weights
pub struct NeuralRouter {
    /// Family ID for socket discovery
    pub(crate) family_id: String,

    /// Discovered primals cache (runtime discovery)
    discovered_primals: Arc<RwLock<HashMap<String, DiscoveredPrimal>>>,

    /// Capability Registry (dynamic registration)
    capability_registry: Arc<RwLock<HashMap<String, Vec<RegisteredCapability>>>>,

    /// Metrics collection
    metrics: Arc<RwLock<Vec<RoutingMetrics>>>,

    /// Routing weights for adaptive dispatch (Layer 4 evolution).
    /// Updated from dispatch outcomes, used by capability.call to
    /// prefer faster/more-reliable providers.
    routing_weights: Arc<RwLock<RoutingWeightTable>>,

    /// Composition patterns — named method sequences forming emergent systems.
    /// Seeded with canonical patterns, extended by graph loading and primal.announce.
    composition_patterns: Arc<RwLock<CompositionPatternRegistry>>,

    /// Capability utilization tracker — records hot/cold method usage.
    /// Fed by every `capability.call` dispatch; queried for graph pre-staging
    /// decisions and the future learned routing layer.
    utilization_tracker: Arc<RwLock<CapabilityUtilizationTracker>>,

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

    /// A/B shadow log counter for L4 weighted routing rollout.
    /// Logs both first-match and weighted choices for the first
    /// `SHADOW_LOG_DISPATCH_LIMIT` dispatches to validate scoring.
    pub(crate) weighted_dispatch_counter: AtomicU64,

    /// Count of dispatches where weighted selection disagreed with first-match.
    pub(crate) weighted_disagreement_counter: AtomicU64,

    /// L5 perceptron dispatcher — shadow mode alongside L4 rule-based routing.
    /// `None` until weights are loaded (mock or trained).
    pub(crate) perceptron: Option<PerceptronDispatcher>,

    /// Pending multi-provider dispatches awaiting outcome data.
    /// Key: `(capability, provider)` — cleared when `record_dispatch_outcome` fires.
    pending_dispatches: RwLock<HashMap<(String, String), perceptron::PendingDispatch>>,

    /// Ring buffer of completed training rows for barraCuda consumption.
    /// Drained via `neural_api.training_data` RPC.
    training_log: RwLock<VecDeque<DispatchTrainingRow>>,
}

impl NeuralRouter {
    /// Create a new Neural Router
    pub fn new(family_id: impl Into<String>) -> Self {
        Self {
            family_id: family_id.into(),
            discovered_primals: Arc::new(RwLock::new(HashMap::new())),
            capability_registry: Arc::new(RwLock::new(HashMap::new())),
            metrics: Arc::new(RwLock::new(Vec::new())),
            routing_weights: Arc::new(RwLock::new(RoutingWeightTable::new())),
            composition_patterns: Arc::new(RwLock::new(
                CompositionPatternRegistry::with_canonical_patterns(),
            )),
            utilization_tracker: Arc::new(RwLock::new(CapabilityUtilizationTracker::new())),
            request_timeout: biomeos_types::constants::timeouts::ROUTER_WEIGHT_EVICTION_INTERVAL,
            living_graph: None,
            protocol_preference: biomeos_types::tarpc_types::protocol_from_env(),
            lazy_rescan_attempted: AtomicBool::new(false),
            self_socket_path: RwLock::new(None),
            weighted_dispatch_counter: AtomicU64::new(0),
            weighted_disagreement_counter: AtomicU64::new(0),
            perceptron: None,
            pending_dispatches: RwLock::new(HashMap::new()),
            training_log: RwLock::new(VecDeque::new()),
        }
    }

    /// Create a new Neural Router with persistent routing weights backed by redb.
    ///
    /// Weights are loaded from the database on startup and flushed after
    /// every mutation, surviving process restarts.
    pub fn with_persistent_weights(
        family_id: impl Into<String>,
        weights_path: &std::path::Path,
    ) -> Self {
        let weights = RoutingWeightTable::open(weights_path);
        Self {
            family_id: family_id.into(),
            discovered_primals: Arc::new(RwLock::new(HashMap::new())),
            capability_registry: Arc::new(RwLock::new(HashMap::new())),
            metrics: Arc::new(RwLock::new(Vec::new())),
            routing_weights: Arc::new(RwLock::new(weights)),
            composition_patterns: Arc::new(RwLock::new(
                CompositionPatternRegistry::with_canonical_patterns(),
            )),
            utilization_tracker: Arc::new(RwLock::new(CapabilityUtilizationTracker::new())),
            request_timeout: biomeos_types::constants::timeouts::ROUTER_WEIGHT_EVICTION_INTERVAL,
            living_graph: None,
            protocol_preference: biomeos_types::tarpc_types::protocol_from_env(),
            lazy_rescan_attempted: AtomicBool::new(false),
            self_socket_path: RwLock::new(None),
            weighted_dispatch_counter: AtomicU64::new(0),
            weighted_disagreement_counter: AtomicU64::new(0),
            perceptron: None,
            pending_dispatches: RwLock::new(HashMap::new()),
            training_log: RwLock::new(VecDeque::new()),
        }
    }

    /// Attach a living graph for protocol-aware routing
    pub fn with_living_graph(mut self, graph: Arc<LivingGraph>) -> Self {
        self.living_graph = Some(graph);
        self
    }

    /// Attach a perceptron dispatcher for L5 shadow-mode routing.
    #[must_use]
    pub fn with_perceptron(mut self, dispatcher: PerceptronDispatcher) -> Self {
        self.perceptron = Some(dispatcher);
        self
    }

    /// Set protocol preference override
    #[must_use]
    pub const fn with_protocol_preference(mut self, preference: ProtocolPreference) -> Self {
        self.protocol_preference = preference;
        self
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

    /// Record a dispatch outcome in the routing weight table.
    ///
    /// Called after every `capability.call` forward to build the adaptive
    /// routing surface. Weights accumulate EWMA latency and error rate,
    /// driving future provider selection. Also completes any pending
    /// training row from `select_primary()`.
    pub async fn record_dispatch_outcome(
        &self,
        capability: &str,
        provider: &str,
        success: bool,
        latency_ms: u64,
    ) {
        let mut weights = self.routing_weights.write().await;
        weights.record_outcome(capability, provider, success, latency_ms);
        drop(weights);

        let key = (capability.to_owned(), provider.to_owned());
        let pending = self.pending_dispatches.write().await.remove(&key);
        if let Some(pd) = pending {
            let row = DispatchTrainingRow {
                capability: pd.capability,
                candidates: pd.candidates,
                features: pd.features,
                chosen_idx: pd.chosen_idx,
                success,
                latency_ms,
                l4_score: pd.l4_score,
                timestamp: chrono::Utc::now().timestamp(),
            };
            const MAX_TRAINING_ROWS: usize = 10_000;
            let mut log = self.training_log.write().await;
            if log.len() >= MAX_TRAINING_ROWS {
                log.pop_front();
            }
            log.push_back(row);
        }
    }

    /// Stash a pending dispatch context for training data emission.
    ///
    /// Called from `select_primary()` when multiple providers are available.
    /// The pending context is completed by `record_dispatch_outcome()`.
    pub(crate) async fn stash_pending_dispatch(
        &self,
        capability: &str,
        provider: &str,
        pending: perceptron::PendingDispatch,
    ) {
        let key = (capability.to_owned(), provider.to_owned());
        let mut map = self.pending_dispatches.write().await;
        // Evict stale entries (>30s) to prevent leaks from dropped dispatches
        map.retain(|_, pd| pd.created_at.elapsed().as_secs() < 30);
        map.insert(key, pending);
    }

    /// Drain all completed training rows for barraCuda consumption.
    pub async fn drain_training_data(&self) -> Vec<DispatchTrainingRow> {
        self.training_log.write().await.drain(..).collect()
    }

    /// Number of buffered training rows.
    pub async fn training_data_count(&self) -> usize {
        self.training_log.read().await.len()
    }

    /// Select the best provider for a capability using routing weights.
    ///
    /// Returns `None` if all candidates are circuit-broken or no candidates
    /// are registered. Falls back to the first candidate if the weight table
    /// has no observations yet.
    pub async fn select_weighted_provider(&self, capability: &str) -> Option<Arc<str>> {
        let registry = self.capability_registry.read().await;
        let providers = registry.get(capability)?;
        if providers.is_empty() {
            return None;
        }

        let candidates: Vec<Arc<str>> = providers.iter().map(|p| p.primal_name.clone()).collect();

        let weights = self.routing_weights.read().await;
        weights
            .select_best(capability, &candidates)
            .cloned()
            .or_else(|| candidates.into_iter().next())
    }

    /// Get a snapshot of routing weights for introspection.
    pub async fn get_routing_weights(&self) -> Vec<ProviderWeight> {
        self.routing_weights.read().await.snapshot()
    }

    /// Get routing weight summary statistics.
    pub async fn get_weight_summary(&self) -> WeightTableSummary {
        self.routing_weights.read().await.summary()
    }

    /// Get A/B shadow routing statistics.
    pub fn shadow_stats(&self) -> (u64, u64) {
        let total = self.weighted_dispatch_counter.load(Ordering::Relaxed);
        let disagreements = self.weighted_disagreement_counter.load(Ordering::Relaxed);
        (total, disagreements)
    }

    /// L5 perceptron shadow statistics: `(total, disagreements)`.
    /// Returns `None` if no perceptron is attached.
    pub fn perceptron_shadow_stats(&self) -> Option<(u64, u64)> {
        self.perceptron.as_ref().map(|p| p.shadow_stats())
    }

    /// Current perceptron phase, if any.
    pub fn perceptron_phase(&self) -> Option<PerceptronPhase> {
        self.perceptron.as_ref().map(|p| p.phase())
    }

    /// Whether the perceptron has remote inference wired.
    pub fn perceptron_has_remote_infer(&self) -> bool {
        self.perceptron
            .as_ref()
            .is_some_and(|p| p.has_remote_infer())
    }

    /// Set a provider affinity hint (from primal.announce cost_hints).
    pub async fn set_provider_affinity(&self, capability: &str, provider: &str, affinity: f64) {
        let mut weights = self.routing_weights.write().await;
        weights.set_affinity(capability, provider, affinity);
    }

    /// Set a provider cost hint (from primal.announce cost_hints).
    pub async fn set_provider_cost_hint(&self, capability: &str, provider: &str, cost: f64) {
        let mut weights = self.routing_weights.write().await;
        weights.set_cost_hint(capability, provider, cost);
    }

    /// Set topology affinity for a provider based on transport endpoint.
    pub async fn set_provider_topology_affinity(
        &self,
        capability: &str,
        provider: &str,
        endpoint: &biomeos_core::TransportEndpoint,
    ) {
        let ta = weights::topology_affinity_for_endpoint(endpoint);
        let mut weights = self.routing_weights.write().await;
        weights.set_topology_affinity(capability, provider, ta);
    }

    /// Whether the routing weight table is backed by persistent storage.
    pub async fn weights_are_persistent(&self) -> bool {
        self.routing_weights.read().await.is_persistent()
    }

    /// Flush routing weights to persistent storage (no-op if in-memory).
    pub async fn flush_weights(&self) {
        self.routing_weights.read().await.flush();
    }

    /// Record a capability method call for utilization tracking.
    pub async fn record_utilization(&self, method: &str) {
        self.utilization_tracker.write().await.record(method);
    }

    /// Get utilization summary as JSON (for RPC responses).
    pub async fn utilization_json(&self) -> serde_json::Value {
        self.utilization_tracker.read().await.to_json()
    }

    /// Get the utilization summary statistics.
    pub async fn utilization_summary(&self) -> UtilizationSummary {
        self.utilization_tracker.read().await.summary()
    }

    /// Get hot methods (top N by call count).
    pub async fn hot_methods(&self, n: usize) -> Vec<MethodUtilization> {
        self.utilization_tracker.read().await.hot_methods(n)
    }

    /// Get cold methods (below threshold).
    pub async fn cold_methods(&self, threshold: u64) -> Vec<MethodUtilization> {
        self.utilization_tracker
            .read()
            .await
            .cold_methods(threshold)
    }

    /// Classify a capability domain into its composition tier.
    pub fn classify_tier(&self, domain: &str, provider: &str) -> CompositionTier {
        CompositionTier::classify(domain, provider)
    }

    /// Get all registered composition patterns.
    pub async fn get_composition_patterns(&self) -> Vec<CompositionPattern> {
        self.composition_patterns
            .read()
            .await
            .all()
            .into_iter()
            .cloned()
            .collect()
    }

    /// Look up a composition pattern by name.
    pub async fn get_pattern(&self, name: &str) -> Option<CompositionPattern> {
        self.composition_patterns.read().await.get(name).cloned()
    }

    /// Register a new composition pattern (from graph loading or primal.announce).
    pub async fn register_composition_pattern(&self, pattern: CompositionPattern) {
        self.composition_patterns.write().await.register(pattern);
    }

    /// Hot-reload composition patterns (re-seed canonical, preserve runtime).
    ///
    /// Useful after mesh topology changes (new gate joins) to refresh
    /// canonical patterns without restarting Neural API.
    pub async fn reload_composition_patterns(&self) -> usize {
        let count = self.composition_patterns.write().await.reload_canonical();
        info!("composition patterns reloaded: {count} patterns active");
        count
    }

    /// Get a tier composition plan.
    pub async fn plan_tier(&self, tier: CompositionTier) -> TierCompositionPlan {
        let registry = self.composition_patterns.read().await;
        composition::plan_tier(tier, &registry)
    }

    /// Get composition patterns as JSON (for RPC responses).
    pub async fn composition_patterns_json(&self) -> serde_json::Value {
        self.composition_patterns.read().await.to_json()
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
