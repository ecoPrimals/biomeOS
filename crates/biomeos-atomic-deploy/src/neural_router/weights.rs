// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Routing weights — adaptive dispatch intelligence.
//!
//! Transforms the static capability registry into a weighted routing surface
//! where dispatch decisions are informed by operational data. Each provider
//! (primal endpoint) accumulates metrics that influence future routing choices.
//!
//! # Evolution model
//!
//! This is Layer 4 of the Neural API evolution: adaptive routing. The weight
//! system treats routing decisions as a forward pass through a simple network:
//!
//! ```text
//! Input: capability.call { domain, operation }
//!   → Candidate providers (registry lookup)
//!   → Score each candidate (latency, error rate, affinity, cost hint)
//!   → Select highest-scoring provider
//!   → Forward request
//!   → Record outcome → update weights
//! ```
//!
//! As more data flows, weights converge toward optimal routing. Layer 5
//! (learned routing) will replace the scoring function with a trained model.

use std::collections::HashMap;
use std::sync::Arc;
use serde::{Deserialize, Serialize};

/// Weight for a single provider serving a capability.
///
/// Combines static hints (from `primal.announce` or config) with dynamic
/// observations (from dispatch outcomes). The `score()` method produces a
/// single routing preference value.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderWeight {
    /// Provider primal name.
    pub provider: Arc<str>,
    /// Capability domain this weight applies to.
    pub capability: Arc<str>,
    /// Exponentially weighted moving average of latency (ms).
    pub ewma_latency_ms: f64,
    /// Exponentially weighted moving average of error rate (0.0–1.0).
    pub ewma_error_rate: f64,
    /// Total successful dispatches through this provider.
    pub success_count: u64,
    /// Total failed dispatches through this provider.
    pub failure_count: u64,
    /// Static affinity hint from primal.announce or config (0.0–1.0).
    /// Higher = preferred. Defaults to 0.5 (neutral).
    pub affinity: f64,
    /// Cost hint from primal.announce (arbitrary units, lower is cheaper).
    /// `None` = no cost information available.
    pub cost_hint: Option<f64>,
    /// Whether this provider is in circuit-breaker open state.
    pub circuit_open: bool,
    /// When the circuit breaker last opened (for half-open probing).
    pub circuit_opened_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Consecutive failures (for circuit breaker threshold).
    pub consecutive_failures: u32,
}

impl ProviderWeight {
    /// Create a new weight with neutral defaults.
    pub fn new(provider: impl Into<Arc<str>>, capability: impl Into<Arc<str>>) -> Self {
        Self {
            provider: provider.into(),
            capability: capability.into(),
            ewma_latency_ms: 50.0, // optimistic default
            ewma_error_rate: 0.0,
            success_count: 0,
            failure_count: 0,
            affinity: 0.5,
            cost_hint: None,
            circuit_open: false,
            circuit_opened_at: None,
            consecutive_failures: 0,
        }
    }

    /// Record a successful dispatch.
    pub fn record_success(&mut self, latency_ms: u64) {
        self.success_count += 1;
        self.consecutive_failures = 0;

        if self.circuit_open {
            self.circuit_open = false;
            self.circuit_opened_at = None;
        }

        self.ewma_latency_ms = ewma(self.ewma_latency_ms, latency_ms as f64, ALPHA);
        self.ewma_error_rate = ewma(self.ewma_error_rate, 0.0, ALPHA);
    }

    /// Record a failed dispatch.
    pub fn record_failure(&mut self) {
        self.failure_count += 1;
        self.consecutive_failures += 1;
        self.ewma_error_rate = ewma(self.ewma_error_rate, 1.0, ALPHA);

        if self.consecutive_failures >= CIRCUIT_BREAKER_THRESHOLD {
            self.circuit_open = true;
            self.circuit_opened_at = Some(chrono::Utc::now());
        }
    }

