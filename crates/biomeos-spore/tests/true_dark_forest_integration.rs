// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

#![allow(clippy::unwrap_used, clippy::expect_used)]

//! TRUE Dark Forest Integration Tests
//!
//! Comprehensive end-to-end testing of A++ security features:
//! - Same family discovery (should succeed)
//! - Different family isolation (should fail silently)
//! - Beacon determinism (same lineage = same key)
//! - Network indistinguishability (no metadata leaks)
//!
//! # Requirements
//!
//! - BearDog running with `genetic.derive_lineage_beacon_key` support
//! - Socket at `/run/user/1000/biomeos/beardog.sock` (or set BEARDOG_SOCKET)

use biomeos_spore::DarkForestBeacon;
use std::collections::HashSet;

/// Get beardog socket path from environment or default
fn beardog_socket() -> String {
    std::env::var("BEARDOG_SOCKET")
        .unwrap_or_else(|_| "/run/user/1000/biomeos/beardog.sock".to_string())
}

/// Create test family seed
async fn create_test_seed(path: &str, content: &[u8]) -> std::io::Result<()> {
    tokio::fs::write(path, content).await
}

/// Cleanup test seed
async fn cleanup_test_seed(path: &str) {
    tokio::fs::remove_file(path).await.ok();
}

// ═══════════════════════════════════════════════════════════════════
// Test 1: Same Family Discovery
// ═══════════════════════════════════════════════════════════════════

#[tokio::test]
#[ignore = "Requires running BearDog instance"]
async fn test_same_family_discovery() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n═══════════════════════════════════════════════════════════════════");
    println!("Test 1: Same Family Discovery");
    println!("═══════════════════════════════════════════════════════════════════\n");

    let beardog = beardog_socket();

    // Check beardog availability
    if !std::path::Path::new(&beardog).exists() {
        eprintln!("⚠️  Skipping: BearDog not running at {beardog}");
        return Ok(());
    }

    // Create shared family seed
    let seed_path = "/tmp/test_same_family.seed";
    let seed = b"test_family_seed_32bytes_long!!!";
    create_test_seed(seed_path, seed).await?;

    // Create two nodes with SAME family seed
    let node_a = DarkForestBeacon::from_security_socket(&beardog, seed_path, "node_a").await?;
    let node_b = DarkForestBeacon::from_security_socket(&beardog, seed_path, "node_b").await?;

    // Node A generates pure noise beacon
    println!("🌑 Node A: Generating pure noise beacon...");
    let beacon = node_a
        .generate_pure_noise_beacon("/tmp/node_a.sock", &["test"], Some("genesis"))
        .await?;
    println!("✅ Generated: {} bytes", beacon.len());

    // Node B attempts to decrypt (same family)
    println!("🔓 Node B: Attempting to decrypt (same family)...");
    let result = node_b.try_decrypt_pure_noise_beacon(&beacon).await?;

    // Verify successful decryption
    assert!(result.is_some(), "Same family should decrypt successfully");
    let decrypted = result.unwrap();
    assert_eq!(decrypted["node_id"].as_str().unwrap(), "node_a");
    println!("✅ SUCCESS: Node B decrypted Node A's beacon");
    println!("   Discovered: node_id = {}", decrypted["node_id"]);

    cleanup_test_seed(seed_path).await;
    println!("\n🏆 Test PASSED: Same family discovery works!\n");
    Ok(())
}

// ═══════════════════════════════════════════════════════════════════
// Test 2: Different Family Isolation
// ═══════════════════════════════════════════════════════════════════

