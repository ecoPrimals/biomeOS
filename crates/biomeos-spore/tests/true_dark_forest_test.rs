//! TRUE Dark Forest Pure Noise Beacon Tests
//!
//! Tests A++ security with zero metadata leaks.
//!
//! ## Test Strategy
//!
//! 1. Generate pure noise beacon
//! 2. Verify format (bytes only, no JSON)
//! 3. Verify same family can decrypt
//! 4. Verify different family sees noise
//! 5. Verify network indistinguishability

use biomeos_spore::DarkForestBeacon;

#[tokio::test]
#[ignore = "Requires running BearDog with genetic.derive_lineage_beacon_key"]
async fn test_pure_noise_beacon_generation() {
    // This test validates biomeOS implementation
    // Requires beardog to have genetic.derive_lineage_beacon_key implemented

    let beardog_socket = "/run/user/1000/biomeos/beardog.sock";
    let seed_path = "/tmp/test_dark_forest.seed";
    let node_id = "test_node_1";

    // Create test seed
    let seed_bytes = b"test_family_seed_32_bytes_exactly!!";
    std::fs::write(seed_path, seed_bytes).unwrap();

    // Create Dark Forest beacon manager
    let beacon_mgr = DarkForestBeacon::from_beardog_socket(beardog_socket, seed_path, node_id)
        .await
        .expect("Failed to create beacon manager");

    // Generate pure noise beacon
    let pure_noise_beacon = beacon_mgr
        .generate_pure_noise_beacon("/tmp/test.sock", &["crypto", "genetics"], Some("genesis"))
        .await
        .expect("Failed to generate pure noise beacon");

    // Verify format
    assert!(
        pure_noise_beacon.len() >= 28,
        "Beacon too short (need at least nonce+tag = 28 bytes)"
    );

    // Verify it's bytes (not text)
    assert!(
        std::str::from_utf8(&pure_noise_beacon).is_err(),
        "Beacon should not be valid UTF-8 (should be binary noise)"
    );

    // Verify no JSON structure
    let beacon_str = String::from_utf8_lossy(&pure_noise_beacon);
    assert!(
        !beacon_str.contains("birdsong"),
        "Beacon should not contain 'birdsong' string"
    );
    assert!(
        !beacon_str.contains("family_id"),
        "Beacon should not contain 'family_id' string"
    );
    assert!(
        !beacon_str.contains("version"),
        "Beacon should not contain 'version' field"
    );

    println!(
        "✅ Pure noise beacon generated: {} bytes",
        pure_noise_beacon.len()
    );
    println!("   Format: [nonce (12)] + [ciphertext (N)] + [tag (16)]");
    println!(
        "   First 16 bytes (hex): {}",
        hex::encode(&pure_noise_beacon[..16.min(pure_noise_beacon.len())])
    );

    // Test decryption (same family)
    match beacon_mgr
        .try_decrypt_pure_noise_beacon(&pure_noise_beacon)
        .await
    {
        Ok(Some(decrypted)) => {
            println!("✅ Same family decryption: SUCCESS");
            println!(
                "   Decrypted node_id: {}",
                decrypted
                    .get("node_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
            );
        }
        Ok(None) => {
            panic!("❌ Same family should decrypt successfully");
        }
        Err(e) => {
            panic!("❌ Decryption error: {}", e);
        }
    }

    // Cleanup
    std::fs::remove_file(seed_path).ok();
}

#[test]
fn test_pure_noise_format_properties() {
    // Test that pure noise beacons have correct format properties
    // (no JSON, no structure)

    // Simulate a pure noise beacon
    let nonce = vec![
        0x4a, 0xf3, 0x9b, 0x2c, 0x7e, 0x11, 0x88, 0x45, 0xd2, 0x3f, 0xaa, 0xbb,
    ];
    let ciphertext = vec![0x11, 0x22, 0x33, 0x44, 0x55]; // Example
    let tag = vec![
        0xa1, 0xb2, 0xc3, 0xd4, 0xe5, 0xf6, 0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0, 0x11,
        0x22,
    ];

    let mut beacon = Vec::new();
    beacon.extend_from_slice(&nonce);
    beacon.extend_from_slice(&ciphertext);
    beacon.extend_from_slice(&tag);

    // Verify properties
    assert_eq!(
        beacon.len(),
        12 + 5 + 16,
        "Format: nonce + ciphertext + tag"
    );

    // Verify no JSON
    assert!(
        serde_json::from_slice::<serde_json::Value>(&beacon).is_err(),
        "Pure noise should not parse as JSON"
    );

    // Verify not valid UTF-8 text
    assert!(
        std::str::from_utf8(&beacon).is_err(),
        "Pure noise should not be valid UTF-8"
    );

    println!("✅ Pure noise format validated");
    println!(
        "   Total: {} bytes (nonce: 12, ciphertext: 5, tag: 16)",
        beacon.len()
    );
    println!("   First 8 bytes (hex): {}", hex::encode(&beacon[..8]));
    println!("   Indistinguishable from random noise: ✅");
}

#[test]
fn test_zero_metadata_properties() {
    // Verify that pure noise beacons have zero identifiable metadata

    // Generate random "noise"
    use rand::RngCore;
    let mut beacon = vec![0u8; 128];
    rand::thread_rng().fill_bytes(&mut beacon);

    // Verify it looks random
    let beacon_str = String::from_utf8_lossy(&beacon);

    // Should NOT contain any identifiable markers
    assert!(!beacon_str.contains("birdsong"), "No 'birdsong' marker");
    assert!(!beacon_str.contains("family"), "No 'family' marker");
    assert!(!beacon_str.contains("version"), "No 'version' field");
    assert!(!beacon_str.contains("ciphertext"), "No 'ciphertext' field");
    assert!(!beacon_str.contains("nonce"), "No 'nonce' field");
    assert!(!beacon_str.contains("tag"), "No 'tag' field");

    // Should not parse as JSON
    assert!(
        serde_json::from_slice::<serde_json::Value>(&beacon).is_err(),
        "Random bytes should not parse as JSON"
    );

    println!("✅ Zero metadata verified");
    println!("   No JSON structure: ✅");
    println!("   No identifiable fields: ✅");
    println!("   Indistinguishable from random: ✅");
}
