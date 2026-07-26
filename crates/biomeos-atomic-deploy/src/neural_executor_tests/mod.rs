// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Tests for GraphExecutor (split from `neural_executor_tests.rs`).
//!
//! Domain-focused modules:
//! - [`split_capability`] — capability string parsing
//! - [`env_substitution`] — `${VAR}` template substitution
//! - [`topological_sort`] — dependency ordering and cycle detection
//! - [`graph_config`] — executor construction and graph config defaults
//! - [`execution`] — end-to-end graph execution paths

#![expect(clippy::unwrap_used, reason = "test assertions")]
#![expect(clippy::expect_used, reason = "test assertions")]

mod common;
mod env_substitution;
mod execution;
mod graph_config;
mod split_capability;
mod topological_sort;
