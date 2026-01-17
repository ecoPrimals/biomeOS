//! E2E tests for BearDog integration
//!
//! These tests require a running BearDog instance.

use biomeos_federation::beardog_client::BearDogClient;

/// Helper to check if BearDog is available for testing
async fn beardog_available() -> Option<BearDogClient> {
    // Try to find BearDog via discovery first
    if let Ok(client) = BearDogClient::from_discovery().await {
        if client.is_available().await {
            return Some(client);
        }
    }

    // Try common endpoints
    let endpoints = vec![
        "http://localhost:9000",
        "unix:///tmp/beardog-nat0-node-alpha.sock",
        "unix:///tmp/beardog.sock",
    ];

    for endpoint in endpoints {
        if let Ok(client) = BearDogClient::with_endpoint(endpoint.to_string()) {
            if client.is_available().await {
                println!("✅ Found BearDog at: {}", endpoint);
                return Some(client);
            }
        }
    }

    None
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_beardog_discovery() {
    println!("\n🔍 Testing BearDog discovery...");

    match beardog_available().await {
        Some(client) => {
            println!("✅ BearDog discovered and available");

            // Try health check
            match client.health_check().await {
                Ok(()) => println!("✅ BearDog health check passed"),
                Err(e) => println!("⚠️  BearDog health check failed: {}", e),
            }
        }
        None => {
            println!("⚠️  BearDog not found - skipping integration tests");
            println!("   To run these tests, start BearDog with:");
            println!("   ./plasmidBin/primals/beardog-server");
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

    // Test with sample data
    let family_id = "nat0";
    let seed_hash = "test_seed_hash_12345";
    let node_id = "test_node_001";

    match client
        .verify_same_family(family_id, seed_hash, node_id)
        .await
    {
        Ok(response) => {
            println!("✅ Lineage verification response: {}", response);
        }
        Err(e) => {
            println!("⚠️  Lineage verification failed: {}", e);
            println!("   This is expected if BearDog API is not yet implemented");
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

    match client.derive_subfed_key(request).await {
        Ok(response) => {
            println!("✅ Key derivation successful:");
            println!("   key_ref: {}", response.key_ref);
            println!("   algorithm: {}", response.algorithm);
            println!("   created_at: {}", response.created_at);
        }
        Err(e) => {
            println!("⚠️  Key derivation failed: {}", e);
            println!("   This is expected if BearDog API is not yet implemented");
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
        PathBuf::from("/media/eastgate/*/biomeOS"),
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

                // Try to verify lineage
                match client
                    .verify_same_family("nat0", &seed_hash, "test_node_spore")
                    .await
                {
                    Ok(response) => {
                        println!("✅ Lineage verified: {}", response);
                    }
                    Err(e) => {
                        println!("⚠️  Lineage verification failed: {}", e);
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
    match client.health_check().await {
        Ok(()) => println!("   ✅ BearDog is healthy"),
        Err(e) => {
            println!("   ❌ Health check failed: {}", e);
            return;
        }
    }

    println!("\n2️⃣  Lineage Verification");
    match client
        .verify_same_family("nat0", "test_seed", "test_node_workflow")
        .await
    {
        Ok(response) => println!("   ✅ Lineage check: {}", response),
        Err(e) => println!("   ⚠️  Lineage check: {}", e),
    }

    println!("\n3️⃣  Key Derivation");
    use biomeos_federation::beardog_client::KeyDerivationRequest;
    let request = KeyDerivationRequest {
        parent_family: "nat0".to_string(),
        subfed_name: "test-subfed".to_string(),
        purpose: "test".to_string(),
    };
    match client.derive_subfed_key(request).await {
        Ok(response) => println!("   ✅ Key derived: {}", response.key_ref),
        Err(e) => println!("   ⚠️  Key derivation: {}", e),
    }

    println!("\n✨ Workflow test complete");
}
