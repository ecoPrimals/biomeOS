// SPDX-License-Identifier: AGPL-3.0-or-later
//
//! Composition broker E2E tests.
//!
//! Validates:
//! 1. Nest signal graph topology for BTSP-required multi-primal pipelines
//! 2. BTSP family-scoped socket detection for composition broker routing
//! 3. riboCipher framing in IPC payloads
//! 4. Provenance Trio completeness in nest.ingest_spore and nest.ingest_dataset

#![expect(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

use std::path::PathBuf;

mod btsp_routing;
mod nest_topology;
mod ribocipher;
mod schema_validation;

fn graphs_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../graphs")
}
