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
use biomeos_core::atomic_client::AtomicClient;

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
/// or initialized with neutral default weights for shadow-mode development.
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
    /// Optional Neural API socket for remote `ml.mlp_infer` calls.
    /// When `Some`, `shadow_compare_remote()` sends features to barraCuda
    /// for inference alongside local scoring.
    remote_infer_socket: Option<String>,
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
            remote_infer_socket: None,
            dispatch_counter: AtomicU64::new(0),
            disagreement_counter: AtomicU64::new(0),
        }
    }

    /// Create a shadow-mode dispatcher with neutral default weights (pre-training).
    #[must_use]
    pub fn shadow_default() -> Self {
        Self::new(PerceptronWeights::neutral_default(), PerceptronPhase::Shadow)
    }

    /// Enable remote inference via barraCuda `ml.mlp_infer` capability call.
    ///
    /// When set, `shadow_compare_remote()` will send feature vectors to the ML
    /// pipeline for server-side inference, comparing results with local scoring.
    #[must_use]
    pub fn with_remote_infer(mut self, neural_api_socket: String) -> Self {
        self.remote_infer_socket = Some(neural_api_socket);
        self
    }

    /// Whether remote inference is wired.
    #[must_use]
    pub fn has_remote_infer(&self) -> bool {
        self.remote_infer_socket.is_some()
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

    /// Run shadow comparison with remote `ml.mlp_infer` alongside local scoring.
    ///
    /// Sends the feature matrix to barraCuda via `capability.call("ml.mlp_infer")`
    /// and compares the remote recommendation with both L4 rule-based and local
    /// perceptron choices. Falls back to local-only if the remote call fails.
    ///
    /// Wire format sent to `ml.mlp_infer`:
    /// ```json
    /// { "features": [[f32; 36], ...], "model": "routing_perceptron" }
    /// ```
    ///
    /// Expected response:
    /// ```json
    /// { "scores": [f32, ...], "model_version": "..." }
    /// ```
    pub async fn shadow_compare_remote(
        &self,
        rule_idx: usize,
        features_per_candidate: &[DispatchFeatures],
        capability: &str,
    ) -> usize {
        let local_idx = self.shadow_compare(rule_idx, features_per_candidate, capability);

        let Some(ref socket) = self.remote_infer_socket else {
            return local_idx;
        };

        let feature_matrix: Vec<Vec<f32>> = features_per_candidate
            .iter()
            .map(|f| f.values.to_vec())
            .collect();

        let params = serde_json::json!({
            "capability": "ml",
            "operation": "mlp_infer",
            "args": {
                "features": feature_matrix,
                "model": "routing_perceptron",
            }
        });

        let client = AtomicClient::unix(socket);
        match client.call("capability.call", params).await {
            Ok(result) => {
                if let Some(scores) = result.get("scores").and_then(|s| s.as_array()) {
                    let remote_idx = scores
                        .iter()
                        .enumerate()
                        .max_by(|(_, a), (_, b)| {
                            let sa = a.as_f64().unwrap_or(f64::NEG_INFINITY);
                            let sb = b.as_f64().unwrap_or(f64::NEG_INFINITY);
                            sa.partial_cmp(&sb).unwrap_or(std::cmp::Ordering::Equal)
                        })
                        .map(|(i, _)| i)
                        .unwrap_or(0);

                    if remote_idx != local_idx {
                        let n = self.dispatch_counter.load(Ordering::Relaxed);
                        if n <= 100 || n % 500 == 0 {
                            tracing::info!(
                                "L5 remote shadow [{n}]: {capability} local={local_idx} remote={remote_idx} (diverge)"
                            );
                        }
                    }
                    remote_idx
                } else {
                    tracing::debug!("ml.mlp_infer: no scores in response for {capability}");
                    local_idx
                }
            }
            Err(e) => {
                tracing::debug!("ml.mlp_infer shadow call failed for {capability}: {e}");
                local_idx
            }
        }
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

/// A single training row for perceptron offline training.
///
/// Emitted after each multi-provider dispatch with outcome data. Consumed by
/// barraCuda `ml.mlp_train` to produce `neural_routing_perceptron.bin`.
#[derive(Debug, Clone, serde::Serialize)]
pub struct DispatchTrainingRow {
    /// Capability domain key (e.g. `"crypto"`, `"storage"`).
    pub capability: String,
    /// Primal names that were candidates for this dispatch.
    pub candidates: Vec<String>,
    /// 36-dim feature vectors per candidate (pre-dispatch EWMA stats).
    pub features: Vec<Vec<f32>>,
    /// Index into `candidates` that L4 selected.
    pub chosen_idx: usize,
    /// Whether the dispatch succeeded.
    pub success: bool,
    /// Wall-clock latency of the dispatch in milliseconds.
    pub latency_ms: u64,
    /// L4 score of the chosen provider at selection time.
    pub l4_score: f64,
    /// Unix timestamp (seconds) of the outcome.
    pub timestamp: i64,
}

/// Pending dispatch context: features captured at selection time, waiting for
/// outcome data from `record_dispatch_outcome`.
#[derive(Debug, Clone)]
pub(crate) struct PendingDispatch {
    pub capability: String,
    pub candidates: Vec<String>,
    pub features: Vec<Vec<f32>>,
    pub chosen_idx: usize,
    pub l4_score: f64,
    pub created_at: std::time::Instant,
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
