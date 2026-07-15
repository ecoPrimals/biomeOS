// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! BTSP coordinator tests split by domain:
//! - [`health`] — overall status, degradation classification, TCP fallback parsing
//! - [`tunnel`] — `create_tunnel` integration with mock providers
//! - [`monitor_recover`] — `monitor_tunnel` and `recover_tunnel` paths
#![expect(clippy::expect_used, reason = "test assertions")]

mod common;
mod health;
mod monitor_recover;
mod tunnel;