    /// Whether this provider should be considered for routing.
    ///
    /// Open circuits are excluded unless enough time has passed for
    /// a half-open probe.
    pub fn is_available(&self) -> bool {
        if !self.circuit_open {
            return true;
        }
        // Half-open: allow one probe after cooldown
        self.circuit_opened_at
            .map(|opened| {
                let elapsed = chrono::Utc::now() - opened;
                elapsed >= chrono::Duration::seconds(CIRCUIT_BREAKER_COOLDOWN_SECS)
            })
            .unwrap_or(true)
    }

    /// Compute a routing score (higher = preferred).
    ///
    /// Scoring function:
    /// ```text
    /// score = affinity * (1 - error_rate) / (1 + normalized_latency) - cost_penalty
    /// ```
    ///
    /// Cold providers (< 5 dispatches) get a slight exploration bonus.
    pub fn score(&self) -> f64 {
        if self.circuit_open && !self.is_available() {
            return 0.0;
        }

        let reliability = 1.0 - self.ewma_error_rate;
        let latency_factor = 1.0 / (1.0 + self.ewma_latency_ms / 100.0);
        let cost_penalty = self.cost_hint.map_or(0.0, |c| c / 1000.0);

        let base = self.affinity * reliability * latency_factor - cost_penalty;

        let total = self.success_count + self.failure_count;
        if total < 5 {
            base + EXPLORATION_BONUS
        } else {
            base
        }
    }

    /// Total dispatches (successes + failures).
    pub fn total_dispatches(&self) -> u64 {
        self.success_count + self.failure_count
    }
}

/// EWMA smoothing factor (0.0–1.0). Higher = more responsive to recent data.
const ALPHA: f64 = 0.3;

/// Consecutive failures before circuit opens.
const CIRCUIT_BREAKER_THRESHOLD: u32 = 5;

/// Seconds before a half-open probe is allowed.
const CIRCUIT_BREAKER_COOLDOWN_SECS: i64 = 30;

/// Bonus score for providers with < 5 observations (encourages exploration).
const EXPLORATION_BONUS: f64 = 0.1;

fn ewma(current: f64, new_sample: f64, alpha: f64) -> f64 {
    alpha * new_sample + (1.0 - alpha) * current
}

/// The routing weight table — adaptive dispatch intelligence.
///
/// Maps `(capability, provider)` → `ProviderWeight`. Used by `capability.call`
/// to select the best provider when multiple can serve a request.
#[derive(Debug, Default)]
pub struct RoutingWeightTable {
    /// (capability, provider) → weight
    weights: HashMap<(Arc<str>, Arc<str>), ProviderWeight>,
}

impl RoutingWeightTable {
    /// Create an empty weight table.
    pub fn new() -> Self {
        Self::default()
    }

    /// Get or create a weight entry for a provider+capability pair.
    pub fn get_or_create(
        &mut self,
        capability: &str,
        provider: &str,
    ) -> &mut ProviderWeight {
        let key = (Arc::from(capability), Arc::from(provider));
        self.weights
            .entry(key)
            .or_insert_with(|| ProviderWeight::new(provider, capability))
    }

    /// Get a weight entry (read-only).
    pub fn get(&self, capability: &str, provider: &str) -> Option<&ProviderWeight> {
        let key = (Arc::from(capability), Arc::from(provider));
        self.weights.get(&key)
    }