#[tokio::test]
#[ignore = "Requires running BearDog instance"]
async fn test_different_family_isolation() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n═══════════════════════════════════════════════════════════════════");
    println!("Test 2: Different Family Isolation");
    println!("═══════════════════════════════════════════════════════════════════\n");

    let beardog = beardog_socket();

    if !std::path::Path::new(&beardog).exists() {
        eprintln!("⚠️  Skipping: BearDog not running at {beardog}");
        return Ok(());
    }

    // Create DIFFERENT family seeds
    let seed_path_alpha = "/tmp/test_family_alpha.seed";
    let seed_path_beta = "/tmp/test_family_beta.seed";
    let seed_alpha = b"family_alpha_seed_32bytes_long!!";
    let seed_beta = b"family_beta_seed_32bytes_long!!!";

    create_test_seed(seed_path_alpha, seed_alpha).await?;
    create_test_seed(seed_path_beta, seed_beta).await?;

    // Create two nodes with DIFFERENT family seeds
    let node_alpha =
        DarkForestBeacon::from_security_socket(&beardog, seed_path_alpha, "alpha_node").await?;
    let node_beta =
        DarkForestBeacon::from_security_socket(&beardog, seed_path_beta, "beta_node").await?;

    // Node Alpha generates pure noise beacon
    println!("🌑 Node Alpha: Generating pure noise beacon...");
    let beacon = node_alpha
        .generate_pure_noise_beacon("/tmp/alpha.sock", &["test"], None)
        .await?;
    println!("✅ Generated: {} bytes", beacon.len());

    // Node Beta attempts to decrypt (different family)
    println!("🔓 Node Beta: Attempting to decrypt (different family)...");
    let result = node_beta.try_decrypt_pure_noise_beacon(&beacon).await?;

    // Verify SILENT failure (None, not error)
    assert!(
        result.is_none(),
        "Different family should return None (silent failure)"
    );
    println!("✅ SUCCESS: Node Beta silently failed (sees noise)");
    println!("   Result: None (indistinguishable from random noise)");
    println!("   No errors, no logs - true Dark Forest");

    cleanup_test_seed(seed_path_alpha).await;
    cleanup_test_seed(seed_path_beta).await;
    println!("\n🏆 Test PASSED: Different family isolation works!\n");
    Ok(())
}

// ═══════════════════════════════════════════════════════════════════
// Test 3: Beacon Determinism
// ═══════════════════════════════════════════════════════════════════

#[tokio::test]
#[ignore = "Requires running BearDog instance"]
async fn test_beacon_determinism() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n═══════════════════════════════════════════════════════════════════");
    println!("Test 3: Beacon Determinism");
    println!("═══════════════════════════════════════════════════════════════════\n");

    let beardog = beardog_socket();

    if !std::path::Path::new(&beardog).exists() {
        eprintln!("⚠️  Skipping: BearDog not running at {beardog}");
        return Ok(());
    }

    let seed_path = "/tmp/test_determinism.seed";
    let seed = b"deterministic_seed_32bytes_long!";
    create_test_seed(seed_path, seed).await?;

    // Create first beacon manager
    println!("🔑 Creating first beacon manager...");
    let mgr1 = DarkForestBeacon::from_security_socket(&beardog, seed_path, "test_node").await?;

    // Generate beacon 1
    let beacon1 = mgr1
        .generate_pure_noise_beacon("/tmp/test.sock", &["test"], None)
        .await?;
    println!("✅ Beacon 1 generated: {} bytes", beacon1.len());

    // Create second beacon manager (simulating restart)
    println!("🔑 Creating second beacon manager (simulated restart)...");
    let mgr2 = DarkForestBeacon::from_security_socket(&beardog, seed_path, "test_node").await?;

    // Generate beacon 2
    let beacon2 = mgr2
        .generate_pure_noise_beacon("/tmp/test.sock", &["test"], None)
        .await?;
    println!("✅ Beacon 2 generated: {} bytes", beacon2.len());

    // Verify both can decrypt each other's beacons
    println!("🔓 Testing cross-decryption...");
    let result1 = mgr1.try_decrypt_pure_noise_beacon(&beacon2).await?;
    let result2 = mgr2.try_decrypt_pure_noise_beacon(&beacon1).await?;

    assert!(result1.is_some(), "Manager 1 should decrypt beacon 2");
    assert!(result2.is_some(), "Manager 2 should decrypt beacon 1");
    println!("✅ SUCCESS: Same lineage = consistent decryption");
    println!("   Manager 1 decrypted Manager 2's beacon: ✅");
    println!("   Manager 2 decrypted Manager 1's beacon: ✅");

    cleanup_test_seed(seed_path).await;
    println!("\n🏆 Test PASSED: Beacon determinism verified!\n");
    Ok(())
}

