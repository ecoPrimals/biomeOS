// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::create_test_router;
use biomeos_atomic_deploy::neural_router::RoutingMetrics;
use chrono::Utc;
use std::sync::Arc;

#[tokio::test]
async fn test_routing_metrics_structure() {
    // Test RoutingMetrics can be created
    let metrics = RoutingMetrics {
        request_id: Arc::from("test-123"),
        capability: Arc::from("secure_http"),
        method: Arc::from("http.get"),
        routed_through: vec![Arc::from("songbird"), Arc::from("beardog")],
        latency_ms: 42,
        success: true,
        timestamp: Utc::now(),
        error: None,
    };

    assert_eq!(metrics.capability.as_ref(), "secure_http");
    assert_eq!(metrics.latency_ms, 42);
    assert!(metrics.success);
}

#[tokio::test]
async fn test_log_metric() {
    let router = create_test_router();

    let metric = RoutingMetrics {
        request_id: Arc::from("test-456"),
        capability: Arc::from("storage"),
        method: Arc::from("storage.write"),
        routed_through: vec![Arc::from("toadstool")],
        latency_ms: 100,
        success: true,
        timestamp: Utc::now(),
        error: None,
    };

    // Log the metric
    router.log_metric(metric).await;

    // Verify it was logged
    let metrics = router.get_metrics().await;
    assert_eq!(metrics.len(), 1);
    assert_eq!(metrics[0].request_id.as_ref(), "test-456");
}

#[tokio::test]
async fn test_clear_metrics() {
    let router = create_test_router();

    // Log a metric
    let metric = RoutingMetrics {
        request_id: Arc::from("test-789"),
        capability: Arc::from("compute"),
        method: Arc::from("compute.execute"),
        routed_through: vec![Arc::from("nucleus")],
        latency_ms: 50,
        success: true,
        timestamp: Utc::now(),
        error: None,
    };

    router.log_metric(metric).await;
    assert_eq!(router.get_metrics().await.len(), 1);

    // Clear metrics
    router.clear_metrics().await;
    assert_eq!(router.get_metrics().await.len(), 0);
}

#[tokio::test]
async fn test_routing_concurrent_metrics() {
    use tokio::task;
    let router = std::sync::Arc::new(create_test_router());

    // Log metrics concurrently
    let mut handles = vec![];

    for i in 0..5 {
        let router_clone = router.clone();
        let handle = task::spawn(async move {
            let metric = RoutingMetrics {
                request_id: Arc::from(format!("test-{i}").as_str()),
                capability: Arc::from("test"),
                method: Arc::from("test.method"),
                routed_through: vec![],
                latency_ms: i * 10,
                success: true,
                timestamp: Utc::now(),
                error: None,
            };
            router_clone.log_metric(metric).await;
        });
        handles.push(handle);
    }

    // Wait for all
    for handle in handles {
        handle.await.unwrap();
    }

    // Should have 5 metrics
    let metrics = router.get_metrics().await;
    assert_eq!(metrics.len(), 5);
}

#[tokio::test]
async fn test_metrics_with_errors() {
    let router = create_test_router();

    // Log metric with error
    let metric = RoutingMetrics {
        request_id: Arc::from("error-test"),
        capability: Arc::from("failed_op"),
        method: Arc::from("op.execute"),
        routed_through: vec![],
        latency_ms: 5,
        success: false,
        timestamp: Utc::now(),
        error: Some("Connection refused".to_string()),
    };

    router.log_metric(metric).await;

    let metrics = router.get_metrics().await;
    assert_eq!(metrics.len(), 1);
    assert!(!metrics[0].success);
    assert!(metrics[0].error.is_some());
}
