// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Additional async integration tests for GraphExecutor (branch coverage — split from `neural_executor_async_tests.rs`).

#![expect(clippy::expect_used, reason = "test assertions use expect for clarity")]

mod node_operations;
mod rpc_capability;
mod verification_health;