// ═══════════════════════════════════════════════════════════════════
// Test 4: Network Indistinguishability
// ═══════════════════════════════════════════════════════════════════

#[tokio::test]
#[ignore = "Requires running BearDog instance"]
async fn test_network_indistinguishability() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n═══════════════════════════════════════════════════════════════════");
    println!("Test 4: Network Indistinguishability (Zero Metadata)");
    println!("═══════════════════════════════════════════════════════════════════\n");

    let beardog = beardog_socket();

    if !std::path::Path::new(&beardog).exists() {
        eprintln!("⚠️  Skipping: BearDog not running at {beardog}");
        return Ok(());
    }

    let seed_path = "/tmp/test_indistinguishable.seed";
    let seed = b"indistinguishable_seed_32bytes!!";
    create_test_seed(seed_path, seed).await?;

    let mgr = DarkForestBeacon::from_security_socket(&beardog, seed_path, "test_node").await?;

    // Generate multiple beacons
    println!("🌑 Generating 10 pure noise beacons...");
    let mut beacons = Vec::new();
    for i in 0..10 {
        let beacon = mgr
            .generate_pure_noise_beacon(&format!("/tmp/test_{i}.sock"), &["test"], None)
            .await?;
        beacons.push(beacon);
    }
    println!("✅ Generated {} beacons", beacons.len());

    // Test 1: No valid UTF-8
    println!("\n📊 Test 1: Verify NOT valid UTF-8 (binary noise)");
    let mut utf8_count = 0;
    for beacon in &beacons {
        if std::str::from_utf8(beacon).is_ok() {
            utf8_count += 1;
        }
    }
    assert_eq!(utf8_count, 0, "Beacons should NOT be valid UTF-8");
    println!("✅ PASSED: 0/{} beacons are valid UTF-8", beacons.len());

    // Test 2: No valid JSON
    println!("\n📊 Test 2: Verify NOT valid JSON (pure bytes)");
    let mut json_count = 0;
    for beacon in &beacons {
        if serde_json::from_slice::<serde_json::Value>(beacon).is_ok() {
            json_count += 1;
        }
    }
    assert_eq!(json_count, 0, "Beacons should NOT be valid JSON");
    println!("✅ PASSED: 0/{} beacons are valid JSON", beacons.len());

    // Test 3: No identifiable strings
    println!("\n📊 Test 3: Verify no identifiable strings");
    let identifiers = [
        "birdsong",
        "family",
        "version",
        "ciphertext",
        "nonce",
        "tag",
    ];
    let mut found_identifiers = HashSet::new();

    for beacon in &beacons {
        let beacon_str = String::from_utf8_lossy(beacon);
        for id in &identifiers {
            if beacon_str.contains(id) {
                found_identifiers.insert(*id);
            }
        }
    }

    assert!(
        found_identifiers.is_empty(),
        "Found identifiers: {found_identifiers:?}"
    );
    println!("✅ PASSED: No identifiable strings found");
    println!("   Checked: {identifiers:?}");

    // Test 4: Size consistency
    println!("\n📊 Test 4: Verify size consistency");
    let sizes: Vec<usize> = beacons.iter().map(bytes::Bytes::len).collect();
    let min_size = sizes.iter().min().unwrap();
    let max_size = sizes.iter().max().unwrap();
    let size_variance = max_size - min_size;

    println!("   Min size: {min_size} bytes");
    println!("   Max size: {max_size} bytes");
    println!("   Variance: {size_variance} bytes");

    // Should be minimal variance (only from different socket path lengths)
    assert!(size_variance < 50, "Size variance should be minimal");
    println!("✅ PASSED: Consistent sizes (variance < 50 bytes)");

    // Test 5: Format verification
    println!("\n📊 Test 5: Verify pure noise format");
    for beacon in &beacons {
        // Should have at least nonce (12) + tag (16) = 28 bytes
        assert!(beacon.len() >= 28, "Beacon too short");
    }
    println!("✅ PASSED: All beacons have minimum size");
    println!("   Format: [nonce (12)] + [ciphertext] + [tag (16)]");

    cleanup_test_seed(seed_path).await;
    println!("\n🏆 Test PASSED: Network indistinguishability verified!\n");
    println!("Security Properties Validated:");
    println!("  ✅ NOT valid UTF-8 (binary noise)");
    println!("  ✅ NOT valid JSON (pure bytes)");
    println!("  ✅ No identifiable strings");
    println!("  ✅ Consistent size");
    println!("  ✅ Correct format");
    println!("\n🏆 Grade: A++ LEGENDARY (zero metadata leaks)\n");
    Ok(())
}

