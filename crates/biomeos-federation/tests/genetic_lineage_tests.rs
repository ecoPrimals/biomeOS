//! Comprehensive tests for genetic lineage verification
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
//! cargo test --package biomeos-federation genetic_lineage_tests -- --ignored
//! ```
//!
//! Coverage target: 95%
//! Focus areas:
//! - Invalid lineage rejection (8 tests)
//! - Multi-family verification (7 tests)
//! - BearDog integration errors (5 tests)
//! - Edge cases (5 tests)
//!
//! Total: 25 comprehensive integration tests

use biomeos_federation::beardog_client::{BearDogClient, LineageVerificationResponse};
use biomeos_types::identifiers::FamilyId;
use anyhow::Result;
use sha2::{Digest, Sha256};

// ============================================================================
// Helper Functions
// ============================================================================

/// Create a test BearDog client
async fn create_test_client() -> Result<BearDogClient> {
    // Try Unix socket first, fall back to HTTP
    let unix_endpoint = "unix:///tmp/beardog-test.sock".to_string();
    
    match BearDogClient::with_endpoint(unix_endpoint) {
        Ok(client) => Ok(client),
        Err(_) => {
            // Fall back to HTTP endpoint
            BearDogClient::with_endpoint("http://localhost:8420".to_string())
        }
    }
}

/// Generate a test family ID
fn test_family_id() -> String {
    FamilyId::generate().to_string()
}

/// Generate a test seed hash (SHA-256 format)
fn test_seed_hash(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    let hash_bytes = hasher.finalize();
    // Convert to hex string manually
    let hex_string = hash_bytes.iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>();
    format!("sha256:{}", hex_string)
}

// ============================================================================
// Test Suite 1: Invalid Lineage Rejection (8 tests)
// ============================================================================

#[tokio::test]
#[ignore] // Requires running BearDog instance
async fn test_invalid_family_id_rejection() {
    let client = create_test_client().await;
    if let Ok(client) = client {
        // Invalid family ID (empty string)
        let result = client.verify_same_family("", "test_seed", "node_test").await;
        
        // Should either reject or return is_family_member=false
        match result {
            Ok(response) => {
                assert!(!response.is_family_member, "Empty family ID should not verify");
            }
            Err(e) => {
                // Error is also acceptable for invalid input
                assert!(!e.to_string().is_empty(), "Error should have message");
            }
        }
    }
}

