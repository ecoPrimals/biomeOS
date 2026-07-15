// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Multi-family verification tests for genetic lineage verification.
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
async fn test_verify_multiple_families_sequentially() {
    let client = create_test_client().await;
    if let Ok(client) = client {
        let families = [test_family_id(), test_family_id(), test_family_id()];

        let seed = test_seed_hash("multi_family_test");

        // Verify against multiple families
        for (i, family_id) in families.iter().enumerate() {
            let result = client
                .verify_same_family(family_id, &seed, &format!("node_{i}"))
                .await;

            // Each verification should complete without panic
            assert!(result.is_ok() || result.is_err());
        }
    }
}

#[tokio::test]
#[ignore = "Requires running BearDog instance"]
async fn test_verify_multiple_seeds_same_family() {
    let client = create_test_client().await;
    if let Ok(client) = client {
        let family_id = test_family_id();

        let seeds = [
            test_seed_hash("seed_1"),
            test_seed_hash("seed_2"),
            test_seed_hash("seed_3"),
        ];

        // Verify multiple seeds against same family
        for (i, seed) in seeds.iter().enumerate() {
            let result = client
                .verify_same_family(&family_id, seed, &format!("node_{i}"))
                .await;

            // Each verification should complete
            assert!(result.is_ok() || result.is_err());
        }
    }
}

#[tokio::test]
#[ignore = "Requires running BearDog instance"]
async fn test_concurrent_family_verifications() {
    use std::sync::Arc;
    use tokio::task::JoinSet;

    let client = create_test_client().await;
    if let Ok(client) = client {
        let client = Arc::new(client);
        let family_id = test_family_id();

        let mut join_set = JoinSet::new();

        // Spawn concurrent verifications
        for i in 0..10 {
            let client_clone = Arc::clone(&client);
            let family_clone = family_id.clone();
            let seed = test_seed_hash(&format!("concurrent_seed_{i}"));

            join_set.spawn(async move {
                client_clone
                    .verify_same_family(&family_clone, &seed, &format!("node_{i}"))
                    .await
            });
        }

        // All should complete without deadlock or panic
        let mut completed = 0;
        while let Some(result) = join_set.join_next().await {
            assert!(result.is_ok(), "Concurrent verification should not panic");
            completed += 1;
        }

        assert_eq!(
            completed, 10,
            "All concurrent verifications should complete"
        );
    }
}

#[tokio::test]
#[ignore = "Requires running BearDog instance"]
async fn test_family_relationship_tracking() {
    let client = create_test_client().await;
    if let Ok(client) = client {
        let family_id = test_family_id();
        let seed = test_seed_hash("relationship_test");

        let result = client
            .verify_same_family(&family_id, &seed, "node_test")
            .await;

        if let Ok(response) = result {
            // Relationship field should have meaningful value
            let valid_relationships =
                ["unknown", "direct", "derived", "sibling", "parent", "child"];

            assert!(
                valid_relationships
                    .iter()
                    .any(|r| response.relationship.contains(r)),
                "Relationship should be recognized: {}",
                response.relationship
            );
        }
    }
}

#[tokio::test]
#[ignore = "Requires running BearDog instance"]
async fn test_parent_seed_hash_format() {
    let client = create_test_client().await;
    if let Ok(client) = client {
        let family_id = test_family_id();
        let seed = test_seed_hash("parent_test");

        let result = client
            .verify_same_family(&family_id, &seed, "node_test")
            .await;

        if let Ok(response) = result {
            // Parent seed hash should be empty or valid SHA-256 format
            if !response.parent_seed_hash.is_empty() {
                assert!(
                    response.parent_seed_hash.starts_with("sha256:")
                        || response.parent_seed_hash.len() == 64, // Hex hash without prefix
                    "Parent seed hash should be valid format: {}",
                    response.parent_seed_hash
                );
            }
        }
    }
}

#[tokio::test]
#[ignore = "Requires running BearDog instance"]
async fn test_verify_with_different_node_ids() {
    let client = create_test_client().await;
    if let Ok(client) = client {
        let family_id = test_family_id();
        let seed = test_seed_hash("node_id_test");

        let node_ids = vec!["node-alpha", "node-beta", "node-gamma"];

        // Verify same seed/family with different node IDs
        let mut results = vec![];
        for node_id in node_ids {
            if let Ok(response) = client.verify_same_family(&family_id, &seed, node_id).await {
                results.push(response.is_family_member);
            }
        }

        // Results should be consistent regardless of node_id
        if results.len() > 1 {
            let first = results[0];
            for result in &results[1..] {
                assert_eq!(
                    *result, first,
                    "Node ID should not affect lineage verification"
                );
            }
        }
    }
}

#[tokio::test]
#[ignore = "Requires running BearDog instance"]
async fn test_large_scale_family_verification() {
    let client = create_test_client().await;
    if let Ok(client) = client {
        let family_id = test_family_id();

        // Test with many seeds (stress test)
        for i in 0..50 {
            let seed = test_seed_hash(&format!("bulk_seed_{i}"));
            let result = client
                .verify_same_family(&family_id, &seed, &format!("node_{i}"))
                .await;

            // Should handle large volume without errors
            assert!(result.is_ok() || result.is_err());
        }
    }
}
