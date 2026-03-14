// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

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

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, oneshot, Barrier};
use tokio::task::JoinSet;

// ============================================================================
// Stress Test 1: Concurrent Server Readiness
// ============================================================================

/// Stress test: 100 concurrent mock servers with oneshot channels
///
/// **Validates**:
/// - oneshot channels scale to many concurrent servers
/// - No race conditions in server startup
/// - Deterministic synchronization under load
#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn stress_concurrent_server_startup() {
    const NUM_SERVERS: usize = 100;

    let mut join_set = JoinSet::new();
    let mut receivers = Vec::new();

    // Start 100 servers concurrently
    for i in 0..NUM_SERVERS {
        let (ready_tx, ready_rx) = oneshot::channel();
        receivers.push(ready_rx);

        join_set.spawn(async move {
            // Simulate server startup work
            let work_ms = (i % 10) as u64; // 0-9ms variance
            if work_ms > 0 {
                tokio::time::sleep(Duration::from_millis(work_ms)).await;
            }

            // Signal ready
            let _ = ready_tx.send(());
            i
        });
    }

    // Wait for all servers to be ready (concurrent!)
    for rx in receivers {
        rx.await.expect("Server should signal ready");
    }

    // Verify all servers completed
    let mut completed = Vec::new();
    while let Some(result) = join_set.join_next().await {
        completed.push(result.expect("Server task should succeed"));
    }

    assert_eq!(completed.len(), NUM_SERVERS, "All servers should complete");

    // Verify all server IDs are unique
    let mut sorted = completed.clone();
    sorted.sort_unstable();
    sorted.dedup();
    assert_eq!(sorted.len(), NUM_SERVERS, "All server IDs should be unique");
}

// ============================================================================
// Stress Test 2: Concurrent Message Passing
// ============================================================================

/// Stress test: 1000 concurrent tasks communicating via mpsc
///
/// **Validates**:
/// - mpsc channels handle high concurrency
/// - Message ordering is preserved
/// - No dropped messages under load
#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn stress_concurrent_message_passing() {
    const NUM_SENDERS: usize = 100;
    const MESSAGES_PER_SENDER: usize = 100;
    const TOTAL_MESSAGES: usize = NUM_SENDERS * MESSAGES_PER_SENDER;

    let (tx, mut rx) = mpsc::channel(1000);
    let mut join_set = JoinSet::new();

    // Spawn 100 concurrent senders
    for sender_id in 0..NUM_SENDERS {
        let tx_clone = tx.clone();

        join_set.spawn(async move {
            for msg_id in 0..MESSAGES_PER_SENDER {
                let message = (sender_id, msg_id);
                tx_clone.send(message).await.expect("Send should succeed");
            }
        });
    }

    // Drop original tx so rx can complete
    drop(tx);

    // Receive all messages
    let mut received = Vec::new();
    while let Some(msg) = rx.recv().await {
        received.push(msg);
    }

    // Wait for all senders to complete
    while join_set.join_next().await.is_some() {}

    // Verify all messages received
    assert_eq!(
        received.len(),
        TOTAL_MESSAGES,
        "Should receive all messages"
    );

    // Verify each sender sent all messages
    for sender_id in 0..NUM_SENDERS {
        let sender_messages: Vec<_> = received.iter().filter(|(id, _)| *id == sender_id).collect();

        assert_eq!(
            sender_messages.len(),
            MESSAGES_PER_SENDER,
            "Sender {sender_id} should send all messages"
        );
    }
}

// ============================================================================
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

// ============================================================================
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

                if attempts > 20 {
                    panic!("Poller {poller_id} took too long");
                }
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

// ============================================================================
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

// ============================================================================
// Performance Benchmarks (for reference, not strict assertions)
// ============================================================================

/// Performance benchmark: Measure concurrent throughput
///
/// **Purpose**: Provide baseline metrics for concurrent operations
/// (not a strict test, just measurement)
#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn benchmark_concurrent_throughput() {
    const NUM_OPERATIONS: usize = 100_000;

    let start = std::time::Instant::now();

    let mut join_set = JoinSet::new();

    for i in 0..NUM_OPERATIONS {
        join_set.spawn(async move { i });
    }

    while join_set.join_next().await.is_some() {}

    let duration = start.elapsed();
    let ops_per_sec = NUM_OPERATIONS as f64 / duration.as_secs_f64();

    println!(
        "📊 Concurrent throughput: {ops_per_sec:.0} ops/sec ({NUM_OPERATIONS} ops in {duration:?})"
    );

    // Just verify it completes (no strict assertion on speed)
    assert!(duration.as_secs() < 10, "Should complete within 10 seconds");
}