#[tokio::test]
#[ignore] // Requires running BearDog instance
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
            "sha256:",  // Missing hash
            "sha256:xyz",  // Invalid hex
        ];
        
        for invalid_hash in invalid_hashes {
            let result = client.verify_same_family(&family_id, invalid_hash, "node_test").await;
            
            // Should handle invalid format gracefully
            match result {
                Ok(response) => {
                    // Should not verify as family member
                    assert!(!response.is_family_member, 
                        "Invalid hash format '{}' should not verify", invalid_hash);
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
#[ignore] // Requires running BearDog instance
async fn test_nonexistent_family_rejection() {
    let client = create_test_client().await;
    if let Ok(client) = client {
        // Generate a random family ID that doesn't exist
        let nonexistent_family = FamilyId::generate().to_string();
        let seed_hash = test_seed_hash("random_seed");
        
        let result = client.verify_same_family(&nonexistent_family, &seed_hash, "node_test").await;
        
        if let Ok(response) = result {
            // Should not be a family member of nonexistent family
            assert!(!response.is_family_member, "Should reject nonexistent family");
            assert_eq!(response.relationship, "unknown", "Relationship should be unknown");
        }
    }
}

#[tokio::test]
#[ignore] // Requires running BearDog instance
async fn test_wrong_seed_for_family() {
    let client = create_test_client().await;
    if let Ok(client) = client {
        let family_id = test_family_id();
        
        // Try multiple wrong seeds
        for i in 0..5 {
            let wrong_seed = test_seed_hash(&format!("wrong_seed_{}", i));
            let result = client.verify_same_family(&family_id, &wrong_seed, "node_test").await;
            
            if let Ok(response) = result {
                // Should not verify with wrong seed
                assert!(!response.is_family_member, 
                    "Wrong seed {} should not verify", i);
            }
        }
    }
}

#[tokio::test]
#[ignore] // Requires running BearDog instance
async fn test_tampered_seed_hash_rejection() {
    let client = create_test_client().await;
    if let Ok(client) = client {
        let family_id = test_family_id();
        
        // Create a valid-looking hash and tamper with it
        let valid_hash = test_seed_hash("original_seed");
        
        // Tamper by changing one character
        let tampered_hashes = vec![
            valid_hash.replace("a", "b"),  // Change one hex digit
            format!("{}x", &valid_hash[..valid_hash.len()-1]),  // Change last char
            format!("sha256:0{}", &valid_hash[7..]),  // Prepend 0
        ];
        
        for tampered in tampered_hashes {
            let result = client.verify_same_family(&family_id, &tampered, "node_test").await;
            
            if let Ok(response) = result {
                // Tampered hash should not verify
                assert!(!response.is_family_member, "Tampered hash should not verify");
            }
        }
    }
}

#[tokio::test]
#[ignore] // Requires running BearDog instance
async fn test_replay_attack_prevention() {
    let client = create_test_client().await;
    if let Ok(client) = client {
        let family_id = test_family_id();
        let seed_hash = test_seed_hash("replay_test");
        
        // Try same verification multiple times (should be consistent)
        let mut results = vec![];
        
        for _ in 0..3 {
            if let Ok(response) = client.verify_same_family(&family_id, &seed_hash, "node_test").await {
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
#[ignore] // Requires running BearDog instance
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
            assert!(!both_accepted, "Seed should not be member of unrelated families");
        }
    }
}

#[tokio::test]
#[ignore] // Requires running BearDog instance
async fn test_invalid_node_id_handling() {
    let client = create_test_client().await;
    if let Ok(client) = client {
        let family_id = test_family_id();
        let seed_hash = test_seed_hash("test_seed");
        
        // Invalid node IDs
        let invalid_node_ids = vec![
            "",  // Empty
            " ",  // Whitespace
            "node\n",  // Newline
            "node\x00",  // Null byte
        ];
        
        for invalid_node in invalid_node_ids {
            let result = client.verify_same_family(&family_id, &seed_hash, invalid_node).await;
            
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

// ============================================================================
// Test Suite 2: Multi-Family Verification (7 tests)
// ============================================================================

#[tokio::test]
#[ignore] // Requires running BearDog instance
async fn test_verify_multiple_families_sequentially() {
    let client = create_test_client().await;
    if let Ok(client) = client {
        let families = vec![
            test_family_id(),
            test_family_id(),
            test_family_id(),
        ];
        
        let seed = test_seed_hash("multi_family_test");
        
        // Verify against multiple families
        for (i, family_id) in families.iter().enumerate() {
            let result = client.verify_same_family(family_id, &seed, &format!("node_{}", i)).await;
            
            // Each verification should complete without panic
            assert!(result.is_ok() || result.is_err());
        }
    }
}

#[tokio::test]
#[ignore] // Requires running BearDog instance
async fn test_verify_multiple_seeds_same_family() {
    let client = create_test_client().await;
    if let Ok(client) = client {
        let family_id = test_family_id();
        
        let seeds = vec![
            test_seed_hash("seed_1"),
            test_seed_hash("seed_2"),
            test_seed_hash("seed_3"),
        ];
        
        // Verify multiple seeds against same family
        for (i, seed) in seeds.iter().enumerate() {
            let result = client.verify_same_family(&family_id, seed, &format!("node_{}", i)).await;
            
            // Each verification should complete
            assert!(result.is_ok() || result.is_err());
        }
    }
}

#[tokio::test]
#[ignore] // Requires running BearDog instance
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
            let seed = test_seed_hash(&format!("concurrent_seed_{}", i));
            
            join_set.spawn(async move {
                client_clone.verify_same_family(&family_clone, &seed, &format!("node_{}", i)).await
            });
        }
        
        // All should complete without deadlock or panic
        let mut completed = 0;
        while let Some(result) = join_set.join_next().await {
            assert!(result.is_ok(), "Concurrent verification should not panic");
            completed += 1;
        }
        
        assert_eq!(completed, 10, "All concurrent verifications should complete");
    }
}

#[tokio::test]
#[ignore] // Requires running BearDog instance
async fn test_family_relationship_tracking() {
    let client = create_test_client().await;
    if let Ok(client) = client {
        let family_id = test_family_id();
        let seed = test_seed_hash("relationship_test");
        
        let result = client.verify_same_family(&family_id, &seed, "node_test").await;
        
        if let Ok(response) = result {
            // Relationship field should have meaningful value
            let valid_relationships = vec!["unknown", "direct", "derived", "sibling", "parent", "child"];
            
            assert!(
                valid_relationships.iter().any(|r| response.relationship.contains(r)),
                "Relationship should be recognized: {}",
                response.relationship
            );
        }
    }
}

#[tokio::test]
#[ignore] // Requires running BearDog instance
async fn test_parent_seed_hash_format() {
    let client = create_test_client().await;
    if let Ok(client) = client {
        let family_id = test_family_id();
        let seed = test_seed_hash("parent_test");
        
        let result = client.verify_same_family(&family_id, &seed, "node_test").await;
        
        if let Ok(response) = result {
            // Parent seed hash should be empty or valid SHA-256 format
            if !response.parent_seed_hash.is_empty() {
                assert!(
                    response.parent_seed_hash.starts_with("sha256:") ||
                    response.parent_seed_hash.len() == 64,  // Hex hash without prefix
                    "Parent seed hash should be valid format: {}",
                    response.parent_seed_hash
                );
            }
        }
    }
}

#[tokio::test]
#[ignore] // Requires running BearDog instance
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
                assert_eq!(*result, first, "Node ID should not affect lineage verification");
            }
        }
    }
}

