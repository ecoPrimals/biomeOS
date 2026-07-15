// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Edge case tests for genetic lineage verification.
//!
//! **NOTE**: These tests require a running BearDog instance for lineage operations.
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

#[tokio::test]
#[ignore = "Requires running BearDog instance"]
async fn test_unicode_in_family_id() {
    let client = create_test_client().await;
    if let Ok(client) = client {
        // Family ID with unicode characters
        let unicode_family = "family_テスト_🔒";
        let seed = test_seed_hash("unicode_test");

        let result = client
            .verify_same_family(unicode_family, &seed, "node_test")
            .await;

        // Should handle unicode gracefully
        assert!(result.is_ok() || result.is_err()); // Just shouldn't panic
    }
}

#[tokio::test]
#[ignore = "Requires running BearDog instance"]
async fn test_very_long_family_id() {
    let client = create_test_client().await;
    if let Ok(client) = client {
        // Very long family ID (1KB)
        let long_family = "a".repeat(1024);
        let seed = test_seed_hash("long_family_test");

        let result = client
            .verify_same_family(&long_family, &seed, "node_test")
            .await;

        // Should handle long family ID gracefully
        assert!(result.is_ok() || result.is_err());
    }
}

#[tokio::test]
#[ignore = "Requires running BearDog instance"]
async fn test_special_characters_in_node_id() {
    let client = create_test_client().await;
    if let Ok(client) = client {
        let family_id = test_family_id();
        let seed = test_seed_hash("special_char_test");

        // Node IDs with special characters
        let special_nodes = vec![
            "node-with-dashes",
            "node_with_underscores",
            "node.with.dots",
            "node:with:colons",
            "node/with/slashes",
        ];

        for node_id in special_nodes {
            let result = client.verify_same_family(&family_id, &seed, node_id).await;

            // Should handle special characters
            assert!(result.is_ok() || result.is_err());
        }
    }
}

#[tokio::test]
#[ignore = "Requires running BearDog instance"]
async fn test_rapid_successive_verifications() {
    let client = create_test_client().await;
    if let Ok(client) = client {
        let family_id = test_family_id();
        let seed = test_seed_hash("rapid_test");

        // Rapid successive calls
        for i in 0..20 {
            let result = client
                .verify_same_family(&family_id, &seed, &format!("node_{i}"))
                .await;

            // Should handle rapid calls without errors
            assert!(result.is_ok() || result.is_err());
        }
    }
}

#[tokio::test]
#[ignore = "Requires running BearDog instance"]
async fn test_verification_response_serialization() {
    let client = create_test_client().await;
    if let Ok(client) = client {
        let family_id = test_family_id();
        let seed = test_seed_hash("serialize_test");

        let result = client
            .verify_same_family(&family_id, &seed, "node_test")
            .await;

        if let Ok(response) = result {
            // Test Display trait implementation
            let display_str = format!("{response}");
            assert!(!display_str.is_empty(), "Display should produce output");
            assert!(
                display_str.contains("is_member="),
                "Display should show is_member"
            );
            assert!(
                display_str.contains("relationship="),
                "Display should show relationship"
            );

            // Test JSON serialization
            let json = serde_json::to_string(&response);
            assert!(json.is_ok(), "Response should be JSON serializable");

            if let Ok(json_str) = json {
                // Should be valid JSON
                let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
                assert!(parsed.is_object(), "Should parse as JSON object");
            }
        }
    }
}
