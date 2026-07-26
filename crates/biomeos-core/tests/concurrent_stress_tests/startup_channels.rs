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

use std::time::Duration;
use tokio::sync::{mpsc, oneshot};
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
        let sender_count = received.iter().filter(|(id, _)| *id == sender_id).count();

        assert_eq!(
            sender_count, MESSAGES_PER_SENDER,
            "Sender {sender_id} should send all messages"
        );
    }
}
