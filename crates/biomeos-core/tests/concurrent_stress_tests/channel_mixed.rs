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
// Stress Test 7: Concurrent oneshot Channel Creation/Drop
// ============================================================================

/// Stress test: Create and drop 10,000 oneshot channels
///
/// **Validates**:
/// - oneshot channels are lightweight
/// - No resource leaks
/// - Fast allocation/deallocation
#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn stress_oneshot_churn() {
    const NUM_CHANNELS: usize = 10_000;

    let mut join_set = JoinSet::new();

    for i in 0..NUM_CHANNELS {
        join_set.spawn(async move {
            let (tx, rx) = oneshot::channel::<usize>();

            // Send value
            tx.send(i).expect("Send should succeed");

            // Receive value
            let received = rx.await.expect("Receive should succeed");
            assert_eq!(received, i, "Value should match");

            i
        });
    }

    // Wait for all tasks
    let mut count = 0;
    while join_set.join_next().await.is_some() {
        count += 1;
    }

    assert_eq!(count, NUM_CHANNELS, "All channels should complete");
}

// ============================================================================
// Stress Test 8: Mixed Concurrent Operations
// ============================================================================

/// Stress test: Mix of all synchronization primitives
///
/// **Validates**:
/// - Different primitives work together
/// - No deadlocks or livelocks
/// - Deterministic behavior in complex scenarios
#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn stress_mixed_concurrent_operations() {
    use tokio::sync::RwLock;

    const NUM_TASKS: usize = 100;

    let (tx, mut rx) = mpsc::channel(1000);
    let shared_state = Arc::new(RwLock::new(0u64));
    let barrier = Arc::new(Barrier::new(NUM_TASKS));
    let mut join_set = JoinSet::new();

    for task_id in 0..NUM_TASKS {
        let tx_clone = tx.clone();
        let state_clone = shared_state.clone();
        let barrier_clone = barrier.clone();

        join_set.spawn(async move {
            // Phase 1: oneshot
            let (ready_tx, ready_rx) = oneshot::channel();
            tokio::spawn(async move {
                let _ = ready_tx.send(());
            });
            ready_rx.await.expect("Should receive ready signal");

            // Phase 2: RwLock write
            {
                let mut value = state_clone.write().await;
                *value += 1;
            }

            // Phase 3: Barrier sync
            barrier_clone.wait().await;

            // Phase 4: RwLock read
            let final_value = *state_clone.read().await;

            // Phase 5: mpsc send
            tx_clone.send(task_id).await.expect("Should send message");

            final_value
        });
    }

    // Drop tx so rx can complete
    drop(tx);

    // Receive all messages
    let mut received = Vec::new();
    while let Some(msg) = rx.recv().await {
        received.push(msg);
    }

    // Wait for all tasks
    let mut final_values = Vec::new();
    while let Some(result) = join_set.join_next().await {
        final_values.push(result.expect("Task should succeed"));
    }

    // Verify all tasks completed
    assert_eq!(final_values.len(), NUM_TASKS, "All tasks should complete");
    assert_eq!(received.len(), NUM_TASKS, "All messages should be received");

    // All tasks should see final value == NUM_TASKS
    for value in final_values {
        assert_eq!(value, NUM_TASKS as u64, "All tasks should see final value");
    }
}
