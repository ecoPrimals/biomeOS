// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Plasmodium unit tests (split from `tests.rs`).

#![expect(clippy::unwrap_used, reason = "test assertions")]
#![expect(clippy::expect_used, reason = "test assertions")]

mod aggregate_capabilities;
mod aggregate_gates;
mod collective;
mod construction;
mod system;
mod types;
