// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

#![expect(clippy::unwrap_used, reason = "test assertions")]
#![expect(clippy::expect_used, reason = "test assertions")]

//! Neural API Routing Tests
//!
//! Comprehensive tests for Neural API capability-based routing,
//! discovery, semantic translation, and HTTP proxying.

mod discovery_registration;
mod metrics;
mod route_register;

use biomeos_atomic_deploy::neural_router::NeuralRouter;

/// Test helper: Create test Neural Router
fn create_test_router() -> NeuralRouter {
    NeuralRouter::new("test")
}
