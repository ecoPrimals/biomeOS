// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Provider weight scoring — per-provider adaptive routing intelligence.
//!
//! Each `ProviderWeight` combines static hints (from `primal.announce`)
//! with dynamic observations (from dispatch outcomes) to produce a
//! single routing score. Circuit breakers protect against cascading
//! failures from unhealthy providers.

use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// EWMA smoothing factor (0.0–1.0). Higher = more responsive to recent data.
pub(crate) const ALPHA: f64 = 0.3;

/// Consecutive failures before circuit opens.
pub(crate) const CIRCUIT_BREAKER_THRESHOLD: u32 = 5;

/// Seconds before a half-open probe is allowed.
pub(crate) const CIRCUIT_BREAKER_COOLDOWN_SECS: i64 = 30;

/// Bonus score for providers with < 5 observations (encourages exploration).
pub(crate) const EXPLORATION_BONUS: f64 = 0.1;

pub(crate) fn ewma(current: f64, new_sample: f64, alpha: f64) -> f64 {
    alpha * new_sample + (1.0 - alpha) * current
}

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
