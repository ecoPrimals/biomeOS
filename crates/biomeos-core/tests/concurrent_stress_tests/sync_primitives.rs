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
use tokio::sync::Barrier;
use tokio::task::JoinSet;
// Stress Test 3: Barrier Synchronization
// ============================================================================

/// Stress test: 100 tasks synchronized with Barrier
///
/// **Validates**:
/// - Barrier correctly synchronizes many tasks
/// - All tasks proceed together after barrier
/// - No early releases or deadlocks
#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn stress_barrier_synchronization() {
    const NUM_TASKS: usize = 100;

    let barrier = Arc::new(Barrier::new(NUM_TASKS));
    let counter = Arc::new(AtomicU64::new(0));
    let mut join_set = JoinSet::new();

    for task_id in 0..NUM_TASKS {
        let barrier_clone = barrier.clone();
        let counter_clone = counter.clone();

        join_set.spawn(async move {
            // Phase 1: Increment counter before barrier
            counter_clone.fetch_add(1, Ordering::SeqCst);

            // Wait at barrier
            barrier_clone.wait().await;

            // Phase 2: After barrier, counter should be NUM_TASKS
            let count = counter_clone.load(Ordering::SeqCst);
            assert_eq!(
                count, NUM_TASKS as u64,
                "Task {task_id} sees count {count} (expected {NUM_TASKS})"
            );

            task_id
        });
    }

    // Wait for all tasks to complete
    let mut results = Vec::new();
    while let Some(result) = join_set.join_next().await {
        results.push(result.expect("Task should succeed"));
    }

    assert_eq!(results.len(), NUM_TASKS, "All tasks should complete");
}

// ============================================================================
// Stress Test 4: Concurrent Read/Write (Arc + RwLock)
// ============================================================================

/// Stress test: Many concurrent readers and writers
///
/// **Validates**:
/// - RwLock allows concurrent reads
/// - Writes are properly serialized
/// - No data races under load
#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn stress_concurrent_read_write() {
    use tokio::sync::RwLock;

    const NUM_READERS: usize = 80;
    const NUM_WRITERS: usize = 20;
    const OPERATIONS_PER_TASK: usize = 100;

    let shared_data = Arc::new(RwLock::new(0u64));
    let mut join_set = JoinSet::new();

    // Spawn concurrent readers
    for _ in 0..NUM_READERS {
        let data = shared_data.clone();

        join_set.spawn(async move {
            for _ in 0..OPERATIONS_PER_TASK {
                let value = *data.read().await;
                // Value is u64 so always non-negative, check it reads successfully
                let _ = value;
            }
        });
    }

    // Spawn concurrent writers
    for _ in 0..NUM_WRITERS {
        let data = shared_data.clone();

        join_set.spawn(async move {
            for _ in 0..OPERATIONS_PER_TASK {
                let mut value = data.write().await;
                *value += 1;
            }
        });
    }

    // Wait for all tasks to complete
    while join_set.join_next().await.is_some() {}

    // Verify final value
    let final_value = *shared_data.read().await;
    let expected = NUM_WRITERS as u64 * OPERATIONS_PER_TASK as u64;

    assert_eq!(
        final_value, expected,
        "Final value should equal total writes"
    );
}
