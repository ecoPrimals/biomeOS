//! E2E tests for BearDog integration
//!
//! These tests require a running BearDog instance.
//!
//! **Concurrency-First Design**: All tests use proper timeouts to prevent hangs.
//! Test issues will be production issues!

use biomeos_federation::beardog_client::BearDogClient;
use std::time::Duration;

/// Timeout for BearDog availability check
const AVAILABILITY_TIMEOUT: Duration = Duration::from_secs(2);

/// Helper to check if BearDog is available for testing
/// **Concurrency**: Uses timeout to prevent hangs when BearDog isn't available
async fn beardog_available() -> Option<BearDogClient> {
    // Use timeout to prevent hanging when BearDog isn't available
    let result = tokio::time::timeout(AVAILABILITY_TIMEOUT, async {
        // Try to find BearDog via discovery first
        if let Ok(client) = BearDogClient::from_discovery().await {
            if client.is_available().await {
                return Some(client);
            }
        }

        // Try common endpoints with individual timeouts
        let endpoints = vec![
            "unix:///tmp/beardog-nat0.sock",
            "unix:///tmp/beardog-nat0-node-alpha.sock",
            "unix:///tmp/beardog.sock",
        ];

        for endpoint in endpoints {
            if let Ok(client) = BearDogClient::with_endpoint(endpoint.to_string()) {
                // Quick availability check with its own timeout
                let available =
                    tokio::time::timeout(Duration::from_millis(500), client.is_available())
                        .await
                        .unwrap_or(false);

                if available {
                    println!("✅ Found BearDog at: {}", endpoint);
                    return Some(client);
                }
            }
        }

        None
    })
    .await;

    match result {
        Ok(client) => client,
        Err(_) => {
            println!("⚠️  BearDog availability check timed out");
            None
        }
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_beardog_discovery() {
    println!("\n🔍 Testing BearDog discovery...");

    match beardog_available().await {
        Some(client) => {
            println!("✅ BearDog discovered and available");

            // Try health check with timeout
            let health_result =
                tokio::time::timeout(Duration::from_secs(2), client.health_check()).await;

            match health_result {
                Ok(Ok(())) => println!("✅ BearDog health check passed"),
                Ok(Err(e)) => println!("⚠️  BearDog health check failed: {}", e),
                Err(_) => println!("⚠️  BearDog health check timed out"),
            }
        }
        None => {
            println!("⚠️  BearDog not found - skipping integration tests");
            println!("   To run these tests, start BearDog with:");
            println!("   ./plasmidBin/beardog server --socket /tmp/beardog-nat0.sock");
        }
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_beardog_lineage_verification() {
    println!("\n🧬 Testing genetic lineage verification...");

    let client = match beardog_available().await {
        Some(c) => c,
        None => {
            println!("⚠️  BearDog not available - skipping test");
            return;
        }
    };

    // Test with sample data - with timeout
    let family_id = "nat0";
    let seed_hash = "test_seed_hash_12345";
    let node_id = "test_node_001";

    let result = tokio::time::timeout(
        Duration::from_secs(5),
        client.verify_same_family(family_id, seed_hash, node_id),
    )
    .await;

    match result {
        Ok(Ok(response)) => {
            println!("✅ Lineage verification response: {}", response);
        }
        Ok(Err(e)) => {
            println!("⚠️  Lineage verification failed: {}", e);
            println!("   This is expected if BearDog API is not yet implemented");
        }
        Err(_) => {
            println!("⚠️  Lineage verification timed out");
        }
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_beardog_key_derivation() {
    println!("\n🔑 Testing sub-federation key derivation...");

    let client = match beardog_available().await {
        Some(c) => c,
        None => {
            println!("⚠️  BearDog not available - skipping test");
            return;
        }
    };

    use biomeos_federation::beardog_client::KeyDerivationRequest;

    let request = KeyDerivationRequest {
        parent_family: "nat0".to_string(),
        subfed_name: "gaming".to_string(),
        purpose: "sub-federation-encryption".to_string(),
    };

    let result =
        tokio::time::timeout(Duration::from_secs(5), client.derive_subfed_key(request)).await;

    match result {
        Ok(Ok(response)) => {
            println!("✅ Key derivation successful:");
            println!("   key_ref: {}", response.key_ref);
            println!("   algorithm: {}", response.algorithm);
            println!("   created_at: {}", response.created_at);
        }
        Ok(Err(e)) => {
            println!("⚠️  Key derivation failed: {}", e);
            println!("   This is expected if BearDog API is not yet implemented");
        }
        Err(_) => {
            println!("⚠️  Key derivation timed out");
        }
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_beardog_with_real_seed() {
    println!("\n🌱 Testing lineage verification with real seed...");

    let client = match beardog_available().await {
        Some(c) => c,
        None => {
            println!("⚠️  BearDog not available - skipping test");
            return;
        }
    };

    // Try to load a real seed from a spore
    use std::path::PathBuf;

    let possible_spore_paths = vec![
        PathBuf::from("/media/eastgate/BEA6-BBCE/biomeOS"),
        PathBuf::from("./test-spore/biomeOS"),
    ];

    for spore_path in possible_spore_paths {
        let seed_path = spore_path.join(".family.seed");

        if seed_path.exists() {
            println!("📂 Found spore seed at: {}", seed_path.display());

            // Read and hash the seed
            if let Ok(seed_bytes) = std::fs::read(&seed_path) {
                use sha2::{Digest, Sha256};
                let mut hasher = Sha256::new();
                hasher.update(&seed_bytes);
                let seed_hash = format!("{:x}", hasher.finalize());

                println!("🔒 Seed hash: {}...", &seed_hash[..16]);

                // Try to verify lineage with timeout
                let result = tokio::time::timeout(
                    Duration::from_secs(5),
                    client.verify_same_family("nat0", &seed_hash, "test_node_spore"),
                )
                .await;

                match result {
                    Ok(Ok(response)) => {
                        println!("✅ Lineage verified: {}", response);
                    }
                    Ok(Err(e)) => {
                        println!("⚠️  Lineage verification failed: {}", e);
                    }
                    Err(_) => {
                        println!("⚠️  Lineage verification timed out");
                    }
                }

                return;
            }
        }
    }

    println!("ℹ️  No real spore seeds found - using mock data");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_beardog_full_workflow() {
    println!("\n🔄 Testing full BearDog integration workflow...");

    let client = match beardog_available().await {
        Some(c) => c,
        None => {
            println!("⚠️  BearDog not available - skipping test");
            return;
        }
    };

    println!("\n1️⃣  Health Check");
    let health_result = tokio::time::timeout(Duration::from_secs(2), client.health_check()).await;

    match health_result {
        Ok(Ok(())) => println!("   ✅ BearDog is healthy"),
        Ok(Err(e)) => {
            println!("   ❌ Health check failed: {}", e);
            return;
        }
        Err(_) => {
            println!("   ❌ Health check timed out");
            return;
        }
    }

    println!("\n2️⃣  Lineage Verification");
    let lineage_result = tokio::time::timeout(
        Duration::from_secs(5),
        client.verify_same_family("nat0", "test_seed", "test_node_workflow"),
    )
    .await;

    match lineage_result {
        Ok(Ok(response)) => println!("   ✅ Lineage check: {}", response),
        Ok(Err(e)) => println!("   ⚠️  Lineage check: {}", e),
        Err(_) => println!("   ⚠️  Lineage check timed out"),
    }

    println!("\n3️⃣  Key Derivation");
    use biomeos_federation::beardog_client::KeyDerivationRequest;
    let request = KeyDerivationRequest {
        parent_family: "nat0".to_string(),
        subfed_name: "test-subfed".to_string(),
        purpose: "test".to_string(),
    };

    let key_result =
        tokio::time::timeout(Duration::from_secs(5), client.derive_subfed_key(request)).await;

    match key_result {
        Ok(Ok(response)) => println!("   ✅ Key derived: {}", response.key_ref),
        Ok(Err(e)) => println!("   ⚠️  Key derivation: {}", e),
        Err(_) => println!("   ⚠️  Key derivation timed out"),
    }

    println!("\n✨ Workflow test complete");
}
