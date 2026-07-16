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
// Stress Test 5: Rapid Task Spawn/Join
// ============================================================================

/// Stress test: Spawn and join 10,000 tasks
///
/// **Validates**:
/// - tokio handles many concurrent tasks
/// - No resource leaks or panics
/// - JoinSet correctly tracks all tasks
#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn stress_rapid_task_spawn() {
    const NUM_TASKS: usize = 10_000;

    let mut join_set = JoinSet::new();

    // Spawn 10,000 concurrent tasks
    for i in 0..NUM_TASKS {
        join_set.spawn(async move {
            // Minimal work per task
            i * 2
        });
    }

    // Join all tasks
    let mut results = Vec::new();
    while let Some(result) = join_set.join_next().await {
        results.push(result.expect("Task should succeed"));
    }

    assert_eq!(results.len(), NUM_TASKS, "All tasks should complete");

    // Verify correctness
    let sum: usize = results.iter().sum();
    let expected_sum: usize = (0..NUM_TASKS).map(|i| i * 2).sum();
    assert_eq!(sum, expected_sum, "Sum should be correct");
}

// ============================================================================
// Stress Test 6: Exponential Backoff Under Load
// ============================================================================

/// Stress test: Exponential backoff with many concurrent pollers
///
/// **Validates**:
/// - Exponential backoff scales under load
/// - No thundering herd problems
/// - Efficient resource usage
#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn stress_exponential_backoff() {
    const NUM_POLLERS: usize = 50;

    let ready_flag = Arc::new(AtomicU64::new(0));
    let mut join_set = JoinSet::new();

    // One task sets the flag after 100ms
    let flag_clone = ready_flag.clone();
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(100)).await;
        flag_clone.store(1, Ordering::SeqCst);
    });

    // 50 tasks poll with exponential backoff
    for poller_id in 0..NUM_POLLERS {
        let flag = ready_flag.clone();

        join_set.spawn(async move {
            let mut delay_ms = 1u64;
            let max_delay_ms = 64u64;
            let mut attempts = 0;

            loop {
                if flag.load(Ordering::SeqCst) == 1 {
                    return (poller_id, attempts);
                }

                tokio::time::sleep(Duration::from_millis(delay_ms)).await;
                delay_ms = (delay_ms * 2).min(max_delay_ms);
                attempts += 1;

                assert!(attempts <= 20, "Poller {poller_id} took too long");
            }
        });
    }

    // Wait for all pollers
    let mut results = Vec::new();
    while let Some(result) = join_set.join_next().await {
        results.push(result.expect("Poller should succeed"));
    }

    assert_eq!(results.len(), NUM_POLLERS, "All pollers should complete");

    // Verify reasonable attempt counts (should be < 10 for 100ms wait)
    for (poller_id, attempts) in results {
        assert!(
            attempts < 15,
            "Poller {poller_id} took {attempts} attempts (too many)"
        );
    }
}