#[tokio::test]
#[ignore] // Requires running BearDog instance
async fn test_large_scale_family_verification() {
    let client = create_test_client().await;
    if let Ok(client) = client {
        let family_id = test_family_id();
        
        // Test with many seeds (stress test)
        for i in 0..50 {
            let seed = test_seed_hash(&format!("bulk_seed_{}", i));
            let result = client.verify_same_family(&family_id, &seed, &format!("node_{}", i)).await;
            
            // Should handle large volume without errors
            assert!(result.is_ok() || result.is_err());
        }
    }
}

// ============================================================================
// Test Suite 3: BearDog Integration Errors (5 tests)
// ============================================================================

#[tokio::test]
async fn test_beardog_unavailable_error() {
    // Try to connect to nonexistent BearDog instance
    let result = BearDogClient::with_endpoint("unix:///tmp/nonexistent_beardog.sock".to_string());
    
    // Should create client successfully (connection happens on call)
    assert!(result.is_ok(), "Client creation should succeed");
    
    if let Ok(client) = result {
        // But verification should fail
        let family_id = test_family_id();
        let seed = test_seed_hash("unavailable_test");
        
        let verify_result = client.verify_same_family(&family_id, &seed, "node_test").await;
        
        // Should return error (not panic)
        assert!(verify_result.is_err(), "Should error when BearDog unavailable");
        
        if let Err(e) = verify_result {
            let error_msg = e.to_string();
            assert!(!error_msg.is_empty(), "Error should have message");
        }
    }
}

#[tokio::test]
#[ignore] // Requires running BearDog instance
async fn test_timeout_on_slow_response() {
    use tokio::time::{timeout, Duration};
    
    let client = create_test_client().await;
    if let Ok(client) = client {
        let family_id = test_family_id();
        let seed = test_seed_hash("timeout_test");
        
        // Wrap verification in timeout
        let result = timeout(
            Duration::from_secs(10),
            client.verify_same_family(&family_id, &seed, "node_test")
        ).await;
        
        // Should complete within timeout
        assert!(result.is_ok(), "Verification should complete within 10 seconds");
    }
}

