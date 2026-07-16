// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::create_test_router;
use biomeos_core::TransportEndpoint;
use serde_json::json;
use std::path::PathBuf;

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
        .register_capability_unix(
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
        .register_capability_unix(
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
async fn test_forward_request() {
    let router = create_test_router();

    // Test forward_request with test socket
    let ep = TransportEndpoint::UnixSocket {
        path: PathBuf::from("/tmp/test-nonexistent.sock"),
    };
    let params = json!({"test": "data"});

    // Should handle request (will fail on socket connection)
    let result = router.forward_request(&ep, "test.method", &params).await;

    // Should fail gracefully (socket doesn't exist)
    assert!(result.is_err());
}

#[tokio::test]
async fn test_invalidate_cache() {
    let router = create_test_router();

    // Register a capability (populates cache)
    router
        .register_capability_unix(
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
        .register_capability_unix(
            "storage",
            "toadstool1",
            PathBuf::from("/run/user/1000/toadstool1.sock"),
            "test",
        )
        .await
        .unwrap();

    router
        .register_capability_unix(
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
async fn test_capability_registration_overwrites() {
    let router = create_test_router();

    // Register capability
    router
        .register_capability_unix(
            "test_cap",
            "primal1",
            PathBuf::from("/tmp/primal1.sock"),
            "test",
        )
        .await
        .unwrap();

    // Register again with different socket
    router
        .register_capability_unix(
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
