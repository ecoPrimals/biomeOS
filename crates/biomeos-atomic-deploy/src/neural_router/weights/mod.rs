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
//!
//! # Module structure
//!
//! - `scoring` — `ProviderWeight` struct, EWMA, circuit breaker, score function
//! - `store` — `RoutingWeightTable`, redb persistence, provider selection
//! - `utilization` — `CapabilityUtilizationTracker`, method-level call frequency

mod scoring;
mod store;
mod utilization;

pub use scoring::{ProviderWeight, topology_affinity_for_endpoint};
pub use store::{RoutingWeightTable, WeightTableSummary};
pub use utilization::{CapabilityUtilizationTracker, MethodUtilization, UtilizationSummary};

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