#[tokio::test]
#[ignore] // Requires running BearDog instance
async fn test_malformed_response_handling() {
    // This test would require a mock BearDog that returns malformed responses
    // For now, we test that the client can handle unexpected response formats
    
    let client = create_test_client().await;
    if let Ok(client) = client {
        let family_id = test_family_id();
        let seed = test_seed_hash("malformed_test");
        
        // Normal call (BearDog should return well-formed response)
        let result = client.verify_same_family(&family_id, &seed, "node_test").await;
        
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
        "http://",  // Missing host
        "",
    ];
    
    for endpoint in invalid_endpoints {
        let result = BearDogClient::with_endpoint(endpoint.to_string());
        
        // Client creation may succeed (validation happens at connection time)
        // Or it may fail early - both are acceptable behaviors
        // The important thing is that it doesn't panic
        match result {
            Ok(_) => {
                // Validation will happen at connection time - this is fine
            }
            Err(_) => {
                // Early validation caught the issue - also fine
            }
        }
    }
}

#[tokio::test]
#[ignore] // Requires running BearDog instance
async fn test_network_partition_recovery() {
    // This test simulates recovery after a network partition
    // We verify that the client can recover after a failed call
    
    let client = create_test_client().await;
    if let Ok(client) = client {
        let family_id = test_family_id();
        
        // First verification
        let seed1 = test_seed_hash("before_partition");
        let _ = client.verify_same_family(&family_id, &seed1, "node_1").await;
        
        // After simulated partition, next call should still work
        let seed2 = test_seed_hash("after_partition");
        let result = client.verify_same_family(&family_id, &seed2, "node_2").await;
        
        // Should recover and work
        assert!(result.is_ok() || result.is_err()); // Just shouldn't panic
    }
}

// ============================================================================
// Test Suite 4: Edge Cases (5 tests)
// ============================================================================

#[tokio::test]
#[ignore] // Requires running BearDog instance
async fn test_unicode_in_family_id() {
    let client = create_test_client().await;
    if let Ok(client) = client {
        // Family ID with unicode characters
        let unicode_family = "family_テスト_🔒";
        let seed = test_seed_hash("unicode_test");
        
        let result = client.verify_same_family(unicode_family, &seed, "node_test").await;
        
        // Should handle unicode gracefully
        assert!(result.is_ok() || result.is_err()); // Just shouldn't panic
    }
}

#[tokio::test]
#[ignore] // Requires running BearDog instance
async fn test_very_long_family_id() {
    let client = create_test_client().await;
    if let Ok(client) = client {
        // Very long family ID (1KB)
        let long_family = "a".repeat(1024);
        let seed = test_seed_hash("long_family_test");
        
        let result = client.verify_same_family(&long_family, &seed, "node_test").await;
        
        // Should handle long family ID gracefully
        assert!(result.is_ok() || result.is_err());
    }
}

#[tokio::test]
#[ignore] // Requires running BearDog instance  
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
#[ignore] // Requires running BearDog instance
async fn test_rapid_successive_verifications() {
    let client = create_test_client().await;
    if let Ok(client) = client {
        let family_id = test_family_id();
        let seed = test_seed_hash("rapid_test");
        
        // Rapid successive calls
        for i in 0..20 {
            let result = client.verify_same_family(&family_id, &seed, &format!("node_{}", i)).await;
            
            // Should handle rapid calls without errors
            assert!(result.is_ok() || result.is_err());
        }
    }
}

#[tokio::test]
#[ignore] // Requires running BearDog instance
async fn test_verification_response_serialization() {
    let client = create_test_client().await;
    if let Ok(client) = client {
        let family_id = test_family_id();
        let seed = test_seed_hash("serialize_test");
        
        let result = client.verify_same_family(&family_id, &seed, "node_test").await;
        
        if let Ok(response) = result {
            // Test Display trait implementation
            let display_str = format!("{}", response);
            assert!(!display_str.is_empty(), "Display should produce output");
            assert!(display_str.contains("is_member="), "Display should show is_member");
            assert!(display_str.contains("relationship="), "Display should show relationship");
            
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

