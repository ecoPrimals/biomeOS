// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

#[tokio::test]
async fn test_handle_dashboard_deprecated_message() {
    let result = super::super::handlers::handle_dashboard(5, false).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_handle_monitor_duration_zero_single_iteration() {
    let result = super::super::handle_monitor(None, 0, Some(0)).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_handle_scale_requires_replicas_or_auto() {
    let result = super::super::handle_scale("any-service".into(), None, false).await;
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("Must specify") || (msg.contains("replicas") && msg.contains("auto")),
        "unexpected: {msg}"
    );
}