    /// Select the best provider for a capability from a list of candidates.
    ///
    /// Returns `None` if no candidates are available (all circuit-broken).
    pub fn select_best<'a>(&self, capability: &str, candidates: &'a [Arc<str>]) -> Option<&'a Arc<str>> {
        let mut best: Option<(&Arc<str>, f64)> = None;

        for candidate in candidates {
            let key = (Arc::from(capability), candidate.clone());
            let score = self
                .weights
                .get(&key)
                .map(|w| {
                    if w.is_available() {
                        w.score()
                    } else {
                        0.0
                    }
                })
                // Unknown providers get a default score (encourages discovery).
                .unwrap_or(0.5 + EXPLORATION_BONUS);

            if score > 0.0 {
                match best {
                    Some((_, best_score)) if score > best_score => {
                        best = Some((candidate, score));
                    }
                    None => {
                        best = Some((candidate, score));
                    }
                    _ => {}
                }
            }
        }

        best.map(|(provider, _)| provider)
    }

    /// Record a dispatch outcome, updating the provider's weight.
    pub fn record_outcome(
        &mut self,
        capability: &str,
        provider: &str,
        success: bool,
        latency_ms: u64,
    ) {
        let weight = self.get_or_create(capability, provider);
        if success {
            weight.record_success(latency_ms);
        } else {
            weight.record_failure();
        }
    }

    /// Set affinity hint for a provider (from primal.announce or config).
    pub fn set_affinity(&mut self, capability: &str, provider: &str, affinity: f64) {
        let weight = self.get_or_create(capability, provider);
        weight.affinity = affinity.clamp(0.0, 1.0);
    }

    /// Set cost hint for a provider (from primal.announce).
    pub fn set_cost_hint(&mut self, capability: &str, provider: &str, cost: f64) {
        let weight = self.get_or_create(capability, provider);
        weight.cost_hint = Some(cost);
    }

    /// All weights for a capability domain.
    pub fn weights_for_capability(&self, capability: &str) -> Vec<&ProviderWeight> {
        self.weights
            .iter()
            .filter(|((cap, _), _)| cap.as_ref() == capability)
            .map(|(_, w)| w)
            .collect()
    }

    /// Snapshot of all weights (for serialization / RPC response).
    pub fn snapshot(&self) -> Vec<ProviderWeight> {
        self.weights.values().cloned().collect()
    }

    /// Number of tracked provider+capability pairs.
    pub fn len(&self) -> usize {
        self.weights.len()
    }

    /// Whether the table is empty.
    pub fn is_empty(&self) -> bool {
        self.weights.is_empty()
    }

    /// Summary statistics for the weight table.
    pub fn summary(&self) -> WeightTableSummary {
        let total_dispatches: u64 = self
            .weights
            .values()
            .map(|w| w.total_dispatches())
            .sum();
        let circuit_open = self.weights.values().filter(|w| w.circuit_open).count();
        let providers: Vec<Arc<str>> = {
            let mut v: Vec<_> = self
                .weights
                .values()
                .map(|w| w.provider.clone())
                .collect();
            v.sort();
            v.dedup();
            v
        };
        let capabilities: Vec<Arc<str>> = {
            let mut v: Vec<_> = self
                .weights
                .values()
                .map(|w| w.capability.clone())
                .collect();
            v.sort();
            v.dedup();
            v
        };

        WeightTableSummary {
            entries: self.weights.len(),
            total_dispatches,
            circuit_open,
            unique_providers: providers.len(),
            unique_capabilities: capabilities.len(),
        }
    }
}

