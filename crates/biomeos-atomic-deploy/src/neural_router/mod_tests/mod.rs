// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Unit tests for [`super::NeuralRouter`] core API in `mod.rs`:
//! provider selection, fallback routing, dispatch outcomes, and configuration.

#![expect(clippy::unwrap_used, reason = "test assertions")]

use super::NeuralRouter;
use biomeos_core::TransportEndpoint;
use tempfile::tempdir;

pub(super) fn unix_ep(path: &std::path::Path) -> TransportEndpoint {
    TransportEndpoint::UnixSocket {
        path: path.to_path_buf(),
    }
}

pub(super) async fn register_crypto_providers(router: &NeuralRouter, slow: &str, fast: &str) {
    let dir = tempdir().expect("tempdir");
    let slow_sock = dir.path().join(format!("{slow}.sock"));
    let fast_sock = dir.path().join(format!("{fast}.sock"));
    router
        .register_capability_unix("crypto", slow, &slow_sock, "test")
        .await
        .expect("register slow");
    router
        .register_capability_unix("crypto", fast, &fast_sock, "test")
        .await
        .expect("register fast");
    std::mem::forget(dir);
}

mod configuration;
mod dispatch_outcomes;
mod fallback_routing;
mod provider_selection;
mod utilization_metrics;
