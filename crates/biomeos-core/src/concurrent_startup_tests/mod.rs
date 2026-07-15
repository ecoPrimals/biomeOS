// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Unit tests for concurrent wave-based primal startup.
//!
//! Extracted from concurrent_startup.rs to keep main module under 1000 LOC.

#![expect(clippy::unwrap_used, reason = "test assertions")]
#![expect(clippy::expect_used, reason = "test assertions")]

mod build_graph;
mod common;
mod start_in_waves;
mod topological_waves;
