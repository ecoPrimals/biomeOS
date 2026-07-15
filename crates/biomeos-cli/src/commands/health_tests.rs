// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Health command tests — split by domain for maintainability.
//!
//! - `utils_tests`: status icons, memory percent, byte formatting
//! - `format_health_summary_tests`: health summary formatting and display
//! - `format_probe_tests`: deep probe result formatting
//! - `format_scan_tests`: system scan result formatting
//! - `display_tests`: status and scan display output paths
//! - `handle_health_tests`: async health command handler integration

#![expect(
    clippy::unwrap_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
#![expect(
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]

#[path = "health_tests/utils_tests.rs"]
mod utils_tests;

#[path = "health_tests/format_health_summary_tests.rs"]
mod format_health_summary_tests;

#[path = "health_tests/format_probe_tests.rs"]
mod format_probe_tests;

#[path = "health_tests/format_scan_tests.rs"]
mod format_scan_tests;

#[path = "health_tests/display_tests.rs"]
mod display_tests;

#[path = "health_tests/handle_health_tests.rs"]
mod handle_health_tests;
