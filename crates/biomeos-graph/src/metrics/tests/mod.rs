// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Metrics tests split by domain:
//! - [`serialization`] — type serde, defaults, and `prefix_end` helper
//! - [`graph_collector`] — graph-level collection, aggregation, and queries
//! - [`node_metrics`] — per-node execution recording and aggregation
#![expect(
    clippy::unwrap_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
#![expect(
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]

mod graph_collector;
mod node_metrics;
mod serialization;
