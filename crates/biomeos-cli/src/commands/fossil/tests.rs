// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Fossil command tests — split by domain for maintainability.
//!
//! - `format_tests`: pure formatting, filtering, cleanup plan, and scan logic
//! - `integration_tests`: CLI command execution with temp dirs and env guards

#![expect(clippy::unwrap_used, reason = "test assertions use unwrap for clarity")]
#![expect(clippy::expect_used, reason = "test assertions use expect for clarity")]

use biomeos_spore::logs::{
    ActiveLogSession, ArchivalReason, FossilIndex, FossilIndexEntry, LogFile,
};
use biomeos_test_utils::env_helpers::TestEnvGuard;
use chrono::Utc;
use serial_test::serial;
use std::path::{Path, PathBuf};

use super::format::{
    compute_cleanup_plan, filter_sessions, format_fossil_detail, format_session_display,
    scan_old_logs,
};
use super::{CleanupPlan, FossilAction, FossilArgs, run};

#[path = "tests/format_tests.rs"]
mod format_tests;

#[path = "tests/integration_tests.rs"]
mod integration_tests;
