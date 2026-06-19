// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Routing tests for Neural API Server (extracted from routing.rs).

#[path = "routing_tests_dispatch.rs"]
mod dispatch;

#[path = "routing_tests_routes.rs"]
mod routes;

#[path = "routing_tests_semantic.rs"]
mod semantic;

#[path = "routing_tests_coverage.rs"]
mod coverage;

mod common {
    use crate::neural_api_server::NeuralApiServer;

    pub(super) fn create_test_server() -> (NeuralApiServer, tempfile::TempDir) {
        let temp = tempfile::tempdir().expect("temp dir");
        std::fs::create_dir_all(temp.path()).expect("create graphs dir");
        let server =
            NeuralApiServer::new(temp.path(), "test_family", temp.path().join("neural.sock"));
        // Prevent lazy socket rescan from finding real primals running on this host.
        server
            .router
            .lazy_rescan_attempted
            .store(true, std::sync::atomic::Ordering::Relaxed);
        (server, temp)
    }
}