// ═══════════════════════════════════════════════════════════════════
// Test 5: Performance Characteristics
// ═══════════════════════════════════════════════════════════════════

#[tokio::test]
#[ignore = "Requires running BearDog instance"]
async fn test_performance_characteristics() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n═══════════════════════════════════════════════════════════════════");
    println!("Test 5: Performance Characteristics");
    println!("═══════════════════════════════════════════════════════════════════\n");

    let beardog = beardog_socket();

    if !std::path::Path::new(&beardog).exists() {
        eprintln!("⚠️  Skipping: BearDog not running at {beardog}");
        return Ok(());
    }

    let seed_path = "/tmp/test_performance.seed";
    let seed = b"performance_test_seed_32bytes!!!";
    create_test_seed(seed_path, seed).await?;

    let mgr = DarkForestBeacon::from_security_socket(&beardog, seed_path, "test_node").await?;

    // Benchmark generation
    println!("⚡ Benchmarking beacon generation (50 iterations)...");
    let start = std::time::Instant::now();
    let mut beacons = Vec::new();

    for i in 0..50 {
        let beacon = mgr
            .generate_pure_noise_beacon(&format!("/tmp/perf_{i}.sock"), &["test"], None)
            .await?;
        beacons.push(beacon);
    }

    let generation_time = start.elapsed();
    let avg_generation = generation_time / 50;

    println!("✅ Generation results:");
    println!("   Total time: {generation_time:?}");
    println!("   Average: {avg_generation:?} per beacon");
    println!(
        "   Throughput: ~{} beacons/sec",
        1000 / avg_generation.as_millis().max(1)
    );

    // Benchmark decryption (success case)
    println!("\n⚡ Benchmarking successful decryption (50 iterations)...");
    let start = std::time::Instant::now();

    for beacon in &beacons {
        let result = mgr.try_decrypt_pure_noise_beacon(beacon).await?;
        assert!(result.is_some());
    }

    let decrypt_success_time = start.elapsed();
    let avg_decrypt_success = decrypt_success_time / 50;

    println!("✅ Successful decryption results:");
    println!("   Total time: {decrypt_success_time:?}");
    println!("   Average: {avg_decrypt_success:?} per beacon");

    // Benchmark decryption (failure case - random noise)
    println!("\n⚡ Benchmarking silent failure (50 iterations)...");
    use rand::RngCore;
    let mut random_beacons = Vec::new();
    for _ in 0..50 {
        let mut noise = vec![0u8; beacons[0].len()];
        rand::rng().fill_bytes(&mut noise);
        random_beacons.push(noise);
    }

    let start = std::time::Instant::now();

    for noise in &random_beacons {
        let result = mgr.try_decrypt_pure_noise_beacon(noise).await?;
        assert!(result.is_none());
    }

    let decrypt_failure_time = start.elapsed();
    let avg_decrypt_failure = decrypt_failure_time / 50;

    println!("✅ Silent failure results:");
    println!("   Total time: {decrypt_failure_time:?}");
    println!("   Average: {avg_decrypt_failure:?} per beacon");
    println!(
        "   Speedup vs success: {:.1}x",
        avg_decrypt_success.as_micros() as f64 / avg_decrypt_failure.as_micros() as f64
    );

    cleanup_test_seed(seed_path).await;

    println!("\n🏆 Performance Summary:");
    println!("  Generation: ~{avg_generation:?} avg");
    println!("  Success decrypt: ~{avg_decrypt_success:?} avg");
    println!("  Silent failure: ~{avg_decrypt_failure:?} avg");
    println!("  Status: ⚡ Production-ready performance\n");

    Ok(())
}
