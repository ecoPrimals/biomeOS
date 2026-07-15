// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! SystemInspector and SystemMonitor tests (split from `tests.rs`).

#![expect(
    clippy::unwrap_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
#![expect(
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]

mod component_health;
mod edge_cases;
mod health_determination;
mod inspector_api;
mod inspector_helpers;
mod monitor;
mod serialization;
mod uptime;
