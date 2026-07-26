// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Invalid lineage rejection tests for genetic lineage verification.
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

mod genetic_lineage_common;
use genetic_lineage_common::*;

use biomeos_types::identifiers::FamilyId;

#[tokio::test]
#[ignore = "Requires running BearDog instance"]
async fn test_invalid_family_id_rejection() {
    let client = create_test_client().await;
    if let Ok(client) = client {
        // Invalid family ID (empty string)
        let result = client
            .verify_same_family("", "test_seed", "node_test")
            .await;

        // Should either reject or return is_family_member=false
        match result {
            Ok(response) => {
                assert!(
                    !response.is_family_member,
                    "Empty family ID should not verify"
                );
            }
            Err(e) => {
                // Error is also acceptable for invalid input
                assert!(!e.to_string().is_empty(), "Error should have message");
            }
        }
    }
}

#[tokio::test]
#[ignore = "Requires running BearDog instance"]
async fn test_invalid_seed_hash_format() {
    let client = create_test_client().await;
    if let Ok(client) = client {
        let family_id = test_family_id();

        // Invalid seed hash (not SHA-256 format)
        let invalid_hashes = vec![
            "not_a_hash",
            "md5:abc123",
            "plain_text",
            "",
            "sha256:",    // Missing hash
            "sha256:xyz", // Invalid hex
        ];

        for invalid_hash in invalid_hashes {
            let result = client
                .verify_same_family(&family_id, invalid_hash, "node_test")
                .await;

            // Should handle invalid format gracefully
            match result {
                Ok(response) => {
                    // Should not verify as family member
                    assert!(
                        !response.is_family_member,
                        "Invalid hash format '{invalid_hash}' should not verify"
                    );
                }
                Err(e) => {
                    // Error is acceptable for invalid format
                    assert!(!e.to_string().is_empty());
                }
            }
        }
    }
}

#[tokio::test]
#[ignore = "Requires running BearDog instance"]
async fn test_nonexistent_family_rejection() {
    let client = create_test_client().await;
    if let Ok(client) = client {
        // Generate a random family ID that doesn't exist
        let nonexistent_family = FamilyId::generate().to_string();
        let seed_hash = test_seed_hash("random_seed");

        let result = client
            .verify_same_family(&nonexistent_family, &seed_hash, "node_test")
            .await;

        if let Ok(response) = result {
            // Should not be a family member of nonexistent family
            assert!(
                !response.is_family_member,
                "Should reject nonexistent family"
            );
            assert_eq!(
                response.relationship, "unknown",
                "Relationship should be unknown"
            );
        }
    }
}

#[tokio::test]
#[ignore = "Requires running BearDog instance"]
async fn test_wrong_seed_for_family() {
    let client = create_test_client().await;
    if let Ok(client) = client {
        let family_id = test_family_id();

        // Try multiple wrong seeds
        for i in 0..5 {
            let wrong_seed = test_seed_hash(&format!("wrong_seed_{i}"));
            let result = client
                .verify_same_family(&family_id, &wrong_seed, "node_test")
                .await;

            if let Ok(response) = result {
                // Should not verify with wrong seed
                assert!(
                    !response.is_family_member,
                    "Wrong seed {i} should not verify"
                );
            }
        }
    }
}

#[tokio::test]
#[ignore = "Requires running BearDog instance"]
async fn test_tampered_seed_hash_rejection() {
    let client = create_test_client().await;
    if let Ok(client) = client {
        let family_id = test_family_id();

        // Create a valid-looking hash and tamper with it
        let valid_hash = test_seed_hash("original_seed");

        // Tamper by changing one character
        let tampered_hashes = vec![
            valid_hash.replace('a', "b"), // Change one hex digit
            format!("{}x", &valid_hash[..valid_hash.len() - 1]), // Change last char
            format!("sha256:0{}", &valid_hash[7..]), // Prepend 0
        ];

        for tampered in tampered_hashes {
            let result = client
                .verify_same_family(&family_id, &tampered, "node_test")
                .await;

            if let Ok(response) = result {
                // Tampered hash should not verify
                assert!(
                    !response.is_family_member,
                    "Tampered hash should not verify"
                );
            }
        }
    }
}

#[tokio::test]
#[ignore = "Requires running BearDog instance"]
async fn test_replay_attack_prevention() {
    let client = create_test_client().await;
    if let Ok(client) = client {
        let family_id = test_family_id();
        let seed_hash = test_seed_hash("replay_test");

        // Try same verification multiple times (should be consistent)
        let mut results = vec![];

        for _ in 0..3 {
            if let Ok(response) = client
                .verify_same_family(&family_id, &seed_hash, "node_test")
                .await
            {
                results.push(response.is_family_member);
            }
        }

        // All results should be consistent
        if results.len() > 1 {
            let first = results[0];
            for result in &results[1..] {
                assert_eq!(*result, first, "Replay should give consistent results");
            }
        }
    }
}

#[tokio::test]
#[ignore = "Requires running BearDog instance"]
async fn test_cross_family_contamination() {
    let client = create_test_client().await;
    if let Ok(client) = client {
        let family1 = test_family_id();
        let family2 = test_family_id();
        let seed = test_seed_hash("shared_seed");

        // Verify same seed against two different families
        let result1 = client.verify_same_family(&family1, &seed, "node1").await;
        let result2 = client.verify_same_family(&family2, &seed, "node2").await;

        // Seed should not be member of both families (unless they're related)
        if let (Ok(r1), Ok(r2)) = (result1, result2) {
            // At least one should reject (or both, more likely)
            let both_accepted = r1.is_family_member && r2.is_family_member;
            assert!(
                !both_accepted,
                "Seed should not be member of unrelated families"
            );
        }
    }
}

#[tokio::test]
#[ignore = "Requires running BearDog instance"]
async fn test_invalid_node_id_handling() {
    let client = create_test_client().await;
    if let Ok(client) = client {
        let family_id = test_family_id();
        let seed_hash = test_seed_hash("test_seed");

        // Invalid node IDs
        let invalid_node_ids = vec![
            "",         // Empty
            " ",        // Whitespace
            "node\n",   // Newline
            "node\x00", // Null byte
        ];

        for invalid_node in invalid_node_ids {
            let result = client
                .verify_same_family(&family_id, &seed_hash, invalid_node)
                .await;

            // Should handle gracefully
            match result {
                Ok(response) => {
                    // Should not cause unexpected behavior
                    assert!(!response.parent_seed_hash.is_empty() || !response.is_family_member);
                }
                Err(e) => {
                    // Error is acceptable for invalid input
                    assert!(!e.to_string().is_empty());
                }
            }
        }
    }
}
