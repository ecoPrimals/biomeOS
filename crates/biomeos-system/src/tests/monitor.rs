// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use crate::SystemMonitor;
use biomeos_types::HealthSubjectType;
use std::sync::atomic::{AtomicUsize, Ordering};

#[test]
fn test_system_monitor_new() {
    let interval = std::time::Duration::from_secs(30);
    let _monitor = SystemMonitor::new(interval);
    // Verify constructor succeeds; interval is used by start_monitoring
}

#[tokio::test(start_paused = true)]
#[ignore = "Slow: get_system_health takes ~1.2s real time; run with --ignored for full coverage"]
async fn test_system_monitor_start_monitoring_receives_reports() {
    let monitor = SystemMonitor::new(std::time::Duration::from_millis(100));
    let report_count = std::sync::Arc::new(AtomicUsize::new(0));
    let count_for_spawn = report_count.clone();
    let notify = std::sync::Arc::new(tokio::sync::Notify::new());
    let notify_for_wait = notify.clone();

    let monitor_handle = tokio::spawn(async move {
        let count = count_for_spawn;
        let notify_clone = notify.clone();
        monitor
            .start_monitoring(move |report| {
                count.fetch_add(1, Ordering::SeqCst);
                notify_clone.notify_one();
                assert_eq!(report.subject.subject_type, HealthSubjectType::System);
            })
            .await
    });

    // Advance time so interval ticks; first get_system_health runs (real time ~1.2s)
    tokio::time::advance(std::time::Duration::from_secs(3)).await;
    // Wait for first report (callback runs after get_system_health completes)
    tokio::time::timeout(
        std::time::Duration::from_secs(3),
        notify_for_wait.notified(),
    )
    .await
    .expect("timeout waiting for report");
    monitor_handle.abort();

    let received = report_count.load(Ordering::SeqCst);
    assert!(
        received >= 1,
        "should receive at least 1 report within 3s, got {received}"
    );
}

#[tokio::test(start_paused = true)]
async fn test_system_monitor_start_monitoring_spawns_and_aborts() {
    let monitor = SystemMonitor::new(std::time::Duration::from_secs(60));
    let monitor_handle = tokio::spawn(async move {
        monitor
            .start_monitoring(|report| {
                assert_eq!(report.subject.subject_type, HealthSubjectType::System);
            })
            .await
    });

    tokio::time::advance(std::time::Duration::from_millis(50)).await;
    monitor_handle.abort();
    let _ = monitor_handle.await;
    // Verify we can spawn and abort without panicking
}
