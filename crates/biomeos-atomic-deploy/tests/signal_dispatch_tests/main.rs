// SPDX-License-Identifier: AGPL-3.0-or-later
//
//! Signal dispatch integration tests.
//!
//! Validates that the composition collapse layer correctly maps atomic
//! signals to graph paths, loads all 27 signal graphs, and intercepts
//! signal-tier capability calls.

#![expect(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

use std::path::PathBuf;

mod core_dispatch;
mod nest_graphs;
mod tower_live_validation;

fn graphs_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../graphs")
}
