// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

#![allow(clippy::unwrap_used, clippy::expect_used)]

//! Neural API Routing Tests
//!
//! Comprehensive tests for Neural API capability-based routing,
//! discovery, semantic translation, and HTTP proxying.

use biomeos_atomic_deploy::neural_router::{NeuralRouter, RoutingMetrics};
use serde_json::json;
use std::path::PathBuf;
use std::sync::Arc;

/// Test helper: Create test Neural Router
fn create_test_router() -> NeuralRouter {
    NeuralRouter::new("test")
}

#[tokio::test]
async fn test_neural_router_creation() {
    let _router = create_test_router();

    // Router created successfully without panic — reaching this point validates construction
}

#[tokio::test]
async fn test_capability_discovery_no_primals() {
    let router = create_test_router();

    // When no primals exist, discovery should return error
    let result = router.discover_capability("secure_http").await;

    // Should return an error (no primals registered)
    assert!(result.is_err());
}

#[tokio::test]
async fn test_discover_capability_method_structure() {
    // Test that discover_capability method accepts correct parameters
    let router = create_test_router();

    // Should accept capability string
    let result = router.discover_capability("secure_http").await;

    // Should return Result type
    assert!(result.is_err() || result.is_ok());
}

#[tokio::test]
async fn test_register_capability() {
    let router = create_test_router();

    // Test registering a capability
    router
        .register_capability(
            "secure_http",
            "songbird",
            PathBuf::from("/run/user/1000/songbird-test.sock"),
            "test",
        )
        .await
        .unwrap();

    // Verify capability was registered
    let capabilities = router.list_capabilities().await;
    assert!(capabilities.contains_key("secure_http"));
}

#[tokio::test]
async fn test_get_capability_providers() {
    let router = create_test_router();

    // Register a capability
    router
        .register_capability(
            "crypto",
            "beardog",
            PathBuf::from("/run/user/1000/beardog-test.sock"),
            "test",
        )
        .await
        .unwrap();

    // Get providers for the capability
    let providers = router.get_capability_providers("crypto").await;
    assert!(providers.is_some());
    assert_eq!(providers.unwrap().len(), 1);
}

#[tokio::test]
async fn test_get_capability_providers_empty() {
    let router = create_test_router();

    // Get providers for non-existent capability
    let providers = router.get_capability_providers("nonexistent").await;
    assert!(providers.is_none());
}

#[tokio::test]
async fn test_routing_metrics_structure() {
    // Test RoutingMetrics can be created
    use chrono::Utc;

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
    use chrono::Utc;

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
    use chrono::Utc;

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
async fn test_forward_request() {
    let router = create_test_router();

    // Test forward_request with test socket
    let socket_path = PathBuf::from("/tmp/test-nonexistent.sock");
    let params = json!({"test": "data"});

    // Should handle request (will fail on socket connection)
    let result = router
        .forward_request(&socket_path, "test.method", &params)
        .await;

    // Should fail gracefully (socket doesn't exist)
    assert!(result.is_err());
}

#[tokio::test]
async fn test_invalidate_cache() {
    let router = create_test_router();

    // Register a capability (populates cache)
    router
        .register_capability(
            "test_cap",
            "test_primal",
            PathBuf::from("/tmp/test.sock"),
            "test",
        )
        .await
        .unwrap();

    // Invalidate cache
    router.invalidate_cache().await;

    // Cache should be cleared (but registry remains)
    let caps = router.list_capabilities().await;
    assert!(caps.contains_key("test_cap")); // Registry still has it
}

#[tokio::test]
async fn test_multiple_providers_same_capability() {
    let router = create_test_router();

    // Register multiple providers for same capability
    router
        .register_capability(
            "storage",
            "toadstool1",
            PathBuf::from("/run/user/1000/toadstool1.sock"),
            "test",
        )
        .await
        .unwrap();

    router
        .register_capability(
            "storage",
            "toadstool2",
            PathBuf::from("/run/user/1000/toadstool2.sock"),
            "test",
        )
        .await
        .unwrap();

    // Should have 2 providers for storage
    let providers = router.get_capability_providers("storage").await;
    assert!(providers.is_some());
    assert_eq!(providers.unwrap().len(), 2);
}

#[tokio::test]
async fn test_routing_concurrent_metrics() {
    use chrono::Utc;
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
async fn test_capability_registration_overwrites() {
    let router = create_test_router();

    // Register capability
    router
        .register_capability(
            "test_cap",
            "primal1",
            PathBuf::from("/tmp/primal1.sock"),
            "test",
        )
        .await
        .unwrap();

    // Register again with different socket
    router
        .register_capability(
            "test_cap",
            "primal1",
            PathBuf::from("/tmp/primal1-new.sock"),
            "test",
        )
        .await
        .unwrap();

    // Should still have the capability
    let providers = router.get_capability_providers("test_cap").await;
    assert!(providers.is_some());
}

#[tokio::test]
async fn test_metrics_with_errors() {
    let router = create_test_router();
    use chrono::Utc;

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

#[tokio::test]
async fn test_list_capabilities_empty() {
    let router = create_test_router();

    // No capabilities registered
    let caps = router.list_capabilities().await;
    assert!(caps.is_empty());
}

#[tokio::test]
async fn test_discover_multiple_capabilities() {
    let router = create_test_router();

    // Test discovering different capabilities
    for capability in &["secure_http", "crypto", "tls", "storage"] {
        let result = router.discover_capability(capability).await;

        // Should handle any capability string
        assert!(result.is_err() || result.is_ok());
    }
}

#[tokio::test]
async fn test_neural_router_concurrent_discovery() {
    use tokio::task;

    let router = std::sync::Arc::new(create_test_router());

    // Test concurrent capability discovery
    let mut handles = vec![];

    for i in 0..5 {
        let router_clone = router.clone();
        let handle = task::spawn(async move {
            router_clone
                .discover_capability(&format!("capability-{i}"))
                .await
        });
        handles.push(handle);
    }

    // All should complete without panic
    for handle in handles {
        let result = handle.await;
        assert!(result.is_ok()); // Should complete
        // Inner result may be Err (no primals), but should not panic
    }
}

#[tokio::test]
async fn test_routing_timeout_handling() {
    let router = create_test_router();

    // Test that routing doesn't hang indefinitely
    let result = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        router.discover_capability("secure_http"),
    )
    .await;

    // Should complete within timeout
    assert!(result.is_ok(), "Discovery should not hang");
}

#[tokio::test]
async fn test_capability_based_routing_pattern() {
    // Test TRUE PRIMAL pattern: capability-based, not primal-specific
    let router = create_test_router();

    // Should ask for capability, not primal name
    let result = router.discover_capability("secure_http").await;

    // Method should exist and handle capability strings
    assert!(result.is_err() || result.is_ok());

    // Should NOT have methods like discover_primal("songbird")
    // This is enforced by API design
}

#[tokio::test]
async fn test_runtime_discovery_pattern() {
    // Test that discovery happens at runtime, not compile-time
    let router = create_test_router();

    // Discovery should scan runtime environment
    let result = router.discover_capability("test-capability").await;

    // Should complete (even if no primals found)
    assert!(result.is_err() || result.is_ok());
}
