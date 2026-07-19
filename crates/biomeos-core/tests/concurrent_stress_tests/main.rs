// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

#![expect(clippy::expect_used, reason = "test assertions")]

//! Concurrent stress tests for biomeOS
//!
//! **Purpose**: Validate that our concurrent evolution is production-ready
//! - Truly concurrent execution (no serial patterns)
//! - Stress test synchronization primitives
//! - Validate zero race conditions
//! - Ensure deterministic behavior under load
//!
//! **Philosophy**: Test issues = Production issues
//! If tests can't handle concurrency, production won't either!

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use tokio::sync::{Barrier, mpsc, oneshot};
use tokio::task::JoinSet;
mod benchmark;
mod channel_mixed;
mod startup_channels;
mod sync_primitives;
mod task_spawn;
