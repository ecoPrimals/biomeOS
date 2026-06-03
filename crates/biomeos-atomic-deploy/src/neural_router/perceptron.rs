// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! L5 Perceptron routing — shadow mode consumer interface.
//!
//! Runs a single-layer perceptron alongside L4 weighted routing to compare
//! decisions. Phase 1 (shadow): the perceptron observes but does not decide.
//! Phase 2 (epsilon-greedy) and Phase 3 (graduation) are gated on training
//! data from barraCuda `ml.mlp_train`.
//!
//! Design: `wateringHole/NEURAL_API_PERCEPTRON_DESIGN.md`

#![forbid(unsafe_code)]

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use super::weights::{ProviderWeight, RoutingWeightTable};

/// Number of input features: 32 one-hot domain slots + 4 numeric per-provider features.
pub const FEATURE_DIM: usize = 36;

/// Weight vector for a single-layer perceptron (36 weights + 1 bias = 37).
pub const WEIGHT_VEC_LEN: usize = FEATURE_DIM + 1;

/// Perceptron operating phase.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PerceptronPhase {
    /// Shadow: rule decides, perceptron observes. Log disagreements.
    Shadow,
    /// Epsilon-greedy: perceptron decides with probability ε, rule otherwise.
    EpsilonGreedy,
    /// Graduated: perceptron decides, rule observes. Auto-fallback on error spike.
    Graduated,
}

/// 36-dimensional feature vector for perceptron dispatch.
///
/// Layout: `[domain_onehot(32), latency_ewma_norm, error_rate, topology_affinity, gate_load_norm]`
#[derive(Debug, Clone)]
pub struct DispatchFeatures {
    /// Flat feature array: `[domain_onehot(32), latency, error, topology, load]`.
    pub values: [f32; FEATURE_DIM],
}

impl DispatchFeatures {
    /// Build features for a candidate provider.
    ///
    /// `domain_idx`: index into the 32-slot one-hot (from capability domain registry).
    /// `weight`: provider's current EWMA stats from `RoutingWeightTable`.
    /// `gate_load`: normalized gate utilization (0.0–1.0).
    pub fn build(domain_idx: usize, weight: Option<&ProviderWeight>, gate_load: f32) -> Self {
        let mut values = [0.0f32; FEATURE_DIM];

        if domain_idx < 32 {
            values[domain_idx] = 1.0;
        }

        let (latency_norm, error_rate, topo_affinity) = match weight {
            Some(w) => (
                (w.ewma_latency_ms as f32 / 500.0).min(1.0),
                w.ewma_error_rate as f32,
                w.topology_affinity as f32,
            ),
            None => (0.5, 0.0, 1.0),
        };

        values[32] = latency_norm;
        values[33] = error_rate;
        values[34] = topo_affinity;
        values[35] = gate_load;

        Self { values }
    }
}

/// Perceptron weight set: a flat array of `WEIGHT_VEC_LEN` (37) f32 values.
///
/// Loaded from `neural_routing_perceptron.bin` (trained by barraCuda `ml.mlp_train`)
/// or initialized with neutral mock weights for shadow-mode development.
#[derive(Debug, Clone)]
pub struct PerceptronWeights {
    /// 36 feature weights + 1 bias (index 36), little-endian f32.
    pub weights: [f32; WEIGHT_VEC_LEN],
}

impl PerceptronWeights {
    /// Neutral default weights for pre-training shadow mode.
    ///
    /// Produces scores that loosely mirror L4 heuristic priorities (prefer low
    /// latency, penalize errors, reward proximity) so shadow-mode disagreements
    /// are meaningful. Replaced by trained weights from `load_from_file()`.
    #[must_use]
    pub fn neutral_default() -> Self {
        let mut weights = [0.0f32; WEIGHT_VEC_LEN];
        // Equal weight on latency (inverse), error_rate (inverse), topology
        weights[32] = -0.3; // prefer lower latency
        weights[33] = -0.5; // penalize errors
        weights[34] = 0.4;  // reward topology proximity
        weights[35] = -0.1; // slight penalty for high gate load
        Self { weights }
    }

    /// Load weights from a flat f32 binary file (little-endian).
    /// Returns `None` if the file doesn't exist or has wrong size.
    #[must_use]
    pub fn load_from_file(path: &std::path::Path) -> Option<Self> {
        let data = std::fs::read(path).ok()?;
        if data.len() != WEIGHT_VEC_LEN * 4 {
            tracing::warn!(
                "perceptron weights file {} has {} bytes, expected {} — using neutral defaults",
                path.display(),
                data.len(),
                WEIGHT_VEC_LEN * 4
            );
            return None;
        }
        let mut weights = [0.0f32; WEIGHT_VEC_LEN];
        for (i, chunk) in data.chunks_exact(4).enumerate() {
            weights[i] = f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
        }
        tracing::info!(
            "perceptron: loaded {} weights from {}",
            WEIGHT_VEC_LEN,
            path.display()
        );
        Some(Self { weights })
    }

    /// Forward pass: dot(weights[0..36], features) + bias.
    #[must_use]
    pub fn score(&self, features: &DispatchFeatures) -> f32 {
        let mut sum = self.weights[FEATURE_DIM]; // bias term
        for i in 0..FEATURE_DIM {
            sum += self.weights[i] * features.values[i];
        }
        sum
    }
}

