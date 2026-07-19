// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

// Sibling tests for engine.rs

#![expect(clippy::unwrap_used, reason = "test")]
#![expect(clippy::expect_used, reason = "test")]

mod auto_escalate;
mod cooldown;
mod escalation_actions;
mod lifecycle;
mod status_metrics;
