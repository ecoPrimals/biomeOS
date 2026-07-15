// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project
//
// Sibling tests for `lifecycle.rs` (handler API surface).

#![expect(clippy::unwrap_used, reason = "test")]
#![expect(clippy::expect_used, reason = "test")]

mod common;
mod composition;
mod composition_health;
mod handler_basics;
mod lifecycle_actions;
mod register_get;
mod state_details;
