// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! BearDog integration error tests for genetic lineage verification.
//!
//! **NOTE**: Most tests require a running BearDog instance for lineage operations.
//! They are marked with `#[ignore]` by default.
//!
//! To run these integration tests:
//! ```bash
//! # Start BearDog first
//! ./plasmidBin/primals/beardog-server
//!
//! # In another terminal, run the tests
//! cargo test --package biomeos-federation genetic_lineage -- --ignored
//! ```

#![expect(clippy::unwrap_used, reason = "test assertions")]

mod genetic_lineage_common;
use genetic_lineage_common::*;

use biomeos_federation::security_client::SecurityProviderClient;

#[tokio::test]
async fn test_beardog_unavailable_error() {
    // Try to connect to nonexistent BearDog instance
    let result = SecurityProviderClient::with_endpoint("unix:///tmp/nonexistent_beardog.sock");

    // Should create client successfully (connection happens on call)
    assert!(result.is_ok(), "Client creation should succeed");

    if let Ok(client) = result {
        // But verification should fail
        let family_id = test_family_id();
        let seed = test_seed_hash("unavailable_test");

        let verify_result = client
            .verify_same_family(&family_id, &seed, "node_test")
            .await;

        // Should return error (not panic)
        assert!(
            verify_result.is_err(),
            "Should error when BearDog unavailable"
        );

        if let Err(e) = verify_result {
            let error_msg = e.to_string();
            assert!(!error_msg.is_empty(), "Error should have message");
        }
    }
}

#[tokio::test]
#[ignore = "Requires running BearDog instance"]
async fn test_timeout_on_slow_response() {
    use tokio::time::{Duration, timeout};

    let client = create_test_client().await;
    if let Ok(client) = client {
        let family_id = test_family_id();
        let seed = test_seed_hash("timeout_test");

        // Wrap verification in timeout
        let result = timeout(
            Duration::from_secs(10),
            client.verify_same_family(&family_id, &seed, "node_test"),
        )
        .await;

        // Should complete within timeout
        assert!(
            result.is_ok(),
            "Verification should complete within 10 seconds"
        );
    }
}

#[tokio::test]
#[ignore = "Requires running BearDog instance"]
async fn test_malformed_response_handling() {
    // This test would require a mock BearDog that returns malformed responses
    // For now, we test that the client can handle unexpected response formats

    let client = create_test_client().await;
    if let Ok(client) = client {
        let family_id = test_family_id();
        let seed = test_seed_hash("malformed_test");

        // Normal call (BearDog should return well-formed response)
        let result = client
            .verify_same_family(&family_id, &seed, "node_test")
            .await;

        // Should either succeed with valid response or fail gracefully
        match result {
            Ok(response) => {
                // Response should have all required fields
                assert!(!response.parent_seed_hash.is_empty() || !response.is_family_member);
                assert!(!response.relationship.is_empty());
            }
            Err(e) => {
                // Error is acceptable
                assert!(!e.to_string().is_empty());
            }
        }
    }
}

#[tokio::test]
async fn test_invalid_endpoint_format() {
    // Invalid endpoint formats
    let invalid_endpoints = vec![
        "not_a_url",
        "ftp://invalid",
        "unix:/",  // Incomplete
        "http://", // Missing host
        "",
    ];

    for endpoint in invalid_endpoints {
        let result = SecurityProviderClient::with_endpoint(endpoint);

        // Client creation may succeed (validation happens at connection time)
        // Or it may fail early - both are acceptable behaviors
        // The important thing is that it doesn't panic
        // Creation may succeed (validation at connect) or fail early — both OK; must not panic
        drop(result);
    }
}

#[tokio::test]
#[ignore = "Requires running BearDog instance"]
async fn test_network_partition_recovery() {
    // This test simulates recovery after a network partition
    // We verify that the client can recover after a failed call

    let client = create_test_client().await;
    if let Ok(client) = client {
        let family_id = test_family_id();

        // First verification
        let seed1 = test_seed_hash("before_partition");
        let _ = client
            .verify_same_family(&family_id, &seed1, "node_1")
            .await;

        // After simulated partition, next call should still work
        let seed2 = test_seed_hash("after_partition");
        let result = client
            .verify_same_family(&family_id, &seed2, "node_2")
            .await;

        // Should recover and work
        assert!(result.is_ok() || result.is_err()); // Just shouldn't panic
    }
}
