// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Async integration tests for GraphExecutor (split from `neural_executor_tests.rs`).
//! Branch-coverage tests live in `neural_executor_async_tests2/`.

#![expect(clippy::unwrap_used, reason = "test assertions")]
#![expect(clippy::expect_used, reason = "test assertions")]

mod common;
mod execution_report;
mod filesystem;
mod logging;
mod node_dispatch;
mod parallelism;
mod rpc_capability;
mod verification_health;

pub(crate) use common::create_test_node;