/// Summary of the routing weight table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeightTableSummary {
    /// Total (capability, provider) pairs tracked.
    pub entries: usize,
    /// Sum of all dispatches across all providers.
    pub total_dispatches: u64,
    /// Number of providers with open circuit breakers.
    pub circuit_open: usize,
    /// Distinct primal providers in the table.
    pub unique_providers: usize,
    /// Distinct capability domains in the table.
    pub unique_capabilities: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_weight_has_neutral_defaults() {
        let w = ProviderWeight::new("beardog", "crypto");
        assert_eq!(w.provider.as_ref(), "beardog");
        assert_eq!(w.affinity, 0.5);
        assert!(!w.circuit_open);
        assert!(w.is_available());
    }

    #[test]
    fn record_success_updates_ewma() {
        let mut w = ProviderWeight::new("beardog", "crypto");
        w.record_success(10);
        assert!(w.ewma_latency_ms < 50.0); // moved toward 10
        assert_eq!(w.success_count, 1);
        assert_eq!(w.consecutive_failures, 0);
    }

    #[test]
    fn record_failure_increments_consecutive() {
        let mut w = ProviderWeight::new("beardog", "crypto");
        for _ in 0..4 {
            w.record_failure();
        }
        assert!(!w.circuit_open);
        w.record_failure(); // 5th = threshold
        assert!(w.circuit_open);
    }

    #[test]
    fn circuit_breaker_blocks_routing() {
        let mut w = ProviderWeight::new("beardog", "crypto");
        for _ in 0..5 {
            w.record_failure();
        }
        assert_eq!(w.score(), 0.0);
    }

    #[test]
    fn success_resets_circuit_breaker() {
        let mut w = ProviderWeight::new("beardog", "crypto");
        for _ in 0..5 {
            w.record_failure();
        }
        assert!(w.circuit_open);
        w.record_success(10);
        assert!(!w.circuit_open);
        assert!(w.score() > 0.0);
    }

    #[test]
    fn lower_latency_scores_higher() {
        let mut fast = ProviderWeight::new("fast", "crypto");
        let mut slow = ProviderWeight::new("slow", "crypto");
        for _ in 0..10 {
            fast.record_success(5);
            slow.record_success(500);
        }
        assert!(fast.score() > slow.score());
    }

    #[test]
    fn higher_affinity_scores_higher() {
        let mut preferred = ProviderWeight::new("preferred", "crypto");
        let mut fallback = ProviderWeight::new("fallback", "crypto");
        preferred.affinity = 0.9;
        fallback.affinity = 0.1;
        for _ in 0..10 {
            preferred.record_success(50);
            fallback.record_success(50);
        }
        assert!(preferred.score() > fallback.score());
    }

    #[test]
    fn table_select_best_prefers_fast_provider() {
        let mut table = RoutingWeightTable::new();
        for _ in 0..10 {
            table.record_outcome("crypto", "fast_beardog", true, 5);
            table.record_outcome("crypto", "slow_beardog", true, 500);
        }
        let candidates = vec![Arc::from("fast_beardog"), Arc::from("slow_beardog")];
        let best = table.select_best("crypto", &candidates);
        assert_eq!(best.map(|b| b.as_ref()), Some("fast_beardog"));
    }

    #[test]
    fn table_select_best_skips_broken_circuit() {
        let mut table = RoutingWeightTable::new();
        for _ in 0..10 {
            table.record_outcome("crypto", "healthy", true, 50);
        }
        for _ in 0..5 {
            table.record_outcome("crypto", "broken", false, 0);
        }
        let candidates = vec![Arc::from("healthy"), Arc::from("broken")];
        let best = table.select_best("crypto", &candidates);
        assert_eq!(best.map(|b| b.as_ref()), Some("healthy"));
    }

    #[test]
    fn table_exploration_bonus_for_unknown_providers() {
        let table = RoutingWeightTable::new();
        let candidates = vec![Arc::from("unknown_provider")];
        let best = table.select_best("crypto", &candidates);
        assert!(best.is_some(), "unknown providers get exploration bonus");
    }

    #[test]
    fn table_set_affinity() {
        let mut table = RoutingWeightTable::new();
        table.set_affinity("crypto", "beardog", 0.9);
        let w = table.get("crypto", "beardog").unwrap();
        assert_eq!(w.affinity, 0.9);
    }

    #[test]
    fn table_set_cost_hint() {
        let mut table = RoutingWeightTable::new();
        table.set_cost_hint("compute", "toadstool", 100.0);
        let w = table.get("compute", "toadstool").unwrap();
        assert_eq!(w.cost_hint, Some(100.0));
    }

    #[test]
    fn table_summary_correct() {
        let mut table = RoutingWeightTable::new();
        table.record_outcome("crypto", "beardog", true, 10);
        table.record_outcome("storage", "nestgate", true, 20);
        table.record_outcome("crypto", "beardog", true, 15);
        let summary = table.summary();
        assert_eq!(summary.entries, 2);
        assert_eq!(summary.total_dispatches, 3);
        assert_eq!(summary.unique_providers, 2);
        assert_eq!(summary.unique_capabilities, 2);
    }

    #[test]
    fn table_snapshot_includes_all() {
        let mut table = RoutingWeightTable::new();
        table.record_outcome("crypto", "beardog", true, 10);
        table.record_outcome("storage", "nestgate", true, 20);
        let snap = table.snapshot();
        assert_eq!(snap.len(), 2);
    }
}