/// Perceptron dispatcher: runs shadow inference alongside L4 routing.
pub struct PerceptronDispatcher {
    weights: PerceptronWeights,
    phase: PerceptronPhase,
    /// Total perceptron shadow dispatches.
    pub dispatch_counter: AtomicU64,
    /// Dispatches where perceptron disagreed with L4 rule-based selection.
    pub disagreement_counter: AtomicU64,
}

impl PerceptronDispatcher {
    /// Create a dispatcher with the given weights and phase.
    #[must_use]
    pub fn new(weights: PerceptronWeights, phase: PerceptronPhase) -> Self {
        Self {
            weights,
            phase,
            dispatch_counter: AtomicU64::new(0),
            disagreement_counter: AtomicU64::new(0),
        }
    }

    /// Create a shadow-mode dispatcher with neutral default weights (pre-training).
    #[must_use]
    pub fn shadow_default() -> Self {
        Self::new(PerceptronWeights::neutral_default(), PerceptronPhase::Shadow)
    }

    /// Current operating phase.
    #[must_use]
    pub fn phase(&self) -> PerceptronPhase {
        self.phase
    }

    /// Recommend the best provider index from candidates.
    ///
    /// Scores each candidate's features through the perceptron and returns
    /// the index of the highest-scoring provider.
    ///
    /// In shadow mode, the caller should compare this with the L4 rule-based
    /// choice and log any disagreement — the perceptron recommendation is
    /// **not** used for the actual dispatch decision.
    #[must_use]
    pub fn recommend(
        &self,
        features_per_candidate: &[DispatchFeatures],
    ) -> usize {
        if features_per_candidate.is_empty() {
            return 0;
        }

        let mut best_idx = 0;
        let mut best_score = f32::NEG_INFINITY;
        for (i, features) in features_per_candidate.iter().enumerate() {
            let score = self.weights.score(features);
            if score > best_score {
                best_score = score;
                best_idx = i;
            }
        }
        best_idx
    }

    /// Run shadow comparison: log whether perceptron agrees with rule-based choice.
    ///
    /// `rule_idx`: the index chosen by L4 weighted routing.
    /// `features_per_candidate`: features for each candidate provider.
    /// `capability`: the capability being routed (for logging).
    ///
    /// Returns the perceptron's recommended index (for telemetry, not dispatch).
    pub fn shadow_compare(
        &self,
        rule_idx: usize,
        features_per_candidate: &[DispatchFeatures],
        capability: &str,
    ) -> usize {
        let nn_idx = self.recommend(features_per_candidate);
        let n = self.dispatch_counter.fetch_add(1, Ordering::Relaxed) + 1;

        if nn_idx != rule_idx {
            self.disagreement_counter.fetch_add(1, Ordering::Relaxed);
            if n <= 1000 || n % 1000 == 0 {
                tracing::info!(
                    "L5 perceptron shadow [{n}]: {capability} rule={rule_idx} nn={nn_idx} (disagree)"
                );
            }
        } else if n <= 100 || n % 500 == 0 {
            tracing::debug!(
                "L5 perceptron shadow [{n}]: {capability} rule={nn_idx} (agree)"
            );
        }

        if n == 100 || n == 500 || n == 1000 || (n > 1000 && n % 5000 == 0) {
            let disagreements = self.disagreement_counter.load(Ordering::Relaxed);
            let rate = (disagreements as f64 / n as f64) * 100.0;
            tracing::info!(
                "L5 perceptron milestone [{n}]: {disagreements} disagreements ({rate:.1}% divergence)"
            );
        }

        nn_idx
    }

    /// Shadow statistics: (total dispatches, disagreements).
    #[must_use]
    pub fn shadow_stats(&self) -> (u64, u64) {
        (
            self.dispatch_counter.load(Ordering::Relaxed),
            self.disagreement_counter.load(Ordering::Relaxed),
        )
    }
}

/// Map a capability domain string to a one-hot index (0..31).
///
/// Uses a stable hash of the domain prefix to assign slots. Collisions are
/// acceptable — the perceptron learns to distinguish via numeric features.
#[must_use]
pub fn domain_to_index(capability: &str) -> usize {
    let domain = capability.split('.').next().unwrap_or(capability);
    let mut hash: u32 = 5381;
    for b in domain.bytes() {
        hash = hash.wrapping_mul(33).wrapping_add(u32::from(b));
    }
    (hash as usize) % 32
}

/// Build feature vectors for a set of candidate providers.
///
/// Reads per-provider EWMA stats from the weight table and combines with
/// capability domain and gate load to produce 36-dim feature vectors.
pub fn build_candidate_features(
    capability: &str,
    candidates: &[Arc<str>],
    weight_table: &RoutingWeightTable,
    gate_load: f32,
) -> Vec<DispatchFeatures> {
    let domain_idx = domain_to_index(capability);

    candidates
        .iter()
        .map(|provider| {
            let weight = weight_table.get(capability, provider);
            DispatchFeatures::build(domain_idx, weight, gate_load)
        })
        .collect()
}

#[cfg(test)]
#[path = "perceptron_tests.rs"]
mod tests;
